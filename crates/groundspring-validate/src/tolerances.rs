// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Tolerance constants and physical bounds for validation.
//!
//! Re-exports from `groundspring::tol` where values match, and defines
//! validation-specific tolerances alongside.

// ── Tolerances ───────────────────────────────────────────────────────────
//
// Re-export from `groundspring::tol` where values match, and define
// validation-specific tolerances alongside.

/// f64 identity — values computed by the same deterministic path on
/// identical inputs.  Only IEEE 754 rounding distinguishes them.
pub const TOL_EXACT: f64 = groundspring::tol::EXACT;

/// Exact arithmetic (add / mul / div) on f64 inputs with at most one
/// transcendental (sqrt, ln) introducing ~1 ULP accumulated error.
pub const TOL_ANALYTICAL: f64 = groundspring::tol::ANALYTICAL;

/// Literature values reported to 3–4 significant decimals (e.g.
/// Dong et al. 2020 sensor MBE/RMSE calibrations).
pub const TOL_LITERATURE: f64 = groundspring::tol::LITERATURE;

/// Bias–variance decomposition fractions where the Pythagorean identity
/// RMSE² = MBE² + σ² amplifies rounding near the fourth decimal.
pub const TOL_DECOMPOSITION: f64 = groundspring::tol::DECOMPOSITION;

/// Finite-sample mean estimators from stochastic algorithms (Gillespie,
/// Monte Carlo) where sampling noise is O(1/√N).
pub const TOL_STOCHASTIC_MEAN: f64 = groundspring::tol::STOCHASTIC;

/// ODE equilibrium values and meteorological parameters where physical
/// measurement precision is ~0.1 unit.
pub const TOL_EQUILIBRIUM: f64 = groundspring::tol::EQUILIBRIUM;

/// Deterministic rerun tolerance — same code, same inputs, same seed.
/// Stricter than `TOL_EXACT` because no algorithmic variation is expected.
pub const TOL_DETERMINISM: f64 = groundspring::tol::DETERMINISM;

// ── Validation-specific tolerances (no library counterpart) ──────────

/// Rarefaction taxon proportions at moderate sequencing depth — multinomial
/// sampling variance at N ≈ 50 000.
pub const TOL_RAREFACTION_PROP: f64 = 0.05;

/// Coarse stochastic regime classification (e.g. "all taxa detected")
/// tolerating ±0.5 in count-like quantities.
pub const TOL_REGIME: f64 = 0.5;

/// Grid-search matching tolerance for locating a disorder/coupling value
/// in a sweep array (e.g. `(w - target).abs() < TOL_GRID_MATCH`).
pub const TOL_GRID_MATCH: f64 = 0.01;

/// Monotonicity slack for physical quantities that should decrease but
/// may exhibit minor non-monotonicity from finite sampling.
pub const TOL_MONOTONIC_SLACK: f64 = 0.15;

/// Threshold for strong model performance: R² ≥ 0.95.
/// Statistical regression fit quality — 95% of variance explained.
pub const THRESHOLD_GOOD_R2: f64 = 0.95;

/// Threshold for strong model agreement: IA ≥ 0.9.
/// Willmott Index of Agreement (d) — 0.9 indicates excellent agreement
/// between modeled and observed values.
pub const THRESHOLD_GOOD_IA: f64 = 0.9;

/// Anderson localization: Lyapunov exponent threshold for strong disorder.
/// γ > 0.3 indicates exponential localization in 1D disordered systems.
pub const THRESHOLD_LARGE_GAMMA: f64 = 0.3;

/// Division-safe epsilon to avoid NaN in `x / y.max(EPS_SAFE_DIV)`.
pub const EPS_SAFE_DIV: f64 = 1e-10;

/// Strict division-safe epsilon for quantities where physical floor is ~1e-15
/// (e.g. diffusion coefficients in m²/s). Below any physically meaningful value.
pub const EPS_SAFE_DIV_STRICT: f64 = 1e-20;

/// Rust vs Python ET₀ method-comparison tolerance.
///
/// Same equations, small rounding diffs from trig intermediates (Ra),
/// Kelvin convention (273.0 vs 273.16), `mul_add` vs multiply-then-add.
/// Hargreaves amplifies Ra differences.
///
/// Provenance: `control/et0_methods/et0_methods.py` (commit `231a3e99`,
/// 2026-03-19) — `python3 control/et0_methods/et0_methods.py`.
/// Observed max delta: PM 0.002, HG 0.004, MK 0.001, TU 0.001, HA 0.001.
/// 0.005 provides 1.25× margin over worst case (Hargreaves 0.004).
pub const TOL_ET0: f64 = 0.005;

// ── Physical bounds ──────────────────────────────────────────────────

/// Minimum plausible daily ET₀ (mm/day).
///
/// Any ET₀ estimate below this is physically implausible —
/// even arid winter days with minimal radiation produce some evaporation.
/// Provenance: FAO-56 Table 2 (lowest reference conditions).
pub const ET0_PLAUSIBLE_MIN_MM: f64 = 0.01;

/// Maximum plausible daily ET₀ (mm/day).
///
/// Even extreme desert summer conditions with Class-A pan coefficients
/// rarely exceed ~15 mm/day. Values above this indicate input error.
/// Provenance: FAO-56 Appendix A (arid regions with Rs > 30 MJ/m²/day).
pub const ET0_PLAUSIBLE_MAX_MM: f64 = 15.0;
