// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Anderson localization in 1D tight-binding models (Exp 008).
//!
//! ```text
//! H ψ(n) = ψ(n+1) + ψ(n-1) + V(n) ψ(n)
//! ```
//!
//! where `V(n)` is a random potential drawn uniformly from `[-W/2, W/2]`.
//! In 1D, Anderson (1958) proved that ALL states are localized for any
//! disorder `W > 0`.  The localization length `ξ ~ C / W²` at the band
//! center `E = 0` (Thouless 1972, Derrida-Gardner).
//!
//! # barracuda delegation
//!
//! When the `barracuda-gpu` feature is enabled, [`lyapunov_exponent`] and
//! [`lyapunov_averaged`] delegate to `barracuda::spectral::lyapunov_*`.
//!
//! **Seed convention:** the local CPU implementation uses `base_seed + i`
//! per realization; barracuda uses `base_seed + r * 1000`.  Results diverge
//! when the feature gate is active — this is expected and documented in
//! `specs/BARRACUDA_EVOLUTION.md` as Phase 2b alignment work.

use crate::prng::Xorshift64;

/// Derrida-Gardner constant for ξ ≈ C / W² at band center.
#[cfg(not(feature = "barracuda"))]
const DERRIDA_GARDNER_CONSTANT: f64 = 96.0;

/// Generate a random potential for the 1D Anderson model.
///
/// Each site gets `V(n) ~ Uniform[-W/2, W/2]` where `W = disorder`.
/// Returns the zero vector for `disorder <= 0`.
#[must_use]
pub fn anderson_potential(n: usize, disorder: f64, seed: u64) -> Vec<f64> {
    if disorder <= 0.0 {
        return vec![0.0; n];
    }
    let half_w = disorder / 2.0;
    let mut rng = Xorshift64::new(seed);
    (0..n)
        .map(|_| rng.next_f64().mul_add(disorder, -half_w))
        .collect()
}

/// Compute the Lyapunov exponent for a given potential and energy.
///
/// When `barracuda-gpu` feature is enabled, delegates to
/// `barracuda::spectral::lyapunov_exponent`.
///
/// Uses the transfer-matrix method with vector renormalization to avoid
/// overflow.  The transfer matrix at site `n` is:
///
/// ```text
/// T_n = [[E - V(n), -1],
///        [1,         0]]
/// ```
///
/// Returns `γ = (1/N) Σ ln(norm)`, the largest Lyapunov exponent.
#[must_use]
pub fn lyapunov_exponent(potential: &[f64], energy: f64) -> f64 {
    #[cfg(feature = "barracuda-gpu")]
    return barracuda::spectral::lyapunov_exponent(potential, energy);
    #[cfg(not(feature = "barracuda-gpu"))]
    lyapunov_exponent_cpu(potential, energy)
}

#[cfg(not(feature = "barracuda-gpu"))]
fn lyapunov_exponent_cpu(potential: &[f64], energy: f64) -> f64 {
    let n = potential.len();
    if n == 0 {
        return 0.0;
    }

    let mut log_growth = 0.0;
    let mut v0: f64 = 1.0;
    let mut v1: f64 = 0.0;

    for &v in potential {
        let new_0 = (energy - v).mul_add(v0, -v1);
        let new_1 = v0;
        v0 = new_0;
        v1 = new_1;

        let norm = v0.hypot(v1);
        if norm > 0.0 {
            log_growth += norm.ln();
            v0 /= norm;
            v1 /= norm;
        }
    }

    log_growth / crate::cast::usize_f64(n)
}

/// Localization length `ξ = 1 / γ`.  Returns `f64::INFINITY` if `γ <= 0`.
#[must_use]
pub fn localization_length(gamma: f64) -> f64 {
    if gamma <= 0.0 {
        return f64::INFINITY;
    }
    1.0 / gamma
}

