// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Validation binary for Experiment 019: Jackknife Error Estimation.
//!
//! Does jackknife error estimation achieve subpercent accuracy for
//! variance of smooth statistics on known distributions?
//!
//! References:
//! - Bazavov et al. (2025) Phys Rev D 111, 094508
//! - Quenouille (1956) Biometrika 43:353-360

#![expect(clippy::cast_precision_loss, clippy::cast_possible_truncation)]

use groundspring::jackknife::{
    block_jackknife_variance, jackknife_bias, jackknife_mean_variance,
    leave_one_out_biased_variance,
};
use groundspring::prng::Xorshift64;
use groundspring::validate::ValidationHarness;
use groundspring_validate::{
    OrExit, f64_field, f64_range, get_array, get_u64, parse_benchmark, print_provenance_header,
    u64_field,
};
use serde_json::Value;

const BENCHMARK: &str =
    include_str!("../../../control/jackknife_estimation/benchmark_jackknife.json");

struct GaussCtx {
    data: Vec<f64>,
    true_mean: f64,
}

fn validate_gaussian(h: &mut ValidationHarness, ctx: &GaussCtx, exp: &Value) {
    println!("\n--- Part 1: Jackknife on Gaussian data ---");
    let r = jackknife_mean_variance(&ctx.data).or_exit("gaussian data >= 2 elements");
    println!("  JK mean = {:.4}, JK var = {:.6}", r.estimate, r.variance);
    h.check_max(
        "Jackknife mean near true mean",
        (r.estimate - ctx.true_mean).abs(),
        f64_field(exp, "gaussian_jk_mean_tol"),
    );
    let (lo, hi) = f64_range(&exp["gaussian_jk_var_range"]);
    h.check_range("Jackknife variance of mean", r.variance, lo, hi);
}

fn validate_exponential(h: &mut ValidationHarness, exp_cfg: &Value, exp: &Value) {
    println!("\n--- Part 2: Jackknife on Exponential data ---");
    let rate = f64_field(exp_cfg, "rate");
    let n = get_u64(exp_cfg, "n_samples").or_exit("n_samples") as usize;
    let seed = u64_field(exp_cfg, "seed");

    let mut rng = Xorshift64::new(seed);
    let data: Vec<f64> = (0..n)
        .map(|_| -rng.next_f64().max(f64::MIN_POSITIVE).ln() / rate)
        .collect();
    let r = jackknife_mean_variance(&data).or_exit("exponential data >= 2 elements");
    let true_mean = 1.0 / rate;
    println!(
        "  JK mean = {:.4} (true = {true_mean:.4}), JK var = {:.6}",
        r.estimate, r.variance
    );
    h.check_max(
        "Exponential JK mean near 1/rate",
        (r.estimate - true_mean).abs(),
        f64_field(exp, "exponential_jk_mean_tol"),
    );
    let (lo, hi) = f64_range(&exp["exponential_jk_var_range"]);
    h.check_range("Exponential JK variance of mean", r.variance, lo, hi);
}

struct CorrCtx {
    data: Vec<f64>,
    block_sizes: Vec<usize>,
}

fn validate_block_and_bias(
    h: &mut ValidationHarness,
    gauss: &GaussCtx,
    corr: &CorrCtx,
    bench: &Value,
    exp: &Value,
) {
    println!("\n--- Part 3: Jackknife bias correction ---");
    let n_f = gauss.data.len() as f64;
    let mean = gauss.data.iter().sum::<f64>() / n_f;
    let full_biased_var: f64 = gauss.data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n_f;
    let loo = leave_one_out_biased_variance(&gauss.data);
    let (bias, corrected) = jackknife_bias(&loo, full_biased_var);
    let true_std = f64_field(&bench["gaussian"], "true_std");
    let true_var = true_std * true_std;
    let naive_err = (full_biased_var - true_var).abs();
    let corrected_err = (corrected - true_var).abs();
    let error_ratio_max = f64_field(exp, "bias_correction_error_ratio_max");
    println!("  Biased var = {full_biased_var:.4}, bias = {bias:.6}, corrected = {corrected:.4}");
    h.check_true(
        "Bias correction reduces error",
        corrected_err < naive_err * error_ratio_max,
    );

    println!("\n--- Part 4: Block jackknife on correlated data ---");
    let monotone_slack = f64_field(exp, "block_jk_monotone_slack");
    let mut block_vars = Vec::new();
    for &bs in &corr.block_sizes {
        let r = block_jackknife_variance(&corr.data, bs).or_exit("block_size valid");
        println!("  block_size={bs:3}: var = {:.6}", r.variance);
        block_vars.push(r.variance);
    }
    let monotone = block_vars
        .windows(2)
        .take(block_vars.len().saturating_sub(2))
        .all(|w| w[0] <= w[1] * monotone_slack);
    h.check_true("Block JK variance increases with block size", monotone);
    let (lo, hi) = f64_range(&exp["block_jk_large_block_var_range"]);
    h.check_range(
        "Large-block variance in expected range",
        *block_vars.last().unwrap_or(&0.0),
        lo,
        hi,
    );
}

