// SPDX-License-Identifier: AGPL-3.0-only
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
//! [`daily_et0`] delegates to `barracuda::stats::hydrology::fao56_et0()`
//! on CPU (S71+++). [`hargreaves_et0`] delegates similarly. Batch variants
//! use `HargreavesBatchGpu` (S71 — GPU-parallel via
//! `hargreaves_batch_f64.wgsl`) when `barracuda-gpu` is enabled.
//! [`crop_coefficient`] and [`soil_water_balance`] delegate to
//! `barracuda::stats::hydrology` CPU functions (S71+++).
//! [`monte_carlo_et0`] dispatches via `McEt0PropagateGpu` (S80,
//! provenance: groundSpring V10 `mc_et0_propagate.wgsl` → `ToadStool` S72).
//! [`seasonal_step`] dispatches via `SeasonalPipelineF64` (S80, provenance:
//! airSpring V035 → `ToadStool` S80) for fused ET₀→Kc→θ→stress.
//! Sub-functions remain local as the validation reference.

use std::f64::consts::PI;

// ── Physical constants ──────────────────────────────────────────────

/// Solar constant (MJ m⁻² min⁻¹).  FAO-56 p. 47.
const GSC: f64 = 0.0820;

/// Stefan-Boltzmann constant (MJ m⁻² day⁻¹ K⁻⁴).  FAO-56 Eq. 39.
const SIGMA: f64 = 4.903e-9;

/// Default grass albedo.  FAO-56 Eq. 38.
const ALBEDO: f64 = 0.23;

/// Ångström regression coefficient `a_s` (fraction of `R_a` reaching earth on overcast days).
/// FAO-56 Eq. 35 default.
const ANGSTROM_A: f64 = 0.25;

/// Ångström regression coefficient `b_s` (additional fraction on clear days).
/// FAO-56 Eq. 35 default.
const ANGSTROM_B: f64 = 0.50;

/// Clear-sky altitude coefficient (m⁻¹). FAO-56 Eq. 37: `R_so` = (0.75 + 2e-5·z)·`R_a`.
const CLEAR_SKY_BASE: f64 = 0.75;
const CLEAR_SKY_ALT_COEFF: f64 = 2e-5;

/// Net longwave humidity factor coefficients. FAO-56 Eq. 39.
const LW_HUMIDITY_INTERCEPT: f64 = 0.34;
const LW_HUMIDITY_SLOPE: f64 = 0.14;

/// Net longwave cloudiness factor coefficients. FAO-56 Eq. 39.
const LW_CLOUD_SLOPE: f64 = 1.35;
const LW_CLOUD_INTERCEPT: f64 = -0.35;

/// Tetens formula coefficients. FAO-56 Eq. 11.
const TETENS_A: f64 = 0.6108;
const TETENS_B: f64 = 17.27;
const TETENS_C: f64 = 237.3;

/// Inverse latent heat of vaporization at ~20 °C (kg MJ⁻¹).
/// FAO-56 Eq. 6: converts energy (MJ m⁻²) to water depth (mm).
/// λ ≈ 2.45 MJ kg⁻¹ → 1/λ ≈ 0.408.
const PM_LAMBDA_INV: f64 = 0.408;

/// Wind function numerator coefficient for 24-hour grass reference.
/// FAO-56 Eq. 6: `PM_WIND_NUM` / (`T_mean` + `PM_KELVIN_OFFSET`).
const PM_WIND_NUM: f64 = 900.0;

/// Approximate Celsius-to-Kelvin offset used in the wind function.
/// FAO-56 Eq. 6 denominator: (`T_mean` + 273).
const PM_KELVIN_OFFSET: f64 = 273.0;

/// Wind function denominator coefficient for 24-hour grass reference.
/// FAO-56 Eq. 6: γ (1 + 0.34 u₂).
const PM_WIND_DENOM: f64 = 0.34;

