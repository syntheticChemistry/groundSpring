// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Scalar error metrics: RMSE, MAE, and MBE on paired `(observed, modeled)` slices.

#[cfg(not(feature = "barracuda"))]
use crate::cast::usize_f64;

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
