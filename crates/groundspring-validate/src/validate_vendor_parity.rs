// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Validation binary for Experiment 027: GPU Vendor Parity for WDM Observables.
//!
//! Validates the methodology for verifying GPU vendor parity — that different
//! GPU implementations produce statistically indistinguishable WDM transport
//! coefficient results.
//!
//! References:
//! - hotSpring vendor parity framework (internal)
//! - IEEE 754-2019 floating-point arithmetic standard

use groundspring::decompose::decompose_error;
use groundspring::prng::Xorshift64;
use groundspring::stats::pearson_r;
use groundspring::validate::ValidationHarness;
use groundspring::wdm::{green_kubo_integrate, synthetic_vacf};
use groundspring_validate::{f64_field, print_provenance_header, u64_field, usize_field};
use serde_json::Value;

const BENCHMARK: &str = include_str!("../../../control/vendor_parity/benchmark_vendor_parity.json");

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
            #[expect(clippy::cast_precision_loss)]
            let t = i as f64 * dt;
            let decay = (-t / tau).exp();
            rng.normal(0.0, noise_amplitude).mul_add(decay, v)
        })
        .collect()
}

#[expect(clippy::too_many_lines)]
fn run() -> i32 {
    let bench: Value = serde_json::from_str(BENCHMARK).expect("valid benchmark JSON");
    let mut harness = ValidationHarness::stdout("Rust Validation: GPU Vendor Parity");

    println!("{}", "=".repeat(72));
    println!("groundSpring Rust Validation: GPU Vendor Parity (Exp 027)");
    println!("{}", "=".repeat(72));
    print_provenance_header(&bench, "GPU Vendor Parity for WDM Observables");

    let model = &bench["model"];
    let exp = &bench["expected_results"];

    let c0 = f64_field(model, "c0");
    let dt = f64_field(model, "dt");
    let d_dim = f64_field(model, "d_dim");
    let n_steps = usize_field(model, "n_steps");
    let n_observables = usize_field(model, "n_observables");
    let tau_min = f64_field(model, "tau_min");
    let tau_max = f64_field(model, "tau_max");
    let noise_amplitude = f64_field(model, "noise_amplitude");
    let epsilon = f64_field(model, "epsilon");
    let seed_a = u64_field(model, "seed_a");
    let seed_b = u64_field(model, "seed_b");

    let mut rng_a = Xorshift64::new(seed_a);
    let mut rng_b = Xorshift64::new(seed_b);

    let denom = (n_observables - 1).max(1);

    let mut d_vendor_a: Vec<f64> = Vec::with_capacity(n_observables);
    let mut d_vendor_b: Vec<f64> = Vec::with_capacity(n_observables);

    for i in 0..n_observables {
        #[expect(clippy::cast_precision_loss)]
        let tau = tau_min + (tau_max - tau_min) * (i as f64) / (denom as f64);

        let vacf_a = synthetic_vacf_noisy(c0, tau, n_steps, dt, noise_amplitude, &mut rng_a);
        let integral_a = green_kubo_integrate(&vacf_a, dt);
        let d_a = integral_a / d_dim;

        let vendor_b_vacf: Vec<f64> = vacf_a
            .iter()
            .map(|&v| epsilon.mul_add(rng_b.normal(0.0, 1.0), v))
            .collect();
        let integral_b = green_kubo_integrate(&vendor_b_vacf, dt);
        let d_b = integral_b / d_dim;

        d_vendor_a.push(d_a);
        d_vendor_b.push(d_b);
    }

    // Relative differences: |D_A - D_B| / |D_A|
    let mut max_rel_diff = 0.0_f64;
    let mut sum_rel_diff = 0.0_f64;
    for (da, db) in d_vendor_a.iter().zip(d_vendor_b.iter()) {
        let d_a_safe = da.abs().max(1e-20);
        let rel_diff = (da - db).abs() / d_a_safe;
        max_rel_diff = max_rel_diff.max(rel_diff);
        sum_rel_diff += rel_diff;
    }
    #[expect(clippy::cast_precision_loss)]
    let mean_rel_diff = sum_rel_diff / (n_observables as f64);

    // Pearson correlation between D_A and D_B
    let correlation = pearson_r(&d_vendor_a, &d_vendor_b);

    // Bias-variance decomposition of D_B - D_A
    let diff: Vec<f64> = d_vendor_b
        .iter()
        .zip(d_vendor_a.iter())
        .map(|(db, da)| db - da)
        .collect();
    #[expect(clippy::cast_precision_loss)]
    let mbe = diff.iter().sum::<f64>() / (n_observables as f64);
    #[expect(clippy::cast_precision_loss)]
    let rmse = (diff.iter().map(|d| d * d).sum::<f64>() / (n_observables as f64)).sqrt();
    let decomp = decompose_error(mbe, rmse);
    let bias_fraction = decomp.bias_fraction;

    // Max absolute difference
    let max_abs_diff = diff.iter().map(|d| d.abs()).fold(0.0_f64, f64::max);

    // All within tolerance
    let max_rel_tol = f64_field(exp, "max_relative_difference");
    let all_within = d_vendor_a
        .iter()
        .zip(d_vendor_b.iter())
        .all(|(da, db)| (da - db).abs() / da.abs().max(1e-20) <= max_rel_tol);

    // Chi-squared per DOF: sum((D_A - D_B)^2 / max(D_A^2, 1e-20)) / n_observables
    let chi2_sum: f64 = d_vendor_a
        .iter()
        .zip(d_vendor_b.iter())
        .map(|(da, db)| {
            let denom_chi2 = (da * da).max(1e-20);
            (da - db) * (da - db) / denom_chi2
        })
        .sum();
    #[expect(clippy::cast_precision_loss)]
    let chi2_per_dof = chi2_sum / (n_observables as f64);

    println!("\n  Max relative diff: {max_rel_diff:.2e}, mean: {mean_rel_diff:.2e}");
    println!("  Vendor correlation: {correlation:.8}");
    println!("  Bias fraction: {bias_fraction:.6}, max abs diff: {max_abs_diff:.2e}");
    println!("  Chi² per DOF: {chi2_per_dof:.6}, all within tol: {all_within}");

    // --- Validation checks (7 total) ---
    println!("\n--- Validation Checks ---");

    harness.check_max(
        "Max relative difference bounded",
        max_rel_diff,
        f64_field(exp, "max_relative_difference"),
    );
    harness.check_max(
        "Mean relative difference bounded",
        mean_rel_diff,
        f64_field(exp, "mean_relative_difference_max"),
    );
    harness.check_min(
        "Vendor correlation above minimum",
        correlation,
        f64_field(exp, "vendor_correlation_min"),
    );
    harness.check_max(
        "Bias fraction below maximum",
        bias_fraction,
        f64_field(exp, "bias_fraction_max"),
    );
    harness.check_max(
        "Max absolute difference bounded",
        max_abs_diff,
        f64_field(exp, "max_absolute_difference"),
    );
    harness.check_true("All observables within tolerance", all_within);
    harness.check_max(
        "Chi-squared per DOF bounded",
        chi2_per_dof,
        f64_field(exp, "chi2_per_dof_max"),
    );

    harness.summary()
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
