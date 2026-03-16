// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Quadratic regression via normal equations (3×3 Cramer).

use crate::cast::usize_f64;

use super::NonlinearFit;
use super::linear::r_squared_from_residuals;

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

    let r_squared =
        r_squared_from_residuals(ys, xs.iter().map(|&x| a.mul_add(x * x, b.mul_add(x, c))));

    Some(NonlinearFit {
        model: "quadratic",
        params: vec![a, b, c],
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
    fn quadratic_concave_up() {
        let xs: Vec<f64> = (0..10).map(|i| f64::from(i) - 5.0).collect();
        let ys: Vec<f64> = xs
            .iter()
            .map(|&x| 2.0_f64.mul_add(x * x, x) + 3.0)
            .collect();
        let fit = fit_quadratic(&xs, &ys).expect("quadratic fit");
        assert!((fit.params[0] - 2.0).abs() < tol::ANALYTICAL);
        assert!((fit.params[1] - 1.0).abs() < tol::ANALYTICAL);
        assert!((fit.params[2] - 3.0).abs() < tol::ANALYTICAL);
        assert!(fit.r_squared > 0.999);
    }
}
