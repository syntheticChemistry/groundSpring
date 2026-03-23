// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Willmott's index of agreement (IA, 1981).

#[cfg(not(feature = "barracuda"))]
use crate::cast::usize_f64;

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
