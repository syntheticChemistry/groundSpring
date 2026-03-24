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
pub mod rawr;
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

#[cfg(any(feature = "tarpc-ipc", test))]
mod ipc_error;

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

pub mod cast;
pub(crate) mod eps;
pub mod provenance_registry;
pub mod tol;

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
}
