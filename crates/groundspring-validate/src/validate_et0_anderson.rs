// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Validation binary for Experiment 022: ET₀ → Anderson Uncertainty Propagation.
//!
//! Chains FAO-56 ET₀ measurement uncertainty through a water balance and
//! Anderson localization model to quantify how humidity-dominated ET₀ error
//! affects localization length predictions.
//!
//! Reference: Allen et al. (1998) FAO-56; Bourgain & Kachkovskiy (2018) GAFA 29:3-43

use groundspring::anderson::lyapunov_averaged;
use groundspring::fao56::{
    actual_vapour_pressure_rh, atmospheric_pressure, clear_sky_radiation,
    extraterrestrial_radiation, mean_saturation_vapour_pressure, net_longwave_radiation,
    net_shortwave_radiation, penman_monteith, psychrometric_constant, slope_vapour_pressure_curve,
    wind_speed_at_2m,
};
use groundspring::prng::Xorshift64;
use groundspring::validate::ValidationHarness;
use groundspring_validate::{
    EPS_SAFE_DIV, f64_field, f64_range, print_provenance_header, usize_field,
};
use serde_json::Value;

const BENCHMARK: &str =
    include_str!("../../../control/et0_anderson_propagation/benchmark_et0_anderson.json");

/// Compute ET₀ from direct solar radiation input (no sunshine-hours conversion).
#[expect(clippy::too_many_arguments, reason = "mirrors FAO-56 weather inputs")]
fn compute_et0_from_rs(
    tmax: f64,
    tmin: f64,
    rhmax: f64,
    rhmin: f64,
    wind_10m: f64,
    rs: f64,
    lat: f64,
    alt: f64,
    doy: u16,
) -> f64 {
    let tmean = f64::midpoint(tmax, tmin);
    let u2 = wind_speed_at_2m(wind_10m, 10.0);
    let delta = slope_vapour_pressure_curve(tmean);
    let p = atmospheric_pressure(alt);
    let gamma = psychrometric_constant(p);
    let es = mean_saturation_vapour_pressure(tmax, tmin);
    let ea = actual_vapour_pressure_rh(tmax, tmin, rhmax, rhmin);
    let vpd = es - ea;

    let ra = extraterrestrial_radiation(lat, doy);
    let rso = clear_sky_radiation(alt, ra);
    let rns = net_shortwave_radiation(rs);
    let rs_rso = if rso > 0.0 { (rs / rso).min(1.0) } else { 0.7 };
    let rnl = net_longwave_radiation(tmax, tmin, ea, rs_rso);
    let rn = rns - rnl;

    penman_monteith(rn, 0.0, tmean, u2, vpd, delta, gamma)
}

/// Simple daily water balance: θ(t+1) = θ(t) + P/D − ET₀·Kc/D, clamped.
fn water_balance(et0: f64, n_days: usize, cfg: &WaterCfg) -> f64 {
    let mut theta = cfg.theta_init;
    for _ in 0..n_days {
        theta = theta + cfg.precip / cfg.depth - et0 * cfg.kc / cfg.depth;
        theta = theta.clamp(cfg.theta_min, cfg.theta_max);
    }
    theta
}

struct WaterCfg {
    depth: f64,
    kc: f64,
    precip: f64,
    theta_init: f64,
    theta_min: f64,
    theta_max: f64,
}

struct AndersonCfg {
    chain_length: usize,
    n_realizations: usize,
    slope: f64,
    intercept: f64,
}

fn theta_to_disorder(theta: f64, slope: f64, intercept: f64) -> f64 {
    slope.mul_add(1.0 - theta, intercept).max(0.1)
}

struct McResult {
    et0_mean: f64,
    et0_cv: f64,
    theta_mean: f64,
    theta_cv: f64,
    xi_mean: f64,
    xi_cv: f64,
}

fn mc_stats(samples: &[f64]) -> (f64, f64, f64) {
    let (mean, std) = groundspring::stats::mean_and_std_dev(samples);
    let cv = std / mean.max(EPS_SAFE_DIV);
    (mean, std, cv)
}

