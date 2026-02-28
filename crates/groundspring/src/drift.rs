// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Drift vs selection in finite populations (Wright-Fisher model).
//!
//! Implements the Wright-Fisher model for population genetics to study
//! when stochastic drift overwhelms deterministic selection. The key
//! parameter is N×s — the product of effective population size and
//! selection coefficient.
//!
//! # References
//!
//! - Anderson (2022) mBio 13:e00354-22
//! - Kimura (1968) Nature 217:624-626
//! - Wright (1931) Genetics 16:97-159
//!
//! # barracuda delegation
//!
//! [`kimura_fixation_prob`] is a pending delegation target for
//! `barracuda::stats::kimura_fixation` — not yet in barracuda as of S68+.
//! [`wright_fisher_fixation`] remains local (serial RNG loop); `ToadStool`
//! S66+ has `WrightFisherGpu` for per-generation GPU dispatch but no
//! multi-trial wrapper.

use crate::cast::usize_f64;
use crate::prng::Xorshift64;

/// Run one Wright-Fisher trial until the advantaged allele fixes or is lost.
///
/// Models `pop_size` diploid individuals (2N alleles). Allele A has fitness
/// `1 + selection` relative to allele a (fitness 1). Starting frequency
/// is `initial_freq`.
///
/// Returns `true` if allele A fixes, `false` if lost.
///
/// # Panics
///
/// Panics if `pop_size` is zero or `initial_freq` is outside [0, 1].
#[must_use]
pub fn wright_fisher_fixation(
    pop_size: usize,
    selection: f64,
    initial_freq: f64,
    seed: u64,
) -> bool {
    assert!(pop_size > 0, "pop_size must be positive");
    assert!(
        (0.0..=1.0).contains(&initial_freq),
        "initial_freq must be in [0, 1]"
    );

    let n_alleles = 2 * pop_size;
    let n_alleles_f = usize_f64(n_alleles);
    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let mut n_a = (initial_freq * n_alleles_f).round() as u64;
    // Factor 10: Wright-Fisher fixation typically takes O(N) generations;
    // 10× gives headroom for slow selection near neutrality.
    let max_gens = 10 * n_alleles;
    let mut rng = Xorshift64::new(seed);
    let n_alleles_u64 = n_alleles as u64;

    for _ in 0..max_gens {
        if n_a == 0 {
            return false;
        }
        if n_a == n_alleles_u64 {
            return true;
        }

        #[expect(clippy::cast_precision_loss)]
        let freq_a = n_a as f64 / n_alleles_f;
        let fitness_a = freq_a * (1.0 + selection);
        let fitness_total = fitness_a + (1.0 - freq_a);
        let prob_a = fitness_a / fitness_total;

        n_a = rng.binomial(n_alleles, prob_a);
    }

    n_a > n_alleles_u64 / 2
}

/// Kimura (1968) analytical fixation probability.
///
/// `P_fix = (1 - exp(-4Ns p₀)) / (1 - exp(-4Ns))`
///
/// For neutral evolution (s=0), returns `initial_freq`.
///
/// Pending delegation to `barracuda::stats::kimura_fixation` — not yet
/// in barracuda as of `ToadStool` S68+ (pure math, no RNG).
#[must_use]
pub fn kimura_fixation_prob(pop_size: usize, selection: f64, initial_freq: f64) -> f64 {
    // TODO(toadstool): wire when barracuda adds stats::kimura_fixation
    // Status S68+: not yet absorbed. Handoff item — pure scalar, trivial kernel.
    // #[cfg(feature = "barracuda")]
    // {
    //     if let Ok(p) = barracuda::stats::kimura_fixation(pop_size, selection, initial_freq) {
    //         return p;
    //     }
    // }
    kimura_fixation_prob_cpu(pop_size, selection, initial_freq)
}

fn kimura_fixation_prob_cpu(pop_size: usize, selection: f64, initial_freq: f64) -> f64 {
    let four_ns = 4.0 * usize_f64(pop_size) * selection;
    if four_ns.abs() < 1e-10 {
        return initial_freq;
    }

    let numerator = 1.0 - (-four_ns * initial_freq).exp();
    let denominator = 1.0 - (-four_ns).exp();
    if denominator.abs() < 1e-15 {
        return initial_freq;
    }

    numerator / denominator
}

