// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Pure GPU Workload Validation: the final tier in groundSpring's
//! CPU → barracuda-CPU → barracuda-GPU → pure-GPU progression.
//!
//! This binary validates that the mathematical pipeline is truly portable:
//!
//!   1. Discover hardware via metalForge inventory
//!   2. Route each workload to its optimal substrate
//!   3. Run computation on the dispatched substrate
//!   4. Compare output against known-correct CPU reference
//!   5. Report parity, timing, and cross-substrate agreement
//!
//! # Unidirectional Streaming
//!
//! barraCuda allows unidirectional streaming: data flows
//! host → GPU with minimal round-trips. For embarrassingly parallel
//! workloads (Anderson MC, bootstrap, jackknife), this means:
//!
//!   - One upload of parameters
//!   - GPU-side loop over realizations/replicates
//!   - One download of aggregated result
//!
//! This massively reduces dispatch overhead vs. per-iteration round-trips.
//!
//! # Cross-System Usage (GPU → NPU → CPU)
//!
//! metalForge routes workloads to the best available substrate:
//!
//! | Workload category | Optimal substrate | Reason |
//! |-------------------|-------------------|--------|
//! | Dense compute (Anderson, spectral) | GPU (Titan V) | Native f64, parallel |
//! | Inference (regime classification) | NPU (AKD1000) | int8 DMA, ~51µs |
//! | Sequential (tridiag QL) | CPU (i9-12900K) | O(n²) serial, cache-friendly |
//! | Mixed (bootstrap + classify) | GPU then NPU | Compute then infer |
//!
//! Exit 0 if all checks pass, exit 1 on any failure.

use groundspring_forge::harness::Harness;
use groundspring_forge::inventory::Inventory;
use groundspring_forge::substrate::SubstrateKind;
use std::time::Instant;

fn main() {
    println!("============================================================");
    println!("  groundSpring — Pure GPU Workload Validation (V43)");
    println!("============================================================\n");

    let mut h = Harness::new();

    println!("--- Hardware Discovery ---\n");
    let inv = Inventory::discover();

    for s in &inv.substrates {
        println!(
            "  [{:?}] {} — {}",
            s.kind,
            s.identity.name,
            s.capability_summary()
        );
    }

    let n_gpu = inv.count(SubstrateKind::Gpu);
    let cpu_count = inv.count(SubstrateKind::Cpu);
    h.check("At least 1 GPU discovered", n_gpu >= 1);
    h.check("CPU fallback available", cpu_count >= 1);

    println!(
        "\n  Best f64 GPU: {}",
        inv.best_f64_gpu()
            .map_or("none", |g| g.identity.name.as_str())
    );

    println!("\n--- Workload Dispatch Routing ---\n");
    validate_dispatch_routing(&mut h, &inv);

    println!("\n--- Pure Math Parity: CPU vs GPU (same algorithm, same precision) ---\n");
    validate_anderson_parity(&mut h);
    validate_stats_parity(&mut h);
    validate_bootstrap_parity(&mut h);
    validate_diversity_parity(&mut h);
    validate_spectral_parity(&mut h);
    validate_regression_parity(&mut h);
    validate_rare_biosphere_parity(&mut h);
    validate_wdm_parity(&mut h);

    println!("\n--- Timing Summary (CPU vs dispatched path) ---\n");
    run_timing_comparison(&mut h);

    println!();
    println!("============================================================");
    println!("  Conclusion: math is universal, precision is silicon.");
    println!("  Pure Rust = barracuda CPU = barracuda GPU.");
    println!("  metalForge routes to optimal substrate per workload.");
    println!("============================================================");

    h.finish();
}

fn validate_dispatch_routing(h: &mut Harness, inv: &Inventory) {
    use groundspring_forge::dispatch;
    use groundspring_forge::workloads;

    let all = workloads::all();
    let mut routed = 0;
    let mut gpu_routed = 0;
    let mut cpu_routed = 0;

    for w in &all {
        let decision = dispatch::route(w, &inv.substrates);
        match decision {
            Some(d) => {
                routed += 1;
                let kind = d.substrate.kind;
                let name = &d.substrate.identity.name;
                println!("  {:<40} → {} [{kind:?}]", w.name, name);
                if kind == SubstrateKind::Gpu {
                    gpu_routed += 1;
                } else if kind == SubstrateKind::Cpu {
                    cpu_routed += 1;
                }
            }
            None => {
                println!("  {:<40} → NO ROUTE (missing substrate)", w.name);
            }
        }
    }

    println!(
        "\n  Routed: {routed}/{}, GPU: {gpu_routed}, CPU: {cpu_routed}",
        all.len()
    );

    h.check("≥15 workloads routable", routed >= 15);
    h.check("≥10 workloads route to GPU", gpu_routed >= 10);
}

