// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Community diversity indices: Simpson, Shannon, Bray-Curtis, evenness.
//!
//! These are pure metric functions operating on count or abundance vectors.
//! Used by rarefaction analysis and available independently for any
//! community comparison workload.

use crate::cast::u64_f64;
#[cfg(not(feature = "barracuda"))]
use crate::cast::usize_f64;

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
    let device = crate::gpu::get_device_f64_safe()?;
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
    let device = crate::gpu::get_device_f64_safe()?;
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
}
