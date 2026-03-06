// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Alternative reference ET₀ methods (Makkink, Turc, Hamon).
//!
//! Simpler methods requiring fewer inputs than the full FAO-56
//! Penman-Monteith equation. Added for Exp 035 multi-method validation.
//! Each delegates to `barracuda::stats::hydrology` when the `barracuda`
//! feature is enabled (airSpring → barraCuda v0.3.2 provenance).

use super::{
    atmospheric_pressure, psychrometric_constant, saturation_vapour_pressure,
    slope_vapour_pressure_curve,
};

/// Makkink reference ET₀ from temperature and solar radiation (mm day⁻¹).
///
/// `ET₀ = 0.61 · Δ/(Δ+γ) · Rs/λ − 0.12`
///
/// Simpler than Penman-Monteith; needs only temperature and solar radiation.
/// Delegates to `barracuda::stats::hydrology::makkink_et0` when the
/// `barracuda` feature is enabled (airSpring → barraCuda v0.3.2).
#[must_use]
pub fn makkink_et0(t_mean_c: f64, rs_mj: f64) -> f64 {
    #[cfg(feature = "barracuda")]
    {
        if let Some(et0) = barracuda::stats::hydrology::makkink_et0(t_mean_c, rs_mj) {
            return et0;
        }
    }
    makkink_et0_cpu(t_mean_c, rs_mj)
}

fn makkink_et0_cpu(t_mean_c: f64, rs_mj: f64) -> f64 {
    if rs_mj < 0.0 {
        return 0.0;
    }
    let delta = slope_vapour_pressure_curve(t_mean_c);
    let p = atmospheric_pressure(0.0);
    let gamma = psychrometric_constant(p);
    let lambda = 2.45; // latent heat of vaporization (MJ/kg)
    (0.61 * (delta / (delta + gamma)))
        .mul_add(rs_mj / lambda, -0.12)
        .max(0.0)
}

/// Turc reference ET₀ from temperature, solar radiation, and humidity (mm day⁻¹).
///
/// For RH ≥ 50: `ET₀ = 0.013 · T/(T+15) · (23.89·Rs + 50)`
/// For RH < 50: multiply by `1 + (50 − RH)/70`
///
/// Delegates to `barracuda::stats::hydrology::turc_et0` when the
/// `barracuda` feature is enabled (airSpring → barraCuda v0.3.2).
#[must_use]
pub fn turc_et0(t_mean_c: f64, rs_mj: f64, rh_mean_pct: f64) -> f64 {
    #[cfg(feature = "barracuda")]
    {
        if let Some(et0) = barracuda::stats::hydrology::turc_et0(t_mean_c, rs_mj, rh_mean_pct) {
            return et0;
        }
    }
    turc_et0_cpu(t_mean_c, rs_mj, rh_mean_pct)
}

fn turc_et0_cpu(t_mean_c: f64, rs_mj: f64, rh_mean_pct: f64) -> f64 {
    if rs_mj < 0.0 {
        return 0.0;
    }
    let rs_cal = rs_mj * 23.89; // MJ/m²/day → cal/cm²/day
    let base = 0.013 * (t_mean_c / (t_mean_c + 15.0)) * (rs_cal + 50.0);
    let correction = if rh_mean_pct >= 50.0 {
        1.0
    } else {
        1.0 + (50.0 - rh_mean_pct) / 70.0
    };
    (base * correction).max(0.0)
}

/// Hamon reference ET₀ from temperature and daylight hours (mm day⁻¹).
///
/// `ET₀ = 0.55 · (N/12)² · e_s(T)/100`
///
/// Simplest ET₀ method — needs only temperature and daylight hours.
/// Delegates to `barracuda::stats::hydrology::hamon_et0` when the
/// `barracuda` feature is enabled (airSpring → barraCuda v0.3.2).
#[must_use]
pub fn hamon_et0(t_mean_c: f64, daylight_hours_n: f64) -> f64 {
    #[cfg(feature = "barracuda")]
    {
        if let Some(et0) = barracuda::stats::hydrology::hamon_et0(t_mean_c, daylight_hours_n) {
            return et0;
        }
    }
    hamon_et0_cpu(t_mean_c, daylight_hours_n)
}

fn hamon_et0_cpu(t_mean_c: f64, daylight_hours_n: f64) -> f64 {
    if daylight_hours_n < 0.0 {
        return 0.0;
    }
    let es = saturation_vapour_pressure(t_mean_c);
    let es_mbar = es * 10.0; // kPa → mbar
    (0.55 * (daylight_hours_n / 12.0).powi(2) * es_mbar / 100.0).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn makkink_positive_summer() {
        let et0 = makkink_et0(20.0, 25.0);
        assert!(
            et0 > 0.0,
            "Makkink summer ET₀ should be positive, got {et0}"
        );
    }

    #[test]
    fn makkink_zero_for_no_radiation() {
        let et0 = makkink_et0(20.0, 0.0);
        assert!(
            et0 <= 0.05,
            "Makkink with zero radiation should be near zero, got {et0}"
        );
    }

    #[test]
    fn makkink_increases_with_radiation() {
        let low = makkink_et0(20.0, 10.0);
        let high = makkink_et0(20.0, 25.0);
        assert!(
            high > low,
            "more radiation → more ET₀: low={low:.2}, high={high:.2}"
        );
    }

    #[test]
    fn makkink_deterministic() {
        let a = makkink_et0(20.0, 25.0);
        let b = makkink_et0(20.0, 25.0);
        assert!((a - b).abs() < f64::EPSILON);
    }

    #[test]
    fn turc_positive_summer() {
        let et0 = turc_et0(20.0, 25.0, 65.0);
        assert!(et0 > 0.0, "Turc summer ET₀ should be positive, got {et0}");
    }

    #[test]
    fn turc_dry_correction_increases_et0() {
        let humid = turc_et0(20.0, 25.0, 60.0);
        let dry = turc_et0(20.0, 25.0, 30.0);
        assert!(
            dry > humid,
            "dry air correction should increase ET₀: humid={humid:.2}, dry={dry:.2}"
        );
    }

    #[test]
    fn turc_deterministic() {
        let a = turc_et0(20.0, 25.0, 65.0);
        let b = turc_et0(20.0, 25.0, 65.0);
        assert!((a - b).abs() < f64::EPSILON);
    }

    #[test]
    fn hamon_positive_summer() {
        let et0 = hamon_et0(20.0, 14.0);
        assert!(et0 > 0.0, "Hamon summer ET₀ should be positive, got {et0}");
    }

    #[test]
    fn hamon_increases_with_temperature() {
        let cool = hamon_et0(10.0, 12.0);
        let warm = hamon_et0(25.0, 12.0);
        assert!(
            warm > cool,
            "warmer → more ET₀: cool={cool:.2}, warm={warm:.2}"
        );
    }

    #[test]
    fn hamon_increases_with_daylight() {
        let short = hamon_et0(20.0, 8.0);
        let long = hamon_et0(20.0, 16.0);
        assert!(
            long > short,
            "longer days → more ET₀: short={short:.2}, long={long:.2}"
        );
    }

    #[test]
    fn hamon_deterministic() {
        let a = hamon_et0(20.0, 14.0);
        let b = hamon_et0(20.0, 14.0);
        assert!((a - b).abs() < f64::EPSILON);
    }
}
