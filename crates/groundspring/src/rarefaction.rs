// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Multinomial rarefaction for sequencing noise analysis.
//!
//! Simulates the effect of finite sequencing depth on taxonomic recovery.
//! A known reference community is sampled at varying depths to determine
//! convergence thresholds for diversity metrics.

use crate::cast::{u64_f64, usize_f64};

/// Shannon diversity index H' = −Σ(pᵢ ln pᵢ).
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::stats::shannon` (natural log convention).
/// Operates on a count vector.  Returns `0.0` if the total count is zero.
#[must_use]
pub fn shannon_diversity(counts: &[u64]) -> f64 {
    #[cfg(feature = "barracuda")]
    {
        let f_counts: Vec<f64> = counts.iter().map(|&c| u64_f64(c)).collect();
        return barracuda::stats::shannon(&f_counts);
    }
    #[allow(unreachable_code)]
    shannon_diversity_cpu(counts)
}

fn shannon_diversity_cpu(counts: &[u64]) -> f64 {
    let total: u64 = counts.iter().sum();
    if total == 0 {
        return 0.0;
    }
    let total_f = u64_f64(total);
    counts
        .iter()
        .filter(|&&c| c > 0)
        .map(|&c| {
            let p = u64_f64(c) / total_f;
            -p * p.ln()
        })
        .sum()
}

/// Pielou's evenness J' = H' / ln(S).
///
/// S is the number of species with non-zero counts.  Returns `1.0` by
/// convention when S ≤ 1 (single species = perfect evenness).
///
/// When the `barracuda` feature is enabled, delegates the S > 1 case to
/// `barracuda::stats::pielou_evenness` (which returns `0.0` for S ≤ 1;
/// groundSpring overrides that to `1.0` for consistency with the ecology
/// convention used by our Python baselines).
#[must_use]
pub fn evenness(counts: &[u64]) -> f64 {
    let s = counts.iter().filter(|&&c| c > 0).count();
    if s <= 1 {
        return 1.0;
    }
    #[cfg(feature = "barracuda")]
    {
        let f_counts: Vec<f64> = counts.iter().map(|&c| u64_f64(c)).collect();
        return barracuda::stats::pielou_evenness(&f_counts);
    }
    #[allow(unreachable_code)]
    evenness_cpu(counts)
}

fn evenness_cpu(counts: &[u64]) -> f64 {
    let h = shannon_diversity(counts);
    let s = counts.iter().filter(|&&c| c > 0).count();
    h / usize_f64(s).ln()
}

/// Number of taxa detected (count > 0).
#[must_use]
pub fn taxa_detected(counts: &[u64]) -> usize {
    counts.iter().filter(|&&c| c > 0).count()
}

/// Deterministic multinomial sampling using [`Xorshift64`](crate::prng::Xorshift64).
///
/// For each of `depth` reads, assigns to a taxon proportional to
/// `abundances` (which must sum to ~1.0).
///
/// This is a pure-Rust replacement for `NumPy`'s `rng.multinomial()`.
#[must_use]
pub fn multinomial_sample(abundances: &[f64], depth: u64, seed: u64) -> Vec<u64> {
    use crate::prng::Xorshift64;

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
        if let Some(last) = cum.last_mut() {
            *last = 1.0;
        }
        cum
    };

    let mut rng = Xorshift64::new(seed);
    for _ in 0..depth {
        let u = rng.next_f64();
        let idx = match cumulative.binary_search_by(|probe| probe.total_cmp(&u)) {
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
        genera_counts.push(usize_f64(taxa_detected(&counts)));
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

    #[test]
    fn evenness_single_species() {
        let counts = [100, 0, 0];
        assert!(
            (evenness(&counts) - 1.0).abs() < 1e-10,
            "s≤1 → evenness=1.0 by convention"
        );
    }

    #[test]
    fn evenness_empty() {
        let counts: [u64; 0] = [];
        assert!(
            (evenness(&counts) - 1.0).abs() < 1e-10,
            "s≤1 → evenness=1.0 by convention"
        );
    }

    #[test]
    fn multinomial_empty_abundances() {
        let counts = multinomial_sample(&[], 100, 42);
        assert!(counts.is_empty());
    }

    #[test]
    fn multinomial_zero_depth() {
        let counts = multinomial_sample(&[0.5, 0.5], 0, 42);
        assert_eq!(counts, vec![0, 0]);
    }

    #[test]
    fn rarefaction_at_depth_convergence() {
        let community = vec![0.5, 0.3, 0.15, 0.05];
        let low = rarefaction_at_depth(&community, 10, 5, 42);
        let high = rarefaction_at_depth(&community, 10_000, 5, 42);

        assert!(high.genera_mean >= low.genera_mean);
        assert!(high.shannon_mean >= low.shannon_mean);
        assert!((high.genera_mean - 4.0).abs() < 0.5);
    }

    #[test]
    fn rarefaction_at_depth_deterministic() {
        let community = vec![0.5, 0.3, 0.2];
        let a = rarefaction_at_depth(&community, 100, 10, 42);
        let b = rarefaction_at_depth(&community, 100, 10, 42);

        assert!((a.shannon_mean - b.shannon_mean).abs() < f64::EPSILON);
        assert!((a.genera_mean - b.genera_mean).abs() < f64::EPSILON);
    }

    #[test]
    fn taxa_detected_empty() {
        let counts: [u64; 0] = [];
        assert_eq!(taxa_detected(&counts), 0);
    }

    #[test]
    fn taxa_detected_all_present() {
        let counts = [1, 2, 3];
        assert_eq!(taxa_detected(&counts), 3);
    }
}
