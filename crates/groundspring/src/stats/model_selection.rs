// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Information-theoretic model selection: AIC and BIC.
//!
//! Provides Akaike Information Criterion (AIC) and Bayesian Information
//! Criterion (BIC) for comparing regression models of different complexity.
//! Lower values indicate a better trade-off between fit and parsimony.
//!
//! Used by the LTEE fitness dynamics reproduction (Wiser et al. 2013) where
//! power-law, hyperbolic, and logarithmic models are compared.

use crate::cast::usize_f64;

use super::NonlinearFit;

/// Result of model comparison via information criteria.
#[derive(Debug, Clone)]
pub struct ModelComparison {
    /// Model name (matches [`NonlinearFit::model`]).
    pub model: &'static str,
    /// Number of fitted parameters.
    pub k: usize,
    /// Residual sum of squares.
    pub rss: f64,
    /// Akaike Information Criterion.
    pub aic: f64,
    /// Bayesian Information Criterion.
    pub bic: f64,
    /// Coefficient of determination.
    pub r_squared: f64,
}

/// Akaike Information Criterion: `n·ln(RSS/n) + 2k`.
///
/// # Panics
///
/// Panics if `n == 0` or `rss <= 0`.
#[must_use]
pub fn aic(n: usize, k: usize, rss: f64) -> f64 {
    assert!(n > 0, "n must be positive");
    assert!(rss > 0.0, "RSS must be positive");
    let n_f = usize_f64(n);
    let k_f = usize_f64(k);
    n_f.mul_add((rss / n_f).ln(), 2.0 * k_f)
}

/// Bayesian (Schwarz) Information Criterion: `n·ln(RSS/n) + k·ln(n)`.
///
/// # Panics
///
/// Panics if `n == 0` or `rss <= 0`.
#[must_use]
pub fn bic(n: usize, k: usize, rss: f64) -> f64 {
    assert!(n > 0, "n must be positive");
    assert!(rss > 0.0, "RSS must be positive");
    let n_f = usize_f64(n);
    let k_f = usize_f64(k);
    n_f.mul_add((rss / n_f).ln(), k_f * n_f.ln())
}

/// Residual sum of squares for a set of observed vs predicted values.
#[must_use]
pub fn rss(ys: &[f64], predictions: &[f64]) -> f64 {
    ys.iter()
        .zip(predictions)
        .map(|(&y, &p)| (y - p).powi(2))
        .sum()
}

fn param_count(model: &str) -> usize {
    match model {
        "quadratic" => 3,
        _ => 2,
    }
}

/// Compare multiple fits on the same dataset using AIC and BIC.
///
/// Returns comparisons sorted by AIC (best first).
#[must_use]
pub fn compare_models(fits: &[NonlinearFit], xs: &[f64], ys: &[f64]) -> Vec<ModelComparison> {
    let n = ys.len();
    let n_f = usize_f64(n);
    let y_mean = ys.iter().sum::<f64>() / n_f;
    let ss_tot: f64 = ys.iter().map(|&y| (y - y_mean).powi(2)).sum();

    let mut comparisons: Vec<ModelComparison> = fits
        .iter()
        .filter_map(|fit| {
            let predictions = predict(fit, xs);
            let residual_ss = rss(ys, &predictions);
            if residual_ss <= 0.0 {
                return None;
            }
            let k = param_count(fit.model);
            let r_sq = if ss_tot > 0.0 {
                1.0 - residual_ss / ss_tot
            } else {
                1.0
            };
            Some(ModelComparison {
                model: fit.model,
                k,
                rss: residual_ss,
                aic: aic(n, k, residual_ss),
                bic: bic(n, k, residual_ss),
                r_squared: r_sq,
            })
        })
        .collect();

    comparisons.sort_by(|a, b| {
        a.aic
            .partial_cmp(&b.aic)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    comparisons
}

fn predict(fit: &NonlinearFit, xs: &[f64]) -> Vec<f64> {
    match fit.model {
        "linear" => xs
            .iter()
            .map(|&x| fit.params[0].mul_add(x, fit.params[1]))
            .collect(),
        "quadratic" => xs
            .iter()
            .map(|&x| fit.params[0].mul_add(x * x, fit.params[1].mul_add(x, fit.params[2])))
            .collect(),
        "exponential" => xs
            .iter()
            .map(|&x| fit.params[0] * (fit.params[1] * x).exp())
            .collect(),
        "logarithmic" => xs
            .iter()
            .map(|&x| {
                if x > 0.0 {
                    fit.params[0].mul_add(x.ln(), fit.params[1])
                } else {
                    f64::NAN
                }
            })
            .collect(),
        "power_law" => xs
            .iter()
            .map(|&x| {
                if x > 0.0 {
                    fit.params[0].mul_add(x.powf(fit.params[1]), 1.0)
                } else {
                    f64::NAN
                }
            })
            .collect(),
        "hyperbolic" => xs
            .iter()
            .map(|&x| fit.params[0].mul_add(x / fit.params[1].mul_add(x, 1.0), 1.0))
            .collect(),
        _ => xs.iter().map(|_| f64::NAN).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aic_basic() {
        let val = aic(100, 2, 50.0);
        let expected = 100.0_f64.mul_add((50.0_f64 / 100.0).ln(), 4.0);
        assert!((val - expected).abs() < 1e-10, "AIC: {val} vs {expected}");
    }

    #[test]
    fn bic_basic() {
        let val = bic(100, 2, 50.0);
        let expected = 100.0_f64.mul_add((50.0_f64 / 100.0).ln(), 2.0 * 100.0_f64.ln());
        assert!((val - expected).abs() < 1e-10, "BIC: {val} vs {expected}");
    }

    #[test]
    fn bic_penalizes_more_than_aic_for_large_n() {
        let aic_val = aic(1000, 3, 100.0);
        let bic_val = bic(1000, 3, 100.0);
        assert!(
            bic_val > aic_val,
            "BIC ({bic_val}) should penalize more than AIC ({aic_val}) for n=1000"
        );
    }

    #[test]
    fn compare_selects_correct_model() {
        let xs: Vec<f64> = (1..=50).map(|i| f64::from(i) * 1000.0).collect();
        let a = 0.01;
        let b = 0.5;
        let ys: Vec<f64> = xs.iter().map(|&x| 1.0 + a * x.powf(b)).collect();

        let fits = crate::stats::fit_all(&xs, &ys);
        let comparisons = compare_models(&fits, &xs, &ys);

        assert!(!comparisons.is_empty());
        let best = &comparisons[0];
        assert_eq!(
            best.model, "power_law",
            "power_law should win for power-law data, got {}",
            best.model
        );
    }

    #[test]
    fn compare_hyperbolic_wins_for_saturating() {
        let a = 0.005;
        let b = 0.001;
        let xs: Vec<f64> = (1..=50).map(|i| f64::from(i) * 1000.0).collect();
        let ys: Vec<f64> = xs.iter().map(|&x| 1.0 + a * x / (1.0 + b * x)).collect();

        let fits = crate::stats::fit_all(&xs, &ys);
        let comparisons = compare_models(&fits, &xs, &ys);

        assert!(!comparisons.is_empty());
        let best = &comparisons[0];
        assert_eq!(
            best.model, "hyperbolic",
            "hyperbolic should win for saturating data, got {}",
            best.model
        );
    }
}
