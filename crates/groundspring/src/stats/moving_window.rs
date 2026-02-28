// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Sliding window descriptive statistics (f64 precision).
//!
//! Provides moving-window mean, variance, min, and max over `&[f64]` data.
//! Used for sensor signal smoothing (Exp 001), temporal drift detection
//! (Exp 002), and precipitation aggregation (Exp 003).
//!
//! # barracuda delegation
//!
//! When the `barracuda` feature is enabled, delegates to
//! `barracuda::stats::moving_window_stats_f64` (absorbed from airSpring
//! in `ToadStool` S66).  The barracuda implementation uses the same
//! two-pass algorithm (exact, not streaming) — identical results.

/// Result of a moving-window statistics computation.
#[derive(Debug, Clone)]
pub struct MovingWindowResult {
    /// Sliding window means (length = `data.len()` − window + 1).
    pub mean: Vec<f64>,
    /// Sliding window population variance.
    pub variance: Vec<f64>,
    /// Sliding window minimum.
    pub min: Vec<f64>,
    /// Sliding window maximum.
    pub max: Vec<f64>,
}

/// Compute moving window statistics over `data` with the given `window_size`.
///
/// Returns `None` if `data.len() < window_size` or `window_size == 0`.
/// Output vectors have length `data.len() − window_size + 1`.
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::stats::moving_window_stats_f64`.
#[must_use]
pub fn moving_window_stats(data: &[f64], window_size: usize) -> Option<MovingWindowResult> {
    if data.len() < window_size || window_size == 0 {
        return None;
    }

    #[cfg(feature = "barracuda")]
    {
        if let Some(r) = barracuda::stats::moving_window_stats_f64(data, window_size) {
            return Some(MovingWindowResult {
                mean: r.mean,
                variance: r.variance,
                min: r.min,
                max: r.max,
            });
        }
    }

    Some(moving_window_stats_cpu(data, window_size))
}

fn moving_window_stats_cpu(data: &[f64], window_size: usize) -> MovingWindowResult {
    let out_len = data.len() - window_size + 1;
    let wf = crate::cast::usize_f64(window_size);
    let mut mean = Vec::with_capacity(out_len);
    let mut variance = Vec::with_capacity(out_len);
    let mut min_vals = Vec::with_capacity(out_len);
    let mut max_vals = Vec::with_capacity(out_len);

    for i in 0..out_len {
        let window = &data[i..i + window_size];
        let sum: f64 = window.iter().sum();
        let m = sum / wf;
        let var = window.iter().map(|&x| (x - m).powi(2)).sum::<f64>() / wf;
        let wmin = window.iter().copied().fold(f64::INFINITY, f64::min);
        let wmax = window.iter().copied().fold(f64::NEG_INFINITY, f64::max);

        mean.push(m);
        variance.push(var);
        min_vals.push(wmin);
        max_vals.push(wmax);
    }

    MovingWindowResult {
        mean,
        variance,
        min: min_vals,
        max: max_vals,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_data() {
        let data = vec![5.0; 10];
        let r = moving_window_stats(&data, 3).unwrap();
        assert_eq!(r.mean.len(), 8);
        for &m in &r.mean {
            assert!((m - 5.0).abs() < f64::EPSILON);
        }
        for &v in &r.variance {
            assert!(v.abs() < f64::EPSILON);
        }
    }

    #[test]
    fn linear_ramp() {
        let data: Vec<f64> = (1..=5).map(f64::from).collect();
        let r = moving_window_stats(&data, 3).unwrap();
        assert_eq!(r.mean.len(), 3);
        assert!((r.mean[0] - 2.0).abs() < 1e-10);
        assert!((r.mean[1] - 3.0).abs() < 1e-10);
        assert!((r.mean[2] - 4.0).abs() < 1e-10);
    }

    #[test]
    fn min_max() {
        let data = vec![3.0, 1.0, 4.0, 1.0, 5.0];
        let r = moving_window_stats(&data, 3).unwrap();
        assert!((r.min[0] - 1.0).abs() < f64::EPSILON);
        assert!((r.max[0] - 4.0).abs() < f64::EPSILON);
        assert!((r.min[2] - 1.0).abs() < f64::EPSILON);
        assert!((r.max[2] - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn window_equals_data() {
        let data = vec![2.0, 4.0, 6.0];
        let r = moving_window_stats(&data, 3).unwrap();
        assert_eq!(r.mean.len(), 1);
        assert!((r.mean[0] - 4.0).abs() < 1e-10);
    }

    #[test]
    fn returns_none_for_too_small() {
        let data = vec![1.0, 2.0];
        assert!(moving_window_stats(&data, 5).is_none());
    }

    #[test]
    fn returns_none_for_zero_window() {
        let data = vec![1.0, 2.0, 3.0];
        assert!(moving_window_stats(&data, 0).is_none());
    }
}
