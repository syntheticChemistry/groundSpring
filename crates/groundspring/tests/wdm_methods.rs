// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Integration tests for WDM transport analysis helpers:
//! [`groundspring::wdm::autocorrelation`], [`groundspring::wdm::optimal_block_size`],
//! and [`groundspring::wdm::finite_size_extrapolate`].

#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use groundspring::prng::Xorshift64;
use groundspring::tol;
use groundspring::wdm::{autocorrelation, finite_size_extrapolate, optimal_block_size};

const PI: f64 = std::f64::consts::PI;

// ── autocorrelation ───────────────────────────────────────────────

#[test]
fn wdm_autocorrelation_constant_series_is_one_at_all_lags() {
    let data = vec![7.0; 200];
    let max_lag = 20;
    let acf = autocorrelation(&data, max_lag);

    assert_eq!(acf.len(), max_lag + 1);
    for (k, &c) in acf.iter().enumerate() {
        assert!(
            (c - 1.0).abs() < tol::EXACT,
            "constant series: ACF({k}) = {c}, expected 1.0"
        );
    }
}

#[test]
fn wdm_autocorrelation_white_noise_decays_toward_zero() {
    let mut rng = Xorshift64::new(123);
    let data: Vec<f64> = (0..2000).map(|_| rng.normal(0.0, 1.0)).collect();
    let acf = autocorrelation(&data, 50);

    assert!(
        (acf[0] - 1.0).abs() < tol::EXACT,
        "ACF(0) must be 1.0, got {}",
        acf[0]
    );

    let tail: f64 = acf[30..].iter().map(|c| c.abs()).sum::<f64>() / 21.0;
    assert!(
        tail < 0.15,
        "white noise tail |ACF| mean should be small, got {tail}"
    );
    assert!(
        acf[25].abs() < acf[1].abs() + 0.2,
        "ACF should decay: |ACF(25)|={} vs |ACF(1)|={}",
        acf[25].abs(),
        acf[1].abs()
    );
}

#[test]
fn wdm_autocorrelation_periodic_signal_reflects_periodicity() {
    let period = 10;
    let n = 500;
    let data: Vec<f64> = (0..n)
        .map(|i| (2.0 * PI * i as f64 / period as f64).sin())
        .collect();
    let acf = autocorrelation(&data, 30);

    assert!(
        (acf[0] - 1.0).abs() < tol::EXACT,
        "ACF(0) must be 1.0, got {}",
        acf[0]
    );

    let at_period = acf[period];
    let at_double_period = acf[2 * period];
    let at_half_period = acf[period / 2];
    let at_off_period = acf[3];

    assert!(
        at_period > 0.9,
        "periodic signal: ACF(period={period}) = {at_period}, expected near 1.0"
    );
    assert!(
        at_double_period > 0.9,
        "periodic signal: ACF(2×period) = {at_double_period}, expected near 1.0"
    );
    assert!(
        at_half_period < -0.9,
        "sine half-period lag should anti-correlate: ACF({}) = {at_half_period}",
        period / 2
    );
    assert!(
        at_off_period.abs() < at_period.abs(),
        "non-periodic lag |ACF(3)|={} should be weaker than |ACF(period)|={}",
        at_off_period.abs(),
        at_period.abs()
    );
}

// ── optimal_block_size ────────────────────────────────────────────

#[test]
fn wdm_optimal_block_size_iid_data_is_small() {
    let mut rng = Xorshift64::new(99);
    let data: Vec<f64> = (0..500).map(|_| rng.normal(0.0, 1.0)).collect();
    let bs = optimal_block_size(&data, 50);

    assert!(bs <= 5, "IID data should yield small block size, got {bs}");
}

#[test]
fn wdm_optimal_block_size_correlated_data_is_larger() {
    let mut rng = Xorshift64::new(42);
    let n = 2000;
    let phi = 0.9;
    let mut data = vec![0.0; n];
    data[0] = rng.normal(0.0, 1.0);
    let noise_std = (1.0_f64 - phi * phi).sqrt();
    for i in 1..n {
        data[i] = phi * data[i - 1] + rng.normal(0.0, noise_std);
    }

    let bs_iid = {
        let mut rng = Xorshift64::new(99);
        let iid: Vec<f64> = (0..n).map(|_| rng.normal(0.0, 1.0)).collect();
        optimal_block_size(&iid, 100)
    };
    let bs_corr = optimal_block_size(&data, 100);

    assert!(
        bs_corr >= 5,
        "correlated AR(1) data should yield larger block size, got {bs_corr}"
    );
    assert!(
        bs_corr > bs_iid,
        "correlated block size ({bs_corr}) should exceed IID ({bs_iid})"
    );
}

