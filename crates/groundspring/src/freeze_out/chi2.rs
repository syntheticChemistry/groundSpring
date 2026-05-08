// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Decomposed chi-squared analysis with per-datum diagnostics.
//!
//! Cross-spring lineage: hotSpring `Chi2Decomposed` (nuclear structure
//! fit quality) → barraCuda S59 `barracuda::stats::chi2` with p-value
//! via regularized incomplete gamma → groundSpring freeze-out analysis.

use crate::cast::usize_f64;

/// Decomposed chi-squared analysis with per-datum diagnostics.
#[derive(Debug, Clone)]
pub struct Chi2Analysis {
    /// Total chi-squared statistic.
    pub chi2_total: f64,
    /// Chi-squared per data point.
    pub chi2_per_datum: f64,
    /// Chi-squared per degree of freedom.
    pub chi2_per_dof: f64,
    /// Degrees of freedom (`n_data` − `n_params`).
    pub dof: usize,
    /// Per-datum chi-squared contributions.
    pub contributions: Vec<f64>,
    /// Residuals (observed − predicted).
    pub residuals: Vec<f64>,
    /// Pulls (residual / uncertainty).
    pub pulls: Vec<f64>,
    /// P-value from the chi-squared distribution.
    pub p_value: f64,
}

/// Decomposed chi-squared analysis with per-datum diagnostics.
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::stats::chi2::chi2_decomposed_weighted` for the full
/// decomposition including p-value computation via the regularized
/// incomplete gamma function. Falls back to a local implementation
/// that computes everything except the p-value (set to NaN).
///
/// # Errors
///
/// Returns [`crate::error::InputError::LengthMismatch`] if `observed`
/// and `predicted` have different lengths.
pub fn chi2_analysis(
    observed: &[f64],
    predicted: &[f64],
    sigma: f64,
    n_params: usize,
) -> Result<Chi2Analysis, crate::error::InputError> {
    if observed.len() != predicted.len() {
        return Err(crate::error::InputError::LengthMismatch {
            first: "observed",
            first_len: observed.len(),
            second: "predicted",
            second_len: predicted.len(),
        });
    }
    #[cfg(feature = "barracuda")]
    {
        let uncertainties: Vec<f64> = vec![sigma; observed.len()];
        if let Ok(decomposed) = barracuda::stats::chi2::chi2_decomposed_weighted(
            observed,
            predicted,
            &uncertainties,
            n_params,
        ) {
            return Ok(Chi2Analysis {
                chi2_total: decomposed.chi2_total,
                chi2_per_datum: decomposed.chi2_per_datum,
                chi2_per_dof: decomposed.chi2_per_dof,
                dof: decomposed.dof,
                contributions: decomposed.contributions,
                residuals: decomposed.residuals,
                pulls: decomposed.pulls,
                p_value: decomposed.p_value,
            });
        }
    }
    Ok(chi2_analysis_cpu(observed, predicted, sigma, n_params))
}

fn chi2_analysis_cpu(
    observed: &[f64],
    predicted: &[f64],
    sigma: f64,
    n_params: usize,
) -> Chi2Analysis {
    let n = observed.len();
    let inv_sigma = 1.0 / sigma;

    let residuals: Vec<f64> = observed
        .iter()
        .zip(predicted.iter())
        .map(|(&o, &p)| o - p)
        .collect();
    let pulls: Vec<f64> = residuals.iter().map(|&r| r * inv_sigma).collect();
    let contributions: Vec<f64> = pulls.iter().map(|&p| p * p).collect();
    let chi2_total: f64 = contributions.iter().sum();
    let dof = n.saturating_sub(n_params);
    let chi2_per_dof = if dof > 0 {
        chi2_total / usize_f64(dof)
    } else {
        chi2_total
    };

    Chi2Analysis {
        chi2_total,
        chi2_per_datum: if n > 0 {
            chi2_total / usize_f64(n)
        } else {
            0.0
        },
        chi2_per_dof,
        dof,
        contributions,
        residuals,
        pulls,
        p_value: f64::NAN,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perfect_prediction_has_zero_chi2() {
        let obs = [1.0, 2.0, 3.0, 4.0];
        let pred = [1.0, 2.0, 3.0, 4.0];
        let result = chi2_analysis(&obs, &pred, 1.0, 2).unwrap();
        assert_eq!(result.chi2_total, 0.0);
        assert_eq!(result.dof, 2);
        assert!(result.residuals.iter().all(|&r| r == 0.0));
    }

    #[test]
    fn known_chi2_value() {
        let obs = [1.0, 2.0, 3.0];
        let pred = [1.1, 2.0, 2.8];
        let result = chi2_analysis(&obs, &pred, 0.1, 1).unwrap();
        assert!((result.chi2_total - 5.0).abs() < crate::tol::EXACT);
        assert_eq!(result.dof, 2);
    }

    #[test]
    fn length_mismatch_returns_error() {
        let obs = [1.0, 2.0];
        let pred = [1.0];
        assert!(chi2_analysis(&obs, &pred, 1.0, 1).is_err());
    }

    #[test]
    fn pulls_are_residuals_over_sigma() {
        let obs = [10.0];
        let pred = [8.0];
        let sigma = 2.0;
        let result = chi2_analysis(&obs, &pred, sigma, 0).unwrap();
        assert!((result.pulls[0] - 1.0).abs() < crate::tol::EXACT);
    }
}
