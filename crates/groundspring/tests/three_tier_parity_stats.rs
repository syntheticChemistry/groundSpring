// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Three-tier parity tests — statistics primitives.
//!
//! Validates that stats, agreement, correlation, regression, bootstrap,
//! and moving-window functions produce identical results regardless of
//! feature mode (default / barracuda / barracuda-gpu).

use groundspring::tol;

// ── stats::metrics ────────────────────────────────────────────────

#[test]
fn mean_parity_known_value() {
    let vals = [1.0, 2.0, 3.0, 4.0, 5.0];
    let m1 = groundspring::stats::mean(&vals);
    let m2 = groundspring::stats::mean(&vals);
    assert_eq!(m1.to_bits(), m2.to_bits(), "mean bitwise");
    assert!((m1 - 3.0).abs() < tol::DETERMINISM, "mean = 3.0: {m1}");
}

#[test]
fn percentile_parity_known_value() {
    let vals = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let p50_1 = groundspring::stats::percentile(&vals, 50.0).unwrap();
    let p50_2 = groundspring::stats::percentile(&vals, 50.0).unwrap();
    assert_eq!(p50_1.to_bits(), p50_2.to_bits(), "p50 bitwise");
    assert!((p50_1 - 5.5).abs() < 1.0, "median near 5.5: {p50_1}");
}

#[test]
fn sample_std_dev_parity_known_value() {
    let vals = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    let s1 = groundspring::stats::sample_std_dev(&vals);
    let s2 = groundspring::stats::sample_std_dev(&vals);
    assert_eq!(s1.to_bits(), s2.to_bits(), "sample_std_dev bitwise");
    assert!(s1 > 0.0, "positive std dev");
}

// ── stats::agreement ──────────────────────────────────────────────

#[test]
fn rmse_parity_known_value() {
    let obs = [1.0, 2.0, 3.0, 4.0, 5.0];
    let mod_ = [1.0, 2.0, 3.0, 4.0, 5.0];
    let r = groundspring::stats::rmse(&obs, &mod_);
    assert!(
        (r - 0.0).abs() < tol::DETERMINISM,
        "perfect fit RMSE = 0: {r}"
    );
    let r2 = groundspring::stats::rmse(&obs, &mod_);
    assert_eq!(r.to_bits(), r2.to_bits(), "rmse bitwise");
}

#[test]
fn mbe_parity_known_value() {
    let obs = [1.0, 2.0, 3.0];
    let mod_ = [2.0, 3.0, 4.0];
    let b1 = groundspring::stats::mbe(&obs, &mod_);
    let b2 = groundspring::stats::mbe(&obs, &mod_);
    assert_eq!(b1.to_bits(), b2.to_bits(), "mbe bitwise");
    assert!(
        (b1 - 1.0).abs() < tol::DETERMINISM,
        "constant +1 bias: {b1}"
    );
}

#[test]
fn r_squared_parity_known_value() {
    let obs = [1.0, 2.0, 3.0, 4.0, 5.0];
    let r2v = groundspring::stats::r_squared(&obs, &obs);
    assert!(
        (r2v - 1.0).abs() < tol::ANALYTICAL,
        "perfect R² = 1.0: {r2v}"
    );
    let r2v2 = groundspring::stats::r_squared(&obs, &obs);
    assert_eq!(r2v.to_bits(), r2v2.to_bits(), "r_squared bitwise");
}

#[test]
fn index_of_agreement_parity_known_value() {
    let obs = [1.0, 2.0, 3.0, 4.0, 5.0];
    let ia = groundspring::stats::index_of_agreement(&obs, &obs);
    assert!((ia - 1.0).abs() < tol::ANALYTICAL, "perfect IA = 1.0: {ia}");
    let ia2 = groundspring::stats::index_of_agreement(&obs, &obs);
    assert_eq!(ia.to_bits(), ia2.to_bits(), "ia bitwise");
}

#[test]
fn hit_rate_parity_known_value() {
    let obs = [10.0, 20.0, 30.0];
    let mod_ = [10.0, 20.0, 30.0];
    let hr = groundspring::stats::hit_rate(&obs, &mod_, 5.0);
    assert!(
        (hr - 1.0).abs() < tol::DETERMINISM,
        "perfect hit rate = 1.0: {hr}"
    );
    let hr2 = groundspring::stats::hit_rate(&obs, &mod_, 5.0);
    assert_eq!(hr.to_bits(), hr2.to_bits(), "hit_rate bitwise");
}

#[test]
fn mae_parity_known_value() {
    let obs = [1.0, 2.0, 3.0, 4.0, 5.0];
    let modeled = [1.2, 2.3, 2.8, 4.1, 5.4];
    let mae1 = groundspring::stats::mae(&obs, &modeled);
    let mae2 = groundspring::stats::mae(&obs, &modeled);
    assert_eq!(mae1.to_bits(), mae2.to_bits(), "MAE bitwise parity");
    assert!(mae1 > 0.0 && mae1 < 1.0, "MAE in expected range: {mae1}");
}

