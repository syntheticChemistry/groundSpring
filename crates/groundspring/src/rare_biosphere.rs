// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Rare biosphere signal detection.
//!
//! Extends [`rarefaction`](crate::rarefaction) to the rare end of the
//! abundance distribution: when does a detected rare microbial lineage
//! represent real biological signal vs. a sequencing artifact?
//!
//! # Key functions
//!
//! - [`chao1`] — non-parametric richness estimator (Chao 1984)
//! - [`detection_power`] — probability of detecting a taxon at given depth
//! - [`detection_threshold`] — minimum depth for target detection power
//! - [`abundance_occupancy`] — detection frequency across replicate samples
//!
//! # References
//!
//! - Anderson, Sogin, Baross (2015) FEMS Microbiol Ecol 91:fiv016
//! - Chao (1984) Scand J Stat 11:265-270
//! - Sogin et al. (2006) PNAS 103:12115-12120
//!
//! # barracuda delegation
//!
//! [`detection_power`] and [`detection_threshold`] are pure math with no
//! RNG — barracuda CPU candidates. [`abundance_occupancy`] and
//! [`tier_detection_rate`] delegate to `BatchedMultinomialGpu` when
//! `barracuda-gpu` is enabled (V42 GPU rewiring, wetSpring bio shader
//! provenance via neuralSpring metalForge S64+).
//! [`chao1`] stays local (integer equality semantics differ from
//! barracuda's float-based classifier).

use crate::cast::{u64_f64, usize_f64};

/// Chao1 non-parametric richness estimator.
///
/// `S_chao1 = S_obs + f₁² / (2·f₂)`
///
/// where `f₁` = singletons (count == 1), `f₂` = doubletons (count == 2).
/// When `f₂ = 0` and `f₁ > 0`, uses the bias-corrected form
/// `S_obs + f₁(f₁ − 1) / 2` (Chao 1984).
///
/// Delegates to `barracuda::stats::diversity::chao1_classic` when the
/// `barracuda` feature is enabled (absorbed in barraCuda S71+++ with
/// Chao 1984 formula and `u64` input — formula parity confirmed).
#[must_use]
pub fn chao1(counts: &[u64]) -> f64 {
    #[cfg(feature = "barracuda")]
    {
        barracuda::stats::diversity::chao1_classic(counts)
    }
    #[cfg(not(feature = "barracuda"))]
    {
        chao1_cpu(counts)
    }
}

#[cfg(not(feature = "barracuda"))]
fn chao1_cpu(counts: &[u64]) -> f64 {
    let s_obs = usize_f64(counts.iter().filter(|&&c| c > 0).count());
    let f1 = usize_f64(counts.iter().filter(|&&c| c == 1).count());
    let f2 = usize_f64(counts.iter().filter(|&&c| c == 2).count());

    if f2 > 0.0 {
        s_obs + (f1 * f1) / (2.0 * f2)
    } else if f1 > 0.0 {
        s_obs + f1 * (f1 - 1.0) / 2.0
    } else {
        s_obs
    }
}

/// Analytical detection probability for a taxon at relative `abundance`
/// given sequencing `depth`.
///
/// `P(detect) = 1 − (1 − p)^D`
///
/// Uses log-exp form for numerical stability at large depths.
#[must_use]
pub fn detection_power(abundance: f64, depth: u64) -> f64 {
    #[cfg(feature = "barracuda")]
    {
        barracuda::stats::evolution::detection_power(abundance, depth)
    }
    #[cfg(not(feature = "barracuda"))]
    {
        if abundance <= 0.0 {
            return 0.0;
        }
        if abundance >= 1.0 {
            return 1.0;
        }
        let log_miss = (1.0 - abundance).ln();
        1.0 - (log_miss * u64_f64(depth)).exp()
    }
}

/// Minimum sequencing depth to detect a taxon at `abundance` with
/// probability ≥ `target_power`.
///
/// `D* = ⌈ ln(1 − P_target) / ln(1 − p) ⌉`
#[must_use]
pub fn detection_threshold(abundance: f64, target_power: f64) -> u64 {
    #[cfg(feature = "barracuda")]
    {
        barracuda::stats::evolution::detection_threshold(abundance, target_power)
    }
    #[cfg(not(feature = "barracuda"))]
    {
        if abundance <= 0.0 || abundance >= 1.0 {
            return 0;
        }
        let d = (1.0 - target_power).log(1.0 - abundance);
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "detection depth ≤ 2^53; ceil guarantees non-negative"
        )]
        let result = d.ceil() as u64;
        result
    }
}

