// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Warm Dense Matter transport analysis utilities.
//!
//! Provides Green-Kubo integration, synthetic autocorrelation generation,
//! and finite-size extrapolation for WDM transport coefficient analysis.
//!
//! These functions support groundSpring's uncertainty quantification
//! methodology applied to molecular dynamics transport coefficients.
//!
//! # barracuda delegation
//!
//! When the `barracuda` feature is enabled:
//! - [`green_kubo_integrate`] delegates to `barracuda::numerical::trapz`
//!   (trapezoidal rule on explicit x-y arrays). Falls back on error.
//! - [`finite_size_extrapolate`] delegates linear regression to
//!   `barracuda::stats::regression::fit_linear` via [`crate::stats::fit_linear`].

/// Numerically integrate an autocorrelation function using the trapezoidal rule.
///
/// Computes ∫₀ᵀ acf(t) dt where T = (len-1) × dt.
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::numerical::trapz` which uses the same trapezoidal rule
/// but on explicit (x, y) arrays. Falls back to the local implementation
/// on error.
#[must_use]
pub fn green_kubo_integrate(acf: &[f64], dt: f64) -> f64 {
    if acf.len() < 2 {
        return 0.0;
    }

    #[cfg(feature = "barracuda")]
    {
        let x: Vec<f64> = (0..acf.len())
            .map(|i| {
                #[expect(clippy::cast_precision_loss)]
                let t = i as f64 * dt;
                t
            })
            .collect();
        if let Ok(val) = barracuda::numerical::trapz(acf, &x) {
            return val;
        }
    }

    green_kubo_integrate_cpu(acf, dt)
}

fn green_kubo_integrate_cpu(acf: &[f64], dt: f64) -> f64 {
    let n = acf.len();
    let sum = 0.5_f64.mul_add(acf[0] + acf[n - 1], acf[1..n - 1].iter().sum::<f64>());
    sum * dt
}

/// Green-Kubo integration with f32 accumulation.
///
/// Simulates reduced-precision GPU arithmetic by casting each value to f32
/// before accumulating. The running sum is maintained in f32 precision
/// throughout, then cast to f64 at the end.
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    reason = "deliberate f64→f32 precision reduction"
)]
pub fn green_kubo_integrate_f32(acf: &[f64], dt: f64) -> f64 {
    if acf.len() < 2 {
        return 0.0;
    }
    let n = acf.len();
    let dt_f32 = dt as f32;
    let sum: f32 = 0.5_f32.mul_add(
        acf[0] as f32 + acf[n - 1] as f32,
        acf[1..n - 1].iter().map(|&v| v as f32).sum::<f32>(),
    );
    f64::from(sum) * f64::from(dt_f32)
}

/// Generate a synthetic velocity autocorrelation function.
///
/// Uses the exponential decay model: C(t) = c₀ × exp(-t/τ).
/// The analytical integral is c₀ × τ.
#[must_use]
pub fn synthetic_vacf(c0: f64, tau: f64, n_steps: usize, dt: f64) -> Vec<f64> {
    (0..n_steps)
        .map(|i| {
            #[expect(clippy::cast_precision_loss)]
            let t = i as f64 * dt;
            c0 * (-t / tau).exp()
        })
        .collect()
}

/// Analytical diffusion coefficient from exponential VACF parameters.
///
/// D = c₀ × τ / d (Green-Kubo relation for isotropic diffusion).
#[must_use]
pub fn analytical_diffusion(c0: f64, tau: f64, d_dim: f64) -> f64 {
    c0 * tau / d_dim
}

/// Finite-size extrapolation for transport coefficients.
///
/// Fits D(N) = D∞ + α / N^(1/d) using linear regression on the
/// transformed variable x = 1/N^(1/d). Returns `(d_inf, alpha, r_squared)`.
///
/// Reference: Yeh & Hummer (2004) J. Phys. Chem. B 108, 15873.
///
/// # Errors
///
/// Returns [`crate::error::InputError::LengthMismatch`] if `sizes` and `values` differ
/// in length, or [`crate::error::InputError::InsufficientData`] if fewer than 2 points.
pub fn finite_size_extrapolate(
    sizes: &[f64],
    values: &[f64],
    d_dim: f64,
) -> Result<(f64, f64, f64), crate::error::InputError> {
    if sizes.len() != values.len() {
        return Err(crate::error::InputError::LengthMismatch {
            first: "sizes",
            first_len: sizes.len(),
            second: "values",
            second_len: values.len(),
        });
    }
    if sizes.len() < 2 {
        return Err(crate::error::InputError::InsufficientData {
            name: "sizes",
            min: 2,
            got: sizes.len(),
        });
    }

    let exponent = 1.0 / d_dim;
    let xs: Vec<f64> = sizes.iter().map(|&s| 1.0 / s.powf(exponent)).collect();

    let fit = crate::stats::fit_linear(&xs, values).ok_or(
        crate::error::InputError::InsufficientData {
            name: "sizes",
            min: 2,
            got: sizes.len(),
        },
    )?;
    Ok((fit.intercept, fit.slope, fit.r_squared))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn green_kubo_exponential_decay() {
        let c0 = 1.0;
        let tau = 10.0;
        let dt = 0.001;
        let n_steps = 100_000;
        let vacf = synthetic_vacf(c0, tau, n_steps, dt);
        let integral = green_kubo_integrate(&vacf, dt);
        let analytical = c0 * tau;
        let rel_err = (integral - analytical).abs() / analytical;
        assert!(rel_err < 0.001, "relative error {rel_err:.6} exceeds 0.1%");
    }

    #[test]
    fn green_kubo_f32_bounded_error() {
        let c0 = 1.0;
        let tau = 10.0;
        let dt = 0.001;
        let n_steps = 100_000;
        let vacf = synthetic_vacf(c0, tau, n_steps, dt);
        let f64_result = green_kubo_integrate(&vacf, dt);
        let f32_result = green_kubo_integrate_f32(&vacf, dt);
        let rel_err = (f32_result - f64_result).abs() / f64_result;
        assert!(rel_err < 0.01, "f32 relative error {rel_err:.6} exceeds 1%");
    }

    #[test]
    fn analytical_diffusion_3d() {
        let d = analytical_diffusion(1.0, 10.0, 3.0);
        assert!((d - 10.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn finite_size_extrapolation_perfect_data() {
        let d_inf_true = 2.0;
        let alpha_true = 5.0;
        let sizes = vec![100.0, 500.0, 1000.0, 5000.0, 10000.0];
        let values: Vec<f64> = sizes
            .iter()
            .map(|&n: &f64| d_inf_true + alpha_true / n.cbrt())
            .collect();
        let (d_inf, alpha, r_sq) = finite_size_extrapolate(&sizes, &values, 3.0).unwrap();
        assert!(
            (d_inf - d_inf_true).abs() < 0.001,
            "D_inf: {d_inf} vs {d_inf_true}"
        );
        assert!(
            (alpha - alpha_true).abs() < 0.01,
            "alpha: {alpha} vs {alpha_true}"
        );
        assert!(r_sq > 0.999, "R²: {r_sq}");
    }

    #[test]
    fn empty_acf_returns_zero() {
        assert!(green_kubo_integrate(&[], 0.001).abs() < f64::EPSILON);
        assert!(green_kubo_integrate(&[1.0], 0.001).abs() < f64::EPSILON);
        assert!(green_kubo_integrate_f32(&[], 0.001).abs() < f64::EPSILON);
        assert!(green_kubo_integrate_f32(&[1.0], 0.001).abs() < f64::EPSILON);
    }

    #[test]
    fn finite_size_extrapolate_length_mismatch() {
        let sizes = vec![100.0, 500.0];
        let values = vec![1.0];
        let result = finite_size_extrapolate(&sizes, &values, 3.0);
        assert!(result.is_err());
    }

    #[test]
    fn finite_size_extrapolate_insufficient_data() {
        let sizes = vec![100.0];
        let values = vec![1.0];
        let result = finite_size_extrapolate(&sizes, &values, 3.0);
        assert!(result.is_err());
    }

    #[test]
    fn finite_size_extrapolate_two_points_minimal() {
        let sizes = vec![100.0, 1000.0];
        let values = vec![3.0, 2.5];
        let result = finite_size_extrapolate(&sizes, &values, 3.0);
        assert!(result.is_ok());
        let (d_inf, _alpha, r_sq) = result.unwrap();
        assert!(d_inf.is_finite());
        assert!((r_sq - 1.0).abs() < 1e-10, "two-point fit R² = 1.0");
    }
}