#[test]
fn nse_parity_known_value() {
    let obs = [1.0, 2.0, 3.0, 4.0, 5.0];
    let modeled = [1.1, 2.2, 2.8, 4.3, 4.9];
    let nse1 = groundspring::stats::nash_sutcliffe(&obs, &modeled);
    let nse2 = groundspring::stats::nash_sutcliffe(&obs, &modeled);
    assert_eq!(nse1.to_bits(), nse2.to_bits(), "NSE bitwise parity");
    assert!(nse1 > 0.9 && nse1 <= 1.0, "NSE near-perfect: {nse1}");
}

// ── stats::correlation ────────────────────────────────────────────

#[test]
fn pearson_r_parity_known_value() {
    let x = [1.0, 2.0, 3.0, 4.0, 5.0];
    let y = [2.0, 4.0, 6.0, 8.0, 10.0];
    let r1 = groundspring::stats::pearson_r(&x, &y);
    let r2 = groundspring::stats::pearson_r(&x, &y);
    assert_eq!(r1.to_bits(), r2.to_bits(), "pearson bitwise");
    assert!(
        (r1 - 1.0).abs() < tol::ANALYTICAL,
        "perfect linear r = 1.0: {r1}"
    );
}

#[test]
fn spearman_r_parity_known_value() {
    let x = [1.0, 2.0, 3.0, 4.0, 5.0];
    let y = [2.0, 4.0, 6.0, 8.0, 10.0];
    let r1 = groundspring::stats::spearman_r(&x, &y);
    let r2 = groundspring::stats::spearman_r(&x, &y);
    assert_eq!(r1.to_bits(), r2.to_bits(), "spearman bitwise");
    assert!(
        (r1 - 1.0).abs() < tol::ANALYTICAL,
        "perfect monotonic rs = 1.0: {r1}"
    );
}

#[test]
fn covariance_parity_known_value() {
    let x = [1.0, 2.0, 3.0, 4.0, 5.0];
    let y = [2.0, 4.0, 6.0, 8.0, 10.0];
    let c1 = groundspring::stats::covariance(&x, &y);
    let c2 = groundspring::stats::covariance(&x, &y);
    assert_eq!(c1.to_bits(), c2.to_bits(), "covariance bitwise");
    assert!(
        c1 > 0.0,
        "positive covariance for positively correlated data: {c1}"
    );
}

// ── stats::regression ─────────────────────────────────────────────

#[test]
fn regression_linear_parity() {
    let xs: Vec<f64> = (0..10).map(f64::from).collect();
    let ys: Vec<f64> = xs.iter().map(|&x| 2.0_f64.mul_add(x, 1.0)).collect();
    let f1 = groundspring::stats::fit_linear(&xs, &ys).unwrap();
    let f2 = groundspring::stats::fit_linear(&xs, &ys).unwrap();
    assert_eq!(f1.slope.to_bits(), f2.slope.to_bits(), "slope bitwise");
    assert_eq!(
        f1.intercept.to_bits(),
        f2.intercept.to_bits(),
        "intercept bitwise"
    );
    assert!(
        (f1.slope - 2.0).abs() < tol::ANALYTICAL,
        "slope = 2.0: {}",
        f1.slope
    );
    assert!(
        (f1.intercept - 1.0).abs() < tol::ANALYTICAL,
        "intercept = 1.0: {}",
        f1.intercept
    );
    assert!(f1.r_squared > 0.999, "R² perfect: {}", f1.r_squared);
}

#[test]
fn regression_quadratic_parity() {
    let xs: Vec<f64> = (-5..=5).map(f64::from).collect();
    let ys: Vec<f64> = xs
        .iter()
        .map(|&x| (2.0 * x).mul_add(x, (-3.0_f64).mul_add(x, 1.0)))
        .collect();
    let f1 = groundspring::stats::fit_quadratic(&xs, &ys).unwrap();
    let f2 = groundspring::stats::fit_quadratic(&xs, &ys).unwrap();
    assert_eq!(f1.params[0].to_bits(), f2.params[0].to_bits(), "a parity");
    assert_eq!(f1.params[1].to_bits(), f2.params[1].to_bits(), "b parity");
    assert_eq!(f1.params[2].to_bits(), f2.params[2].to_bits(), "c parity");
    assert!(
        (f1.params[0] - 2.0).abs() < tol::INTEGRATION,
        "a = {}",
        f1.params[0]
    );
    assert!(f1.r_squared > 0.999, "R² = {}", f1.r_squared);
}

#[test]
fn regression_exponential_parity() {
    let xs: Vec<f64> = (0..10).map(f64::from).collect();
    let a = 5.0_f64;
    let b = -0.3_f64;
    let ys: Vec<f64> = xs.iter().map(|&x| a * (b * x).exp()).collect();
    let f1 = groundspring::stats::fit_exponential(&xs, &ys).unwrap();
    let f2 = groundspring::stats::fit_exponential(&xs, &ys).unwrap();
    assert_eq!(f1.params[0].to_bits(), f2.params[0].to_bits(), "a parity");
    assert_eq!(f1.params[1].to_bits(), f2.params[1].to_bits(), "b parity");
    assert!(f1.r_squared > 0.99, "R² = {}", f1.r_squared);
}

