// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team
#![forbid(unsafe_code)]

//! Benchmark parity harness: same workloads as `kokkos_baseline/src/main.cpp`.
//!
//! Runs the identical algorithms with the identical parameters and PRNG seeds
//! so results can be compared directly against the Kokkos Tier 1 baseline.

use std::time::Instant;

use groundspring::anderson::{anderson_potential, lyapunov_exponent};
use groundspring::bootstrap::bootstrap_mean;
use groundspring::prng::Xorshift64;
use groundspring::stats::{mean, pearson_r, std_dev};
use groundspring_validate::OrExit;

fn bench_anderson(results: &mut Vec<(&str, f64, f64)>) {
    const N_SITES: usize = 10_000;
    const DISORDER: f64 = 4.0;
    const N_REALIZATIONS: usize = 500;
    const ENERGY: f64 = 0.0;
    const BASE_SEED: u64 = 42;

    println!("=== Anderson Localization (Lyapunov Exponent) ===");
    println!("  N={N_SITES}, W={DISORDER}, realizations={N_REALIZATIONS}, E={ENERGY}");

    let t0 = Instant::now();
    let mut gamma_sum = 0.0;
    for r in 0..N_REALIZATIONS {
        let seed = BASE_SEED + r as u64;
        let pot = anderson_potential(N_SITES, DISORDER, seed);
        gamma_sum += lyapunov_exponent(&pot, ENERGY);
    }
    #[expect(clippy::cast_precision_loss, reason = "N_REALIZATIONS=500 ≪ 2^53")]
    let n_real_f = N_REALIZATIONS as f64;
    let gamma_avg = gamma_sum / n_real_f;
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
    results.push(("anderson_lyapunov_averaged", gamma_avg, elapsed));
}

fn bench_stats(results: &mut Vec<(&str, f64, f64)>) {
    const N: usize = 1_000_000;
    const SEED: u64 = 12345;

    println!("=== Statistical Reductions (N={N}) ===");

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
    results.push(("mean", m, mean_us));

    let t0 = Instant::now();
    let sd = std_dev(&data);
    let var_val = sd * sd;
    let var_us = t0.elapsed().as_secs_f64() * 1e6;
    println!("  variance = {var_val:.10} ({var_us:.0} us)");
    results.push(("variance", var_val, var_us));

    let t0 = Instant::now();
    let r = pearson_r(&data, &data2);
    let pearson_us = t0.elapsed().as_secs_f64() * 1e6;
    println!("  pearson_r = {r:.10} ({pearson_us:.0} us)\n");
    results.push(("pearson_r", r, pearson_us));
}

fn bench_bootstrap(results: &mut Vec<(&str, f64, f64)>) {
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
    results.push(("bootstrap_mean", br.estimate, elapsed));
}

fn main() {
    println!("groundSpring Rust Tier 2 Baseline");
    println!("  Backend: CPU (default features)\n");

    let mut results: Vec<(&str, f64, f64)> = Vec::new();

    bench_anderson(&mut results);
    bench_stats(&mut results);
    bench_bootstrap(&mut results);

    println!("=== JSON Benchmark Output ===");
    println!("{{");
    println!("  \"_source\": \"Rust Tier 2 baseline — groundSpring\",");
    println!("  \"_provenance\": {{");
    println!("    \"baseline_date\": \"2026-03-04\",");
    println!("    \"backend\": \"CPU (default features)\",");
    println!("    \"generated_by\": \"bench_kokkos_parity\"");
    println!("  }},");
    println!("  \"results\": [");
    for (i, (name, value, elapsed)) in results.iter().enumerate() {
        let comma = if i + 1 < results.len() { "," } else { "" };
        println!(
            "    {{\"name\": \"{name}\", \"value\": {value:.15e}, \"elapsed_us\": {elapsed:.1}}}{comma}"
        );
    }
    println!("  ]");
    println!("}}");
}
