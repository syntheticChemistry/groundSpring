// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! FAO-56 intermediate equation functions.
//!
//! Building-block functions for the Penman-Monteith reference ET₀ chain.
//! Each function cites the exact FAO-56 equation number. These are called
//! by [`super::daily_et0`] and related high-level wrappers.

use super::constants::{
    ALBEDO, ANGSTROM_A, ANGSTROM_B, CLEAR_SKY_ALT_COEFF, CLEAR_SKY_BASE, GSC, LW_CLOUD_INTERCEPT,
    LW_CLOUD_SLOPE, LW_HUMIDITY_INTERCEPT, LW_HUMIDITY_SLOPE, PM_KELVIN_OFFSET, PM_LAMBDA_INV,
    PM_WIND_DENOM, PM_WIND_NUM, SIGMA, TETENS_A, TETENS_B, TETENS_C,
};
use std::f64::consts::PI;

/// Saturation vapour pressure at temperature `t_c` (kPa).
///
/// FAO-56 Eq. 11: e°(T) = 0.6108 exp(17.27 T / (T + 237.3))
#[must_use]
pub fn saturation_vapour_pressure(t_c: f64) -> f64 {
    TETENS_A * (TETENS_B * t_c / (t_c + TETENS_C)).exp()
}

/// Slope of the saturation vapour pressure curve (kPa °C⁻¹).
///
/// FAO-56 Eq. 13.
#[must_use]
pub fn slope_vapour_pressure_curve(t_c: f64) -> f64 {
    let es = saturation_vapour_pressure(t_c);
    4098.0 * es / (t_c + TETENS_C).powi(2)
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
    (ANGSTROM_A + ANGSTROM_B * n / big_n) * ra
}

