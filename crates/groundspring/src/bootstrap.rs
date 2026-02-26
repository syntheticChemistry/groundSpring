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
//! When the `barracuda` feature is enabled:
//! - `bootstrap_mean` delegates to `barracuda::stats::bootstrap_mean()`
//! - `rawr_mean` delegates to `barracuda::stats::rawr_mean()` (since S66)

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
/// `barracuda::stats::bootstrap_mean`.
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
        if let Ok(ci) = barracuda::stats::bootstrap_mean(data, n_replicates, confidence, seed) {
            return BootstrapResult {
                estimate: ci.estimate,
                ci_lower: ci.lower,
                ci_upper: ci.upper,
                std_error: ci.std_error,
            };
        }
    }

    bootstrap_mean_cpu(data, n_replicates, confidence, seed)
}

fn bootstrap_mean_cpu(
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
            #[expect(
                clippy::cast_possible_truncation,
                reason = "n fits in u64 on all targets"
            )]
            let idx = (rng.next_u64() % (n as u64)) as usize;
            sum += data[idx];
        }
        means.push(sum / crate::cast::usize_f64(n));
    }

    percentile_ci(&means, n_replicates, confidence)
}

/// RAWR (Bayesian bootstrap) confidence interval for the mean.
///
/// Generates Dirichlet(1,...,1) weights (via normalized Exp(1) variates)
/// and computes the weighted mean for each replicate.
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::stats::rawr_mean` (absorbed in `ToadStool` S66).
///
/// # Panics
///
/// Panics if `data` is empty or `confidence` is outside (0, 1).
#[must_use]
pub fn rawr_mean(data: &[f64], n_replicates: usize, confidence: f64, seed: u64) -> BootstrapResult {
    assert!(!data.is_empty(), "data must not be empty");
    assert!(
        (0.0..1.0).contains(&(1.0 - confidence)),
        "confidence must be in (0, 1)"
    );

    #[cfg(feature = "barracuda")]
    {
        if let Ok(ci) = barracuda::stats::rawr_mean(data, n_replicates, confidence, seed) {
            return BootstrapResult {
                estimate: ci.estimate,
                ci_lower: ci.lower,
                ci_upper: ci.upper,
                std_error: ci.std_error,
            };
        }
    }

    rawr_mean_cpu(data, n_replicates, confidence, seed)
}

/// Cap for -ln(0) fallback when Exp(1) variate would be infinite.
const EXP_VARIATE_CAP: f64 = 30.0;

fn rawr_mean_cpu(data: &[f64], n_replicates: usize, confidence: f64, seed: u64) -> BootstrapResult {
    let n = data.len();
    let mut rng = Xorshift64::new(seed);
    let mut means = Vec::with_capacity(n_replicates);

    for _ in 0..n_replicates {
        let mut weights = Vec::with_capacity(n);
        let mut wsum = 0.0;
        for _ in 0..n {
            let u = rng.next_f64();
            let w = if u > 0.0 { -u.ln() } else { EXP_VARIATE_CAP };
            weights.push(w);
            wsum += w;
        }

        let mut weighted_mean = 0.0;
        for (j, &d) in data.iter().enumerate() {
            weighted_mean = (weights[j] / wsum).mul_add(d, weighted_mean);
        }
        means.push(weighted_mean);
    }

    percentile_ci(&means, n_replicates, confidence)
}

/// Compute the percentile confidence interval from a pre-filled
/// replicate distribution.  Shared by both bootstrap and RAWR.
fn percentile_ci(means: &[f64], n_replicates: usize, confidence: f64) -> BootstrapResult {
    let mut sorted: Vec<f64> = means.to_vec();
    sorted.sort_unstable_by(f64::total_cmp);

    let alpha = 1.0 - confidence;
    let n_f = crate::cast::usize_f64(n_replicates);
    let lo_idx = crate::cast::f64_usize(alpha / 2.0 * n_f);
    let hi_idx = crate::cast::f64_usize((1.0 - alpha / 2.0) * n_f).min(n_replicates - 1);

    let estimate: f64 = sorted.iter().sum::<f64>() / n_f;
    let variance: f64 = sorted.iter().map(|&m| (m - estimate).powi(2)).sum::<f64>() / n_f;

    BootstrapResult {
        estimate,
        ci_lower: sorted[lo_idx],
        ci_upper: sorted[hi_idx],
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
        let b_width = b.ci_upper - b.ci_lower;
        let r_width = r.ci_upper - r.ci_lower;
        assert!(
            (b.estimate - r.estimate).abs() < 0.5 || b_width != r_width,
            "methods should produce comparable estimates but may differ in CI width"
        );
        assert!(
            b_width > 0.0 && r_width > 0.0,
            "both methods must produce non-degenerate CIs"
        );
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
