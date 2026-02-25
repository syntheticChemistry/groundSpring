// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Multinomial rarefaction for sequencing noise analysis.
//!
//! Simulates the effect of finite sequencing depth on taxonomic recovery.
//! A known reference community is sampled at varying depths to determine
//! convergence thresholds for diversity metrics.

/// Shannon diversity index H' = −Σ(pᵢ ln pᵢ).
///
/// Operates on a count vector.  Returns `0.0` if the total count is zero.
#[must_use]
pub fn shannon_diversity(counts: &[u64]) -> f64 {
    let total: u64 = counts.iter().sum();
    if total == 0 {
        return 0.0;
    }
    let total_f = total as f64;
    counts
        .iter()
        .filter(|&&c| c > 0)
        .map(|&c| {
            let p = c as f64 / total_f;
            -p * p.ln()
        })
        .sum()
}

/// Pielou's evenness J' = H' / ln(S).
///
/// S is the number of species with non-zero counts.
#[must_use]
pub fn evenness(counts: &[u64]) -> f64 {
    let s = counts.iter().filter(|&&c| c > 0).count();
    if s <= 1 {
        return 1.0;
    }
    let h = shannon_diversity(counts);
    h / (s as f64).ln()
}

/// Number of taxa detected (count > 0).
#[must_use]
pub fn taxa_detected(counts: &[u64]) -> usize {
    counts.iter().filter(|&&c| c > 0).count()
}

/// Simple deterministic multinomial sampling using a linear congruential
/// generator seeded by `seed`.
///
/// For each of `depth` reads, assigns to a taxon proportional to
/// `abundances` (which must sum to ~1.0).
///
/// This is a pure-Rust replacement for `NumPy`'s `rng.multinomial()`.
#[must_use]
pub fn multinomial_sample(abundances: &[f64], depth: u64, seed: u64) -> Vec<u64> {
    let n = abundances.len();
    let mut counts = vec![0u64; n];
    if n == 0 || depth == 0 {
        return counts;
    }

    let cumulative: Vec<f64> = {
        let mut cum = Vec::with_capacity(n);
        let mut acc = 0.0;
        for &a in abundances {
            acc += a;
            cum.push(acc);
        }
        // Ensure the last element is exactly 1.0 for robustness
        if let Some(last) = cum.last_mut() {
            *last = 1.0;
        }
        cum
    };

    let mut state = seed;
    for _ in 0..depth {
        // Xorshift64 PRNG — fast, deterministic, adequate for sampling
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let u = (state as f64) / (u64::MAX as f64);

        // Binary search for the taxon
        let idx = match cumulative
            .binary_search_by(|probe| probe.partial_cmp(&u).unwrap_or(std::cmp::Ordering::Equal))
        {
            Ok(i) | Err(i) => i.min(n - 1),
        };
        counts[idx] += 1;
    }

    counts
}

/// Rarefaction result at a single depth.
#[derive(Debug, Clone)]
pub struct RarefactionResult {
    /// Sequencing depth (number of reads).
    pub depth: u64,
    /// Number of replicates.
    pub n_replicates: usize,
    /// Mean number of genera detected.
    pub genera_mean: f64,
    /// Std dev of genera detected.
    pub genera_std: f64,
    /// Mean Shannon diversity.
    pub shannon_mean: f64,
    /// Std dev of Shannon diversity.
    pub shannon_std: f64,
}

/// Run rarefaction at a given depth with multiple replicates.
#[must_use]
pub fn rarefaction_at_depth(
    abundances: &[f64],
    depth: u64,
    n_replicates: usize,
    base_seed: u64,
) -> RarefactionResult {
    let mut genera_counts = Vec::with_capacity(n_replicates);
    let mut shannon_values = Vec::with_capacity(n_replicates);

    for rep in 0..n_replicates {
        let seed = base_seed.wrapping_add(depth).wrapping_add(rep as u64);
        let counts = multinomial_sample(abundances, depth, seed);
        genera_counts.push(taxa_detected(&counts) as f64);
        shannon_values.push(shannon_diversity(&counts));
    }

    let genera_mean = crate::stats::mean(&genera_counts);
    let genera_std = crate::stats::std_dev(&genera_counts);
    let shannon_mean = crate::stats::mean(&shannon_values);
    let shannon_std = crate::stats::std_dev(&shannon_values);

    RarefactionResult {
        depth,
        n_replicates,
        genera_mean,
        genera_std,
        shannon_mean,
        shannon_std,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shannon_uniform() {
        let counts = [100, 100, 100, 100];
        let expected = (4.0_f64).ln();
        assert!((shannon_diversity(&counts) - expected).abs() < 1e-10);
    }

    #[test]
    fn shannon_single_species() {
        let counts = [1000, 0, 0, 0];
        assert!(shannon_diversity(&counts).abs() < 1e-10);
    }

    #[test]
    fn shannon_empty() {
        let counts: [u64; 0] = [];
        assert!((shannon_diversity(&counts)).abs() < 1e-10);
    }

    #[test]
    fn taxa_detected_counts_nonzero() {
        let counts = [10, 0, 5, 0, 1];
        assert_eq!(taxa_detected(&counts), 3);
    }

    #[test]
    fn multinomial_deterministic() {
        let abundances = [0.5, 0.3, 0.2];
        let r1 = multinomial_sample(&abundances, 10_000, 42);
        let r2 = multinomial_sample(&abundances, 10_000, 42);
        assert_eq!(r1, r2, "Same seed must produce identical samples");
    }

    #[test]
    fn multinomial_different_seeds_differ() {
        let abundances = [0.5, 0.3, 0.2];
        let r1 = multinomial_sample(&abundances, 10_000, 42);
        let r2 = multinomial_sample(&abundances, 10_000, 99);
        assert_ne!(r1, r2);
    }

    #[test]
    fn multinomial_total_equals_depth() {
        let abundances = [0.25, 0.25, 0.25, 0.25];
        let counts = multinomial_sample(&abundances, 1000, 42);
        let total: u64 = counts.iter().sum();
        assert_eq!(total, 1000);
    }

    #[test]
    fn evenness_uniform() {
        let counts = [100, 100, 100, 100];
        assert!((evenness(&counts) - 1.0).abs() < 1e-10);
    }
}
