// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Validation binary for bias-variance decomposition.
//!
//! Hardcoded expected values from Dong et al. (2020), analytically derived.
//! Provenance: `random_std` = sqrt(RMSE² − MBE²), `bias_fraction` = MBE²/RMSE².
//! Source: Agriculture 10(12), 598. DOI: 10.3390/agriculture10120598

use groundspring::decompose::{decompose_error, noise_floor_reduction};
use groundspring::validate;

/// Sensor-soil configuration for validation.
struct Case {
    sensor: &'static str,
    soil: &'static str,
    mbe: f64,
    rmse: f64,
    expected_random_std: f64,
    expected_bias_fraction: f64,
}

/// Noise floor cases.
struct NoiseFloorCase {
    sensor: &'static str,
    soil: &'static str,
    factory_rmse: f64,
    corrected_rmse: f64,
}

#[allow(clippy::too_many_lines)]
fn main() {
    validate::reset();

    println!("{}", "=".repeat(72));
    println!("groundSpring Rust Validation: Bias-Variance Decomposition");
    println!("  Source: Dong et al. (2020) — analytically derived expected values");
    println!("{}", "=".repeat(72));

    // Hardcoded from benchmark_sensor_noise.json _provenance
    let cases = [
        Case {
            sensor: "CS616",
            soil: "sand",
            mbe: -0.01,
            rmse: 0.017,
            expected_random_std: 0.0137,
            expected_bias_fraction: 0.346,
        },
        Case {
            sensor: "CS616",
            soil: "loamy_sand",
            mbe: -0.03,
            rmse: 0.039,
            expected_random_std: 0.0249,
            expected_bias_fraction: 0.5917,
        },
        Case {
            sensor: "CS616",
            soil: "sandy_clay_loam",
            mbe: -0.02,
            rmse: 0.039,
            expected_random_std: 0.0334,
            expected_bias_fraction: 0.263,
        },
        Case {
            sensor: "EC5",
            soil: "sand",
            mbe: 0.03,
            rmse: 0.038,
            expected_random_std: 0.0233,
            expected_bias_fraction: 0.6233,
        },
        Case {
            sensor: "EC5",
            soil: "loamy_sand",
            mbe: -0.03,
            rmse: 0.035,
            expected_random_std: 0.0180,
            expected_bias_fraction: 0.7347,
        },
        Case {
            sensor: "EC5",
            soil: "sandy_clay_loam",
            mbe: -0.05,
            rmse: 0.057,
            expected_random_std: 0.0274,
            expected_bias_fraction: 0.7695,
        },
    ];

    println!("\n--- Bias-Variance Decomposition ---");
    for c in &cases {
        let d = decompose_error(c.mbe, c.rmse);

        let _ = validate::check_approx(
            &format!("{} {} bias", c.sensor, c.soil),
            d.bias,
            c.mbe,
            0.001,
        );
        let _ = validate::check_approx(
            &format!("{} {} random_std", c.sensor, c.soil),
            d.random_std,
            c.expected_random_std,
            0.001,
        );
        let _ = validate::check_approx(
            &format!("{} {} bias_fraction", c.sensor, c.soil),
            d.bias_fraction,
            c.expected_bias_fraction,
            0.005,
        );

        // Pythagorean identity: RMSE² = MBE² + σ²
        let reconstructed = (d.bias_sq + d.variance).sqrt();
        let _ = validate::check_approx(
            &format!("{} {} pythagorean", c.sensor, c.soil),
            reconstructed,
            c.rmse,
            1e-10,
        );
    }

    let nf_cases = [
        NoiseFloorCase {
            sensor: "CS616",
            soil: "sand",
            factory_rmse: 0.017,
            corrected_rmse: 0.006,
        },
        NoiseFloorCase {
            sensor: "CS616",
            soil: "loamy_sand",
            factory_rmse: 0.023,
            corrected_rmse: 0.021,
        },
        NoiseFloorCase {
            sensor: "CS616",
            soil: "sandy_clay_loam",
            factory_rmse: 0.039,
            corrected_rmse: 0.012,
        },
        NoiseFloorCase {
            sensor: "EC5",
            soil: "sand",
            factory_rmse: 0.018,
            corrected_rmse: 0.004,
        },
        NoiseFloorCase {
            sensor: "EC5",
            soil: "loamy_sand",
            factory_rmse: 0.026,
            corrected_rmse: 0.006,
        },
        NoiseFloorCase {
            sensor: "EC5",
            soil: "sandy_clay_loam",
            factory_rmse: 0.047,
            corrected_rmse: 0.020,
        },
    ];

    println!("\n--- Noise Floor ---");
    for c in &nf_cases {
        let nf = noise_floor_reduction(c.factory_rmse, c.corrected_rmse);
        let _ = validate::check_true(
            &format!("{} {} corrected <= factory", c.sensor, c.soil),
            nf.corrected_rmse <= nf.factory_rmse,
        );

        // Pythagorean: factory² = removed² + corrected²
        let reconstructed = nf.removed_error.hypot(nf.noise_floor);
        let _ = validate::check_approx(
            &format!("{} {} nf pythagorean", c.sensor, c.soil),
            reconstructed,
            c.factory_rmse,
            1e-10,
        );
    }

    let exit_code = validate::summary("Rust Validation: Bias-Variance Decomposition");
    std::process::exit(exit_code);
}
