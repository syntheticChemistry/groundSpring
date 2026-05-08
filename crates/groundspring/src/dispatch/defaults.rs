// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Named dispatch defaults for RPC parameter resolution.
//!
//! RPC callers may omit optional parameters; these defaults mirror the
//! Bazavov et al. (2016) benchmark configuration and standard spectral
//! analysis conventions so that a bare `capability.call` returns sensible
//! results without requiring the caller to know domain physics.

/// Default Tikhonov regularization for spectral feature extraction.
///
/// Balances noise suppression against spectral peak resolution
/// in the correlator → spectral-function inversion (Exp 028).
/// Matches the `tol::RECONSTRUCTION` tier (1e-4).
pub(super) const DEFAULT_REGULARIZATION: f64 = crate::tol::RECONSTRUCTION;

/// Default time-step spacing for correlator τ grid (spectral features).
///
/// 0.1 matches the Euclidean-time lattice spacing convention in
/// hotSpring Exp 015/022 and barraCuda benchmark correlator data.
pub(super) const DEFAULT_TAU_STEP: f64 = 0.1;

/// Default angular frequency spacing for spectral ω grid.
///
/// 0.2 provides sufficient resolution for Matsubara peak detection
/// while keeping the kernel matrix well-conditioned.
pub(super) const DEFAULT_OMEGA_STEP: f64 = 0.2;

/// Default measurement uncertainty σ for freeze-out fits.
pub(super) const DEFAULT_SIGMA: f64 = 1.0;

/// Default T₀ grid lower bound (in `MeV`) — Bazavov et al. (2016).
pub(super) const DEFAULT_T0_LO: f64 = 100.0;
/// Default T₀ grid upper bound (in `MeV`).
pub(super) const DEFAULT_T0_HI: f64 = 200.0;
/// Default T₀ grid step size (in `MeV`).
pub(super) const DEFAULT_T0_STEP: f64 = 1.0;
/// Default κ₂ grid lower bound — Bazavov et al. (2016).
pub(super) const DEFAULT_K2_LO: f64 = 0.001;
/// Default κ₂ grid upper bound.
pub(super) const DEFAULT_K2_HI: f64 = 0.05;
/// Default κ₂ grid step size.
pub(super) const DEFAULT_K2_STEP: f64 = 0.001;

/// Default centre energy for Anderson validation (mid-band).
pub(super) const DEFAULT_ENERGY: f64 = 0.0;

/// Default bootstrap confidence level (95th percentile).
pub(super) const DEFAULT_CONFIDENCE: f64 = 0.95;

/// Default station elevation (sea level, metres) for FAO-56 ET₀.
pub(super) const DEFAULT_ELEVATION_M: f64 = 0.0;

/// Default maximum relative humidity (%) — typical humid climate.
pub(super) const DEFAULT_RHMAX_PCT: f64 = 80.0;

/// Default minimum relative humidity (%) — typical daytime drop.
pub(super) const DEFAULT_RHMIN_PCT: f64 = 40.0;

/// Default margin for rule-based regime classification (spacing-ratio window).
pub(super) const DEFAULT_REGIME_MARGIN: f64 = 0.1;

/// Default reproducibility seed for stochastic methods.
///
/// The answer to the Ultimate Question — ensures deterministic results
/// when callers omit the seed parameter, matching the convention used
/// across hotSpring, wetSpring, and airSpring dispatch layers.
pub(super) const DEFAULT_SEED: u64 = 42;

/// Default Anderson lattice size — 10 000 sites.
///
/// Provenance: standard 1D lattice length in Kachkovskiy (Paper 2)
/// and Anderson localization finite-size scaling studies. Balances
/// accuracy with sub-second evaluation on CPU.
pub(super) const DEFAULT_ANDERSON_N_SITES: u64 = 10_000;

/// Default Anderson disorder strength W = 4.0.
///
/// Provenance: W = 4.0 sits in the strongly localized regime for 1D
/// Anderson (all states are localized for W > 0), producing
/// localization lengths accessible to finite-size lattices.
/// Validated: hotSpring Exp 015 disorder sweeps, groundSpring Exp 031.
pub(super) const DEFAULT_ANDERSON_DISORDER: f64 = 4.0;

/// Default number of disorder realizations for Anderson averaging.
///
/// 20 realizations balances statistical averaging with evaluation time.
/// Provenance: finite-size analysis convention in Papers 2 & 3.
pub(super) const DEFAULT_ANDERSON_REALIZATIONS: u64 = 20;

/// Default bootstrap replicate count — 10 000.
///
/// Standard recommendation (Efron & Tibshirani 1993) for percentile-
/// bootstrap CIs with moderate sample sizes.
pub(super) const DEFAULT_N_BOOTSTRAP: u64 = 10_000;

/// Default spectral ω grid size — 50 points.
///
/// Sufficient resolution for Matsubara peak detection in lattice QCD
/// correlator spectral reconstruction (Exp 028).
pub(super) const DEFAULT_N_OMEGA: u64 = 50;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confidence_in_unit_interval() {
        assert!(DEFAULT_CONFIDENCE > 0.0 && DEFAULT_CONFIDENCE < 1.0);
    }

    #[test]
    fn grid_bounds_ordered() {
        assert!(DEFAULT_T0_LO < DEFAULT_T0_HI);
        assert!(DEFAULT_K2_LO < DEFAULT_K2_HI);
    }

    #[test]
    fn grid_steps_positive() {
        assert!(DEFAULT_T0_STEP > 0.0);
        assert!(DEFAULT_K2_STEP > 0.0);
        assert!(DEFAULT_TAU_STEP > 0.0);
        assert!(DEFAULT_OMEGA_STEP > 0.0);
    }

    #[test]
    fn rh_defaults_physical() {
        assert!(DEFAULT_RHMIN_PCT < DEFAULT_RHMAX_PCT);
        assert!(DEFAULT_RHMAX_PCT <= 100.0);
    }
}
