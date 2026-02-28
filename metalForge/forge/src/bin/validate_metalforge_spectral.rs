// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! GPU live validation: spectral function reconstruction via Tikhonov
//! regularization. Compares CPU-only Cholesky with barracuda-dispatched
//! solver (`solve_f64_cpu` or `solve_f64` GPU when available).
//!
//! # Provenance
//!
//! - **Expected values**: Analytical Gaussian peak ρ(ω) = Gauss(3.0, 0.5).
//!   Noiseless round-trip `RMSE(G, G_rec)` < 1e-6 (regularization-limited).
//!   Peak location within ±1.0 of true centre (3.0).
//! - **Tolerance**: RMSE < 1e-6 (noiseless). CPU/dispatched solutions agree
//!   to relative error < 1e-10 (different LU/Cholesky factorisations).
//! - **Reference**: Bazavov et al. (2025) arXiv 2501.12259.
//!
//! Exit 0 if all checks pass, exit 1 on any failure.

use groundspring_forge::harness::Harness;
use std::time::Instant;

fn run_spectral_checks(harness: &mut Harness) {
    let n_tau = 20;
    let n_omega = 40;
    let true_centre = 3.0;
    let true_width = 0.5;
    let lambda = 1e-12;

    let tau: Vec<f64> = (1..=n_tau)
        .map(|idx| {
            #[expect(clippy::cast_precision_loss)]
            let val = idx as f64 * 2.0 / n_tau as f64;
            val
        })
        .collect();
    let omega: Vec<f64> = (1..=n_omega)
        .map(|idx| {
            #[expect(clippy::cast_precision_loss)]
            let val = idx as f64 * 8.0 / n_omega as f64;
            val
        })
        .collect();

    let kernel = groundspring::spectral_recon::build_kernel(&tau, &omega);
    let rho_true = groundspring::spectral_recon::gaussian_peak(&omega, true_centre, true_width, 1.0);
    let g_data = groundspring::spectral_recon::forward_correlator(&kernel, &rho_true, n_tau, n_omega);

    println!("\n--- CPU-only Tikhonov (local Cholesky) ---\n");
    let cpu_start = Instant::now();
    let rho_cpu = cpu_tikhonov_solve(&kernel, &g_data, lambda, n_tau, n_omega);
    let cpu_us = cpu_start.elapsed().as_micros();
    let g_cpu = groundspring::spectral_recon::forward_correlator(&kernel, &rho_cpu, n_tau, n_omega);
    let rmse_cpu = groundspring::spectral_recon::rmse(&g_data, &g_cpu);
    let peak_cpu = groundspring::spectral_recon::peak_index(&rho_cpu);

    println!("  RMSE(G, G_rec) = {rmse_cpu:.2e}");
    println!("  Peak at ω = {:.2} (true = {true_centre:.1})", omega[peak_cpu]);
    println!("  Time: {cpu_us} µs");

    harness.check("CPU RMSE < 1e-6", rmse_cpu < 1e-6);
    harness.check(
        "CPU peak within ±1.0",
        (omega[peak_cpu] - true_centre).abs() < 1.0,
    );

    println!("\n--- Dispatched Tikhonov (feature-gated barracuda) ---\n");
    let disp_start = Instant::now();
    let rho_disp =
        groundspring::spectral_recon::tikhonov_solve(&kernel, &g_data, lambda, n_tau, n_omega);
    let disp_us = disp_start.elapsed().as_micros();
    let g_disp =
        groundspring::spectral_recon::forward_correlator(&kernel, &rho_disp, n_tau, n_omega);
    let rmse_disp = groundspring::spectral_recon::rmse(&g_data, &g_disp);
    let peak_disp = groundspring::spectral_recon::peak_index(&rho_disp);

    println!("  RMSE(G, G_rec) = {rmse_disp:.2e}");
    println!(
        "  Peak at ω = {:.2} (true = {true_centre:.1})",
        omega[peak_disp]
    );
    println!("  Time: {disp_us} µs");

    harness.check("Dispatched RMSE < 1e-6", rmse_disp < 1e-6);
    harness.check(
        "Dispatched peak within ±1.0",
        (omega[peak_disp] - true_centre).abs() < 1.0,
    );

    println!("\n--- CPU ↔ Dispatched Parity ---\n");

    let max_diff: f64 = rho_cpu
        .iter()
        .zip(&rho_disp)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0_f64, f64::max);
    let rho_max: f64 = rho_cpu.iter().fold(0.0_f64, |m, &v| m.max(v.abs()));
    let rel_diff = if rho_max > 0.0 {
        max_diff / rho_max
    } else {
        0.0
    };

    println!("  Max |ρ_cpu - ρ_disp| = {max_diff:.2e}");
    println!("  Relative difference  = {rel_diff:.2e}");

    harness.check("Solutions agree (rel < 1e-8)", rel_diff < 1e-8);
    harness.check("Peak location matches", peak_cpu == peak_disp);

    println!("\n--- Timing Summary ---\n");
    println!("  CPU:        {cpu_us} µs");
    println!("  Dispatched: {disp_us} µs");
    if disp_us > 0 {
        #[expect(clippy::cast_precision_loss)]
        let speedup = cpu_us as f64 / disp_us as f64;
        println!("  Ratio (CPU/Dispatched): {speedup:.2}x");
    }
}

