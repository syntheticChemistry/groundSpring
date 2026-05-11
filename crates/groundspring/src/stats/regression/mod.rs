// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Regression fitting: linear, quadratic, exponential, logarithmic.
//!
//! Provides `fit_linear` for ordinary least squares plus higher-order fits
//! when barracuda is available. Used by [`crate::wdm`] (finite-size
//! extrapolation) and [`crate::transport`] (log-log MSD regression).
//!
//! # barracuda delegation
//!
//! When the `barracuda` feature is enabled:
//! - [`fit_linear`] delegates to `barracuda::stats::regression::fit_linear`
//! - [`fit_quadratic`] delegates to `barracuda::stats::regression::fit_quadratic`
//! - [`fit_exponential`] delegates to `barracuda::stats::regression::fit_exponential`
//! - [`fit_logarithmic`] delegates to `barracuda::stats::regression::fit_logarithmic`
//!
//! Each falls back to a local CPU implementation on error or when barracuda
//! is not available.

mod linear;
mod nonlinear;
mod quadratic;

/// Result of a simple linear regression fit.
#[derive(Debug, Clone, Copy)]
pub struct LinearFit {
    /// y-intercept (value of y when x = 0).
    pub intercept: f64,
    /// Slope (change in y per unit change in x).
    pub slope: f64,
    /// Coefficient of determination (R²). 1.0 = perfect fit.
    pub r_squared: f64,
}

/// Result of a nonlinear regression fit.
#[derive(Debug, Clone)]
pub struct NonlinearFit {
    /// Model name.
    pub model: &'static str,
    /// Model parameters (interpretation depends on model).
    pub params: Vec<f64>,
    /// Coefficient of determination (R²). 1.0 = perfect fit.
    pub r_squared: f64,
}

pub use linear::fit_linear;
pub use nonlinear::{fit_exponential, fit_hyperbolic, fit_logarithmic, fit_power_law};
pub use quadratic::fit_quadratic;

/// Fit all six models and return those that converge.
///
/// Runs [`fit_linear`], [`fit_quadratic`], [`fit_exponential`],
/// [`fit_logarithmic`], [`fit_power_law`], and [`fit_hyperbolic`]
/// on the same data, collecting any that succeed.
/// Useful for automated model comparison and best-fit selection.
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::stats::regression::fit_all`.
///
/// # Panics
///
/// Panics if `xs` and `ys` have different lengths.
#[must_use]
pub fn fit_all(xs: &[f64], ys: &[f64]) -> Vec<NonlinearFit> {
    assert_eq!(xs.len(), ys.len(), "xs and ys must have equal length");

    #[cfg(feature = "barracuda")]
    {
        let fits = barracuda::stats::regression::fit_all(xs, ys);
        if !fits.is_empty() {
            return fits
                .into_iter()
                .map(|f| NonlinearFit {
                    model: f.model,
                    params: f.params,
                    r_squared: f.r_squared,
                })
                .collect();
        }
    }

    let linear = fit_linear(xs, ys).map(|f| NonlinearFit {
        model: "linear",
        params: vec![f.slope, f.intercept],
        r_squared: f.r_squared,
    });

    [
        linear,
        fit_quadratic(xs, ys),
        fit_exponential(xs, ys),
        fit_logarithmic(xs, ys),
        fit_power_law(xs, ys),
        fit_hyperbolic(xs, ys),
    ]
    .into_iter()
    .flatten()
    .collect()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use crate::tol;

    #[test]
    fn fit_all_returns_multiple_models() {
        let xs = [1.0, 2.0, 3.0, 4.0, 5.0];
        let ys = [2.0, 4.0, 6.0, 8.0, 10.0];
        let fits = fit_all(&xs, &ys);
        assert!(
            fits.len() >= 2,
            "should fit at least linear + logarithmic, got {}",
            fits.len()
        );
        let has_linear = fits.iter().any(|f| f.model == "linear");
        assert!(has_linear, "should include linear model");
    }

    #[test]
    fn fit_all_linear_has_best_r2_for_line() {
        let xs = [1.0, 2.0, 3.0, 4.0, 5.0];
        let ys = [3.0, 5.0, 7.0, 9.0, 11.0];
        let fits = fit_all(&xs, &ys);
        let linear = fits.iter().find(|f| f.model == "linear").unwrap();
        assert!(
            (linear.r_squared - 1.0).abs() < tol::ANALYTICAL,
            "perfect line should have R² = 1.0"
        );
        for f in &fits {
            assert!(
                linear.r_squared >= f.r_squared - tol::ANALYTICAL,
                "linear should be best fit for perfect line"
            );
        }
    }

    #[test]
    fn fit_all_insufficient_data() {
        let fits = fit_all(&[1.0], &[1.0]);
        assert!(fits.is_empty(), "single point should return no fits");
    }

    #[test]
    fn fit_all_empty() {
        let fits: Vec<NonlinearFit> = fit_all(&[], &[]);
        assert!(fits.is_empty());
    }
}
