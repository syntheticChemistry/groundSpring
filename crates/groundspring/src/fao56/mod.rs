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
//! provenance: groundSpring V10 `mc_et0_propagate.wgsl` → barraCuda S72).
//! [`seasonal_step`] dispatches via `SeasonalPipelineF64` (S80, provenance:
//! airSpring V035 → barraCuda S80) for fused ET₀→Kc→θ→stress.
//! [`seasonal_multi_day`] wraps multi-day runs with `StatefulPipeline`
//! for day-over-day state tracking (barraCuda S80, airSpring V039).
//! Sub-functions remain local as the validation reference.

mod constants;
mod equations;
mod pipeline;

pub use equations::*;
pub use pipeline::{
    monte_carlo_et0, seasonal_multi_day, seasonal_step, Et0Uncertainties, McEt0Result,
    SeasonalCellInputs, SeasonalOutput, SeasonalParams,
};

use constants::{HARGREAVES_COEFF, HARGREAVES_TEMP_OFFSET};

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
/// `barracuda` feature is enabled (absorbed in barraCuda S71+++).
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

pub(crate) fn daily_et0_cpu(inp: &DailyWeatherInputs) -> f64 {
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

/// Hargreaves reference ET₀ from temperature only (mm day⁻¹).
///
/// `ET₀ = 0.0023 · (T_mean + 17.8) · (T_max − T_min)^0.5 · Ra`
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::stats::hydrology::hargreaves_et0` (absorbed from
/// airSpring V035 via barraCuda S71+++).
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
/// (airSpring V035 → barraCuda S71+++).
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

/// Interpolate crop coefficient between growth stages.
///
/// FAO-56 §6.3: linear interpolation of Kc within a growth stage.
/// Delegates to `barracuda::stats::hydrology::crop_coefficient` when
/// the `barracuda` feature is enabled (airSpring → barraCuda S71+++).
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
/// → barraCuda S71+++).
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
    use crate::tol;
    use std::f64::consts::PI;

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
    fn example_18_et0() {
        let inp = example_18_inputs();
        let et0 = daily_et0(&inp);
        assert!(
            (et0 - 3.88).abs() < tol::EQUILIBRIUM,
            "FAO-56 Example 18: ET₀ ≈ 3.88 mm/day, got {et0:.4}"
        );
    }

    #[test]
    fn example_18_intermediates() {
        let inp = example_18_inputs();
        let tmean = f64::midpoint(inp.tmax_c, inp.tmin_c);
        let es = mean_saturation_vapour_pressure(inp.tmax_c, inp.tmin_c);
        let ea = actual_vapour_pressure_rh(inp.tmax_c, inp.tmin_c, inp.rhmax_pct, inp.rhmin_pct);

        assert!((tmean - 16.9).abs() < tol::EQUILIBRIUM);
        assert!(
            (es - 2.0).abs() < tol::EQUILIBRIUM,
            "es ≈ 2.0 kPa, got {es:.4}"
        );
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
                (val - scalar).abs() < tol::ANALYTICAL,
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

    #[test]
    fn crop_coefficient_endpoints() {
        let kc_start = crop_coefficient(0.3, 1.2, 0, 30);
        let kc_end = crop_coefficient(0.3, 1.2, 30, 30);
        assert!(
            (kc_start - 0.3).abs() < tol::STOCHASTIC,
            "day 0 should be kc_prev, got {kc_start}"
        );
        assert!(
            (kc_end - 1.2).abs() < tol::STOCHASTIC,
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

    #[test]
    fn soil_water_balance_basic() {
        let theta = soil_water_balance(100.0, 10.0, 5.0, 8.0, 200.0);
        assert!(
            (theta - 107.0).abs() < tol::STOCHASTIC,
            "100+10+5-8=107, got {theta}"
        );
    }

    #[test]
    fn soil_water_balance_capped_at_fc() {
        let theta = soil_water_balance(190.0, 20.0, 0.0, 2.0, 200.0);
        assert!(
            (theta - 200.0).abs() < tol::STOCHASTIC,
            "should cap at FC=200, got {theta}"
        );
    }

    #[test]
    fn soil_water_balance_floor_at_zero() {
        let theta = soil_water_balance(5.0, 0.0, 0.0, 20.0, 200.0);
        assert!(theta >= 0.0, "should not go negative, got {theta}");
    }
}