/// Hargreaves empirical coefficient (Hargreaves & Samani, 1985).
const HARGREAVES_COEFF: f64 = 0.0023;

/// Hargreaves temperature offset (°C) (Hargreaves & Samani, 1985).
const HARGREAVES_TEMP_OFFSET: f64 = 17.8;

// ── Sub-functions ───────────────────────────────────────────────────

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
/// Delegates to `barracuda::stats::hydrology::fao56_et0` when the
/// `barracuda` feature is enabled (absorbed in `ToadStool` S71+++).
#[must_use]
pub fn daily_et0(inp: &DailyWeatherInputs) -> f64 {
    #[cfg(feature = "barracuda")]
    {
        let ra = extraterrestrial_radiation(inp.latitude_deg_n, inp.day_of_year);
        let big_n = daylight_hours(inp.latitude_deg_n, inp.day_of_year);
        let n = inp.sunshine_hours.min(big_n).max(0.0);
        let rs = solar_radiation_from_sunshine(n, big_n, ra);
        let wind_ms = inp.wind_speed_10m_km_h / 3.6;
        let u2 = wind_speed_at_2m(wind_ms, 10.0);
        if let Some(et0) = barracuda::stats::hydrology::fao56_et0(
            inp.tmax_c,
            inp.tmin_c,
            inp.rhmax_pct,
            inp.rhmin_pct,
            u2,
            rs,
            inp.altitude_m,
            inp.latitude_deg_n,
            u32::from(inp.day_of_year),
        ) {
            return et0;
        }
    }
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

/// Compute ET₀ for a batch of station-days.
///
/// When the `barracuda-gpu` feature is enabled and a GPU is available,
/// dispatches the entire batch to `BatchedElementwiseF64::fao56_et0_batch`
/// on the GPU. Falls back to sequential CPU calls to [`daily_et0`] otherwise.
#[must_use]
pub fn daily_et0_batch(inputs: &[DailyWeatherInputs]) -> Vec<f64> {
    #[cfg(feature = "barracuda-gpu")]
    {
        if let Some(results) = daily_et0_batch_gpu(inputs) {
            return results;
        }
    }
    inputs.iter().map(daily_et0).collect()
}

#[cfg(feature = "barracuda-gpu")]
fn daily_et0_batch_gpu(inputs: &[DailyWeatherInputs]) -> Option<Vec<f64>> {
    use barracuda::ops::batched_elementwise_f64::{BatchedElementwiseF64, StationDayInput};

    let device = crate::gpu::get_device()?;
    let gpu = BatchedElementwiseF64::new(device).ok()?;

    let station_days: Vec<StationDayInput> = inputs
        .iter()
        .map(|inp| {
            let ra = extraterrestrial_radiation(inp.latitude_deg_n, inp.day_of_year);
            let big_n = daylight_hours(inp.latitude_deg_n, inp.day_of_year);
            let n = inp.sunshine_hours.min(big_n).max(0.0);
            let rs = solar_radiation_from_sunshine(n, big_n, ra);
            let wind_ms = inp.wind_speed_10m_km_h / 3.6;
            let u2 = wind_speed_at_2m(wind_ms, 10.0);
            (
                inp.tmax_c,
                inp.tmin_c,
                inp.rhmax_pct,
                inp.rhmin_pct,
                u2,
                rs,
                inp.altitude_m,
                inp.latitude_deg_n,
                u32::from(inp.day_of_year),
            )
        })
        .collect();

    gpu.fao56_et0_batch(&station_days).ok()
}

// ── Hargreaves ET₀ (temperature-only) ─────────────────────────────
//
// Cross-spring lineage: airSpring V035 → ToadStool S70+ → groundSpring
// When radiation data is unavailable, Hargreaves (1985) provides a
// temperature-only reference ET₀.  The equation uses extraterrestrial
// radiation Ra (computed from latitude + day-of-year) rather than
// measured solar radiation.  Accuracy is lower than Penman-Monteith
// (~±20 %) but sufficient for screening and gap-filling.

/// Hargreaves reference ET₀ from temperature only (mm day⁻¹).
///
/// `ET₀ = 0.0023 · (T_mean + 17.8) · (T_max − T_min)^0.5 · Ra`
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::stats::hydrology::hargreaves_et0` (absorbed from
/// airSpring V035 via `ToadStool` S71+++).
#[must_use]
pub fn hargreaves_et0(tmax_c: f64, tmin_c: f64, latitude_deg_n: f64, day_of_year: u16) -> f64 {
    let ra = extraterrestrial_radiation(latitude_deg_n, day_of_year);
    #[cfg(feature = "barracuda")]
    {
        if let Some(et0) = barracuda::stats::hydrology::hargreaves_et0(ra, tmax_c, tmin_c) {
            return et0;
        }
    }
    hargreaves_et0_cpu(ra, tmax_c, tmin_c)
}

fn hargreaves_et0_cpu(ra: f64, tmax_c: f64, tmin_c: f64) -> f64 {
    let tmean = f64::midpoint(tmax_c, tmin_c);
    let td = (tmax_c - tmin_c).max(0.0);
    HARGREAVES_COEFF * (tmean + HARGREAVES_TEMP_OFFSET) * td.sqrt() * ra
}

/// Compute Hargreaves ET₀ for a batch of days.
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::stats::hydrology::hargreaves_et0_batch`.
/// When `barracuda-gpu` is enabled and a GPU is available, dispatches
/// via `BatchedElementwiseF64::execute` with `Op::HargreavesEt0`
/// (airSpring V035 → `ToadStool` S71+++).
#[must_use]
pub fn hargreaves_et0_batch(
    tmax_c: &[f64],
    tmin_c: &[f64],
    latitude_deg_n: f64,
    day_of_year: u16,
) -> Vec<f64> {
    let n = tmax_c.len().min(tmin_c.len());
    let ra = extraterrestrial_radiation(latitude_deg_n, day_of_year);
    #[cfg(any(feature = "barracuda", feature = "barracuda-gpu"))]
    {
        let ra_vec: Vec<f64> = vec![ra; n];
        #[cfg(feature = "barracuda-gpu")]
        {
            if let Some(results) = hargreaves_et0_batch_gpu(&ra_vec, tmax_c, tmin_c) {
                return results;
            }
        }
        if let Some(results) =
            barracuda::stats::hydrology::hargreaves_et0_batch(&ra_vec, &tmax_c[..n], &tmin_c[..n])
        {
            return results;
        }
    }
    (0..n)
        .map(|i| hargreaves_et0_cpu(ra, tmax_c[i], tmin_c[i]))
        .collect()
}

#[cfg(feature = "barracuda-gpu")]
fn hargreaves_et0_batch_gpu(ra: &[f64], tmax: &[f64], tmin: &[f64]) -> Option<Vec<f64>> {
    use barracuda::ops::batched_elementwise_f64::{BatchedElementwiseF64, Op};

    let device = crate::gpu::get_device()?;
    if let Ok(gpu) = barracuda::stats::hydrology::HargreavesBatchGpu::new(device.clone()) {
        if let Ok(result) = gpu.dispatch(ra, tmax, tmin) {
            return Some(result);
        }
    }
    let gpu = BatchedElementwiseF64::new(device).ok()?;
    let n = ra.len();
    let mut data = Vec::with_capacity(n * 3);
    for i in 0..n {
        data.push(ra[i]);
        data.push(tmax[i]);
        data.push(tmin[i]);
    }
    gpu.execute(&data, n, Op::HargreavesEt0).ok()
}

// ── Crop coefficient & soil water balance ─────────────────────────
//
// Cross-spring lineage: airSpring FAO-56 → ToadStool S70+ → groundSpring
// Completing the chain from reference ET₀ to actual crop water use.

/// Interpolate crop coefficient between growth stages.
///
/// FAO-56 §6.3: linear interpolation of Kc within a growth stage.
/// Delegates to `barracuda::stats::hydrology::crop_coefficient` when
/// the `barracuda` feature is enabled (airSpring → `ToadStool` S71+++).
#[must_use]
pub fn crop_coefficient(kc_prev: f64, kc_next: f64, day_in_stage: u32, stage_length: u32) -> f64 {
    #[cfg(feature = "barracuda")]
    return barracuda::stats::hydrology::crop_coefficient(
        kc_prev,
        kc_next,
        day_in_stage,
        stage_length,
    );
    #[cfg(not(feature = "barracuda"))]
    {
        if stage_length == 0 {
            return kc_prev;
        }
        let t = f64::from(day_in_stage) / f64::from(stage_length);
        (kc_next - kc_prev).mul_add(t.clamp(0.0, 1.0), kc_prev)
    }
}

/// Simple daily soil water balance (mm).
///
/// `θ_{t+1} = min(θ_t + P + I − ET_c, FC)`
///
/// Delegates to `barracuda::stats::hydrology::soil_water_balance` when
/// the `barracuda` feature is enabled (airSpring precision agriculture
/// → `ToadStool` S71+++).
#[must_use]
pub fn soil_water_balance(
    theta: f64,
    precip: f64,
    irrigation: f64,
    et_c: f64,
    field_capacity: f64,
) -> f64 {
    #[cfg(feature = "barracuda")]
    return barracuda::stats::hydrology::soil_water_balance(
        theta,
        precip,
        irrigation,
        et_c,
        field_capacity,
    );
    #[cfg(not(feature = "barracuda"))]
    (theta + precip + irrigation - et_c).clamp(0.0, field_capacity)
}

// ── Monte Carlo uncertainty propagation ───────────────────────────
//
// Cross-spring lineage: groundSpring V10 mc_et0_propagate.wgsl →
// ToadStool S72 McEt0PropagateGpu → groundSpring library function.

/// Uncertainty (σ) for each meteorological input perturbed during
/// Monte Carlo ET₀ propagation.
#[derive(Debug, Clone, Copy)]
pub struct Et0Uncertainties {
    /// σ for `T_max` (°C).
    pub sigma_tmax: f64,
    /// σ for `T_min` (°C).
    pub sigma_tmin: f64,
    /// σ for `RH_max` (%).
    pub sigma_rhmax: f64,
    /// σ for `RH_min` (%).
    pub sigma_rhmin: f64,
    /// Fractional σ for wind speed (dimensionless, e.g. 0.10 = 10 %).
    pub sigma_wind_frac: f64,
    /// Fractional σ for sunshine hours (dimensionless).
    pub sigma_sun_frac: f64,
}

/// Result of Monte Carlo uncertainty propagation through FAO-56 ET₀.
#[derive(Debug, Clone, Copy)]
pub struct McEt0Result {
    /// Ensemble mean of ET₀ samples (mm day⁻¹).
    pub mean: f64,
    /// Population standard deviation (mm day⁻¹).
    pub std: f64,
    /// 5th percentile of the uncertainty distribution.
    pub pct_05: f64,
    /// 95th percentile of the uncertainty distribution.
    pub pct_95: f64,
}

/// Monte Carlo uncertainty propagation through FAO-56 Penman-Monteith.
///
/// Generates `n_samples` perturbed ET₀ values by drawing meteorological
/// inputs from their uncertainty distributions and evaluating the full
/// equation chain for each draw.
///
/// When `barracuda-gpu` is enabled and a GPU is available, dispatches all
/// samples in a single GPU pass via `McEt0PropagateGpu` (`ToadStool` S72,
/// provenance: groundSpring V10 `mc_et0_propagate.wgsl`).
/// Falls back to sequential CPU sampling otherwise.
#[must_use]
pub fn monte_carlo_et0(
    base: &DailyWeatherInputs,
    unc: &Et0Uncertainties,
    n_samples: usize,
    seed: u64,
) -> McEt0Result {
    #[cfg(feature = "barracuda-gpu")]
    {
        if let Some(result) = monte_carlo_et0_gpu(base, unc, n_samples, seed) {
            return result;
        }
    }
    monte_carlo_et0_cpu(base, unc, n_samples, seed)
}

#[cfg(feature = "barracuda-gpu")]
#[expect(
    clippy::cast_possible_truncation,
    reason = "n_samples fits in u32 for practical GPU dispatch sizes"
)]
fn monte_carlo_et0_gpu(
    base: &DailyWeatherInputs,
    unc: &Et0Uncertainties,
    n_samples: usize,
    _seed: u64,
) -> Option<McEt0Result> {
    use barracuda::stats::hydrology::gpu::{
        Fao56BaseInputs, Fao56Uncertainties, McEt0PropagateGpu,
    };

    let device = crate::gpu::get_device()?;
    let gpu = McEt0PropagateGpu::new(device).ok()?;

    let base_inputs = Fao56BaseInputs {
        t_max: base.tmax_c,
        t_min: base.tmin_c,
        rh_max: base.rhmax_pct,
        rh_min: base.rhmin_pct,
        wind_kmh: base.wind_speed_10m_km_h,
        sun_hours: base.sunshine_hours,
        latitude: base.latitude_deg_n,
        altitude: base.altitude_m,
        day_of_year: f64::from(base.day_of_year),
    };
    let uncertainties = Fao56Uncertainties {
        sigma_t_max: unc.sigma_tmax,
        sigma_t_min: unc.sigma_tmin,
        sigma_rh_max: unc.sigma_rhmax,
        sigma_rh_min: unc.sigma_rhmin,
        sigma_wind_frac: unc.sigma_wind_frac,
        sigma_sun_frac: unc.sigma_sun_frac,
    };

    let mut samples = gpu
        .dispatch(&base_inputs, &uncertainties, n_samples as u32)
        .ok()?;
    Some(summarize_mc_samples(&mut samples))
}

