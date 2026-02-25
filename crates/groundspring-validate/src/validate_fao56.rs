// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Validation binary for FAO-56 error propagation (Exp 003).
//!
//! Reference inputs, Monte Carlo configuration, and expected ranges loaded
//! from the benchmark JSON — single source of truth with full provenance.
//!
//! Reference: Allen et al. (1998) FAO Irrigation and Drainage Paper 56.

use groundspring::fao56::{self, DailyWeatherInputs};
use groundspring::prng::Xorshift64;
use groundspring::validate::ValidationHarness;
use serde_json::Value;

const BENCHMARK: &str =
    include_str!("../../../control/error_propagation/benchmark_error_propagation.json");

fn f64_field(v: &Value, key: &str) -> f64 {
    v[key]
        .as_f64()
        .unwrap_or_else(|| panic!("missing f64 field: {key}"))
}

fn u64_field(v: &Value, key: &str) -> u64 {
    v[key]
        .as_u64()
        .unwrap_or_else(|| panic!("missing u64 field: {key}"))
}

/// Monte Carlo result summary for uncertainty propagation.
struct McResult {
    /// Ensemble mean of ET₀ samples.
    mean: f64,
    /// Population standard deviation of ET₀ samples.
    std: f64,
    /// 5th percentile of ET₀ distribution.
    pct_05: f64,
    /// 95th percentile of ET₀ distribution.
    pct_95: f64,
}

/// Run Monte Carlo error propagation through FAO-56.
fn monte_carlo_et0(
    base: &DailyWeatherInputs,
    uncertainties: &Value,
    n_samples: usize,
    seed: u64,
) -> McResult {
    let sigma_t = f64_field(&uncertainties["tmax_c"], "std");
    let sigma_rh = f64_field(&uncertainties["rhmax_pct"], "std");
    let wind_frac = f64_field(&uncertainties["wind_m_s"], "std_fraction");
    let rs_frac = f64_field(&uncertainties["Rs_mj_m2"], "std_fraction");

    let mut rng = Xorshift64::new(seed);
    let mut samples = Vec::with_capacity(n_samples);

    for _ in 0..n_samples {
        let perturbed = DailyWeatherInputs {
            tmax_c: rng.normal(base.tmax_c, sigma_t),
            tmin_c: rng
                .normal(base.tmin_c, sigma_t)
                .min(base.tmax_c + rng.normal(0.0, sigma_t) - 1.0),
            rhmax_pct: rng.normal(base.rhmax_pct, sigma_rh).clamp(10.0, 100.0),
            rhmin_pct: rng.normal(base.rhmin_pct, sigma_rh).clamp(5.0, 100.0),
            wind_speed_10m_km_h: rng
                .normal(
                    base.wind_speed_10m_km_h,
                    base.wind_speed_10m_km_h * wind_frac,
                )
                .max(0.5),
            sunshine_hours: rng
                .normal(base.sunshine_hours, base.sunshine_hours * rs_frac)
                .max(0.0),
            ..*base
        };
        samples.push(fao56::daily_et0(&perturbed));
    }

    #[expect(clippy::cast_precision_loss)]
    let n = samples.len() as f64;
    let mean = samples.iter().sum::<f64>() / n;
    let variance = samples.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n;

    samples.sort_by(f64::total_cmp);

    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let pct_05 = samples[(0.05 * n) as usize];
    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let pct_95 = samples[(0.95 * n) as usize];

    McResult {
        mean,
        std: variance.sqrt(),
        pct_05,
        pct_95,
    }
}

/// Perturb a single variable group for sensitivity analysis.
fn perturb_one(
    base: &DailyWeatherInputs,
    group: usize,
    uncertainties: &Value,
    rng: &mut Xorshift64,
) -> DailyWeatherInputs {
    let sigma_t = f64_field(&uncertainties["tmax_c"], "std");
    let sigma_rh = f64_field(&uncertainties["rhmax_pct"], "std");
    let wind_frac = f64_field(&uncertainties["wind_m_s"], "std_fraction");
    let rs_frac = f64_field(&uncertainties["Rs_mj_m2"], "std_fraction");

    match group {
        0 => {
            let dt_max = rng.normal(0.0, sigma_t);
            let dt_min = rng.normal(0.0, sigma_t);
            DailyWeatherInputs {
                tmax_c: base.tmax_c + dt_max,
                tmin_c: (base.tmin_c + dt_min).min(base.tmax_c + dt_max - 1.0),
                ..*base
            }
        }
        1 => DailyWeatherInputs {
            rhmax_pct: rng.normal(base.rhmax_pct, sigma_rh).clamp(10.0, 100.0),
            rhmin_pct: rng.normal(base.rhmin_pct, sigma_rh).clamp(5.0, 100.0),
            ..*base
        },
        2 => DailyWeatherInputs {
            wind_speed_10m_km_h: rng
                .normal(
                    base.wind_speed_10m_km_h,
                    base.wind_speed_10m_km_h * wind_frac,
                )
                .max(0.5),
            ..*base
        },
        _ => DailyWeatherInputs {
            sunshine_hours: rng
                .normal(base.sunshine_hours, base.sunshine_hours * rs_frac)
                .max(0.0),
            ..*base
        },
    }
}

