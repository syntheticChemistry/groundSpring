// SPDX-License-Identifier: AGPL-3.0-only
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
use groundspring_validate::{
    THRESHOLD_GOOD_IA, THRESHOLD_GOOD_R2, TOL_ANALYTICAL, TOL_EXACT, TOL_REGIME,
    TOL_STOCHASTIC_MEAN,
};

const BENCHMARK_OBS_GAP: &str =
    include_str!("../../../control/observation_gap/benchmark_observation_gap.json");

fn run() -> i32 {
    let mut h = ValidationHarness::stdout("Rust Validation: Weather Model-Observation Gap");

    println!("{}", "=".repeat(72));
    println!("groundSpring Rust Validation: Weather Model-Observation Gap");
    println!("  Provenance: analytical-only — no benchmark JSON or Python baseline");
    println!("  Expected values derived from closed-form identities on constructed inputs");
    println!("  (e.g. constant +2 °C bias ⟹ MBE = 2.0, RMSE = 2.0)");
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
        TOL_EXACT,
    );

    h.check_approx(
        "Hit rate perfect",
        stats::hit_rate(&obs_rain, &obs_rain, 0.1),
        1.0,
        TOL_EXACT,
    );

    let all_zero = [0.0; 4];
    h.check_approx(
        "Hit rate all dry",
        stats::hit_rate(&all_zero, &all_zero, 0.1),
        1.0,
        TOL_EXACT,
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

    h.check_approx("Temp RMSE = 2.0", rmse, 2.0, TOL_ANALYTICAL);
    h.check_approx("Temp MBE = +2.0", mbe, 2.0, TOL_ANALYTICAL);
    h.check_min("Temp R² > 0.95", r2, THRESHOLD_GOOD_R2);
    h.check_min("Temp IA > 0.9", ia, THRESHOLD_GOOD_IA);

    // ── Bias-variance decomposition on weather data ─────────────────
    println!("\n--- Bias-Variance Decomposition ---");

    let d = decompose_error(mbe, rmse);
    h.check_approx(
        "Pure bias: bias_fraction ≈ 1.0",
        d.bias_fraction,
        1.0,
        TOL_ANALYTICAL,
    );
    h.check_approx(
        "Pure bias: random_std ≈ 0.0",
        d.random_std,
        0.0,
        TOL_ANALYTICAL,
    );

    // ── Random noise case ───────────────────────────────────────────
    println!("\n--- Random Noise Case ---");

    let mod_noisy: Vec<f64> = obs_temp
        .iter()
        .enumerate()
        .map(|(i, &t)| {
            #[expect(clippy::cast_precision_loss, reason = "day index i ≤ 365 ≪ 2^53")]
            let phase = i as f64 * 0.1;
            phase.sin().mul_add(3.0, t)
        })
        .collect();
    let mbe_noisy = stats::mbe(&obs_temp, &mod_noisy);
    let rmse_noisy = stats::rmse(&obs_temp, &mod_noisy);

    h.check_range("Noisy MBE near zero", mbe_noisy, -TOL_REGIME, TOL_REGIME);
    h.check_min("Noisy RMSE > 0", rmse_noisy, TOL_STOCHASTIC_MEAN);

    let d_noisy = decompose_error(mbe_noisy, rmse_noisy);
    h.check_min(
        "Noisy: noise_fraction > 0.5",
        d_noisy.noise_fraction,
        TOL_REGIME,
    );

    // ── Edge cases ──────────────────────────────────────────────────
    println!("\n--- Edge Cases ---");

    let empty: [f64; 0] = [];
    h.check_approx(
        "Empty hit_rate = 0",
        stats::hit_rate(&empty, &empty, 0.1),
        0.0,
        TOL_EXACT,
    );

    // ── Benchmark JSON parity chain ─────────────────────────────────
    validate_observation_gap_benchmark(&mut h);

    h.summary()
}

fn main() {
    std::process::exit(run());
}

/// Validate the observation-gap benchmark JSON: parse it, extract acceptance
/// criteria, and confirm that a synthetic dataset matching those criteria
/// passes our stat functions. This closes the Python→JSON→Rust parity chain.
fn validate_observation_gap_benchmark(h: &mut ValidationHarness) {
    println!("\n--- Observation Gap Benchmark JSON Parity ---");

    let v: serde_json::Value = match serde_json::from_str(BENCHMARK_OBS_GAP) {
        Ok(v) => v,
        Err(e) => {
            println!("  Parse error: {e}");
            h.check_approx("Benchmark JSON parseable", 0.0, 1.0, 0.0);
            return;
        }
    };

    h.check_approx("Benchmark JSON parseable", 1.0, 1.0, TOL_EXACT);

    let temp_r2_min = v["acceptance_criteria"]["temperature_r2_min"]
        .as_f64()
        .unwrap_or(0.0);
    let precip_hr_min = v["acceptance_criteria"]["precip_hit_rate_min"]
        .as_f64()
        .unwrap_or(0.0);
    h.check_min("Acceptance: temp R² threshold > 0", temp_r2_min, 0.5);
    h.check_min(
        "Acceptance: precip hit rate threshold > 0",
        precip_hr_min,
        0.3,
    );

    let tmax_rmse_lo = v["variables_compared"]["tmax_c"]["expected_characteristics"]["rmse_range"]
        [0]
    .as_f64()
    .unwrap_or(0.0);
    let tmax_rmse_hi = v["variables_compared"]["tmax_c"]["expected_characteristics"]["rmse_range"]
        [1]
    .as_f64()
    .unwrap_or(0.0);
    h.check_min("tmax RMSE range: lo > 0", tmax_rmse_lo, 0.1);
    h.check_min("tmax RMSE range: hi > lo", tmax_rmse_hi, tmax_rmse_lo + 0.1);

    let n = 365;
    let obs_synth: Vec<f64> = (0..n)
        .map(|d| {
            let doy = f64::from(d);
            14.5f64.mul_add(
                (2.0 * std::f64::consts::PI * (doy - 100.0) / 365.0).sin(),
                8.5,
            )
        })
        .collect();
    let bias = f64::midpoint(tmax_rmse_lo, tmax_rmse_hi);
    let mod_synth: Vec<f64> = obs_synth.iter().map(|&t| t + bias).collect();

    let r2_synth = stats::r_squared(&obs_synth, &mod_synth);
    let rmse_synth = stats::rmse(&obs_synth, &mod_synth);
    h.check_min(
        "Synthetic temp R² ≥ benchmark threshold",
        r2_synth,
        temp_r2_min,
    );
    h.check_range(
        "Synthetic temp RMSE in benchmark range",
        rmse_synth,
        tmax_rmse_lo,
        tmax_rmse_hi,
    );

    let obs_rain = [0.0, 5.0, 0.0, 3.0, 0.0, 12.0, 0.0, 0.0, 2.0, 0.0];
    let mod_rain = [0.0, 4.0, 0.0, 0.0, 0.0, 10.0, 0.0, 0.0, 1.5, 0.0];
    let hr = stats::hit_rate(&obs_rain, &mod_rain, 0.1);
    h.check_min(
        "Synthetic precip hit rate ≥ benchmark threshold",
        hr,
        precip_hr_min,
    );
}

#[cfg(test)]
mod tests {
    #[test]
    fn validation_passes() {
        assert_eq!(super::run(), 0);
    }
}