fn propagate_mc(
    fao: &Value,
    water: &WaterCfg,
    anderson: &AndersonCfg,
    n_mc: usize,
    n_days: usize,
    seed: u64,
) -> McResult {
    let unc = &fao["uncertainties"];
    let t_sig = f64_field(unc, "tmax_sigma");
    let humidity_sigma = f64_field(unc, "rh_sigma");
    let wind_sigma = f64_field(unc, "wind_sigma");
    let radiation_sigma = f64_field(unc, "rs_sigma");
    let lat = f64_field(fao, "latitude_deg");
    let alt = f64_field(fao, "altitude_m");
    #[expect(
        clippy::cast_possible_truncation,
        reason = "day_of_year 1–366 fits u16"
    )]
    let doy = usize_field(fao, "day_of_year") as u16;

    let tmax_base = f64_field(fao, "tmax_c");
    let tmin_base = f64_field(fao, "tmin_c");
    let rhmax_base = f64_field(fao, "rhmax_pct");
    let rhmin_base = f64_field(fao, "rhmin_pct");
    let wind_base = f64_field(fao, "wind_10m_m_s");
    let rs_base = f64_field(fao, "rs_mj_m2_day");

    let mut rng = Xorshift64::new(seed);
    let mut et0_samples = Vec::with_capacity(n_mc);
    let mut theta_samples = Vec::with_capacity(n_mc);
    let mut xi_samples = Vec::with_capacity(n_mc);

    for i in 0..n_mc {
        let tmax = tmax_base + rng.normal(0.0, t_sig);
        let tmin = (tmin_base + rng.normal(0.0, t_sig)).min(tmax - 0.5);
        let rhmax = (rhmax_base + rng.normal(0.0, humidity_sigma)).clamp(10.0, 100.0);
        let rhmin = (rhmin_base + rng.normal(0.0, humidity_sigma)).clamp(5.0, rhmax);
        let wind = (wind_base + rng.normal(0.0, wind_sigma)).max(0.5);
        let rs = (rs_base + rng.normal(0.0, radiation_sigma)).max(0.1);

        let et0 = compute_et0_from_rs(tmax, tmin, rhmax, rhmin, wind, rs, lat, alt, doy).max(0.1);
        et0_samples.push(et0);

        let theta = water_balance(et0, n_days, water);
        theta_samples.push(theta);

        let w_eff = theta_to_disorder(theta, anderson.slope, anderson.intercept);
        let gamma = lyapunov_averaged(
            anderson.chain_length,
            w_eff,
            0.0,
            anderson.n_realizations,
            42 + i as u64,
        );
        xi_samples.push(1.0 / gamma.max(EPS_SAFE_DIV));
    }

    let (et0_mean, _, et0_cv) = mc_stats(&et0_samples);
    let (theta_mean, _, theta_cv) = mc_stats(&theta_samples);
    let (xi_mean, _, xi_cv) = mc_stats(&xi_samples);

    McResult {
        et0_mean,
        et0_cv,
        theta_mean,
        theta_cv,
        xi_mean,
        xi_cv,
    }
}

fn sensitivity_variance_fractions(fao: &Value, seed: u64) -> [f64; 4] {
    let n_per = 200;
    let t_sig = f64_field(&fao["uncertainties"], "tmax_sigma");
    let humidity_sigma = f64_field(&fao["uncertainties"], "rh_sigma");
    let wind_sigma = f64_field(&fao["uncertainties"], "wind_sigma");
    let radiation_sigma = f64_field(&fao["uncertainties"], "rs_sigma");
    let lat = f64_field(fao, "latitude_deg");
    let alt = f64_field(fao, "altitude_m");
    #[expect(
        clippy::cast_possible_truncation,
        reason = "day_of_year 1–366 fits u16"
    )]
    let doy = usize_field(fao, "day_of_year") as u16;
    let tmax_b = f64_field(fao, "tmax_c");
    let tmin_b = f64_field(fao, "tmin_c");
    let rhmax_b = f64_field(fao, "rhmax_pct");
    let rhmin_b = f64_field(fao, "rhmin_pct");
    let wind_b = f64_field(fao, "wind_10m_m_s");
    let rs_b = f64_field(fao, "rs_mj_m2_day");

    let mut rng = Xorshift64::new(seed + 7777);

    let perturb_var = |var_idx: usize, rng: &mut Xorshift64| -> f64 {
        let mut et0s = Vec::with_capacity(n_per);
        for _ in 0..n_per {
            let (tmax, tmin, rhmax, rhmin, wind, rs) = match var_idx {
                0 => {
                    let tx = tmax_b + rng.normal(0.0, t_sig);
                    let tn = (tmin_b + rng.normal(0.0, t_sig)).min(tx - 0.5);
                    (tx, tn, rhmax_b, rhmin_b, wind_b, rs_b)
                }
                1 => {
                    let rxm = (rhmax_b + rng.normal(0.0, humidity_sigma)).clamp(10.0, 100.0);
                    let rxn = (rhmin_b + rng.normal(0.0, humidity_sigma)).clamp(5.0, rxm);
                    (tmax_b, tmin_b, rxm, rxn, wind_b, rs_b)
                }
                2 => {
                    let w = (wind_b + rng.normal(0.0, wind_sigma)).max(0.5);
                    (tmax_b, tmin_b, rhmax_b, rhmin_b, w, rs_b)
                }
                _ => {
                    let r = (rs_b + rng.normal(0.0, radiation_sigma)).max(0.1);
                    (tmax_b, tmin_b, rhmax_b, rhmin_b, wind_b, r)
                }
            };
            et0s.push(compute_et0_from_rs(
                tmax, tmin, rhmax, rhmin, wind, rs, lat, alt, doy,
            ));
        }
        let (_, std, _) = mc_stats(&et0s);
        std * std
    };

    let vars: [f64; 4] = [
        perturb_var(0, &mut rng),
        perturb_var(1, &mut rng),
        perturb_var(2, &mut rng),
        perturb_var(3, &mut rng),
    ];
    let total: f64 = vars.iter().sum();
    if total > 0.0 {
        [
            vars[0] / total,
            vars[1] / total,
            vars[2] / total,
            vars[3] / total,
        ]
    } else {
        [0.25; 4]
    }
}

