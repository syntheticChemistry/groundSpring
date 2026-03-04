// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Correlation coefficients and covariance.
//!
//! Pearson (linear), Spearman (monotonic rank) and sample covariance.
//!
//! # barracuda delegation
//!
//! When the `barracuda` feature is enabled each function delegates to the
//! corresponding GPU-ready implementation, falling back to the always-compiled
//! CPU path on error.

use crate::cast::usize_f64;

use super::metrics::mean;

/// Pearson correlation coefficient.
///
/// When `barracuda-gpu` is enabled, dispatches to `CorrelationF64` GPU
/// kernel. Otherwise delegates to `barracuda::stats::pearson_correlation`
/// (CPU) or the local implementation.
/// Returns `0.0` when total sum of squares is zero for either variable.
///
/// # Panics
///
/// Panics if `x` and `y` have different lengths.
#[must_use]
pub fn pearson_r(x: &[f64], y: &[f64]) -> f64 {
    assert_eq!(x.len(), y.len(), "x and y must have equal length");
    #[cfg(feature = "barracuda-gpu")]
    {
        if let Some(r) = pearson_r_gpu(x, y) {
            return r;
        }
    }
    #[cfg(feature = "barracuda")]
    if let Ok(r) = barracuda::stats::pearson_correlation(x, y) {
        return if r.is_nan() { 0.0 } else { r };
    }
    pearson_r_cpu(x, y)
}

#[cfg(feature = "barracuda-gpu")]
fn pearson_r_gpu(x: &[f64], y: &[f64]) -> Option<f64> {
    if x.is_empty() {
        return Some(0.0);
    }
    let device = crate::gpu::get_device()?;
    let gpu = barracuda::ops::correlation_f64_wgsl::CorrelationF64::new(device).ok()?;
    let r = gpu.correlation(x, y).ok()?;
    Some(if r.is_nan() { 0.0 } else { r })
}

