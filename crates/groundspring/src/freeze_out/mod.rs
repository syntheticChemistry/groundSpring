// SPDX-License-Identifier: AGPL-3.0-only
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
//! When `barracuda` is enabled, the grid-search result is refined via
//! `barracuda::optimize::lbfgs_numerical` (L-BFGS with numerical gradient,
//! absorbed from airSpring V035 → barraCuda S84).
//! [`chi_squared`] and [`freeze_out_curve`] stay local (scalar ops).
//!
//! ## S80 evolution: batched Nelder-Mead GPU
//!
//! `barracuda::optimize::batched_nelder_mead_gpu` (barraCuda S80) enables
//! multi-start derivative-free optimization. [`nelder_mead_multi_start`]
//! exposes this as an alternative to L-BFGS for non-smooth landscapes.

mod chi2;
mod curve;
mod grid;
mod nelder_mead;

pub use chi2::{Chi2Analysis, chi2_analysis};
pub use curve::{chi_squared, chi_squared_per_dof, freeze_out_curve};
pub use grid::grid_fit_2d;
pub use nelder_mead::{NelderMeadMultiStartResult, nelder_mead_multi_start};

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

/// Shared validation for [`GridFitConfig`] slice lengths.
const fn validate_config_lengths(
    config: &GridFitConfig<'_>,
) -> Result<(), crate::error::InputError> {
    if config.observed.len() != config.mu_b.len() {
        return Err(crate::error::InputError::LengthMismatch {
            first: "observed",
            first_len: config.observed.len(),
            second: "mu_b",
            second_len: config.mu_b.len(),
        });
    }
    Ok(())
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;
    use crate::tol;

    #[test]
    fn curve_at_zero() {
        let t = freeze_out_curve(155.0, 0.013, 0.0);
        assert!((t - 155.0).abs() < tol::EXACT, "T_f(0) should equal T0");
    }

    #[test]
    fn curve_monotone_decreasing() {
        let t0 = 155.0;
        let k2 = 0.013;
        let prev = freeze_out_curve(t0, k2, 0.0);
        for mu in (50..=400).step_by(50) {
            let t = freeze_out_curve(t0, k2, f64::from(mu));
            assert!(t <= prev + tol::EXACT, "T_f should decrease with mu_B");
        }
    }

    #[test]
    fn chi2_zero_at_truth() {
        let obs = vec![1.0, 2.0, 3.0];
        let pred = vec![1.0, 2.0, 3.0];
        let c2 = chi_squared(&obs, &pred, 1.0).unwrap();
        assert!(c2.abs() < tol::STRICT);
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
        assert!(
            (r.kappa2 - k2).abs() < tol::DECOMPOSITION,
            "k2: got {}",
            r.kappa2
        );
    }

    #[test]
    fn chi2_per_dof_correct() {
        let c = chi_squared_per_dof(14.0, 9, 2);
        assert!((c - 2.0).abs() < tol::EXACT);
    }

    #[test]
    fn chi2_analysis_perfect_fit() {
        let obs = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let a = chi2_analysis(&obs, &obs, 1.0, 2).unwrap();
        assert!(a.chi2_total.abs() < tol::STRICT, "perfect fit → χ²=0");
        assert_eq!(a.dof, 3);
        assert!(a.residuals.iter().all(|&r| r.abs() < tol::STRICT));
        assert!(a.pulls.iter().all(|&p| p.abs() < tol::STRICT));
    }

    #[test]
    fn chi2_analysis_known_value() {
        let obs = vec![1.0, 2.0, 3.0];
        let pred = vec![1.1, 1.9, 3.2];
        let a = chi2_analysis(&obs, &pred, 0.1, 0).unwrap();
        let expected_chi2 = 6.0_f64;
        assert!(
            (a.chi2_total - expected_chi2).abs() < tol::ANALYTICAL,
            "χ²={}, expected {expected_chi2}",
            a.chi2_total
        );
        assert_eq!(a.contributions.len(), 3);
        assert_eq!(a.residuals.len(), 3);
    }

    #[test]
    fn chi2_analysis_residual_signs() {
        let obs = vec![5.0, 3.0];
        let pred = vec![4.0, 4.0];
        let a = chi2_analysis(&obs, &pred, 1.0, 0).unwrap();
        assert!(
            (a.residuals[0] - 1.0).abs() < tol::STRICT,
            "obs > pred → positive residual"
        );
        assert!(
            (a.residuals[1] - (-1.0)).abs() < tol::STRICT,
            "obs < pred → negative residual"
        );
    }

    #[test]
    fn chi2_analysis_length_mismatch() {
        assert!(chi2_analysis(&[1.0, 2.0], &[1.0], 1.0, 0).is_err());
    }
}
