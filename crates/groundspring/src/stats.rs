// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Core statistical metrics shared across all groundSpring experiments.
//!
//! These are the canonical implementations of RMSE, MBE, R², and Index of
//! Agreement.  All functions operate on slices to enable zero-copy usage
//! from any data source.

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
    (sum_sq / n as f64).sqrt()
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
    sum / n as f64
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
    let mean_obs: f64 = observed.iter().sum::<f64>() / n as f64;
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
    let mean_obs: f64 = observed.iter().sum::<f64>() / n as f64;
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
    values.iter().sum::<f64>() / values.len() as f64
}

/// Population standard deviation.
#[must_use]
pub fn std_dev(values: &[f64]) -> f64 {
    let n = values.len();
    if n == 0 {
        return 0.0;
    }
    let m = mean(values);
    let variance = values.iter().map(|v| (v - m).powi(2)).sum::<f64>() / n as f64;
    variance.sqrt()
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
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let rank = p / 100.0 * (sorted.len() - 1) as f64;
    let lo = rank.floor() as usize;
    let hi = rank.ceil() as usize;
    if lo == hi {
        sorted[lo]
    } else {
        let frac = rank - lo as f64;
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
}