fn pearson_r_cpu(x: &[f64], y: &[f64]) -> f64 {
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

/// Spearman rank correlation coefficient.
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::stats::spearman_correlation`.
/// Returns `0.0` for empty slices, when variance is zero, or when the
/// `barracuda` delegate reports an error (matching the local fallback).
///
/// # Panics
///
/// Panics if `x` and `y` have different lengths.
#[must_use]
pub fn spearman_r(x: &[f64], y: &[f64]) -> f64 {
    assert_eq!(x.len(), y.len(), "x and y must have equal length");
    #[cfg(feature = "barracuda")]
    if let Ok(r) = barracuda::stats::correlation::spearman_correlation(x, y) {
        return if r.is_nan() { 0.0 } else { r };
    }
    spearman_r_cpu(x, y)
}

fn spearman_r_cpu(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len();
    if n < 2 {
        return 0.0;
    }
    let rx = rank(x);
    let ry = rank(y);
    pearson_r(&rx, &ry)
}

fn rank(data: &[f64]) -> Vec<f64> {
    let mut indexed: Vec<(usize, f64)> = data.iter().copied().enumerate().collect();
    indexed.sort_by(|a, b| f64::total_cmp(&a.1, &b.1));

    let mut ranks = vec![0.0; data.len()];
    let mut start = 0;
    while start < indexed.len() {
        let tie_end = indexed[start..]
            .iter()
            .position(|item| f64::total_cmp(&item.1, &indexed[start].1).is_ne())
            .map_or(indexed.len(), |offset| start + offset);

        let avg_rank = f64::midpoint(usize_f64(start + 1), usize_f64(tie_end));
        for &(orig_idx, _) in &indexed[start..tie_end] {
            ranks[orig_idx] = avg_rank;
        }
        start = tie_end;
    }
    ranks
}

/// Sample covariance between two slices (Bessel-corrected, divides by N−1).
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::stats::correlation::covariance`.
/// Returns `0.0` for slices with fewer than 2 elements, or when the
/// `barracuda` delegate reports an error (matching the local fallback).
///
/// # Panics
///
/// Panics if `x` and `y` have different lengths.
#[must_use]
pub fn covariance(x: &[f64], y: &[f64]) -> f64 {
    assert_eq!(x.len(), y.len(), "x and y must have equal length");
    #[cfg(feature = "barracuda")]
    if let Ok(c) = barracuda::stats::correlation::covariance(x, y) {
        return c;
    }
    covariance_cpu(x, y)
}

fn covariance_cpu(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len();
    if n < 2 {
        return 0.0;
    }
    let mx = mean(x);
    let my = mean(y);
    let sum: f64 = x
        .iter()
        .zip(y)
        .map(|(&xi, &yi)| (xi - mx) * (yi - my))
        .sum();
    sum / usize_f64(n - 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tol;

    // Tolerance key:
    //   1e-12  — exact arithmetic identity, limited only by f64 rounding
    //   1e-10  — known analytical value with one intermediate division
    //   0.01   — near-linear data where r² ≈ R² within rounding

    #[test]
    fn pearson_r_perfect_positive() {
        let x = [1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((pearson_r(&x, &x) - 1.0).abs() < tol::EXACT);
    }

    #[test]
    fn pearson_r_perfect_negative() {
        let x = [1.0, 2.0, 3.0, 4.0, 5.0];
        let y = [5.0, 4.0, 3.0, 2.0, 1.0];
        assert!((pearson_r(&x, &y) + 1.0).abs() < tol::EXACT);
    }

    #[test]
    fn pearson_r_constant_is_zero() {
        let x = [1.0, 2.0, 3.0];
        let y = [5.0, 5.0, 5.0];
        assert!(pearson_r(&x, &y).abs() < tol::EXACT);
    }

    #[test]
    fn pearson_r_empty() {
        let empty: [f64; 0] = [];
        assert!(pearson_r(&empty, &empty).abs() < tol::EXACT);
    }

    #[test]
    fn pearson_r_squared_matches_r2_for_linear() {
        use crate::stats::r_squared;

        let obs = [1.0, 2.0, 3.0, 4.0, 5.0];
        let modeled = [1.1, 2.05, 3.0, 3.95, 4.9];
        let r = pearson_r(&obs, &modeled);
        let r2_via_pearson = r * r;
        let r2_direct = r_squared(&obs, &modeled);
        assert!(
            (r2_via_pearson - r2_direct).abs() < tol::STOCHASTIC,
            "r² ≈ R² for near-linear data"
        );
    }

    #[test]
    fn spearman_r_perfect_monotonic() {
        let x = [1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((spearman_r(&x, &x) - 1.0).abs() < tol::EXACT);
    }

    #[test]
    fn spearman_r_perfect_negative_monotonic() {
        let x = [1.0, 2.0, 3.0, 4.0, 5.0];
        let y = [5.0, 4.0, 3.0, 2.0, 1.0];
        assert!((spearman_r(&x, &y) + 1.0).abs() < tol::EXACT);
    }

    #[test]
    fn spearman_r_nonlinear_monotonic() {
        let x = [1.0, 2.0, 3.0, 4.0, 5.0];
        let y = [1.0, 4.0, 9.0, 16.0, 25.0];
        assert!(
            (spearman_r(&x, &y) - 1.0).abs() < tol::EXACT,
            "monotonic nonlinear → r_s = 1"
        );
    }

    #[test]
    fn spearman_r_empty() {
        let empty: [f64; 0] = [];
        assert!(spearman_r(&empty, &empty).abs() < tol::EXACT);
    }

    #[test]
    fn spearman_r_with_ties() {
        let x = [1.0, 2.0, 2.0, 3.0];
        let y = [1.0, 2.0, 3.0, 4.0];
        let rs = spearman_r(&x, &y);
        assert!(rs > 0.9, "high positive with ties, got {rs}");
    }

    #[test]
    fn covariance_positive() {
        let x = [1.0, 2.0, 3.0, 4.0, 5.0];
        let y = [2.0, 4.0, 6.0, 8.0, 10.0];
        let c = covariance(&x, &y);
        // Cov(x, 2x) = 2·Var(x) = 2·2.5 = 5.0
        assert!(
            (c - 5.0).abs() < tol::ANALYTICAL,
            "Cov(x, 2x) = 2·Var(x) = 5.0, got {c}"
        );
    }

    #[test]
    fn covariance_zero_for_independent() {
        let x = [1.0, 2.0, 3.0, 4.0];
        let y = [3.0, 3.0, 3.0, 3.0];
        assert!(covariance(&x, &y).abs() < tol::EXACT);
    }

    #[test]
    fn covariance_single_element() {
        assert!(covariance(&[42.0], &[42.0]).abs() < tol::EXACT);
    }
}
