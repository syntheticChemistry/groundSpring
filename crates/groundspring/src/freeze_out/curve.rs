// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Forward model and chi-squared evaluation for the freeze-out polynomial.
//!
//! `T_f(μ_B) = T₀ (1 - κ₂ (μ_B/T₀)²)`
//!
//! [`freeze_out_curve`] and [`chi_squared`] stay local (scalar ops, below
//! GPU promotion threshold).

use crate::cast::usize_f64;

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

/// Chi-squared for the freeze-out model at a single (T₀, κ₂) point.
///
/// Shared by grid search (CPU and GPU), L-BFGS refinement, and
/// Nelder-Mead multi-start — avoids four copies of the same loop.
#[inline]
pub(super) fn chi2_freeze_out(
    observed: &[f64],
    mu_b: &[f64],
    t0: f64,
    k2: f64,
    inv_sigma2: f64,
) -> f64 {
    observed
        .iter()
        .zip(mu_b.iter())
        .map(|(&o, &mu)| (o - freeze_out_curve(t0, k2, mu)).powi(2) * inv_sigma2)
        .sum()
}
