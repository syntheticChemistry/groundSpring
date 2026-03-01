// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

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
pub mod ode;
pub mod prng;
pub mod quasispecies;
pub mod rare_biosphere;
pub mod rarefaction;
pub mod seismic;
pub mod spectral_recon;
pub mod stats;
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
#[allow(clippy::missing_const_for_fn)] // const only in non-GPU builds; runtime probe with barracuda-gpu
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

#[cfg(feature = "biomeos")]
pub mod biomeos;

#[cfg(feature = "biomeos")]
pub mod nestgate;

#[cfg(feature = "npu")]
pub mod npu;

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
    #[expect(clippy::cast_precision_loss)]
    pub const fn usize_f64(n: usize) -> f64 {
        n as f64
    }

    /// Convert a `u64` count to `f64`.
    ///
    /// Exact for values up to 2^53.  Used in rarefaction and PRNG where
    /// counts are sequencing depths or taxonomic totals.
    #[inline]
    #[expect(clippy::cast_precision_loss)]
    pub const fn u64_f64(n: u64) -> f64 {
        n as f64
    }

    /// Convert a non-negative `f64` to `usize` (truncating toward zero).
    ///
    /// Used for index computation from floating-point rank/position values.
    #[inline]
    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub const fn f64_usize(x: f64) -> usize {
        x as usize
    }
}

#[cfg(test)]
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