/// Compute detection frequency (occupancy) for each species across
/// `n_samples` independent draws at a given `depth`.
///
/// Returns a vector of length `community.len()` with values in \[0, 1\].
#[must_use]
pub fn abundance_occupancy(
    community: &[f64],
    depth: u64,
    n_samples: usize,
    base_seed: u64,
) -> Vec<f64> {
    #[cfg(feature = "barracuda-gpu")]
    {
        if let Some(occ) = abundance_occupancy_gpu(community, depth, n_samples, base_seed) {
            return occ;
        }
    }
    abundance_occupancy_cpu(community, depth, n_samples, base_seed)
}

/// GPU-accelerated occupancy via `BatchedMultinomialGpu` (wetSpring bio shader S64+).
///
/// Converts community → cumulative probabilities, runs batched GPU multinomial,
/// then reduces counts → presence/absence fractions on host.
#[cfg(feature = "barracuda-gpu")]
fn abundance_occupancy_gpu(
    community: &[f64],
    depth: u64,
    n_samples: usize,
    base_seed: u64,
) -> Option<Vec<f64>> {
    use crate::cast::usize_f64;
    use barracuda::ops::bio::{BatchedMultinomialConfig, BatchedMultinomialGpu};

    let device = crate::gpu::get_device()?;

    let cumulative = community_to_cumulative(community);
    let mut seeds = generate_xoshiro_seeds(n_samples, base_seed);

    let config = BatchedMultinomialConfig {
        cumulative_probs: true,
        seed: None,
    };
    let gpu = BatchedMultinomialGpu::new(device).ok()?;
    #[expect(clippy::cast_possible_truncation, reason = "GPU u32 counts fit in u64")]
    let counts = gpu
        .sample(
            &cumulative,
            Some(&mut seeds),
            depth as u32,
            n_samples as u32,
            config,
        )
        .ok()?;

    let n_taxa = community.len();
    let n_f = usize_f64(n_samples);
    let mut occupancy = vec![0.0_f64; n_taxa];
    for rep in 0..n_samples {
        let row = &counts[rep * n_taxa..(rep + 1) * n_taxa];
        for (i, &c) in row.iter().enumerate() {
            if c > 0 {
                occupancy[i] += 1.0;
            }
        }
    }
    for occ in &mut occupancy {
        *occ /= n_f;
    }
    Some(occupancy)
}

fn abundance_occupancy_cpu(
    community: &[f64],
    depth: u64,
    n_samples: usize,
    base_seed: u64,
) -> Vec<f64> {
    use crate::prng::Xorshift64;
    use crate::rarefaction::multinomial_sample;

    let n = community.len();
    let mut detection_counts = vec![0u64; n];
    let mut seed_rng = Xorshift64::new(base_seed);

    for _ in 0..n_samples {
        let seed = seed_rng.next_u64();
        let counts = multinomial_sample(community, depth, seed);
        for (i, &c) in counts.iter().enumerate() {
            if c > 0 {
                detection_counts[i] += 1;
            }
        }
    }

    let n_f = usize_f64(n_samples);
    detection_counts.iter().map(|&c| u64_f64(c) / n_f).collect()
}

/// Mean singleton fraction: average across replicates of
/// (singletons / observed species).
#[must_use]
pub fn singleton_fraction(
    community: &[f64],
    depth: u64,
    n_replicates: usize,
    base_seed: u64,
) -> f64 {
    use crate::prng::Xorshift64;
    use crate::rarefaction::multinomial_sample;

    let mut rng = Xorshift64::new(base_seed);
    let mut total = 0.0;

    for _ in 0..n_replicates {
        let seed = rng.next_u64();
        let counts = multinomial_sample(community, depth, seed);
        let s_obs = counts.iter().filter(|&&c| c > 0).count();
        let f1 = counts.iter().filter(|&&c| c == 1).count();
        if s_obs > 0 {
            total += usize_f64(f1) / usize_f64(s_obs);
        }
    }

    total / usize_f64(n_replicates)
}

/// Mean Chao1 estimate across replicates at a given depth.
#[must_use]
pub fn mean_chao1_at_depth(
    community: &[f64],
    depth: u64,
    n_replicates: usize,
    base_seed: u64,
) -> (f64, f64) {
    use crate::prng::Xorshift64;
    use crate::rarefaction::{multinomial_sample, taxa_detected};

    let mut rng = Xorshift64::new(base_seed);
    let mut chao1_sum = 0.0;
    let mut sobs_sum = 0.0;

    for _ in 0..n_replicates {
        let seed = rng.next_u64();
        let counts = multinomial_sample(community, depth, seed);
        chao1_sum += chao1(&counts);
        sobs_sum += usize_f64(taxa_detected(&counts));
    }

    let n = usize_f64(n_replicates);
    (chao1_sum / n, sobs_sum / n)
}

