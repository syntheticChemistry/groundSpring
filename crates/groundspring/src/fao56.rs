// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! FAO-56 Penman-Monteith reference evapotranspiration.
//!
//! Pure-Rust port of the equation chain from Allen et al. (1998)
//! "Crop evapotranspiration — Guidelines for computing crop water
//! requirements", FAO Irrigation and Drainage Paper 56.
//!
//! Each function cites the exact FAO-56 equation number.  Intermediate
//! values can be checked against the worked examples in Chapter 4.
//!
//! # barracuda delegation
//!
//! [`daily_et0`] is a pending delegation target for
//! `barracuda::stats::hydrology::fao56_et0()` — scalar form not yet in
//! barracuda as of S68+. `ToadStool` has `hargreaves_et0` (temperature-only),
//! `crop_coefficient`, `soil_water_balance`, and GPU batch
//! `BatchedElementwiseF64::fao56_et0_batch` but no standalone scalar
//! Penman-Monteith. Sub-functions remain local as the validation reference.

use std::f64::consts::PI;

// ── Physical constants ──────────────────────────────────────────────

/// Solar constant (MJ m⁻² min⁻¹).  FAO-56 p. 47.
const GSC: f64 = 0.0820;

/// Stefan-Boltzmann constant (MJ m⁻² day⁻¹ K⁻⁴).  FAO-56 Eq. 39.
const SIGMA: f64 = 4.903e-9;

/// Default grass albedo.  FAO-56 Eq. 38.
const ALBEDO: f64 = 0.23;

// ── Sub-functions ───────────────────────────────────────────────────

/// Saturation vapour pressure at temperature `t_c` (kPa).
///
/// FAO-56 Eq. 11: e°(T) = 0.6108 exp(17.27 T / (T + 237.3))
#[must_use]
pub fn saturation_vapour_pressure(t_c: f64) -> f64 {
    0.6108 * (17.27 * t_c / (t_c + 237.3)).exp()
}

/// Slope of the saturation vapour pressure curve (kPa °C⁻¹).
///
/// FAO-56 Eq. 13.
#[must_use]
pub fn slope_vapour_pressure_curve(t_c: f64) -> f64 {
    let es = saturation_vapour_pressure(t_c);
    4098.0 * es / (t_c + 237.3).powi(2)
}

/// Atmospheric pressure from elevation (kPa).
///
/// FAO-56 Eq. 7: P = 101.3 ((293 − 0.0065 z) / 293)^5.26
#[must_use]
pub fn atmospheric_pressure(altitude_m: f64) -> f64 {
    101.3 * (0.0065f64.mul_add(-altitude_m, 293.0) / 293.0).powf(5.26)
}

/// Psychrometric constant (kPa °C⁻¹).
///
/// FAO-56 Eq. 8: γ = 0.000665 P
#[must_use]
pub fn psychrometric_constant(pressure_kpa: f64) -> f64 {
    0.000_665 * pressure_kpa
}

/// Wind speed at 2 m from measurement at height `z` (m/s).
///
/// FAO-56 Eq. 47: u₂ = `u_z` · 4.87 / ln(67.8 z − 5.42)
#[must_use]
pub fn wind_speed_at_2m(uz: f64, z: f64) -> f64 {
    uz * 4.87 / (67.8_f64.mul_add(z, -5.42)).ln()
}

/// Mean saturation vapour pressure (kPa).
///
/// FAO-56 Eq. 12: `e_s` = (`e°(T_max)` + `e°(T_min)`) / 2
#[must_use]
pub fn mean_saturation_vapour_pressure(tmax_c: f64, tmin_c: f64) -> f64 {
    f64::midpoint(
        saturation_vapour_pressure(tmax_c),
        saturation_vapour_pressure(tmin_c),
    )
}

/// Actual vapour pressure from relative humidity (kPa).
///
/// FAO-56 Eq. 17.
#[must_use]
pub fn actual_vapour_pressure_rh(tmax_c: f64, tmin_c: f64, rhmax_pct: f64, rhmin_pct: f64) -> f64 {
    let e_tmin = saturation_vapour_pressure(tmin_c);
    let e_tmax = saturation_vapour_pressure(tmax_c);
    e_tmin.mul_add(rhmax_pct / 100.0, e_tmax * (rhmin_pct / 100.0)) / 2.0
}

