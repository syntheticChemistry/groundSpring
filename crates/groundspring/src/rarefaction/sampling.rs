// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Multinomial sampling engine for rarefaction analysis.
//!
//! Simulates sequencing reads by drawing from a community abundance
//! distribution. GPU-accelerated batch dispatch via barraCuda when available.

/// Deterministic multinomial sampling using [`Xorshift64`](crate::prng::Xorshift64).
///
/// For each of `depth` reads, assigns to a taxon proportional to
/// `abundances` (which must sum to ~1.0).
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::ops::bio::multinomial_sample_cpu` (absorbed from
/// wetSpring S15, groundSpring V62 → barraCuda S93). Uses barraCuda's
/// LCG PRNG — sequences differ from the local Xorshift64 path but
/// distributional properties are identical.
///
/// This is a pure-Rust replacement for `NumPy`'s `rng.multinomial()`.
#[must_use]
pub fn multinomial_sample(abundances: &[f64], depth: u64, seed: u64) -> Vec<u64> {
    let n = abundances.len();
    if n == 0 || depth == 0 {
        return vec![0u64; n];
    }

    #[cfg(feature = "barracuda-gpu")]
    {
        multinomial_sample_barracuda(abundances, depth, seed)
    }
    #[cfg(not(feature = "barracuda-gpu"))]
    multinomial_sample_cpu(abundances, depth, seed)
}

#[cfg(feature = "barracuda-gpu")]
fn multinomial_sample_barracuda(abundances: &[f64], depth: u64, seed: u64) -> Vec<u64> {
    let cumulative = abundances_to_cumulative(abundances);
    #[expect(
        clippy::cast_possible_truncation,
        reason = "sequencing depth ≤ 10^6, fits u32"
    )]
    let depth_u32 = depth as u32;
    let mut rng = crate::prng::Xorshift64::new(seed);
    let counts_u32 =
        barracuda::ops::bio::multinomial_sample_cpu(&cumulative, depth_u32, &mut || rng.next_f64());
    counts_u32.iter().map(|&c| u64::from(c)).collect()
}

#[cfg(not(feature = "barracuda-gpu"))]
fn multinomial_sample_cpu(abundances: &[f64], depth: u64, seed: u64) -> Vec<u64> {
    use crate::prng::Xorshift64;

    let n = abundances.len();
    let mut counts = vec![0u64; n];
    let cumulative: Vec<f64> = abundances_to_cumulative(abundances);

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

/// Convert raw abundances to cumulative probabilities.
///
/// Used internally for CPU multinomial sampling and as the adapter for
/// `BatchedMultinomialGpu` which expects cumulative probabilities.
pub(super) fn abundances_to_cumulative(abundances: &[f64]) -> Vec<f64> {
    let mut cum = Vec::with_capacity(abundances.len());
    let mut acc = 0.0;
    for &a in abundances {
        acc += a;
        cum.push(acc);
    }
    if let Some(last) = cum.last_mut() {
        *last = 1.0;
    }
    cum
}

/// Batch multinomial sampling across multiple replicates.
///
/// When the `barracuda-gpu` feature is enabled and a GPU is available,
/// dispatches all replicates in a single GPU batch via `BatchedMultinomialGpu`.
/// Falls back to sequential CPU sampling otherwise.
///
/// Returns a vector of count vectors, one per replicate.
#[must_use]
pub fn multinomial_sample_batch(
    abundances: &[f64],
    depth: u64,
    n_replicates: usize,
    base_seed: u64,
) -> Vec<Vec<u64>> {
    #[cfg(feature = "barracuda-gpu")]
    {
        if let Some(result) =
            multinomial_sample_batch_gpu(abundances, depth, n_replicates, base_seed)
        {
            return result;
        }
    }
    multinomial_sample_batch_cpu(abundances, depth, n_replicates, base_seed)
}

pub(super) fn multinomial_sample_batch_cpu(
    abundances: &[f64],
    depth: u64,
    n_replicates: usize,
    base_seed: u64,
) -> Vec<Vec<u64>> {
    (0..n_replicates)
        .map(|i| multinomial_sample(abundances, depth, base_seed.wrapping_add(i as u64)))
        .collect()
}

#[cfg(feature = "barracuda-gpu")]
fn multinomial_sample_batch_gpu(
    abundances: &[f64],
    depth: u64,
    n_replicates: usize,
    base_seed: u64,
) -> Option<Vec<Vec<u64>>> {
    use barracuda::ops::bio::{BatchedMultinomialConfig, BatchedMultinomialGpu};

    let device = crate::gpu::get_device_f64_safe()?;
    let cumulative = abundances_to_cumulative(abundances);
    let n_taxa = abundances.len();

    let mut seeds = Vec::with_capacity(n_replicates * 4);
    let mut rng = crate::prng::Xoshiro128StarStar::new(base_seed);
    for _ in 0..n_replicates {
        for _ in 0..4 {
            seeds.push(rng.next_u32());
        }
    }

    #[expect(
        clippy::cast_possible_truncation,
        reason = "sequencing depth ≤ 10^6, fits u32"
    )]
    let depth_u32 = depth as u32;
    #[expect(
        clippy::cast_possible_truncation,
        reason = "n_replicates ≤ 100, fits u32"
    )]
    let n_reps_u32 = n_replicates as u32;

    let config = BatchedMultinomialConfig {
        cumulative_probs: true,
        seed: None,
    };
    let gpu = BatchedMultinomialGpu::new(device).ok()?;
    let flat_counts = gpu
        .sample(&cumulative, Some(&mut seeds), depth_u32, n_reps_u32, config)
        .ok()?;

    let result: Vec<Vec<u64>> = flat_counts
        .chunks_exact(n_taxa)
        .map(|chunk| chunk.iter().map(|&c| u64::from(c)).collect())
        .collect();

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn multinomial_batch_total_equals_depth() {
        let abundances = [0.4, 0.3, 0.2, 0.1];
        let depth = 1000;
        let batch = multinomial_sample_batch(&abundances, depth, 10, 42);
        assert_eq!(batch.len(), 10);
        for counts in &batch {
            let total: u64 = counts.iter().sum();
            assert_eq!(total, depth);
        }
    }

    #[test]
    fn multinomial_batch_parity_cpu_vs_dispatch() {
        let abundances = [0.5, 0.3, 0.2];
        let batch = multinomial_sample_batch(&abundances, 500, 20, 42);
        let cpu = multinomial_sample_batch_cpu(&abundances, 500, 20, 42);

        if cfg!(feature = "barracuda-gpu") {
            for (b, c) in batch.iter().zip(&cpu) {
                let b_total: u64 = b.iter().sum();
                let c_total: u64 = c.iter().sum();
                assert_eq!(b_total, c_total, "totals must match");
            }
        } else {
            assert_eq!(batch, cpu);
        }
    }

    #[test]
    fn multinomial_batch_empty() {
        let result = multinomial_sample_batch(&[], 100, 5, 42);
        assert_eq!(result.len(), 5);
        for counts in &result {
            assert!(counts.is_empty());
        }
    }

    #[test]
    fn abundances_to_cumulative_sums_to_one() {
        let abundances = [0.25, 0.25, 0.25, 0.25];
        let cum = abundances_to_cumulative(&abundances);
        assert!((cum.last().unwrap() - 1.0).abs() < f64::EPSILON);
        for i in 1..cum.len() {
            assert!(cum[i] >= cum[i - 1]);
        }
    }
}