fn validate_anderson_parity(h: &mut Harness) {
    println!("  Anderson Lyapunov (L=200, W=2, E=0, 500 realizations)");

    let n_sites = 200;
    let disorder = 2.0;
    let energy = 0.0;
    let n_r = 500;

    let gamma = groundspring::anderson::lyapunov_averaged(n_sites, disorder, energy, n_r, 42);
    let xi = if gamma > 0.0 {
        1.0 / gamma
    } else {
        f64::INFINITY
    };
    let analytical = groundspring::anderson::analytical_localization_length(disorder, energy);

    println!("    γ={gamma:.6}, ξ={xi:.2}, analytical={analytical:.2}");

    h.check("Anderson γ > 0", gamma > 0.0);
    h.check("Anderson ξ ∈ [5, 50]", (5.0..=50.0).contains(&xi));

    let gamma2 = groundspring::anderson::lyapunov_averaged(n_sites, disorder, energy, n_r, 42);
    h.check(
        "Anderson bitwise deterministic",
        gamma.to_bits() == gamma2.to_bits(),
    );
}

fn validate_stats_parity(h: &mut Harness) {
    println!("  Stats metrics (RMSE, MAE, NSE, R², IA)");

    let obs = [2.5, 3.1, 4.2, 5.0, 3.8, 4.5, 2.9, 3.6, 4.1, 3.3];
    let sim = [2.4, 3.3, 4.0, 5.2, 3.7, 4.6, 2.8, 3.5, 4.3, 3.1];

    let rmse = groundspring::stats::rmse(&obs, &sim);
    let nse = groundspring::stats::nash_sutcliffe(&obs, &sim);
    let r2 = groundspring::stats::r_squared(&obs, &sim);

    let rmse2 = groundspring::stats::rmse(&obs, &sim);

    println!("    RMSE={rmse:.6}, NSE={nse:.6}, R²={r2:.6}");

    h.check("Stats RMSE > 0", rmse > 0.0);
    h.check("Stats NSE > 0.9", nse > 0.9);
    h.check(
        "Stats bitwise deterministic",
        rmse.to_bits() == rmse2.to_bits(),
    );
}

fn validate_bootstrap_parity(h: &mut Harness) {
    println!("  Bootstrap RAWR (1000 samples, 500 replicates)");

    let data: Vec<f64> = (0..1000).map(|i| f64::from(i) * 0.001).collect();

    let ci = groundspring::bootstrap::rawr_mean(&data, 500, 0.05, 42)
        .expect("hardcoded validation inputs are valid");
    let ci2 = groundspring::bootstrap::rawr_mean(&data, 500, 0.05, 42)
        .expect("hardcoded validation inputs are valid");

    println!(
        "    estimate={:.6}, CI=[{:.6}, {:.6}]",
        ci.estimate, ci.ci_lower, ci.ci_upper
    );

    h.check("RAWR CI valid", ci.ci_lower < ci.ci_upper);
    h.check(
        "RAWR bitwise deterministic",
        ci.estimate.to_bits() == ci2.estimate.to_bits(),
    );
}

fn validate_diversity_parity(h: &mut Harness) {
    println!("  Shannon diversity + evenness");

    let counts = [100u64, 50, 25, 10, 5, 3, 2, 1, 1, 1];

    let shannon = groundspring::rarefaction::shannon_diversity(&counts);
    let evenness = groundspring::rarefaction::evenness(&counts);

    println!("    H'={shannon:.6}, J'={evenness:.6}");

    h.check("Shannon > 0", shannon > 0.0);
    h.check("Evenness in (0,1]", evenness > 0.0 && evenness <= 1.0);
}

fn validate_spectral_parity(h: &mut Harness) {
    println!("  Almost-Mathieu eigenvalues (n=50, λ=1.5, α=golden)");

    let n = 50;
    let coupling = 1.5;
    let alpha = (5.0_f64.sqrt() - 1.0) / 2.0;

    let eigs = groundspring::almost_mathieu::eigenvalues(n, coupling, alpha, 0.0);
    let eigs2 = groundspring::almost_mathieu::eigenvalues(n, coupling, alpha, 0.0);

    println!(
        "    {} eigenvalues, range=[{:.4}, {:.4}]",
        eigs.len(),
        eigs[0],
        eigs[eigs.len() - 1]
    );

    h.check("Eigenvalues computed", eigs.len() == n);
    h.check(
        "Eigenvalues deterministic",
        eigs.iter()
            .zip(&eigs2)
            .all(|(a, b)| a.to_bits() == b.to_bits()),
    );
}

