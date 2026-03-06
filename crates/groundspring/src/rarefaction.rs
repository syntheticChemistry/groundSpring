// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Multinomial rarefaction for sequencing noise analysis.
//!
//! Simulates the effect of finite sequencing depth on taxonomic recovery.
//! A known reference community is sampled at varying depths to determine
//! convergence thresholds for diversity metrics.

use crate::cast::{u64_f64, usize_f64};

/// Simpson diversity index: `1 − Σ pᵢ²`.  Higher = more diverse (0 to 1).
///
/// Takes raw count data.  A perfectly even community of S species gives
/// `1 − 1/S`; a single-species community gives 0.
///
/// When `barracuda-gpu` is enabled and a GPU device is available,
/// delegates to `FusedMapReduceF64::simpson_index` for GPU-accelerated
/// computation. Falls back to `barracuda::stats::simpson` CPU delegation
/// when `barracuda` is enabled.
///
/// # Examples
///
/// ```
/// let even_4 = vec![100u64, 100, 100, 100];
/// let d = groundspring::rarefaction::simpson_diversity(&even_4);
/// assert!((d - 0.75).abs() < 1e-12);  // 1 − 4×0.25²
/// ```
#[must_use]
pub fn simpson_diversity(counts: &[u64]) -> f64 {
    #[cfg(feature = "barracuda-gpu")]
    {
        let f_counts: Vec<f64> = counts.iter().map(|&c| u64_f64(c)).collect();
        if let Some(sum_p2) = simpson_diversity_gpu(&f_counts) {
            return 1.0 - sum_p2;
        }
        barracuda::stats::simpson(&f_counts)
    }
    #[cfg(all(feature = "barracuda", not(feature = "barracuda-gpu")))]
    {
        let f_counts: Vec<f64> = counts.iter().map(|&c| u64_f64(c)).collect();
        barracuda::stats::simpson(&f_counts)
    }
    #[cfg(not(feature = "barracuda"))]
    simpson_diversity_cpu(counts)
}

#[cfg(feature = "barracuda-gpu")]
fn simpson_diversity_gpu(f_counts: &[f64]) -> Option<f64> {
    let device = crate::gpu::get_device()?;
    let fmr = barracuda::ops::fused_map_reduce_f64::FusedMapReduceF64::new(device).ok()?;
    fmr.simpson_index(f_counts).ok()
}

#[cfg(not(feature = "barracuda"))]
fn simpson_diversity_cpu(counts: &[u64]) -> f64 {
    let total: u64 = counts.iter().sum();
    if total == 0 {
        return 0.0;
    }
    let total_f = u64_f64(total);
    let sum_p2: f64 = counts
        .iter()
        .filter(|&&c| c > 0)
        .map(|&c| {
            let p = u64_f64(c) / total_f;
            p * p
        })
        .sum();
    1.0 - sum_p2
}

/// Bray-Curtis dissimilarity: `Σ|aᵢ − bᵢ| / Σ(aᵢ + bᵢ)`.  Range \[0, 1\].
///
/// Returns 0.0 for identical communities, 1.0 for completely disjoint
/// communities.  Takes `f64` abundance vectors (count or relative).
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::stats::bray_curtis` (absorbed from wetSpring in S64).
///
/// # Panics
///
/// Panics if `a` and `b` have different lengths.
#[must_use]
pub fn bray_curtis(a: &[f64], b: &[f64]) -> f64 {
    assert_eq!(a.len(), b.len(), "length mismatch");
    #[cfg(feature = "barracuda")]
    {
        barracuda::stats::bray_curtis(a, b)
    }
    #[cfg(not(feature = "barracuda"))]
    bray_curtis_cpu(a, b)
}

#[cfg(not(feature = "barracuda"))]
fn bray_curtis_cpu(a: &[f64], b: &[f64]) -> f64 {
    let mut num = 0.0;
    let mut den = 0.0;
    for (&ai, &bi) in a.iter().zip(b) {
        num += (ai - bi).abs();
        den += ai + bi;
    }
    if den == 0.0 { 0.0 } else { num / den }
}

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

/// Shannon diversity index H' = −Σ(pᵢ ln pᵢ).
///
/// When `barracuda-gpu` is enabled and a GPU device is available,
/// delegates to `FusedMapReduceF64::shannon_entropy` for GPU-accelerated
/// computation. Falls back to `barracuda::stats::shannon` CPU delegation
/// when `barracuda` is enabled, or to the local CPU implementation.
///
/// Operates on a count vector.  Returns `0.0` if the total count is zero.
///
/// # Examples
///
/// ```
/// let even_4 = vec![100u64, 100, 100, 100];
/// let h = groundspring::rarefaction::shannon_diversity(&even_4);
/// assert!((h - 4.0_f64.ln()).abs() < 1e-12);
/// ```
#[must_use]
pub fn shannon_diversity(counts: &[u64]) -> f64 {
    #[cfg(feature = "barracuda-gpu")]
    {
        let f_counts: Vec<f64> = counts.iter().map(|&c| u64_f64(c)).collect();
        if let Some(val) = shannon_diversity_gpu(&f_counts) {
            return val;
        }
        barracuda::stats::shannon(&f_counts)
    }
    #[cfg(all(feature = "barracuda", not(feature = "barracuda-gpu")))]
    {
        let f_counts: Vec<f64> = counts.iter().map(|&c| u64_f64(c)).collect();
        barracuda::stats::shannon(&f_counts)
    }
    #[cfg(not(feature = "barracuda"))]
    shannon_diversity_cpu(counts)
}

