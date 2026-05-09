// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Shared tolerance constants for validation assertions.
//!
//! Use these named constants instead of bare float literals in tests and
//! validation code. Each tier corresponds to a specific numerical regime:
//! - **DETERMINISM** — bitwise reproducibility (1e-15)
//! - **STRICT** — summation with extended precision (1e-14)
//! - **EXACT** — summation-only paths (1e-12)
//! - **ANALYTICAL** — one transcendental (sqrt, ln) (1e-10)
//! - **INTEGRATION** — ODE RK4 accumulation (1e-8)
//! - **`CDF_APPROX`** — CDF/erf approximation (1e-6)
//! - **ROUNDTRIP** — CDF↔PPF round-trip (1e-5)
//! - **RECONSTRUCTION** — spectral Tikhonov roundtrip (1e-4)
//! - **LITERATURE** — published 3–4 sig figs (0.001)
//! - **DECOMPOSITION** — bias-variance fractions (0.005)
//! - **STOCHASTIC** — O(1/√N) mean estimator (0.01)
//! - **`NORM_2PCT`** — ~2% normalization (0.02)
//! - **EQUILIBRIUM** — ODE equilibrium / measurement (0.1)

/// Bitwise determinism — reproducibility across platforms (CPU/GPU).
///
/// Provenance: f64 machine epsilon is 2.22e-16; this is ~5× that,
/// covering FMA contraction differences between x86 and ARM.
/// Validated: `tests/determinism.rs`, all 34 experiments.
pub const DETERMINISM: f64 = 1e-15;

/// f64 identity — summation-only paths (no transcendentals).
///
/// Provenance: Kahan compensated summation error is O(N·ε) where
/// ε = 2.22e-16; for N ≤ 10⁴ this is < 1e-12.
/// Validated: `validate_rarefaction`, `validate_jackknife`.
pub const EXACT: f64 = 1e-12;

/// Summation-only with extended precision or compensated arithmetic.
///
/// Provenance: stricter than `EXACT` for paths where we control the
/// summation order (e.g. Neumaier compensated sum).
/// Validated: `validate_notill_sampling`.
pub const STRICT: f64 = 1e-14;

/// One transcendental (sqrt, ln) introducing ~1 ULP of error.
///
/// Provenance: IEEE 754 permits 1 ULP for correctly rounded
/// transcendentals; composition of sqrt + division ≈ 2 ULP ≈ 4.4e-16,
/// padded to 1e-10 for safety.
/// Validated: `validate_anderson`, `validate_transport`.
pub const ANALYTICAL: f64 = 1e-10;

/// CDF/erf approximation (A&S 7.1.26, two-layer composition).
///
/// Provenance: Abramowitz & Stegun formula 7.1.26 has max error
/// 1.5e-7; our chi² CDF compounds erf twice, giving ~1e-6.
/// Source: Abramowitz & Stegun (1964), §7.1.26.
/// Validated: `validate_decompose`, `validate_freeze_out`.
pub const CDF_APPROX: f64 = 1e-6;

/// CDF↔PPF round-trip (both approximations compound).
///
/// Provenance: PPF inverts CDF via Newton iteration (3 steps);
/// round-trip error ≈ `CDF_APPROX`² ≈ 1e-12, but we pad for
/// edge cases near 0/1.
/// Validated: `validate_decompose` round-trip checks.
pub const ROUNDTRIP: f64 = 1e-5;

/// ODE integration error (RK4 O(dt⁴) accumulation).
///
/// Provenance: Runge-Kutta 4th order local error is O(dt⁵),
/// global error O(dt⁴). With dt = 0.01, 1000 steps → ~1e-8.
/// Validated: `validate_bistable`, `validate_drift`.
pub const INTEGRATION: f64 = 1e-8;

/// Published results with 3–4 significant decimal digits.
///
/// Provenance: scientific literature typically reports 3–4 sig figs;
/// matching within 0.001 confirms faithful reproduction.
/// Validated: `validate_fao56`, `validate_et0_methods`.
pub const LITERATURE: f64 = 0.001;

/// Bias–variance decomposition fractions (Pythagorean identity rounding).
///
/// Provenance: bias²/MSE + variance/MSE should sum to 1.0;
/// floating-point fraction rounding introduces ~0.5% error.
/// Validated: `validate_decompose`.
pub const DECOMPOSITION: f64 = 0.005;

/// Stochastic mean estimator with O(1/√N) convergence.
///
/// Provenance: CLT gives σ/√N convergence; with N = 10⁴
/// and σ ≈ 1, standard error ≈ 0.01.
/// Validated: `validate_rawr`, `validate_resampling_conv`.
pub const STOCHASTIC: f64 = 0.01;

/// ODE equilibrium / physical measurement precision.
///
/// Provenance: physical measurements (sensors, weather stations)
/// have ~10% precision; ODE steady-state detection uses similar
/// threshold for convergence.
/// Validated: `validate_weather`, `validate_et0_anderson`.
pub const EQUILIBRIUM: f64 = 0.1;

/// Quantized inference — NPU int8 round-trip quantization error.
///
/// Provenance: AKD1000 int8 DMA classification quantizes f64→int8→f64;
/// round-trip error envelope is ~25%. Used exclusively by metalForge
/// for NPU cross-substrate comparison.
/// Validated: `validate_metalforge_cross_substrate`, `validate_npu_anderson`.
pub const QUANTIZED: f64 = 0.25;