/// Track Shannon diversity under pure neutral drift (multi-species Wright-Fisher).
///
/// Returns a vector of Shannon diversities, one per generation.
///
/// # Panics
///
/// Panics if `n_species` or `pop_size` is zero.
#[must_use]
pub fn neutral_diversity_trajectory(
    n_species: usize,
    pop_size: usize,
    n_generations: usize,
    seed: u64,
) -> Vec<f64> {
    assert!(n_species > 0 && pop_size > 0);

    let mut rng = Xorshift64::new(seed);
    let base_count = pop_size / n_species;
    let mut abundances: Vec<u64> = vec![base_count as u64; n_species];
    let remainder = pop_size - base_count * n_species;
    abundances[0] += remainder as u64;

    let mut diversities = Vec::with_capacity(n_generations);
    let pop_f = usize_f64(pop_size);

    for _ in 0..n_generations {
        let mut shannon = 0.0;
        for &a in &abundances {
            if a > 0 {
                let p = crate::cast::u64_f64(a) / pop_f;
                shannon -= p * p.ln();
            }
        }
        diversities.push(shannon);

        // Multinomial sampling: sequential binomial decomposition
        let mut remaining = pop_size as u64;
        let total: u64 = abundances.iter().sum();
        let mut remaining_prob_mass = crate::cast::u64_f64(total);
        let mut new_abundances = vec![0u64; n_species];

        for sp in 0..n_species - 1 {
            if remaining == 0 {
                break;
            }
            let prob = crate::cast::u64_f64(abundances[sp]) / remaining_prob_mass;
            #[expect(clippy::cast_possible_truncation)]
            let n_remaining = remaining as usize;
            new_abundances[sp] = rng.binomial(n_remaining, prob);
            remaining = remaining.saturating_sub(new_abundances[sp]);
            remaining_prob_mass -= crate::cast::u64_f64(abundances[sp]);
        }
        new_abundances[n_species - 1] = remaining;
        abundances = new_abundances;
    }

    diversities
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kimura_neutral() {
        let p = kimura_fixation_prob(100, 0.0, 0.5);
        // Kimura formula with s=0 returns initial_freq exactly; 1e-10 absorbs floating-point in special-case branch.
        assert!((p - 0.5).abs() < 1e-10, "neutral fixation should be p₀");
    }

    #[test]
    fn kimura_strong_selection() {
        let p = kimura_fixation_prob(1000, 0.01, 0.5);
        assert!(p > 0.5, "positive selection should increase fixation");
        assert!(p < 1.0, "fixation probability should be < 1");
    }

    #[test]
    fn kimura_increases_with_n() {
        let p_small = kimura_fixation_prob(50, 0.01, 0.5);
        let p_large = kimura_fixation_prob(1000, 0.01, 0.5);
        assert!(
            p_large > p_small,
            "fixation prob should increase with N for s > 0"
        );
    }

    #[test]
    fn wf_deterministic() {
        let r1 = wright_fisher_fixation(100, 0.01, 0.5, 42);
        let r2 = wright_fisher_fixation(100, 0.01, 0.5, 42);
        assert_eq!(r1, r2, "same seed should give same result");
    }

    #[test]
    fn diversity_declines_under_drift() {
        let div = neutral_diversity_trajectory(10, 50, 200, 42);
        assert!(
            div.last().copied().unwrap_or(0.0) < div[0],
            "diversity should decline"
        );
    }

    #[test]
    fn larger_pop_preserves_diversity() {
        let div_small = neutral_diversity_trajectory(10, 50, 200, 42);
        let div_large = neutral_diversity_trajectory(10, 500, 200, 42);
        assert!(
            div_large.last().copied().unwrap_or(0.0) > div_small.last().copied().unwrap_or(0.0),
            "larger populations should preserve more diversity"
        );
    }
}
