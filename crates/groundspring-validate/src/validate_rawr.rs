// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Validation binary for Experiment 007: RAWR Resampling.
//!
//! Compares standard bootstrap vs RAWR (Bayesian bootstrap) for
//! confidence interval estimation on Gaussian, skewed, and correlated data.
//!
//! Reference: Wang et al. (2021) Bioinformatics (ISMB) 37:i111-i119

use groundspring::bootstrap::{bootstrap_mean, rawr_mean};
use groundspring::prng::Xorshift64;
use groundspring::validate::ValidationHarness;
use serde_json::Value;

const BENCHMARK: &str =
    include_str!("../../../control/rawr_resampling/benchmark_rawr_resampling.json");

fn f64_field(v: &Value, key: &str) -> f64 {
    v[key]
        .as_f64()
        .unwrap_or_else(|| panic!("missing f64 field: {key}"))
}

#[expect(clippy::cast_possible_truncation)]
fn usize_field(v: &Value, key: &str) -> usize {
    v[key]
        .as_u64()
        .unwrap_or_else(|| panic!("missing u64 field: {key}")) as usize
}

fn u64_field(v: &Value, key: &str) -> u64 {
    v[key]
        .as_u64()
        .unwrap_or_else(|| panic!("missing u64 field: {key}"))
}

fn f64_range(arr: &Value) -> (f64, f64) {
    let a = arr.as_array().expect("expected JSON array for range");
    (
        a[0].as_f64().expect("range lower bound"),
        a[1].as_f64().expect("range upper bound"),
    )
}

fn generate_normal(n: usize, mu: f64, sigma: f64, seed: u64) -> Vec<f64> {
    let mut rng = Xorshift64::new(seed);
    (0..n).map(|_| rng.normal(mu, sigma)).collect()
}

fn generate_lognormal(n: usize, mu_ln: f64, sigma_ln: f64, seed: u64) -> Vec<f64> {
    generate_normal(n, mu_ln, sigma_ln, seed)
        .into_iter()
        .map(f64::exp)
        .collect()
}

fn generate_ar1(n: usize, mu: f64, sigma: f64, rho: f64, seed: u64) -> Vec<f64> {
    let normals = generate_normal(n, 0.0, sigma * rho.mul_add(-rho, 1.0).sqrt(), seed);
    let mut data = Vec::with_capacity(n);
    data.push(mu + normals[0]);
    for i in 1..n {
        data.push(rho.mul_add(data[i - 1] - mu, mu) + normals[i]);
    }
    data
}

#[expect(clippy::cast_precision_loss)]
fn coverage_test(
    data_gen: impl Fn(u64) -> Vec<f64>,
    true_param: f64,
    method: impl Fn(&[f64], usize, f64, u64) -> groundspring::bootstrap::BootstrapResult,
    n_trials: usize,
    n_bootstrap: usize,
    confidence: f64,
    base_seed: u64,
) -> f64 {
    let mut covers = 0usize;
    for trial in 0..n_trials {
        let data = data_gen(base_seed + trial as u64);
        let result = method(
            &data,
            n_bootstrap,
            confidence,
            base_seed + 100_000 + trial as u64,
        );
        if result.ci_lower <= true_param && true_param <= result.ci_upper {
            covers += 1;
        }
    }
    covers as f64 / n_trials as f64
}

