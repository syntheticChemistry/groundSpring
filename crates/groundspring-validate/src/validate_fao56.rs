// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team
#![forbid(unsafe_code)]

//! Validation binary for FAO-56 error propagation (Exp 003).
//!
//! Reference inputs, Monte Carlo configuration, and expected ranges loaded
//! from the benchmark JSON — single source of truth with full provenance.
//!
//! Reference: Allen et al. (1998) FAO Irrigation and Drainage Paper 56.

use groundspring::fao56::{self, DailyWeatherInputs};
use groundspring::prng::Xorshift64;
use groundspring::validate::ValidationHarness;
use groundspring_validate::{
    OrExit, SANITY_DAYLIGHT_HOURS, SANITY_EA_KPA, SANITY_ES_KPA, SANITY_MC_CV_PCT, SANITY_P_KPA,
    SANITY_U2_MS, SANITY_VARIANCE_SUM, TOL_DETERMINISM, TOL_EQUILIBRIUM, TOL_ET0_BASELINE,
    array_field, f64_field, get_f64_range, parse_benchmark, print_provenance_header, u64_field,
};
use serde_json::Value;

const BENCHMARK: &str =
    include_str!("../../../control/error_propagation/benchmark_error_propagation.json");

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

/// Uncertainty parameters extracted once from the benchmark JSON.
struct Uncertainties {
    sigma_t: f64,
    sigma_rh: f64,
    wind_frac: f64,
    rs_frac: f64,
}

impl Uncertainties {
    fn from_json(v: &Value) -> Self {
        Self {
            sigma_t: f64_field(&v["tmax_c"], "std"),
            sigma_rh: f64_field(&v["rhmax_pct"], "std"),
            wind_frac: f64_field(&v["wind_m_s"], "std_fraction"),
            rs_frac: f64_field(&v["Rs_mj_m2"], "std_fraction"),
        }
    }
}

/// Run Monte Carlo error propagation through FAO-56.
fn monte_carlo_et0(
    base: &DailyWeatherInputs,
    unc: &Uncertainties,
    n_samples: usize,
    seed: u64,
) -> McResult {
    let mut rng = Xorshift64::new(seed);
    let mut samples = Vec::with_capacity(n_samples);

    for _ in 0..n_samples {
        let perturbed = DailyWeatherInputs {
            tmax_c: rng.normal(base.tmax_c, unc.sigma_t),
            tmin_c: rng
                .normal(base.tmin_c, unc.sigma_t)
                .min(base.tmax_c + rng.normal(0.0, unc.sigma_t) - 1.0),
            rhmax_pct: rng.normal(base.rhmax_pct, unc.sigma_rh).clamp(10.0, 100.0),
            rhmin_pct: rng.normal(base.rhmin_pct, unc.sigma_rh).clamp(5.0, 100.0),
            wind_speed_10m_km_h: rng
                .normal(
                    base.wind_speed_10m_km_h,
                    base.wind_speed_10m_km_h * unc.wind_frac,
                )
                .max(0.5),
            sunshine_hours: rng
                .normal(base.sunshine_hours, base.sunshine_hours * unc.rs_frac)
                .max(0.0),
            ..*base
        };
        samples.push(fao56::daily_et0(&perturbed));
    }

    let (mean, std) = groundspring::stats::mean_and_std_dev(&samples);

    samples.sort_by(f64::total_cmp);

    let pct_05 = groundspring::stats::percentile(&samples, 5.0).or_exit("valid percentile");
    let pct_95 = groundspring::stats::percentile(&samples, 95.0).or_exit("valid percentile");

    McResult {
        mean,
        std,
        pct_05,
        pct_95,
    }
}

