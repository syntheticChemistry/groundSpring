// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Validation binary for weather model-observation gap analysis (Exp 002).
//!
//! All checks use analytically constructed data with closed-form expected
//! values — no benchmark JSON or Python baseline is required.  This is
//! intentional: the goal is to verify stat primitives (hit rate, RMSE,
//! MBE, R², IA, decompose) against exact mathematical identities.
//!
//! Provenance: expected values are derivable from the input arrays by
//! inspection (e.g. constant +2 °C bias ⟹ MBE = 2.0, RMSE = 2.0).

use groundspring::decompose::decompose_error;
use groundspring::stats;
use groundspring::validate::ValidationHarness;

fn main() {
    let mut h = ValidationHarness::stdout("Rust Validation: Weather Model-Observation Gap");

    println!("{}", "=".repeat(72));
    println!("groundSpring Rust Validation: Weather Model-Observation Gap");
    println!("  Verifies stats + hit_rate on weather-domain data");
    println!("{}", "=".repeat(72));

    // ── Hit rate analytical cases ───────────────────────────────────
    // Tol 1e-12: all checks are exact integer-ratio results (0.75 = 6/8,
    // 1.0 = 8/8); 1e-12 handles IEEE 754 representation only.
    println!("\n--- Precipitation Hit Rate ---");

    let obs_rain = [0.0, 5.0, 0.0, 3.0, 0.0, 12.0, 0.0, 0.0];
    let mod_rain = [0.0, 4.0, 0.0, 0.0, 0.2, 10.0, 0.0, 0.0];
    // 6/8 days agree on occurrence → hit_rate = 0.75
    h.check_approx(
        "Hit rate known",
        stats::hit_rate(&obs_rain, &mod_rain, 0.1),
        0.75,
        1e-12,
    );

    h.check_approx(
        "Hit rate perfect",
        stats::hit_rate(&obs_rain, &obs_rain, 0.1),
        1.0,
        1e-12,
    );

    let all_zero = [0.0; 4];
    h.check_approx(
        "Hit rate all dry",
        stats::hit_rate(&all_zero, &all_zero, 0.1),
        1.0,
        1e-12,
    );

    // ── Temperature-like paired data (constant bias) ────────────────
    // Tol 1e-10: RMSE/MBE pass through a sum of 365 terms; each
    // f64 add has ≤ 0.5 ULP error, so accumulated error ≤ 365 × ε/2
    // ≈ 4e-14 — 1e-10 provides ~2500× margin.
    println!("\n--- Temperature Stats (constant +2°C bias) ---");

    let obs_temp: Vec<f64> = (0..365)
        .map(|d| {
            let doy = f64::from(d);
            14.5f64.mul_add(
                (2.0 * std::f64::consts::PI * (doy - 100.0) / 365.0).sin(),
                8.5,
            )
        })
        .collect();
    let mod_temp: Vec<f64> = obs_temp.iter().map(|&t| t + 2.0).collect();

    let rmse = stats::rmse(&obs_temp, &mod_temp);
    let mbe = stats::mbe(&obs_temp, &mod_temp);
    let r2 = stats::r_squared(&obs_temp, &mod_temp);
    let ia = stats::index_of_agreement(&obs_temp, &mod_temp);

    h.check_approx("Temp RMSE = 2.0", rmse, 2.0, 1e-10);
    h.check_approx("Temp MBE = +2.0", mbe, 2.0, 1e-10);
    h.check_min("Temp R² > 0.95", r2, 0.95);
    h.check_min("Temp IA > 0.9", ia, 0.9);

    // ── Bias-variance decomposition on weather data ─────────────────
    println!("\n--- Bias-Variance Decomposition ---");

    let d = decompose_error(mbe, rmse);
    h.check_approx(
        "Pure bias: bias_fraction ≈ 1.0",
        d.bias_fraction,
        1.0,
        1e-10,
    );
    h.check_approx("Pure bias: random_std ≈ 0.0", d.random_std, 0.0, 1e-10);

    // ── Random noise case ───────────────────────────────────────────
    println!("\n--- Random Noise Case ---");

    let mod_noisy: Vec<f64> = obs_temp
        .iter()
        .enumerate()
        .map(|(i, &t)| {
            #[expect(clippy::cast_precision_loss)]
            let phase = i as f64 * 0.1;
            phase.sin().mul_add(3.0, t)
        })
        .collect();
    let mbe_noisy = stats::mbe(&obs_temp, &mod_noisy);
    let rmse_noisy = stats::rmse(&obs_temp, &mod_noisy);

    h.check_range("Noisy MBE near zero", mbe_noisy, -0.5, 0.5);
    h.check_min("Noisy RMSE > 0", rmse_noisy, 0.01);

    let d_noisy = decompose_error(mbe_noisy, rmse_noisy);
    h.check_min("Noisy: noise_fraction > 0.5", d_noisy.noise_fraction, 0.5);

    // ── Edge cases ──────────────────────────────────────────────────
    println!("\n--- Edge Cases ---");

    let empty: [f64; 0] = [];
    h.check_approx(
        "Empty hit_rate = 0",
        stats::hit_rate(&empty, &empty, 0.1),
        0.0,
        1e-12,
    );

    let exit_code = h.summary();
    std::process::exit(exit_code);
}
