// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Probability distribution functions and goodness-of-fit statistics.
//!
//! - Normal CDF Φ(x) via Abramowitz & Stegun 7.1.26 (max |ε| < 1.5×10⁻⁷)
//! - Inverse normal CDF Φ⁻¹(p) via Acklam rational approximation
//! - Chi-squared statistic for goodness-of-fit testing
//!
//! # barracuda delegation
//!
//! When the `barracuda` feature is enabled each function delegates to the
//! corresponding GPU-ready implementation.  CPU implementations are always
//! compiled and serve as the fallback.

/// Standard normal cumulative distribution function Φ(x).
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::stats::norm_cdf`.
/// Uses the Abramowitz & Stegun rational approximation (7.1.26).
#[must_use]
pub fn norm_cdf(x: f64) -> f64 {
    #[cfg(feature = "barracuda")]
    {
        barracuda::stats::norm_cdf(x)
    }
    #[cfg(not(feature = "barracuda"))]
    norm_cdf_cpu(x)
}

#[cfg(not(feature = "barracuda"))]
fn norm_cdf_cpu(x: f64) -> f64 {
    0.5_f64.mul_add(erf_cpu(x / std::f64::consts::SQRT_2), 0.5)
}

/// Error function via Abramowitz & Stegun 7.1.26.  Max |ε| < 1.5×10⁻⁷.
#[cfg(not(feature = "barracuda"))]
fn erf_cpu(x: f64) -> f64 {
    let sign = x.signum();
    let a = x.abs();
    let t = 1.0 / a.mul_add(0.327_591_1, 1.0);
    let erfc = t
        * (0.254_829_592
            + t * (-0.284_496_736
                + t * (1.421_413_741 + t * (-1.453_152_027 + t * 1.061_405_429))))
        * (-a * a).exp();
    sign * (1.0 - erfc)
}

/// Inverse standard normal CDF (quantile function) Φ⁻¹(p).
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::stats::norm_ppf`.
/// Uses the Beasley-Springer-Moro algorithm.
///
/// # Panics
///
/// Panics if `p` is not in (0, 1).
#[must_use]
pub fn norm_ppf(p: f64) -> f64 {
    assert!(p > 0.0 && p < 1.0, "norm_ppf requires p ∈ (0, 1), got {p}");
    #[cfg(feature = "barracuda")]
    {
        barracuda::stats::norm_ppf(p)
    }
    #[cfg(not(feature = "barracuda"))]
    norm_ppf_cpu(p)
}

/// Acklam rational approximation — relative error < 1.15×10⁻⁹.
#[cfg(not(feature = "barracuda"))]
#[expect(
    clippy::suboptimal_flops,
    clippy::excessive_precision,
    reason = "Acklam rational approximation coefficients require full precision"
)]
fn norm_ppf_cpu(p: f64) -> f64 {
    const A: [f64; 6] = [
        -3.969_683_028_665_376e1,
        2.209_460_984_245_205e2,
        -2.759_285_104_469_687e2,
        1.383_577_518_672_690e2,
        -3.066_479_806_614_716e1,
        2.506_628_277_459_239e0,
    ];
    const B: [f64; 5] = [
        -5.447_609_879_822_406e1,
        1.615_858_368_580_409e2,
        -1.556_989_798_598_866e2,
        6.680_131_188_771_972e1,
        -1.328_068_155_288_572e1,
    ];
    const C: [f64; 6] = [
        -7.784_894_002_430_293e-3,
        -3.223_964_580_411_365e-1,
        -2.400_758_277_161_838e0,
        -2.549_732_539_343_734e0,
        4.374_664_141_464_968e0,
        2.938_163_982_698_783e0,
    ];
    const D: [f64; 4] = [
        7.784_695_709_041_462e-3,
        3.224_671_290_700_398e-1,
        2.445_134_137_142_996e0,
        3.754_408_661_907_416e0,
    ];

    const P_LOW: f64 = 0.024_25;
    const P_HIGH: f64 = 1.0 - P_LOW;

    if p < P_LOW {
        let q = (-2.0 * p.ln()).sqrt();
        (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    } else if p <= P_HIGH {
        let q = p - 0.5;
        let r = q * q;
        (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5]) * q
            / (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0)
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    }
}