#[cfg(feature = "barracuda-gpu")]
fn shannon_diversity_gpu(f_counts: &[f64]) -> Option<f64> {
    let device = crate::gpu::get_device()?;
    let fmr = barracuda::ops::fused_map_reduce_f64::FusedMapReduceF64::new(device).ok()?;
    fmr.shannon_entropy(f_counts).ok()
}

#[cfg(not(feature = "barracuda"))]
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
        barracuda::stats::pielou_evenness(&f_counts)
    }
    #[cfg(not(feature = "barracuda"))]
    evenness_cpu(counts)
}

#[cfg(not(feature = "barracuda"))]
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

    #[cfg(feature = "barracuda")]
    {
        multinomial_sample_barracuda(abundances, depth, seed)
    }
    #[cfg(not(feature = "barracuda"))]
    multinomial_sample_cpu(abundances, depth, seed)
}

#[cfg(feature = "barracuda")]
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

#[cfg(not(feature = "barracuda"))]
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
fn abundances_to_cumulative(abundances: &[f64]) -> Vec<f64> {
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

fn multinomial_sample_batch_cpu(
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

    let device = crate::gpu::get_device()?;
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
    fn shannon_uniform() {
        let counts = [100, 100, 100, 100];
        let expected = (4.0_f64).ln();
        assert!((shannon_diversity(&counts) - expected).abs() < tol::ANALYTICAL);
    }

    #[test]
    fn shannon_single_species() {
        let counts = [1000, 0, 0, 0];
        assert!(shannon_diversity(&counts).abs() < tol::ANALYTICAL);
    }

    #[test]
    fn shannon_empty() {
        let counts: [u64; 0] = [];
        assert!((shannon_diversity(&counts)).abs() < tol::ANALYTICAL);
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
        assert!((evenness(&counts) - 1.0).abs() < tol::ANALYTICAL);
    }

    #[test]
    fn evenness_single_species() {
        let counts = [100, 0, 0];
        assert!(
            (evenness(&counts) - 1.0).abs() < tol::ANALYTICAL,
            "s≤1 → evenness=1.0 by convention"
        );
    }

    #[test]
    fn evenness_empty() {
        let counts: [u64; 0] = [];
        assert!(
            (evenness(&counts) - 1.0).abs() < tol::ANALYTICAL,
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

    #[test]
    fn simpson_uniform() {
        let counts = [100, 100, 100, 100];
        assert!((simpson_diversity(&counts) - 0.75).abs() < tol::ANALYTICAL);
    }

    #[test]
    fn simpson_single_species() {
        let counts = [1000, 0, 0, 0];
        assert!(simpson_diversity(&counts).abs() < tol::ANALYTICAL);
    }

    #[test]
    fn simpson_empty() {
        let counts: [u64; 0] = [];
        assert!(simpson_diversity(&counts).abs() < tol::ANALYTICAL);
    }

    #[test]
    fn simpson_bounded() {
        let counts = [50, 30, 20];
        let d = simpson_diversity(&counts);
        assert!(d > 0.0 && d < 1.0, "Simpson should be in (0,1), got {d}");
    }

    #[test]
    fn bray_curtis_identical() {
        let a = [10.0, 20.0, 30.0];
        assert!(bray_curtis(&a, &a).abs() < f64::EPSILON);
    }

    #[test]
    fn bray_curtis_disjoint() {
        let a = [10.0, 0.0, 0.0];
        let b = [0.0, 0.0, 10.0];
        assert!((bray_curtis(&a, &b) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn bray_curtis_symmetry() {
        let a = [10.0, 20.0, 30.0, 0.0, 5.0];
        let b = [15.0, 10.0, 25.0, 5.0, 0.0];
        // Symmetry is exact; 1e-15 is stricter than EXACT for identical-path comparison.
        assert!((bray_curtis(&a, &b) - bray_curtis(&b, &a)).abs() < 1e-15);
    }

    #[test]
    fn bray_curtis_bounded() {
        let a = [10.0, 20.0, 30.0];
        let b = [15.0, 10.0, 25.0];
        let bc = bray_curtis(&a, &b);
        assert!(
            (0.0..=1.0).contains(&bc),
            "Bray-Curtis should be in [0,1], got {bc}"
        );
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
        let cum = super::abundances_to_cumulative(&abundances);
        assert!((cum.last().unwrap() - 1.0).abs() < f64::EPSILON);
        for i in 1..cum.len() {
            assert!(cum[i] >= cum[i - 1]);
        }
    }
}
