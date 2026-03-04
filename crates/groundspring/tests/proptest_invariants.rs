// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Property-based tests verifying mathematical invariants across the
//! `groundspring` library.  These complement deterministic unit tests by
//! exercising edge cases that manual test vectors cannot cover.

use proptest::prelude::*;

use groundspring::stats::{
    mbe, mean, norm_cdf, norm_ppf, pearson_r, r_squared, rmse, sample_std_dev, std_dev,
};
use groundspring::tol;

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn finite_vec(min_len: usize, max_len: usize) -> impl Strategy<Value = Vec<f64>> {
    proptest::collection::vec(-1e6_f64..1e6, min_len..=max_len)
}

fn positive_vec(min_len: usize, max_len: usize) -> impl Strategy<Value = Vec<f64>> {
    proptest::collection::vec(0.001_f64..1e6, min_len..=max_len)
}

// ---------------------------------------------------------------------------
// mean / std_dev
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn mean_of_constant_is_constant(c in -1e6_f64..1e6, n in 2_usize..100) {
        let v = vec![c; n];
        prop_assert!((mean(&v) - c).abs() < tol::INTEGRATION, "mean({c}) = {}", mean(&v));
    }

    #[test]
    fn std_dev_of_constant_is_zero(c in -1e6_f64..1e6, n in 2_usize..100) {
        let v = vec![c; n];
        let sd = std_dev(&v);
        let tol_val = c.abs().mul_add(tol::EXACT, tol::STRICT);
        prop_assert!(sd < tol_val, "std_dev({c}) = {sd}");
    }

    #[test]
    fn std_dev_is_nonnegative(v in finite_vec(2, 200)) {
        prop_assert!(std_dev(&v) >= 0.0);
    }

    #[test]
    fn sample_std_dev_ge_population(v in finite_vec(3, 200)) {
        prop_assert!(sample_std_dev(&v) >= std_dev(&v) - tol::EXACT);
    }
}

// ---------------------------------------------------------------------------
// RMSE / MBE
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn rmse_identical_is_zero(v in finite_vec(1, 200)) {
        prop_assert!(rmse(&v, &v) < tol::ANALYTICAL);
    }

    #[test]
    fn rmse_is_nonnegative(
        a in finite_vec(2, 100),
    ) {
        let b: Vec<f64> = a.iter().map(|x| x + 1.0).collect();
        prop_assert!(rmse(&a, &b) >= 0.0);
    }

    #[test]
    fn mbe_identical_is_zero(v in finite_vec(1, 200)) {
        prop_assert!(mbe(&v, &v).abs() < tol::ANALYTICAL);
    }

    #[test]
    fn mbe_antisymmetric(
        a in finite_vec(2, 100),
    ) {
        let b: Vec<f64> = a.iter().map(|x| x + 1.0).collect();
        prop_assert!((mbe(&a, &b) + mbe(&b, &a)).abs() < tol::INTEGRATION);
    }
}

// ---------------------------------------------------------------------------
// R² / pearson_r
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn r_squared_perfect_fit(v in positive_vec(3, 100)) {
        let r2 = r_squared(&v, &v);
        prop_assert!((r2 - 1.0).abs() < tol::INTEGRATION, "R² = {r2}");
    }

    #[test]
    fn pearson_perfect_positive(v in positive_vec(3, 100)) {
        let r = pearson_r(&v, &v);
        prop_assert!((r - 1.0).abs() < tol::INTEGRATION, "r = {r}");
    }

    #[test]
    fn pearson_bounded(a in finite_vec(3, 100), b in finite_vec(3, 100)) {
        let len = a.len().min(b.len());
        if len < 3 { return Ok(()); }
        let a = &a[..len];
        let b = &b[..len];
        let r = pearson_r(a, b);
        prop_assert!((-1.0 - tol::INTEGRATION..=1.0 + tol::INTEGRATION).contains(&r), "r = {r}");
    }
}

// ---------------------------------------------------------------------------
// Normal CDF / PPF round-trip
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn cdf_ppf_round_trip(p in 0.01_f64..0.99) {
        let z = norm_ppf(p);
        let recovered = norm_cdf(z);
        prop_assert!(
            (recovered - p).abs() < tol::CDF_APPROX,
            "CDF(PPF({p})) = {recovered}"
        );
    }

    #[test]
    fn cdf_is_monotone(a in -4.0_f64..4.0, b in -4.0_f64..4.0) {
        if a < b {
            prop_assert!(norm_cdf(a) <= norm_cdf(b) + tol::EXACT);
        }
    }

    #[test]
    fn cdf_bounded(z in -10.0_f64..10.0) {
        let p = norm_cdf(z);
        prop_assert!((0.0..=1.0).contains(&p), "CDF({z}) = {p}");
    }
}