/// Chi-squared goodness-of-fit statistic: Σ (O−E)² / E.
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::stats::chi2_decomposed` and returns the total statistic.
/// Returns `0.0` for empty slices.
///
/// # Panics
///
/// Panics if `observed` and `expected` have different lengths.
#[must_use]
pub fn chi2_statistic(observed: &[f64], expected: &[f64]) -> f64 {
    assert_eq!(
        observed.len(),
        expected.len(),
        "observed and expected must have equal length"
    );
    #[cfg(feature = "barracuda")]
    if let Ok(r) = barracuda::stats::chi2_decomposed(observed, expected, 0) {
        return r.chi2_total;
    }
    chi2_statistic_cpu(observed, expected)
}

fn chi2_statistic_cpu(observed: &[f64], expected: &[f64]) -> f64 {
    if observed.is_empty() {
        return 0.0;
    }
    observed
        .iter()
        .zip(expected)
        .filter(|(_, e)| **e != 0.0)
        .map(|(o, e)| (*o - *e).powi(2) / *e)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tol;

    // Tolerance key (crate::tol):
    //   EXACT       — exact arithmetic identity
    //   ANALYTICAL  — known analytical value through one rational expression
    //   CDF_APPROX  — erf composition through CDF (A&S 7.1.26, two layers)
    //   ROUNDTRIP   — CDF↔PPF round-trip (both approximations compound)
    //   LITERATURE  — Φ(1.96) = 0.975 checked to 3 decimal places
    //   STOCHASTIC  — PPF known values checked to 2 decimal places

    #[test]
    fn norm_cdf_symmetry() {
        assert!((norm_cdf(0.0) - 0.5).abs() < tol::CDF_APPROX);
    }

    #[test]
    fn norm_cdf_known_values() {
        assert!((norm_cdf(1.0) - 0.841_344_746_068_543).abs() < tol::CDF_APPROX);
        assert!((norm_cdf(-1.0) - 0.158_655_253_931_457).abs() < tol::CDF_APPROX);
        assert!((norm_cdf(1.96) - 0.975).abs() < tol::LITERATURE);
    }

    #[test]
    fn norm_cdf_complement() {
        let x = 2.0;
        assert!((norm_cdf(x) + norm_cdf(-x) - 1.0).abs() < tol::CDF_APPROX);
    }

    #[test]
    fn norm_ppf_known_values() {
        assert!((norm_ppf(0.5)).abs() < tol::CDF_APPROX);
        assert!((norm_ppf(0.975) - 1.96).abs() < tol::STOCHASTIC);
        assert!((norm_ppf(0.025) + 1.96).abs() < tol::STOCHASTIC);
    }

    #[test]
    fn norm_ppf_cdf_roundtrip() {
        for &p in &[0.01, 0.1, 0.25, 0.5, 0.75, 0.9, 0.99] {
            let x = norm_ppf(p);
            let p_back = norm_cdf(x);
            assert!(
                (p - p_back).abs() < tol::ROUNDTRIP,
                "roundtrip: norm_cdf(norm_ppf({p})) = {p_back}"
            );
        }
    }

    #[test]
    fn chi2_statistic_perfect_fit() {
        let obs = [10.0, 20.0, 30.0];
        assert!(chi2_statistic(&obs, &obs).abs() < tol::EXACT);
    }

    #[test]
    fn chi2_statistic_known_value() {
        let obs = [10.0, 20.0, 30.0];
        let exp = [15.0, 15.0, 30.0];
        let chi2 = chi2_statistic(&obs, &exp);
        // (10-15)²/15 + (20-15)²/15 + (30-30)²/30 = 25/15 + 25/15 = 10/3
        assert!(
            (chi2 - 10.0 / 3.0).abs() < tol::ANALYTICAL,
            "expected 10/3, got {chi2}"
        );
    }

    #[test]
    fn chi2_statistic_empty() {
        let empty: [f64; 0] = [];
        assert!(chi2_statistic(&empty, &empty).abs() < tol::EXACT);
    }
}
