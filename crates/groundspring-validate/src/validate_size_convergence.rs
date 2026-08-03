// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team
#![forbid(unsafe_code)]

//! Validation binary for Experiment 026: System-size convergence for WDM transport.
//!
//! Validates the analysis methodology for finite-size extrapolation of WDM
//! transport coefficients. Uses synthetic D(N) = D∞ + α/N^(1/d) data with
//! noise to test linear regression extrapolation and convergence detection.
//!
//! Reference: Yeh & Hummer (2004) J. Phys. Chem. B 108, 15873

use groundspring::prng::Xorshift64;
use groundspring::validate::ValidationHarness;
use groundspring::wdm::finite_size_extrapolate;
use groundspring_validate::{
    OrExit, f64_field, f64_range, get_array, parse_benchmark, print_provenance_header, usize_field,
};
use serde_json::Value;

const BENCHMARK: &str =
    include_str!("../../../control/size_convergence/benchmark_size_convergence.json");

#[expect(
    clippy::cast_precision_loss,
    reason = "system sizes from JSON are < 2^53, conversion exact"
)]
fn json_number_to_f64(v: &Value) -> f64 {
    v.as_f64()
        .or_else(|| v.as_u64().map(|u| u as f64))
        .or_else(|| v.as_i64().map(|i| i as f64))
        .or_exit("JSON value must be numeric")
}

struct FitResults {
    d_inf_fit: f64,
    alpha_fit: f64,
    r_squared: f64,
    extrapolation_rel_err: f64,
    convergence_n: Option<f64>,
    mean_at_largest_n: f64,
    residual_std: f64,
}

fn compute_fit(
    system_sizes: &[f64],
    d_mean: &[f64],
    d_dim: f64,
    d_inf_true: f64,
    threshold: f64,
) -> FitResults {
    let exponent = 1.0 / d_dim;
    let (d_inf_fit, alpha_fit, r_squared) =
        finite_size_extrapolate(system_sizes, d_mean, d_dim).or_exit("sizes >= 2");

    let extrapolation_rel_err = if d_inf_true == 0.0 {
        0.0
    } else {
        (d_inf_fit - d_inf_true).abs() / d_inf_true
    };

    let mut convergence_n: Option<f64> = None;
    for (idx, &n) in system_sizes.iter().enumerate() {
        let rel_err = if d_inf_fit == 0.0 {
            f64::INFINITY
        } else {
            (d_mean[idx] - d_inf_fit).abs() / d_inf_fit
        };
        if rel_err < threshold {
            convergence_n = Some(n);
            break;
        }
    }

    let mean_at_largest_n = d_mean[d_mean.len() - 1];

    let fitted: Vec<f64> = system_sizes
        .iter()
        .map(|&n| d_inf_fit + alpha_fit / n.powf(exponent))
        .collect();
    let residuals: Vec<f64> = d_mean
        .iter()
        .zip(fitted.iter())
        .map(|(a, b)| a - b)
        .collect();
    #[expect(
        clippy::cast_precision_loss,
        reason = "residuals len = system_sizes len ≪ 2^53"
    )]
    let n_pts = residuals.len() as f64;
    let res_mean = residuals.iter().sum::<f64>() / n_pts;
    let residual_var = residuals
        .iter()
        .map(|r| (r - res_mean).powi(2))
        .sum::<f64>()
        / n_pts;

    FitResults {
        d_inf_fit,
        alpha_fit,
        r_squared,
        extrapolation_rel_err,
        convergence_n,
        mean_at_largest_n,
        residual_std: residual_var.sqrt(),
    }
}

fn run() -> i32 {
    let bench = parse_benchmark(BENCHMARK);
    let mut h = ValidationHarness::from_args("Rust Validation: Size Convergence");
    print_provenance_header(
        &bench,
        "System-size Convergence for WDM Transport (Exp 026)",
    );

    let model = &bench["model"];
    let exp = &bench["expected_results"];

    let d_inf_true = f64_field(model, "d_inf_true");
    let alpha_true = f64_field(model, "alpha_true");
    let d_dim = f64_field(model, "d_dim");
    let n_replicas = usize_field(model, "n_replicas");
    let noise_std = f64_field(model, "noise_std");
    let seed = model["seed"].as_u64().unwrap_or(42);
    let threshold_pct = f64_field(model, "convergence_threshold_pct");
    let threshold = threshold_pct / 100.0;

    let system_sizes: Vec<f64> = get_array(model, "system_sizes")
        .or_exit("system_sizes array")
        .iter()
        .map(json_number_to_f64)
        .collect();

    let mut rng = Xorshift64::new(seed);

    let exponent = 1.0 / d_dim;
    let mut d_values: Vec<Vec<f64>> = Vec::with_capacity(system_sizes.len());
    for &n in &system_sizes {
        let signal = d_inf_true + alpha_true / n.powf(exponent);
        let mut row: Vec<f64> = Vec::with_capacity(n_replicas);
        for _ in 0..n_replicas {
            let noise = rng.normal(0.0, noise_std);
            row.push(signal + noise);
        }
        d_values.push(row);
    }

    let d_mean: Vec<f64> = d_values
        .iter()
        .map(|row| {
            #[expect(clippy::cast_precision_loss, reason = "n_replicas from JSON ≪ 2^53")]
            let n_rep = row.len() as f64;
            row.iter().sum::<f64>() / n_rep
        })
        .collect();

    let f = compute_fit(&system_sizes, &d_mean, d_dim, d_inf_true, threshold);

    println!(
        "\n  D∞ (fitted): {:.6}, α (fitted): {:.6}, R²: {:.6}",
        f.d_inf_fit, f.alpha_fit, f.r_squared
    );
    println!("  Convergence at N: {:?}", f.convergence_n);
    println!(
        "  Mean D at largest N: {:.6}, residual std: {:.6}",
        f.mean_at_largest_n, f.residual_std
    );

    println!("\n--- Validation Checks ---");

    let d_inf_tol = f64_field(exp, "d_inf_tolerance");
    h.check_range(
        "Extrapolated D∞ within tolerance of true",
        f.d_inf_fit,
        d_inf_true - d_inf_tol,
        d_inf_true + d_inf_tol,
    );

    let (alpha_lo, alpha_hi) = f64_range(&exp["alpha_range"]);
    h.check_range(
        "Fitted α in expected range",
        f.alpha_fit,
        alpha_lo,
        alpha_hi,
    );

    h.check_min(
        "R² above minimum",
        f.r_squared,
        f64_field(exp, "r_squared_min"),
    );
    h.check_max(
        "Extrapolation relative error bounded",
        f.extrapolation_rel_err,
        f64_field(exp, "extrapolation_relative_error_max"),
    );

    let convergence_n_max = f64_field(exp, "convergence_n_max");
    h.check_true(
        "Convergence achieved by N_max",
        f.convergence_n
            .is_some_and(|n_conv| n_conv <= convergence_n_max),
    );

    let (mean_lo, mean_hi) = f64_range(&exp["mean_at_largest_n_range"]);
    h.check_range(
        "Mean D at largest N in expected range",
        f.mean_at_largest_n,
        mean_lo,
        mean_hi,
    );
    h.check_max(
        "Residual std bounded",
        f.residual_std,
        f64_field(exp, "residual_std_max"),
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
