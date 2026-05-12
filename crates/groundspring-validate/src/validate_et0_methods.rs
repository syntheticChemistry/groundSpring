// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team
#![forbid(unsafe_code)]

//! Validation binary for Exp 035: Multi-Method ET₀ Cross-Validation.
//!
//! Compares five ET₀ methods (Penman-Monteith, Hargreaves, Makkink, Turc,
//! Hamon) against Python baselines. Each method delegates to
//! `barracuda::stats::hydrology` when the `barracuda` feature is enabled,
//! proving pure Rust math matches interpreted Python.
//!
//! Pipeline: Python baseline → Rust (this binary) → barracuda CPU → barracuda GPU.
//!
//! Cross-spring lineage:
//! - Penman-Monteith + Hargreaves: airSpring V035 → barraCuda S71+++
//! - Makkink, Turc, Hamon: airSpring V068/V069 → barraCuda v0.3.2
//! - All methods: groundSpring V78 sovereign fallback + delegation
//!
//! References:
//!   Allen et al. (1998) FAO Irrigation and Drainage Paper 56.
//!   Makkink (1957) Neth J Agr Sci 5:290-305.
//!   Turc (1961) Ann Agron 12:13-49.
//!   Hamon (1963) J Hydraul Div ASCE 89:97-120.
//!   Hargreaves & Samani (1985) Appl Eng Agric 1:96-99.

use groundspring::fao56::{self, DailyWeatherInputs};
use groundspring::validate::ValidationHarness;
use groundspring_validate::{
    ET0_PLAUSIBLE_MAX_MM, ET0_PLAUSIBLE_MIN_MM, TOL_DETERMINISM, TOL_EQUILIBRIUM, TOL_ET0,
    f64_field, parse_benchmark, print_provenance_header,
};
use serde_json::Value;

const BENCHMARK: &str = include_str!("../../../control/et0_methods/benchmark_et0_methods.json");

/// Build the reference-site weather input from the benchmark JSON.
fn build_site_input(site: &Value) -> DailyWeatherInputs {
    DailyWeatherInputs {
        tmax_c: f64_field(site, "tmax_c"),
        tmin_c: f64_field(site, "tmin_c"),
        rhmax_pct: f64_field(site, "rhmax_pct"),
        rhmin_pct: f64_field(site, "rhmin_pct"),
        wind_speed_10m_km_h: f64_field(site, "wind_speed_10m_km_h"),
        sunshine_hours: f64_field(site, "sunshine_hours"),
        latitude_deg_n: f64_field(site, "latitude_deg_n"),
        altitude_m: f64_field(site, "altitude_m"),
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "day_of_year 1–366 fits u16"
        )]
        day_of_year: f64_field(site, "day_of_year") as u16,
    }
}

/// Part 1: Compute all five ET₀ methods and check against Python baselines.
fn validate_reference(
    harness: &mut ValidationHarness,
    bench: &Value,
    base: &DailyWeatherInputs,
    t_mean: f64,
    rh_avg: f64,
    solar_rad: f64,
    daylight: f64,
) -> [f64; 5] {
    println!("\n--- Part 1: Reference Site ET₀ (Uccle, Belgium, 6 July) ---");

    let penman = fao56::daily_et0(base);
    let hargreaves = fao56::hargreaves_et0(
        base.tmax_c,
        base.tmin_c,
        base.latitude_deg_n,
        base.day_of_year,
    );
    let makkink = fao56::makkink_et0(t_mean, solar_rad);
    let turc = fao56::turc_et0(t_mean, solar_rad, rh_avg);
    let hamon = fao56::hamon_et0(t_mean, daylight);

    let ref_et0 = &bench["reference_et0"];
    let py_penman = f64_field(ref_et0, "penman_monteith");
    let py_hargreaves = f64_field(ref_et0, "hargreaves");
    let py_makkink = f64_field(ref_et0, "makkink");
    let py_turc = f64_field(ref_et0, "turc");
    let py_hamon = f64_field(ref_et0, "hamon");

    println!("  PM:  {penman:.6} mm/day (expected {py_penman:.6})");
    println!("  HG:  {hargreaves:.6} mm/day (expected {py_hargreaves:.6})");
    println!("  MK:  {makkink:.6} mm/day (expected {py_makkink:.6})");
    println!("  TU:  {turc:.6} mm/day (expected {py_turc:.6})");
    println!("  HA:  {hamon:.6} mm/day (expected {py_hamon:.6})");

    harness.check_approx("PM ET₀ matches Python", penman, py_penman, TOL_ET0);
    harness.check_approx("HG ET₀ matches Python", hargreaves, py_hargreaves, TOL_ET0);
    harness.check_approx("MK ET₀ matches Python", makkink, py_makkink, TOL_ET0);
    harness.check_approx("TU ET₀ matches Python", turc, py_turc, TOL_ET0);
    harness.check_approx("HA ET₀ matches Python", hamon, py_hamon, TOL_ET0);

    [penman, hargreaves, makkink, turc, hamon]
}