/// One-at-a-time sensitivity analysis.
///
/// Returns variance fractions for (temperature, humidity, wind, radiation).
fn sensitivity_analysis(
    base: &DailyWeatherInputs,
    uncertainties: &Value,
    n_samples: usize,
    seed: u64,
) -> [f64; 4] {
    let mut variances = [0.0_f64; 4];

    for group in 0..4_u64 {
        let mut rng = Xorshift64::new(seed.wrapping_add(group));
        let mut et0_values = Vec::with_capacity(n_samples);
        for _ in 0..n_samples {
            #[expect(clippy::cast_possible_truncation)]
            let perturbed = perturb_one(base, group as usize, uncertainties, &mut rng);
            et0_values.push(fao56::daily_et0(&perturbed));
        }
        #[expect(clippy::cast_precision_loss)]
        let n = et0_values.len() as f64;
        let mean = et0_values.iter().sum::<f64>() / n;
        #[expect(clippy::cast_possible_truncation)]
        {
            variances[group as usize] =
                et0_values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n;
        }
    }

    let total: f64 = variances.iter().sum();
    if total > 0.0 {
        for v in &mut variances {
            *v /= total;
        }
    }
    variances
}

fn main() {
    let bench: Value = serde_json::from_str(BENCHMARK).expect("valid benchmark JSON");
    let mut h = ValidationHarness::stdout("Rust Validation: FAO-56 Error Propagation");

    println!("{}", "=".repeat(72));
    println!("groundSpring Rust Validation: FAO-56 Error Propagation");
    println!(
        "  Source: {}",
        bench["_source"].as_str().unwrap_or("FAO-56 Exp 003")
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

    let ref_day = &bench["reference_day"];
    let inp = &ref_day["inputs"];
    let base = DailyWeatherInputs {
        tmax_c: f64_field(inp, "tmax_c"),
        tmin_c: f64_field(inp, "tmin_c"),
        rhmax_pct: f64_field(inp, "rhmax_pct"),
        rhmin_pct: f64_field(inp, "rhmin_pct"),
        wind_speed_10m_km_h: f64_field(inp, "wind_speed_10m_km_h"),
        sunshine_hours: f64_field(inp, "sunshine_hours"),
        latitude_deg_n: f64_field(inp, "latitude_deg_n"),
        altitude_m: f64_field(inp, "altitude_m"),
        #[allow(clippy::cast_possible_truncation)]
        day_of_year: u64_field(inp, "day_of_year") as u16,
    };
    let expected_et0 = f64_field(ref_day, "expected_et0_mm_day");

    let mc_cfg = &bench["monte_carlo_config"];
    #[expect(clippy::cast_possible_truncation)]
    let n_mc = u64_field(mc_cfg, "n_samples") as usize;
    let mc_seed = u64_field(mc_cfg, "seed");

    let uncertainties = &bench["input_uncertainties"];

    let ranking: Vec<&str> = bench["sensitivity_analysis"]["expected_ranking"]
        .as_array()
        .expect("ranking")
        .iter()
        .map(|v| v.as_str().expect("ranking item"))
        .collect();

    // ── Baseline ET₀ ────────────────────────────────────────────────
    println!("\n--- Part 1: Baseline ET₀ ---");

    let et0 = fao56::daily_et0(&base);
    println!("  Baseline ET₀: {et0:.4} mm/day");
    println!("  Expected:     {expected_et0:.4} mm/day");

    h.check_range(
        "Baseline ET₀",
        et0,
        expected_et0 - 0.10,
        expected_et0 + 0.10,
    );

    // ── Intermediate verification ───────────────────────────────────
    // Wide ranges (e.g. e_s 1.8–2.2) are sanity bounds, not precision
    // checks — they verify the equation chain produces physically
    // plausible intermediates, not bit-exact results.
    println!("\n--- Part 2: Intermediate Values ---");

    let tmean = f64::midpoint(base.tmax_c, base.tmin_c);
    h.check_approx("T_mean", tmean, 16.9, 0.1);

    let es = fao56::mean_saturation_vapour_pressure(base.tmax_c, base.tmin_c);
    h.check_range("e_s (kPa)", es, 1.8, 2.2);

    let ea =
        fao56::actual_vapour_pressure_rh(base.tmax_c, base.tmin_c, base.rhmax_pct, base.rhmin_pct);
    h.check_range("e_a (kPa)", ea, 1.2, 1.6);

    let p = fao56::atmospheric_pressure(base.altitude_m);
    h.check_range("P (kPa)", p, 99.0, 102.0);

    let u2 = fao56::wind_speed_at_2m(base.wind_speed_10m_km_h / 3.6, 10.0);
    h.check_range("u₂ (m/s)", u2, 1.5, 2.5);

    let n_hours = fao56::daylight_hours(base.latitude_deg_n, base.day_of_year);
    h.check_range("Daylight hours", n_hours, 15.0, 17.0);

    // ── Determinism ─────────────────────────────────────────────────
    println!("\n--- Part 3: Determinism ---");

    let et0_b = fao56::daily_et0(&base);
    h.check_true("ET₀ is deterministic", (et0 - et0_b).abs() < f64::EPSILON);

    validate_monte_carlo(
        &mut h,
        &base,
        uncertainties,
        n_mc,
        mc_seed,
        expected_et0,
        &bench["expected_results"],
    );
    validate_sensitivity(&mut h, &base, uncertainties, n_mc, mc_seed, &ranking);

    let exit_code = h.summary();
    std::process::exit(exit_code);
}

/// Monte Carlo uncertainty propagation checks.
fn validate_monte_carlo(
    h: &mut ValidationHarness,
    base: &DailyWeatherInputs,
    uncertainties: &Value,
    n_mc: usize,
    mc_seed: u64,
    expected_et0: f64,
    mc_expected: &Value,
) {
    let et0_mean_range = mc_expected["et0_mean_range"].as_array().expect("range");
    let et0_std_range = mc_expected["et0_std_range"].as_array().expect("range");
    println!("\n--- Part 4: Monte Carlo (N={n_mc}) ---");

    let mc = monte_carlo_et0(base, uncertainties, n_mc, mc_seed);

    println!("  ET₀ mean: {:.4} mm/day", mc.mean);
    println!("  ET₀ std:  {:.4} mm/day", mc.std);
    println!("  90% CI:   [{:.4}, {:.4}]", mc.pct_05, mc.pct_95);

    h.check_range(
        "MC ET₀ mean",
        mc.mean,
        et0_mean_range[0].as_f64().unwrap_or(3.5),
        et0_mean_range[1].as_f64().unwrap_or(4.3),
    );
    h.check_range(
        "MC ET₀ std",
        mc.std,
        et0_std_range[0].as_f64().unwrap_or(0.05),
        et0_std_range[1].as_f64().unwrap_or(0.8),
    );

    let cv = mc.std / mc.mean * 100.0;
    h.check_range("MC CV (%)", cv, 1.0, 15.0);
    h.check_true(
        "90% CI brackets expected",
        mc.pct_05 < expected_et0 && mc.pct_95 > expected_et0,
    );

    let mc2 = monte_carlo_et0(base, uncertainties, n_mc, mc_seed);
    h.check_true(
        "MC is deterministic",
        (mc.mean - mc2.mean).abs() < f64::EPSILON,
    );
}

/// One-at-a-time sensitivity analysis checks.
fn validate_sensitivity(
    h: &mut ValidationHarness,
    base: &DailyWeatherInputs,
    uncertainties: &Value,
    n_mc: usize,
    mc_seed: u64,
    ranking: &[&str],
) {
    println!("\n--- Part 5: Sensitivity Analysis ---");

    let fracs = sensitivity_analysis(base, uncertainties, n_mc / 2, mc_seed);
    let labels = ["temperature", "humidity", "wind", "radiation"];

    for (label, &frac) in labels.iter().zip(&fracs) {
        println!("  {label:15}: {:.1}% of variance", frac * 100.0);
    }

    let frac_sum: f64 = fracs.iter().sum();
    h.check_range("Variance fractions sum ≈ 1.0", frac_sum, 0.9, 1.1);

    let max_idx = fracs
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .map_or(0, |(i, _)| i);
    let top_contributor = labels[max_idx];
    h.check_true(
        &format!("Top contributor ({top_contributor}) matches expected ranking"),
        ranking.iter().take(2).any(|&r| r == top_contributor),
    );
}