/// Clear-sky solar radiation (MJ m⁻² day⁻¹).
///
/// FAO-56 Eq. 37: `R_so` = (0.75 + 2×10⁻⁵ z) `R_a`
#[must_use]
pub fn clear_sky_radiation(altitude_m: f64, ra: f64) -> f64 {
    altitude_m.mul_add(CLEAR_SKY_ALT_COEFF, CLEAR_SKY_BASE) * ra
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
    let humidity_factor = LW_HUMIDITY_SLOPE.mul_add(-ea_kpa.sqrt(), LW_HUMIDITY_INTERCEPT);
    let cloudiness_factor = LW_CLOUD_SLOPE.mul_add(rs_over_rso, LW_CLOUD_INTERCEPT);
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
    let numerator = (PM_LAMBDA_INV * delta).mul_add(
        rn - g,
        gamma * (PM_WIND_NUM / (tmean_c + PM_KELVIN_OFFSET)) * u2 * vpd_kpa,
    );
    let denominator = gamma.mul_add(PM_WIND_DENOM.mul_add(u2, 1.0), delta);
    numerator / denominator
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use crate::tol;

    #[test]
    fn svp_at_20c() {
        let es = saturation_vapour_pressure(20.0);
        assert!(
            (es - 2.338).abs() < tol::DECOMPOSITION,
            "FAO-56 Table 2.3: e°(20) ≈ 2.338 kPa"
        );
    }

    #[test]
    fn slope_at_20c() {
        let d = slope_vapour_pressure_curve(20.0);
        assert!(
            (d - 0.1447).abs() < tol::DECOMPOSITION,
            "FAO-56 Table 2.4: Δ(20) ≈ 0.1447"
        );
    }

    #[test]
    fn pressure_at_sea_level() {
        assert!((atmospheric_pressure(0.0) - 101.3).abs() < tol::EQUILIBRIUM);
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
            (u2 - 2.078).abs() < tol::STOCHASTIC,
            "FAO-56 Example 18: u2 ≈ 2.078 m/s"
        );
    }

    #[test]
    fn psychrometric_constant_at_sea_level() {
        let gamma = psychrometric_constant(101.3);
        assert!(
            (gamma - 0.0674).abs() < tol::DECOMPOSITION,
            "FAO-56 Eq. 8: γ ≈ 0.0674, got {gamma:.4}"
        );
    }

    #[test]
    fn solar_declination_summer_solstice() {
        let delta = solar_declination(172);
        let delta_deg = delta.to_degrees();
        assert!(
            (delta_deg - 23.45).abs() < 1.0,
            "summer solstice δ ≈ 23.45°, got {delta_deg:.2}"
        );
    }

    #[test]
    fn inverse_relative_distance_range() {
        for doy in [1, 105, 187, 365] {
            let dr = inverse_relative_distance(doy);
            assert!(
                (0.96..=1.04).contains(&dr),
                "d_r should be ~1.0 ± 0.033, got {dr} at doy {doy}"
            );
        }
    }

    #[test]
    fn sunset_hour_angle_equator_equinox() {
        let phi = 0.0_f64.to_radians();
        let delta = solar_declination(80);
        let ws = sunset_hour_angle(phi, delta);
        assert!(
            (ws - std::f64::consts::PI / 2.0).abs() < 0.2,
            "equatorial equinox ωs ≈ π/2, got {ws:.3}"
        );
    }

    #[test]
    fn extraterrestrial_radiation_summer() {
        let ra = extraterrestrial_radiation(50.8, 187);
        assert!(
            (35.0..50.0).contains(&ra),
            "Uccle July Ra ≈ 40 MJ/m²/day, got {ra:.1}"
        );
    }

    #[test]
    fn clear_sky_radiation_at_sea_level() {
        let ra = 40.0;
        let rso = clear_sky_radiation(0.0, ra);
        assert!((rso - 30.0).abs() < 1.0, "Rso = 0.75·Ra = 30, got {rso:.1}");
    }

    #[test]
    fn net_shortwave_radiation_albedo() {
        let rns = net_shortwave_radiation(20.0);
        assert!(
            (rns - 15.4).abs() < 0.1,
            "Rns = (1-0.23)·20 = 15.4, got {rns:.1}"
        );
    }

    #[test]
    fn saturation_vp_monotone() {
        let es_10 = saturation_vapour_pressure(10.0);
        let es_30 = saturation_vapour_pressure(30.0);
        assert!(es_30 > es_10, "VP should increase with temperature");
    }

    #[test]
    fn slope_vp_positive() {
        let slope = slope_vapour_pressure_curve(20.0);
        assert!(slope > 0.0, "slope should be positive");
    }

    #[test]
    fn atmospheric_pressure_sea_level() {
        let p = atmospheric_pressure(0.0);
        assert!((p - 101.3).abs() < 0.1, "P(z=0) = {p}, expected ~101.3");
    }

    #[test]
    fn atmospheric_pressure_decreases_with_altitude() {
        let p_0 = atmospheric_pressure(0.0);
        let p_1000 = atmospheric_pressure(1000.0);
        assert!(p_1000 < p_0, "pressure should decrease with altitude");
    }

    #[test]
    fn wind_speed_identity_at_2m() {
        let u2 = wind_speed_at_2m(2.0, 2.0);
        assert!((u2 - 2.0).abs() < 0.01, "u2 at z=2 should be ~input");
    }

    #[test]
    fn wind_speed_higher_at_10m() {
        let u10 = 3.0;
        let u2 = wind_speed_at_2m(u10, 10.0);
        assert!(u2 < u10, "wind at 2m should be less than at 10m");
    }

    #[test]
    fn mean_saturation_vp_is_average() {
        let es = mean_saturation_vapour_pressure(30.0, 10.0);
        let manual = f64::midpoint(
            saturation_vapour_pressure(30.0),
            saturation_vapour_pressure(10.0),
        );
        assert!((es - manual).abs() < f64::EPSILON);
    }

    #[test]
    fn solar_declination_summer_positive() {
        // Summer solstice ~DOY 172: declination should be positive (northern hemisphere tilt)
        let delta = solar_declination(172);
        assert!(delta > 0.0, "summer solstice δ = {delta}, expected > 0");
    }

    #[test]
    fn solar_declination_winter_negative() {
        // Winter solstice ~DOY 355: declination should be negative
        let delta = solar_declination(355);
        assert!(delta < 0.0, "winter solstice δ = {delta}, expected < 0");
    }

    #[test]
    fn extraterrestrial_radiation_positive() {
        let ra = extraterrestrial_radiation(42.0, 172);
        assert!(ra > 0.0, "Ra should be positive");
    }

    #[test]
    fn daylight_summer_longer_than_winter() {
        let n_summer = daylight_hours(42.0, 172);
        let n_winter = daylight_hours(42.0, 355);
        assert!(
            n_summer > n_winter,
            "summer N={n_summer}, winter N={n_winter}"
        );
    }

    #[test]
    fn net_shortwave_less_than_incoming() {
        let rs = 20.0;
        let rns = net_shortwave_radiation(rs);
        assert!(rns < rs, "Rns should be less than Rs (albedo reflection)");
        assert!(rns > 0.0);
    }

    #[test]
    fn clear_sky_radiation_positive() {
        let ra = extraterrestrial_radiation(42.0, 172);
        let rso = clear_sky_radiation(200.0, ra);
        assert!(rso > 0.0 && rso < ra, "Rso should be between 0 and Ra");
    }

    #[test]
    fn penman_monteith_typical_range() {
        // Typical summer day inputs for mid-latitude location
        let rn = 15.0;
        let g = 0.5;
        let tmean = 25.0;
        let u2 = 2.0;
        let delta = slope_vapour_pressure_curve(tmean);
        let gamma = psychrometric_constant(atmospheric_pressure(200.0));
        let es = mean_saturation_vapour_pressure(30.0, 20.0);
        let ea = actual_vapour_pressure_rh(30.0, 20.0, 80.0, 40.0);
        let vpd = es - ea;

        let et0 = penman_monteith(rn, g, tmean, u2, vpd, delta, gamma);
        assert!(
            (1.0..12.0).contains(&et0),
            "ET₀ = {et0} mm/day, expected 1-12 for typical conditions"
        );
    }
}
