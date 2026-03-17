// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Spectral theory and linear algebra parity checks: Anderson localization,
//! Almost-Mathieu, Tikhonov reconstruction, eigendecomposition, band structure,
//! and PRNG stream determinism.

use groundspring::spectral_recon::{LAMBDA_NOISY, LAMBDA_PARITY};
use groundspring::tol;
use groundspring_forge::harness::Harness;
use std::time::Instant;

/// Run all spectral-domain parity checks.
pub fn validate_all(h: &mut Harness) {
    validate_anderson_spectral_parity(h);
    validate_almost_mathieu_parity(h);
    validate_spectral_recon_parity(h);
    validate_cholesky_gpu_parity(h);
    validate_tridiag_eigh_parity(h);
    validate_band_structure_parity(h);
    validate_prng_stream_parity(h);
}

fn validate_anderson_spectral_parity(h: &mut Harness) {
    println!("\n--- Anderson Localization Parity (spectral-localization lineage S26) ---\n");

    let n_sites = 200;
    let disorder = 2.0;
    let energy = 0.0;

    let t0 = Instant::now();
    let gamma1 = groundspring::anderson::lyapunov_averaged(n_sites, disorder, energy, 100, 42);
    let us1 = t0.elapsed().as_micros();

    let t1 = Instant::now();
    let gamma2 = groundspring::anderson::lyapunov_averaged(n_sites, disorder, energy, 100, 42);
    let us2 = t1.elapsed().as_micros();

    let xi = if gamma1 > 0.0 {
        1.0 / gamma1
    } else {
        f64::INFINITY
    };
    let analytical = groundspring::anderson::analytical_localization_length(disorder, energy);

    println!("  γ={gamma1:.6}, ξ={xi:.2}, analytical_ξ={analytical:.2}");
    println!("  Run 1: {us1} µs, Run 2: {us2} µs");

    h.check("γ > 0 (localized)", gamma1 > 0.0);
    h.check("ξ ∈ [5, 50]", (5.0..=50.0).contains(&xi));
    h.check("Analytical ξ > 0", analytical > 0.0);
    h.check(
        "Deterministic (bitwise)",
        gamma1.to_bits() == gamma2.to_bits(),
    );
}

fn validate_almost_mathieu_parity(h: &mut Harness) {
    println!("\n--- Almost-Mathieu Parity (spectral-quasiperiodic lineage S26) ---\n");

    let n = 50;
    let coupling = 1.5;
    let alpha = (5.0_f64.sqrt() - 1.0) / 2.0;
    let theta = 0.0;

    let ham = groundspring::almost_mathieu::hamiltonian(n, coupling, alpha, theta);
    h.check("Hamiltonian dimension n²", ham.len() == n * n);

    let eigenvalues = groundspring::almost_mathieu::eigenvalues(n, coupling, alpha, theta);
    let eigenvalues2 = groundspring::almost_mathieu::eigenvalues(n, coupling, alpha, theta);

    h.check(
        "Eigenvalues deterministic",
        eigenvalues
            .iter()
            .zip(&eigenvalues2)
            .all(|(a, b)| a.to_bits() == b.to_bits()),
    );

    let mut ev_for_r = eigenvalues;
    let r = groundspring::almost_mathieu::level_spacing_ratio(&mut ev_for_r);

    println!("  n={n}, λ={coupling}, α=golden, <r>={r:.4}");

    h.check("Level spacing ratio > 0", r > 0.0 && r < 1.0);
}

fn validate_spectral_recon_parity(h: &mut Harness) {
    println!("\n--- Spectral Reconstruction Parity (spectral-recon lineage S39) ---\n");

    let n_tau: u32 = 20;
    let n_omega: u32 = 30;
    let kernel: Vec<f64> = (0..n_tau * n_omega)
        .map(|idx| {
            let tau = f64::from(idx / n_omega) * 0.05;
            let omega = f64::from(idx % n_omega) * 0.5;
            (-tau * omega).exp()
        })
        .collect();
    let g: Vec<f64> = (0..n_tau).map(|i| (-f64::from(i) * 0.1).exp()).collect();
    let n_tau = n_tau as usize;
    let n_omega = n_omega as usize;

    let rho =
        groundspring::spectral_recon::tikhonov_solve(&kernel, &g, LAMBDA_NOISY, n_tau, n_omega);
    let rho2 =
        groundspring::spectral_recon::tikhonov_solve(&kernel, &g, LAMBDA_NOISY, n_tau, n_omega);

    println!("  n_tau={n_tau}, n_omega={n_omega}, rho[0]={:.6}", rho[0]);

    h.check("Spectral function non-empty", !rho.is_empty());
    h.check(
        "Spectral function has values",
        rho.iter().any(|&r| r.abs() > 0.0),
    );
    h.check(
        "Tikhonov deterministic",
        rho.iter()
            .zip(&rho2)
            .all(|(a, b)| a.to_bits() == b.to_bits()),
    );
}

