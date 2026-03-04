// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Monte Carlo uncertainty propagation and seasonal fused pipeline for FAO-56.
//!
//! Extracted from `fao56/mod.rs` to keep each file under 1 000 lines.
//! Cross-spring lineage:
//! - Monte Carlo: groundSpring V10 `mc_et0_propagate.wgsl` → barraCuda S72
//! - Seasonal: airSpring V035 → barraCuda S80 `SeasonalPipelineF64`
//! - Multi-day: airSpring V039 → barraCuda S80 `StatefulPipeline`

use super::{crop_coefficient, daily_et0, DailyWeatherInputs};

// ── Physical clamp bounds for perturbed meteorological inputs ────────

/// Minimum relative humidity (%) — physical floor for arid conditions.
const RH_MIN_FLOOR_PCT: f64 = 5.0;
/// Maximum relative humidity (%) — saturation ceiling.
const RH_MAX_CEIL_PCT: f64 = 100.0;
/// Minimum physically plausible `RH_max` (%) for Monte Carlo perturbation.
const RHMAX_FLOOR_PCT: f64 = 10.0;
/// Minimum wind speed (km/h) to avoid division-by-zero in Penman-Monteith.
const WIND_SPEED_FLOOR_KMH: f64 = 0.5;

// ── Monte Carlo uncertainty propagation ───────────────────────────

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
// Cross-spring lineage: airSpring V035 → barraCuda S80
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
/// (barraCuda S80, provenance: airSpring V035).
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

// ── Multi-day stateful pipeline ─────────────────────────────────
//
// Cross-spring lineage: airSpring V039 → barraCuda S80 `StatefulPipeline`
// Carries soil moisture from day N as input to day N+1.

/// Run the seasonal pipeline over multiple consecutive days, carrying
/// soil moisture state forward automatically.
///
/// When the `barracuda-gpu` feature is enabled, wraps the per-day dispatch
/// in a `barracuda::pipeline::StatefulPipeline<WaterBalanceState>` for
/// structured day-over-day state tracking (barraCuda S80, provenance:
/// airSpring V039).
///
/// Falls back to a simple loop with manual state propagation otherwise.
#[must_use]
pub fn seasonal_multi_day(
    cells: &[SeasonalCellInputs],
    daily_params: &[SeasonalParams],
) -> Vec<Vec<SeasonalOutput>> {
    #[cfg(feature = "barracuda-gpu")]
    {
        seasonal_multi_day_stateful(cells, daily_params)
    }
    #[cfg(not(feature = "barracuda-gpu"))]
    seasonal_multi_day_loop(cells, daily_params)
}

#[cfg(feature = "barracuda-gpu")]
fn seasonal_multi_day_stateful(
    cells: &[SeasonalCellInputs],
    daily_params: &[SeasonalParams],
) -> Vec<Vec<SeasonalOutput>> {
    use barracuda::pipeline::{StatefulPipeline, WaterBalanceState};

    let mut pipeline = StatefulPipeline::<WaterBalanceState>::new();
    if let Some(first) = cells.first() {
        pipeline.state.soil_moisture = first.theta_prev;
    }

    let mut all_outputs = Vec::with_capacity(daily_params.len());
    let mut current_cells: Vec<SeasonalCellInputs> = cells.to_vec();

    for params in daily_params {
        let outputs = seasonal_step(&current_cells, params);
        for (cell, out) in current_cells.iter_mut().zip(outputs.iter()) {
            cell.theta_prev = out.theta_new;
        }
        pipeline.state.soil_moisture = outputs
            .first()
            .map_or(pipeline.state.soil_moisture, |o| o.theta_new);
        all_outputs.push(outputs);
    }
    all_outputs
}

#[cfg(not(feature = "barracuda-gpu"))]
fn seasonal_multi_day_loop(
    cells: &[SeasonalCellInputs],
    daily_params: &[SeasonalParams],
) -> Vec<Vec<SeasonalOutput>> {
    let mut all_outputs = Vec::with_capacity(daily_params.len());
    let mut current_cells: Vec<SeasonalCellInputs> = cells.to_vec();

    for params in daily_params {
        let outputs = seasonal_step(&current_cells, params);
        for (cell, out) in current_cells.iter_mut().zip(outputs.iter()) {
            cell.theta_prev = out.theta_new;
        }
        all_outputs.push(outputs);
    }
    all_outputs
}