/// Solar declination (radians).
///
/// FAO-56 Eq. 24: δ = 0.409 sin(2π J/365 − 1.39)
#[must_use]
pub fn solar_declination(day_of_year: u16) -> f64 {
    0.409
        * (2.0 * PI / 365.0)
            .mul_add(f64::from(day_of_year), -1.39)
            .sin()
}

/// Inverse relative earth-sun distance factor.
///
/// FAO-56 Eq. 23: `d_r` = 1 + 0.033 cos(2π J/365)
#[must_use]
pub fn inverse_relative_distance(day_of_year: u16) -> f64 {
    0.033f64.mul_add((2.0 * PI / 365.0 * f64::from(day_of_year)).cos(), 1.0)
}

/// Sunset hour angle (radians).
///
/// FAO-56 Eq. 25: `ω_s` = arccos(−tan φ · tan δ)
#[must_use]
pub fn sunset_hour_angle(latitude_rad: f64, declination_rad: f64) -> f64 {
    let arg = -latitude_rad.tan() * declination_rad.tan();
    arg.clamp(-1.0, 1.0).acos()
}

/// Extraterrestrial radiation (MJ m⁻² day⁻¹).
///
/// FAO-56 Eq. 21.
#[must_use]
pub fn extraterrestrial_radiation(latitude_deg: f64, day_of_year: u16) -> f64 {
    let phi = latitude_deg.to_radians();
    let dr = inverse_relative_distance(day_of_year);
    let delta = solar_declination(day_of_year);
    let ws = sunset_hour_angle(phi, delta);

    (24.0 * 60.0 / PI)
        * GSC
        * dr
        * ws.mul_add(phi.sin() * delta.sin(), phi.cos() * delta.cos() * ws.sin())
}

/// Maximum possible daylight hours.
///
/// FAO-56 Eq. 34: N = 24/π · `ω_s`
#[must_use]
pub fn daylight_hours(latitude_deg: f64, day_of_year: u16) -> f64 {
    let phi = latitude_deg.to_radians();
    let delta = solar_declination(day_of_year);
    let ws = sunset_hour_angle(phi, delta);
    24.0 / PI * ws
}

/// Solar radiation from sunshine duration (MJ m⁻² day⁻¹).
///
/// FAO-56 Eq. 35 (Ångström): `R_s` = (`a_s` + `b_s` n/N) `R_a`
#[must_use]
pub fn solar_radiation_from_sunshine(n: f64, big_n: f64, ra: f64) -> f64 {
    (0.25 + 0.50 * n / big_n) * ra
}

/// Clear-sky solar radiation (MJ m⁻² day⁻¹).
///
/// FAO-56 Eq. 37: `R_so` = (0.75 + 2×10⁻⁵ z) `R_a`
#[must_use]
pub fn clear_sky_radiation(altitude_m: f64, ra: f64) -> f64 {
    altitude_m.mul_add(2e-5, 0.75) * ra
}

/// Net shortwave radiation (MJ m⁻² day⁻¹).
///
/// FAO-56 Eq. 38: `R_ns` = (1 − α) `R_s`
#[must_use]
pub fn net_shortwave_radiation(rs: f64) -> f64 {
    (1.0 - ALBEDO) * rs
}

/// Net longwave radiation (MJ m⁻² day⁻¹).
///
/// FAO-56 Eq. 39.
#[must_use]
pub fn net_longwave_radiation(tmax_c: f64, tmin_c: f64, ea_kpa: f64, rs_over_rso: f64) -> f64 {
    let tmax_k4 = (tmax_c + 273.16_f64).powi(4);
    let tmin_k4 = (tmin_c + 273.16_f64).powi(4);
    let avg_k4 = f64::midpoint(tmax_k4, tmin_k4);
    let humidity_factor = 0.14_f64.mul_add(-ea_kpa.sqrt(), 0.34);
    let cloudiness_factor = 1.35_f64.mul_add(rs_over_rso, -0.35);
    SIGMA * avg_k4 * humidity_factor * cloudiness_factor
}

