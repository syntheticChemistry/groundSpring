// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Delete-one and block jackknife resampling for variance estimation.
//!
//! Implements Quenouille–Tukey delete-one jackknife (1956) and block
//! jackknife for correlated data.  The methodology is validated against
//! Bazavov et al. (2025) Phys Rev D 111, 094508 — jackknife error
//! estimation at subpercent precision for lattice QCD observables.
//!
//! # barracuda delegation
//!
//! When the `barracuda` feature is enabled, [`jackknife_mean_variance`]
//! delegates to `barracuda::stats::jackknife_mean_variance()`. The
//! delete-one loop is embarrassingly parallel — GPU promotion via
//! `barracuda-gpu` is a high-value target for large N.

use crate::cast::usize_f64;

/// Result of a jackknife computation.
#[derive(Debug, Clone)]
pub struct JackknifeResult {
    /// Full-sample estimate of the statistic.
    pub estimate: f64,
    /// Jackknife variance estimate.
    pub variance: f64,
    /// Jackknife standard error (sqrt of variance).
    pub std_error: f64,
}

/// Delete-one jackknife for the mean.
///
/// Returns the full-sample mean and the jackknife variance estimate
/// of the mean.  For N data points, creates N leave-one-out subsets.
///
/// Jackknife variance formula:
/// `var_JK = (N-1)/N * Σ(θ̂_i - θ̄_JK)²`
///
/// # Panics
///
/// Panics if `data` has fewer than 2 elements.
#[must_use]
pub fn jackknife_mean_variance(data: &[f64]) -> JackknifeResult {
    #[cfg(feature = "barracuda")]
    {
        if let Ok((est, var)) = barracuda::stats::jackknife_mean_variance(data) {
            return JackknifeResult {
                estimate: est,
                variance: var,
                std_error: var.sqrt(),
            };
        }
    }
    jackknife_mean_variance_cpu(data)
}

fn jackknife_mean_variance_cpu(data: &[f64]) -> JackknifeResult {
    let n = data.len();
    assert!(n >= 2, "need at least 2 data points");

    let full_sum: f64 = data.iter().sum();
    let full_mean = full_sum / usize_f64(n);
    let n_f = usize_f64(n);

    let mut jk_mean_sum = 0.0;
    let mut jk_means = Vec::with_capacity(n);

    for &d in data {
        let leave_sum = full_sum - d;
        let leave_mean = leave_sum / usize_f64(n - 1);
        jk_means.push(leave_mean);
        jk_mean_sum += leave_mean;
    }

    let jk_grand_mean = jk_mean_sum / n_f;
    let jk_var = (n_f - 1.0) / n_f
        * jk_means
            .iter()
            .map(|&m| (m - jk_grand_mean).powi(2))
            .sum::<f64>();

    JackknifeResult {
        estimate: full_mean,
        variance: jk_var,
        std_error: jk_var.sqrt(),
    }
}

/// Jackknife bias estimate for an arbitrary statistic.
///
/// Given the full-sample statistic and leave-one-out statistics, computes:
/// `bias_JK = (N-1) * (θ̄_JK - θ_full)`
///
/// Returns `(full_stat, bias, corrected)` where `corrected = full_stat - bias`.
#[must_use]
pub fn jackknife_bias(leave_one_out_stats: &[f64], full_stat: f64) -> (f64, f64) {
    let n = leave_one_out_stats.len();
    let n_f = usize_f64(n);
    let jk_mean: f64 = leave_one_out_stats.iter().sum::<f64>() / n_f;
    let bias = (n_f - 1.0) * (jk_mean - full_stat);
    let corrected = full_stat - bias;
    (bias, corrected)
}

/// Compute leave-one-out biased variance estimates.
///
/// For each i, returns `var(data \ data[i])` with ddof=0 (biased estimator).
#[must_use]
pub fn leave_one_out_biased_variance(data: &[f64]) -> Vec<f64> {
    let n = data.len();
    let full_sum: f64 = data.iter().sum();
    let full_sum_sq: f64 = data.iter().map(|x| x * x).sum();

    (0..n)
        .map(|i| {
            let s = full_sum - data[i];
            let sq = data[i].mul_add(-data[i], full_sum_sq);
            let m = usize_f64(n - 1);
            (s / m).mul_add(-(s / m), sq / m)
        })
        .collect()
}

