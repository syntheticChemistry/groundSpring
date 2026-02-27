// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Simple linear regression: `y = intercept + slope × x`.
//!
//! Provides `fit_linear` for ordinary least squares on `&[f64]` slices.
//! Used by [`crate::wdm`] (finite-size extrapolation) and [`crate::transport`]
//! (log-log MSD regression). Consolidates the two prior implementations into
//! a single source of truth.
//!
//! # barracuda delegation
//!
//! When the `barracuda` feature is enabled, [`fit_linear`] delegates to
//! `barracuda::stats::regression::fit_linear`, falling back to the local
//! implementation on error.

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

#[expect(clippy::similar_names, clippy::suspicious_operation_groupings)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perfect_positive_line() {
        let xs = [1.0, 2.0, 3.0, 4.0, 5.0];
        let ys = [3.0, 5.0, 7.0, 9.0, 11.0];
        let fit = fit_linear(&xs, &ys).unwrap();
        assert!((fit.slope - 2.0).abs() < 1e-10);
        assert!((fit.intercept - 1.0).abs() < 1e-10);
        assert!((fit.r_squared - 1.0).abs() < 1e-10);
    }

    #[test]
    fn constant_y() {
        let xs = [1.0, 2.0, 3.0, 4.0];
        let ys = [5.0, 5.0, 5.0, 5.0];
        let fit = fit_linear(&xs, &ys).unwrap();
        assert!(fit.slope.abs() < 1e-10);
        assert!((fit.intercept - 5.0).abs() < 1e-10);
        assert!((fit.r_squared - 1.0).abs() < 1e-10);
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
        assert!((fit.slope - (-3.0)).abs() < 1e-10);
        assert!((fit.intercept - 10.0).abs() < 1e-10);
    }
}