/// Spectral reconstruction RMSE (Tikhonov regularized inversion).
///
/// Provenance: Tikhonov regularization trades bias for stability;
/// typical RMSE for well-conditioned problems is 1e-4.
/// Source: Hansen (1998), "Rank-Deficient and Discrete Ill-Posed Problems".
/// Validated: `validate_spectral_recon`.
pub const RECONSTRUCTION: f64 = 1e-4;

/// ~2% normalization tolerance for integral conservation.
///
/// Provenance: trapezoidal quadrature on coarse grids (N ≤ 100)
/// with error O(h²) ≈ 1e-4, padded for boundary effects.
/// Validated: `validate_quasispecies`, `validate_band_edge`.
pub const NORM_2PCT: f64 = 0.02;

// ─── Upstream Contract Pins ─────────────────────────────────────────────────
//
// Tolerance pins that bind groundSpring to specific barraCuda v0.3.7
// behaviour. If barraCuda changes these upstream values, the pinned tests
// will break immediately — intentional early detection of contract drift.
//
// Pattern absorbed from neuralSpring V124 (`UPSTREAM_HYDRO_*`,
// `UPSTREAM_PHYSICS_*`, `UPSTREAM_BIO_*`).

/// Pin: `barracuda::spectral::find_all_eigenvalues` must reproduce
/// tridiagonal eigenvalues within this tolerance for Anderson localization.
///
/// Contract: barraCuda v0.3.7, Jacobi rotation eigensolver.
/// Validated: `validate_anderson`, `validate_transport`.
pub const UPSTREAM_SPECTRAL_EIGH: f64 = ANALYTICAL;

/// Pin: `barracuda::ops::bio::multinomial_sample_cpu` must preserve
/// rarefaction diversity indices within this tolerance.
///
/// Contract: barraCuda v0.3.7, sequential multinomial sampler.
/// Validated: `validate_rarefaction`, `validate_rare_biosphere`.
pub const UPSTREAM_BIO_MULTINOMIAL: f64 = DETERMINISM;

/// Pin: `barracuda::optimize::brent` root finder must converge
/// within this tolerance for Penman-Monteith ET₀ computation.
///
/// Contract: barraCuda v0.3.7, Brent's method.
/// Validated: `validate_fao56`, `validate_et0_anderson`.
pub const UPSTREAM_OPTIMIZE_BRENT: f64 = ANALYTICAL;

/// Pin: `barracuda::spectral::anderson_sweep_averaged` must produce
/// Lyapunov exponents within this tolerance for disorder sweep.
///
/// Contract: barraCuda v0.3.7, averaged sweep over 100+ realizations.
/// Validated: `validate_anderson`, `validate_uncertainty_bridge`.
pub const UPSTREAM_SPECTRAL_ANDERSON: f64 = LITERATURE;

/// Pin: `barracuda::spectral::almost_mathieu_hamiltonian` must produce
/// eigenvalues matching Hofstadter's butterfly within this tolerance.
///
/// Contract: barraCuda v0.3.7, dense Hamiltonian construction.
/// Validated: `validate_quasiperiodic`.
pub const UPSTREAM_SPECTRAL_ALMOST_MATHIEU: f64 = EXACT;

/// Pin: `barracuda::spectral::detect_bands` must find band edges
/// within this tolerance for tight-binding chains.
///
/// Contract: barraCuda v0.3.7, gap-based band detection.
/// Validated: `validate_band_edge`.
pub const UPSTREAM_SPECTRAL_BANDS: f64 = ANALYTICAL;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiers_are_strictly_ordered() {
        assert!(DETERMINISM < STRICT);
        assert!(STRICT < EXACT);
        assert!(EXACT < ANALYTICAL);
        assert!(ANALYTICAL < INTEGRATION);
        assert!(INTEGRATION < CDF_APPROX);
        assert!(CDF_APPROX < ROUNDTRIP);
        assert!(ROUNDTRIP < RECONSTRUCTION);
        assert!(RECONSTRUCTION < LITERATURE);
        assert!(LITERATURE < DECOMPOSITION);
        assert!(DECOMPOSITION < STOCHASTIC);
        assert!(STOCHASTIC < NORM_2PCT);
        assert!(NORM_2PCT < EQUILIBRIUM);
        assert!(EQUILIBRIUM < QUANTIZED);
    }

    #[test]
    fn upstream_pins_match_base_tiers() {
        assert_eq!(UPSTREAM_SPECTRAL_EIGH, ANALYTICAL);
        assert_eq!(UPSTREAM_BIO_MULTINOMIAL, DETERMINISM);
        assert_eq!(UPSTREAM_OPTIMIZE_BRENT, ANALYTICAL);
        assert_eq!(UPSTREAM_SPECTRAL_ANDERSON, LITERATURE);
        assert_eq!(UPSTREAM_SPECTRAL_ALMOST_MATHIEU, EXACT);
        assert_eq!(UPSTREAM_SPECTRAL_BANDS, ANALYTICAL);
    }

    #[test]
    fn all_tiers_are_positive() {
        for &t in &[
            DETERMINISM,
            STRICT,
            EXACT,
            ANALYTICAL,
            INTEGRATION,
            CDF_APPROX,
            ROUNDTRIP,
            RECONSTRUCTION,
            LITERATURE,
            DECOMPOSITION,
            STOCHASTIC,
            NORM_2PCT,
            EQUILIBRIUM,
            QUANTIZED,
        ] {
            assert!(t > 0.0);
        }
    }
}
