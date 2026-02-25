// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Core statistical metrics shared across all groundSpring experiments.
//!
//! These are the canonical implementations of RMSE, MBE, R², and Index of
//! Agreement.  All functions operate on slices to enable zero-copy usage
//! from any data source.

use crate::cast::{f64_usize, usize_f64};

/// Root Mean Square Error between observed and modeled values.
///
/// Returns `0.0` for empty slices.
///
/// # Panics
///
/// Panics if `observed` and `modeled` have different lengths.
#[must_use]
pub fn rmse(observed: &[f64], modeled: &[f64]) -> f64 {
    assert_eq!(
        observed.len(),
        modeled.len(),
        "observed and modeled must have equal length"
    );
    let n = observed.len();
    if n == 0 {
        return 0.0;
    }
    let sum_sq: f64 = observed
        .iter()
        .zip(modeled)
        .map(|(o, m)| (o - m).powi(2))
        .sum();
    (sum_sq / usize_f64(n)).sqrt()
}

/// Mean Bias Error (modeled − observed).
///
/// Positive MBE indicates the model overestimates.
///
/// # Panics
///
/// Panics if `observed` and `modeled` have different lengths.
#[must_use]
pub fn mbe(observed: &[f64], modeled: &[f64]) -> f64 {
    assert_eq!(
        observed.len(),
        modeled.len(),
        "observed and modeled must have equal length"
    );
    let n = observed.len();
    if n == 0 {
        return 0.0;
    }
    let sum: f64 = observed.iter().zip(modeled).map(|(o, m)| m - o).sum();
    sum / usize_f64(n)
}

/// Coefficient of determination (R²).
///
/// Returns `0.0` when total sum of squares is zero (constant observation).
///
/// # Panics
///
/// Panics if `observed` and `modeled` have different lengths.
#[must_use]
pub fn r_squared(observed: &[f64], modeled: &[f64]) -> f64 {
    assert_eq!(
        observed.len(),
        modeled.len(),
        "observed and modeled must have equal length"
    );
    let n = observed.len();
    if n == 0 {
        return 0.0;
    }
    let mean_obs: f64 = observed.iter().sum::<f64>() / usize_f64(n);
    let ss_res: f64 = observed
        .iter()
        .zip(modeled)
        .map(|(o, m)| (o - m).powi(2))
        .sum();
    let ss_tot: f64 = observed.iter().map(|o| (o - mean_obs).powi(2)).sum();
    if ss_tot == 0.0 {
        return 0.0;
    }
    1.0 - ss_res / ss_tot
}

/// Index of Agreement (Willmott 1981).
///
/// Ranges from 0.0 (no agreement) to 1.0 (perfect agreement).
///
/// # Panics
///
/// Panics if `observed` and `modeled` have different lengths.
#[must_use]
pub fn index_of_agreement(observed: &[f64], modeled: &[f64]) -> f64 {
    assert_eq!(
        observed.len(),
        modeled.len(),
        "observed and modeled must have equal length"
    );
    let n = observed.len();
    if n == 0 {
        return 0.0;
    }
    let mean_obs: f64 = observed.iter().sum::<f64>() / usize_f64(n);
    let numerator: f64 = observed
        .iter()
        .zip(modeled)
        .map(|(o, m)| (o - m).powi(2))
        .sum();
    let denominator: f64 = observed
        .iter()
        .zip(modeled)
        .map(|(o, m)| ((m - mean_obs).abs() + (o - mean_obs).abs()).powi(2))
        .sum();
    if denominator == 0.0 {
        return 0.0;
    }
    1.0 - numerator / denominator
}

/// Arithmetic mean of a slice.
///
/// Returns `0.0` for empty slices.
#[must_use]
pub fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / usize_f64(values.len())
}

/// Population standard deviation (divides by N).
///
/// groundSpring uses population variance for total-population metrics like
/// RMSE decomposition.  For sample-based estimates, use [`sample_std_dev`].
#[must_use]
pub fn std_dev(values: &[f64]) -> f64 {
    let n = values.len();
    if n == 0 {
        return 0.0;
    }
    let m = mean(values);
    let variance = values.iter().map(|v| (v - m).powi(2)).sum::<f64>() / usize_f64(n);
    variance.sqrt()
}

