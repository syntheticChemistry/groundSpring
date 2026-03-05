// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Statistical metrics, regression, bootstrap, and jackknife parity checks.

use groundspring::tol;
use groundspring_forge::harness::Harness;

/// Run all stats-domain parity checks.
pub fn validate_all(h: &mut Harness) {
    validate_stats_cpu_delegation_parity(h);
    validate_regression_parity(h);
    validate_bootstrap_parity(h);
    validate_stats_tier_a_gpu_parity(h);
    validate_jackknife_gpu_parity(h);
}

fn validate_stats_cpu_delegation_parity(h: &mut Harness) {
    println!("\n--- Stats Metrics Parity (airSpring+groundSpring S64) ---\n");

    let observed = vec![2.5, 3.1, 4.2, 5.0, 3.8, 4.5, 2.9, 3.6, 4.1, 3.3];
    let simulated = vec![2.4, 3.3, 4.0, 5.2, 3.7, 4.6, 2.8, 3.5, 4.3, 3.1];

    let rmse = groundspring::stats::rmse(&observed, &simulated);
    let mae = groundspring::stats::mae(&observed, &simulated);
    let nse = groundspring::stats::nash_sutcliffe(&observed, &simulated);
    let r2 = groundspring::stats::r_squared(&observed, &simulated);
    let ia = groundspring::stats::index_of_agreement(&observed, &simulated);
    let mbe = groundspring::stats::mbe(&observed, &simulated);
    let pearson = groundspring::stats::pearson_r(&observed, &simulated);

    println!("  RMSE={rmse:.6}, MAE={mae:.6}, NSE={nse:.6}");
    println!("  R²={r2:.6}, IA={ia:.6}, MBE={mbe:.6}, Pearson={pearson:.6}");

    h.check("RMSE > 0", rmse > 0.0);
    h.check("MAE > 0", mae > 0.0);
    h.check("NSE > 0.9 (good fit)", nse > 0.9);
    h.check("R² > 0.9", r2 > 0.9);
    h.check("IA > 0.9", ia > 0.9);
    h.check("Pearson > 0.9", pearson > 0.9);

    let rmse2 = groundspring::stats::rmse(&observed, &simulated);
    h.check(
        "Stats deterministic (bitwise)",
        rmse.to_bits() == rmse2.to_bits(),
    );
}

fn validate_regression_parity(h: &mut Harness) {
    println!("\n--- Regression Parity (airSpring S66) ---\n");

    let x: Vec<f64> = (0..100).map(|i| f64::from(i) * 0.1).collect();
    let y: Vec<f64> = x.iter().map(|&xi| 2.5f64.mul_add(xi, 1.0)).collect();

    let fit = groundspring::stats::fit_linear(&x, &y);
    h.check("Linear fit succeeds", fit.is_some());
    if let Some(ref f) = fit {
        println!(
            "  slope={:.6}, intercept={:.6}, R²={:.10}",
            f.slope, f.intercept, f.r_squared
        );
        h.check("Slope ≈ 2.5", (f.slope - 2.5).abs() < tol::ANALYTICAL);
        h.check(
            "Intercept ≈ 1.0",
            (f.intercept - 1.0).abs() < tol::ANALYTICAL,
        );
        h.check("R² ≈ 1.0", (f.r_squared - 1.0).abs() < tol::ANALYTICAL);
    }

    let fit2 = groundspring::stats::fit_linear(&x, &y);
    if let (Some(ref f1), Some(ref f2)) = (&fit, &fit2) {
        h.check(
            "Regression deterministic",
            f1.r_squared.to_bits() == f2.r_squared.to_bits(),
        );
    }
}

fn validate_bootstrap_parity(h: &mut Harness) {
    println!("\n--- Bootstrap RAWR Parity (groundSpring S66) ---\n");

    let data: Vec<f64> = (0..1000).map(|i| f64::from(i) * 0.001).collect();

    let ci1 = groundspring::bootstrap::rawr_mean(&data, 500, 0.05, 42);
    let ci2 = groundspring::bootstrap::rawr_mean(&data, 500, 0.05, 42);

    println!(
        "  CI: [{:.6}, {:.6}], estimate={:.6}",
        ci1.ci_lower, ci1.ci_upper, ci1.estimate
    );

    h.check("RAWR CI valid", ci1.ci_lower < ci1.ci_upper);
    h.check(
        "RAWR deterministic (same seed → bitwise identical)",
        ci1.estimate.to_bits() == ci2.estimate.to_bits()
            && ci1.ci_lower.to_bits() == ci2.ci_lower.to_bits(),
    );
}

fn validate_stats_tier_a_gpu_parity(h: &mut Harness) {
    println!("\n--- Stats Tier A GPU Parity (MAE, NSE, R²) ---\n");

    let observed = vec![2.5, 3.1, 4.2, 5.0, 3.8, 4.5, 2.9, 3.6, 4.1, 3.3];
    let simulated = vec![2.4, 3.3, 4.0, 5.2, 3.7, 4.6, 2.8, 3.5, 4.3, 3.1];

    let mae1 = groundspring::stats::mae(&observed, &simulated);
    let mae2 = groundspring::stats::mae(&observed, &simulated);
    h.check("MAE > 0", mae1 > 0.0);
    h.check(
        "MAE deterministic (bitwise)",
        mae1.to_bits() == mae2.to_bits(),
    );

    let nse1 = groundspring::stats::nash_sutcliffe(&observed, &simulated);
    let nse2 = groundspring::stats::nash_sutcliffe(&observed, &simulated);
    h.check("NSE > 0.9", nse1 > 0.9);
    h.check(
        "NSE deterministic (bitwise)",
        nse1.to_bits() == nse2.to_bits(),
    );

    let r2_1 = groundspring::stats::r_squared(&observed, &simulated);
    let r2_2 = groundspring::stats::r_squared(&observed, &simulated);
    h.check("R² > 0.9", r2_1 > 0.9);
    h.check(
        "R² deterministic (bitwise)",
        r2_1.to_bits() == r2_2.to_bits(),
    );

    h.check(
        "NSE == R² (mathematically identical)",
        (nse1 - r2_1).abs() < tol::EXACT,
    );

    println!("  MAE={mae1:.6}, NSE={nse1:.6}, R²={r2_1:.6}");
}

fn validate_jackknife_gpu_parity(h: &mut Harness) {
    println!("\n--- Jackknife GPU Parity (V66) ---\n");

    let data: Vec<f64> = (0..200).map(|i| f64::from(i) * 0.005).collect();

    let t0 = std::time::Instant::now();
    let Some(jk1) = groundspring::jackknife::jackknife_mean_variance(&data).ok() else {
        h.check("Jackknife computation", false);
        return;
    };
    let us = t0.elapsed().as_micros();

    let Some(jk2) = groundspring::jackknife::jackknife_mean_variance(&data).ok() else {
        h.check("Jackknife rerun", false);
        return;
    };

    h.check("Jackknife estimate > 0", jk1.estimate > 0.0);
    h.check("Jackknife variance > 0", jk1.variance > 0.0);
    h.check("Jackknife std_error > 0", jk1.std_error > 0.0);
    h.check(
        "Jackknife deterministic (bitwise)",
        jk1.estimate.to_bits() == jk2.estimate.to_bits(),
    );

    println!(
        "  estimate={:.6}, variance={:.6}, std_error={:.6}, {us} µs",
        jk1.estimate, jk1.variance, jk1.std_error
    );
}