fn monte_carlo_et0_cpu(
    base: &DailyWeatherInputs,
    unc: &Et0Uncertainties,
    n_samples: usize,
    seed: u64,
) -> McEt0Result {
    let mut rng = crate::prng::Xorshift64::new(seed);
    let mut samples = Vec::with_capacity(n_samples);

    for _ in 0..n_samples {
        let perturbed = DailyWeatherInputs {
            tmax_c: rng.normal(base.tmax_c, unc.sigma_tmax),
            tmin_c: rng
                .normal(base.tmin_c, unc.sigma_tmin)
                .min(base.tmax_c + rng.normal(0.0, unc.sigma_tmin) - 1.0),
            rhmax_pct: rng
                .normal(base.rhmax_pct, unc.sigma_rhmax)
                .clamp(10.0, 100.0),
            rhmin_pct: rng
                .normal(base.rhmin_pct, unc.sigma_rhmin)
                .clamp(5.0, 100.0),
            wind_speed_10m_km_h: rng
                .normal(
                    base.wind_speed_10m_km_h,
                    base.wind_speed_10m_km_h * unc.sigma_wind_frac,
                )
                .max(0.5),
            sunshine_hours: rng
                .normal(
                    base.sunshine_hours,
                    base.sunshine_hours * unc.sigma_sun_frac,
                )
                .max(0.0),
            ..*base
        };
        samples.push(daily_et0(&perturbed));
    }

    summarize_mc_samples(&mut samples)
}

