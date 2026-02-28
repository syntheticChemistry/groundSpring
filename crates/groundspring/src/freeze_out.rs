// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Freeze-out curve fitting and chi-squared inverse problem.
//!
//! Implements the polynomial forward model and 2D grid-search chi-squared
//! minimization from Bazavov et al. (2016) Phys Rev D 93, 014512.
//!
//! The freeze-out curve parameterizes the QCD transition temperature as
//! a function of baryon chemical potential:
//! `T_f(μ_B) = T₀ (1 - κ₂ (μ_B/T₀)²)`
//!
//! # barracuda delegation
//!
//! [`grid_fit_2d`] is embarrassingly parallel — each (T₀, κ₂) grid
//! point evaluates independently. GPU promotion via `barracuda-gpu`
//! dispatches as a 2D workgroup with per-point chi-squared reduction.
//! [`chi_squared`] and [`freeze_out_curve`] stay local (scalar ops).

use crate::cast::usize_f64;

/// Result of a 2D grid-search chi-squared fit.
#[derive(Debug, Clone)]
pub struct GridFitResult {
    /// Best-fit T₀ parameter.
    pub t0: f64,
    /// Best-fit κ₂ parameter.
    pub kappa2: f64,
    /// Chi-squared value at the best fit.
    pub chi_squared: f64,
    /// Chi-squared per degree of freedom.
    pub chi2_per_dof: f64,
}

/// Evaluate the freeze-out curve at a single `μ_B`.
///
/// `T_f(μ_B) = T₀ (1 - κ₂ (μ_B/T₀)²)`
#[inline]
#[must_use]
pub fn freeze_out_curve(t0: f64, kappa2: f64, mu_b: f64) -> f64 {
    let r = mu_b / t0;
    (-kappa2).mul_add(r * r, 1.0) * t0
}

/// Chi-squared statistic for uniform errors.
///
/// `χ² = Σ((obs_i - pred_i) / σ)²`
///
/// # Errors
///
/// Returns [`crate::error::InputError::LengthMismatch`] if `observed` and
/// `predicted` have different lengths.
pub fn chi_squared(
    observed: &[f64],
    predicted: &[f64],
    sigma: f64,
) -> Result<f64, crate::error::InputError> {
    if observed.len() != predicted.len() {
        return Err(crate::error::InputError::LengthMismatch {
            first: "observed",
            first_len: observed.len(),
            second: "predicted",
            second_len: predicted.len(),
        });
    }
    let inv_sigma2 = 1.0 / (sigma * sigma);
    Ok(observed
        .iter()
        .zip(predicted.iter())
        .map(|(&o, &p)| (o - p).powi(2) * inv_sigma2)
        .sum())
}

/// Chi-squared per degree of freedom.
#[inline]
#[must_use]
pub fn chi_squared_per_dof(chi2: f64, n_data: usize, n_params: usize) -> f64 {
    chi2 / usize_f64(n_data - n_params)
}

/// 2D grid search over (T₀, κ₂) minimizing chi-squared.
///
/// Evaluates the freeze-out model on a regular grid and returns the
/// parameters with lowest chi-squared.
///
/// # Errors
///
/// Returns [`crate::error::InputError::LengthMismatch`] if
/// `config.observed` and `config.mu_b` have different lengths.
pub fn grid_fit_2d(config: &GridFitConfig<'_>) -> Result<GridFitResult, crate::error::InputError> {
    if config.observed.len() != config.mu_b.len() {
        return Err(crate::error::InputError::LengthMismatch {
            first: "observed",
            first_len: config.observed.len(),
            second: "mu_b",
            second_len: config.mu_b.len(),
        });
    }
    // TODO(toadstool): wire when barracuda adds ops::grid::grid_fit_2d_f64
    // Status S68+: not yet absorbed. Embarrassingly parallel 2D chi-squared
    // grid search — high-value GPU target. Handoff item.
    // #[cfg(feature = "barracuda-gpu")]
    // {
    //     if let Ok(result) = barracuda::ops::grid::grid_fit_2d_f64(
    //         config.observed, config.mu_b, config.sigma,
    //         config.t0_lo, config.t0_hi, config.t0_step,
    //         config.k2_lo, config.k2_hi, config.k2_step,
    //     ) {
    //         return Ok(GridFitResult {
    //             t0: result.0, kappa2: result.1, chi_squared: result.2,
    //             chi2_per_dof: chi_squared_per_dof(result.2, config.observed.len(), 2),
    //         });
    //     }
    // }
    Ok(grid_fit_2d_cpu(config))
}

