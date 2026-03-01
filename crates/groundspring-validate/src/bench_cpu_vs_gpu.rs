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

fn bench_kimura(iters: u32) -> BenchEntry {
    let cpu_ms = bench(
        || {
            let mut total = 0.0_f64;
            for n in [100, 500, 1000, 5000, 10_000] {
                for &s in &[0.001, 0.01, 0.1] {
                    total += groundspring::drift::kimura_fixation_prob(n, s, 0.01);
                }
            }
            std::hint::black_box(total);
        },
        iters * 100,
    );

    BenchEntry {
        name: "Kimura fixation (15 configs)",
        cpu_ms,
        gpu_ms: None,
        speedup: None,
    }
}

fn bench_jackknife(iters: u32) -> BenchEntry {
    let data: Vec<f64> = (0..500).map(|i| (f64::from(i) * 0.3).sin()).collect();

    let cpu_ms = bench(
        || {
            let r = groundspring::jackknife::jackknife_mean_variance(&data).unwrap();
            std::hint::black_box(r);
        },
        iters,
    );

    BenchEntry {
        name: "Jackknife mean/var (500 points)",
        cpu_ms,
        gpu_ms: None,
        speedup: None,
    }
}

fn bench_chao1(iters: u32) -> BenchEntry {
    let counts: Vec<u64> = (0_u64..200)
        .map(|i| {
            if i < 50 {
                1
            } else if i < 100 {
                2
            } else {
                i * 3
            }
        })
        .collect();

    let cpu_ms = bench(
        || {
            let r = groundspring::rare_biosphere::chao1(&counts);
            std::hint::black_box(r);
        },
        iters * 1000,
    );

    BenchEntry {
        name: "Chao1 richness (200 taxa)",
        cpu_ms,
        gpu_ms: None,
        speedup: None,
    }
}

fn bench_fao56_scalar(iters: u32) -> BenchEntry {
    let inp = groundspring::fao56::example_18_inputs();

    let cpu_ms = bench(
        || {
            let r = groundspring::fao56::daily_et0(&inp);
            std::hint::black_box(r);
        },
        iters * 100,
    );

    BenchEntry {
        name: "FAO-56 scalar ET₀ (1 station-day)",
        cpu_ms,
        gpu_ms: None,
        speedup: None,
    }
}

fn bench_seismic(iters: u32) -> BenchEntry {
    let stations = vec![
        groundspring::seismic::Station {
            code: "STA1".into(),
            lat: 37.0,
            lon: -89.0,
        },
        groundspring::seismic::Station {
            code: "STA2".into(),
            lat: 37.5,
            lon: -88.0,
        },
        groundspring::seismic::Station {
            code: "STA3".into(),
            lat: 38.0,
            lon: -89.5,
        },
        groundspring::seismic::Station {
            code: "STA4".into(),
            lat: 37.2,
            lon: -88.5,
        },
    ];
    let observed = vec![("STA1", 5.0), ("STA2", 4.5), ("STA3", 6.0), ("STA4", 4.8)];
    let config = groundspring::seismic::GridSearchConfig {
        lat_range: (36.0, 39.0),
        lon_range: (-90.0, -87.0),
        depth_range: (0.0, 30.0),
        grid_spacing_deg: 0.1,
        depth_spacing_km: 5.0,
        vp: 6.0,
    };

    let cpu_ms = bench(
        || {
            let r = groundspring::seismic::grid_search_inversion(&observed, &stations, &config);
            std::hint::black_box(r);
        },
        iters,
    );

    BenchEntry {
        name: "Seismic inversion (31×31×7 grid)",
        cpu_ms,
        gpu_ms: None,
        speedup: None,
    }
}

fn bench_freeze_out(iters: u32) -> BenchEntry {
    let mu_b: Vec<f64> = (0..20).map(|i| f64::from(i) * 25.0).collect();
    let observed: Vec<f64> = mu_b
        .iter()
        .map(|&m| (-0.013_f64).mul_add((m / 155.0).powi(2), 1.0) * 155.0)
        .collect();
    let config = groundspring::freeze_out::GridFitConfig {
        observed: &observed,
        mu_b: &mu_b,
        sigma: 2.0,
        t0_lo: 140.0,
        t0_hi: 170.0,
        t0_step: 0.5,
        k2_lo: 0.005,
        k2_hi: 0.025,
        k2_step: 0.0005,
    };

    let cpu_ms = bench(
        || {
            let r = groundspring::freeze_out::grid_fit_2d(&config).unwrap();
            std::hint::black_box(r);
        },
        iters,
    );

    BenchEntry {
        name: "Freeze-out grid fit (61×41 grid)",
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
        bench_fao56_scalar(iters),
        bench_kimura(iters),
        bench_jackknife(iters),
        bench_chao1(iters),
        bench_seismic(iters),
        bench_freeze_out(iters),
        bench_rare_biosphere(iters),
        bench_anderson(iters),
        bench_diversity(iters),
    ];

    print_results(&entries);
    println!("\n================================================================");
}