fn run() -> i32 {
    let bench: Value = serde_json::from_str(BENCHMARK).expect("valid benchmark JSON");
    let mut h = ValidationHarness::stdout("Rust Validation: ET₀ → Anderson Propagation");
    print_provenance_header(&bench, "ET₀ → Anderson Propagation (Exp 022)");

    let fao = &bench["fao56_inputs"];
    let water_cfg = &bench["water_balance"];
    let anderson_cfg = &bench["anderson_model"];
    let prop = &bench["propagation"];
    let exp = &bench["expected"];

    let water = WaterCfg {
        depth: f64_field(water_cfg, "soil_depth_mm"),
        kc: f64_field(water_cfg, "crop_coefficient"),
        precip: f64_field(water_cfg, "daily_precip_mm"),
        theta_init: f64_field(water_cfg, "theta_initial"),
        theta_min: f64_field(water_cfg, "theta_min"),
        theta_max: f64_field(water_cfg, "theta_max"),
    };

    let anderson = AndersonCfg {
        chain_length: usize_field(anderson_cfg, "chain_length"),
        n_realizations: usize_field(anderson_cfg, "n_realizations"),
        slope: f64_field(anderson_cfg, "theta_to_disorder_slope"),
        intercept: f64_field(anderson_cfg, "theta_to_disorder_intercept"),
    };

    let n_mc = usize_field(prop, "n_mc_samples");
    let n_days = usize_field(prop, "n_days");
    let seed = prop["mc_seed"].as_u64().unwrap_or(2026);

    println!("\n--- Step 1: Monte Carlo propagation ---");
    let r = propagate_mc(fao, &water, &anderson, n_mc, n_days, seed);
    println!("  ET₀: mean={:.3} mm/day, CV={:.4}", r.et0_mean, r.et0_cv);
    println!("  θ:   mean={:.3}, CV={:.4}", r.theta_mean, r.theta_cv);
    println!("  ξ:   mean={:.1}, CV={:.4}", r.xi_mean, r.xi_cv);

    let (et0_lo, et0_hi) = f64_range(&exp["et0_mean_range"]);
    h.check_range("ET₀ mean", r.et0_mean, et0_lo, et0_hi);

    let (cv_lo, cv_hi) = f64_range(&exp["et0_cv_range"]);
    h.check_range("ET₀ CV", r.et0_cv, cv_lo, cv_hi);

    let (th_lo, th_hi) = f64_range(&exp["theta_final_range"]);
    h.check_range("θ final mean", r.theta_mean, th_lo, th_hi);

    let (tcv_lo, tcv_hi) = f64_range(&exp["theta_cv_range"]);
    h.check_range("θ CV", r.theta_cv, tcv_lo, tcv_hi);

    let (xi_lo, xi_hi) = f64_range(&exp["xi_cv_range"]);
    h.check_range("ξ CV", r.xi_cv, xi_lo, xi_hi);

    println!("\n--- Step 2: FAO-56 sensitivity ---");
    let fracs = sensitivity_variance_fractions(fao, seed);
    let labels = ["temperature", "humidity", "wind", "radiation"];
    for (l, f) in labels.iter().zip(fracs.iter()) {
        println!("  {l:12}: {:.1}% of ET₀ variance", f * 100.0);
    }
    h.check_true(
        "Humidity dominates ET₀ uncertainty",
        fracs[1] > fracs[0] && fracs[1] > fracs[2] && fracs[1] > fracs[3],
    );

    println!("\n--- Step 3: Uncertainty propagation ---");
    let ratio = r.xi_cv / r.et0_cv.max(EPS_SAFE_DIV);
    let ratio_min = f64_field(exp, "xi_cv_to_et0_cv_ratio_min");
    println!(
        "  ξ CV ({:.4}) / ET₀ CV ({:.4}) = ratio {ratio:.3}",
        r.xi_cv, r.et0_cv
    );
    h.check_min(
        "ET₀ uncertainty propagates through Anderson (ratio ≥ 0.5)",
        ratio,
        ratio_min,
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