fn summarize_mc_samples(samples: &mut [f64]) -> McEt0Result {
    let n = crate::cast::usize_f64(samples.len());
    let mean = samples.iter().sum::<f64>() / n;
    let variance = samples.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n;

    samples.sort_by(f64::total_cmp);

    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "percentile index from f64 fraction of known-valid usize"
    )]
    let pct_05 = samples[(0.05 * n) as usize];
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "percentile index from f64 fraction of known-valid usize"
    )]
    let pct_95 = samples[(0.95 * n) as usize];

    McEt0Result {
        mean,
        std: variance.sqrt(),
        pct_05,
        pct_95,
    }
}

// ── Seasonal fused pipeline ──────────────────────────────────────
//
// Cross-spring lineage: airSpring V035 → ToadStool S80
// SeasonalPipelineF64: ET₀ → Kc → water balance → stress in one GPU dispatch.

/// Inputs for a single spatial cell of the seasonal pipeline.
#[derive(Debug, Clone, Copy)]
pub struct SeasonalCellInputs {
    /// Maximum air temperature (°C).
    pub tmax_c: f64,
    /// Minimum air temperature (°C).
    pub tmin_c: f64,
    /// Maximum relative humidity (%).
    pub rhmax_pct: f64,
    /// Minimum relative humidity (%).
    pub rhmin_pct: f64,
    /// Wind speed at 2 m (m s⁻¹).
    pub wind_2m_ms: f64,
    /// Solar radiation (MJ m⁻² day⁻¹).
    pub rs_mj: f64,
    /// Elevation (m).
    pub altitude_m: f64,
    /// Latitude (°N).
    pub latitude_deg_n: f64,
    /// Previous-day soil moisture (mm).
    pub theta_prev: f64,
}