fn validate_comparison_and_determinism(
    h: &mut ValidationHarness,
    gauss: &GaussCtx,
    bench: &Value,
    exp: &Value,
) {
    println!("\n--- Part 5: Jackknife vs bootstrap comparison ---");
    let jk = jackknife_mean_variance(&gauss.data).or_exit("gaussian data >= 2 elements");
    let boot_cfg = &bench["bootstrap_comparison"];
    let boot_seed = u64_field(boot_cfg, "seed");
    let n_boot = get_u64(boot_cfg, "n_bootstrap").or_exit("n_bootstrap") as usize;
    let mut rng = Xorshift64::new(boot_seed);
    let n = gauss.data.len();
    let mut boot_means = Vec::with_capacity(n_boot);
    for _ in 0..n_boot {
        let mut s = 0.0;
        for _ in 0..n {
            let idx = (rng.next_u64() % (n as u64)) as usize;
            s += gauss.data[idx];
        }
        boot_means.push(s / n as f64);
    }
    let bm: f64 = boot_means.iter().sum::<f64>() / n_boot as f64;
    let boot_var: f64 =
        boot_means.iter().map(|&x| (x - bm).powi(2)).sum::<f64>() / (n_boot - 1) as f64;
    let ratio = if boot_var > 0.0 {
        jk.variance / boot_var
    } else {
        f64::INFINITY
    };
    println!(
        "  JK var = {:.6}, Bootstrap var = {boot_var:.6}, ratio = {ratio:.3}",
        jk.variance
    );
    let (lo, hi) = f64_range(&exp["jk_bootstrap_ratio_range"]);
    h.check_range("Jackknife/bootstrap variance ratio", ratio, lo, hi);

    println!("\n--- Part 6: Determinism ---");
    let r1 = jackknife_mean_variance(&gauss.data).or_exit("gaussian data >= 2");
    let r2 = jackknife_mean_variance(&gauss.data).or_exit("gaussian data >= 2");
    h.check_true(
        "Jackknife deterministic",
        r1.estimate.to_bits() == r2.estimate.to_bits()
            && r1.variance.to_bits() == r2.variance.to_bits(),
    );
}

fn run() -> i32 {
    let bench = parse_benchmark(BENCHMARK);
    let mut h = ValidationHarness::stdout("Rust Validation: Jackknife Error Estimation");

    print_provenance_header(&bench, "Jackknife Error Estimation");

    let gauss_cfg = &bench["gaussian"];
    let exp_cfg = &bench["exponential"];
    let corr_cfg = &bench["correlated"];
    let exp = &bench["expected_results"];

    let n_gauss = get_u64(gauss_cfg, "n_samples").or_exit("n") as usize;
    let true_mean = f64_field(gauss_cfg, "true_mean");
    let true_std = f64_field(gauss_cfg, "true_std");
    let seed_g = u64_field(gauss_cfg, "seed");

    let mut rng = Xorshift64::new(seed_g);
    let gauss_data: Vec<f64> = (0..n_gauss)
        .map(|_| rng.normal(true_mean, true_std))
        .collect();

    let gauss = GaussCtx {
        data: gauss_data,
        true_mean,
    };

    let n_corr = get_u64(corr_cfg, "n_samples").or_exit("n") as usize;
    let phi = f64_field(corr_cfg, "ar1_phi");
    let corr_mean = f64_field(corr_cfg, "true_mean");
    let corr_std = f64_field(corr_cfg, "true_std");
    let seed_c = u64_field(corr_cfg, "seed");
    let innovation_std = corr_std * phi.mul_add(-phi, 1.0).sqrt();

    let mut rng_c = Xorshift64::new(seed_c);
    let mut corr_data = vec![0.0; n_corr];
    corr_data[0] = rng_c.normal(corr_mean, corr_std);
    for i in 1..n_corr {
        corr_data[i] = phi.mul_add(corr_data[i - 1] - corr_mean, corr_mean)
            + rng_c.normal(0.0, innovation_std);
    }

    let block_sizes: Vec<usize> = get_array(corr_cfg, "block_sizes")
        .or_exit("block_sizes")
        .iter()
        .map(|v| v.as_u64().or_exit("u64") as usize)
        .collect();

    let corr = CorrCtx {
        data: corr_data,
        block_sizes,
    };

    validate_gaussian(&mut h, &gauss, exp);
    validate_exponential(&mut h, exp_cfg, exp);
    validate_block_and_bias(&mut h, &gauss, &corr, &bench, exp);
    validate_comparison_and_determinism(&mut h, &gauss, &bench, exp);

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
