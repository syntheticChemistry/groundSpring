// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team
#![forbid(unsafe_code)]

//! GPU head-to-head benchmark: `BarraCuda` WGSL vs Kokkos CUDA.
//!
//! Runs the identical algorithms, parameters, and PRNG seeds as
//! `kokkos_baseline/src/main.cpp` compiled with CUDA, but dispatched
//! through `BarraCuda`'s WGSL compute shaders via `wgpu`.
//!
//! Build:
//!     cargo run --release --features barracuda-gpu --bin bench-gpu-vs-kokkos

use std::time::Instant;

use groundspring::anderson::{anderson_potential, lyapunov_averaged, lyapunov_exponent};
use groundspring::bootstrap::bootstrap_mean;
use groundspring::prng::Xorshift64;
use groundspring::stats::{mean, pearson_r, std_dev};
use groundspring_validate::OrExit;

struct BenchResult {
    name: &'static str,
    value: f64,
    elapsed_us: f64,
}

fn bench_anderson(results: &mut Vec<BenchResult>) {
    const N_SITES: usize = 10_000;
    const DISORDER: f64 = 4.0;
    const N_REALIZATIONS: usize = 500;
    const ENERGY: f64 = 0.0;
    const BASE_SEED: u64 = 42;

    println!("=== Anderson Localization (Lyapunov Exponent) ===");
    println!("  N={N_SITES}, W={DISORDER}, realizations={N_REALIZATIONS}, E={ENERGY}");

    let t0 = Instant::now();
    let gamma_avg = lyapunov_averaged(N_SITES, DISORDER, ENERGY, N_REALIZATIONS, BASE_SEED);
    let elapsed = t0.elapsed().as_secs_f64() * 1e6;

    let xi = if gamma_avg > 0.0 {
        1.0 / gamma_avg
    } else {
        f64::INFINITY
    };
    println!("  gamma_avg = {gamma_avg:.10}  (xi = {xi:.4})");
    println!(
        "  Derrida-Gardner: xi ~ 96/W^2 = {:.4}",
        96.0 / (DISORDER * DISORDER)
    );
    println!("  elapsed: {elapsed:.0} us\n");
    results.push(BenchResult {
        name: "anderson_lyapunov_averaged",
        value: gamma_avg,
        elapsed_us: elapsed,
    });

    let pot = anderson_potential(N_SITES, DISORDER, BASE_SEED);
    let t0 = Instant::now();
    let gamma_single = lyapunov_exponent(&pot, ENERGY);
    let elapsed_single = t0.elapsed().as_secs_f64() * 1e6;
    println!("  single-realization gamma = {gamma_single:.10}  ({elapsed_single:.0} us)");
    results.push(BenchResult {
        name: "anderson_lyapunov_single",
        value: gamma_single,
        elapsed_us: elapsed_single,
    });
}

fn bench_stats(results: &mut Vec<BenchResult>) {
    const N: usize = 1_000_000;
    const SEED: u64 = 12345;

    println!("\n=== Statistical Reductions (N={N}) ===");

    let mut data = Vec::with_capacity(N);
    let mut data2 = Vec::with_capacity(N);
    for i in 0..N {
        let mut rng = Xorshift64::new(SEED + i as u64);
        let v = rng.next_f64() * 100.0;
        data.push(v);
        let mut rng2 = Xorshift64::new(SEED + 1_000_000 + i as u64);
        let noise = rng2.next_f64() * 10.0;
        data2.push(v.mul_add(0.8, noise + 5.0));
    }

    let t0 = Instant::now();
    let m = mean(&data);
    let mean_us = t0.elapsed().as_secs_f64() * 1e6;
    println!("  mean = {m:.10} ({mean_us:.0} us)");
    results.push(BenchResult {
        name: "mean",
        value: m,
        elapsed_us: mean_us,
    });

    let t0 = Instant::now();
    let sd = std_dev(&data);
    let var_val = sd * sd;
    let var_us = t0.elapsed().as_secs_f64() * 1e6;
    println!("  variance = {var_val:.10} ({var_us:.0} us)");
    results.push(BenchResult {
        name: "variance",
        value: var_val,
        elapsed_us: var_us,
    });

    let t0 = Instant::now();
    let r = pearson_r(&data, &data2);
    let pearson_us = t0.elapsed().as_secs_f64() * 1e6;
    println!("  pearson_r = {r:.10} ({pearson_us:.0} us)\n");
    results.push(BenchResult {
        name: "pearson_r",
        value: r,
        elapsed_us: pearson_us,
    });
}

fn bench_bootstrap(results: &mut Vec<BenchResult>) {
    const N: usize = 10_000;
    const N_REPLICATES: usize = 5_000;
    const CONFIDENCE: f64 = 0.95;
    const SEED: u64 = 99;

    println!("=== Bootstrap Resampling (N={N}, B={N_REPLICATES}) ===");

    let mut data = Vec::with_capacity(N);
    for i in 0..N {
        let mut rng = Xorshift64::new(SEED + i as u64);
        data.push(rng.next_f64().mul_add(50.0, 25.0));
    }

    let t0 = Instant::now();
    let br = bootstrap_mean(&data, N_REPLICATES, CONFIDENCE, SEED).or_exit("bootstrap_mean");
    let elapsed = t0.elapsed().as_secs_f64() * 1e6;

    println!(
        "    bootstrap: estimate={:.10} ci=[{:.10}, {:.10}]",
        br.estimate, br.ci_lower, br.ci_upper
    );
    println!("  elapsed: {elapsed:.0} us\n");
    results.push(BenchResult {
        name: "bootstrap_mean",
        value: br.estimate,
        elapsed_us: elapsed,
    });
}

fn print_json_output(results: &[BenchResult], backend: &str) {
    println!("=== JSON Benchmark Output ===");
    println!("{{");
    println!("  \"_source\": \"BarraCuda GPU Tier 2 baseline — groundSpring\",");
    println!("  \"_provenance\": {{");
    println!("    \"baseline_date\": \"2026-03-04\",");
    println!("    \"backend\": \"{backend}\",");
    println!("    \"generated_by\": \"bench_gpu_vs_kokkos\"");
    println!("  }},");
    println!("  \"results\": [");
    for (i, r) in results.iter().enumerate() {
        let comma = if i + 1 < results.len() { "," } else { "" };
        println!(
            "    {{\"name\": \"{}\", \"value\": {:.15e}, \"elapsed_us\": {:.1}}}{}",
            r.name, r.value, r.elapsed_us, comma
        );
    }
    println!("  ]");
    println!("}}");
}

fn main() {
    let gpu_active = groundspring::gpu_available();

    println!("================================================================");
    println!("groundSpring — GPU Head-to-Head Benchmark");
    println!("  BarraCuda WGSL vs Kokkos CUDA (same workloads)");
    println!("================================================================\n");

    if gpu_active {
        println!("GPU: ACTIVE (barracuda-gpu, wgpu device found)");
    } else {
        println!("GPU: NOT AVAILABLE — results will be CPU fallback");
    }
    println!();

    let mut results: Vec<BenchResult> = Vec::new();

    if gpu_active {
        let warmup = vec![1.0_f64; 1000];
        let _ = mean(&warmup);
        println!("  (GPU warmup complete)\n");
    }

    bench_anderson(&mut results);
    bench_stats(&mut results);
    bench_bootstrap(&mut results);

    let backend = if gpu_active {
        "BarraCuda WGSL (wgpu)"
    } else {
        "CPU fallback"
    };
    print_json_output(&results, backend);
    println!("\n================================================================");
}