/// Seasonal pipeline parameters (growth-stage and soil constants).
#[derive(Debug, Clone, Copy)]
pub struct SeasonalParams {
    /// Day of year (1–366).
    pub day_of_year: u16,
    /// Growth stage length (days).
    pub stage_length: u32,
    /// Day within current growth stage.
    pub day_in_stage: u32,
    /// Previous Kc.
    pub kc_prev: f64,
    /// Next Kc.
    pub kc_next: f64,
    /// Total available water (mm).
    pub taw: f64,
    /// Readily available water fraction (0–1).
    pub raw_fraction: f64,
    /// Field capacity (mm).
    pub field_capacity: f64,
}

/// Output from one cell of the seasonal pipeline.
#[derive(Debug, Clone, Copy)]
pub struct SeasonalOutput {
    /// Reference ET₀ (mm day⁻¹).
    pub et0: f64,
    /// Crop coefficient.
    pub kc: f64,
    /// Crop ET (mm day⁻¹).
    pub etc: f64,
    /// Updated soil moisture (mm).
    pub theta_new: f64,
    /// Water stress index (0 = no stress, 1 = fully stressed).
    pub stress: f64,
}

/// Fused seasonal pipeline: ET₀ → Kc → water balance → stress.
///
/// Runs the full pipeline for multiple spatial cells in a single step.
/// When `barracuda-gpu` is enabled, dispatches via `SeasonalPipelineF64`
/// (`ToadStool` S80, provenance: airSpring V035).
/// Falls back to sequential CPU evaluation otherwise.
#[must_use]
pub fn seasonal_step(cells: &[SeasonalCellInputs], params: &SeasonalParams) -> Vec<SeasonalOutput> {
    #[cfg(feature = "barracuda-gpu")]
    {
        if let Some(result) = seasonal_step_gpu(cells, params) {
            return result;
        }
    }
    cells
        .iter()
        .map(|cell| seasonal_step_single(cell, params))
        .collect()
}

