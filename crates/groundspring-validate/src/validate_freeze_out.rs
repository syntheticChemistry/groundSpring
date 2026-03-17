// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Validation binary for Experiment 020: Freeze-Out Inverse Problem.
//!
//! Can grid-search chi-squared fitting recover known parameters from
//! noisy polynomial observables?
//!
//! References:
//! - Bazavov et al. (2016) Phys Rev D 93, 014512

use groundspring::freeze_out::{
    GridFitConfig, chi_squared, chi_squared_per_dof, freeze_out_curve, grid_fit_2d,
};
use groundspring::prng::Xorshift64;
use groundspring::validate::ValidationHarness;
use groundspring_validate::{
    OrExit, TOL_ANALYTICAL, TOL_EXACT, f64_field, f64_range, get_f64_vec, parse_benchmark,
    print_provenance_header, u64_field, usize_field,
};
use serde_json::Value;

const BENCHMARK: &str =
    include_str!("../../../control/freeze_out_inverse/benchmark_freeze_out.json");

struct ModelCtx {
    true_t0: f64,
    true_k2: f64,
    mu_b: Vec<f64>,
    noise_std: f64,
    seed: u64,
    n_rep: usize,
    true_curve: Vec<f64>,
}

fn validate_forward(h: &mut ValidationHarness, ctx: &ModelCtx) {
    println!("\n--- Part 1: Forward model correctness ---");
    let t_at_0 = freeze_out_curve(ctx.true_t0, ctx.true_k2, 0.0);
    println!("  T_f(0) = {t_at_0:.4} (expect {:.4})", ctx.true_t0);
    h.check_true("T_f(0) = T0", (t_at_0 - ctx.true_t0).abs() < TOL_EXACT);

    let t_at_400 = freeze_out_curve(ctx.true_t0, ctx.true_k2, 400.0);
    let r = 400.0 / ctx.true_t0;
    let expected = ctx.true_t0 * (-ctx.true_k2).mul_add(r * r, 1.0);
    println!("  T_f(400) = {t_at_400:.4} (expect {expected:.4})");
    h.check_true(
        "T_f(400) matches formula",
        (t_at_400 - expected).abs() < TOL_ANALYTICAL,
    );
}

fn validate_chi2_and_grid(h: &mut ValidationHarness, ctx: &ModelCtx, grid: &Value, exp: &Value) {
    println!("\n--- Part 2: Chi-squared at truth ---");
    let mut rng = Xorshift64::new(ctx.seed);
    let obs: Vec<f64> = ctx
        .true_curve
        .iter()
        .map(|&t| t + rng.normal(0.0, ctx.noise_std))
        .collect();
    let c2 = chi_squared(&obs, &ctx.true_curve, ctx.noise_std).or_exit("equal lengths");
    let n_dof = ctx.mu_b.len() - 2;
    let c2_dof = chi_squared_per_dof(c2, ctx.mu_b.len(), 2);
    println!("  chi2 = {c2:.3}, chi2/dof = {c2_dof:.3} (dof = {n_dof})");
    h.check_max(
        "Chi2/dof at truth reasonable",
        c2_dof,
        f64_field(exp, "chi2_per_dof_max"),
    );

    println!("\n--- Part 3: Grid search recovery ---");
    let (t0_lo, t0_hi) = f64_range(&grid["t0_range"]);
    let (k2_lo, k2_hi) = f64_range(&grid["kappa2_range"]);
    let cfg = GridFitConfig {
        observed: &obs,
        mu_b: &ctx.mu_b,
        sigma: ctx.noise_std,
        t0_lo,
        t0_hi,
        t0_step: f64_field(grid, "t0_step"),
        k2_lo,
        k2_hi,
        k2_step: f64_field(grid, "kappa2_step"),
    };
    let r = grid_fit_2d(&cfg).or_exit("observed and mu_b have equal length");
    println!(
        "  Best T0 = {:.2} (true {:.2}), kappa2 = {:.4} (true {:.4})",
        r.t0, ctx.true_t0, r.kappa2, ctx.true_k2
    );
    h.check_max(
        "T0 recovery error",
        (r.t0 - ctx.true_t0).abs(),
        f64_field(exp, "t0_recovery_tol"),
    );
    h.check_max(
        "kappa2 recovery error",
        (r.kappa2 - ctx.true_k2).abs(),
        f64_field(exp, "kappa2_recovery_tol"),
    );
}

