# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""
Tolerance constants mirroring groundspring::tol and groundspring-validate::tolerances.

Single source of truth for Python baselines — every Python script should import
tolerances from this module (or from control.common which re-exports them).
Values MUST stay in sync with:
  - crates/groundspring/src/lib.rs          → pub mod tol { … }
  - crates/groundspring/src/lib.rs          → pub(crate) mod eps { … }
  - crates/groundspring-validate/src/tolerances.rs
"""

from __future__ import annotations

# ── Core tol::* (crates/groundspring/src/lib.rs) ─────────────────────

# Bitwise determinism — reproducibility across platforms (CPU/GPU).
# Provenance: f64 ε ≈ 2.22e-16; ~5× for FMA contraction diffs.
TOL_DETERMINISM: float = 1e-15

# f64 identity — summation-only paths (no transcendentals).
# Provenance: Kahan compensated summation error O(N·ε), N ≤ 10⁴.
TOL_EXACT: float = 1e-12

# Summation-only with extended/compensated precision.
# Provenance: stricter than EXACT for Neumaier compensated sum paths.
TOL_STRICT: float = 1e-14

# One transcendental (sqrt, ln) introducing ~1 ULP.
# Provenance: IEEE 754 permits 1 ULP, composition ≈ 2 ULP ≈ 4.4e-16, padded.
TOL_ANALYTICAL: float = 1e-10

# CDF/erf approximation (A&S 7.1.26, two-layer composition).
# Provenance: max error 1.5e-7; chi² CDF compounds erf twice → ~1e-6.
TOL_CDF_APPROX: float = 1e-6

# CDF↔PPF round-trip (both approximations compound).
# Provenance: PPF inverts CDF via Newton (3 steps); round-trip ≈ CDF_APPROX².
TOL_ROUNDTRIP: float = 1e-5

# ODE integration error (RK4 O(dt⁴) accumulation).
# Provenance: local O(dt⁵), global O(dt⁴), dt = 0.01, 1000 steps → ~1e-8.
TOL_INTEGRATION: float = 1e-8

# Published results with 3–4 significant decimal digits.
# Provenance: scientific literature 3–4 sig figs → match within 0.001.
TOL_LITERATURE: float = 0.001

# Bias–variance decomposition fractions (Pythagorean identity rounding).
# Provenance: bias²/MSE + variance/MSE ≈ 1.0; fraction rounding ~0.5%.
TOL_DECOMPOSITION: float = 0.005

# Stochastic mean estimator with O(1/√N) convergence.
# Provenance: CLT gives σ/√N; N = 10⁴, σ ≈ 1 → SE ≈ 0.01.
TOL_STOCHASTIC: float = 0.01

# ODE equilibrium / physical measurement precision.
# Provenance: sensor precision ~10%; ODE steady-state convergence threshold.
TOL_EQUILIBRIUM: float = 0.1

# Spectral reconstruction RMSE (Tikhonov regularized inversion).
# Provenance: Hansen (1998); typical RMSE for well-conditioned problems.
TOL_RECONSTRUCTION: float = 1e-4

# ~2% normalization tolerance for integral conservation.
# Provenance: trapezoidal quadrature on coarse grids (N ≤ 100), O(h²).
TOL_NORM_2PCT: float = 0.02

# ── Validation-specific (crates/groundspring-validate/src/tolerances.rs) ──

# Rarefaction taxon proportions at moderate sequencing depth.
# Provenance: multinomial sampling variance at N ≈ 50 000.
TOL_RAREFACTION_PROP: float = 0.05

# Coarse stochastic regime classification (e.g. "all taxa detected").
TOL_REGIME: float = 0.5

# Grid-search matching tolerance for sweep array lookup.
TOL_GRID_MATCH: float = 0.01

# Monotonicity slack for quantities that should decrease but may fluctuate.
TOL_MONOTONIC_SLACK: float = 0.15

# Rust↔Python ET₀ method-comparison tolerance.
# Provenance: same equations, small rounding diffs from trig intermediates,
# Kelvin convention, mul_add vs multiply-then-add. Observed max delta 0.004.
TOL_ET0: float = 0.005

# ── Thresholds ────────────────────────────────────────────────────────

# R² ≥ 0.95 — strong model performance.
THRESHOLD_GOOD_R2: float = 0.95

# Index of Agreement ≥ 0.9 — excellent agreement.
THRESHOLD_GOOD_IA: float = 0.9

# Anderson localization: γ > 0.3 indicates exponential localization.
THRESHOLD_LARGE_GAMMA: float = 0.3

# ── Epsilon guards (crates/groundspring/src/lib.rs → eps) ─────────────

# Division-safe epsilon to avoid NaN.
EPS_SAFE_DIV: float = 1e-10

# Stricter division-safe epsilon (diffusion coefficients ~1e-15 m²/s).
EPS_SAFE_DIV_STRICT: float = 1e-20

# Near-zero guard for log/entropy computations.
EPS_LOG_FLOOR: float = 1e-15

# Underflow guard for condition number / matrix element magnitude.
EPS_UNDERFLOW: float = 1e-300

# ── Physical bounds ───────────────────────────────────────────────────

# Minimum plausible daily ET₀ (mm/day).
# Provenance: FAO-56 Table 2 (lowest reference conditions).
ET0_PLAUSIBLE_MIN_MM: float = 0.01

# Maximum plausible daily ET₀ (mm/day).
# Provenance: FAO-56 Appendix A (arid regions with Rs > 30 MJ/m²/day).
ET0_PLAUSIBLE_MAX_MM: float = 15.0