/// Tier detection rate: fraction of (species, replicate) pairs detected.
#[must_use]
pub fn tier_detection_rate(
    community: &[f64],
    tier_lo: usize,
    tier_hi: usize,
    depth: u64,
    n_replicates: usize,
    base_seed: u64,
) -> f64 {
    #[cfg(feature = "barracuda-gpu")]
    {
        if let Some(rate) =
            tier_detection_rate_gpu(community, tier_lo, tier_hi, depth, n_replicates, base_seed)
        {
            return rate;
        }
    }
    tier_detection_rate_cpu(community, tier_lo, tier_hi, depth, n_replicates, base_seed)
}

/// GPU-accelerated tier detection via `BatchedMultinomialGpu` (wetSpring bio shader S64+).
#[cfg(feature = "barracuda-gpu")]
fn tier_detection_rate_gpu(
    community: &[f64],
    tier_lo: usize,
    tier_hi: usize,
    depth: u64,
    n_replicates: usize,
    base_seed: u64,
) -> Option<f64> {
    use crate::cast::usize_f64;
    use barracuda::ops::bio::{BatchedMultinomialConfig, BatchedMultinomialGpu};

    let device = crate::gpu::get_device()?;

    let cumulative = community_to_cumulative(community);
    let mut seeds = generate_xoshiro_seeds(n_replicates, base_seed);

    let config = BatchedMultinomialConfig {
        cumulative_probs: true,
        seed: None,
    };
    let gpu = BatchedMultinomialGpu::new(device).ok()?;
    #[expect(clippy::cast_possible_truncation, reason = "GPU u32 counts fit in u64")]
    let counts = gpu
        .sample(
            &cumulative,
            Some(&mut seeds),
            depth as u32,
            n_replicates as u32,
            config,
        )
        .ok()?;

    let n_taxa = community.len();
    let n_species = tier_hi - tier_lo;
    let mut detections = 0usize;
    for rep in 0..n_replicates {
        let row = &counts[rep * n_taxa..(rep + 1) * n_taxa];
        for &count in &row[tier_lo..tier_hi] {
            if count > 0 {
                detections += 1;
            }
        }
    }
    Some(usize_f64(detections) / usize_f64(n_species * n_replicates))
}

fn tier_detection_rate_cpu(
    community: &[f64],
    tier_lo: usize,
    tier_hi: usize,
    depth: u64,
    n_replicates: usize,
    base_seed: u64,
) -> f64 {
    use crate::prng::Xorshift64;
    use crate::rarefaction::multinomial_sample;

    let mut rng = Xorshift64::new(base_seed);
    let n_species = tier_hi - tier_lo;
    let mut detections = 0usize;

    for _ in 0..n_replicates {
        let seed = rng.next_u64();
        let counts = multinomial_sample(community, depth, seed);
        for &count in &counts[tier_lo..tier_hi] {
            if count > 0 {
                detections += 1;
            }
        }
    }

    usize_f64(detections) / usize_f64(n_species * n_replicates)
}

/// Convert a community probability vector to cumulative probabilities
/// for `BatchedMultinomialGpu`.
#[cfg(feature = "barracuda-gpu")]
fn community_to_cumulative(community: &[f64]) -> Vec<f64> {
    let total: f64 = community.iter().sum();
    let mut cumulative = Vec::with_capacity(community.len());
    let mut running = 0.0;
    for &p in community {
        running += p / total;
        cumulative.push(running);
    }
    if let Some(last) = cumulative.last_mut() {
        *last = 1.0;
    }
    cumulative
}