/// Part 2+3: Cross-method agreement + determinism.
fn validate_agreement_and_determinism(
    harness: &mut ValidationHarness,
    base: &DailyWeatherInputs,
    t_mean: f64,
    rh_avg: f64,
    solar_rad: f64,
    daylight: f64,
    results: &[f64; 5],
) {
    println!("\n--- Part 2: Cross-Method Agreement ---");

    harness.check_true("All methods positive", results.iter().all(|&v| v > 0.0));
    harness.check_true(
        "All methods in plausible range",
        results
            .iter()
            .all(|&v| v > ET0_PLAUSIBLE_MIN_MM && v < ET0_PLAUSIBLE_MAX_MM),
    );

    let spread = results.iter().copied().fold(f64::NEG_INFINITY, f64::max)
        - results.iter().copied().fold(f64::INFINITY, f64::min);
    harness.check_true("Cross-method spread < 15 mm/day", spread < 15.0);

    println!("\n--- Part 3: Determinism ---");

    let rerun = [
        fao56::daily_et0(base),
        fao56::hargreaves_et0(
            base.tmax_c,
            base.tmin_c,
            base.latitude_deg_n,
            base.day_of_year,
        ),
        fao56::makkink_et0(t_mean, solar_rad),
        fao56::turc_et0(t_mean, solar_rad, rh_avg),
        fao56::hamon_et0(t_mean, daylight),
    ];
    let labels = ["PM", "HG", "MK", "TU", "HA"];
    for (i, label) in labels.iter().enumerate() {
        harness.check_true(
            &format!("{label} deterministic"),
            (results[i] - rerun[i]).abs() < TOL_DETERMINISM,
        );
    }
}

/// Part 4+5: Seasonal variation + intermediate values.
fn validate_seasonal_and_intermediates(
    harness: &mut ValidationHarness,
    bench: &Value,
    t_mean: f64,
    extra_rad: f64,
    daylight: f64,
    solar_rad: f64,
) {
    println!("\n--- Part 4: Seasonal Variation ---");

    let Some(seasonal) = bench["seasonal"].as_array() else {
        harness.check_true("seasonal array present", false);
        return;
    };
    let seasonal_pm: Vec<f64> = seasonal
        .iter()
        .map(|s| f64_field(s, "penman_monteith"))
        .collect();
    let seasonal_mk: Vec<f64> = seasonal.iter().map(|s| f64_field(s, "makkink")).collect();

    for entry in seasonal {
        let label = entry["label"].as_str().unwrap_or("?");
        println!(
            "  {label:8}: PM={:.4}  MK={:.4}",
            f64_field(entry, "penman_monteith"),
            f64_field(entry, "makkink"),
        );
    }

    harness.check_true("PM: summer > winter", seasonal_pm[2] > seasonal_pm[0]);
    harness.check_true("MK: summer > winter", seasonal_mk[2] > seasonal_mk[0]);

    println!("\n--- Part 5: Intermediate Values ---");

    let inter = &bench["intermediates"];
    harness.check_approx("T_mean", t_mean, f64_field(inter, "tmean"), TOL_EQUILIBRIUM);
    harness.check_approx("Ra", extra_rad, f64_field(inter, "ra"), TOL_EQUILIBRIUM);
    harness.check_approx(
        "N (daylight hours)",
        daylight,
        f64_field(inter, "daylight_hours"),
        TOL_EQUILIBRIUM,
    );
    harness.check_approx("Rs", solar_rad, f64_field(inter, "rs"), TOL_EQUILIBRIUM);
}

fn run() -> i32 {
    let bench = parse_benchmark(BENCHMARK);
    let mut harness =
        ValidationHarness::from_args("Rust Validation: Multi-Method ET₀ Cross-Validation");

    print_provenance_header(&bench, "Multi-Method ET₀ Cross-Validation");

    let base = build_site_input(&bench["site"]);
    let t_mean = f64::midpoint(base.tmax_c, base.tmin_c);
    let rh_avg = f64::midpoint(base.rhmax_pct, base.rhmin_pct);

    let extra_rad = fao56::extraterrestrial_radiation(base.latitude_deg_n, base.day_of_year);
    let daylight = fao56::daylight_hours(base.latitude_deg_n, base.day_of_year);
    let sunshine = base.sunshine_hours.min(daylight).max(0.0);
    let solar_rad = fao56::solar_radiation_from_sunshine(sunshine, daylight, extra_rad);

    let results = validate_reference(
        &mut harness,
        &bench,
        &base,
        t_mean,
        rh_avg,
        solar_rad,
        daylight,
    );
    validate_agreement_and_determinism(
        &mut harness,
        &base,
        t_mean,
        rh_avg,
        solar_rad,
        daylight,
        &results,
    );
    validate_seasonal_and_intermediates(
        &mut harness,
        &bench,
        t_mean,
        extra_rad,
        daylight,
        solar_rad,
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