/// FAO-56 Penman-Monteith reference ET₀ (mm day⁻¹).
///
/// FAO-56 Eq. 6.
#[must_use]
pub fn penman_monteith(
    rn: f64,
    g: f64,
    tmean_c: f64,
    u2: f64,
    vpd_kpa: f64,
    delta: f64,
    gamma: f64,
) -> f64 {
    let numerator =
        (0.408 * delta).mul_add(rn - g, gamma * (900.0 / (tmean_c + 273.0)) * u2 * vpd_kpa);
    let denominator = gamma.mul_add(0.34_f64.mul_add(u2, 1.0), delta);
    numerator / denominator
}

// ── High-level wrappers ─────────────────────────────────────────────

/// Inputs for a daily ET₀ computation (FAO-56 Example 18 pattern).
#[derive(Debug, Clone, Copy)]
pub struct DailyWeatherInputs {
    /// Maximum air temperature (°C).
    pub tmax_c: f64,
    /// Minimum air temperature (°C).
    pub tmin_c: f64,
    /// Maximum relative humidity (%).
    pub rhmax_pct: f64,
    /// Minimum relative humidity (%).
    pub rhmin_pct: f64,
    /// Wind speed at 10 m height (km h⁻¹).
    pub wind_speed_10m_km_h: f64,
    /// Actual sunshine duration (hours).
    pub sunshine_hours: f64,
    /// Site latitude (°N, negative for southern hemisphere).
    pub latitude_deg_n: f64,
    /// Site elevation above sea level (m).
    pub altitude_m: f64,
    /// Day of year (1–366).
    pub day_of_year: u16,
}

/// Compute daily reference ET₀ from weather observations.
///
/// Implements the full FAO-56 Eq. 6 chain with RH data and wind
/// height conversion (Example 18 pattern).
///
/// Pending delegation to `barracuda::stats::hydrology::fao56_et0` — not
/// yet in barracuda as of S68+ (scalar). `ToadStool` has `hargreaves_et0`
/// (temperature-only), `crop_coefficient`, `soil_water_balance`, and
/// `BatchedElementwiseF64::fao56_et0_batch` (GPU batch) but no standalone
/// scalar Penman-Monteith.
#[must_use]
pub fn daily_et0(inp: &DailyWeatherInputs) -> f64 {
    // TODO(toadstool): wire when barracuda adds stats::hydrology::fao56_et0 (scalar)
    // Status S68+: hargreaves_et0 available; fao56_et0_batch (GPU) available;
    // scalar fao56_et0 not yet absorbed.
    // #[cfg(feature = "barracuda")]
    // {
    //     if let Ok(et0) = barracuda::stats::hydrology::fao56_et0(
    //         inp.tmax_c,
    //         inp.tmin_c,
    //         inp.rhmax_pct,
    //         inp.rhmin_pct,
    //         inp.wind_speed_10m_km_h,
    //         inp.sunshine_hours,
    //         inp.altitude_m,
    //         inp.latitude_deg_n,
    //         inp.day_of_year,
    //     ) {
    //         return et0;
    //     }
    // }
    daily_et0_cpu(inp)
}

fn daily_et0_cpu(inp: &DailyWeatherInputs) -> f64 {
    let tmean = f64::midpoint(inp.tmax_c, inp.tmin_c);
    let uz_ms = inp.wind_speed_10m_km_h / 3.6;
    let u2 = wind_speed_at_2m(uz_ms, 10.0);

    let delta = slope_vapour_pressure_curve(tmean);
    let p = atmospheric_pressure(inp.altitude_m);
    let gamma = psychrometric_constant(p);
    let es = mean_saturation_vapour_pressure(inp.tmax_c, inp.tmin_c);
    let ea = actual_vapour_pressure_rh(inp.tmax_c, inp.tmin_c, inp.rhmax_pct, inp.rhmin_pct);
    let vpd = es - ea;

    let ra = extraterrestrial_radiation(inp.latitude_deg_n, inp.day_of_year);
    let big_n = daylight_hours(inp.latitude_deg_n, inp.day_of_year);
    let n = inp.sunshine_hours.min(big_n).max(0.0);
    let rs = solar_radiation_from_sunshine(n, big_n, ra);
    let rso = clear_sky_radiation(inp.altitude_m, ra);
    let rns = net_shortwave_radiation(rs);

    // FAO-56 §3.5.2: when Rso = 0 (e.g. polar night), use 0.7 as the
    // Rs/Rso ratio — the midpoint of the FAO-56 cloudiness factor range
    // [0.33, 1.0], matching the "moderately cloudy" assumption for missing
    // radiation data (Allen et al. 1998, Eq. 39 notes).
    let rs_rso = if rso > 0.0 { (rs / rso).min(1.0) } else { 0.7 };
    let rnl = net_longwave_radiation(inp.tmax_c, inp.tmin_c, ea, rs_rso);
    let rn = rns - rnl;

    penman_monteith(rn, 0.0, tmean, u2, vpd, delta, gamma)
}

