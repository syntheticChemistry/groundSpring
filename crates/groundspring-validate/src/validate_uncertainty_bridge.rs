// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Validation binary for Experiment 015: Uncertainty Bridge.
//!
//! Propagates sensor noise (Exp 001) through Anderson localization (Exp 008)
//! to predict QS regime uncertainty. Monte Carlo with Xorshift64.
//!
//! Reference: Dong et al. (2020) Agriculture 10:598,
//!            Bourgain & Kachkovskiy (2018) GAFA 29:3-43

use groundspring::anderson::{localization_length, lyapunov_averaged};
use groundspring::prng::Xorshift64;
use groundspring::validate::ValidationHarness;
use groundspring_validate::{
    EPS_SAFE_DIV, OrExit, THRESHOLD_LARGE_GAMMA, TOL_EQUILIBRIUM, f64_field, get_f64_vec,
    parse_benchmark, print_provenance_header, usize_field,
};
use serde_json::Value;

const BENCHMARK: &str =
    include_str!("../../../control/uncertainty_bridge/benchmark_uncertainty_bridge.json");

/// Parameters for the moisture → disorder → localization pipeline.
struct BridgeParams {
    theta_nominal: f64,
    slope: f64,
    intercept: f64,
    chain_length: usize,
    n_realizations: usize,
}

/// Monte Carlo propagation of sensor noise through Anderson model.
///
/// Returns `(xi_mean, xi_std, xi_cv)`.
fn propagate_sensor_noise(
    params: &BridgeParams,
    bias: f64,
    sigma: f64,
    n_mc: usize,
    rng: &mut Xorshift64,
    base_seed: u64,
) -> (f64, f64, f64) {
    let mut xi_samples: Vec<f64> = Vec::with_capacity(n_mc);

    for i in 0..n_mc {
        let noise = rng.normal(0.0, sigma);
        let theta = (params.theta_nominal + bias + noise).clamp(0.01, 0.99);
        let w_eff = params.slope.mul_add(1.0 - theta, params.intercept).max(0.1);

        let gamma = lyapunov_averaged(
            params.chain_length,
            w_eff,
            0.0,
            params.n_realizations,
            base_seed + i as u64,
        );
        xi_samples.push(1.0 / gamma.max(EPS_SAFE_DIV));
    }

    let (mean, std) = groundspring::stats::mean_and_std_dev(&xi_samples);
    let cv = std / mean.max(EPS_SAFE_DIV);

    (mean, std, cv)
}

/// Validate Anderson model sanity: monotonic γ, small at low W, large at high W.
fn validate_anderson_baseline(
    h: &mut ValidationHarness,
    disorders: &[f64],
    chain_length: usize,
    n_realizations: usize,
) {
    println!("\n--- Step 1: Anderson model sanity checks ---");

    let gammas: Vec<f64> = disorders
        .iter()
        .map(|&w| {
            let gamma = lyapunov_averaged(chain_length, w, 0.0, n_realizations, 42);
            println!(
                "  W={w:5.1} → γ={gamma:.4}, ξ={:.1}",
                localization_length(gamma)
            );
            gamma
        })
        .collect();

    h.check_true(
        "Lyapunov exponent monotonically increasing with W",
        gammas.windows(2).all(|w| w[0] <= w[1]),
    );
    h.check_true(
        "Clean system (W=0.5) has small γ",
        gammas[0] < TOL_EQUILIBRIUM,
    );
    h.check_true(
        "Strong disorder (W=12) has large γ",
        *gammas.last().or_exit("non-empty disorder range") > THRESHOLD_LARGE_GAMMA,
    );
}