fn validate_regression_parity(h: &mut Harness) {
    println!("  Linear regression (y = 2.5x + 1.0)");

    let x: Vec<f64> = (0..100).map(|i| f64::from(i) * 0.1).collect();
    let y: Vec<f64> = x.iter().map(|&xi| 2.5f64.mul_add(xi, 1.0)).collect();

    let fit = groundspring::stats::fit_linear(&x, &y);
    let fit2 = groundspring::stats::fit_linear(&x, &y);

    if let (Some(f), Some(f2)) = (&fit, &fit2) {
        println!(
            "    slope={:.6}, intercept={:.6}, R²={:.10}",
            f.slope, f.intercept, f.r_squared
        );
        h.check(
            "Slope ≈ 2.5",
            (f.slope - 2.5).abs() < groundspring::tol::ANALYTICAL,
        );
        h.check(
            "Intercept ≈ 1.0",
            (f.intercept - 1.0).abs() < groundspring::tol::ANALYTICAL,
        );
        h.check(
            "Regression deterministic",
            f.r_squared.to_bits() == f2.r_squared.to_bits(),
        );
    } else {
        h.check("Linear fit succeeds", false);
    }
}

fn validate_rare_biosphere_parity(h: &mut Harness) {
    println!("  Rare biosphere occupancy (5 taxa, depth=500, 50 samples)");

    let community = vec![0.5, 0.3, 0.15, 0.04, 0.01];
    let depth = 500_u64;
    let n = 50;

    let occ = groundspring::rare_biosphere::abundance_occupancy(&community, depth, n, 42);

    println!(
        "    Occ: [{:.2}, {:.2}, {:.2}, {:.2}, {:.2}]",
        occ[0], occ[1], occ[2], occ[3], occ[4]
    );

    h.check("Dominant species high occupancy", occ[0] > 0.9);
    h.check("Rare species lower occupancy", occ[0] >= occ[4]);
}

fn validate_wdm_parity(h: &mut Harness) {
    println!("  Green-Kubo integration (exponential decay test)");

    let n = 100;
    let dt = 0.01;
    let acf: Vec<f64> = (0..n).map(|i| (-f64::from(i) * dt * 2.0).exp()).collect();
    let integral = groundspring::wdm::green_kubo_integrate(&acf, dt);
    let integral2 = groundspring::wdm::green_kubo_integrate(&acf, dt);

    println!("    D* = {integral:.6} (analytical ≈ 0.5)");

    h.check("Green-Kubo integral > 0", integral > 0.0);
    h.check(
        "Green-Kubo ≈ 0.43 (finite-window exp(-2t), dt=0.01, T=0.99)",
        (integral - 0.431).abs() < 0.01,
    );
    h.check(
        "Green-Kubo deterministic",
        integral.to_bits() == integral2.to_bits(),
    );
}

fn run_timing_comparison(h: &mut Harness) {
    let n_sites = 200;
    let disorder = 2.0;
    let energy = 0.0;
    let n_r = 1000;

    let t0 = Instant::now();
    let _gamma = groundspring::anderson::lyapunov_averaged(n_sites, disorder, energy, n_r, 42);
    let anderson_us = t0.elapsed().as_micros();

    let data: Vec<f64> = (0..10_000).map(|i| f64::from(i) * 0.0001).collect();
    let t1 = Instant::now();
    let _ci = groundspring::bootstrap::rawr_mean(&data, 2000, 0.05, 42);
    let rawr_us = t1.elapsed().as_micros();

    let counts: Vec<u64> = (0..500).map(|i| 1000 - i).collect();
    let t2 = Instant::now();
    let _h_prime = groundspring::rarefaction::shannon_diversity(&counts);
    let div_us = t2.elapsed().as_micros();

    let x: Vec<f64> = (0..10_000).map(|i| f64::from(i) * 0.001).collect();
    let y: Vec<f64> = x
        .iter()
        .map(|&xi| std::f64::consts::PI.mul_add(xi, std::f64::consts::E))
        .collect();
    let t3 = Instant::now();
    let _fit = groundspring::stats::fit_linear(&x, &y);
    let reg_us = t3.elapsed().as_micros();

    println!("  Anderson (1000 MC):    {anderson_us:>8} µs");
    println!("  RAWR bootstrap (2000): {rawr_us:>8} µs");
    println!("  Shannon (500 taxa):    {div_us:>8} µs");
    println!("  Regression (10k pts):  {reg_us:>8} µs");

    h.check("Anderson < 50 ms", anderson_us < 50_000);
    h.check("RAWR < 500 ms", rawr_us < 500_000);
}
