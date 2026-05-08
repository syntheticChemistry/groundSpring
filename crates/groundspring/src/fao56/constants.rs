// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! FAO-56 physical constants.
//!
//! Equation parameters from Allen et al. (1998) "Crop evapotranspiration —
//! Guidelines for computing crop water requirements", FAO Irrigation and
//! Drainage Paper 56. Each constant cites the relevant FAO-56 equation.

/// Solar constant (MJ m⁻² min⁻¹).  FAO-56 p. 47.
pub const GSC: f64 = 0.0820;

/// Stefan-Boltzmann constant (MJ m⁻² day⁻¹ K⁻⁴).  FAO-56 Eq. 39.
pub const SIGMA: f64 = 4.903e-9;

/// Default grass albedo.  FAO-56 Eq. 38.
pub const ALBEDO: f64 = 0.23;

/// Ångström regression coefficient `a_s` (fraction of `R_a` reaching earth on overcast days).
/// FAO-56 Eq. 35 default.
pub const ANGSTROM_A: f64 = 0.25;

/// Ångström regression coefficient `b_s` (additional fraction on clear days).
/// FAO-56 Eq. 35 default.
pub const ANGSTROM_B: f64 = 0.50;

/// Clear-sky altitude coefficient (m⁻¹). FAO-56 Eq. 37: `R_so` = (0.75 + 2e-5·z)·`R_a`.
pub const CLEAR_SKY_BASE: f64 = 0.75;
pub const CLEAR_SKY_ALT_COEFF: f64 = 2e-5;

/// Net longwave humidity factor coefficients. FAO-56 Eq. 39.
pub const LW_HUMIDITY_INTERCEPT: f64 = 0.34;
pub const LW_HUMIDITY_SLOPE: f64 = 0.14;

/// Net longwave cloudiness factor coefficients. FAO-56 Eq. 39.
pub const LW_CLOUD_SLOPE: f64 = 1.35;
pub const LW_CLOUD_INTERCEPT: f64 = -0.35;

/// Tetens formula coefficients. FAO-56 Eq. 11.
pub const TETENS_A: f64 = 0.6108;
pub const TETENS_B: f64 = 17.27;
pub const TETENS_C: f64 = 237.3;

/// Inverse latent heat of vaporization at ~20 °C (kg MJ⁻¹).
/// FAO-56 Eq. 6: converts energy (MJ m⁻²) to water depth (mm).
/// λ ≈ 2.45 MJ kg⁻¹ → 1/λ ≈ 0.408.
pub const PM_LAMBDA_INV: f64 = 0.408;

/// Wind function numerator coefficient for 24-hour grass reference.
/// FAO-56 Eq. 6: `PM_WIND_NUM` / (`T_mean` + `PM_KELVIN_OFFSET`).
pub const PM_WIND_NUM: f64 = 900.0;

/// Approximate Celsius-to-Kelvin offset used in the wind function.
/// FAO-56 Eq. 6 denominator: (`T_mean` + 273).
pub const PM_KELVIN_OFFSET: f64 = 273.0;

/// Wind function denominator coefficient for 24-hour grass reference.
/// FAO-56 Eq. 6: γ (1 + 0.34 u₂).
pub const PM_WIND_DENOM: f64 = 0.34;

/// Hargreaves empirical coefficient (Hargreaves & Samani, 1985).
pub const HARGREAVES_COEFF: f64 = 0.0023;

/// Hargreaves temperature offset (°C) (Hargreaves & Samani, 1985).
pub const HARGREAVES_TEMP_OFFSET: f64 = 17.8;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tetens_at_20c() {
        let t = 20.0;
        let e_sat = TETENS_A * ((TETENS_B * t) / (t + TETENS_C)).exp();
        assert!((e_sat - 2.338).abs() < 0.01);
    }

    #[test]
    fn albedo_in_range() {
        assert!(ALBEDO > 0.0 && ALBEDO < 1.0);
    }

    #[test]
    fn angstrom_coefficients_sum_to_expected() {
        assert!((ANGSTROM_A + ANGSTROM_B - 0.75).abs() < 1e-10);
    }

    #[test]
    fn pm_lambda_inv_reasonable() {
        assert!((PM_LAMBDA_INV - 0.408).abs() < 1e-10);
    }
}