fn validate_cholesky_gpu_parity(h: &mut Harness) {
    println!("\n--- Cholesky GPU Parity (Phase 2b) ---\n");

    let n_tau = 15_usize;
    let n_omega = 20_usize;
    let tau: Vec<f64> = (1..=15_u32).map(|i| f64::from(i) * 2.0 / 15.0).collect();
    let omega: Vec<f64> = (1..=20_u32).map(|i| f64::from(i) * 6.0 / 20.0).collect();
    let rho_true = groundspring::spectral_recon::gaussian_peak(&omega, 2.5, 0.4, 1.0);
    let kernel = groundspring::spectral_recon::build_kernel(&tau, &omega);
    let g = groundspring::spectral_recon::forward_correlator(&kernel, &rho_true, n_tau, n_omega);

    let t0 = Instant::now();
    let rho_rec =
        groundspring::spectral_recon::tikhonov_solve(&kernel, &g, LAMBDA_PARITY, n_tau, n_omega);
    let us = t0.elapsed().as_micros();

    let g_rec = groundspring::spectral_recon::forward_correlator(&kernel, &rho_rec, n_tau, n_omega);
    let rmse = groundspring::spectral_recon::rmse(&g, &g_rec);
    let peak = groundspring::spectral_recon::peak_index(&rho_rec);

    println!("  RMSE={rmse:.2e}, peak_ω={:.2}, {us} µs", omega[peak]);

    h.check("Cholesky RMSE < RECONSTRUCTION", rmse < tol::RECONSTRUCTION);
    h.check("Cholesky peak near 2.5", (omega[peak] - 2.5).abs() < 1.0);

    let rho2 =
        groundspring::spectral_recon::tikhonov_solve(&kernel, &g, LAMBDA_PARITY, n_tau, n_omega);
    h.check(
        "Cholesky deterministic",
        rho_rec
            .iter()
            .zip(&rho2)
            .all(|(a, b)| a.to_bits() == b.to_bits()),
    );
}

fn validate_tridiag_eigh_parity(h: &mut Harness) {
    println!("\n--- Tridiag Eigh Parity (Phase 2b) ---\n");

    let n = 50;
    let diag: Vec<f64> = (0..50_i32).map(|i| f64::from(i) * 0.3).collect();
    let offdiag = vec![1.0; n - 1];

    let t0 = Instant::now();
    let (vals, vecs) =
        groundspring::transport::tridiag_eigh(&diag, &offdiag).expect("eigendecomposition");
    let us = t0.elapsed().as_micros();

    println!(
        "  n={n}, λ_min={:.6}, λ_max={:.6}, {us} µs",
        vals[0],
        vals[n - 1]
    );

    h.check(
        "Eigenvalues ascending",
        vals.windows(2).all(|w| w[0] <= w[1]),
    );

    let norm: f64 = (0..n).map(|row| vecs[row * n] * vecs[row * n]).sum();
    h.check(
        "First eigenvector normalized",
        (norm - 1.0).abs() < tol::INTEGRATION,
    );

    for k in 0..3 {
        let mut hv = vec![0.0; n];
        for j in 0..n {
            hv[j] += diag[j] * vecs[j * n + k];
            if j > 0 {
                hv[j] += offdiag[j - 1] * vecs[(j - 1) * n + k];
            }
            if j + 1 < n {
                hv[j] += offdiag[j] * vecs[(j + 1) * n + k];
            }
        }
        let residual: f64 = (0..n)
            .map(|j| vals[k].mul_add(-vecs[j * n + k], hv[j]).powi(2))
            .sum::<f64>()
            .sqrt();
        h.check(
            &format!("Eigenpair {k} residual < INTEGRATION"),
            residual < tol::INTEGRATION,
        );
    }

    let (vals2, _) =
        groundspring::transport::tridiag_eigh(&diag, &offdiag).expect("eigendecomposition");
    h.check(
        "Tridiag eigh deterministic",
        vals.iter()
            .zip(&vals2)
            .all(|(a, b)| a.to_bits() == b.to_bits()),
    );
}