/// Sample standard deviation (Bessel-corrected, divides by N−1).
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::stats::correlation::std_dev`.
/// Returns `0.0` for slices with fewer than 2 elements.
#[must_use]
pub fn sample_std_dev(values: &[f64]) -> f64 {
    #[cfg(feature = "barracuda")]
    {
        barracuda::stats::correlation::std_dev(values).unwrap_or(0.0)
    }
    #[cfg(not(feature = "barracuda"))]
    {
        let n = values.len();
        if n < 2 {
            return 0.0;
        }
        let m = mean(values);
        let variance = values.iter().map(|v| (v - m).powi(2)).sum::<f64>() / usize_f64(n - 1);
        variance.sqrt()
    }
}

/// Pearson correlation coefficient.
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::stats::pearson_correlation`.
/// Returns `0.0` when total sum of squares is zero for either variable.
///
/// # Panics
///
/// Panics if `x` and `y` have different lengths.
#[must_use]
pub fn pearson_r(x: &[f64], y: &[f64]) -> f64 {
    assert_eq!(x.len(), y.len(), "x and y must have equal length");
    #[cfg(feature = "barracuda")]
    {
        let r = barracuda::stats::pearson_correlation(x, y).unwrap_or(0.0);
        if r.is_nan() {
            0.0
        } else {
            r
        }
    }
    #[cfg(not(feature = "barracuda"))]
    {
        let n = x.len();
        if n == 0 {
            return 0.0;
        }
        let mx = mean(x);
        let my = mean(y);
        let mut cov = 0.0_f64;
        let mut vx = 0.0_f64;
        let mut vy = 0.0_f64;
        for (&xi, &yi) in x.iter().zip(y) {
            let dx = xi - mx;
            let dy = yi - my;
            cov += dx * dy;
            vx += dx * dx;
            vy += dy * dy;
        }
        let denom = (vx * vy).sqrt();
        if denom == 0.0 {
            0.0
        } else {
            cov / denom
        }
    }
}