fn validate_replicates_and_determinism(
    h: &mut ValidationHarness,
    ctx: &ModelCtx,
    grid: &Value,
    exp: &Value,
) {
    println!("\n--- Part 4: Replicate coverage ---");
    let (t0_lo, t0_hi) = f64_range(&grid["t0_range"]);
    let (k2_lo, k2_hi) = f64_range(&grid["kappa2_range"]);
    let t0_step = f64_field(grid, "t0_step");
    let k2_step = f64_field(grid, "kappa2_step");
    let t0_tol = f64_field(exp, "t0_recovery_tol");
    let k2_tol = f64_field(exp, "kappa2_recovery_tol");

    let mut hits = 0_usize;
    for i in 0..ctx.n_rep {
        let mut rng = Xorshift64::new(ctx.seed + (i as u64) + 1);
        let obs: Vec<f64> = ctx
            .true_curve
            .iter()
            .map(|&t| t + rng.normal(0.0, ctx.noise_std))
            .collect();
        let cfg = GridFitConfig {
            observed: &obs,
            mu_b: &ctx.mu_b,
            sigma: ctx.noise_std,
            t0_lo,
            t0_hi,
            t0_step,
            k2_lo,
            k2_hi,
            k2_step,
        };
        let r = grid_fit_2d(&cfg).or_exit("observed and mu_b have equal length");
        if (r.t0 - ctx.true_t0).abs() <= t0_tol && (r.kappa2 - ctx.true_k2).abs() <= k2_tol {
            hits += 1;
        }
    }
    #[expect(clippy::cast_precision_loss, reason = "hits and n_rep ≤ 100 ≪ 2^53")]
    let frac = hits as f64 / ctx.n_rep as f64;
    println!("  Coverage: {hits}/{} = {frac:.2}", ctx.n_rep);
    h.check_min(
        "Replicate coverage",
        frac,
        f64_field(exp, "replicate_coverage_min"),
    );

    println!("\n--- Part 5: Noise degrades precision ---");
    let low_sigma = ctx.noise_std * 0.1;
    let mut rng_low = Xorshift64::new(ctx.seed + 999);
    let obs_low: Vec<f64> = ctx
        .true_curve
        .iter()
        .map(|&t| t + rng_low.normal(0.0, low_sigma))
        .collect();
    let cfg_low = GridFitConfig {
        observed: &obs_low,
        mu_b: &ctx.mu_b,
        sigma: low_sigma,
        t0_lo,
        t0_hi,
        t0_step,
        k2_lo,
        k2_hi,
        k2_step,
    };
    let mut rng_hi = Xorshift64::new(ctx.seed);
    let obs_hi: Vec<f64> = ctx
        .true_curve
        .iter()
        .map(|&t| t + rng_hi.normal(0.0, ctx.noise_std))
        .collect();
    let cfg_hi = GridFitConfig {
        observed: &obs_hi,
        mu_b: &ctx.mu_b,
        sigma: ctx.noise_std,
        t0_lo,
        t0_hi,
        t0_step,
        k2_lo,
        k2_hi,
        k2_step,
    };
    let r_low = grid_fit_2d(&cfg_low).or_exit("observed and mu_b have equal length");
    let r_hi = grid_fit_2d(&cfg_hi).or_exit("observed and mu_b have equal length");
    let err_low = (r_low.t0 - ctx.true_t0).abs() + (r_low.kappa2 - ctx.true_k2).abs();
    let err_hi = (r_hi.t0 - ctx.true_t0).abs() + (r_hi.kappa2 - ctx.true_k2).abs();
    let noise_slack = f64_field(exp, "noise_degradation_slack");
    println!("  Low-noise err = {err_low:.4}, high-noise err = {err_hi:.4}");
    h.check_true(
        "Lower noise improves recovery",
        err_low <= err_hi + noise_slack,
    );

    println!("\n--- Part 6: Determinism ---");
    let mut r1 = Xorshift64::new(ctx.seed);
    let mut r2 = Xorshift64::new(ctx.seed);
    let o1: Vec<f64> = ctx
        .true_curve
        .iter()
        .map(|&t| t + r1.normal(0.0, ctx.noise_std))
        .collect();
    let o2: Vec<f64> = ctx
        .true_curve
        .iter()
        .map(|&t| t + r2.normal(0.0, ctx.noise_std))
        .collect();
    h.check_true("Observations deterministic", o1 == o2);
}

#[expect(
    clippy::similar_names,
    reason = "t0/k2 and lo/hi are domain-standard names"
)]
fn run() -> i32 {
    let bench = parse_benchmark(BENCHMARK);
    let mut h = ValidationHarness::stdout("Rust Validation: Freeze-Out Inverse Problem");

    println!("{}", "=".repeat(72));
    println!("groundSpring Rust Validation: Freeze-Out (Exp 020)");
    println!("{}", "=".repeat(72));
    print_provenance_header(&bench, "Freeze-Out Inverse Problem");

    let model = &bench["model"];
    let grid = &bench["grid"];
    let exp = &bench["expected_results"];

    let true_t0 = f64_field(model, "true_t0");
    let true_k2 = f64_field(model, "true_kappa2");
    let mu_b: Vec<f64> = get_f64_vec(model, "mu_b_values").or_exit("mu_b");
    let noise_std = f64_field(model, "noise_std");
    let seed = u64_field(model, "seed");
    let n_rep = usize_field(model, "n_replicates");

    let true_curve: Vec<f64> = mu_b
        .iter()
        .map(|&m| freeze_out_curve(true_t0, true_k2, m))
        .collect();

    let ctx = ModelCtx {
        true_t0,
        true_k2,
        mu_b,
        noise_std,
        seed,
        n_rep,
        true_curve,
    };

    validate_forward(&mut h, &ctx);
    validate_chi2_and_grid(&mut h, &ctx, grid, exp);
    validate_replicates_and_determinism(&mut h, &ctx, grid, exp);

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
