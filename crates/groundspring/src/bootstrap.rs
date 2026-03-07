// SPDX-License-Identifier: AGPL-3.0-only
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
/// When `barracuda-gpu` is enabled, dispatches via `BootstrapMeanGpu`
/// for parallel resample computation on GPU. Falls back to
/// `barracuda::stats::bootstrap_mean` (CPU), then to a local
/// implementation.
///
/// # Panics
///
/// Panics if `data` is empty or `confidence` is outside (0, 1).
///
/// # Examples
///
/// ```
/// let data: Vec<f64> = (0..100).map(|i| f64::from(i) * 0.01).collect();
/// let ci = groundspring::bootstrap::bootstrap_mean(&data, 500, 0.05, 42);
/// assert!(ci.ci_lower < ci.ci_upper);
/// assert!(ci.ci_lower <= ci.estimate && ci.estimate <= ci.ci_upper);
/// ```
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

    #[cfg(feature = "barracuda-gpu")]
    {
        if let Some(result) = bootstrap_mean_gpu(data, n_replicates, confidence, seed) {
            return result;
        }
    }

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

#[cfg(feature = "barracuda-gpu")]
fn bootstrap_mean_gpu(
    data: &[f64],
    n_replicates: usize,
    confidence: f64,
    seed: u64,
) -> Option<BootstrapResult> {
    let device = crate::gpu::get_device_f64_safe()?;
    let gpu = barracuda::stats::bootstrap::BootstrapMeanGpu::new(device).ok()?;
    #[expect(
        clippy::cast_possible_truncation,
        reason = "n_replicates and seed fit in u32 for GPU dispatch"
    )]
    let means = gpu.dispatch(data, n_replicates as u32, seed as u32).ok()?;
    Some(percentile_ci(&means, means.len(), confidence))
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
/// `barracuda::stats::rawr_mean` (absorbed in barraCuda S66).
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

/// Percentile bootstrap confidence interval for the median.
///
/// More robust than [`bootstrap_mean`] for skewed or heavy-tailed data.
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::stats::bootstrap_median` (absorbed in barraCuda S64).
///
/// # Panics
///
/// Panics if `data` is empty or `confidence` is outside (0, 1).
#[must_use]
pub fn bootstrap_median(
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
        if let Ok(ci) = barracuda::stats::bootstrap_median(data, n_replicates, confidence, seed) {
            return BootstrapResult {
                estimate: ci.estimate,
                ci_lower: ci.lower,
                ci_upper: ci.upper,
                std_error: ci.std_error,
            };
        }
    }

    bootstrap_median_cpu(data, n_replicates, confidence, seed)
}

fn bootstrap_median_cpu(
    data: &[f64],
    n_replicates: usize,
    confidence: f64,
    seed: u64,
) -> BootstrapResult {
    let n = data.len();
    let mut rng = Xorshift64::new(seed);
    let mut medians = Vec::with_capacity(n_replicates);
    let mut resample = Vec::with_capacity(n);

    for _ in 0..n_replicates {
        resample.clear();
        for _ in 0..n {
            #[expect(
                clippy::cast_possible_truncation,
                reason = "n fits in u64 on all targets"
            )]
            let idx = (rng.next_u64() % (n as u64)) as usize;
            resample.push(data[idx]);
        }
        resample.sort_unstable_by(f64::total_cmp);
        let median = if n.is_multiple_of(2) {
            f64::midpoint(resample[n / 2 - 1], resample[n / 2])
        } else {
            resample[n / 2]
        };
        medians.push(median);
    }

    percentile_ci(&medians, n_replicates, confidence)
}

/// Percentile bootstrap confidence interval for the standard deviation.
///
/// Useful for quantifying uncertainty in variability estimates from
/// small samples (common in field ecology and lattice QCD).
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::stats::bootstrap_std` (absorbed in barraCuda S64).
///
/// # Panics
///
/// Panics if `data` has fewer than 2 elements or `confidence` is outside (0, 1).
#[must_use]
pub fn bootstrap_std(
    data: &[f64],
    n_replicates: usize,
    confidence: f64,
    seed: u64,
) -> BootstrapResult {
    assert!(data.len() >= 2, "need at least 2 data points for std");
    assert!(
        (0.0..1.0).contains(&(1.0 - confidence)),
        "confidence must be in (0, 1)"
    );

    #[cfg(feature = "barracuda")]
    {
        if let Ok(ci) = barracuda::stats::bootstrap_std(data, n_replicates, confidence, seed) {
            return BootstrapResult {
                estimate: ci.estimate,
                ci_lower: ci.lower,
                ci_upper: ci.upper,
                std_error: ci.std_error,
            };
        }
    }

    bootstrap_std_cpu(data, n_replicates, confidence, seed)
}