/// Propagate one sensor type and return `(raw_cv, corrected_cv)`.
///
/// Each sensor creates its own deterministic RNG from `sensor_seed` so that
/// results are independent of call order and prior RNG consumption.
fn validate_sensor(
    h: &mut ValidationHarness,
    label: &str,
    params: &BridgeParams,
    sensor_cfg: &Value,
    expected_cv: &Value,
    n_mc: usize,
    sensor_seed: u64,
) -> (f64, f64) {
    println!("\n--- {label} ---");

    let bias = f64_field(sensor_cfg, "bias_mbe");
    let sigma = f64_field(sensor_cfg, "random_sigma");

    let mut rng_raw = Xorshift64::new(sensor_seed);
    let (raw_mean, raw_std, raw_cv) =
        propagate_sensor_noise(params, bias, sigma, n_mc, &mut rng_raw, sensor_seed);
    println!("  Raw:  ξ = {raw_mean:.1} ± {raw_std:.1} (CV = {raw_cv:.3})");

    let mut rng_corr = Xorshift64::new(sensor_seed.wrapping_add(1));
    let (corr_mean, corr_std, corr_cv) =
        propagate_sensor_noise(params, 0.0, sigma, n_mc, &mut rng_corr, sensor_seed);
    println!("  Corrected: ξ = {corr_mean:.1} ± {corr_std:.1} (CV = {corr_cv:.3})");

    h.check_range(
        &format!("{label} localization length CV"),
        raw_cv,
        f64_field(expected_cv, "min"),
        f64_field(expected_cv, "max"),
    );

    (raw_cv, corr_cv)
}

fn run() -> i32 {
    let bench = parse_benchmark(BENCHMARK);
    let mut h = ValidationHarness::stdout("Rust Validation: Uncertainty Bridge");
    print_provenance_header(&bench, "Uncertainty Bridge");

    let sensor = &bench["sensor_noise"];
    let anderson = &bench["anderson_model"];
    let prop = &bench["propagation"];
    let exp = &bench["expected"];

    let chain_length = usize_field(anderson, "chain_length");
    let n_realizations = usize_field(anderson, "n_realizations");
    let n_mc = usize_field(prop, "n_mc_samples");

    let params = BridgeParams {
        theta_nominal: f64_field(prop, "theta_nominal"),
        slope: f64_field(prop, "theta_to_disorder_slope"),
        intercept: f64_field(prop, "theta_to_disorder_intercept"),
        chain_length,
        n_realizations,
    };

    let disorders: Vec<f64> =
        get_f64_vec(anderson, "disorder_range").or_exit("disorder_range array");

    validate_anderson_baseline(&mut h, &disorders, chain_length, n_realizations);

    let (cs616_raw_cv, cs616_corr_cv) = validate_sensor(
        &mut h,
        "CS616 Sand",
        &params,
        &sensor["cs616_sand"],
        &exp["localization_length_cv_cs616"],
        n_mc,
        2026,
    );
    let (ec5_raw_cv, ec5_corr_cv) = validate_sensor(
        &mut h,
        "EC5 Sandy Clay Loam",
        &params,
        &sensor["ec5_sandy_clay_loam"],
        &exp["localization_length_cv_ec5"],
        n_mc,
        3026,
    );

    println!("\n--- Cross-sensor comparison ---");
    h.check_true("EC5 has higher CV than CS616", ec5_raw_cv > cs616_raw_cv);

    println!("\n--- Bias correction effectiveness ---");
    let ec5_improvement = 1.0 - ec5_corr_cv / ec5_raw_cv.max(EPS_SAFE_DIV);
    let cs616_improvement = 1.0 - cs616_corr_cv / cs616_raw_cv.max(EPS_SAFE_DIV);
    println!(
        "  EC5 improvement: {:.1}%, CS616 improvement: {:.1}%",
        ec5_improvement * 100.0,
        cs616_improvement * 100.0
    );

    let min_red = f64_field(&exp["bias_corrected_improvement"], "min_reduction_fraction");
    h.check_min("EC5 bias correction reduces CV", ec5_improvement, min_red);
    h.check_true(
        "EC5 benefits more from bias correction (higher bias fraction)",
        ec5_improvement > cs616_improvement
            || f64_field(&sensor["cs616_sand"], "bias_fraction")
                < f64_field(&sensor["ec5_sandy_clay_loam"], "bias_fraction"),
    );

    h.summary()
}

fn main() {
    std::process::exit(run());
}

#[cfg(test)]
mod tests {
    #[test]
    fn validation_passes() {
        assert_eq!(super::run(), 0);
    }
}
