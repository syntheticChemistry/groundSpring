// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Nonlinear regression: exponential and logarithmic fits.

use super::NonlinearFit;
use super::linear::{fit_linear_cpu, r_squared_from_residuals};

/// Fit `y = a·exp(b·x)` via log-linearized least squares.
///
/// Filters to positive y-values (required for log transform). Returns
/// `None` if fewer than 2 valid points remain.
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::stats::regression::fit_exponential`.
///
/// # Panics
///
/// Panics if `xs` and `ys` have different lengths.
#[must_use]
pub fn fit_exponential(xs: &[f64], ys: &[f64]) -> Option<NonlinearFit> {
    assert_eq!(xs.len(), ys.len(), "xs and ys must have equal length");

    if xs.len() < 2 {
        return None;
    }

    #[cfg(feature = "barracuda")]
    if let Some(fit) = barracuda::stats::regression::fit_exponential(xs, ys) {
        return Some(NonlinearFit {
            model: "exponential",
            params: fit.params,
            r_squared: fit.r_squared,
        });
    }

    fit_exponential_cpu(xs, ys)
}

fn fit_exponential_cpu(xs: &[f64], ys: &[f64]) -> Option<NonlinearFit> {
    let valid: Vec<(f64, f64)> = xs
        .iter()
        .zip(ys)
        .filter(|(_, y)| **y > 0.0)
        .map(|(x, y)| (*x, *y))
        .collect();
    if valid.len() < 2 {
        return None;
    }

    let xv: Vec<f64> = valid.iter().map(|&(x, _)| x).collect();
    let log_y: Vec<f64> = valid.iter().map(|&(_, y)| y.ln()).collect();
    let lin = fit_linear_cpu(&xv, &log_y)?;

    let b = lin.slope;
    let a = lin.intercept.exp();

    let yv: Vec<f64> = valid.iter().map(|&(_, y)| y).collect();
    let r_squared = r_squared_from_residuals(&yv, xv.iter().map(|&x| a * (b * x).exp()));

    Some(NonlinearFit {
        model: "exponential",
        params: vec![a, b],
        r_squared,
    })
}

/// Fit `y = a·ln(x) + b` via linearized least squares.
///
/// Filters to x > 0 (required for log transform). Returns `None` if
/// fewer than 2 valid points remain.
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::stats::regression::fit_logarithmic`.
///
/// # Panics
///
/// Panics if `xs` and `ys` have different lengths.
#[must_use]
pub fn fit_logarithmic(xs: &[f64], ys: &[f64]) -> Option<NonlinearFit> {
    assert_eq!(xs.len(), ys.len(), "xs and ys must have equal length");

    if xs.len() < 2 {
        return None;
    }

    #[cfg(feature = "barracuda")]
    if let Some(fit) = barracuda::stats::regression::fit_logarithmic(xs, ys) {
        return Some(NonlinearFit {
            model: "logarithmic",
            params: fit.params,
            r_squared: fit.r_squared,
        });
    }

    fit_logarithmic_cpu(xs, ys)
}

fn fit_logarithmic_cpu(xs: &[f64], ys: &[f64]) -> Option<NonlinearFit> {
    let valid: Vec<(f64, f64)> = xs
        .iter()
        .zip(ys)
        .filter(|(x, _)| **x > 0.0)
        .map(|(x, y)| (*x, *y))
        .collect();
    if valid.len() < 2 {
        return None;
    }

    let ln_x: Vec<f64> = valid.iter().map(|&(x, _)| x.ln()).collect();
    let yv: Vec<f64> = valid.iter().map(|&(_, y)| y).collect();
    let lin = fit_linear_cpu(&ln_x, &yv)?;

    let a = lin.slope;
    let b = lin.intercept;

    let r_squared = r_squared_from_residuals(&yv, valid.iter().map(|&(x, _)| a.mul_add(x.ln(), b)));

    Some(NonlinearFit {
        model: "logarithmic",
        params: vec![a, b],
        r_squared,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::tol;

    #[test]
    fn exponential_perfect_decay() {
        let a = 5.0;
        let b = -0.3;
        let xs: Vec<f64> = (0..10).map(f64::from).collect();
        let ys: Vec<f64> = xs.iter().map(|&x| a * (b * x).exp()).collect();
        let fit = fit_exponential(&xs, &ys).unwrap();
        assert!(
            (fit.params[0] - a).abs() < tol::EQUILIBRIUM,
            "a: {}",
            fit.params[0]
        );
        assert!(
            (fit.params[1] - b).abs() < tol::STOCHASTIC,
            "b: {}",
            fit.params[1]
        );
        assert!(fit.r_squared > 0.99);
    }

    #[test]
    fn exponential_no_positive_y() {
        let xs = [1.0, 2.0, 3.0];
        let ys = [-1.0, -2.0, -3.0];
        assert!(fit_exponential(&xs, &ys).is_none());
    }

    #[test]
    fn exponential_growth() {
        let xs: Vec<f64> = (0..8).map(|i| f64::from(i) * 0.5).collect();
        let a = 2.0_f64;
        let b = 0.8_f64;
        let ys: Vec<f64> = xs.iter().map(|&x| a * (b * x).exp()).collect();
        let fit = fit_exponential(&xs, &ys).expect("exponential fit");
        assert!((fit.params[0] - a).abs() < 0.1);
        assert!((fit.params[1] - b).abs() < 0.1);
        assert!(fit.r_squared > 0.99);
    }

    #[test]
    fn logarithmic_perfect_log() {
        let a = 3.0;
        let b = 2.0;
        let xs: Vec<f64> = (1..=10).map(f64::from).collect();
        let ys: Vec<f64> = xs.iter().map(|&x| a * x.ln() + b).collect();
        let fit = fit_logarithmic(&xs, &ys).unwrap();
        assert!(
            (fit.params[0] - a).abs() < tol::INTEGRATION,
            "a: {}",
            fit.params[0]
        );
        assert!(
            (fit.params[1] - b).abs() < tol::INTEGRATION,
            "b: {}",
            fit.params[1]
        );
        assert!(fit.r_squared > 0.999);
    }

    #[test]
    fn logarithmic_insufficient_data() {
        assert!(fit_logarithmic(&[1.0], &[1.0]).is_none());
    }

    #[test]
    fn logarithmic_noisy() {
        let xs: Vec<f64> = (1..=20).map(f64::from).collect();
        let a = 4.0;
        let b = 1.5;
        let ys: Vec<f64> = xs.iter().map(|&x| a * x.ln() + b).collect();
        let fit = fit_logarithmic(&xs, &ys).expect("logarithmic fit");
        assert!((fit.params[0] - a).abs() < 0.01);
        assert!((fit.params[1] - b).abs() < 0.01);
        assert!(fit.r_squared > 0.99);
    }

    #[test]
    fn logarithmic_no_positive_x() {
        let xs = [-1.0, -2.0, -3.0, -4.0];
        let ys = [1.0, 2.0, 3.0, 4.0];
        assert!(fit_logarithmic(&xs, &ys).is_none());
    }
}
