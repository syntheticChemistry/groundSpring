// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Percentile bootstrap resampling for confidence interval estimation.
//!
//! Implements the percentile bootstrap (Efron 1979) for mean, median, and
//! standard deviation. RAWR resampling lives in [`crate::rawr`].
//!
//! # barracuda delegation
//!
//! When the `barracuda` feature is enabled:
//! - `bootstrap_mean` delegates to `barracuda::stats::bootstrap_mean()`

use crate::prng::Xorshift64;

pub use crate::rawr::rawr_mean;

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

/// Validate common bootstrap preconditions.
///
/// `min_len` is the minimum data length (1 for mean/median/RAWR, 2 for std).
pub(crate) fn validate_bootstrap_inputs(
    data: &[f64],
    min_len: usize,
    confidence: f64,
) -> Result<(), crate::error::InputError> {
    if data.len() < min_len {
        return Err(crate::error::InputError::InsufficientData {
            name: "data",
            min: min_len,
            got: data.len(),
        });
    }
    if !(0.0..1.0).contains(&(1.0 - confidence)) {
        return Err(crate::error::InputError::OutOfRange {
            name: "confidence",
            lo: 0.0,
            hi: 1.0,
            got: confidence,
        });
    }
    Ok(())
}

/// Map a barracuda `BootstrapCI` to our `BootstrapResult`.
#[cfg(feature = "barracuda")]
pub(crate) const fn from_barracuda_ci(ci: &barracuda::stats::bootstrap::BootstrapCI) -> BootstrapResult {
    BootstrapResult {
        estimate: ci.estimate,
        ci_lower: ci.lower,
        ci_upper: ci.upper,
        std_error: ci.std_error,
    }
}