#[expect(clippy::cast_precision_loss, clippy::float_cmp, clippy::too_many_lines)]
fn run() -> i32 {
    let bench: Value = serde_json::from_str(BENCHMARK).expect("valid benchmark JSON");
    let mut h = ValidationHarness::stdout("Rust Validation: RAWR Resampling");

    let exp = &bench["expected_results"];

    println!("{}", "=".repeat(72));
    println!("groundSpring Rust Validation: RAWR Resampling");
    println!("{}", "=".repeat(72));

    // ── Part 1: Gaussian ──────────────────────────────────────────────
    println!("\n--- Part 1: Gaussian ---");

    let tc = &bench["test_cases"]["gaussian"];
    let n = usize_field(tc, "n");
    let mu = f64_field(tc, "mu");
    let sigma = f64_field(tc, "sigma");
    let n_boot = usize_field(tc, "n_bootstrap");
    let conf = f64_field(tc, "confidence");
    let seed = u64_field(tc, "seed");

    let data = generate_normal(n, mu, sigma, seed);
    let boot_r = bootstrap_mean(&data, n_boot, conf, seed + 1);
    let rawr_r = rawr_mean(&data, n_boot, conf, seed + 2);

    println!(
        "  Bootstrap: {:.3} [{:.3}, {:.3}] w={:.3}",
        boot_r.estimate,
        boot_r.ci_lower,
        boot_r.ci_upper,
        boot_r.ci_upper - boot_r.ci_lower
    );
    println!(
        "  RAWR:      {:.3} [{:.3}, {:.3}] w={:.3}",
        rawr_r.estimate,
        rawr_r.ci_lower,
        rawr_r.ci_upper,
        rawr_r.ci_upper - rawr_r.ci_lower
    );

    h.check_true(
        "Bootstrap estimate near true μ",
        (boot_r.estimate - mu).abs() < 2.0 * sigma,
    );
    h.check_true(
        "RAWR estimate near true μ",
        (rawr_r.estimate - mu).abs() < 2.0 * sigma,
    );

    let (ci_lo, ci_hi) = f64_range(&exp["gaussian_bootstrap_ci_width_range"]);
    h.check_range(
        "Bootstrap CI width",
        boot_r.ci_upper - boot_r.ci_lower,
        ci_lo,
        ci_hi,
    );

    let n_cov_trials = 200;
    let (cov_lo, cov_hi) = f64_range(&exp["gaussian_coverage_range"]);

    let boot_cov = coverage_test(
        |s| generate_normal(n, mu, sigma, s),
        mu,
        bootstrap_mean,
        n_cov_trials,
        n_boot,
        conf,
        seed + 5000,
    );
    let rawr_cov = coverage_test(
        |s| generate_normal(n, mu, sigma, s),
        mu,
        rawr_mean,
        n_cov_trials,
        n_boot,
        conf,
        seed + 6000,
    );

    println!("  Bootstrap coverage: {boot_cov:.3}");
    println!("  RAWR coverage:      {rawr_cov:.3}");

    h.check_range("Bootstrap Gaussian coverage", boot_cov, cov_lo, cov_hi);
    h.check_range("RAWR Gaussian coverage", rawr_cov, cov_lo, cov_hi);

    // ── Part 2: Skewed ────────────────────────────────────────────────
    println!("\n--- Part 2: Skewed ---");

    let tc_s = &bench["test_cases"]["skewed"];
    let n_s = usize_field(tc_s, "n");
    let mu_ln = f64_field(tc_s, "lognormal_mu");
    let sigma_ln = f64_field(tc_s, "lognormal_sigma");
    let true_mean_s = (mu_ln + sigma_ln * sigma_ln / 2.0).exp();
    let n_boot_s = usize_field(tc_s, "n_bootstrap");
    let conf_s = f64_field(tc_s, "confidence");
    let seed_s = u64_field(tc_s, "seed");

    let (skew_lo, skew_hi) = f64_range(&exp["skewed_coverage_range"]);

    let boot_cov_s = coverage_test(
        |s| generate_lognormal(n_s, mu_ln, sigma_ln, s),
        true_mean_s,
        bootstrap_mean,
        n_cov_trials,
        n_boot_s,
        conf_s,
        seed_s + 5000,
    );
    let rawr_cov_s = coverage_test(
        |s| generate_lognormal(n_s, mu_ln, sigma_ln, s),
        true_mean_s,
        rawr_mean,
        n_cov_trials,
        n_boot_s,
        conf_s,
        seed_s + 6000,
    );

    println!("  True mean: {true_mean_s:.4}");
    println!("  Bootstrap coverage: {boot_cov_s:.3}");
    println!("  RAWR coverage:      {rawr_cov_s:.3}");

    h.check_range("Bootstrap skewed coverage", boot_cov_s, skew_lo, skew_hi);
    h.check_range("RAWR skewed coverage", rawr_cov_s, skew_lo, skew_hi);

    // ── Part 3: Correlated ────────────────────────────────────────────
    println!("\n--- Part 3: Correlated ---");

    let tc_c = &bench["test_cases"]["correlated"];
    let n_c = usize_field(tc_c, "n");
    let mu_c = f64_field(tc_c, "mu");
    let sigma_c = f64_field(tc_c, "sigma");
    let rho = f64_field(tc_c, "rho");
    let n_boot_c = usize_field(tc_c, "n_bootstrap");
    let conf_c = f64_field(tc_c, "confidence");
    let seed_c = u64_field(tc_c, "seed");

    let mut boot_mses = Vec::new();
    let mut rawr_mses = Vec::new();
    for trial in 0..n_cov_trials {
        let data_ar = generate_ar1(n_c, mu_c, sigma_c, rho, seed_c + trial as u64);
        let br = bootstrap_mean(&data_ar, n_boot_c, conf_c, seed_c + 200_000 + trial as u64);
        let rr = rawr_mean(&data_ar, n_boot_c, conf_c, seed_c + 300_000 + trial as u64);
        boot_mses.push((br.estimate - mu_c).powi(2));
        rawr_mses.push((rr.estimate - mu_c).powi(2));
    }

    let boot_rmse = (boot_mses.iter().sum::<f64>() / n_cov_trials as f64).sqrt();
    let rawr_rmse = (rawr_mses.iter().sum::<f64>() / n_cov_trials as f64).sqrt();
    let ratio = if boot_rmse > 0.0 {
        rawr_rmse / boot_rmse
    } else {
        1.0
    };

    println!("  Bootstrap RMSE: {boot_rmse:.4}");
    println!("  RAWR RMSE:      {rawr_rmse:.4}");
    println!("  Ratio:          {ratio:.3}");

    h.check_max(
        "RAWR/Bootstrap RMSE ratio",
        ratio,
        f64_field(exp, "correlated_rawr_mse_ratio_max"),
    );

    // ── Part 4: Determinism ───────────────────────────────────────────
    println!("\n--- Part 4: Determinism ---");

    let det_data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let b1 = bootstrap_mean(&det_data, 500, 0.95, 9999);
    let b2 = bootstrap_mean(&det_data, 500, 0.95, 9999);
    h.check_true("Bootstrap deterministic", b1.estimate == b2.estimate);

    let r1 = rawr_mean(&det_data, 500, 0.95, 8888);
    let r2 = rawr_mean(&det_data, 500, 0.95, 8888);
    h.check_true("RAWR deterministic", r1.estimate == r2.estimate);

    h.check_true(
        "Bootstrap ≠ RAWR (different methods)",
        b1.estimate != r1.estimate,
    );

    h.summary()
}

fn main() {
    std::process::exit(run());
}

#[cfg(test)]
mod tests {
    #[test]
    fn validation_passes() {
        assert_eq!(super::run(), 0);
    }
}