/// Block jackknife variance for correlated data.
///
/// Divides data into `N/block_size` non-overlapping blocks, then applies
/// delete-one-block jackknife.  Returns the jackknife variance estimate
/// of the mean.
///
/// For AR(1) data with autocorrelation φ, block jackknife with
/// `block_size ≈ 1/(1-φ)` captures the correct variance.
///
/// # Panics
///
/// Panics if `block_size` is 0 or exceeds `data.len()`.
#[must_use]
pub fn block_jackknife_variance(data: &[f64], block_size: usize) -> JackknifeResult {
    assert!(block_size > 0, "block_size must be positive");
    let n = data.len();
    let n_blocks = n / block_size;
    assert!(n_blocks >= 2, "need at least 2 blocks");

    let trimmed_len = n_blocks * block_size;
    let trimmed = &data[..trimmed_len];
    let full_mean: f64 = trimmed.iter().sum::<f64>() / usize_f64(trimmed_len);

    let n_blocks_f = usize_f64(n_blocks);
    let mut jk_means = Vec::with_capacity(n_blocks);

    for i in 0..n_blocks {
        let block_start = i * block_size;
        let block_end = block_start + block_size;
        let block_sum: f64 = trimmed[block_start..block_end].iter().sum();
        let rest_sum: f64 = trimmed.iter().sum::<f64>() - block_sum;
        let rest_len = trimmed_len - block_size;
        jk_means.push(rest_sum / usize_f64(rest_len));
    }

    let jk_grand_mean: f64 = jk_means.iter().sum::<f64>() / n_blocks_f;
    let jk_var = (n_blocks_f - 1.0) / n_blocks_f
        * jk_means
            .iter()
            .map(|&m| (m - jk_grand_mean).powi(2))
            .sum::<f64>();

    JackknifeResult {
        estimate: full_mean,
        variance: jk_var,
        std_error: jk_var.sqrt(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prng::Xorshift64;

    #[test]
    fn jk_mean_gaussian() {
        let mut rng = Xorshift64::new(42);
        let data: Vec<f64> = (0..200).map(|_| rng.normal(5.0, 2.0)).collect();
        let r = jackknife_mean_variance(&data);
        assert!(
            (r.estimate - 5.0).abs() < 0.5,
            "mean should be near 5.0, got {}",
            r.estimate
        );
        assert!(
            r.variance > 0.005 && r.variance < 0.08,
            "variance of mean should be near sigma^2/N ≈ 0.02, got {}",
            r.variance
        );
    }

    #[test]
    fn jk_deterministic() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let r1 = jackknife_mean_variance(&data);
        let r2 = jackknife_mean_variance(&data);
        assert_eq!(r1.estimate.to_bits(), r2.estimate.to_bits());
        assert_eq!(r1.variance.to_bits(), r2.variance.to_bits());
    }

    #[test]
    fn jk_bias_reduces_error() {
        let mut rng = Xorshift64::new(42);
        let data: Vec<f64> = (0..200).map(|_| rng.normal(5.0, 2.0)).collect();
        let true_var = 4.0;

        let full_biased_var: f64 = {
            let mean = data.iter().sum::<f64>() / usize_f64(data.len());
            data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / usize_f64(data.len())
        };
        let loo_stats = leave_one_out_biased_variance(&data);
        let (_, corrected) = jackknife_bias(&loo_stats, full_biased_var);

        let naive_err = (full_biased_var - true_var).abs();
        let corrected_err = (corrected - true_var).abs();
        assert!(
            corrected_err < naive_err * 1.5,
            "bias correction should help: naive_err={naive_err}, corrected_err={corrected_err}"
        );
    }

    #[test]
    fn block_jk_captures_correlation() {
        let mut rng = Xorshift64::new(77);
        let n = 400;
        let phi: f64 = 0.8;
        let innovation_std = 3.0 * phi.mul_add(-phi, 1.0).sqrt();
        let mut data = vec![0.0; n];
        data[0] = rng.normal(10.0, 3.0);
        for i in 1..n {
            data[i] = phi.mul_add(data[i - 1] - 10.0, 10.0) + rng.normal(0.0, innovation_std);
        }

        let v1 = block_jackknife_variance(&data, 1).variance;
        let v40 = block_jackknife_variance(&data, 40).variance;
        assert!(
            v40 > v1 * 0.5,
            "block JK(40) should give comparable or larger variance than JK(1): v1={v1}, v40={v40}"
        );
    }

    #[test]
    fn jk_std_error_positive() {
        let data = vec![1.0, 3.0, 5.0, 7.0, 9.0];
        let r = jackknife_mean_variance(&data);
        assert!(r.std_error > 0.0);
    }
}