#[allow(
    clippy::similar_names,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn grid_fit_2d_cpu(config: &GridFitConfig<'_>) -> GridFitResult {
    let n_data = config.observed.len();
    let inv_sigma2 = 1.0 / (config.sigma * config.sigma);

    let mut best_chi2 = f64::INFINITY;
    let mut best_t0 = config.t0_lo;
    let mut best_k2 = config.k2_lo;

    let n_t0 = ((config.t0_hi - config.t0_lo) / config.t0_step).ceil() as usize + 1;
    let n_k2 = ((config.k2_hi - config.k2_lo) / config.k2_step).ceil() as usize + 1;

    let mut pred = vec![0.0; n_data];

    for it in 0..n_t0 {
        let t0 = usize_f64(it).mul_add(config.t0_step, config.t0_lo);
        for ik in 0..n_k2 {
            let k2 = usize_f64(ik).mul_add(config.k2_step, config.k2_lo);
            for (j, &mu) in config.mu_b.iter().enumerate() {
                pred[j] = freeze_out_curve(t0, k2, mu);
            }
            let c2: f64 = config
                .observed
                .iter()
                .zip(pred.iter())
                .map(|(&o, &p)| (o - p).powi(2) * inv_sigma2)
                .sum();
            if c2 < best_chi2 {
                best_chi2 = c2;
                best_t0 = t0;
                best_k2 = k2;
            }
        }
    }

    let n_params = 2;
    GridFitResult {
        t0: best_t0,
        kappa2: best_k2,
        chi_squared: best_chi2,
        chi2_per_dof: chi_squared_per_dof(best_chi2, n_data, n_params),
    }
}

/// Configuration for a 2D grid-search fit.
#[derive(Debug, Clone, Copy)]
pub struct GridFitConfig<'a> {
    /// Observed data points.
    pub observed: &'a [f64],
    /// Corresponding `μ_B` values.
    pub mu_b: &'a [f64],
    /// Measurement uncertainty (uniform σ).
    pub sigma: f64,
    /// T₀ grid lower bound.
    pub t0_lo: f64,
    /// T₀ grid upper bound.
    pub t0_hi: f64,
    /// T₀ grid step size.
    pub t0_step: f64,
    /// κ₂ grid lower bound.
    pub k2_lo: f64,
    /// κ₂ grid upper bound.
    pub k2_hi: f64,
    /// κ₂ grid step size.
    pub k2_step: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curve_at_zero() {
        let t = freeze_out_curve(155.0, 0.013, 0.0);
        assert!((t - 155.0).abs() < 1e-12, "T_f(0) should equal T0");
    }

    #[test]
    fn curve_monotone_decreasing() {
        let t0 = 155.0;
        let k2 = 0.013;
        let prev = freeze_out_curve(t0, k2, 0.0);
        for mu in (50..=400).step_by(50) {
            let t = freeze_out_curve(t0, k2, f64::from(mu));
            assert!(t <= prev + 1e-12, "T_f should decrease with mu_B");
        }
    }

    #[test]
    fn chi2_zero_at_truth() {
        let obs = vec![1.0, 2.0, 3.0];
        let pred = vec![1.0, 2.0, 3.0];
        let c2 = chi_squared(&obs, &pred, 1.0).unwrap();
        assert!(c2.abs() < 1e-14);
    }

    #[test]
    fn grid_recovers_noiseless() {
        let t0 = 155.0;
        let k2 = 0.013;
        let mu_b: Vec<f64> = (0..9).map(|i| f64::from(i) * 50.0).collect();
        let obs: Vec<f64> = mu_b.iter().map(|&m| freeze_out_curve(t0, k2, m)).collect();

        let cfg = GridFitConfig {
            observed: &obs,
            mu_b: &mu_b,
            sigma: 1.0,
            t0_lo: 150.0,
            t0_hi: 160.0,
            t0_step: 0.5,
            k2_lo: 0.008,
            k2_hi: 0.020,
            k2_step: 0.001,
        };
        let r = grid_fit_2d(&cfg).unwrap();
        assert!((r.t0 - t0).abs() < 1.0, "T0: got {}", r.t0);
        assert!((r.kappa2 - k2).abs() < 0.002, "k2: got {}", r.kappa2);
    }

    #[test]
    fn chi2_per_dof_correct() {
        let c = chi_squared_per_dof(14.0, 9, 2);
        assert!((c - 2.0).abs() < 1e-12);
    }
}
