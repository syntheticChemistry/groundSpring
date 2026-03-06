// SPDX-License-Identifier: AGPL-3.0-only
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

use crate::cast::usize_f64;

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

/// Fit `y = intercept + slope × x` via ordinary least squares.
///
/// Returns `None` when fewer than 2 data points are provided or when
/// all x-values are identical (zero variance → undefined slope).
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::stats::regression::fit_linear`, falling back to the
/// always-compiled CPU path on error.
///
/// # Panics
///
/// Panics if `xs` and `ys` have different lengths.
#[must_use]
pub fn fit_linear(xs: &[f64], ys: &[f64]) -> Option<LinearFit> {
    assert_eq!(xs.len(), ys.len(), "xs and ys must have equal length");

    if xs.len() < 2 {
        return None;
    }

    #[cfg(feature = "barracuda")]
    if let Some(fit) = barracuda::stats::regression::fit_linear(xs, ys) {
        return Some(LinearFit {
            intercept: fit.params[1],
            slope: fit.params[0],
            r_squared: fit.r_squared,
        });
    }

    fit_linear_cpu(xs, ys)
}

#[expect(
    clippy::similar_names,
    clippy::suspicious_operation_groupings,
    reason = "mathematical variable names (sx, sy, sxy, sxx) follow regression notation"
)]
fn fit_linear_cpu(xs: &[f64], ys: &[f64]) -> Option<LinearFit> {
    let n = usize_f64(xs.len());
    let x_mean = xs.iter().sum::<f64>() / n;
    let y_mean = ys.iter().sum::<f64>() / n;

    let (mut ss_xy, mut ss_xx, mut ss_yy) = (0.0, 0.0, 0.0);
    for (&x, &y) in xs.iter().zip(ys) {
        let dx = x - x_mean;
        let dy = y - y_mean;
        ss_xy += dx * dy;
        ss_xx += dx * dx;
        ss_yy += dy * dy;
    }

    if ss_xx == 0.0 {
        return None;
    }

    let slope = ss_xy / ss_xx;
    let intercept = slope.mul_add(-x_mean, y_mean);
    let r_squared = if ss_yy > 0.0 {
        (ss_xy * ss_xy) / (ss_xx * ss_yy)
    } else {
        1.0
    };

    Some(LinearFit {
        intercept,
        slope,
        r_squared,
    })
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

/// Fit `y = a·x² + b·x + c` via normal equations (3×3 Cramer).
///
/// Returns `None` when fewer than 3 data points are provided or the
/// Vandermonde system is singular.
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::stats::regression::fit_quadratic`.
///
/// # Panics
///
/// Panics if `xs` and `ys` have different lengths.
#[must_use]
pub fn fit_quadratic(xs: &[f64], ys: &[f64]) -> Option<NonlinearFit> {
    assert_eq!(xs.len(), ys.len(), "xs and ys must have equal length");

    if xs.len() < 3 {
        return None;
    }

    #[cfg(feature = "barracuda")]
    if let Some(fit) = barracuda::stats::regression::fit_quadratic(xs, ys) {
        return Some(NonlinearFit {
            model: "quadratic",
            params: fit.params,
            r_squared: fit.r_squared,
        });
    }

    fit_quadratic_cpu(xs, ys)
}

/// Below this threshold, Cramer's rule treats the system as singular.
const SINGULARITY_THRESHOLD: f64 = 1e-30;

/// 3×3 determinant via cofactor expansion.
#[expect(
    clippy::suboptimal_flops,
    reason = "Sarrus rule is clearer than nested mul_add for 3×3"
)]
fn det3(m: [[f64; 3]; 3]) -> f64 {
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}

/// Solve a 3×3 system Mx = rhs via Cramer's rule.
fn cramer3(m: [[f64; 3]; 3], rhs: [f64; 3]) -> Option<[f64; 3]> {
    let d = det3(m);
    if d.abs() < SINGULARITY_THRESHOLD {
        return None;
    }
    let mut result = [0.0; 3];
    for col in 0..3 {
        let mut mc = m;
        for row in 0..3 {
            mc[row][col] = rhs[row];
        }
        result[col] = det3(mc) / d;
    }
    Some(result)
}

