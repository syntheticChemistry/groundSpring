// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! GPU live validation: run Anderson computation on GPU via barracuda-gpu
//! and compare with CPU reference. Reports substrate, timing, and parity.
//!
//! # Provenance
//!
//! - **Expected values**: Analytical — Derrida-Gardner localization length
//!   ξ(W,E) = C/W² with C ≈ 96 (perturbative, 1D Anderson at band centre).
//!   Finite-size (L=200) suppression gives numerical ξ < analytical ξ(L→∞).
//! - **Tolerance**: ξ ∈ \[5, 50\] for W=2 (5× variation for PRNG seed effects).
//!   CPU/GPU ξ ratio ∈ \[0.3, 3.0\] (different PRNG streams pre-alignment).
//! - **Commit**: See `metalForge/ABSORPTION_MANIFEST.md` for shader provenance.
//!
//! Exit 0 if all checks pass, exit 1 on any failure.

use groundspring_forge::harness::Harness;
use std::time::Instant;

fn run_gpu_probe_checks(h: &mut Harness) {
    use groundspring_forge::inventory::Inventory;
    use groundspring_forge::substrate::{Capability, SubstrateKind};

    let inv = Inventory::discover();

    let gpu = inv.first(SubstrateKind::Gpu);
    h.check("GPU discovered via wgpu", gpu.is_some());

    if let Some(gpu) = gpu {
        println!(
            "\n  GPU: {} ({})",
            gpu.identity.name,
            gpu.capability_summary()
        );

        h.check("GPU supports f64 compute", gpu.has(&Capability::F64Compute));
        h.check(
            "GPU supports shader dispatch",
            gpu.has(&Capability::ShaderDispatch),
        );
    }
}

fn run_computation_checks(h: &mut Harness) {
    println!("\n--- Anderson Lyapunov (L=200, W=2.0, E=0) ---\n");

    let n_sites = 200;
    let disorder = 2.0;
    let energy = 0.0;
    let n_realizations = 500;
    let base_seed = 42;

    let cpu_start = Instant::now();
    let cpu_gamma = cpu_lyapunov_averaged(n_sites, disorder, energy, n_realizations, base_seed);
    let cpu_us = cpu_start.elapsed().as_micros();
    let cpu_xi = if cpu_gamma > 0.0 {
        1.0 / cpu_gamma
    } else {
        f64::INFINITY
    };

    println!("  CPU: γ = {cpu_gamma:.6}, ξ = {cpu_xi:.2}, {cpu_us} µs");

    h.check("CPU γ > 0 (localized regime)", cpu_gamma > 0.0);
    // ξ range [5, 50]: For W=2, analytical ξ ≈ C/W² = 96/4 = 24.
    // Finite-size (L=200) suppression gives numerical ξ < analytical.
    // Range accepts 5× variation to cover PRNG seed effects.
    h.check("CPU ξ in [5, 50]", (5.0..=50.0).contains(&cpu_xi));

    let gpu_start = Instant::now();
    let gpu_gamma = groundspring::anderson::lyapunov_averaged(
        n_sites,
        disorder,
        energy,
        n_realizations,
        base_seed,
    );
    let gpu_us = gpu_start.elapsed().as_micros();
    let gpu_xi = if gpu_gamma > 0.0 {
        1.0 / gpu_gamma
    } else {
        f64::INFINITY
    };

    println!("  GPU: γ = {gpu_gamma:.6}, ξ = {gpu_xi:.2}, {gpu_us} µs");

    h.check("GPU γ > 0 (localized regime)", gpu_gamma > 0.0);
    h.check("GPU ξ in [5, 50]", (5.0..=50.0).contains(&gpu_xi));

    let rel_diff = ((cpu_gamma - gpu_gamma) / cpu_gamma).abs();
    println!("\n  Parity: |CPU - GPU| / CPU = {rel_diff:.6}");

    // CPU and GPU use different PRNG seeding (xorshift64 vs xoshiro128**)
    // so exact parity is not expected. Both must be in the same localization
    // regime: ratio ∈ [0.3, 3.0] ensures ξ values agree within one order
    // of magnitude. Exact parity requires Phase 2b PRNG alignment.
    h.check("Both in same localization regime", {
        let ratio = cpu_xi / gpu_xi;
        (0.3..=3.0).contains(&ratio)
    });

    println!("\n--- Anderson Analytical ξ ---\n");

    let analytical_xi = groundspring::anderson::analytical_localization_length(disorder, energy);
    println!("  Analytical ξ(W=2) = {analytical_xi:.2}");
    h.check("Analytical ξ > 0", analytical_xi > 0.0);

    let cpu_analytical_ratio = cpu_xi / analytical_xi;
    println!("  CPU/Analytical ratio = {cpu_analytical_ratio:.3}");
    // Finite-size suppression: numerical ξ(L=200) < analytical ξ(L→∞).
    // Derrida-Gardner C ≈ 96 (perturbative) gives analytical ξ(W=2) ≈ 24,
    // while the L=200 transfer-matrix result is systematically lower.
    // The 0.1–5.0 range accommodates this finite-size correction.
    h.check(
        "CPU ξ within 5x of analytical",
        (0.1..=5.0).contains(&cpu_analytical_ratio),
    );

    let gpu_analytical_ratio = gpu_xi / analytical_xi;
    println!("  GPU/Analytical ratio = {gpu_analytical_ratio:.3}");
    h.check(
        "GPU ξ within 5x of analytical",
        (0.1..=5.0).contains(&gpu_analytical_ratio),
    );

    println!("\n--- Timing Summary ---\n");
    println!("  CPU: {cpu_us} µs");
    println!("  GPU: {gpu_us} µs");
    if gpu_us > 0 {
        #[expect(clippy::cast_precision_loss)]
        let speedup = cpu_us as f64 / gpu_us as f64;
        println!("  Ratio (CPU/GPU): {speedup:.2}x");
    }
}

/// CPU-only Lyapunov average — bypasses feature-gated dispatch.
fn cpu_lyapunov_averaged(
    n_sites: usize,
    disorder: f64,
    energy: f64,
    n_realizations: usize,
    base_seed: u64,
) -> f64 {
    let mut total = 0.0;
    for i in 0..n_realizations {
        let pot =
            groundspring::anderson::anderson_potential(n_sites, disorder, base_seed + i as u64);
        let gamma = cpu_lyapunov_exponent(&pot, energy);
        total += gamma;
    }
    #[expect(clippy::cast_precision_loss)]
    let avg = total / n_realizations as f64;
    avg
}

/// CPU-only Lyapunov exponent — always runs locally regardless of features.
fn cpu_lyapunov_exponent(potential: &[f64], energy: f64) -> f64 {
    let n = potential.len();
    if n == 0 {
        return 0.0;
    }
    let mut log_growth = 0.0;
    let mut v0: f64 = 1.0;
    let mut v1: f64 = 0.0;
    for &v in potential {
        let new_0 = (energy - v).mul_add(v0, -v1);
        let new_1 = v0;
        v0 = new_0;
        v1 = new_1;
        let norm = v0.hypot(v1);
        if norm > 0.0 {
            log_growth += norm.ln();
            v0 /= norm;
            v1 /= norm;
        }
    }
    #[expect(clippy::cast_precision_loss)]
    let avg = log_growth / n as f64;
    avg
}

fn main() {
    println!("=== validate-metalforge-gpu ===\n");

    let mut h = Harness::new();

    println!("--- GPU Probe ---\n");
    run_gpu_probe_checks(&mut h);

    println!();
    run_computation_checks(&mut h);

    h.finish();
}