/// Generate xoshiro128** seed array: `n_reps * 4` u32 values.
#[cfg(feature = "barracuda-gpu")]
fn generate_xoshiro_seeds(n_reps: usize, base_seed: u64) -> Vec<u32> {
    use crate::prng::Xorshift64;
    let mut rng = Xorshift64::new(base_seed);
    let mut seeds = Vec::with_capacity(n_reps * 4);
    for _ in 0..n_reps * 4 {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "RNG u64 → u32 seed; high bits discarded intentionally"
        )]
        let s = rng.next_u64() as u32;
        seeds.push(s.max(1));
    }
    seeds
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tol;

    #[test]
    fn chao1_all_abundant() {
        let counts = [100, 200, 300, 400];
        assert!((chao1(&counts) - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn chao1_with_singletons_and_doubletons() {
        let counts = [100, 50, 2, 2, 1, 1, 1];
        let s_obs = 7.0;
        let f1 = 3.0;
        let f2 = 2.0;
        let expected = s_obs + f1 * f1 / (2.0 * f2);
        assert!((chao1(&counts) - expected).abs() < tol::ANALYTICAL);
    }

    #[test]
    fn chao1_no_doubletons() {
        let counts = [100, 50, 1, 1, 1];
        let s_obs = 5.0;
        let f1 = 3.0;
        let expected = s_obs + f1 * (f1 - 1.0) / 2.0;
        assert!((chao1(&counts) - expected).abs() < tol::ANALYTICAL);
    }

    #[test]
    fn chao1_empty() {
        let counts: [u64; 0] = [];
        assert!(chao1(&counts).abs() < f64::EPSILON);
    }

    #[test]
    fn detection_power_zero_abundance() {
        assert!(detection_power(0.0, 1000).abs() < f64::EPSILON);
    }

    #[test]
    fn detection_power_certain_at_full_abundance() {
        assert!((detection_power(1.0, 1) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn detection_power_known_value() {
        let p = detection_power(0.003, 998);
        assert!(p > 0.94 && p < 0.96, "P ≈ 0.95 at threshold depth");
    }

    #[test]
    fn detection_threshold_known_value() {
        assert_eq!(detection_threshold(0.003, 0.95), 998);
        assert_eq!(detection_threshold(0.004, 0.95), 748);
    }

    #[test]
    fn detection_threshold_monotone() {
        let d1 = detection_threshold(0.003, 0.95);
        let d2 = detection_threshold(0.004, 0.95);
        let d3 = detection_threshold(0.008, 0.95);
        assert!(d1 > d2 && d2 > d3);
    }

    #[test]
    fn abundance_occupancy_deterministic() {
        let community = vec![0.5, 0.3, 0.15, 0.05];
        let a = abundance_occupancy(&community, 100, 10, 42);
        let b = abundance_occupancy(&community, 100, 10, 42);
        assert_eq!(a, b);
    }

    #[test]
    fn singleton_fraction_decreases_with_depth() {
        let community = vec![0.4, 0.3, 0.2, 0.05, 0.03, 0.02];
        let sf_low = singleton_fraction(&community, 50, 20, 42);
        let sf_high = singleton_fraction(&community, 5000, 20, 42);
        assert!(
            sf_low > sf_high,
            "singleton fraction should decrease with depth"
        );
    }

    #[test]
    fn mean_chao1_deterministic() {
        let community = vec![0.5, 0.3, 0.2];
        let (c1, s1) = mean_chao1_at_depth(&community, 100, 10, 42);
        let (c2, s2) = mean_chao1_at_depth(&community, 100, 10, 42);
        assert!((c1 - c2).abs() < f64::EPSILON);
        assert!((s1 - s2).abs() < f64::EPSILON);
    }

    #[test]
    fn tier_detection_rate_deterministic() {
        let community = vec![0.5, 0.3, 0.15, 0.04, 0.01];
        let r1 = tier_detection_rate(&community, 0, 3, 200, 10, 42);
        let r2 = tier_detection_rate(&community, 0, 3, 200, 10, 42);
        assert!((r1 - r2).abs() < f64::EPSILON);
    }

    #[test]
    fn tier_detection_rate_abundant_near_one() {
        let community = vec![0.5, 0.3, 0.15, 0.04, 0.01];
        let rate = tier_detection_rate(&community, 0, 3, 5000, 50, 42);
        assert!(
            rate > 0.95,
            "abundant species should be detected, rate={rate}"
        );
    }

    #[test]
    fn tier_detection_rate_rare_lower() {
        let community = vec![0.5, 0.3, 0.15, 0.04, 0.01];
        let abundant = tier_detection_rate(&community, 0, 3, 20, 50, 42);
        let rare = tier_detection_rate(&community, 3, 5, 20, 50, 42);
        assert!(
            abundant >= rare,
            "abundant tier ({abundant}) should be >= rare ({rare})"
        );
    }

    #[test]
    fn detection_threshold_edge_cases() {
        assert_eq!(detection_threshold(0.0, 0.95), 0);
        assert_eq!(detection_threshold(1.0, 0.95), 0);
    }

    #[test]
    fn chao1_only_singletons() {
        let counts = [1, 1, 1, 0, 0];
        let est = chao1(&counts);
        let s_obs = 3.0;
        let f1 = 3.0;
        let expected = s_obs + f1 * (f1 - 1.0) / 2.0;
        assert!((est - expected).abs() < 1e-10);
    }
}