fn bootstrap_std_cpu(
    data: &[f64],
    n_replicates: usize,
    confidence: f64,
    seed: u64,
) -> BootstrapResult {
    let n = data.len();
    let n_f = crate::cast::usize_f64(n);
    let mut rng = Xorshift64::new(seed);
    let mut stds = Vec::with_capacity(n_replicates);

    let mut resample_buf = Vec::with_capacity(n);
    for _ in 0..n_replicates {
        resample_buf.clear();
        for _ in 0..n {
            #[expect(
                clippy::cast_possible_truncation,
                reason = "n fits in u64 on all targets"
            )]
            let idx = (rng.next_u64() % (n as u64)) as usize;
            resample_buf.push(data[idx]);
        }
        let sample_mean = resample_buf.iter().sum::<f64>() / n_f;
        let var = resample_buf
            .iter()
            .map(|&x| (x - sample_mean).powi(2))
            .sum::<f64>()
            / n_f;
        stds.push(var.sqrt());
    }

    percentile_ci(&stds, n_replicates, confidence)
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
#[expect(clippy::float_cmp, reason = "bitwise determinism test")]
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
    fn bootstrap_median_deterministic() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let r1 = bootstrap_median(&data, 500, 0.95, 42);
        let r2 = bootstrap_median(&data, 500, 0.95, 42);
        assert_eq!(r1.estimate, r2.estimate);
        assert_eq!(r1.ci_lower, r2.ci_lower);
    }

    #[test]
    fn bootstrap_median_robust_to_outlier() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 1000.0];
        let median_r = bootstrap_median(&data, 1000, 0.95, 42);
        let mean_r = bootstrap_mean(&data, 1000, 0.95, 42);
        assert!(
            median_r.estimate < mean_r.estimate,
            "median ({}) should be less than mean ({}) with outlier",
            median_r.estimate,
            mean_r.estimate
        );
    }

    #[test]
    fn bootstrap_std_deterministic() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let r1 = bootstrap_std(&data, 500, 0.95, 42);
        let r2 = bootstrap_std(&data, 500, 0.95, 42);
        assert_eq!(r1.estimate, r2.estimate);
    }

    #[test]
    fn bootstrap_std_positive() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let r = bootstrap_std(&data, 500, 0.95, 42);
        assert!(
            r.estimate > 0.0,
            "std should be positive, got {}",
            r.estimate
        );
        assert!(
            r.ci_lower < r.ci_upper,
            "CI should have width: [{}, {}]",
            r.ci_lower,
            r.ci_upper
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

    #[test]
    fn bootstrap_mean_single_value() {
        let data = vec![7.0];
        let r = bootstrap_mean(&data, 200, 0.95, 42);
        assert!(
            (r.estimate - 7.0).abs() < 1e-12,
            "single-value bootstrap mean should be 7.0"
        );
        assert!(r.std_error < 1e-12, "single-value bootstrap has zero SE");
    }

    #[test]
    fn rawr_ci_width_positive() {
        let data: Vec<f64> = (0..50).map(f64::from).collect();
        let r = rawr_mean(&data, 500, 0.95, 42);
        assert!(r.ci_upper > r.ci_lower, "RAWR CI width must be positive");
        assert!(r.std_error > 0.0);
    }

    #[test]
    fn bootstrap_std_ci_contains_analytical() {
        let data: Vec<f64> = (1..=100).map(f64::from).collect();
        let r = bootstrap_std(&data, 1000, 0.95, 42);
        let analytical_std = 29.01; // std of 1..100 ≈ 29.01
        assert!(
            r.ci_lower < analytical_std && analytical_std < r.ci_upper,
            "CI [{}, {}] should contain analytical std ~{analytical_std}",
            r.ci_lower,
            r.ci_upper,
        );
    }

    #[test]
    fn bootstrap_median_ci_contains_analytical() {
        let data: Vec<f64> = (1..=99).map(f64::from).collect();
        let r = bootstrap_median(&data, 1000, 0.95, 42);
        assert!(
            r.ci_lower < 50.0 && 50.0 < r.ci_upper,
            "CI [{}, {}] should contain 50.0",
            r.ci_lower,
            r.ci_upper,
        );
    }

    #[test]
    fn rawr_different_seeds_differ() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let r1 = rawr_mean(&data, 500, 0.95, 42);
        let r2 = rawr_mean(&data, 500, 0.95, 99);
        assert!(
            r1.ci_lower.to_bits() != r2.ci_lower.to_bits()
                || r1.ci_upper.to_bits() != r2.ci_upper.to_bits()
                || r1.estimate.to_bits() != r2.estimate.to_bits(),
            "at least one field should differ between seeds"
        );
    }

    #[test]
    fn bootstrap_mean_cpu_direct() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let r = bootstrap_mean_cpu(&data, 200, 0.95, 42);
        assert!(r.ci_lower < r.ci_upper);
        assert!((r.estimate - 3.0).abs() < 1.0);
    }

    #[test]
    fn rawr_mean_cpu_direct() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let r = rawr_mean_cpu(&data, 200, 0.95, 42);
        assert!(r.ci_lower < r.ci_upper);
        assert!((r.estimate - 3.0).abs() < 1.0);
    }

    #[test]
    fn bootstrap_median_cpu_direct() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let r = bootstrap_median_cpu(&data, 200, 0.95, 42);
        assert!(r.ci_lower <= r.estimate);
        assert!(r.estimate <= r.ci_upper);
    }

    #[test]
    fn bootstrap_std_cpu_direct() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let r = bootstrap_std_cpu(&data, 200, 0.95, 42);
        assert!(r.estimate > 0.0);
        assert!(r.ci_lower > 0.0);
    }

    #[test]
    fn percentile_ci_known_values() {
        let data: Vec<f64> = (0..100).map(f64::from).collect();
        let r = percentile_ci(&data, 100, 0.95);
        assert!((r.estimate - 49.5).abs() < 0.01);
        assert!(r.ci_lower < r.ci_upper);
        assert!(r.std_error > 0.0);
    }

    #[test]
    fn bootstrap_mean_large_sample() {
        let data: Vec<f64> = (0..500).map(|i| f64::from(i) * 0.1).collect();
        let r = bootstrap_mean(&data, 200, 0.95, 7);
        assert!((r.estimate - 24.95).abs() < 2.0);
        assert!(r.std_error > 0.0);
    }

    #[test]
    fn rawr_single_value() {
        let data = vec![42.0];
        let r = rawr_mean(&data, 200, 0.95, 1);
        assert!((r.estimate - 42.0).abs() < 1e-12);
    }

    #[test]
    fn bootstrap_median_even_length() {
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let r = bootstrap_median(&data, 500, 0.95, 42);
        assert!(r.ci_lower <= r.estimate);
        assert!(r.estimate <= r.ci_upper);
    }

    #[test]
    fn bootstrap_std_uniform_data() {
        let data = vec![5.0; 20];
        let r = bootstrap_std(&data, 200, 0.95, 42);
        assert!(r.estimate < 1e-12, "std of constant data should be ~0");
    }

    #[test]
    fn rawr_ci_narrows_with_n() {
        let mut rng = crate::prng::Xorshift64::new(77);
        let small: Vec<f64> = (0..20).map(|_| rng.next_f64() * 10.0).collect();
        let large: Vec<f64> = (0..200).map(|_| rng.next_f64() * 10.0).collect();
        let r_small = rawr_mean(&small, 500, 0.95, 42);
        let r_large = rawr_mean(&large, 500, 0.95, 42);
        assert!(
            r_large.ci_upper - r_large.ci_lower < r_small.ci_upper - r_small.ci_lower,
            "larger sample should have narrower RAWR CI"
        );
    }

    #[test]
    fn bootstrap_confidence_level_90() {
        let data: Vec<f64> = (1..=100).map(f64::from).collect();
        let r90 = bootstrap_mean(&data, 1000, 0.90, 42);
        let r95 = bootstrap_mean(&data, 1000, 0.95, 42);
        assert!(
            r90.ci_upper - r90.ci_lower <= r95.ci_upper - r95.ci_lower + 1.0,
            "90% CI should generally be narrower than 95% CI"
        );
    }
}
