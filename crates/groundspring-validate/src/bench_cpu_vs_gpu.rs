// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! CPU vs GPU timing benchmark for groundSpring.
//!
//! Measures wall-clock time for GPU-capable functions across all three
//! tiers: default (CPU), barracuda CPU, and barracuda-gpu. When compiled
//! with `--features barracuda-gpu`, the batch functions will automatically
//! dispatch to GPU if available, falling back to CPU otherwise.
//!
//! Usage:
//!     cargo run --release --features barracuda-gpu --bin bench-cpu-vs-gpu

use std::time::Instant;

struct BenchEntry {
    name: &'static str,
    cpu_ms: f64,
    gpu_ms: Option<f64>,
    speedup: Option<f64>,
}

fn bench<F: Fn()>(f: F, iters: u32) -> f64 {
    let start = Instant::now();
    for _ in 0..iters {
        f();
    }
    let elapsed = start.elapsed();
    elapsed.as_secs_f64() * 1000.0 / f64::from(iters)
}

fn bench_gillespie(iters: u32) -> BenchEntry {
    let rates = vec![1.0_f64; 10];
    let cpu_ms = bench(
        || {
            let mut means = Vec::with_capacity(100);
            for i in 0..100_u64 {
                let traj = groundspring::gillespie::birth_death_ssa(&rates, 1.0, 10, 200.0, 42 + i);
                means.push(groundspring::gillespie::time_averaged_mean(&traj, 50.0));
            }
            std::hint::black_box(means);
        },
        iters,
    );

    let gpu_ms = bench(
        || {
            let r = groundspring::gillespie::birth_death_ssa_batch(
                &rates, 1.0, 10, 200.0, 100, 50.0, 42,
            );
            std::hint::black_box(r);
        },
        iters,
    );

    BenchEntry {
        name: "Gillespie SSA (100 trajectories)",
        cpu_ms,
        gpu_ms: Some(gpu_ms),
        speedup: Some(cpu_ms / gpu_ms),
    }
}

fn bench_wright_fisher(iters: u32) -> BenchEntry {
    let cpu_ms = bench(
        || {
            let mut count = 0_usize;
            for i in 0..100_u64 {
                if groundspring::drift::wright_fisher_fixation(200, 0.01, 0.5, 42 + i) {
                    count += 1;
                }
            }
            std::hint::black_box(count);
        },
        iters,
    );

    let gpu_ms = bench(
        || {
            let count = groundspring::drift::wright_fisher_fixation_batch(200, 0.01, 0.5, 100, 42);
            std::hint::black_box(count);
        },
        iters,
    );

    BenchEntry {
        name: "Wright-Fisher fixation (100 trials)",
        cpu_ms,
        gpu_ms: Some(gpu_ms),
        speedup: Some(cpu_ms / gpu_ms),
    }
}

fn bench_fao56(iters: u32) -> BenchEntry {
    let inputs: Vec<_> = (0..500)
        .map(|_| groundspring::fao56::example_18_inputs())
        .collect();

    let cpu_ms = bench(
        || {
            let r: Vec<f64> = inputs.iter().map(groundspring::fao56::daily_et0).collect();
            std::hint::black_box(r);
        },
        iters,
    );

    let gpu_ms = bench(
        || {
            let r = groundspring::fao56::daily_et0_batch(&inputs);
            std::hint::black_box(r);
        },
        iters,
    );

    BenchEntry {
        name: "FAO-56 ET₀ (500 station-days)",
        cpu_ms,
        gpu_ms: Some(gpu_ms),
        speedup: Some(cpu_ms / gpu_ms),
    }
}

fn bench_rare_biosphere(iters: u32) -> BenchEntry {
    let mut community = vec![0.001; 200];
    community[0] = 0.5;
    let total: f64 = community.iter().sum();
    for c in &mut community {
        *c /= total;
    }

    let cpu_ms = bench(
        || {
            let r = groundspring::rare_biosphere::abundance_occupancy(&community, 100, 10000, 42);
            std::hint::black_box(r);
        },
        iters,
    );

    BenchEntry {
        name: "Rare biosphere (200sp × 100 samples)",
        cpu_ms,
        gpu_ms: None,
        speedup: None,
    }
}

fn bench_anderson(iters: u32) -> BenchEntry {
    let potential: Vec<f64> = (0..1000)
        .map(|i| (f64::from(i) * 0.1).sin() * 2.0)
        .collect();

    let cpu_ms = bench(
        || {
            let r = groundspring::anderson::lyapunov_exponent(&potential, 0.5);
            std::hint::black_box(r);
        },
        iters,
    );

    BenchEntry {
        name: "Anderson Lyapunov (1000 sites)",
        cpu_ms,
        gpu_ms: None,
        speedup: None,
    }
}

fn bench_diversity(iters: u32) -> BenchEntry {
    let cpu_ms = bench(
        || {
            let r = groundspring::drift::neutral_diversity_trajectory(20, 200, 500, 42);
            std::hint::black_box(r);
        },
        iters,
    );

    BenchEntry {
        name: "Neutral diversity (20sp × 500 gens)",
        cpu_ms,
        gpu_ms: None,
        speedup: None,
    }
}

fn print_results(entries: &[BenchEntry]) {
    println!(
        "\n{:<45} {:>10} {:>14} {:>10}",
        "Workload", "CPU (ms)", "Batch/GPU (ms)", "Speedup"
    );
    println!("{}", "-".repeat(82));

    for e in entries {
        match (e.gpu_ms, e.speedup) {
            (Some(gpu), Some(sp)) => {
                println!(
                    "{:<45} {:>10.2} {:>14.2} {:>9.1}×",
                    e.name, e.cpu_ms, gpu, sp
                );
            }
            _ => {
                println!("{:<45} {:>10.2} {:>14} {:>10}", e.name, e.cpu_ms, "-", "-");
            }
        }
    }

    println!("{}", "-".repeat(82));
    println!("\nNotes:");
    println!("  - 'CPU' = sequential single calls (no barracuda)");
    println!("  - 'Batch/GPU' = batch API (GPU if available, CPU fallback)");
    println!("  - Speedup = CPU / Batch, measures batch dispatch advantage");
    println!("  - Workloads marked '-' have only single-path implementations");

    if groundspring::gpu_available() {
        println!("  - GPU path ACTIVE: batch functions dispatched to GPU");
    } else {
        println!("  - GPU path INACTIVE: batch functions using CPU fallback");
    }
}

fn main() {
    println!("================================================================");
    println!("groundSpring — CPU vs GPU Performance Benchmark");
    println!("================================================================\n");

    if groundspring::gpu_available() {
        println!("GPU: AVAILABLE (barracuda-gpu feature enabled, device found)");
    } else {
        println!("GPU: NOT AVAILABLE (feature disabled or no device)");
    }

    let iters = 5;
    println!("\nRunning benchmarks ({iters} iterations each)...\n");

    let entries = vec![
        bench_gillespie(iters),
        bench_wright_fisher(iters),
        bench_fao56(iters),
        bench_rare_biosphere(iters),
        bench_anderson(iters),
        bench_diversity(iters),
    ];

    print_results(&entries);
    println!("\n================================================================");
}
