// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team
#![forbid(unsafe_code)]

//! Validation binary for Experiment 024: Aggregate Stability Measurement Noise.
//!
//! Applies bias-variance decomposition (Exp 001 methodology) to soil aggregate
//! stability measurements, checking whether measurement noise allows
//! distinguishing Anderson localization regimes (`d_eff` = 2 vs `d_eff` = 3).
//!
//! Reference: Nimmo & Perkins (2002), Kemper & Rosenau (1986),
//!            Bourgain & Kachkovskiy (2018) GAFA 29:3-43

use groundspring::decompose::decompose_error;
use groundspring::prng::Xorshift64;
use groundspring::stats;
use groundspring::validate::ValidationHarness;
use groundspring_validate::{
    EPS_SAFE_DIV, f64_field, f64_range, parse_benchmark, print_provenance_header, usize_field,
};

const BENCHMARK: &str =
    include_str!("../../../control/aggregate_stability/benchmark_aggregate_stability.json");

fn simulate_wsa(wsa_true: f64, bias: f64, sigma: f64, n: usize, rng: &mut Xorshift64) -> Vec<f64> {
    (0..n)
        .map(|_| wsa_true + bias + rng.normal(0.0, sigma))
        .collect()
}

fn wsa_to_d_eff(wsa: &[f64], slope: f64, intercept: f64) -> Vec<f64> {
    wsa.iter().map(|&w| slope.mul_add(w, intercept)).collect()
}

fn percentile(data: &mut [f64], p: f64) -> f64 {
    data.sort_by(f64::total_cmp);
    #[expect(
        clippy::cast_precision_loss,
        reason = "percentile index from n_measurements ≪ 2^53"
    )]
    let max_idx = (data.len() - 1) as f64;
    let idx = (p / 100.0 * max_idx).clamp(0.0, max_idx);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "idx is clamped to [0, len-1]"
    )]
    let lo = idx.floor() as usize;
    let hi = (lo + 1).min(data.len() - 1);
    let frac = idx - idx.floor();
    data[lo] + frac * (data[hi] - data[lo])
}

fn regimes_distinguishable(d_tilled: &[f64], d_notill: &[f64], gap_thresh: f64) -> bool {
    let mut t = d_tilled.to_vec();
    let mut n = d_notill.to_vec();
    let t_high = percentile(&mut t, 97.5);
    let n_low = percentile(&mut n, 2.5);
    let t_low = percentile(&mut t, 2.5);
    let n_high = percentile(&mut n, 97.5);
    let overlap = t_high.min(n_high) - t_low.max(n_low);
    overlap < gap_thresh
}

struct StateCtx<'a> {
    wsa_meas: &'a [f64],
    wsa_true: f64,
    d_eff: &'a [f64],
    exp_d_range: (f64, f64),
    exp_cv_range: (f64, f64),
    exp_bf_range: (f64, f64),
    cal_slope: f64,
}

fn validate_state(h: &mut ValidationHarness, label: &str, ctx: &StateCtx<'_>) -> f64 {
    let (wsa_meas, wsa_true, d_eff) = (ctx.wsa_meas, ctx.wsa_true, ctx.d_eff);
    let (exp_d_range, exp_cv_range, exp_bf_range) =
        (ctx.exp_d_range, ctx.exp_cv_range, ctx.exp_bf_range);
    let cal_slope = ctx.cal_slope;
    let (d_mean, d_std) = groundspring::stats::mean_and_std_dev(d_eff);
    let d_cv = d_std / d_mean.max(EPS_SAFE_DIV);

    h.check_range(
        &format!("{label} d_eff mean"),
        d_mean,
        exp_d_range.0,
        exp_d_range.1,
    );
    h.check_range(
        &format!("{label} d_eff CV"),
        d_cv,
        exp_cv_range.0,
        exp_cv_range.1,
    );

    let truth: Vec<f64> = vec![wsa_true; wsa_meas.len()];
    let mbe = stats::mbe(wsa_meas, &truth);
    let rmse = stats::rmse(wsa_meas, &truth);
    let decomp = decompose_error(mbe, rmse);

    println!(
        "  {label}: d_eff mean={d_mean:.3}, CV={d_cv:.4}, bias_frac={:.3}",
        decomp.bias_fraction
    );

    h.check_range(
        &format!("{label} bias fraction"),
        decomp.bias_fraction,
        exp_bf_range.0,
        exp_bf_range.1,
    );

    cal_slope * decomp.random_std
}

fn run() -> i32 {
    let bench = parse_benchmark(BENCHMARK);
    let mut h = ValidationHarness::from_args("Rust Validation: Aggregate Stability");
    print_provenance_header(&bench, "Aggregate Stability Noise (Exp 024)");

    let soil = &bench["soil_states"];
    let noise = &bench["measurement_noise"];
    let cal = &bench["calibration"];
    let exp = &bench["expected"];

    let bias = f64_field(noise, "bias_mbe");
    let sigma = f64_field(noise, "random_sigma");
    let n_meas = usize_field(noise, "n_measurements");
    let seed = noise["seed"].as_u64().unwrap_or(2026);
    let slope = f64_field(cal, "slope");
    let intercept = f64_field(cal, "intercept");

    let mut rng = Xorshift64::new(seed);

    println!("\n--- Part 1: Simulated WSA ---");
    let tilled_wsa_true = f64_field(&soil["tilled"], "wsa_true");
    let notill_wsa_true = f64_field(&soil["notill"], "wsa_true");

    let tilled_wsa = simulate_wsa(tilled_wsa_true, bias, sigma, n_meas, &mut rng);
    let notill_wsa = simulate_wsa(notill_wsa_true, bias, sigma, n_meas, &mut rng);

    let tilled_d = wsa_to_d_eff(&tilled_wsa, slope, intercept);
    let notill_d = wsa_to_d_eff(&notill_wsa, slope, intercept);

    println!("\n--- Part 2: d_eff ranges and bias decomposition ---");
    let noise_floor_tilled = validate_state(
        &mut h,
        "Tilled",
        &StateCtx {
            wsa_meas: &tilled_wsa,
            wsa_true: tilled_wsa_true,
            d_eff: &tilled_d,
            exp_d_range: f64_range(&exp["tilled_d_eff_range"]),
            exp_cv_range: f64_range(&exp["d_eff_cv_range"]),
            exp_bf_range: f64_range(&exp["bias_fraction_range"]),
            cal_slope: slope,
        },
    );
    let noise_floor_notill = validate_state(
        &mut h,
        "No-till",
        &StateCtx {
            wsa_meas: &notill_wsa,
            wsa_true: notill_wsa_true,
            d_eff: &notill_d,
            exp_d_range: f64_range(&exp["notill_d_eff_range"]),
            exp_cv_range: f64_range(&exp["d_eff_cv_range"]),
            exp_bf_range: f64_range(&exp["bias_fraction_range"]),
            cal_slope: slope,
        },
    );

    println!("\n--- Part 3: Regime discrimination ---");
    let distinguishable = regimes_distinguishable(&tilled_d, &notill_d, 0.5);
    println!("  Tilled vs no-till distinguishable: {distinguishable}");
    h.check_true("Regimes distinguishable", distinguishable);

    println!("\n--- Part 4: Noise floor vs regime gap ---");
    let regime_gap = 1.0;
    let floor_ok = noise_floor_tilled < regime_gap && noise_floor_notill < regime_gap;
    println!(
        "  Tilled noise floor: {noise_floor_tilled:.4}, No-till: {noise_floor_notill:.4}, gap: {regime_gap}"
    );
    h.check_true("Noise floor below regime gap", floor_ok);

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
