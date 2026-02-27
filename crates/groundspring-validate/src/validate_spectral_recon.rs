// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Validation binary for Experiment 021: Spectral Function Reconstruction.
//!
//! Can Tikhonov regularization recover a spectral peak from a noisy
//! integral transform?
//!
//! References:
//! - Bazavov et al. (2025) arXiv 2501.12259
//! - Tikhonov & Arsenin (1977)

#![expect(clippy::cast_precision_loss, clippy::cast_possible_truncation)]

use groundspring::prng::Xorshift64;
use groundspring::spectral_recon::{
    build_kernel, forward_correlator, gaussian_peak, peak_index, rmse, tikhonov_solve,
};
use groundspring::validate::ValidationHarness;
use groundspring_validate::{f64_field, f64_range, print_provenance_header, u64_field};
use serde_json::Value;

const BENCHMARK: &str =
    include_str!("../../../control/spectral_recon/benchmark_spectral_recon.json");

struct GridCtx {
    omega: Vec<f64>,
    n_tau: usize,
    n_omega: usize,
    rho_true: Vec<f64>,
    kernel: Vec<f64>,
    g_exact: Vec<f64>,
}

fn setup_grid(bench: &Value) -> GridCtx {
    let grid = &bench["grid"];
    let sf = &bench["spectral_function"];
    let n_tau = grid["n_tau"].as_u64().expect("n_tau") as usize;
    let n_omega = grid["n_omega"].as_u64().expect("n_omega") as usize;
    let tau_max = f64_field(grid, "tau_max");
    let omega_max = f64_field(grid, "omega_max");

    let tau: Vec<f64> = (1..=n_tau)
        .map(|i| (i as f64) * tau_max / (n_tau as f64))
        .collect();
    let omega: Vec<f64> = (1..=n_omega)
        .map(|i| (i as f64) * omega_max / (n_omega as f64))
        .collect();

    let center = f64_field(sf, "omega_center");
    let width = f64_field(sf, "omega_width");
    let amp = f64_field(sf, "amplitude");
    let rho_true = gaussian_peak(&omega, center, width, amp);
    let kernel = build_kernel(&tau, &omega);
    let g_exact = forward_correlator(&kernel, &rho_true, n_tau, n_omega);

    GridCtx {
        omega,
        n_tau,
        n_omega,
        rho_true,
        kernel,
        g_exact,
    }
}

fn validate_forward(h: &mut ValidationHarness, ctx: &GridCtx, exp: &Value) {
    println!("\n--- Part 1: Noiseless forward model ---");
    let rho_rt = tikhonov_solve(&ctx.kernel, &ctx.g_exact, 1e-12, ctx.n_tau, ctx.n_omega);
    let g_rt = forward_correlator(&ctx.kernel, &rho_rt, ctx.n_tau, ctx.n_omega);
    let r = rmse(&ctx.g_exact, &g_rt);
    println!("  Noiseless roundtrip RMSE = {r:.2e}");
    h.check_max(
        "Noiseless forward RMSE",
        r,
        f64_field(exp, "forward_rmse_noiseless_max"),
    );
}

fn validate_cholesky(h: &mut ValidationHarness, ctx: &GridCtx, exp: &Value) {
    println!("\n--- Part 2: Cholesky residual ---");
    let rho_nl = tikhonov_solve(&ctx.kernel, &ctx.g_exact, 1e-12, ctx.n_tau, ctx.n_omega);
    let g_nl = forward_correlator(&ctx.kernel, &rho_nl, ctx.n_tau, ctx.n_omega);
    let max_res: f64 = ctx
        .g_exact
        .iter()
        .zip(g_nl.iter())
        .map(|(&a, &b)| (a - b).abs())
        .fold(0.0_f64, f64::max);
    println!("  Max residual = {max_res:.2e}");
    h.check_max(
        "Cholesky max residual",
        max_res,
        f64_field(exp, "cholesky_residual_max"),
    );
}

fn validate_noisy_recon(h: &mut ValidationHarness, ctx: &GridCtx, bench: &Value, exp: &Value) {
    println!("\n--- Part 3: Noisy reconstruction ---");
    let noise_cfg = &bench["noise"];
    let reg = &bench["regularization"];
    let sigma = f64_field(noise_cfg, "correlator_noise_std");
    let seed = u64_field(noise_cfg, "seed");
    let lam_opt = f64_field(reg, "optimal_lambda");

    let mut rng = Xorshift64::new(seed);
    let g_noisy: Vec<f64> = ctx
        .g_exact
        .iter()
        .map(|&g| g + rng.normal(0.0, sigma))
        .collect();
    let rho_recon = tikhonov_solve(&ctx.kernel, &g_noisy, lam_opt, ctx.n_tau, ctx.n_omega);
    let pi = peak_index(&rho_recon);
    let peak_w = ctx.omega[pi];
    let sf = &bench["spectral_function"];
    let center = f64_field(sf, "omega_center");
    println!("  Peak at ω = {peak_w:.2} (true = {center:.2})");
    h.check_max(
        "Peak location error",
        (peak_w - center).abs(),
        f64_field(exp, "peak_location_tol"),
    );
    h.check_true("Peak value positive", rho_recon[pi] > 0.0);

    println!("\n--- Part 4: Regularization trade-off ---");
    let lambdas: Vec<f64> = reg["lambda_values"]
        .as_array()
        .expect("lambda_values")
        .iter()
        .map(|v| v.as_f64().expect("f64"))
        .collect();
    let mut rmses = Vec::new();
    for &lam in &lambdas {
        let rho_l = tikhonov_solve(&ctx.kernel, &g_noisy, lam, ctx.n_tau, ctx.n_omega);
        let r = rmse(&rho_l, &ctx.rho_true);
        println!("  λ = {lam:.0e}: RMSE = {r:.6}");
        rmses.push(r);
    }
    let opt_idx = 2;
    let opt_rmse = rmses[opt_idx];
    h.check_true("Small lambda amplifies noise", rmses[0] >= opt_rmse * 0.5);
    h.check_true(
        "Large lambda over-smooths",
        *rmses.last().unwrap_or(&0.0) >= opt_rmse * 0.5,
    );
    let (lo, hi) = f64_range(&exp["optimal_lambda_rmse_range"]);
    h.check_range("Optimal lambda RMSE in range", opt_rmse, lo, hi);

    println!("\n--- Part 5: Determinism ---");
    let mut rng2 = Xorshift64::new(seed);
    let g_noisy2: Vec<f64> = ctx
        .g_exact
        .iter()
        .map(|&g| g + rng2.normal(0.0, sigma))
        .collect();
    let rho2 = tikhonov_solve(&ctx.kernel, &g_noisy2, lam_opt, ctx.n_tau, ctx.n_omega);
    h.check_true("Reconstruction deterministic", rho_recon == rho2);
}

fn run() -> i32 {
    let bench: Value = serde_json::from_str(BENCHMARK).expect("valid benchmark JSON");
    let mut h = ValidationHarness::stdout("Rust Validation: Spectral Function Reconstruction");

    println!("{}", "=".repeat(72));
    println!("groundSpring Rust Validation: Spectral Recon (Exp 021)");
    println!("{}", "=".repeat(72));
    print_provenance_header(&bench, "Spectral Function Reconstruction");

    let exp = &bench["expected_results"];
    let ctx = setup_grid(&bench);

    validate_forward(&mut h, &ctx, exp);
    validate_cholesky(&mut h, &ctx, exp);
    validate_noisy_recon(&mut h, &ctx, &bench, exp);

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
