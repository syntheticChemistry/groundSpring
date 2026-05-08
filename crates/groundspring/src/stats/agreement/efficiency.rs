// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Nash–Sutcliffe efficiency (NSE) and coefficient of determination (R²).
//!
//! For paired `(observed, modeled)` slices these are mathematically identical;
//! both use the same `1 - SS_res / SS_tot` computation on the non-`barracuda`
//! path (implemented in the internal `coefficient` submodule).

#[cfg(not(feature = "barracuda"))]
use super::coefficient::coefficient_of_efficiency;
#[cfg(feature = "barracuda-gpu")]
use super::coefficient::coefficient_of_efficiency_gpu;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nse_perfect() {
        let obs = [1.0, 2.0, 3.0];
        assert!((nash_sutcliffe(&obs, &obs) - 1.0).abs() < crate::tol::EXACT);
    }

    #[test]
    fn r_squared_perfect() {
        let obs = [10.0, 20.0, 30.0];
        assert!((r_squared(&obs, &obs) - 1.0).abs() < crate::tol::EXACT);
    }

    #[test]
    fn nse_mean_model_is_zero() {
        let obs = [1.0, 2.0, 3.0];
        let mean = [2.0, 2.0, 2.0];
        assert!(nash_sutcliffe(&obs, &mean).abs() < crate::tol::EXACT);
    }

    #[test]
    fn nse_and_r2_are_identical() {
        let obs = [1.0, 3.0, 5.0, 7.0];
        let model = [1.2, 2.8, 5.1, 6.9];
        let nse = nash_sutcliffe(&obs, &model);
        let r2 = r_squared(&obs, &model);
        assert!((nse - r2).abs() < crate::tol::EXACT);
    }
}
