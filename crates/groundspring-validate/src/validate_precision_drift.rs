// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Validation binary for Experiment 025: f32 vs f64 Transport Coefficient Precision Drift.
//!
//! Validates the analysis methodology for detecting f32→f64 precision drift
//! in WDM Green-Kubo integration of velocity autocorrelation functions.
//!
//! Reference: IEEE 754-2019, Higham (2002) Accuracy and Stability of Numerical Algorithms

use groundspring::decompose::decompose_error;
use groundspring::prng::Xorshift64;
use groundspring::stats;
use groundspring::validate::ValidationHarness;
use groundspring::wdm::{green_kubo_integrate, green_kubo_integrate_f32, synthetic_vacf};
use groundspring_validate::{
    OrExit, f64_field, f64_range, get_f64_vec, parse_benchmark, print_provenance_header,
    usize_field,
};

const BENCHMARK: &str =
    include_str!("../../../control/precision_drift/benchmark_precision_drift.json");

fn synthetic_vacf_noisy(
    c0: f64,
    tau: f64,
    n_steps: usize,
    dt: f64,
    noise_amplitude: f64,
    rng: &mut Xorshift64,
) -> Vec<f64> {
    let base = synthetic_vacf(c0, tau, n_steps, dt);
    base.into_iter()
        .enumerate()
        .map(|(i, v)| {
            #[expect(clippy::cast_precision_loss, reason = "VACF index i ≤ n_steps ≪ 2^53")]
            let t = i as f64 * dt;
            let decay = (-t / tau).exp();
            rng.normal(0.0, noise_amplitude).mul_add(decay, v)
        })
        .collect()
}