#[test]
fn wdm_optimal_block_size_always_at_least_one() {
    let cases: Vec<Vec<f64>> = vec![vec![1.0; 50], vec![1.0, 2.0, 3.0, 4.0, 5.0], {
        let mut rng = Xorshift64::new(7);
        (0..100).map(|_| rng.normal(0.0, 1.0)).collect()
    }];

    for data in &cases {
        let bs = optimal_block_size(data, 10);
        assert!(
            bs >= 1,
            "block size must be >= 1, got {bs} for len={}",
            data.len()
        );
    }
}

// ── finite_size_extrapolate ──────────────────────────────────────

#[test]
fn wdm_finite_size_extrapolate_convergent_one_over_n_sequence() {
    let d_inf_true = 1.5;
    let alpha_true = 2.0;
    let d_dim = 1.0;
    let sizes = vec![50.0, 100.0, 200.0, 500.0, 1000.0];
    let values: Vec<f64> = sizes.iter().map(|&n| d_inf_true + alpha_true / n).collect();

    let (d_inf, alpha, r_sq) = finite_size_extrapolate(&sizes, &values, d_dim).unwrap();

    assert!(d_inf.is_finite(), "D_inf must be finite, got {d_inf}");
    assert!(alpha.is_finite(), "alpha must be finite, got {alpha}");
    assert!(r_sq.is_finite(), "R² must be finite, got {r_sq}");
    assert!(
        (d_inf - d_inf_true).abs() < tol::LITERATURE,
        "D_inf: {d_inf} vs {d_inf_true}"
    );
    assert!(
        (alpha - alpha_true).abs() < tol::STOCHASTIC,
        "alpha: {alpha} vs {alpha_true}"
    );
    assert!(
        r_sq > 0.999,
        "R² should be near 1.0 for perfect 1/N data, got {r_sq}"
    );
}

#[test]
fn wdm_finite_size_extrapolate_constant_values_returns_same_value() {
    let constant = 4.25;
    let sizes = vec![100.0, 250.0, 500.0, 1000.0, 2000.0];
    let values = vec![constant; sizes.len()];

    let (d_inf, alpha, r_sq) = finite_size_extrapolate(&sizes, &values, 3.0).unwrap();

    assert!(
        (d_inf - constant).abs() < tol::ANALYTICAL,
        "constant series: D_inf = {d_inf}, expected {constant}"
    );
    assert!(
        alpha.abs() < tol::ANALYTICAL,
        "constant series: slope alpha = {alpha}, expected ~0"
    );
    assert!(
        r_sq >= 0.0 && r_sq <= 1.0 + tol::ANALYTICAL,
        "R² should be in [0, 1], got {r_sq}"
    );
}

#[test]
fn wdm_finite_size_extrapolate_output_finite_and_reasonable() {
    let d_inf_true = 2.0;
    let alpha_true = 5.0;
    let sizes = vec![100.0, 500.0, 1000.0, 5000.0, 10000.0];
    let values: Vec<f64> = sizes
        .iter()
        .map(|&n: &f64| d_inf_true + alpha_true / n.cbrt())
        .collect();

    let (d_inf, alpha, r_sq) = finite_size_extrapolate(&sizes, &values, 3.0).unwrap();

    assert!(d_inf.is_finite(), "D_inf must be finite");
    assert!(alpha.is_finite(), "alpha must be finite");
    assert!(r_sq.is_finite(), "R² must be finite");
    assert!(
        (0.0..=1.0).contains(&r_sq) || (r_sq - 1.0).abs() < tol::ANALYTICAL,
        "R² should be reasonable, got {r_sq}"
    );
    assert!(
        d_inf > 0.0 && d_inf < 10.0,
        "D_inf should be in a reasonable range, got {d_inf}"
    );
}