/// FAO-56 Example 18 reference inputs (Uccle, Belgium, 6 July).
///
/// Expected ET₀ = 3.88 mm day⁻¹.
#[must_use]
pub const fn example_18_inputs() -> DailyWeatherInputs {
    DailyWeatherInputs {
        tmax_c: 21.5,
        tmin_c: 12.3,
        rhmax_pct: 84.0,
        rhmin_pct: 63.0,
        wind_speed_10m_km_h: 10.0,
        sunshine_hours: 9.25,
        latitude_deg_n: 50.8,
        altitude_m: 100.0,
        day_of_year: 187,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn svp_at_20c() {
        let es = saturation_vapour_pressure(20.0);
        assert!(
            (es - 2.338).abs() < 0.005,
            "FAO-56 Table 2.3: e°(20) ≈ 2.338 kPa"
        );
    }

    #[test]
    fn slope_at_20c() {
        let d = slope_vapour_pressure_curve(20.0);
        assert!(
            (d - 0.1447).abs() < 0.005,
            "FAO-56 Table 2.4: Δ(20) ≈ 0.1447"
        );
    }

    #[test]
    fn pressure_at_sea_level() {
        assert!((atmospheric_pressure(0.0) - 101.3).abs() < 0.1);
    }

    #[test]
    fn pressure_at_100m() {
        let p = atmospheric_pressure(100.0);
        assert!((p - 100.1).abs() < 0.3);
    }

    #[test]
    fn wind_conversion_10m_to_2m() {
        let u2 = wind_speed_at_2m(2.778, 10.0);
        assert!(
            (u2 - 2.078).abs() < 0.01,
            "FAO-56 Example 18: u2 ≈ 2.078 m/s"
        );
    }

    #[test]
    fn example_18_et0() {
        let inp = example_18_inputs();
        let et0 = daily_et0(&inp);
        assert!(
            (et0 - 3.88).abs() < 0.10,
            "FAO-56 Example 18: ET₀ ≈ 3.88 mm/day, got {et0:.4}"
        );
    }

    #[test]
    fn example_18_intermediates() {
        let inp = example_18_inputs();
        let tmean = f64::midpoint(inp.tmax_c, inp.tmin_c);
        let es = mean_saturation_vapour_pressure(inp.tmax_c, inp.tmin_c);
        let ea = actual_vapour_pressure_rh(inp.tmax_c, inp.tmin_c, inp.rhmax_pct, inp.rhmin_pct);

        assert!((tmean - 16.9).abs() < 0.1);
        assert!((es - 2.0).abs() < 0.1, "es ≈ 2.0 kPa, got {es:.4}");
        assert!((ea - 1.41).abs() < 0.05, "ea ≈ 1.41 kPa, got {ea:.4}");
    }

    #[test]
    fn daylight_hours_summer_mid_latitude() {
        let n = daylight_hours(50.8, 187);
        assert!(
            n > 15.0 && n < 17.0,
            "Uccle July daylight ≈ 16h, got {n:.1}"
        );
    }

    #[test]
    fn et0_deterministic() {
        let inp = example_18_inputs();
        let a = daily_et0(&inp);
        let b = daily_et0(&inp);
        assert!((a - b).abs() < f64::EPSILON);
    }
}
