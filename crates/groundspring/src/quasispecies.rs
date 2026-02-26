// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Eigen quasispecies model and error threshold.
//!
//! Implements the single-peak fitness landscape quasispecies model
//! (Eigen 1971) to study the **error threshold**: the mutation rate
//! above which self-replicating information is destroyed by noise.
//!
//! This is groundSpring's most fundamental signal-vs-noise experiment:
//! at what mutation rate does heritable information (signal) collapse
//! into random noise?
//!
//! # Key functions
//!
//! - [`error_threshold`] — analytical `μ_c` = 1 − σ^(−1/L)
//! - [`master_frequency_analytical`] — steady-state `x_m`
//! - [`quasispecies_simulation`] — stochastic Wright-Fisher + mutation
//!
//! # References
//!
//! - Dolson et al. (2023) J R Soc Interface 20(208)
//! - Eigen (1971) Naturwiss 58:465-523
//! - Eigen & Schuster (1977) Naturwiss 64:541-565

use crate::cast::usize_f64;
use crate::prng::Xorshift64;

/// Analytical error threshold for the single-peak landscape.
///
/// `μ_c = 1 − σ^(−1/L)`
///
/// Below this mutation rate, the master sequence maintains a stable
/// subpopulation. Above it, the population randomizes.
#[must_use]
pub fn error_threshold(sigma: f64, genome_length: usize) -> f64 {
    1.0 - sigma.powf(-1.0 / usize_f64(genome_length))
}

/// Analytical steady-state master sequence frequency.
///
/// `x_m = max(0, (σ·Q − 1) / (σ − 1))`
///
/// where `Q = (1 − μ)^L` is the per-genome copying fidelity.
#[must_use]
pub fn master_frequency_analytical(sigma: f64, mu: f64, genome_length: usize) -> f64 {
    let q = (1.0 - mu).powf(usize_f64(genome_length));
    let x_m = sigma.mul_add(q, -1.0) / (sigma - 1.0);
    x_m.max(0.0)
}

/// Simulate quasispecies dynamics on a single-peak fitness landscape.
///
/// Returns a vector of master sequence frequencies, one per generation.
///
/// Each individual is either "master" (fitness `sigma`) or "mutant"
/// (fitness 1). Selection uses fitness-proportionate sampling;
/// mutation converts master→mutant with probability `1 − Q` where
/// `Q = (1 − mu)^L`. Back-mutation is neglected (exponentially rare
/// for large L).
///
/// # Panics
///
/// Panics if `pop_size` is zero.
#[must_use]
pub fn quasispecies_simulation(
    pop_size: usize,
    genome_length: usize,
    sigma: f64,
    mu: f64,
    n_generations: usize,
    seed: u64,
) -> Vec<f64> {
    assert!(pop_size > 0, "pop_size must be positive");

    let q = (1.0 - mu).powf(usize_f64(genome_length));
    let pop_f = usize_f64(pop_size);
    let mut rng = Xorshift64::new(seed);
    let mut n_master = pop_size / 2;

    let mut freqs = Vec::with_capacity(n_generations);
    for _ in 0..n_generations {
        let freq = usize_f64(n_master) / pop_f;
        freqs.push(freq);

        #[expect(clippy::cast_precision_loss)]
        let n_master_f = n_master as f64;
        #[expect(clippy::cast_precision_loss)]
        let n_mutant_f = (pop_size - n_master) as f64;
        let fitness_total = sigma.mul_add(n_master_f, n_mutant_f);
        let p_master = (sigma * n_master_f) / fitness_total;

        let n_selected = rng.binomial(pop_size, p_master);
        #[expect(clippy::cast_possible_truncation)]
        {
            n_master = rng.binomial(n_selected as usize, q) as usize;
        }
    }

    freqs
}

/// Compute mean fitness from master frequency.
///
/// `φ = σ·x_m + 1·(1 − x_m) = 1 + (σ − 1)·x_m`
#[must_use]
pub fn mean_fitness(sigma: f64, master_freq: f64) -> f64 {
    sigma.mul_add(master_freq, 1.0 - master_freq)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_threshold_known_value() {
        let mu_c = error_threshold(10.0, 100);
        assert!(
            (mu_c - 0.02276).abs() < 0.001,
            "sigma=10, L=100 → mu_c ≈ 0.02276, got {mu_c}"
        );
    }

    #[test]
    fn master_freq_below_threshold() {
        let x_m = master_frequency_analytical(10.0, 0.01, 100);
        assert!(x_m > 0.1, "below threshold, master should survive: {x_m}");
    }

    #[test]
    fn master_freq_above_threshold() {
        let x_m = master_frequency_analytical(10.0, 0.04, 100);
        assert!(
            x_m < f64::EPSILON,
            "above threshold, master should vanish: {x_m}"
        );
    }

    #[test]
    fn master_freq_at_zero_mutation() {
        let x_m = master_frequency_analytical(10.0, 0.0, 100);
        assert!((x_m - 1.0).abs() < 1e-10, "zero mutation → x_m = 1");
    }

    #[test]
    fn error_threshold_increases_with_sigma() {
        let mu_c_low = error_threshold(2.0, 100);
        let mu_c_high = error_threshold(20.0, 100);
        assert!(
            mu_c_high > mu_c_low,
            "higher fitness advantage allows more mutation"
        );
    }

    #[test]
    fn simulation_deterministic() {
        let f1 = quasispecies_simulation(500, 50, 10.0, 0.01, 100, 42);
        let f2 = quasispecies_simulation(500, 50, 10.0, 0.01, 100, 42);
        assert_eq!(f1, f2, "same seed must give same trajectory");
    }

    #[test]
    fn simulation_below_threshold_master_survives() {
        let freqs = quasispecies_simulation(2000, 100, 10.0, 0.01, 500, 42);
        let tail_mean: f64 = freqs[250..].iter().sum::<f64>() / 250.0;
        assert!(
            tail_mean > 0.05,
            "below threshold, master should have significant frequency: {tail_mean}"
        );
    }

    #[test]
    fn simulation_above_threshold_master_lost() {
        let freqs = quasispecies_simulation(2000, 100, 10.0, 0.04, 500, 42);
        let tail_mean: f64 = freqs[250..].iter().sum::<f64>() / 250.0;
        assert!(
            tail_mean < 0.02,
            "above threshold, master should be rare: {tail_mean}"
        );
    }

    #[test]
    fn mean_fitness_correct() {
        assert!((mean_fitness(10.0, 0.5) - 5.5).abs() < 1e-10);
        assert!((mean_fitness(10.0, 0.0) - 1.0).abs() < 1e-10);
        assert!((mean_fitness(10.0, 1.0) - 10.0).abs() < 1e-10);
    }
}