#[test]
fn regression_logarithmic_parity() {
    let xs: Vec<f64> = (1..=10).map(f64::from).collect();
    let a = 3.0_f64;
    let b = 2.0_f64;
    let ys: Vec<f64> = xs.iter().map(|&x| a.mul_add(x.ln(), b)).collect();
    let f1 = groundspring::stats::fit_logarithmic(&xs, &ys).unwrap();
    let f2 = groundspring::stats::fit_logarithmic(&xs, &ys).unwrap();
    assert_eq!(f1.params[0].to_bits(), f2.params[0].to_bits(), "a parity");
    assert_eq!(f1.params[1].to_bits(), f2.params[1].to_bits(), "b parity");
    assert!(
        (f1.params[0] - a).abs() < tol::INTEGRATION,
        "a = {}",
        f1.params[0]
    );
    assert!(
        (f1.params[1] - b).abs() < tol::INTEGRATION,
        "b = {}",
        f1.params[1]
    );
}

// ── bootstrap ─────────────────────────────────────────────────────

#[test]
fn bootstrap_mean_parity_deterministic() {
    let data = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let r1 = groundspring::bootstrap::bootstrap_mean(&data, 1000, 0.95, 42);
    let r2 = groundspring::bootstrap::bootstrap_mean(&data, 1000, 0.95, 42);
    assert_eq!(r1.estimate.to_bits(), r2.estimate.to_bits(), "mean bitwise");
    assert_eq!(
        r1.ci_lower.to_bits(),
        r2.ci_lower.to_bits(),
        "ci_lower bitwise"
    );
    assert_eq!(
        r1.ci_upper.to_bits(),
        r2.ci_upper.to_bits(),
        "ci_upper bitwise"
    );
    assert!(
        (r1.estimate - 5.5).abs() < 1.0,
        "mean near 5.5: {}",
        r1.estimate
    );
}

#[test]
fn bootstrap_mean_parity_ci_contains_true() {
    let data = [2.0, 4.0, 6.0, 8.0, 10.0];
    let r = groundspring::bootstrap::bootstrap_mean(&data, 2000, 0.95, 99);
    assert!(
        r.ci_lower <= 6.0 && r.ci_upper >= 6.0,
        "95% CI should contain true mean 6.0"
    );
}

#[test]
fn rawr_mean_parity_deterministic() {
    let data = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let r1 = groundspring::bootstrap::rawr_mean(&data, 1000, 0.95, 42);
    let r2 = groundspring::bootstrap::rawr_mean(&data, 1000, 0.95, 42);
    assert_eq!(r1.estimate.to_bits(), r2.estimate.to_bits(), "rawr bitwise");
    assert_eq!(
        r1.std_error.to_bits(),
        r2.std_error.to_bits(),
        "rawr se bitwise"
    );
}

#[test]
fn bootstrap_median_parity_deterministic() {
    let data = [1.0, 3.0, 5.0, 7.0, 9.0, 11.0, 13.0];
    let r1 = groundspring::bootstrap::bootstrap_median(&data, 1000, 0.95, 7);
    let r2 = groundspring::bootstrap::bootstrap_median(&data, 1000, 0.95, 7);
    assert_eq!(
        r1.estimate.to_bits(),
        r2.estimate.to_bits(),
        "median bitwise"
    );
    assert!(
        (r1.estimate - 7.0).abs() < 2.0,
        "median near 7.0: {}",
        r1.estimate
    );
}

#[test]
fn bootstrap_std_parity_deterministic() {
    let data = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    let r1 = groundspring::bootstrap::bootstrap_std(&data, 1000, 0.95, 13);
    let r2 = groundspring::bootstrap::bootstrap_std(&data, 1000, 0.95, 13);
    assert_eq!(r1.estimate.to_bits(), r2.estimate.to_bits(), "std bitwise");
    assert!(r1.estimate > 0.0, "std positive");
}

// ── stats::moving_window ──────────────────────────────────────────

#[test]
fn moving_window_stats_parity_deterministic() {
    let data = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let r1 = groundspring::stats::moving_window_stats(&data, 3).unwrap();
    let r2 = groundspring::stats::moving_window_stats(&data, 3).unwrap();
    assert_eq!(r1.mean.len(), r2.mean.len(), "length parity");
    for (a, b) in r1.mean.iter().zip(r2.mean.iter()) {
        assert_eq!(a.to_bits(), b.to_bits(), "mean bitwise");
    }
    for (a, b) in r1.variance.iter().zip(r2.variance.iter()) {
        assert_eq!(a.to_bits(), b.to_bits(), "variance bitwise");
    }
}

#[test]
fn moving_window_stats_parity_known_value() {
    let data = [1.0, 2.0, 3.0];
    let r = groundspring::stats::moving_window_stats(&data, 3).unwrap();
    assert!(
        (r.mean[0] - 2.0).abs() < tol::ANALYTICAL,
        "window mean = 2.0"
    );
}
