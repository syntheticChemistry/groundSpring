// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

use crate::fao56::{DailyWeatherInputs, crop_coefficient, daily_et0};

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

    let device = crate::gpu::get_device_f64_safe()?;
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn test_cell() -> SeasonalCellInputs {
        SeasonalCellInputs {
            tmax_c: 30.0,
            tmin_c: 18.0,
            rhmax_pct: 80.0,
            rhmin_pct: 40.0,
            wind_2m_ms: 2.0,
            rs_mj: 20.0,
            altitude_m: 200.0,
            latitude_deg_n: 42.0,
            theta_prev: 80.0,
        }
    }

    fn test_params() -> SeasonalParams {
        SeasonalParams {
            day_of_year: 180,
            stage_length: 30,
            day_in_stage: 15,
            kc_prev: 0.6,
            kc_next: 1.15,
            taw: 120.0,
            raw_fraction: 0.5,
            field_capacity: 100.0,
        }
    }

    #[test]
    fn seasonal_step_single_cell() {
        let cells = vec![test_cell()];
        let outputs = seasonal_step(&cells, &test_params());
        assert_eq!(outputs.len(), 1);
        let out = &outputs[0];
        assert!(out.et0 > 0.0 && out.et0 < 15.0);
        assert!(out.kc > 0.0);
        assert!(out.etc > 0.0);
        assert!(out.theta_new >= 0.0 && out.theta_new <= test_params().field_capacity);
        assert!((0.0..=1.0).contains(&out.stress));
    }

    #[test]
    fn seasonal_step_deterministic() {
        let cells = vec![test_cell()];
        let a = seasonal_step(&cells, &test_params());
        let b = seasonal_step(&cells, &test_params());
        assert_eq!(a[0].et0.to_bits(), b[0].et0.to_bits());
        assert_eq!(a[0].theta_new.to_bits(), b[0].theta_new.to_bits());
    }

    #[test]
    fn seasonal_multi_day_carries_state() {
        let cells = vec![test_cell()];
        let days: Vec<SeasonalParams> = (0..5)
            .map(|i| SeasonalParams {
                day_of_year: 180 + i,
                day_in_stage: 15 + u32::from(i),
                ..test_params()
            })
            .collect();
        let outputs = seasonal_multi_day(&cells, &days);
        assert_eq!(outputs.len(), 5);
        for day_out in &outputs {
            assert_eq!(day_out.len(), 1);
            assert!(day_out[0].et0 > 0.0);
        }
        let soil: Vec<f64> = outputs.iter().map(|d| d[0].theta_new).collect();
        assert!(
            soil.windows(2).all(|w| w[0] >= w[1]),
            "soil moisture should decrease or stay constant with ET"
        );
    }

    #[test]
    fn seasonal_kc_interpolation() {
        let cells = vec![test_cell()];
        let start = SeasonalParams {
            day_in_stage: 0,
            ..test_params()
        };
        let mid = test_params();
        let end = SeasonalParams {
            day_in_stage: 30,
            ..test_params()
        };
        let o_start = seasonal_step(&cells, &start);
        let o_mid = seasonal_step(&cells, &mid);
        let o_end = seasonal_step(&cells, &end);
        assert!(o_start[0].kc <= o_mid[0].kc);
        assert!(o_mid[0].kc <= o_end[0].kc);
    }
}