fn validate_band_structure_parity(h: &mut Harness) {
    println!("\n--- Band Structure Parity (spectral-band lineage S26) ---\n");

    let n_periods = 20;
    let potential = &[0.5, -0.5];
    let hopping = 1.0;

    let (diag, offdiag) =
        groundspring::band_structure::periodic_hamiltonian(potential, hopping, n_periods);
    let (eigenvalues, _) =
        groundspring::transport::tridiag_eigh(&diag, &offdiag).expect("eigendecomposition");
    let bands = groundspring::band_structure::detect_band_ranges(&eigenvalues, 2.0);
    // Band-edge matching tolerance: 0.05 accounts for finite grid
    // discretisation (n_periods=20, 2 points per period → Δk ≈ 0.05).
    let frac = groundspring::band_structure::eigenvalue_band_fraction(
        &eigenvalues,
        potential,
        hopping,
        0.05,
    );

    println!(
        "  {} eigenvalues, {} bands, {:.1}% in bands",
        eigenvalues.len(),
        bands.len(),
        frac * 100.0
    );

    h.check("Eigenvalues computed", !eigenvalues.is_empty());
    h.check("At least 1 band detected", !bands.is_empty());
    // 95% threshold: periodic Bloch theory guarantees all eigenvalues lie in
    // bands; 5% slack accommodates finite-chain edge effects (N=20 periods).
    h.check("≥95% eigenvalues within bands", frac >= 0.95);

    let (eigenvalues2, _) =
        groundspring::transport::tridiag_eigh(&diag, &offdiag).expect("eigendecomposition");
    h.check(
        "Band structure deterministic",
        eigenvalues
            .iter()
            .zip(&eigenvalues2)
            .all(|(a, b)| a.to_bits() == b.to_bits()),
    );
}

fn validate_prng_stream_parity(h: &mut Harness) {
    println!("\n--- PRNG Stream Parity (Phase 2b) ---\n");

    let xorshift_deterministic = {
        let mut left = groundspring::prng::Xorshift64::new(42);
        let mut right = groundspring::prng::Xorshift64::new(42);
        (0..1000).all(|_| left.next_u64() == right.next_u64())
    };
    h.check("Xorshift64 stream deterministic", xorshift_deterministic);

    let xoshiro_deterministic = {
        let mut left = groundspring::prng::Xoshiro128StarStar::new(42);
        let mut right = groundspring::prng::Xoshiro128StarStar::new(42);
        (0..1000).all(|_| left.next_u32() == right.next_u32())
    };
    h.check("Xoshiro128** stream deterministic", xoshiro_deterministic);

    let mut rng = groundspring::prng::Xoshiro128StarStar::new(42);
    let vals: Vec<f64> = (0..10_000).map(|_| rng.next_f64()).collect();
    let all_unit = vals.iter().all(|&v| (0.0..1.0).contains(&v));
    h.check("Xoshiro f64 in [0,1)", all_unit);

    let mean: f64 = vals.iter().sum::<f64>() / 10_000.0;
    println!("  Xoshiro mean={mean:.6} (expected ~0.5)");
    // 0.02 tolerance: E[U(0,1)] = 0.5, σ = 1/√12, SE = σ/√N ≈ 0.0029 for
    // N=10000. Threshold ≈ 7 SE — generous for deterministic check.
    h.check("Xoshiro mean near 0.5", (mean - 0.5).abs() < 0.02);

    let mut xor = groundspring::prng::DefaultRng::new(42);
    let mut gpu = groundspring::prng::GpuAlignedRng::new(42);
    let xor_val = xor.next_f64();
    let gpu_val = gpu.next_f64();
    h.check(
        "DefaultRng and GpuAlignedRng both produce valid f64",
        (0.0..1.0).contains(&xor_val) && (0.0..1.0).contains(&gpu_val),
    );
}
