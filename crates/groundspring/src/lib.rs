// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team
#![forbid(unsafe_code)]
#![deny(clippy::expect_used, clippy::unwrap_used)]

//! # groundSpring
//!
//! Measurement noise characterization primitives for the ecoPrimals ecosystem.
//!
//! groundSpring provides the statistical building blocks for decomposing
//! measurement error into correctable bias and irreducible noise across
//! scientific domains.
//!
//! ## Modules
//!
//! - [`stats`] — Core statistical metrics (RMSE, MBE, R², IA, hit rate, moving window)
//! - [`decompose`] — Bias-variance error decomposition
//! - [`drift`] — Drift vs selection in finite populations (Wright-Fisher)
//! - [`prng`] — Deterministic pseudo-random number generation
//! - [`rare_biosphere`] — Rare biosphere detection (Chao1, detection power)
//! - [`rarefaction`] — Multinomial rarefaction for sequencing noise analysis
//! - [`seismic`] — Travel-time computation and source inversion
//! - [`spectral_recon`] — Spectral function reconstruction (Tikhonov regularization)
//! - [`fao56`] — FAO-56 Penman-Monteith reference evapotranspiration
//! - [`freeze_out`] — Freeze-out curve chi-squared fitting (2D grid search)
//! - [`kinetics`] — Hill-function kinetics (shared by bistable + multi-signal)
//! - [`ode`] — Generic RK4 integrator for fixed-size ODE systems
//! - [`gillespie`] — Gillespie SSA for stochastic chemical kinetics
//! - [`jackknife`] — Delete-one and block jackknife resampling
//! - [`bootstrap`] — Bootstrap and RAWR resampling confidence intervals
//! - [`anderson`] — Anderson localization / Lyapunov exponents (1D, 2D, 3D)
//! - [`almost_mathieu`] — Almost-Mathieu quasiperiodic localization / level spacing
//! - [`band_structure`] — Band structure of periodic tight-binding chains
//! - [`bistable`] — Bistable phenotypic switching (c-di-GMP circuit)
//! - [`esn`] — Echo State Network regime classifier (cross-spring: hotSpring ESN)
//! - [`multisignal`] — Multi-signal QS integration (CAI-1 + AI-2)
//! - [`quasispecies`] — Eigen quasispecies model and error threshold
//! - [`tissue_anderson`] — Anderson localization in tissue geometry (Paper 12 immunological)
//! - [`transport`] — Wavepacket transport in tight-binding chains
//! - [`wdm`] — Warm Dense Matter transport analysis (Green-Kubo, finite-size extrapolation)
//! - `lanczos` — Lanczos eigensolver for large sparse systems (behind `barracuda-gpu`)
//! - `biomeos` — biomeOS Neural API client (behind `biomeos` feature)
//! - `npu` — NPU integration for Akida neuromorphic inference (behind `npu` feature)
//! - [`error`] — Typed input validation errors (`InputError`)
//! - [`linalg`] — Linear algebra primitives (tridiagonal eigensolver)
//! - [`validate`] — Validation harness (pass/fail with counters)

pub mod error;

pub mod almost_mathieu;
pub mod anderson;
// NOTE: `linalg` is intentionally listed before its consumers (transport,
// band_structure) to make the dependency direction visible in `lib.rs`.
// `transport` re-exports `linalg::{tridiag_eigh, EighError}` for backward
// compatibility; new code should prefer `linalg::` directly.
pub mod band_structure;
pub mod bistable;
pub mod bootstrap;
pub mod decompose;
pub mod drift;
pub mod esn;
pub mod fao56;
pub mod freeze_out;
pub mod gillespie;
pub mod jackknife;
pub mod kinetics;
pub mod linalg;
pub mod multisignal;
pub mod niche;
pub mod ode;
pub mod primal_names;
pub mod prng;
pub mod quasispecies;
pub mod rare_biosphere;
pub mod rarefaction;
pub mod seismic;
pub mod spectral_recon;
pub mod stats;
pub mod tissue_anderson;
pub mod transport;
pub mod validate;
pub mod wdm;

#[cfg(feature = "barracuda-gpu")]
pub mod lanczos;

#[cfg(feature = "barracuda-gpu")]
pub(crate) mod gpu;

/// Returns `true` when the `barracuda-gpu` feature is enabled *and* a GPU device
/// is available at runtime. Always returns `false` when compiled without
/// `barracuda-gpu`.
#[cfg_attr(
    not(feature = "barracuda-gpu"),
    expect(
        clippy::missing_const_for_fn,
        reason = "const only in non-GPU builds; runtime probe with barracuda-gpu"
    )
)]
#[must_use]
pub fn gpu_available() -> bool {
    #[cfg(feature = "barracuda-gpu")]
    {
        gpu::get_device().is_some()
    }
    #[cfg(not(feature = "barracuda-gpu"))]
    {
        false
    }
}

/// Hardware-aware precision routing advice for GPU dispatch paths.
///
/// Queries the barraCuda `GpuDriverProfile` to determine which f64 precision
/// strategy is safe for the detected hardware. Returns `None` when compiled
/// without `barracuda-gpu` or when no GPU is available.
///
/// See [`barracuda::device::driver_profile::PrecisionRoutingAdvice`] for
/// the four routing tiers (`F64Native`, `F64NativeNoSharedMem`, `Df64Only`, `F32Only`).
#[cfg(feature = "barracuda-gpu")]
#[must_use]
pub fn gpu_precision_routing() -> Option<gpu::PrecisionRoutingAdvice> {
    gpu::precision_routing()
}

#[cfg(feature = "biomeos")]
pub mod biomeos;

#[cfg(feature = "biomeos")]
pub mod dispatch;

#[cfg(feature = "biomeos")]
pub mod nestgate;

