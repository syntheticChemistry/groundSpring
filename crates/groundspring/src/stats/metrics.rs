// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Descriptive statistics: mean, standard deviation, percentile.
//!
//! Single-distribution functions that operate on one `&[f64]` slice.
//! For paired-observation agreement metrics (RMSE, MBE, R², etc.),
//! see the sibling [`super::agreement`] module.

#[cfg(not(feature = "barracuda"))]
use crate::cast::f64_usize;
use crate::cast::usize_f64;

/// Arithmetic mean of a slice.
///
/// When `barracuda-gpu` is enabled and a GPU is available, dispatches to
/// `SumReduceF64::mean`. Otherwise delegates to `barracuda::stats::mean`
/// (CPU) or the local implementation.
/// Returns `0.0` for empty slices.
#[must_use]
pub fn mean(values: &[f64]) -> f64 {
    #[cfg(feature = "barracuda-gpu")]
    {
        if let Some(m) = mean_gpu(values) {
            return m;
        }
    }
    #[cfg(feature = "barracuda")]
    {
        barracuda::stats::mean(values)
    }
    #[cfg(not(feature = "barracuda"))]
    mean_cpu(values)
}

#[cfg(feature = "barracuda-gpu")]
fn mean_gpu(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return Some(0.0);
    }
    let device = crate::gpu::get_device()?;
    barracuda::ops::sum_reduce_f64::SumReduceF64::mean(device, values).ok()
}

#[cfg(not(feature = "barracuda"))]
fn mean_cpu(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / usize_f64(values.len())
}

/// Population standard deviation (divides by N).
///
/// groundSpring uses population variance for total-population metrics like
/// RMSE decomposition.  For sample-based estimates, use [`sample_std_dev`].
/// When `barracuda-gpu` is enabled, dispatches to `VarianceReduceF64`.
#[must_use]
pub fn std_dev(values: &[f64]) -> f64 {
    #[cfg(feature = "barracuda-gpu")]
    {
        if let Some(s) = std_dev_gpu(values) {
            return s;
        }
    }
    std_dev_cpu(values)
}

fn std_dev_cpu(values: &[f64]) -> f64 {
    let n = values.len();
    if n == 0 {
        return 0.0;
    }
    let m = mean(values);
    let variance = values.iter().map(|v| (v - m).powi(2)).sum::<f64>() / usize_f64(n);
    variance.sqrt()
}

#[cfg(feature = "barracuda-gpu")]
fn std_dev_gpu(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return Some(0.0);
    }
    let device = crate::gpu::get_device()?;
    barracuda::ops::variance_reduce_f64::VarianceReduceF64::population_std(device, values).ok()
}

/// Sample standard deviation (Bessel-corrected, divides by N−1).
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::stats::correlation::std_dev`, falling back to the local
/// implementation on error.
/// Returns `0.0` for slices with fewer than 2 elements.
#[must_use]
pub fn sample_std_dev(values: &[f64]) -> f64 {
    #[cfg(feature = "barracuda")]
    {
        if let Ok(s) = barracuda::stats::correlation::std_dev(values) {
            return s;
        }
    }
    sample_std_dev_cpu(values)
}

fn sample_std_dev_cpu(values: &[f64]) -> f64 {
    let n = values.len();
    if n < 2 {
        return 0.0;
    }
    let m = mean(values);
    let variance = values.iter().map(|v| (v - m).powi(2)).sum::<f64>() / usize_f64(n - 1);
    variance.sqrt()
}

/// Percentile of a sorted copy of `values` (0–100 scale).
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::stats::percentile`.
///
/// # Errors
///
/// Returns [`crate::error::InputError::OutOfRange`] if `p` is not in `[0.0, 100.0]`.
pub fn percentile(values: &[f64], p: f64) -> Result<f64, crate::error::InputError> {
    if !(0.0..=100.0).contains(&p) {
        return Err(crate::error::InputError::OutOfRange {
            name: "p",
            lo: 0.0,
            hi: 100.0,
            got: p,
        });
    }
    #[cfg(feature = "barracuda")]
    {
        Ok(barracuda::stats::percentile(values, p))
    }
    #[cfg(not(feature = "barracuda"))]
    Ok(percentile_cpu(values, p))
}

#[cfg(not(feature = "barracuda"))]
fn percentile_cpu(values: &[f64], p: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted: Vec<f64> = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let rank = p / 100.0 * usize_f64(sorted.len() - 1);
    let lo = f64_usize(rank.floor());
    let hi = f64_usize(rank.ceil());
    if lo == hi {
        sorted[lo]
    } else {
        let frac = rank - usize_f64(lo);
        sorted[lo].mul_add(1.0 - frac, sorted[hi] * frac)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tol;

    #[test]
    fn mean_empty() {
        let empty: [f64; 0] = [];
        assert!(mean(&empty).abs() < tol::EXACT);
    }

    #[test]
    fn std_dev_empty() {
        let empty: [f64; 0] = [];
        assert!(std_dev(&empty).abs() < tol::EXACT);
    }

    #[test]
    fn std_dev_constant() {
        let vals = [4.0, 4.0, 4.0];
        assert!(std_dev(&vals).abs() < tol::EXACT);
    }

    #[test]
    fn std_dev_known_value() {
        let vals = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let s = std_dev(&vals);
        assert!(
            (s - 2.0).abs() < tol::EXACT,
            "population σ should be 2.0, got {s}"
        );
    }

    #[test]
    fn percentile_median() {
        let vals = [1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((percentile(&vals, 50.0).unwrap() - 3.0).abs() < tol::EXACT);
    }

    #[test]
    fn percentile_empty() {
        let empty: [f64; 0] = [];
        assert!(percentile(&empty, 50.0).unwrap().abs() < tol::EXACT);
    }

    #[test]
    fn percentile_interpolation() {
        let vals = [1.0, 2.0, 3.0, 4.0];
        let p25 = percentile(&vals, 25.0).unwrap();
        assert!(
            (p25 - 1.75).abs() < tol::EXACT,
            "P25 of [1,2,3,4] = 1.75, got {p25}"
        );
    }

    #[test]
    fn percentile_out_of_range() {
        assert!(percentile(&[1.0], -1.0).is_err());
        assert!(percentile(&[1.0], 101.0).is_err());
    }

    #[test]
    fn sample_std_dev_bessel_correction() {
        let vals = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let pop = std_dev(&vals);
        let samp = sample_std_dev(&vals);
        assert!(samp > pop, "sample std > population std");
        assert!(
            (samp - 2.138).abs() < tol::STOCHASTIC,
            "known sample σ ≈ 2.138, got {samp}"
        );
    }

    #[test]
    fn sample_std_dev_single_element() {
        assert!(sample_std_dev(&[42.0]).abs() < tol::EXACT);
    }

    #[test]
    fn sample_std_dev_empty() {
        let empty: [f64; 0] = [];
        assert!(sample_std_dev(&empty).abs() < tol::EXACT);
    }
}