/// CPU-only Tikhonov solve — bypasses feature-gated dispatch.
fn cpu_tikhonov_solve(
    kernel: &[f64],
    data: &[f64],
    reg_lambda: f64,
    n_tau: usize,
    n_omega: usize,
) -> Vec<f64> {
    let ktk = mat_transpose_mul(kernel, kernel, n_tau, n_omega, n_omega);
    let ktg = mat_transpose_vec(kernel, data, n_tau, n_omega);

    let mut mat_a = ktk;
    for i in 0..n_omega {
        mat_a[i * n_omega + i] += reg_lambda;
    }

    cholesky_solve(&mat_a, &ktg, n_omega)
}

fn mat_transpose_mul(
    mat_a: &[f64],
    mat_b: &[f64],
    rows: usize,
    cols_a: usize,
    cols_b: usize,
) -> Vec<f64> {
    let mut out = vec![0.0; cols_a * cols_b];
    for i in 0..cols_a {
        for j in 0..cols_b {
            let mut acc = 0.0;
            for l in 0..rows {
                acc = mat_a[l * cols_a + i].mul_add(mat_b[l * cols_b + j], acc);
            }
            out[i * cols_b + j] = acc;
        }
    }
    out
}

fn mat_transpose_vec(mat: &[f64], vec_in: &[f64], rows: usize, cols: usize) -> Vec<f64> {
    let mut out = vec![0.0; cols];
    for i in 0..cols {
        let mut acc = 0.0;
        for l in 0..rows {
            acc = mat[l * cols + i].mul_add(vec_in[l], acc);
        }
        out[i] = acc;
    }
    out
}

fn cholesky_solve(mat_a: &[f64], rhs: &[f64], dim: usize) -> Vec<f64> {
    let mut low = vec![0.0_f64; dim * dim];
    for i in 0..dim {
        for j in 0..=i {
            let mut acc: f64 = 0.0;
            for k in 0..j {
                acc = low[i * dim + k].mul_add(low[j * dim + k], acc);
            }
            if i == j {
                low[i * dim + j] = (mat_a[i * dim + i] - acc).sqrt();
            } else {
                low[i * dim + j] = (mat_a[i * dim + j] - acc) / low[j * dim + j];
            }
        }
    }
    let mut y_vec = vec![0.0; dim];
    for i in 0..dim {
        let mut acc = 0.0;
        for j in 0..i {
            acc = low[i * dim + j].mul_add(y_vec[j], acc);
        }
        y_vec[i] = (rhs[i] - acc) / low[i * dim + i];
    }
    let mut x_vec = vec![0.0; dim];
    for i in (0..dim).rev() {
        let mut acc = 0.0;
        for j in (i + 1)..dim {
            acc = low[j * dim + i].mul_add(x_vec[j], acc);
        }
        x_vec[i] = (y_vec[i] - acc) / low[i * dim + i];
    }
    x_vec
}

fn main() {
    println!("=== validate-metalforge-spectral ===\n");
    println!("Exp 021: Spectral function reconstruction (Bazavov 2025)");
    println!("Tikhonov regularization — CPU Cholesky vs dispatched solver\n");

    let mut harness = Harness::new();
    run_spectral_checks(&mut harness);
    harness.finish();
}
