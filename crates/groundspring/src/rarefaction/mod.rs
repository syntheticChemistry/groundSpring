// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Multinomial rarefaction for sequencing noise analysis.
//!
//! Simulates the effect of finite sequencing depth on taxonomic recovery.
//! A known reference community is sampled at varying depths to determine
//! convergence thresholds for diversity metrics.
//!
//! ## Module structure
//!
//! - `diversity` — Simpson, Shannon, Bray-Curtis, Pielou evenness, taxa count
//! - `sampling` — deterministic multinomial sampling (CPU + GPU batch)
//! - This module — analytical rarefaction curves and depth-sweep orchestration

mod diversity;
mod sampling;

pub use diversity::{bray_curtis, evenness, shannon_diversity, simpson_diversity, taxa_detected};
pub use sampling::{multinomial_sample, multinomial_sample_batch};

use crate::cast::{u64_f64, usize_f64};

/// Analytical (hypergeometric) rarefaction curve: expected species at
/// each subsampling depth.
///
/// Implements `E[S_n] = S − Σ C(N−Nᵢ, n) / C(N, n)` via log-space
/// computation for numerical stability at large counts.  No RNG needed.
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::stats::rarefaction_curve` (absorbed from wetSpring S64).
#[must_use]
pub fn analytical_rarefaction(counts: &[u64], depths: &[u64]) -> Vec<f64> {
    #[cfg(feature = "barracuda")]
    {
        let f_counts: Vec<f64> = counts.iter().map(|&c| u64_f64(c)).collect();
        let f_depths: Vec<f64> = depths.iter().map(|&d| u64_f64(d)).collect();
        barracuda::stats::rarefaction_curve(&f_counts, &f_depths)
    }
    #[cfg(not(feature = "barracuda"))]
    analytical_rarefaction_cpu(counts, depths)
}

#[cfg(not(feature = "barracuda"))]
fn analytical_rarefaction_cpu(counts: &[u64], depths: &[u64]) -> Vec<f64> {
    let total: u64 = counts.iter().sum();
    if total == 0 {
        return vec![0.0; depths.len()];
    }
    let s_obs = usize_f64(counts.iter().filter(|&&c| c > 0).count());

    depths
        .iter()
        .map(|&depth| {
            if depth == 0 {
                return 0.0;
            }
            if depth >= total {
                return s_obs;
            }
            let mut expected = 0.0;
            for &c in counts {
                if c == 0 {
                    continue;
                }
                let absent_log = log_hypergeometric_absent(total, c, depth);
                expected += 1.0 - absent_log.exp();
            }
            expected
        })
        .collect()
}

/// `log(C(N−Nᵢ, n) / C(N, n))` in log-space for numerical stability.
#[cfg(not(feature = "barracuda"))]
fn log_hypergeometric_absent(big_n: u64, ni: u64, n: u64) -> f64 {
    if ni >= big_n {
        return f64::NEG_INFINITY;
    }
    let remainder = big_n - ni;
    if n > remainder {
        return f64::NEG_INFINITY;
    }
    let mut log_ratio = 0.0_f64;
    for k in 0..n {
        log_ratio += u64_f64(remainder - k).ln() - u64_f64(big_n - k).ln();
    }
    log_ratio
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
///
/// Uses [`multinomial_sample_batch`] so the full set of replicates is
/// dispatched to `BatchedMultinomialGpu` when the `barracuda-gpu`
/// feature is enabled, falling back to sequential CPU sampling
/// otherwise.  Seed scheme: `base_seed + depth + i` for replicate `i`,
/// matching the original per-sample loop for bitwise determinism.
#[must_use]
pub fn rarefaction_at_depth(
    abundances: &[f64],
    depth: u64,
    n_replicates: usize,
    base_seed: u64,
) -> RarefactionResult {
    let batch_seed = base_seed.wrapping_add(depth);
    let batch = multinomial_sample_batch(abundances, depth, n_replicates, batch_seed);

    let mut genera_counts = Vec::with_capacity(n_replicates);
    let mut shannon_values = Vec::with_capacity(n_replicates);
    for counts in &batch {
        genera_counts.push(usize_f64(taxa_detected(counts)));
        shannon_values.push(shannon_diversity(counts));
    }

    let (genera_mean, genera_std) = crate::stats::mean_and_std_dev(&genera_counts);
    let (shannon_mean, shannon_std) = crate::stats::mean_and_std_dev(&shannon_values);

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
    use crate::tol;

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
    fn analytical_rarefaction_full_depth() {
        let counts = [10, 20, 30, 5];
        let total: u64 = counts.iter().sum();
        let curve = analytical_rarefaction(&counts, &[total]);
        assert!((curve[0] - 4.0).abs() < tol::ANALYTICAL);
    }

    #[test]
    fn analytical_rarefaction_monotonic() {
        let counts = [50, 30, 20, 10, 5, 3, 2, 1];
        let depths: Vec<u64> = (1..=120).collect();
        let curve = analytical_rarefaction(&counts, &depths);
        for i in 1..curve.len() {
            assert!(
                curve[i] >= curve[i - 1] - tol::ANALYTICAL,
                "not monotonic at depth {}",
                depths[i]
            );
        }
    }

    #[test]
    fn analytical_rarefaction_zero_depth() {
        let counts = [10, 20, 30];
        let curve = analytical_rarefaction(&counts, &[0]);
        assert!(curve[0].abs() < f64::EPSILON);
    }

    #[test]
    fn analytical_rarefaction_empty_community() {
        let counts: [u64; 0] = [];
        let curve = analytical_rarefaction(&counts, &[10, 20]);
        assert!(curve.iter().all(|&v| v.abs() < f64::EPSILON));
    }
}
