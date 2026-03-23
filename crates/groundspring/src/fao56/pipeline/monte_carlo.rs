// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

use super::{RH_MAX_CEIL_PCT, RH_MIN_FLOOR_PCT, RHMAX_FLOOR_PCT, WIND_SPEED_FLOOR_KMH};
use crate::cast::f64_usize;
use crate::fao56::{DailyWeatherInputs, daily_et0};

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
/// samples in a single GPU pass via `McEt0PropagateGpu` (barraCuda S72,
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

    let device = crate::gpu::get_device_f64_safe()?;
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
    let mut rng = crate::prng::DefaultRng::new(seed);
    let mut samples = Vec::with_capacity(n_samples);

    for _ in 0..n_samples {
        let perturbed = DailyWeatherInputs {
            tmax_c: rng.normal(base.tmax_c, unc.sigma_tmax),
            tmin_c: rng
                .normal(base.tmin_c, unc.sigma_tmin)
                .min(base.tmax_c + rng.normal(0.0, unc.sigma_tmin) - 1.0),
            rhmax_pct: rng
                .normal(base.rhmax_pct, unc.sigma_rhmax)
                .clamp(RHMAX_FLOOR_PCT, RH_MAX_CEIL_PCT),
            rhmin_pct: rng
                .normal(base.rhmin_pct, unc.sigma_rhmin)
                .clamp(RH_MIN_FLOOR_PCT, RH_MAX_CEIL_PCT),
            wind_speed_10m_km_h: rng
                .normal(
                    base.wind_speed_10m_km_h,
                    base.wind_speed_10m_km_h * unc.sigma_wind_frac,
                )
                .max(WIND_SPEED_FLOOR_KMH),
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

/// Population mean and variance via Welford's algorithm (single-pass, stable).
///
/// When `barracuda-gpu` is enabled, tries the fused GPU reduce path first.
fn mc_mean_variance(data: &[f64]) -> (f64, f64) {
    #[cfg(feature = "barracuda-gpu")]
    {
        if let Some(pair) = mc_mean_variance_gpu(data) {
            return pair;
        }
    }
    crate::stats::metrics::welford_population(data)
}

#[cfg(feature = "barracuda-gpu")]
fn mc_mean_variance_gpu(data: &[f64]) -> Option<(f64, f64)> {
    let device = crate::gpu::get_device_f64_safe()?;
    let var_op = barracuda::ops::variance_f64_wgsl::VarianceF64::new(device).ok()?;
    let mv: (f64, f64) = var_op.mean_variance(data, 0).ok()?.into();
    Some(mv)
}

fn summarize_mc_samples(samples: &mut [f64]) -> McEt0Result {
    let (mean, variance) = mc_mean_variance(samples);
    let n = crate::cast::usize_f64(samples.len());

    samples.sort_by(f64::total_cmp);

    let pct_05 = samples[f64_usize(0.05 * n)];
    let pct_95 = samples[f64_usize(0.95 * n)];

    McEt0Result {
        mean,
        std: variance.sqrt(),
        pct_05,
        pct_95,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fao56::DailyWeatherInputs;

    fn test_base() -> DailyWeatherInputs {
        DailyWeatherInputs {
            tmax_c: 30.0,
            tmin_c: 18.0,
            rhmax_pct: 80.0,
            rhmin_pct: 40.0,
            wind_speed_10m_km_h: 8.0,
            sunshine_hours: 10.0,
            latitude_deg_n: 42.0,
            altitude_m: 200.0,
            day_of_year: 180,
        }
    }

    fn test_unc() -> Et0Uncertainties {
        Et0Uncertainties {
            sigma_tmax: 0.5,
            sigma_tmin: 0.5,
            sigma_rhmax: 3.0,
            sigma_rhmin: 3.0,
            sigma_wind_frac: 0.10,
            sigma_sun_frac: 0.05,
        }
    }

    #[test]
    fn monte_carlo_cpu_produces_valid_distribution() {
        let result = monte_carlo_et0(&test_base(), &test_unc(), 500, 42);
        assert!(result.mean > 0.0 && result.mean < 15.0);
        assert!(result.std > 0.0 && result.std < result.mean);
        assert!(result.pct_05 < result.mean);
        assert!(result.pct_95 > result.mean);
        assert!(result.pct_05 < result.pct_95);
    }

    #[test]
    fn monte_carlo_cpu_deterministic() {
        let a = monte_carlo_et0(&test_base(), &test_unc(), 200, 42);
        let b = monte_carlo_et0(&test_base(), &test_unc(), 200, 42);
        assert_eq!(a.mean.to_bits(), b.mean.to_bits());
        assert_eq!(a.std.to_bits(), b.std.to_bits());
    }

    #[test]
    fn monte_carlo_different_seeds_differ() {
        let a = monte_carlo_et0(&test_base(), &test_unc(), 200, 42);
        let b = monte_carlo_et0(&test_base(), &test_unc(), 200, 99);
        assert_ne!(a.mean.to_bits(), b.mean.to_bits());
    }

    #[test]
    fn summarize_mc_samples_known_values() {
        let mut samples: Vec<f64> = (1..=100).map(f64::from).collect();
        let result = summarize_mc_samples(&mut samples);
        assert!((result.mean - 50.5).abs() < 0.01);
        assert!(result.pct_05 < 10.0);
        assert!(result.pct_95 > 90.0);
    }
}
