// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Shared coefficient-of-efficiency computation for R² and NSE (paired
//! `(observed, modeled)` slices).  See [`crate::stats::r_squared`] and
//! [`crate::stats::nash_sutcliffe`].

#[cfg(not(feature = "barracuda"))]
use crate::cast::usize_f64;

/// Shared coefficient-of-efficiency computation used by both R² and NSE.
///
/// Computes `1 - SS_res / SS_tot` where `SS_tot` uses the observed mean.
/// R² and NSE are mathematically identical for the (observed, modeled)
/// formulation — they differ only in naming convention across domains.
#[cfg(not(feature = "barracuda"))]
pub(super) fn coefficient_of_efficiency(observed: &[f64], modeled: &[f64]) -> f64 {
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
pub(super) fn coefficient_of_efficiency_gpu(observed: &[f64], modeled: &[f64]) -> Option<f64> {
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

#[cfg(test)]
mod tests {
    #[test]
    fn perfect_model_has_efficiency_one() {
        #[cfg(not(feature = "barracuda"))]
        {
            let obs = [1.0, 2.0, 3.0, 4.0];
            assert!((super::coefficient_of_efficiency(&obs, &obs) - 1.0).abs() < crate::tol::EXACT);
        }
    }

    #[test]
    fn mean_model_has_efficiency_zero() {
        #[cfg(not(feature = "barracuda"))]
        {
            let obs = [1.0, 2.0, 3.0, 4.0];
            let mean_model = [2.5, 2.5, 2.5, 2.5];
            assert!(super::coefficient_of_efficiency(&obs, &mean_model).abs() < crate::tol::EXACT);
        }
    }

    #[test]
    #[expect(
        clippy::float_cmp,
        reason = "exact zero return for empty-slice edge case"
    )]
    fn empty_returns_zero() {
        #[cfg(not(feature = "barracuda"))]
        assert_eq!(super::coefficient_of_efficiency(&[], &[]), 0.0);
    }
}
