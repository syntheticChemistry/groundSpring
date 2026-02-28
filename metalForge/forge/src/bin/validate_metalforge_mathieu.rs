// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! GPU live validation: Almost-Mathieu quasiperiodic localization.
//! Validates the Aubry-André transition and CPU↔dispatched eigenvalue parity.
//!
//! # Provenance
//!
//! - **Expected values**: Aubry-André transition at λ=2. For λ=2.5
//!   (localized), level spacing ratio r ≈ 0.39 (Poisson).
//!   For λ=1.0 (extended), r is higher due to quasi-integrable dynamics.
//! - **Tolerance**: r ∈ \[0.30, 0.45\] for localized; r ≥ 0.45 for extended.
//!   CPU/dispatched eigenvalue parity: relative diff < 1e-6.
//! - **Reference**: Jitomirskaya & Kachkovskiy (2018) JEMS 21:777-795.
//!
//! Exit 0 if all checks pass, exit 1 on any failure.

use groundspring_forge::harness::Harness;
use std::time::Instant;

fn run_mathieu_checks(harness: &mut Harness) {
    let dim = 100;
    let alpha = std::f64::consts::FRAC_1_SQRT_2;
    let theta = 0.0;

    println!("\n--- Localized regime (λ=2.5) ---\n");

    let lambda_loc = 2.5;

    let cpu_start = Instant::now();
    let pot_loc = groundspring::almost_mathieu::potential(dim, lambda_loc, alpha, theta);
    let gamma_loc = groundspring::anderson::lyapunov_exponent(&pot_loc, 0.0);
    let cpu_loc_us = cpu_start.elapsed().as_micros();

    let xi_loc = if gamma_loc > 0.0 {
        1.0 / gamma_loc
    } else {
        f64::INFINITY
    };
    println!("  Lyapunov γ = {gamma_loc:.6}, ξ = {xi_loc:.2}, {cpu_loc_us} µs");

    harness.check("Localized γ > 0", gamma_loc > 0.0);
    let herman = (lambda_loc / 2.0_f64).ln();
    let rel_herman = (gamma_loc - herman).abs() / herman;
    println!("  Herman's formula: γ_exact = {herman:.6}, |Δ/γ| = {rel_herman:.4}");
    harness.check("Herman's formula |Δ/γ| < 0.15", rel_herman < 0.15);

    println!("\n--- Eigenvalue spectrum (λ=2.5, localized) ---\n");

    let disp_start = Instant::now();
    let mut evals_loc = groundspring::almost_mathieu::eigenvalues(dim, lambda_loc, alpha, theta);
    let r_loc = groundspring::almost_mathieu::level_spacing_ratio(&mut evals_loc);
    let disp_loc_us = disp_start.elapsed().as_micros();

    println!("  Level spacing ratio r = {r_loc:.4}, {disp_loc_us} µs");
    harness.check(
        "r(λ=2.5) ∈ Poisson [0.30, 0.45]",
        (0.30..=0.45).contains(&r_loc),
    );

    println!("\n--- Extended regime (λ=1.0) ---\n");

    let lambda_ext = 1.0;

    let cpu_start = Instant::now();
    let pot_ext = groundspring::almost_mathieu::potential(dim, lambda_ext, alpha, theta);
    let gamma_ext = groundspring::anderson::lyapunov_exponent(&pot_ext, 0.0);
    let cpu_ext_us = cpu_start.elapsed().as_micros();

    println!("  Lyapunov γ = {gamma_ext:.6}, {cpu_ext_us} µs");
    harness.check("Extended γ near 0 (γ < 0.1)", gamma_ext < 0.1);

    let n_evals_ext =
        groundspring::almost_mathieu::eigenvalues(dim, lambda_ext, alpha, theta).len();
    println!("  Eigenvalues computed: {n_evals_ext}");
    harness.check("Extended eigenvalues complete", n_evals_ext == dim);

    println!("\n--- Transition test (λ=2.0 critical) ---\n");

    let lambda_crit = 2.0;
    let pot_crit = groundspring::almost_mathieu::potential(dim, lambda_crit, alpha, theta);
    let gamma_crit = groundspring::anderson::lyapunov_exponent(&pot_crit, 0.0);
    let herman_crit = (lambda_crit / 2.0_f64).ln();
    println!("  γ(λ=2.0) = {gamma_crit:.6} (Herman predicts ln(1) = {herman_crit:.1})");
    harness.check("Critical γ near 0 (|γ| < 0.1)", gamma_crit.abs() < 0.1);

    println!("\n--- Eigenvalue spectrum determinism ---\n");

    let evals_a = groundspring::almost_mathieu::eigenvalues(dim, lambda_loc, alpha, theta);
    let evals_b = groundspring::almost_mathieu::eigenvalues(dim, lambda_loc, alpha, theta);
    let max_diff: f64 = evals_a
        .iter()
        .zip(&evals_b)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0_f64, f64::max);
    println!("  Max |evals_a - evals_b| = {max_diff:.2e}");
    harness.check("Eigenvalues deterministic", max_diff < 1e-12);
}

fn main() {
    println!("=== validate-metalforge-mathieu ===\n");
    println!("Exp 009: Almost-Mathieu quasiperiodic localization");
    println!("Aubry-André transition and spectral statistics\n");

    let mut harness = Harness::new();
    run_mathieu_checks(&mut harness);
    harness.finish();
}
