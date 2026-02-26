// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Hill-function kinetics shared by bistable and multi-signal ODE models.
//!
//! The Hill function `x^n / (K^n + x^n)` is the standard sigmoidal response
//! used throughout enzyme kinetics and gene regulation. Both the bistable
//! (Exp 010) and multi-signal (Exp 011) ODE systems need it for DGC/PDE
//! regulation, biofilm formation, and QS signal transduction.
//!
//! # barracuda delegation
//!
//! When the `barracuda` feature is enabled, `hill` and `hill_repress` delegate
//! to `barracuda::stats::hill` and `barracuda::stats::monod` (the repression
//! form is `1 - hill`).

/// Activating Hill function: `x^n / (K^n + x^n)`.
///
/// Returns 0 for non-positive `x`, saturates to 1 as `x → ∞`.
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::stats::hill`.
#[must_use]
#[inline]
pub fn hill(x: f64, k: f64, n: f64) -> f64 {
    #[cfg(feature = "barracuda")]
    {
        if let Ok(v) = barracuda::stats::hill(x, k, n) {
            return v;
        }
    }
    hill_cpu(x, k, n)
}

fn hill_cpu(x: f64, k: f64, n: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    let xn = x.powf(n);
    xn / (k.powf(n) + xn)
}

/// Repressing Hill function: `K^n / (K^n + x^n)`.
///
/// Returns 1 for non-positive `x`, decays to 0 as `x → ∞`.
/// This is `1 - hill(x, k, n)`.
#[must_use]
#[inline]
pub fn hill_repress(x: f64, k: f64, n: f64) -> f64 {
    if x <= 0.0 {
        return 1.0;
    }
    let kn = k.powf(n);
    kn / (kn + x.powf(n))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hill_zero_input() {
        assert!(hill(0.0, 1.0, 2.0).abs() < f64::EPSILON);
        assert!(hill(-1.0, 1.0, 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn hill_at_half_saturation() {
        let v = hill(1.0, 1.0, 2.0);
        assert!(
            (v - 0.5).abs() < 1e-12,
            "hill(K, K, n) should be 0.5, got {v}"
        );
    }

    #[test]
    fn hill_saturation() {
        let v = hill(1000.0, 1.0, 2.0);
        assert!(v > 0.999, "hill should saturate near 1, got {v}");
    }

    #[test]
    fn hill_repress_zero_input() {
        assert!((hill_repress(0.0, 1.0, 2.0) - 1.0).abs() < f64::EPSILON);
        assert!((hill_repress(-1.0, 1.0, 2.0) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn hill_repress_at_half_saturation() {
        let v = hill_repress(1.0, 1.0, 2.0);
        assert!(
            (v - 0.5).abs() < 1e-12,
            "hill_repress(K, K, n) should be 0.5, got {v}"
        );
    }

    #[test]
    fn hill_repress_complement() {
        let x = 2.5;
        let k = 1.0;
        let n = 3.0;
        let sum = hill(x, k, n) + hill_repress(x, k, n);
        assert!(
            (sum - 1.0).abs() < 1e-12,
            "hill + hill_repress should be 1, got {sum}"
        );
    }
}
