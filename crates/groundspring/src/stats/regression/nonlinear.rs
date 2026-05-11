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

/// Fit `y = 1 + A·x^b` (offset power law) via log-linearized least squares.
///
/// Filters to points where `x > 0` and `y > 1` (required for the log
/// transform `ln(y-1) = ln(A) + b·ln(x)`).  Returns `None` if fewer
/// than 2 valid points remain or the fit is degenerate.
///
/// This is the Wiser et al. (2013) power-law fitness model where `x` is
/// generations and `y` is relative fitness.
///
/// # Panics
///
/// Panics if `xs` and `ys` have different lengths.
#[must_use]
pub fn fit_power_law(xs: &[f64], ys: &[f64]) -> Option<NonlinearFit> {
    assert_eq!(xs.len(), ys.len(), "xs and ys must have equal length");

    if xs.len() < 2 {
        return None;
    }

    let valid: Vec<(f64, f64)> = xs
        .iter()
        .zip(ys)
        .filter(|&(&x, &y)| x > 0.0 && y > 1.0)
        .map(|(&x, &y)| (x, y))
        .collect();
    if valid.len() < 2 {
        return None;
    }

    let ln_x: Vec<f64> = valid.iter().map(|&(x, _)| x.ln()).collect();
    let ln_ym1: Vec<f64> = valid.iter().map(|&(_, y)| (y - 1.0).ln()).collect();
    let lin = fit_linear_cpu(&ln_x, &ln_ym1)?;

    let b = lin.slope;
    let a_coeff = lin.intercept.exp();

    let yv: Vec<f64> = valid.iter().map(|&(_, y)| y).collect();
    let r_squared = r_squared_from_residuals(
        &yv,
        valid.iter().map(|&(x, _)| a_coeff.mul_add(x.powf(b), 1.0)),
    );

    Some(NonlinearFit {
        model: "power_law",
        params: vec![a_coeff, b],
        r_squared,
    })
}

/// Fit `y = 1 + a·x/(1 + b·x)` (hyperbolic / Michaelis-Menten offset)
/// via reciprocal linearization.
///
/// Rearranges to `x/(y-1) = 1/a + (b/a)·x` and applies linear OLS.
/// Filters to `x > 0`, `y > 1`. Returns `None` if fewer than 2 valid
/// points remain or derived parameters are non-physical (`a <= 0`).
///
/// This is the Wiser et al. (2013) hyperbolic fitness model where fitness
/// approaches an asymptote `1 + a/b`.
///
/// # Panics
///
/// Panics if `xs` and `ys` have different lengths.
#[must_use]
pub fn fit_hyperbolic(xs: &[f64], ys: &[f64]) -> Option<NonlinearFit> {
    assert_eq!(xs.len(), ys.len(), "xs and ys must have equal length");

    if xs.len() < 2 {
        return None;
    }

    let valid: Vec<(f64, f64)> = xs
        .iter()
        .zip(ys)
        .filter(|&(&x, &y)| x > 0.0 && y > 1.0)
        .map(|(&x, &y)| (x, y))
        .collect();
    if valid.len() < 2 {
        return None;
    }

    let xv: Vec<f64> = valid.iter().map(|&(x, _)| x).collect();
    let ratio: Vec<f64> = valid.iter().map(|&(x, y)| x / (y - 1.0)).collect();
    let lin = fit_linear_cpu(&xv, &ratio)?;

    let inv_a = lin.intercept;
    let b_over_a = lin.slope;

    if inv_a <= 0.0 {
        return None;
    }

    let a = 1.0 / inv_a;
    let b = b_over_a * a;

    if b < 0.0 {
        return None;
    }

    let yv: Vec<f64> = valid.iter().map(|&(_, y)| y).collect();
    let r_squared =
        r_squared_from_residuals(&yv, valid.iter().map(|&(x, _)| 1.0 + a * x / (1.0 + b * x)));

    Some(NonlinearFit {
        model: "hyperbolic",
        params: vec![a, b],
        r_squared,
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
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

    #[test]
    fn power_law_perfect() {
        let a = 0.01;
        let b = 0.5;
        let xs: Vec<f64> = (1..=20).map(|i| f64::from(i) * 500.0).collect();
        let ys: Vec<f64> = xs.iter().map(|&x| 1.0 + a * x.powf(b)).collect();
        let fit = fit_power_law(&xs, &ys).unwrap();
        assert_eq!(fit.model, "power_law");
        assert!((fit.params[1] - b).abs() < 0.01, "b: {}", fit.params[1]);
        assert!(fit.r_squared > 0.999, "R²: {}", fit.r_squared);
    }

    #[test]
    fn power_law_insufficient_data() {
        assert!(fit_power_law(&[1.0], &[2.0]).is_none());
    }

    #[test]
    fn power_law_no_valid_points() {
        let xs = [0.0, -1.0];
        let ys = [0.5, 0.8];
        assert!(fit_power_law(&xs, &ys).is_none());
    }

    #[test]
    fn hyperbolic_perfect() {
        let a = 0.001;
        let b = 0.0001;
        let xs: Vec<f64> = (1..=20).map(|i| f64::from(i) * 2500.0).collect();
        let ys: Vec<f64> = xs.iter().map(|&x| 1.0 + a * x / (1.0 + b * x)).collect();
        let fit = fit_hyperbolic(&xs, &ys).unwrap();
        assert_eq!(fit.model, "hyperbolic");
        assert!(fit.r_squared > 0.999, "R²: {}", fit.r_squared);
    }

    #[test]
    fn hyperbolic_insufficient_data() {
        assert!(fit_hyperbolic(&[1.0], &[2.0]).is_none());
    }

    #[test]
    fn hyperbolic_asymptote() {
        let a = 0.005;
        let b = 0.001;
        let asymptote = 1.0 + a / b;
        let xs: Vec<f64> = (1..=50).map(|i| f64::from(i) * 1000.0).collect();
        let ys: Vec<f64> = xs.iter().map(|&x| 1.0 + a * x / (1.0 + b * x)).collect();
        let fit = fit_hyperbolic(&xs, &ys).unwrap();
        let fit_asymptote = 1.0 + fit.params[0] / fit.params[1];
        assert!(
            (fit_asymptote - asymptote).abs() < 0.1,
            "asymptote: {fit_asymptote} vs {asymptote}"
        );
    }
}
