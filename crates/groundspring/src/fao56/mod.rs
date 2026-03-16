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
mod crop_soil;
mod daily;
mod equations;
mod et0_methods;
mod hargreaves;
mod pipeline;

pub use crop_soil::*;
pub use daily::*;
pub use equations::*;
pub use et0_methods::{
    hamon_et0, makkink_et0, thornthwaite_et0, thornthwaite_heat_index, turc_et0,
};
pub use hargreaves::*;
pub use pipeline::{
    Et0Uncertainties, McEt0Result, SeasonalCellInputs, SeasonalOutput, SeasonalParams,
    monte_carlo_et0, seasonal_multi_day, seasonal_step,
};