#[expect(
    clippy::similar_names,
    clippy::many_single_char_names,
    reason = "mathematical notation: a, b, c coefficients in quadratic fit"
)]
fn fit_quadratic_cpu(xs: &[f64], ys: &[f64]) -> Option<NonlinearFit> {
    let n = usize_f64(xs.len());
    let sx: f64 = xs.iter().sum();
    let sx2: f64 = xs.iter().map(|x| x * x).sum();
    let sx3: f64 = xs.iter().map(|x| x.powi(3)).sum();
    let sx4: f64 = xs.iter().map(|x| x.powi(4)).sum();
    let sy: f64 = ys.iter().sum();
    let sxy: f64 = xs.iter().zip(ys).map(|(&x, &y)| x * y).sum();
    let sx2y: f64 = xs.iter().zip(ys).map(|(&x, &y)| x * x * y).sum();

    let m = [[sx4, sx3, sx2], [sx3, sx2, sx], [sx2, sx, n]];
    let rhs = [sx2y, sxy, sy];
    let [a, b, c] = cramer3(m, rhs)?;

    let y_mean = sy / n;
    let ss_tot: f64 = ys.iter().map(|&y| (y - y_mean).powi(2)).sum();
    let ss_res: f64 = xs
        .iter()
        .zip(ys)
        .map(|(&x, &y)| {
            let pred = a.mul_add(x * x, b.mul_add(x, c));
            (y - pred).powi(2)
        })
        .sum();
    let r_squared = if ss_tot > 0.0 {
        1.0 - ss_res / ss_tot
    } else {
        1.0
    };

    Some(NonlinearFit {
        model: "quadratic",
        params: vec![a, b, c],
        r_squared,
    })
}

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
    let y_mean: f64 = yv.iter().sum::<f64>() / usize_f64(yv.len());
    let ss_tot: f64 = yv.iter().map(|&y| (y - y_mean).powi(2)).sum();
    let ss_res: f64 = xv
        .iter()
        .zip(&yv)
        .map(|(&x, &y)| {
            let pred = a * (b * x).exp();
            (y - pred).powi(2)
        })
        .sum();
    let r_squared = if ss_tot > 0.0 {
        1.0 - ss_res / ss_tot
    } else {
        1.0
    };

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

    let y_mean: f64 = yv.iter().sum::<f64>() / usize_f64(yv.len());
    let ss_tot: f64 = yv.iter().map(|&y| (y - y_mean).powi(2)).sum();
    let ss_res: f64 = valid
        .iter()
        .map(|&(x, y)| (y - a.mul_add(x.ln(), b)).powi(2))
        .sum();
    let r_squared = if ss_tot > 0.0 {
        1.0 - ss_res / ss_tot
    } else {
        1.0
    };

    Some(NonlinearFit {
        model: "logarithmic",
        params: vec![a, b],
        r_squared,
    })
}

/// Fit all four models and return those that converge.
///
/// Runs [`fit_linear`], [`fit_quadratic`], [`fit_exponential`], and
/// [`fit_logarithmic`] on the same data, collecting any that succeed.
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
    ]
    .into_iter()
    .flatten()
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tol;

    #[test]
    fn perfect_positive_line() {
        let xs = [1.0, 2.0, 3.0, 4.0, 5.0];
        let ys = [3.0, 5.0, 7.0, 9.0, 11.0];
        let fit = fit_linear(&xs, &ys).unwrap();
        assert!((fit.slope - 2.0).abs() < tol::ANALYTICAL);
        assert!((fit.intercept - 1.0).abs() < tol::ANALYTICAL);
        assert!((fit.r_squared - 1.0).abs() < tol::ANALYTICAL);
    }

    #[test]
    fn constant_y() {
        let xs = [1.0, 2.0, 3.0, 4.0];
        let ys = [5.0, 5.0, 5.0, 5.0];
        let fit = fit_linear(&xs, &ys).unwrap();
        assert!(fit.slope.abs() < tol::ANALYTICAL);
        assert!((fit.intercept - 5.0).abs() < tol::ANALYTICAL);
        assert!((fit.r_squared - 1.0).abs() < tol::ANALYTICAL);
    }

    #[test]
    fn constant_x_returns_none() {
        let xs = [3.0, 3.0, 3.0];
        let ys = [1.0, 2.0, 3.0];
        assert!(fit_linear(&xs, &ys).is_none());
    }

    #[test]
    fn insufficient_data_returns_none() {
        assert!(fit_linear(&[1.0], &[2.0]).is_none());
        assert!(fit_linear(&[], &[]).is_none());
    }

    #[test]
    fn negative_slope() {
        let xs = [0.0, 1.0, 2.0, 3.0];
        let ys = [10.0, 7.0, 4.0, 1.0];
        let fit = fit_linear(&xs, &ys).unwrap();
        assert!((fit.slope - (-3.0)).abs() < tol::ANALYTICAL);
        assert!((fit.intercept - 10.0).abs() < tol::ANALYTICAL);
    }

    #[test]
    fn quadratic_perfect_parabola() {
        let xs: Vec<f64> = (-5..=5).map(f64::from).collect();
        let ys: Vec<f64> = xs
            .iter()
            .map(|&x| (2.0 * x).mul_add(x, (-3.0f64).mul_add(x, 1.0)))
            .collect();
        let fit = fit_quadratic(&xs, &ys).unwrap();
        assert!(
            (fit.params[0] - 2.0).abs() < tol::INTEGRATION,
            "a: {}",
            fit.params[0]
        );
        assert!(
            (fit.params[1] - (-3.0)).abs() < tol::INTEGRATION,
            "b: {}",
            fit.params[1]
        );
        assert!(
            (fit.params[2] - 1.0).abs() < tol::INTEGRATION,
            "c: {}",
            fit.params[2]
        );
        assert!(fit.r_squared > 0.999);
    }

    #[test]
    fn quadratic_insufficient_data() {
        assert!(fit_quadratic(&[1.0, 2.0], &[1.0, 4.0]).is_none());
        assert!(fit_quadratic(&[1.0], &[1.0]).is_none());
    }

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