/// Standard percentile bootstrap confidence interval for the mean.
///
/// When `barracuda-gpu` is enabled, dispatches via `BootstrapMeanGpu`
/// for parallel resample computation on GPU. Falls back to
/// `barracuda::stats::bootstrap_mean` (CPU), then to a local
/// implementation.
///
/// # Errors
///
/// Returns [`InputError`](crate::error::InputError) if `data` is empty
/// or `confidence` is outside (0, 1).
///
/// # Examples
///
/// ```
/// let data: Vec<f64> = (0..100).map(|i| f64::from(i) * 0.01).collect();
/// let ci = groundspring::bootstrap::bootstrap_mean(&data, 500, 0.05, 42).unwrap();
/// assert!(ci.ci_lower < ci.ci_upper);
/// assert!(ci.ci_lower <= ci.estimate && ci.estimate <= ci.ci_upper);
/// ```
pub fn bootstrap_mean(
    data: &[f64],
    n_replicates: usize,
    confidence: f64,
    seed: u64,
) -> Result<BootstrapResult, crate::error::InputError> {
    validate_bootstrap_inputs(data, 1, confidence)?;

    #[cfg(feature = "barracuda-gpu")]
    {
        if let Some(result) = bootstrap_mean_gpu(data, n_replicates, confidence, seed) {
            return Ok(result);
        }
    }

    #[cfg(feature = "barracuda")]
    {
        if let Ok(ci) = barracuda::stats::bootstrap_mean(data, n_replicates, confidence, seed) {
            return Ok(from_barracuda_ci(&ci));
        }
    }

    Ok(bootstrap_mean_cpu(data, n_replicates, confidence, seed))
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
            let idx = (rng.next_u64() % crate::cast::usize_u64(n)) as usize;
            sum += data[idx];
        }
        means.push(sum / crate::cast::usize_f64(n));
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
/// # Errors
///
/// Returns [`InputError`](crate::error::InputError) if `data` is empty
/// or `confidence` is outside (0, 1).
pub fn bootstrap_median(
    data: &[f64],
    n_replicates: usize,
    confidence: f64,
    seed: u64,
) -> Result<BootstrapResult, crate::error::InputError> {
    validate_bootstrap_inputs(data, 1, confidence)?;

    #[cfg(feature = "barracuda")]
    {
        if let Ok(ci) = barracuda::stats::bootstrap_median(data, n_replicates, confidence, seed) {
            return Ok(from_barracuda_ci(&ci));
        }
    }

    Ok(bootstrap_median_cpu(data, n_replicates, confidence, seed))
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
            let idx = (rng.next_u64() % crate::cast::usize_u64(n)) as usize;
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
/// # Errors
///
/// Returns [`InputError`](crate::error::InputError) if `data` has fewer
/// than 2 elements or `confidence` is outside (0, 1).
pub fn bootstrap_std(
    data: &[f64],
    n_replicates: usize,
    confidence: f64,
    seed: u64,
) -> Result<BootstrapResult, crate::error::InputError> {
    validate_bootstrap_inputs(data, 2, confidence)?;

    #[cfg(feature = "barracuda")]
    {
        if let Ok(ci) = barracuda::stats::bootstrap_std(data, n_replicates, confidence, seed) {
            return Ok(from_barracuda_ci(&ci));
        }
    }

    Ok(bootstrap_std_cpu(data, n_replicates, confidence, seed))
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
            let idx = (rng.next_u64() % crate::cast::usize_u64(n)) as usize;
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
pub(crate) fn percentile_ci(means: &[f64], n_replicates: usize, confidence: f64) -> BootstrapResult {
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
#[expect(
    clippy::float_cmp,
    reason = "bitwise determinism test"
)]
#[expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_deterministic() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let r1 = bootstrap_mean(&data, 500, 0.95, 42).unwrap();
        let r2 = bootstrap_mean(&data, 500, 0.95, 42).unwrap();
        assert_eq!(r1.estimate, r2.estimate);
        assert_eq!(r1.ci_lower, r2.ci_lower);
    }

    #[test]
    fn bootstrap_ci_contains_true_mean() {
        let mut rng = Xorshift64::new(42);
        let data: Vec<f64> = (0..200)
            .map(|_| (rng.next_f64() - 0.5).mul_add(4.0, 5.0))
            .collect();
        let r = bootstrap_mean(&data, 1000, 0.95, 123).unwrap();
        assert!(
            r.ci_lower <= 5.0 && 5.0 <= r.ci_upper,
            "CI [{}, {}] should contain 5.0",
            r.ci_lower,
            r.ci_upper
        );
    }

    #[test]
    fn bootstrap_median_deterministic() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let r1 = bootstrap_median(&data, 500, 0.95, 42).unwrap();
        let r2 = bootstrap_median(&data, 500, 0.95, 42).unwrap();
        assert_eq!(r1.estimate, r2.estimate);
        assert_eq!(r1.ci_lower, r2.ci_lower);
    }

    #[test]
    fn bootstrap_median_robust_to_outlier() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 1000.0];
        let median_r = bootstrap_median(&data, 1000, 0.95, 42).unwrap();
        let mean_r = bootstrap_mean(&data, 1000, 0.95, 42).unwrap();
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
        let r1 = bootstrap_std(&data, 500, 0.95, 42).unwrap();
        let r2 = bootstrap_std(&data, 500, 0.95, 42).unwrap();
        assert_eq!(r1.estimate, r2.estimate);
    }

    #[test]
    fn bootstrap_std_positive() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let r = bootstrap_std(&data, 500, 0.95, 42).unwrap();
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
        let r_small = bootstrap_mean(&data_small, 1000, 0.95, 42).unwrap();
        let r_large = bootstrap_mean(&data_large, 1000, 0.95, 42).unwrap();
        assert!(
            r_large.ci_upper - r_large.ci_lower < r_small.ci_upper - r_small.ci_lower,
            "larger sample should have narrower CI"
        );
    }

    #[test]
    fn bootstrap_mean_single_value() {
        let data = vec![7.0];
        let r = bootstrap_mean(&data, 200, 0.95, 42).unwrap();
        assert!(
            (r.estimate - 7.0).abs() < 1e-12,
            "single-value bootstrap mean should be 7.0"
        );
        assert!(r.std_error < 1e-12, "single-value bootstrap has zero SE");
    }

    #[test]
    fn bootstrap_std_ci_contains_analytical() {
        let data: Vec<f64> = (1..=100).map(f64::from).collect();
        let r = bootstrap_std(&data, 1000, 0.95, 42).unwrap();
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
        let r = bootstrap_median(&data, 1000, 0.95, 42).unwrap();
        assert!(
            r.ci_lower < 50.0 && 50.0 < r.ci_upper,
            "CI [{}, {}] should contain 50.0",
            r.ci_lower,
            r.ci_upper,
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
        let r = bootstrap_mean(&data, 200, 0.95, 7).unwrap();
        assert!((r.estimate - 24.95).abs() < 2.0);
        assert!(r.std_error > 0.0);
    }

    #[test]
    fn bootstrap_median_even_length() {
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let r = bootstrap_median(&data, 500, 0.95, 42).unwrap();
        assert!(r.ci_lower <= r.estimate);
        assert!(r.estimate <= r.ci_upper);
    }

    #[test]
    fn bootstrap_std_uniform_data() {
        let data = vec![5.0; 20];
        let r = bootstrap_std(&data, 200, 0.95, 42).unwrap();
        assert!(r.estimate < 1e-12, "std of constant data should be ~0");
    }

    #[test]
    fn bootstrap_confidence_level_90() {
        let data: Vec<f64> = (1..=100).map(f64::from).collect();
        let r90 = bootstrap_mean(&data, 1000, 0.90, 42).unwrap();
        let r95 = bootstrap_mean(&data, 1000, 0.95, 42).unwrap();
        assert!(
            r90.ci_upper - r90.ci_lower <= r95.ci_upper - r95.ci_lower + 1.0,
            "90% CI should generally be narrower than 95% CI"
        );
    }

    #[test]
    fn bootstrap_empty_data_returns_error() {
        let empty: Vec<f64> = vec![];
        assert!(bootstrap_mean(&empty, 100, 0.95, 42).is_err());
        assert!(bootstrap_median(&empty, 100, 0.95, 42).is_err());
    }

    #[test]
    fn bootstrap_bad_confidence_returns_error() {
        let data = vec![1.0, 2.0, 3.0];
        assert!(bootstrap_mean(&data, 100, 1.5, 42).is_err());
        assert!(bootstrap_mean(&data, 100, -0.1, 42).is_err());
    }

    #[test]
    fn bootstrap_std_needs_two_elements() {
        let one = vec![1.0];
        assert!(bootstrap_std(&one, 100, 0.95, 42).is_err());
        let two = vec![1.0, 2.0];
        assert!(bootstrap_std(&two, 100, 0.95, 42).is_ok());
    }
}
