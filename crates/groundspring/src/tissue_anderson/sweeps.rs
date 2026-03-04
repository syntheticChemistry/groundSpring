// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Parameter sweep functions for tissue Anderson experiments.
//!
//! Barrier disruption and dimensional duality sweeps for Paper 12 §2.4,
//! quantifying the promotion–collapse duality between tillage and scratching.

use crate::anderson::{anderson_potential, lyapunov_exponent};
use crate::cast::usize_f64;
use crate::prng::Xorshift64;

use super::compartments::{disrupted_epidermis, inflamed_dermis};
use super::{simulate_tissue, xi_from_gamma};

/// A single point in the barrier disruption sweep.
#[derive(Debug, Clone, Copy)]
pub struct BarrierSweepPoint {
    /// Fraction of epidermis disrupted (0.0 = healthy, 1.0 = fully breached).
    pub breach_fraction: f64,
    /// Effective dimensionality of epidermis.
    pub d_eff_epidermis: f64,
    /// Lyapunov exponent in epidermis.
    pub gamma_epidermis: f64,
    /// Lyapunov exponent in dermis.
    pub gamma_dermis: f64,
    /// Localization length in epidermis.
    pub xi_epidermis: f64,
    /// Localization length in dermis.
    pub xi_dermis: f64,
    /// Whether the barrier is considered breached (`d_eff > 2.5`).
    pub barrier_breached: bool,
    /// Whether cytokine signal can cross from dermis through breached epidermis.
    pub signal_crosses_barrier: bool,
}

/// Sweep barrier disruption fraction from 0 (healthy) to 1 (fully breached).
///
/// At each disruption level, simulates a two-compartment system (epidermis +
/// dermis) and tracks how `d_eff`, localization length, and signal propagation
/// evolve. This quantifies the dimensional promotion threshold.
#[must_use]
pub fn barrier_disruption_sweep(
    n_points: usize,
    n_realizations: usize,
    base_seed: u64,
) -> Vec<BarrierSweepPoint> {
    (0..n_points)
        .map(|i| {
            let frac = if n_points > 1 {
                usize_f64(i) / usize_f64(n_points - 1)
            } else {
                0.0
            };
            let epi = disrupted_epidermis(frac);
            let derm = inflamed_dermis();
            let result =
                simulate_tissue(&[epi, derm], n_realizations, base_seed + (i as u64) * 100);
            BarrierSweepPoint {
                breach_fraction: frac,
                d_eff_epidermis: result.d_eff_system.min(3.0),
                gamma_epidermis: result.gamma_per_compartment[0],
                gamma_dermis: result.gamma_per_compartment[1],
                xi_epidermis: result.xi_per_compartment[0],
                xi_dermis: result.xi_per_compartment[1],
                barrier_breached: result.barrier_breached,
                signal_crosses_barrier: result.barrier_breached && result.signal_extended[0],
            }
        })
        .collect()
}

/// A point in the dimensional duality sweep.
#[derive(Debug, Clone, Copy)]
pub struct DualityPoint {
    /// Sweep parameter: -1 (collapse) to +1 (promotion).
    pub parameter: f64,
    /// Effective dimensionality.
    pub d_eff: f64,
    /// Lyapunov exponent.
    pub gamma: f64,
    /// Localization length.
    pub xi: f64,
    /// Anderson regime: "localized" or "extended".
    pub regime: &'static str,
    /// Physical context: "collapse (tillage)" or "promotion (scratching)".
    pub context: &'static str,
}

/// Dimensional promotion sweep for Paper 12 §2.4.
///
/// Computes the duality between Paper 06 (tillage collapse) and Paper 12
/// (scratching promotion) by sweeping a parameter from -1 (full collapse)
/// through 0 (neutral) to +1 (full promotion).
#[must_use]
pub fn dimensional_duality_sweep(
    n_points: usize,
    n_realizations: usize,
    base_seed: u64,
) -> Vec<DualityPoint> {
    (0..n_points)
        .map(|i| {
            let param = if n_points > 1 {
                (usize_f64(i) / usize_f64(n_points - 1)).mul_add(2.0, -1.0)
            } else {
                0.0
            };
            let d_eff = (2.5 + param * 0.5).clamp(2.0, 3.0);
            let n_sites = 500;
            let w = 2.0;
            let mut rng = Xorshift64::new(base_seed + i as u64);

            let mut gamma_sum = 0.0;
            for _ in 0..n_realizations {
                let seed = rng.next_u64();
                let potential = anderson_potential(n_sites, w, seed);
                gamma_sum += lyapunov_exponent(&potential, 0.0);
            }
            let gamma = gamma_sum / usize_f64(n_realizations);
            let xi = xi_from_gamma(gamma);

            DualityPoint {
                parameter: param,
                d_eff,
                gamma,
                xi,
                regime: if d_eff < 2.5 { "localized" } else { "extended" },
                context: if param < 0.0 {
                    "collapse (tillage)"
                } else {
                    "promotion (scratching)"
                },
            }
        })
        .collect()
}
