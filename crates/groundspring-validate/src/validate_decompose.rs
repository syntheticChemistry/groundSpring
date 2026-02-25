// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Validation binary for bias-variance decomposition.
//!
//! Expected values loaded at compile time from the benchmark JSON —
//! single source of truth with full provenance (script, commit, date).
//!
//! Source: Dong et al. (2020) Agriculture 10(12), 598.
//! DOI: 10.3390/agriculture10120598

use groundspring::decompose::{decompose_error, noise_floor_reduction};
use groundspring::validate::ValidationHarness;
use serde_json::Value;

const BENCHMARK: &str = include_str!("../../../control/sensor_noise/benchmark_sensor_noise.json");

fn f64_field(v: &Value, key: &str) -> f64 {
    v[key]
        .as_f64()
        .unwrap_or_else(|| panic!("missing f64 field: {key}"))
}

fn main() {
    let bench: Value = serde_json::from_str(BENCHMARK).expect("valid benchmark JSON");
    let mut h = ValidationHarness::stdout("Rust Validation: Bias-Variance Decomposition");

    println!("{}", "=".repeat(72));
    println!("groundSpring Rust Validation: Bias-Variance Decomposition");
    println!(
        "  Source: {}",
        bench["_source"].as_str().unwrap_or("Dong et al. (2020)")
    );
    println!(
        "  Provenance: commit {}, {}",
        bench["_provenance"]["baseline_commit"]
            .as_str()
            .unwrap_or("unknown"),
        bench["_provenance"]["baseline_date"]
            .as_str()
            .unwrap_or("unknown"),
    );
    println!("{}", "=".repeat(72));

    let sensors: Vec<&str> = bench["sensors"]
        .as_array()
        .expect("sensors array")
        .iter()
        .map(|v| v.as_str().expect("sensor string"))
        .collect();

    let soils: Vec<&str> = bench["soil_types"]
        .as_array()
        .expect("soil_types array")
        .iter()
        .map(|v| v.as_str().expect("soil string"))
        .collect();

    // ── Bias-Variance Decomposition ─────────────────────────────────
    //
    // Tol 0.001: random_std = sqrt(RMSE² - MBE²) with 3-decimal inputs
    // introduces ≤ 0.0005 rounding; 0.001 gives ~2× margin.
    // Tol 0.005: bias_fraction = MBE²/RMSE² is a ratio of small numbers;
    // 0.005 absorbs rounding at the 3rd decimal of both inputs.

    println!("\n--- Bias-Variance Decomposition ---");
    for &sensor in &sensors {
        for &soil in &soils {
            let cal = &bench["factory_calibration_stats"][sensor][soil];
            let mbe = f64_field(cal, "mbe");
            let rmse = f64_field(cal, "rmse");

            let expected = &bench["expected_results"][sensor][soil];
            let exp_random_std = f64_field(expected, "random_std");
            let exp_bias_fraction = f64_field(expected, "bias_fraction");

            let d = decompose_error(mbe, rmse);

            h.check_approx(&format!("{sensor} {soil} bias"), d.bias, mbe, 0.001);
            h.check_approx(
                &format!("{sensor} {soil} random_std"),
                d.random_std,
                exp_random_std,
                0.001,
            );
            h.check_approx(
                &format!("{sensor} {soil} bias_fraction"),
                d.bias_fraction,
                exp_bias_fraction,
                0.005,
            );

            let reconstructed = (d.bias_sq + d.variance).sqrt();
            h.check_approx(
                &format!("{sensor} {soil} pythagorean"),
                reconstructed,
                rmse,
                1e-10,
            );
        }
    }

    // ── Noise Floor ─────────────────────────────────────────────────

    println!("\n--- Noise Floor ---");
    for &sensor in &sensors {
        for &soil in &soils {
            let cs = &bench["corrected_stats"][sensor][soil];
            let factory_rmse = f64_field(cs, "factory_rmse");
            let corrected_rmse = f64_field(cs, "corrected_rmse");

            let nf = noise_floor_reduction(factory_rmse, corrected_rmse);
            h.check_true(
                &format!("{sensor} {soil} corrected <= factory"),
                nf.corrected_rmse <= nf.factory_rmse,
            );

            let reconstructed = nf.removed_error.hypot(nf.noise_floor);
            h.check_approx(
                &format!("{sensor} {soil} nf pythagorean"),
                reconstructed,
                factory_rmse,
                1e-10,
            );
        }
    }

    let exit_code = h.summary();
    std::process::exit(exit_code);
}