#[expect(
    clippy::too_many_lines,
    reason = "validation harness with f32/f64 precision drift checks"
)]
fn run() -> i32 {
    let bench = parse_benchmark(BENCHMARK);
    let mut h = ValidationHarness::stdout("Rust Validation: Precision Drift");
    print_provenance_header(&bench, "f32 vs f64 Precision Drift (Exp 025)");

    let model = &bench["model"];
    let exp = &bench["expected_results"];

    let c0 = f64_field(model, "c0");
    let dt = f64_field(model, "dt");
    let d_dim = f64_field(model, "d_dim");
    let n_steps = usize_field(model, "n_steps");
    let noise_amplitude = f64_field(model, "noise_amplitude");
    let n_realizations = usize_field(model, "n_realizations");
    let seed = model["seed"].as_u64().unwrap_or(42);

    let tau_values: Vec<f64> = get_f64_vec(model, "tau_values").or_exit("tau_values array");

    let mut rng = Xorshift64::new(seed);

    // Noiseless run for f64 vs analytical (check 1)
    #[expect(clippy::cast_precision_loss, reason = "n_steps from JSON ≪ 2^53")]
    let t_max = (n_steps - 1) as f64 * dt;
    let mut f64_noiseless_max_rel_err = 0.0_f64;
    for &tau in &tau_values {
        let vacf_clean = synthetic_vacf(c0, tau, n_steps, dt);
        let integral_f64_clean = green_kubo_integrate(&vacf_clean, dt);
        let analytical_finite = c0 * tau * (1.0 - (-t_max / tau).exp());
        if analytical_finite != 0.0 {
            let rel_err = (integral_f64_clean - analytical_finite).abs() / analytical_finite;
            f64_noiseless_max_rel_err = f64_noiseless_max_rel_err.max(rel_err);
        }
    }

    let mut all_integral_f64: Vec<f64> = Vec::new();
    let mut all_integral_f32: Vec<f64> = Vec::new();
    let mut tau_per_realization: Vec<f64> = Vec::new();

    println!("\n--- Part 1: Green-Kubo integration (f64 vs f32) ---");
    for &tau in &tau_values {
        let mut f64_vals: Vec<f64> = Vec::new();
        let mut rel_errors: Vec<f64> = Vec::new();

        for _ in 0..n_realizations {
            let vacf = synthetic_vacf_noisy(c0, tau, n_steps, dt, noise_amplitude, &mut rng);
            let gk_f64 = green_kubo_integrate(&vacf, dt);
            let gk_f32 = green_kubo_integrate_f32(&vacf, dt);

            all_integral_f64.push(gk_f64);
            all_integral_f32.push(gk_f32);
            tau_per_realization.push(tau);

            f64_vals.push(gk_f64);
            if gk_f64 != 0.0 {
                rel_errors.push((gk_f32 - gk_f64) / gk_f64);
            }
        }

        let (mean_f64, std_f64) = groundspring::stats::mean_and_std_dev(&f64_vals);
        let mean_rel = if rel_errors.is_empty() {
            0.0
        } else {
            groundspring::stats::mean(&rel_errors)
        };
        println!(
            "  tau={tau:.1}: f64 mean={mean_f64:.6}, std={std_f64:.6}, mean_rel_err={mean_rel:.6}"
        );
    }

    println!("\n--- Part 2: Validation Checks ---");

    // Check 1: f64 max relative error vs analytical (noiseless VACF)
    h.check_max(
        "f64 max relative error vs analytical (noiseless)",
        f64_noiseless_max_rel_err,
        f64_field(exp, "f64_analytical_max_error"),
    );

    // Check 2: f32 max relative error vs f64
    let f32_max_rel_err = all_integral_f64
        .iter()
        .zip(all_integral_f32.iter())
        .filter(|(val_f64, _)| **val_f64 != 0.0)
        .map(|(val_f64, val_f32)| ((val_f32 - val_f64) / val_f64).abs())
        .fold(0.0_f64, f64::max);
    h.check_max(
        "f32 max relative error vs f64",
        f32_max_rel_err,
        f64_field(exp, "f32_relative_error_max"),
    );

    // Check 3: Mean relative error (bias) in range
    let errors_f32_minus_f64: Vec<f64> = all_integral_f64
        .iter()
        .zip(all_integral_f32.iter())
        .filter(|(val_f64, _)| **val_f64 != 0.0)
        .map(|(val_f64, val_f32)| (val_f32 - val_f64) / val_f64)
        .collect();
    let mean_rel_err = if errors_f32_minus_f64.is_empty() {
        0.0
    } else {
        groundspring::stats::mean(&errors_f32_minus_f64)
    };
    let (mean_lo, mean_hi) = f64_range(&exp["mean_relative_error_range"]);
    h.check_range(
        "Mean relative error (bias detection) in range",
        mean_rel_err,
        mean_lo,
        mean_hi,
    );

    // Check 4: Bias fraction above minimum
    let raw_errors: Vec<f64> = all_integral_f32
        .iter()
        .zip(all_integral_f64.iter())
        .map(|(a, b)| a - b)
        .collect();
    let mbe = if raw_errors.is_empty() {
        0.0
    } else {
        groundspring::stats::mean(&raw_errors)
    };
    let rmse = if raw_errors.is_empty() {
        0.0
    } else {
        groundspring::stats::rmse(&all_integral_f64, &all_integral_f32)
    };
    let decomp = decompose_error(mbe, rmse);
    h.check_min(
        "Bias fraction above minimum",
        decomp.bias_fraction,
        f64_field(exp, "bias_fraction_min"),
    );

    // Check 5: Max absolute diffusion error (f32 vs f64)
    let max_diff_err = all_integral_f64
        .iter()
        .zip(all_integral_f32.iter())
        .map(|(val_f64, val_f32)| (val_f32 / d_dim - val_f64 / d_dim).abs())
        .fold(0.0_f64, f64::max);
    h.check_max(
        "Max absolute diffusion error (f32 vs f64)",
        max_diff_err,
        f64_field(exp, "max_diffusion_absolute_error"),
    );

    // Check 6: Error-magnitude correlation (|f32-f64| vs expected integral c0*tau)
    let abs_errors: Vec<f64> = all_integral_f64
        .iter()
        .zip(all_integral_f32.iter())
        .filter(|(val_f64, _)| **val_f64 != 0.0)
        .map(|(val_f64, val_f32)| (val_f32 - val_f64).abs())
        .collect();
    let expected_magnitudes: Vec<f64> = all_integral_f64
        .iter()
        .zip(tau_per_realization.iter())
        .filter(|(val_f64, _)| **val_f64 != 0.0)
        .map(|(_, tau)| c0 * tau)
        .collect();
    let corr = if abs_errors.len() >= 2 && abs_errors.len() == expected_magnitudes.len() {
        let r = stats::pearson_r(&abs_errors, &expected_magnitudes);
        if r.is_nan() { 0.0 } else { r }
    } else {
        0.0
    };
    h.check_min(
        "Error-magnitude correlation (larger integrals → larger errors)",
        corr,
        f64_field(exp, "error_magnitude_correlation_min"),
    );

    // Check 7: Relative error std bounded
    let rel_err_std = if errors_f32_minus_f64.len() >= 2 {
        let mean = mean_rel_err;
        let var = errors_f32_minus_f64
            .iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>()
            / {
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "errors len from realizations × tau ≪ 2^53"
                )]
                {
                    errors_f32_minus_f64.len() as f64
                }
            };
        var.sqrt()
    } else {
        0.0
    };
    h.check_max(
        "Relative error std",
        rel_err_std,
        f64_field(exp, "relative_error_std_max"),
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