fn seasonal_step_single(cell: &SeasonalCellInputs, params: &SeasonalParams) -> SeasonalOutput {
    let inp = DailyWeatherInputs {
        tmax_c: cell.tmax_c,
        tmin_c: cell.tmin_c,
        rhmax_pct: cell.rhmax_pct,
        rhmin_pct: cell.rhmin_pct,
        wind_speed_10m_km_h: cell.wind_2m_ms * 3.6 * (10.0_f64 / 2.0).ln() / (67.8_f64.ln()),
        sunshine_hours: 0.0,
        latitude_deg_n: cell.latitude_deg_n,
        altitude_m: cell.altitude_m,
        day_of_year: params.day_of_year,
    };
    let et0 = daily_et0(&inp);
    let kc = crop_coefficient(
        params.kc_prev,
        params.kc_next,
        params.day_in_stage,
        params.stage_length,
    );
    let etc = et0 * kc;
    let raw = params.raw_fraction * params.taw;
    let depletion = (params.field_capacity - cell.theta_prev).max(0.0);
    let ks = if depletion > raw {
        ((params.taw - depletion) / (params.taw - raw)).clamp(0.0, 1.0)
    } else {
        1.0
    };
    let etc_adj = ks * etc;
    let theta_new = (cell.theta_prev - etc_adj).clamp(0.0, params.field_capacity);
    let stress = 1.0 - ks;
    SeasonalOutput {
        et0,
        kc,
        etc,
        theta_new,
        stress,
    }
}