/// Perturb a single variable group for sensitivity analysis.
///
/// Groups: 0 = temperature, 1 = humidity, 2 = wind, 3 = radiation.
fn perturb_one(
    base: &DailyWeatherInputs,
    group: usize,
    unc: &Uncertainties,
    rng: &mut Xorshift64,
) -> DailyWeatherInputs {
    match group {
        0 => {
            let dt_max = rng.normal(0.0, unc.sigma_t);
            let dt_min = rng.normal(0.0, unc.sigma_t);
            DailyWeatherInputs {
                tmax_c: base.tmax_c + dt_max,
                tmin_c: (base.tmin_c + dt_min).min(base.tmax_c + dt_max - 1.0),
                ..*base
            }
        }
        1 => DailyWeatherInputs {
            rhmax_pct: rng.normal(base.rhmax_pct, unc.sigma_rh).clamp(10.0, 100.0),
            rhmin_pct: rng.normal(base.rhmin_pct, unc.sigma_rh).clamp(5.0, 100.0),
            ..*base
        },
        2 => DailyWeatherInputs {
            wind_speed_10m_km_h: rng
                .normal(
                    base.wind_speed_10m_km_h,
                    base.wind_speed_10m_km_h * unc.wind_frac,
                )
                .max(0.5),
            ..*base
        },
        _ => DailyWeatherInputs {
            sunshine_hours: rng
                .normal(base.sunshine_hours, base.sunshine_hours * unc.rs_frac)
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
    unc: &Uncertainties,
    n_samples: usize,
    seed: u64,
) -> [f64; 4] {
    let mut variances = [0.0_f64; 4];

    for group in 0..4_u64 {
        let mut rng = Xorshift64::new(seed.wrapping_add(group));
        let mut et0_values = Vec::with_capacity(n_samples);
        for _ in 0..n_samples {
            #[expect(clippy::cast_possible_truncation, reason = "group 0..4 fits usize")]
            let perturbed = perturb_one(base, group as usize, unc, &mut rng);
            et0_values.push(fao56::daily_et0(&perturbed));
        }
        let (_, std) = groundspring::stats::mean_and_std_dev(&et0_values);
        #[expect(clippy::cast_possible_truncation, reason = "group 0..4 fits usize")]
        {
            variances[group as usize] = std * std;
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

/// Monte Carlo uncertainty propagation checks.
///
/// Tol: `et0_mean_range` and `et0_std_range` from benchmark JSON represent
/// the physical range of MC outcomes across seeds; CV 1–15% is the
/// documented coefficient of variation for FAO-56 with WMO sensor uncertainty.
fn validate_monte_carlo(
    h: &mut ValidationHarness,
    base: &DailyWeatherInputs,
    unc: &Uncertainties,
    n_mc: usize,
    mc_seed: u64,
    expected_et0: f64,
    mc_expected: &Value,
) {
    let (et0_mean_lo, et0_mean_hi) =
        get_f64_range(&mc_expected["et0_mean_range"]).or_exit("et0_mean_range");
    let (et0_std_lo, et0_std_hi) =
        get_f64_range(&mc_expected["et0_std_range"]).or_exit("et0_std_range");
    println!("\n--- Part 4: Monte Carlo (N={n_mc}) ---");

    let mc = monte_carlo_et0(base, unc, n_mc, mc_seed);

    println!("  ET₀ mean: {:.4} mm/day", mc.mean);
    println!("  ET₀ std:  {:.4} mm/day", mc.std);
    println!("  90% CI:   [{:.4}, {:.4}]", mc.pct_05, mc.pct_95);

    h.check_range("MC ET₀ mean", mc.mean, et0_mean_lo, et0_mean_hi);
    h.check_range("MC ET₀ std", mc.std, et0_std_lo, et0_std_hi);

    let cv = mc.std / mc.mean * 100.0;
    h.check_range("MC CV (%)", cv, SANITY_MC_CV_PCT.0, SANITY_MC_CV_PCT.1);
    h.check_true(
        "90% CI brackets expected",
        mc.pct_05 < expected_et0 && mc.pct_95 > expected_et0,
    );

    let mc2 = monte_carlo_et0(base, unc, n_mc, mc_seed);
    h.check_true(
        "MC is deterministic",
        (mc.mean - mc2.mean).abs() < TOL_DETERMINISM,
    );
}

/// One-at-a-time sensitivity analysis checks.
fn validate_sensitivity(
    h: &mut ValidationHarness,
    base: &DailyWeatherInputs,
    unc: &Uncertainties,
    n_mc: usize,
    mc_seed: u64,
    ranking: &[&str],
) {
    println!("\n--- Part 5: Sensitivity Analysis ---");

    let fracs = sensitivity_analysis(base, unc, n_mc / 2, mc_seed);
    let labels = ["temperature", "humidity", "wind", "radiation"];

    for (label, &frac) in labels.iter().zip(&fracs) {
        println!("  {label:15}: {:.1}% of variance", frac * 100.0);
    }

    let frac_sum: f64 = fracs.iter().sum();
    h.check_range(
        "Variance fractions sum ≈ 1.0",
        frac_sum,
        SANITY_VARIANCE_SUM.0,
        SANITY_VARIANCE_SUM.1,
    );

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

fn run() -> i32 {
    let bench = parse_benchmark(BENCHMARK);
    let mut h = ValidationHarness::from_args("Rust Validation: FAO-56 Error Propagation");

    print_provenance_header(&bench, "FAO-56 Error Propagation");

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
        #[expect(
            clippy::cast_possible_truncation,
            reason = "day_of_year 1–366 fits u16"
        )]
        day_of_year: u64_field(inp, "day_of_year") as u16,
    };
    let expected_et0 = f64_field(ref_day, "expected_et0_mm_day");

    let mc_cfg = &bench["monte_carlo_config"];
    #[expect(
        clippy::cast_possible_truncation,
        reason = "n_samples from JSON ≤ 10000, fits usize"
    )]
    let n_mc = u64_field(mc_cfg, "n_samples") as usize;
    let mc_seed = u64_field(mc_cfg, "seed");

    let unc = Uncertainties::from_json(&bench["input_uncertainties"]);

    let ranking: Vec<&str> = array_field(&bench["sensitivity_analysis"], "expected_ranking")
        .iter()
        .map(|v| v.as_str().or_exit("ranking item"))
        .collect();

    // ── Baseline ET₀ ────────────────────────────────────────────────
    println!("\n--- Part 1: Baseline ET₀ ---");

    let et0 = fao56::daily_et0(&base);
    println!("  Baseline ET₀: {et0:.4} mm/day");
    println!("  Expected:     {expected_et0:.4} mm/day");

    h.check_range(
        "Baseline ET₀",
        et0,
        expected_et0 - TOL_ET0_BASELINE,
        expected_et0 + TOL_ET0_BASELINE,
    );

    // ── Intermediate verification ───────────────────────────────────
    println!("\n--- Part 2: Intermediate Values ---");

    let tmean = f64::midpoint(base.tmax_c, base.tmin_c);
    h.check_approx("T_mean", tmean, 16.9, TOL_EQUILIBRIUM);

    let es = fao56::mean_saturation_vapour_pressure(base.tmax_c, base.tmin_c);
    h.check_range("e_s (kPa)", es, SANITY_ES_KPA.0, SANITY_ES_KPA.1);

    let ea =
        fao56::actual_vapour_pressure_rh(base.tmax_c, base.tmin_c, base.rhmax_pct, base.rhmin_pct);
    h.check_range("e_a (kPa)", ea, SANITY_EA_KPA.0, SANITY_EA_KPA.1);

    let p = fao56::atmospheric_pressure(base.altitude_m);
    h.check_range("P (kPa)", p, SANITY_P_KPA.0, SANITY_P_KPA.1);

    // Provenance: 10.0 m is the WMO standard anemometer height
    // (WMO-No. 8, Guide to Meteorological Instruments, §5.8.1).
    let u2 = fao56::wind_speed_at_2m(base.wind_speed_10m_km_h / 3.6, 10.0);
    h.check_range("u₂ (m/s)", u2, SANITY_U2_MS.0, SANITY_U2_MS.1);

    let n_hours = fao56::daylight_hours(base.latitude_deg_n, base.day_of_year);
    h.check_range(
        "Daylight hours",
        n_hours,
        SANITY_DAYLIGHT_HOURS.0,
        SANITY_DAYLIGHT_HOURS.1,
    );

    // ── Determinism ─────────────────────────────────────────────────
    println!("\n--- Part 3: Determinism ---");

    let et0_b = fao56::daily_et0(&base);
    h.check_true(
        "ET₀ is deterministic",
        (et0 - et0_b).abs() < TOL_DETERMINISM,
    );

    validate_monte_carlo(
        &mut h,
        &base,
        &unc,
        n_mc,
        mc_seed,
        expected_et0,
        &bench["expected_results"],
    );
    validate_sensitivity(&mut h, &base, &unc, n_mc, mc_seed, &ranking);

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
