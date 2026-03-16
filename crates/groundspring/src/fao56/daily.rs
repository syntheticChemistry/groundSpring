// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! FAO-56 Penman-Monteith daily reference ET₀.
//!
//! Core API for the full FAO-56 Eq. 6 chain with RH data and wind
//! height conversion (Example 18 pattern).

use super::equations::{
    actual_vapour_pressure_rh, atmospheric_pressure, clear_sky_radiation, daylight_hours,
    extraterrestrial_radiation, mean_saturation_vapour_pressure, net_longwave_radiation,
    net_shortwave_radiation, penman_monteith, psychrometric_constant, slope_vapour_pressure_curve,
    solar_radiation_from_sunshine, wind_speed_at_2m,
};

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

/// CPU-only Penman-Monteith ET₀ computation (reference path).
///
/// Used as the local fallback when the `barracuda` feature is not enabled
/// and as the cross-substrate reference for GPU parity validation.
#[must_use]
pub fn daily_et0_cpu(inp: &DailyWeatherInputs) -> f64 {
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
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::super::equations::{
        daylight_hours, extraterrestrial_radiation, mean_saturation_vapour_pressure,
        solar_radiation_from_sunshine, wind_speed_at_2m,
    };
    use super::*;
    use crate::tol;

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
        let ea = super::super::equations::actual_vapour_pressure_rh(
            inp.tmax_c,
            inp.tmin_c,
            inp.rhmax_pct,
            inp.rhmin_pct,
        );

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
    fn extraterrestrial_radiation_summer() {
        let ra = extraterrestrial_radiation(50.8, 187);
        assert!(
            (35.0..50.0).contains(&ra),
            "Uccle July Ra ≈ 40 MJ/m²/day, got {ra:.1}"
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
    fn hargreaves_vs_penman_same_order() {
        let inp = example_18_inputs();
        let pm = daily_et0(&inp);
        let hg = super::super::hargreaves::hargreaves_et0(
            inp.tmax_c,
            inp.tmin_c,
            inp.latitude_deg_n,
            inp.day_of_year,
        );
        let ratio = hg / pm;
        assert!(
            (0.3..3.5).contains(&ratio),
            "Hargreaves/PM ratio={ratio:.2}, expected same order of magnitude"
        );
    }

    #[test]
    fn all_et0_methods_same_order_of_magnitude() {
        let inp = example_18_inputs();
        let pm = daily_et0(&inp);
        let hg = super::super::hargreaves::hargreaves_et0(
            inp.tmax_c,
            inp.tmin_c,
            inp.latitude_deg_n,
            inp.day_of_year,
        );
        let ra = extraterrestrial_radiation(inp.latitude_deg_n, inp.day_of_year);
        let big_n = daylight_hours(inp.latitude_deg_n, inp.day_of_year);
        let n = inp.sunshine_hours.min(big_n).max(0.0);
        let rs = solar_radiation_from_sunshine(n, big_n, ra);
        let tmean = f64::midpoint(inp.tmax_c, inp.tmin_c);
        let rh_mean = f64::midpoint(inp.rhmax_pct, inp.rhmin_pct);

        let mk = super::super::et0_methods::makkink_et0(tmean, rs);
        let tu = super::super::et0_methods::turc_et0(tmean, rs, rh_mean);
        let ha = super::super::et0_methods::hamon_et0(tmean, big_n);

        // Thornthwaite outputs mm/month — normalize to mm/day for comparison.
        let monthly = [
            2.0, 3.0, 7.0, 12.0, 16.9, 20.0, 22.0, 21.0, 17.0, 12.0, 6.0, 3.0,
        ];
        let hi = super::super::et0_methods::thornthwaite_heat_index(&monthly);
        let th_monthly = super::super::et0_methods::thornthwaite_et0(tmean, hi, big_n, 30.0);
        let th = th_monthly / 30.0;

        for (name, val) in [
            ("PM", pm),
            ("HG", hg),
            ("MK", mk),
            ("TU", tu),
            ("HA", ha),
            ("TH", th),
        ] {
            assert!(
                val > 0.0 && val < 20.0,
                "{name} ET₀ should be in (0, 20) mm/day, got {val:.2}"
            );
        }
    }
}