#[cfg(feature = "biomeos")]
pub mod provenance;

#[cfg(feature = "npu")]
pub mod npu;

#[cfg(feature = "tarpc-ipc")]
pub mod ipc;

/// Re-export of the Nautilus Shell evolutionary reservoir computing crate.
///
/// The Nautilus Shell (`bingoCube/nautilus`) is a feed-forward reservoir that
/// uses evolutionary board populations instead of temporal recurrence (ESN).
/// Key types: `NautilusBrain`, `NautilusShell`, `DriftMonitor`, `EdgeSeeder`.
///
/// Enable with `--features nautilus`.
///
/// # Cross-spring lineage
///
/// `primalTools/bingoCube/nautilus` — hotSpring Exp 024+028 QCD phase boundary
/// prediction (5.3% LOO error, 540× cost reduction via quenched→dynamical
/// transfer). The shell is portable, serializable to JSON, and mergeable
/// across instances.
#[cfg(feature = "nautilus")]
pub use bingocube_nautilus as nautilus;

/// Centralized numeric cast helpers.
///
/// `usize` and `u64` → `f64` conversions are unavoidable in numerical code
/// (Rust has no `From<usize>` for `f64` because usize may be 64-bit).
/// These helpers document the safety argument once and keep cast lints
/// targeted rather than blanket-allowed.
pub(crate) mod cast {
    /// Convert a collection length (`usize`) to `f64`.
    ///
    /// Exact for lengths up to 2^53 (≈ 9 × 10¹⁵), far beyond practical memory.
    #[inline]
    #[expect(clippy::cast_precision_loss, reason = "exact for lengths up to 2^53")]
    pub const fn usize_f64(n: usize) -> f64 {
        n as f64
    }

    /// Convert a `u64` count to `f64`.
    ///
    /// Exact for values up to 2^53.  Used in rarefaction and PRNG where
    /// counts are sequencing depths or taxonomic totals.
    #[inline]
    #[expect(clippy::cast_precision_loss, reason = "exact for values up to 2^53")]
    pub const fn u64_f64(n: u64) -> f64 {
        n as f64
    }

    /// Convert a non-negative `f64` to `usize` (truncating toward zero).
    ///
    /// Used for index computation from floating-point rank/position values.
    #[inline]
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "callers ensure x is non-negative and within usize range"
    )]
    pub const fn f64_usize(x: f64) -> usize {
        x as usize
    }
}

/// Shared tolerance constants for validation assertions.
///
/// Use these named constants instead of bare float literals in tests and
/// validation code. Each tier corresponds to a specific numerical regime:
/// - **DETERMINISM** — bitwise reproducibility (1e-15)
/// - **STRICT** — summation with extended precision (1e-14)
/// - **EXACT** — summation-only paths (1e-12)
/// - **ANALYTICAL** — one transcendental (sqrt, ln) (1e-10)
/// - **INTEGRATION** — ODE RK4 accumulation (1e-8)
/// - **`CDF_APPROX`** — CDF/erf approximation (1e-6)
/// - **RECONSTRUCTION** — spectral Tikhonov roundtrip (1e-4)
/// - **LITERATURE** — published 3–4 sig figs (0.001)
/// - **DECOMPOSITION** — bias-variance fractions (0.005)
/// - **STOCHASTIC** — O(1/√N) mean estimator (0.01)
/// - **`NORM_2PCT`** — ~2% normalization (0.02)
/// - **EQUILIBRIUM** — ODE equilibrium / measurement (0.1)
pub mod tol {
    // Each tolerance is a named tier with provenance: why this value,
    // what math governs it, and where it was validated.

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
}

/// Production epsilon guards (division safety, underflow, SSA floor).
/// Test tolerances live in [`tol`].
pub(crate) mod eps {
    /// Division-safe epsilon (avoid NaN in `x / y.max(eps::SAFE_DIV)`).
    pub const SAFE_DIV: f64 = 1e-10;
    /// Gillespie SSA steady-state guard (~10× `f64::EPSILON`).
    #[cfg(feature = "barracuda-gpu")]
    pub const SSA_FLOOR: f64 = 1e-15;
    /// Near-zero guard for log/entropy computations.
    ///
    /// Probabilities below this threshold are treated as zero in entropy
    /// sums to avoid `-0 × log(0)` NaN. Also used for coefficient-of-variation
    /// denominators in multi-head uncertainty measurement.
    pub const LOG_FLOOR: f64 = 1e-15;
    /// Underflow guard for condition number / matrix element magnitude.
    ///
    /// Used by [`crate::linalg`] QL iteration to detect near-zero off-diagonal
    /// elements that would cause division overflow in implicit shift computation.
    pub const UNDERFLOW: f64 = 1e-300;
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;

    #[test]
    fn gpu_available_without_feature_is_false() {
        let available = gpu_available();
        #[cfg(not(feature = "barracuda-gpu"))]
        assert!(!available);
        #[cfg(feature = "barracuda-gpu")]
        let _ = available;
    }

    #[test]
    fn cast_usize_f64_exact_for_small() {
        assert!((cast::usize_f64(0) - 0.0).abs() < f64::EPSILON);
        assert!((cast::usize_f64(1) - 1.0).abs() < f64::EPSILON);
        assert!((cast::usize_f64(1_000_000) - 1e6).abs() < f64::EPSILON);
    }

    #[test]
    fn cast_u64_f64_exact_for_small() {
        assert!((cast::u64_f64(0) - 0.0).abs() < f64::EPSILON);
        assert!((cast::u64_f64(42) - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn cast_f64_usize_truncates() {
        assert_eq!(cast::f64_usize(3.7), 3);
        assert_eq!(cast::f64_usize(0.0), 0);
        assert_eq!(cast::f64_usize(100.999), 100);
    }
}
