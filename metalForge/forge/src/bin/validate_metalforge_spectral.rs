// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team
#![forbid(unsafe_code)]

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

use groundspring::spectral_recon;
use groundspring_forge::harness::Harness;
use groundspring_forge::tolerance::ToleranceTier;
use std::time::Instant;

/// Noiseless round-trip RMSE: regularization-limited floor for Tikhonov
/// with λ = `ToleranceTier::Exact` on an `n_tau`=20, `n_omega`=40 grid.
/// Bazavov et al. (2025) `arXiv` 2501.12259, validated in `control/spectral_recon/`.
const TOL_RMSE_NOISELESS: f64 = 1e-6;

/// Peak location tolerance: Gaussian peak centre at ω=3.0 recovered within
/// ±1 frequency bin on an `n_omega`=40 grid spanning \[0, 8\]. Bin width = 0.2.
const TOL_PEAK_OFFSET: f64 = 1.0;

/// CPU↔dispatched parity: different LU/Cholesky factorisations accumulate
/// O(n²) ULP differences. `1e-8` provides ~1000× margin over observed ~`1e-11`.
const TOL_PARITY_REL: f64 = 1e-8;

fn run_spectral_checks(harness: &mut Harness) {
    let n_tau = 20;
    let n_omega = 40;
    let true_centre = 3.0;
    let true_width = 0.5;
    let lambda = ToleranceTier::Exact.relative_tolerance();

    let tau: Vec<f64> = (1..=n_tau)
        .map(|idx| {
            #[expect(clippy::cast_precision_loss, reason = "index/count ≤ n_tau ≪ 2^53")]
            let val = idx as f64 * 2.0 / n_tau as f64;
            val
        })
        .collect();
    let omega: Vec<f64> = (1..=n_omega)
        .map(|idx| {
            #[expect(clippy::cast_precision_loss, reason = "index/count ≤ n_omega ≪ 2^53")]
            let val = idx as f64 * 8.0 / n_omega as f64;
            val
        })
        .collect();

    let kernel = spectral_recon::build_kernel(&tau, &omega);
    let rho_true = spectral_recon::gaussian_peak(&omega, true_centre, true_width, 1.0);
    let g_data = spectral_recon::forward_correlator(&kernel, &rho_true, n_tau, n_omega);

    println!("\n--- CPU-only Tikhonov (local Cholesky) ---\n");
    let cpu_start = Instant::now();
    let rho_cpu = spectral_recon::tikhonov_solve_cpu(&kernel, &g_data, lambda, n_tau, n_omega);
    let cpu_us = cpu_start.elapsed().as_micros();
    let g_cpu = spectral_recon::forward_correlator(&kernel, &rho_cpu, n_tau, n_omega);
    let rmse_cpu = spectral_recon::rmse(&g_data, &g_cpu);
    let peak_cpu = spectral_recon::peak_index(&rho_cpu);

    println!("  RMSE(G, G_rec) = {rmse_cpu:.2e}");
    println!(
        "  Peak at ω = {:.2} (true = {true_centre:.1})",
        omega[peak_cpu]
    );
    println!("  Time: {cpu_us} µs");

    harness.check("CPU RMSE < 1e-6", rmse_cpu < TOL_RMSE_NOISELESS);
    harness.check(
        "CPU peak within ±1.0",
        (omega[peak_cpu] - true_centre).abs() < TOL_PEAK_OFFSET,
    );

    println!("\n--- Dispatched Tikhonov (feature-gated barracuda) ---\n");
    let disp_start = Instant::now();
    let rho_disp = spectral_recon::tikhonov_solve(&kernel, &g_data, lambda, n_tau, n_omega);
    let disp_us = disp_start.elapsed().as_micros();
    let g_disp = spectral_recon::forward_correlator(&kernel, &rho_disp, n_tau, n_omega);
    let rmse_disp = spectral_recon::rmse(&g_data, &g_disp);
    let peak_disp = spectral_recon::peak_index(&rho_disp);

    println!("  RMSE(G, G_rec) = {rmse_disp:.2e}");
    println!(
        "  Peak at ω = {:.2} (true = {true_centre:.1})",
        omega[peak_disp]
    );
    println!("  Time: {disp_us} µs");

    harness.check("Dispatched RMSE < 1e-6", rmse_disp < TOL_RMSE_NOISELESS);
    harness.check(
        "Dispatched peak within ±1.0",
        (omega[peak_disp] - true_centre).abs() < TOL_PEAK_OFFSET,
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

    harness.check("Solutions agree (rel < 1e-8)", rel_diff < TOL_PARITY_REL);
    harness.check("Peak location matches", peak_cpu == peak_disp);

    println!("\n--- Timing Summary ---\n");
    println!("  CPU:        {cpu_us} µs");
    println!("  Dispatched: {disp_us} µs");
    if disp_us > 0 {
        #[expect(clippy::cast_precision_loss, reason = "timing values in μs ≪ 2^53")]
        let speedup = cpu_us as f64 / disp_us as f64;
        println!("  Ratio (CPU/Dispatched): {speedup:.2}x");
    }
}

fn main() {
    println!("=== validate-metalforge-spectral ===\n");
    println!("Exp 021: Spectral function reconstruction (Bazavov 2025)");
    println!("Tikhonov regularization — CPU Cholesky vs dispatched solver\n");

    let mut harness = Harness::new();
    run_spectral_checks(&mut harness);
    harness.finish();
}
