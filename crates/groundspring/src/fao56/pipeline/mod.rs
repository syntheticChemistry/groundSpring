// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Monte Carlo uncertainty propagation and seasonal fused pipeline for FAO-56.
//!
//! Extracted from `fao56/mod.rs` to keep each file under 1 000 lines.
//! Cross-spring lineage:
//! - Monte Carlo: groundSpring V10 `mc_et0_propagate.wgsl` → barraCuda S72
//! - Seasonal: airSpring V035 → barraCuda S80 `SeasonalPipelineF64`
//! - Multi-day: airSpring V039 → barraCuda S80 `StatefulPipeline`

mod monte_carlo;
mod seasonal;

// ── Physical clamp bounds for perturbed meteorological inputs ────────

/// Minimum relative humidity (%) — physical floor for arid conditions.
pub const RH_MIN_FLOOR_PCT: f64 = 5.0;
/// Maximum relative humidity (%) — saturation ceiling.
pub const RH_MAX_CEIL_PCT: f64 = 100.0;
/// Minimum physically plausible `RH_max` (%) for Monte Carlo perturbation.
pub const RHMAX_FLOOR_PCT: f64 = 10.0;
/// Minimum wind speed (km/h) to avoid division-by-zero in Penman-Monteith.
pub const WIND_SPEED_FLOOR_KMH: f64 = 0.5;

// ── Re-exports ─────────────────────────────────────────────────────

pub use monte_carlo::{Et0Uncertainties, McEt0Result, monte_carlo_et0};
pub use seasonal::{
    SeasonalCellInputs, SeasonalOutput, SeasonalParams, seasonal_multi_day, seasonal_step,
};
