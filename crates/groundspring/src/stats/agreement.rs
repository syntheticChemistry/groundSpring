// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Paired-observation agreement and error metrics.
//!
//! All functions take `(observed, modeled)` slice pairs and quantify how
//! well the model reproduces the observations. When the `barracuda`
//! feature is enabled, each metric delegates to `barracuda::stats`.

#[cfg(not(feature = "barracuda"))]
use crate::cast::usize_f64;

/// Shared coefficient-of-efficiency computation used by both R² and NSE.
///
/// Computes `1 - SS_res / SS_tot` where `SS_tot` uses the observed mean.
/// R² and NSE are mathematically identical for the (observed, modeled)
/// formulation — they differ only in naming convention across domains.
#[cfg(not(feature = "barracuda"))]
fn coefficient_of_efficiency(observed: &[f64], modeled: &[f64]) -> f64 {
    let n = observed.len();
    if n == 0 {
        return 0.0;
    }
    let mean_obs: f64 = observed.iter().sum::<f64>() / usize_f64(n);
    let ss_res: f64 = observed
        .iter()
        .zip(modeled)
        .map(|(o, m)| (o - m).powi(2))
        .sum();
    let ss_tot: f64 = observed.iter().map(|o| (o - mean_obs).powi(2)).sum();
    if ss_tot == 0.0 {
        return 0.0;
    }
    1.0 - ss_res / ss_tot
}

/// GPU-accelerated coefficient of efficiency (R²/NSE) via two
/// `FusedMapReduceF64::sum_of_squares` dispatches for `SS_res` and `SS_tot`.
#[cfg(feature = "barracuda-gpu")]
fn coefficient_of_efficiency_gpu(observed: &[f64], modeled: &[f64]) -> Option<f64> {
    if observed.is_empty() {
        return Some(0.0);
    }
    let device = crate::gpu::get_device_f64_safe()?;
    let fmr = barracuda::ops::fused_map_reduce_f64::FusedMapReduceF64::new(device).ok()?;
    let mean_obs = barracuda::ops::sum_reduce_f64::SumReduceF64::mean(
        crate::gpu::get_device_f64_safe()?,
        observed,
    )
    .ok()?;
    let residuals: Vec<f64> = observed.iter().zip(modeled).map(|(o, m)| o - m).collect();
    let deviations: Vec<f64> = observed.iter().map(|o| o - mean_obs).collect();
    let ss_res = fmr.sum_of_squares(&residuals).ok()?;
    let ss_tot = fmr.sum_of_squares(&deviations).ok()?;
    if ss_tot == 0.0 {
        return Some(0.0);
    }
    Some(1.0 - ss_res / ss_tot)
}

/// Root Mean Square Error between observed and modeled values.
///
/// When `barracuda-gpu` is enabled and a GPU is available, computes
/// RMSE via `FusedMapReduceF64::sum_of_squares` on residuals.
/// Otherwise delegates to `barracuda::stats::rmse` (CPU) or the local
/// implementation.  Returns `0.0` for empty slices.
///
/// # Panics
///
/// Panics if `observed` and `modeled` have different lengths.
///
/// # Examples
///
/// ```
/// let obs = [1.0, 2.0, 3.0];
/// let mod_ = [1.1, 2.0, 2.9];
/// let r = groundspring::stats::rmse(&obs, &mod_);
/// assert!(r > 0.0 && r < 0.15);
/// assert_eq!(groundspring::stats::rmse(&[], &[]), 0.0);
/// ```
#[must_use]
pub fn rmse(observed: &[f64], modeled: &[f64]) -> f64 {
    assert_eq!(
        observed.len(),
        modeled.len(),
        "observed and modeled must have equal length"
    );
    #[cfg(feature = "barracuda-gpu")]
    {
        if let Some(r) = rmse_gpu(observed, modeled) {
            return r;
        }
    }
    #[cfg(feature = "barracuda")]
    {
        barracuda::stats::rmse(observed, modeled)
    }
    #[cfg(not(feature = "barracuda"))]
    rmse_cpu(observed, modeled)
}

#[cfg(feature = "barracuda-gpu")]
fn rmse_gpu(observed: &[f64], modeled: &[f64]) -> Option<f64> {
    if observed.is_empty() {
        return Some(0.0);
    }
    let device = crate::gpu::get_device_f64_safe()?;
    let residuals: Vec<f64> = observed.iter().zip(modeled).map(|(o, m)| o - m).collect();
    let gpu = barracuda::ops::fused_map_reduce_f64::FusedMapReduceF64::new(device).ok()?;
    let ss = gpu.sum_of_squares(&residuals).ok()?;
    Some((ss / crate::cast::usize_f64(residuals.len())).sqrt())
}