/// Analytical localization length from disorder strength and energy.
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::special::anderson_transport::localization_length` which uses
/// the perturbative result `ξ(W, E) ≈ 105.2 / W²` at the band center.
/// The local fallback uses `ξ ≈ C / W²` with `C ≈ 96` from Derrida-Gardner.
///
/// This is an analytical estimate — for numerical results, use
/// [`lyapunov_exponent`] + [`localization_length`].
#[must_use]
pub fn analytical_localization_length(disorder: f64, energy: f64) -> f64 {
    #[cfg(feature = "barracuda")]
    return barracuda::special::anderson_transport::localization_length(disorder, energy);
    #[cfg(not(feature = "barracuda"))]
    analytical_localization_length_cpu(disorder, energy)
}

#[cfg(not(feature = "barracuda"))]
fn analytical_localization_length_cpu(disorder: f64, energy: f64) -> f64 {
    if disorder <= 0.0 {
        return f64::INFINITY;
    }
    let _ = energy;
    DERRIDA_GARDNER_CONSTANT / (disorder * disorder)
}

/// Average Lyapunov exponent over many disorder realizations.
///
/// When `barracuda-gpu` feature is enabled, delegates to
/// `barracuda::spectral::lyapunov_averaged`.  Note: barracuda uses
/// `base_seed + r * 1000` per realization; the local fallback uses
/// `base_seed + i` — results will differ across feature gates until
/// Phase 2b PRNG alignment.
#[must_use]
pub fn lyapunov_averaged(
    n_sites: usize,
    disorder: f64,
    energy: f64,
    n_realizations: usize,
    base_seed: u64,
) -> f64 {
    #[cfg(feature = "barracuda-gpu")]
    return barracuda::spectral::lyapunov_averaged(
        n_sites,
        disorder,
        energy,
        n_realizations,
        base_seed,
    );
    #[cfg(not(feature = "barracuda-gpu"))]
    lyapunov_averaged_cpu(n_sites, disorder, energy, n_realizations, base_seed)
}

#[cfg(not(feature = "barracuda-gpu"))]
fn lyapunov_averaged_cpu(
    n_sites: usize,
    disorder: f64,
    energy: f64,
    n_realizations: usize,
    base_seed: u64,
) -> f64 {
    let mut total = 0.0;
    for i in 0..n_realizations {
        let pot = anderson_potential(n_sites, disorder, base_seed + i as u64);
        total += lyapunov_exponent(&pot, energy);
    }
    total / crate::cast::usize_f64(n_realizations)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_system_zero_lyapunov() {
        let pot = anderson_potential(10000, 0.0, 42);
        let gamma = lyapunov_exponent(&pot, 0.0);
        // W=0 gives deterministic transfer matrix (identity-like); γ=0 exactly; 0.001 absorbs any residual numerical drift.
        assert!(gamma.abs() < 0.001, "clean system γ={gamma}, expected ~0");
    }

    #[test]
    fn disorder_gives_positive_lyapunov() {
        let gamma = lyapunov_averaged(10000, 2.0, 0.0, 10, 42);
        assert!(
            gamma > 0.0,
            "disordered system should have γ > 0, got {gamma}"
        );
    }

    #[test]
    fn lyapunov_increases_with_disorder() {
        let g1 = lyapunov_averaged(10000, 1.0, 0.0, 10, 42);
        let g2 = lyapunov_averaged(10000, 4.0, 0.0, 10, 42);
        assert!(g2 > g1, "γ(W=4)={g2} should exceed γ(W=1)={g1}");
    }

    #[test]
    fn localization_length_decreases_with_disorder() {
        let g1 = lyapunov_averaged(10000, 1.0, 0.0, 10, 42);
        let g2 = lyapunov_averaged(10000, 4.0, 0.0, 10, 42);
        let xi1 = localization_length(g1);
        let xi2 = localization_length(g2);
        assert!(xi2 < xi1, "ξ(W=4)={xi2} should be less than ξ(W=1)={xi1}");
    }

    #[test]
    fn potential_deterministic() {
        let p1 = anderson_potential(100, 2.0, 42);
        let p2 = anderson_potential(100, 2.0, 42);
        assert_eq!(p1, p2);
    }

    #[test]
    fn potential_different_seed() {
        let p1 = anderson_potential(100, 2.0, 42);
        let p2 = anderson_potential(100, 2.0, 99);
        assert_ne!(p1, p2);
    }

    #[test]
    fn analytical_localization_length_decreases_with_disorder() {
        let xi1 = analytical_localization_length(1.0, 0.0);
        let xi2 = analytical_localization_length(2.0, 0.0);
        assert!(xi2 < xi1, "ξ(W=2)={xi2} should be less than ξ(W=1)={xi1}");
    }

    #[test]
    fn analytical_localization_length_clean_system_large() {
        let xi = analytical_localization_length(0.0, 0.0);
        assert!(
            xi > 1000.0 || xi.is_infinite(),
            "clean system should have ξ ≫ lattice constant, got {xi}"
        );
    }

    #[test]
    fn analytical_vs_numerical_same_order() {
        let g = lyapunov_averaged(100_000, 1.0, 0.0, 20, 42);
        let xi_numerical = localization_length(g);
        let xi_analytical = analytical_localization_length(1.0, 0.0);
        assert!(
            xi_analytical > 10.0 && xi_numerical > 10.0,
            "both should be large: analytical={xi_analytical}, numerical={xi_numerical}"
        );
    }

    #[test]
    fn thouless_scaling() {
        let g = lyapunov_averaged(100_000, 1.0, 0.0, 20, 42);
        let xi = localization_length(g);
        let c = xi * 1.0_f64.powi(2);
        // Derrida-Gardner C≈96; 20 realizations give ~√20 sample variance; [60,140] is ~±45% around 96.
        assert!(
            (60.0..140.0).contains(&c),
            "Thouless coefficient C={c}, expected ~96"
        );
    }
}
