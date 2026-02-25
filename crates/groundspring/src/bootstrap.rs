// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Bootstrap and RAWR resampling for confidence interval estimation.
//!
//! Implements the percentile bootstrap (Efron 1979) and
//! RAWR — Resampling with Analytical Weights for Reproducibility
//! (Wang et al. 2021, Bioinformatics/ISMB).
//!
//! # barracuda delegation
//!
//! When the `barracuda` feature is enabled, `bootstrap_mean` can delegate to
//! `barracuda::stats::bootstrap_ci()` for CPU or
//! `bootstrap_mean_f64.wgsl` for GPU.

use crate::prng::Xorshift64;

/// Result of a bootstrap or RAWR confidence interval computation.
#[derive(Debug, Clone)]
pub struct BootstrapResult {
    /// Point estimate (mean of bootstrap distribution).
    pub estimate: f64,
    /// Lower bound of the confidence interval.
    pub ci_lower: f64,
    /// Upper bound of the confidence interval.
    pub ci_upper: f64,
    /// Standard error of the bootstrap distribution.
    pub std_error: f64,
}

/// Standard percentile bootstrap confidence interval for the mean.
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::stats::bootstrap_mean` for the core computation.
///
/// # Panics
///
/// Panics if `data` is empty or `confidence` is outside (0, 1).
#[must_use]
pub fn bootstrap_mean(
    data: &[f64],
    n_replicates: usize,
    confidence: f64,
    seed: u64,
) -> BootstrapResult {
    assert!(!data.is_empty(), "data must not be empty");
    assert!(
        (0.0..1.0).contains(&(1.0 - confidence)),
        "confidence must be in (0, 1)"
    );

    #[cfg(feature = "barracuda")]
    {
        let ci = barracuda::stats::bootstrap_mean(data, n_replicates, confidence, seed)
            .expect("barracuda bootstrap_mean failed");
        BootstrapResult {
            estimate: ci.estimate,
            ci_lower: ci.lower,
            ci_upper: ci.upper,
            std_error: ci.std_error,
        }
    }

    #[cfg(not(feature = "barracuda"))]
    {
        bootstrap_mean_local(data, n_replicates, confidence, seed)
    }
}

#[cfg(not(feature = "barracuda"))]
fn bootstrap_mean_local(
    data: &[f64],
    n_replicates: usize,
    confidence: f64,
    seed: u64,
) -> BootstrapResult {
    let n = data.len();
    let mut rng = Xorshift64::new(seed);
    let mut means = Vec::with_capacity(n_replicates);

    for _ in 0..n_replicates {
        let mut sum = 0.0;
        for _ in 0..n {
            #[expect(clippy::cast_possible_truncation, reason = "n fits in u64 on all targets")]
            let idx = (rng.next_u64() % (n as u64)) as usize;
            sum += data[idx];
        }
        means.push(sum / crate::cast::usize_f64(n));
    }

    means.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let alpha = 1.0 - confidence;
    let lo_idx = crate::cast::f64_usize(alpha / 2.0 * crate::cast::usize_f64(n_replicates));
    let hi_idx = crate::cast::f64_usize(
        (1.0 - alpha / 2.0) * crate::cast::usize_f64(n_replicates),
    );
    let hi_idx = hi_idx.min(n_replicates - 1);

    let estimate: f64 = means.iter().sum::<f64>() / crate::cast::usize_f64(n_replicates);
    let variance: f64 = means
        .iter()
        .map(|&m| (m - estimate).powi(2))
        .sum::<f64>()
        / crate::cast::usize_f64(n_replicates);

    BootstrapResult {
        estimate,
        ci_lower: means[lo_idx],
        ci_upper: means[hi_idx],
        std_error: variance.sqrt(),
    }
}

/// RAWR (Bayesian bootstrap) confidence interval for the mean.
///
/// Generates Dirichlet(1,...,1) weights (via normalized Exp(1) variates)
/// and computes the weighted mean for each replicate.
///
/// # Panics
///
/// Panics if `data` is empty or `confidence` is outside (0, 1).
#[must_use]
pub fn rawr_mean(
    data: &[f64],
    n_replicates: usize,
    confidence: f64,
    seed: u64,
) -> BootstrapResult {
    assert!(!data.is_empty(), "data must not be empty");
    assert!(
        (0.0..1.0).contains(&(1.0 - confidence)),
        "confidence must be in (0, 1)"
    );

    let n = data.len();
    let mut rng = Xorshift64::new(seed);
    let mut means = Vec::with_capacity(n_replicates);

    for _ in 0..n_replicates {
        let mut weights = Vec::with_capacity(n);
        let mut wsum = 0.0;
        for _ in 0..n {
            let u = rng.next_f64();
            let w = if u > 0.0 { -u.ln() } else { 30.0 };
            weights.push(w);
            wsum += w;
        }

        let mut weighted_mean = 0.0;
        for (j, &d) in data.iter().enumerate() {
            weighted_mean = (weights[j] / wsum).mul_add(d, weighted_mean);
        }
        means.push(weighted_mean);
    }

    means.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let alpha = 1.0 - confidence;
    let lo_idx = crate::cast::f64_usize(alpha / 2.0 * crate::cast::usize_f64(n_replicates));
    let hi_idx = crate::cast::f64_usize(
        (1.0 - alpha / 2.0) * crate::cast::usize_f64(n_replicates),
    );
    let hi_idx = hi_idx.min(n_replicates - 1);

    let estimate: f64 = means.iter().sum::<f64>() / crate::cast::usize_f64(n_replicates);
    let variance: f64 = means
        .iter()
        .map(|&m| (m - estimate).powi(2))
        .sum::<f64>()
        / crate::cast::usize_f64(n_replicates);

    BootstrapResult {
        estimate,
        ci_lower: means[lo_idx],
        ci_upper: means[hi_idx],
        std_error: variance.sqrt(),
    }
}

#[cfg(test)]
#[expect(clippy::float_cmp)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_deterministic() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let r1 = bootstrap_mean(&data, 500, 0.95, 42);
        let r2 = bootstrap_mean(&data, 500, 0.95, 42);
        assert_eq!(r1.estimate, r2.estimate);
        assert_eq!(r1.ci_lower, r2.ci_lower);
    }

    #[test]
    fn rawr_deterministic() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let r1 = rawr_mean(&data, 500, 0.95, 42);
        let r2 = rawr_mean(&data, 500, 0.95, 42);
        assert_eq!(r1.estimate, r2.estimate);
    }

    #[test]
    fn bootstrap_ci_contains_true_mean() {
        let mut rng = Xorshift64::new(42);
        let data: Vec<f64> = (0..200)
            .map(|_| (rng.next_f64() - 0.5).mul_add(4.0, 5.0))
            .collect();
        let r = bootstrap_mean(&data, 1000, 0.95, 123);
        assert!(
            r.ci_lower <= 5.0 && 5.0 <= r.ci_upper,
            "CI [{}, {}] should contain 5.0",
            r.ci_lower,
            r.ci_upper
        );
    }

    #[test]
    fn rawr_ci_contains_true_mean() {
        let mut rng = Xorshift64::new(42);
        let data: Vec<f64> = (0..200)
            .map(|_| (rng.next_f64() - 0.5).mul_add(4.0, 5.0))
            .collect();
        let r = rawr_mean(&data, 1000, 0.95, 123);
        assert!(
            r.ci_lower <= 5.0 && 5.0 <= r.ci_upper,
            "RAWR CI [{}, {}] should contain 5.0",
            r.ci_lower,
            r.ci_upper
        );
    }

    #[test]
    fn bootstrap_different_from_rawr() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let b = bootstrap_mean(&data, 500, 0.95, 42);
        let r = rawr_mean(&data, 500, 0.95, 42);
        assert_ne!(b.estimate, r.estimate, "methods should produce different estimates");
    }

    #[test]
    fn ci_width_narrows_with_n() {
        let mut rng = Xorshift64::new(77);
        let data_small: Vec<f64> = (0..20).map(|_| rng.next_f64() * 10.0).collect();
        let data_large: Vec<f64> = (0..200).map(|_| rng.next_f64() * 10.0).collect();
        let r_small = bootstrap_mean(&data_small, 1000, 0.95, 42);
        let r_large = bootstrap_mean(&data_large, 1000, 0.95, 42);
        assert!(
            r_large.ci_upper - r_large.ci_lower < r_small.ci_upper - r_small.ci_lower,
            "larger sample should have narrower CI"
        );
    }
}