#[cfg(not(feature = "barracuda"))]
fn rmse_cpu(observed: &[f64], modeled: &[f64]) -> f64 {
    let n = observed.len();
    if n == 0 {
        return 0.0;
    }
    let sum_sq: f64 = observed
        .iter()
        .zip(modeled)
        .map(|(o, m)| (o - m).powi(2))
        .sum();
    (sum_sq / usize_f64(n)).sqrt()
}

/// Mean Absolute Error between observed and modeled values.
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::stats::mae` (absorbed from airSpring/groundSpring S64).
/// Returns `0.0` for empty slices.
///
/// # Panics
///
/// Panics if `observed` and `modeled` have different lengths.
#[must_use]
pub fn mae(observed: &[f64], modeled: &[f64]) -> f64 {
    assert_eq!(
        observed.len(),
        modeled.len(),
        "observed and modeled must have equal length"
    );
    #[cfg(feature = "barracuda-gpu")]
    {
        if let Some(m) = mae_gpu(observed, modeled) {
            return m;
        }
    }
    #[cfg(feature = "barracuda")]
    {
        barracuda::stats::mae(observed, modeled)
    }
    #[cfg(not(feature = "barracuda"))]
    mae_cpu(observed, modeled)
}

#[cfg(feature = "barracuda-gpu")]
fn mae_gpu(observed: &[f64], modeled: &[f64]) -> Option<f64> {
    if observed.is_empty() {
        return Some(0.0);
    }
    let residuals: Vec<f64> = observed.iter().zip(modeled).map(|(o, m)| o - m).collect();
    let device = crate::gpu::get_device_f64_safe()?;
    let fmr = barracuda::ops::fused_map_reduce_f64::FusedMapReduceF64::new(device).ok()?;
    let l1 = fmr.l1_norm(&residuals).ok()?;
    Some(l1 / crate::cast::usize_f64(residuals.len()))
}

#[cfg(not(feature = "barracuda"))]
fn mae_cpu(observed: &[f64], modeled: &[f64]) -> f64 {
    let n = observed.len();
    if n == 0 {
        return 0.0;
    }
    observed
        .iter()
        .zip(modeled)
        .map(|(o, m)| (o - m).abs())
        .sum::<f64>()
        / usize_f64(n)
}

/// Nash-Sutcliffe Efficiency (NSE).
///
/// Mathematically identical to [`r_squared`] for the (observed, modeled)
/// formulation — both use the same `1 - SS_res / SS_tot` computation.  Exposed as a separate
/// API because hydrology (FAO-56, water balance) uses the NSE name while
/// statistics uses R².
///
/// NSE = 1 is perfect; NSE = 0 means the model is no better than the mean;
/// NSE < 0 means the model is worse than the mean.
///
/// # Panics
///
/// Panics if `observed` and `modeled` have different lengths.
#[must_use]
pub fn nash_sutcliffe(observed: &[f64], modeled: &[f64]) -> f64 {
    assert_eq!(
        observed.len(),
        modeled.len(),
        "observed and modeled must have equal length"
    );
    #[cfg(feature = "barracuda-gpu")]
    {
        if let Some(nse) = coefficient_of_efficiency_gpu(observed, modeled) {
            return nse;
        }
    }
    #[cfg(feature = "barracuda")]
    {
        barracuda::stats::nash_sutcliffe(observed, modeled)
    }
    #[cfg(not(feature = "barracuda"))]
    coefficient_of_efficiency(observed, modeled)
}

/// Mean Bias Error (modeled − observed).
///
/// Positive MBE indicates the model overestimates.
/// When `barracuda-gpu` is enabled, dispatches to `SumReduceF64::mean`
/// on the residual vector.
///
/// # Panics
///
/// Panics if `observed` and `modeled` have different lengths.
#[must_use]
pub fn mbe(observed: &[f64], modeled: &[f64]) -> f64 {
    assert_eq!(
        observed.len(),
        modeled.len(),
        "observed and modeled must have equal length"
    );
    #[cfg(feature = "barracuda-gpu")]
    {
        if let Some(b) = mbe_gpu(observed, modeled) {
            return b;
        }
    }
    #[cfg(feature = "barracuda")]
    {
        barracuda::stats::mbe(observed, modeled)
    }
    #[cfg(not(feature = "barracuda"))]
    mbe_cpu(observed, modeled)
}

#[cfg(feature = "barracuda-gpu")]
fn mbe_gpu(observed: &[f64], modeled: &[f64]) -> Option<f64> {
    if observed.is_empty() {
        return Some(0.0);
    }
    let device = crate::gpu::get_device_f64_safe()?;
    let residuals: Vec<f64> = observed.iter().zip(modeled).map(|(o, m)| m - o).collect();
    barracuda::ops::sum_reduce_f64::SumReduceF64::mean(device, &residuals).ok()
}

#[cfg(not(feature = "barracuda"))]
fn mbe_cpu(observed: &[f64], modeled: &[f64]) -> f64 {
    let n = observed.len();
    if n == 0 {
        return 0.0;
    }
    observed
        .iter()
        .zip(modeled)
        .map(|(o, m)| m - o)
        .sum::<f64>()
        / usize_f64(n)
}

/// Coefficient of determination (R²).
///
/// Mathematically identical to [`nash_sutcliffe`] for the (observed, modeled)
/// formulation.  Returns `0.0` when total sum of squares is zero.
///
/// # Panics
///
/// Panics if `observed` and `modeled` have different lengths.
#[must_use]
pub fn r_squared(observed: &[f64], modeled: &[f64]) -> f64 {
    assert_eq!(
        observed.len(),
        modeled.len(),
        "observed and modeled must have equal length"
    );
    #[cfg(feature = "barracuda-gpu")]
    {
        if let Some(r2) = coefficient_of_efficiency_gpu(observed, modeled) {
            return r2;
        }
    }
    #[cfg(feature = "barracuda")]
    {
        barracuda::stats::r_squared(observed, modeled)
    }
    #[cfg(not(feature = "barracuda"))]
    coefficient_of_efficiency(observed, modeled)
}

/// Index of Agreement (Willmott 1981).
///
/// Ranges from 0.0 (no agreement) to 1.0 (perfect agreement).
///
/// # Panics
///
/// Panics if `observed` and `modeled` have different lengths.
#[must_use]
pub fn index_of_agreement(observed: &[f64], modeled: &[f64]) -> f64 {
    assert_eq!(
        observed.len(),
        modeled.len(),
        "observed and modeled must have equal length"
    );
    #[cfg(feature = "barracuda")]
    {
        barracuda::stats::index_of_agreement(observed, modeled)
    }
    #[cfg(not(feature = "barracuda"))]
    index_of_agreement_cpu(observed, modeled)
}

#[cfg(not(feature = "barracuda"))]
fn index_of_agreement_cpu(observed: &[f64], modeled: &[f64]) -> f64 {
    let n = observed.len();
    if n == 0 {
        return 0.0;
    }
    let mean_obs: f64 = observed.iter().sum::<f64>() / usize_f64(n);
    let numerator: f64 = observed
        .iter()
        .zip(modeled)
        .map(|(o, m)| (o - m).powi(2))
        .sum();
    let denominator: f64 = observed
        .iter()
        .zip(modeled)
        .map(|(o, m)| ((m - mean_obs).abs() + (o - mean_obs).abs()).powi(2))
        .sum();
    if denominator == 0.0 {
        return 0.0;
    }
    1.0 - numerator / denominator
}

/// Fraction of days where observed and modeled agree on occurrence.
///
/// A day "occurs" if the value exceeds `threshold`.  Returns the
/// fraction of days where both agree (both above or both at-or-below).
/// Returns `0.0` for empty slices.
///
/// # Panics
///
/// Panics if `observed` and `modeled` have different lengths.
#[must_use]
pub fn hit_rate(observed: &[f64], modeled: &[f64], threshold: f64) -> f64 {
    assert_eq!(
        observed.len(),
        modeled.len(),
        "observed and modeled must have equal length"
    );
    #[cfg(feature = "barracuda")]
    {
        barracuda::stats::hit_rate(observed, modeled, threshold)
    }
    #[cfg(not(feature = "barracuda"))]
    hit_rate_cpu(observed, modeled, threshold)
}

#[cfg(not(feature = "barracuda"))]
fn hit_rate_cpu(observed: &[f64], modeled: &[f64], threshold: f64) -> f64 {
    let n = observed.len();
    if n == 0 {
        return 0.0;
    }
    let agree = observed
        .iter()
        .zip(modeled)
        .filter(|(&o, &m)| (o > threshold) == (m > threshold))
        .count();
    usize_f64(agree) / usize_f64(n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tol;

    #[test]
    fn rmse_identical_is_zero() {
        let x = [1.0, 2.0, 3.0];
        assert!((rmse(&x, &x)).abs() < tol::EXACT);
    }

    #[test]
    fn rmse_known_value() {
        let obs = [1.0, 2.0, 3.0];
        let modeled = [1.1, 2.1, 3.1];
        assert!((rmse(&obs, &modeled) - 0.1).abs() < tol::ANALYTICAL);
    }

    #[test]
    fn rmse_empty() {
        let empty: [f64; 0] = [];
        assert!(rmse(&empty, &empty).abs() < tol::EXACT);
    }

    #[test]
    fn mbe_overestimate_positive() {
        let obs = [1.0, 2.0, 3.0];
        let modeled = [1.5, 2.5, 3.5];
        assert!((mbe(&obs, &modeled) - 0.5).abs() < tol::EXACT);
    }

    #[test]
    fn mbe_empty() {
        let empty: [f64; 0] = [];
        assert!(mbe(&empty, &empty).abs() < tol::EXACT);
    }

    #[test]
    fn r2_perfect() {
        let x = [1.0, 2.0, 3.0, 4.0];
        assert!((r_squared(&x, &x) - 1.0).abs() < tol::EXACT);
    }

    #[test]
    fn r2_mean_model_is_zero() {
        let obs = [1.0, 2.0, 3.0];
        let modeled = [2.0, 2.0, 2.0];
        assert!(r_squared(&obs, &modeled).abs() < tol::EXACT);
    }

    #[test]
    fn r2_empty() {
        let empty: [f64; 0] = [];
        assert!(r_squared(&empty, &empty).abs() < tol::EXACT);
    }

    #[test]
    fn r2_constant_observation() {
        let obs = [3.0, 3.0, 3.0];
        let modeled = [3.1, 2.9, 3.0];
        assert!(r_squared(&obs, &modeled).abs() < tol::EXACT);
    }

    #[test]
    fn ia_perfect() {
        let x = [1.0, 2.0, 3.0, 4.0];
        assert!((index_of_agreement(&x, &x) - 1.0).abs() < tol::EXACT);
    }

    #[test]
    fn ia_empty() {
        let empty: [f64; 0] = [];
        assert!(index_of_agreement(&empty, &empty).abs() < tol::EXACT);
    }

    #[test]
    fn ia_constant_denominator_zero() {
        let obs = [5.0, 5.0, 5.0];
        let modeled = [5.0, 5.0, 5.0];
        assert!((index_of_agreement(&obs, &modeled)).abs() < tol::EXACT);
    }

    #[test]
    fn hit_rate_perfect() {
        let obs = [0.0, 5.0, 0.0, 3.0];
        assert!((hit_rate(&obs, &obs, 0.1) - 1.0).abs() < tol::EXACT);
    }

    #[test]
    fn hit_rate_known_value() {
        let obs = [0.0, 5.0, 0.0, 3.0];
        let modeled = [0.0, 4.0, 0.0, 0.0];
        assert!((hit_rate(&obs, &modeled, 0.1) - 0.75).abs() < tol::EXACT);
    }

    #[test]
    fn hit_rate_empty() {
        let empty: [f64; 0] = [];
        assert!(hit_rate(&empty, &empty, 0.1).abs() < tol::EXACT);
    }

    #[test]
    fn mae_identical_is_zero() {
        let x = [1.0, 2.0, 3.0];
        assert!(mae(&x, &x).abs() < tol::EXACT);
    }

    #[test]
    fn mae_known_value() {
        let obs = [1.0, 2.0, 3.0];
        let modeled = [1.5, 2.5, 3.5];
        assert!((mae(&obs, &modeled) - 0.5).abs() < tol::EXACT);
    }

    #[test]
    fn mae_empty() {
        let empty: [f64; 0] = [];
        assert!(mae(&empty, &empty).abs() < tol::EXACT);
    }

    #[test]
    fn nse_perfect() {
        let x = [1.0, 2.0, 3.0, 4.0];
        assert!((nash_sutcliffe(&x, &x) - 1.0).abs() < tol::EXACT);
    }

    #[test]
    fn nse_mean_model_is_zero() {
        let obs = [1.0, 2.0, 3.0];
        let modeled = [2.0, 2.0, 2.0];
        assert!(nash_sutcliffe(&obs, &modeled).abs() < tol::EXACT);
    }

    #[test]
    fn nse_empty() {
        let empty: [f64; 0] = [];
        assert!(nash_sutcliffe(&empty, &empty).abs() < tol::EXACT);
    }

    #[test]
    fn nse_equals_r2_for_same_inputs() {
        let obs = [1.0, 2.0, 3.0, 4.0, 5.0];
        let modeled = [1.1, 2.2, 2.8, 4.3, 4.9];
        let nse = nash_sutcliffe(&obs, &modeled);
        let r2 = r_squared(&obs, &modeled);
        assert!(
            (nse - r2).abs() < tol::ANALYTICAL,
            "NSE should equal R² for the same inputs: nse={nse}, r2={r2}"
        );
    }
}
