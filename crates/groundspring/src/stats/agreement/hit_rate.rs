// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Thresholded occurrence agreement (hit rate) between paired series.

#[cfg(not(feature = "barracuda"))]
use crate::cast::usize_f64;

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
        .filter(|&(&o, &m)| (o > threshold) == (m > threshold))
        .count();
    usize_f64(agree) / usize_f64(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perfect_agreement() {
        let a = [1.0, 0.0, 2.0, 0.0];
        let b = [1.5, 0.0, 3.0, 0.0];
        assert!((hit_rate(&a, &b, 0.5) - 1.0).abs() < crate::tol::EXACT);
    }

    #[test]
    fn no_agreement() {
        let a = [1.0, 1.0];
        let b = [0.0, 0.0];
        assert_eq!(hit_rate(&a, &b, 0.5), 0.0);
    }

    #[test]
    fn empty_slices() {
        assert_eq!(hit_rate(&[], &[], 0.5), 0.0);
    }

    #[test]
    fn half_agreement() {
        let a = [1.0, 1.0, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0, 1.0];
        assert!((hit_rate(&a, &b, 0.5) - 0.5).abs() < crate::tol::EXACT);
    }
}