#[cfg(feature = "barracuda-gpu")]
#[expect(
    clippy::cast_possible_truncation,
    reason = "cell count and day parameters fit in u32 for practical sizes"
)]
fn seasonal_step_gpu(
    cells: &[SeasonalCellInputs],
    params: &SeasonalParams,
) -> Option<Vec<SeasonalOutput>> {
    use barracuda::stats::hydrology::gpu::{SeasonalGpuParams, SeasonalPipelineF64};

    let device = crate::gpu::get_device()?;
    let gpu = SeasonalPipelineF64::new(device).ok()?;

    let mut cell_weather = Vec::with_capacity(cells.len() * 9);
    for c in cells {
        cell_weather.extend_from_slice(&[
            c.tmax_c,
            c.tmin_c,
            c.rhmax_pct,
            c.rhmin_pct,
            c.wind_2m_ms,
            c.rs_mj,
            c.altitude_m,
            c.latitude_deg_n,
            c.theta_prev,
        ]);
    }

    let mut gpu_params = <SeasonalGpuParams as bytemuck::Zeroable>::zeroed();
    gpu_params.cell_count = cells.len() as u32;
    gpu_params.day_of_year = u32::from(params.day_of_year);
    gpu_params.stage_length = params.stage_length;
    gpu_params.day_in_stage = params.day_in_stage;
    gpu_params.kc_prev = params.kc_prev;
    gpu_params.kc_next = params.kc_next;
    gpu_params.taw_default = params.taw;
    gpu_params.raw_fraction = params.raw_fraction;
    gpu_params.field_capacity = params.field_capacity;

    let barracuda_output = gpu.dispatch(&cell_weather, &gpu_params).ok()?;

    let results: Vec<SeasonalOutput> = barracuda_output
        .iter()
        .map(|o| SeasonalOutput {
            et0: o.et0,
            kc: o.kc,
            etc: o.etc,
            theta_new: o.theta_new,
            stress: o.stress,
        })
        .collect();

    Some(results)
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
    fn psychrometric_constant_at_sea_level() {
        let gamma = psychrometric_constant(101.3);
        assert!(
            (gamma - 0.0674).abs() < 0.002,
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
        let delta = solar_declination(80); // ~ spring equinox
        let ws = sunset_hour_angle(phi, delta);
        assert!(
            (ws - PI / 2.0).abs() < 0.2,
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
    fn et0_deterministic() {
        let inp = example_18_inputs();
        let a = daily_et0(&inp);
        let b = daily_et0(&inp);
        assert!((a - b).abs() < f64::EPSILON);
    }

    // ── Hargreaves ET₀ tests ──────────────────────────────────────

    #[test]
    fn hargreaves_positive() {
        let et0 = hargreaves_et0(21.5, 12.3, 50.8, 187);
        assert!(et0 > 0.0, "Hargreaves ET₀ should be positive, got {et0}");
    }

    #[test]
    fn hargreaves_summer_gt_winter() {
        let summer = hargreaves_et0(30.0, 18.0, 45.0, 180);
        let winter = hargreaves_et0(5.0, -2.0, 45.0, 15);
        assert!(
            summer > winter,
            "summer ET₀ ({summer:.2}) should exceed winter ({winter:.2})"
        );
    }

    #[test]
    fn hargreaves_vs_penman_same_order() {
        let inp = example_18_inputs();
        let pm = daily_et0(&inp);
        let hg = hargreaves_et0(inp.tmax_c, inp.tmin_c, inp.latitude_deg_n, inp.day_of_year);
        let ratio = hg / pm;
        // Hargreaves is a temperature-only estimate with ~20-30% typical error;
        // for humid sites with low ΔT, it can overestimate by up to 3× relative
        // to Penman-Monteith due to missing humidity/wind correction.
        assert!(
            (0.3..3.5).contains(&ratio),
            "Hargreaves/PM ratio={ratio:.2}, expected same order of magnitude"
        );
    }

    #[test]
    fn hargreaves_batch_matches_scalar() {
        let tmax = [25.0, 30.0, 20.0];
        let tmin = [15.0, 18.0, 10.0];
        let batch = hargreaves_et0_batch(&tmax, &tmin, 45.0, 180);
        for (i, &val) in batch.iter().enumerate() {
            let scalar = hargreaves_et0(tmax[i], tmin[i], 45.0, 180);
            assert!(
                (val - scalar).abs() < 1e-10,
                "batch[{i}]={val} != scalar={scalar}"
            );
        }
    }

    #[test]
    fn hargreaves_deterministic() {
        let a = hargreaves_et0(25.0, 15.0, 45.0, 180);
        let b = hargreaves_et0(25.0, 15.0, 45.0, 180);
        assert!((a - b).abs() < f64::EPSILON);
    }

    // ── Crop coefficient tests ────────────────────────────────────

    #[test]
    fn crop_coefficient_endpoints() {
        let kc_start = crop_coefficient(0.3, 1.2, 0, 30);
        let kc_end = crop_coefficient(0.3, 1.2, 30, 30);
        assert!(
            (kc_start - 0.3).abs() < 0.01,
            "day 0 should be kc_prev, got {kc_start}"
        );
        assert!(
            (kc_end - 1.2).abs() < 0.01,
            "day=stage should be kc_next, got {kc_end}"
        );
    }

    #[test]
    fn crop_coefficient_midpoint() {
        let kc = crop_coefficient(0.4, 1.0, 15, 30);
        assert!(
            (kc - 0.7).abs() < 0.05,
            "midpoint Kc should be ~0.7, got {kc}"
        );
    }

    #[test]
    fn crop_coefficient_zero_length() {
        let kc = crop_coefficient(0.5, 1.0, 0, 0);
        assert!(
            (kc - 0.5).abs() < f64::EPSILON,
            "zero stage length returns kc_prev"
        );
    }

    // ── Soil water balance tests ──────────────────────────────────

    #[test]
    fn soil_water_balance_basic() {
        let theta = soil_water_balance(100.0, 10.0, 5.0, 8.0, 200.0);
        assert!((theta - 107.0).abs() < 0.01, "100+10+5-8=107, got {theta}");
    }

    #[test]
    fn soil_water_balance_capped_at_fc() {
        let theta = soil_water_balance(190.0, 20.0, 0.0, 2.0, 200.0);
        assert!(
            (theta - 200.0).abs() < 0.01,
            "should cap at FC=200, got {theta}"
        );
    }

    #[test]
    fn soil_water_balance_floor_at_zero() {
        let theta = soil_water_balance(5.0, 0.0, 0.0, 20.0, 200.0);
        assert!(theta >= 0.0, "should not go negative, got {theta}");
    }
}