/// Spearman rank correlation coefficient.
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::stats::spearman_correlation`.
/// Returns `0.0` for empty slices or when variance is zero.
///
/// # Panics
///
/// Panics if `x` and `y` have different lengths.
#[must_use]
pub fn spearman_r(x: &[f64], y: &[f64]) -> f64 {
    assert_eq!(x.len(), y.len(), "x and y must have equal length");
    #[cfg(feature = "barracuda")]
    {
        let r = barracuda::stats::correlation::spearman_correlation(x, y).unwrap_or(0.0);
        if r.is_nan() {
            0.0
        } else {
            r
        }
    }
    #[cfg(not(feature = "barracuda"))]
    {
        fn rank(data: &[f64]) -> Vec<f64> {
            let mut indexed: Vec<(usize, f64)> = data.iter().copied().enumerate().collect();
            indexed.sort_by(|a, b| f64::total_cmp(&a.1, &b.1));
            let mut ranks = vec![0.0; data.len()];
            let mut i = 0;
            while i < indexed.len() {
                let mut j = i;
                while j < indexed.len() && f64::total_cmp(&indexed[j].1, &indexed[i].1).is_eq() {
                    j += 1;
                }
                let avg_rank = usize_f64(i + j + 1) / 2.0;
                for item in &indexed[i..j] {
                    ranks[item.0] = avg_rank;
                }
                i = j;
            }
            ranks
        }
        let n = x.len();
        if n < 2 {
            return 0.0;
        }
        let rx = rank(x);
        let ry = rank(y);
        pearson_r(&rx, &ry)
    }
}

/// Fraction of days where observed and modeled agree on occurrence.
///
/// A day "occurs" if the value exceeds `threshold`.  Returns the
/// fraction of days where both agree (both above or both at-or-below).
///
/// Returns `0.0` for empty slices.
///
/// # Panics
///
/// Panics if `observed` and `modeled` have different lengths.
#[must_use]
pub fn hit_rate(observed: &[f64], modeled: &[f64], threshold: f64) -> f64 {
    assert_eq!(
        observed.len(),
        modeled.len(),
        "observed and modeled must have equal length"
    );
    let n = observed.len();
    if n == 0 {
        return 0.0;
    }
    let agree = observed
        .iter()
        .zip(modeled)
        .filter(|(&o, &m)| (o > threshold) == (m > threshold))
        .count();
    usize_f64(agree) / usize_f64(n)
}

/// Percentile of a sorted copy of `values` (0–100 scale).
///
/// # Panics
///
/// Panics if `p` is not in the range 0.0–100.0.
#[must_use]
pub fn percentile(values: &[f64], p: f64) -> f64 {
    assert!((0.0..=100.0).contains(&p), "percentile must be 0–100");
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted: Vec<f64> = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let rank = p / 100.0 * usize_f64(sorted.len() - 1);
    let lo = f64_usize(rank.floor());
    let hi = f64_usize(rank.ceil());
    if lo == hi {
        sorted[lo]
    } else {
        let frac = rank - usize_f64(lo);
        sorted[lo].mul_add(1.0 - frac, sorted[hi] * frac)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rmse_identical_is_zero() {
        let x = [1.0, 2.0, 3.0];
        assert!((rmse(&x, &x)).abs() < 1e-12);
    }

    #[test]
    fn rmse_known_value() {
        let obs = [1.0, 2.0, 3.0];
        let modeled = [1.1, 2.1, 3.1];
        assert!((rmse(&obs, &modeled) - 0.1).abs() < 1e-10);
    }

    #[test]
    fn mbe_overestimate_positive() {
        let obs = [1.0, 2.0, 3.0];
        let modeled = [1.5, 2.5, 3.5];
        assert!((mbe(&obs, &modeled) - 0.5).abs() < 1e-12);
    }

    #[test]
    fn r2_perfect() {
        let x = [1.0, 2.0, 3.0, 4.0];
        assert!((r_squared(&x, &x) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn r2_mean_model_is_zero() {
        let obs = [1.0, 2.0, 3.0];
        let modeled = [2.0, 2.0, 2.0];
        assert!(r_squared(&obs, &modeled).abs() < 1e-12);
    }

    #[test]
    fn ia_perfect() {
        let x = [1.0, 2.0, 3.0, 4.0];
        assert!((index_of_agreement(&x, &x) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn percentile_median() {
        let vals = [1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((percentile(&vals, 50.0) - 3.0).abs() < 1e-12);
    }

    #[test]
    fn hit_rate_perfect() {
        let obs = [0.0, 5.0, 0.0, 3.0];
        assert!((hit_rate(&obs, &obs, 0.1) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn hit_rate_known_value() {
        let obs = [0.0, 5.0, 0.0, 3.0];
        let modeled = [0.0, 4.0, 0.0, 0.0];
        assert!((hit_rate(&obs, &modeled, 0.1) - 0.75).abs() < 1e-12);
    }

    #[test]
    fn hit_rate_empty() {
        let empty: [f64; 0] = [];
        assert!(hit_rate(&empty, &empty, 0.1).abs() < 1e-12);
    }

    #[test]
    fn rmse_empty() {
        let empty: [f64; 0] = [];
        assert!(rmse(&empty, &empty).abs() < 1e-12);
    }

    #[test]
    fn mbe_empty() {
        let empty: [f64; 0] = [];
        assert!(mbe(&empty, &empty).abs() < 1e-12);
    }

    #[test]
    fn r2_empty() {
        let empty: [f64; 0] = [];
        assert!(r_squared(&empty, &empty).abs() < 1e-12);
    }

    #[test]
    fn r2_constant_observation() {
        let obs = [3.0, 3.0, 3.0];
        let modeled = [3.1, 2.9, 3.0];
        assert!(r_squared(&obs, &modeled).abs() < 1e-12);
    }

    #[test]
    fn ia_empty() {
        let empty: [f64; 0] = [];
        assert!(index_of_agreement(&empty, &empty).abs() < 1e-12);
    }

    #[test]
    fn ia_constant_denominator_zero() {
        let obs = [5.0, 5.0, 5.0];
        let modeled = [5.0, 5.0, 5.0];
        assert!((index_of_agreement(&obs, &modeled)).abs() < 1e-12);
    }

    #[test]
    fn mean_empty() {
        let empty: [f64; 0] = [];
        assert!(mean(&empty).abs() < 1e-12);
    }

    #[test]
    fn std_dev_empty() {
        let empty: [f64; 0] = [];
        assert!(std_dev(&empty).abs() < 1e-12);
    }

    #[test]
    fn std_dev_constant() {
        let vals = [4.0, 4.0, 4.0];
        assert!(std_dev(&vals).abs() < 1e-12);
    }

    #[test]
    fn percentile_empty() {
        let empty: [f64; 0] = [];
        assert!(percentile(&empty, 50.0).abs() < 1e-12);
    }

    #[test]
    fn percentile_interpolation() {
        let vals = [1.0, 2.0, 3.0, 4.0];
        let p25 = percentile(&vals, 25.0);
        assert!(
            (p25 - 1.75).abs() < 1e-12,
            "P25 of [1,2,3,4] = 1.75, got {p25}"
        );
    }

    #[test]
    fn sample_std_dev_bessel_correction() {
        let vals = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let pop = std_dev(&vals);
        let samp = sample_std_dev(&vals);
        assert!(samp > pop, "sample std > population std");
        // mean=5, Σ(x-μ)²=32, s²=32/7≈4.571, s≈2.138
        assert!(
            (samp - 2.138).abs() < 0.01,
            "known sample σ ≈ 2.138, got {samp}"
        );
    }

    #[test]
    fn sample_std_dev_single_element() {
        assert!(sample_std_dev(&[42.0]).abs() < 1e-12);
    }

    #[test]
    fn sample_std_dev_empty() {
        let empty: [f64; 0] = [];
        assert!(sample_std_dev(&empty).abs() < 1e-12);
    }

    #[test]
    fn pearson_r_perfect_positive() {
        let x = [1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((pearson_r(&x, &x) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn pearson_r_perfect_negative() {
        let x = [1.0, 2.0, 3.0, 4.0, 5.0];
        let y = [5.0, 4.0, 3.0, 2.0, 1.0];
        assert!((pearson_r(&x, &y) + 1.0).abs() < 1e-12);
    }

    #[test]
    fn pearson_r_constant_is_zero() {
        let x = [1.0, 2.0, 3.0];
        let y = [5.0, 5.0, 5.0];
        assert!(pearson_r(&x, &y).abs() < 1e-12);
    }

    #[test]
    fn pearson_r_empty() {
        let empty: [f64; 0] = [];
        assert!(pearson_r(&empty, &empty).abs() < 1e-12);
    }

    #[test]
    fn pearson_r_squared_matches_r2_for_linear() {
        let obs = [1.0, 2.0, 3.0, 4.0, 5.0];
        let modeled = [1.1, 2.05, 3.0, 3.95, 4.9];
        let r = pearson_r(&obs, &modeled);
        let r2_via_pearson = r * r;
        let r2_direct = r_squared(&obs, &modeled);
        assert!(
            (r2_via_pearson - r2_direct).abs() < 0.01,
            "r² ≈ R² for near-linear data"
        );
    }

    #[test]
    fn spearman_r_perfect_monotonic() {
        let x = [1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((spearman_r(&x, &x) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn spearman_r_perfect_negative_monotonic() {
        let x = [1.0, 2.0, 3.0, 4.0, 5.0];
        let y = [5.0, 4.0, 3.0, 2.0, 1.0];
        assert!((spearman_r(&x, &y) + 1.0).abs() < 1e-12);
    }

    #[test]
    fn spearman_r_nonlinear_monotonic() {
        let x = [1.0, 2.0, 3.0, 4.0, 5.0];
        let y = [1.0, 4.0, 9.0, 16.0, 25.0];
        assert!(
            (spearman_r(&x, &y) - 1.0).abs() < 1e-12,
            "monotonic nonlinear → r_s = 1"
        );
    }

    #[test]
    fn spearman_r_empty() {
        let empty: [f64; 0] = [];
        assert!(spearman_r(&empty, &empty).abs() < 1e-12);
    }

    #[test]
    fn spearman_r_with_ties() {
        let x = [1.0, 2.0, 2.0, 3.0];
        let y = [1.0, 2.0, 3.0, 4.0];
        let rs = spearman_r(&x, &y);
        assert!(rs > 0.9, "high positive with ties, got {rs}");
    }
}
