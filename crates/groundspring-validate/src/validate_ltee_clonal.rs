// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team
#![forbid(unsafe_code)]

//! Experiment 038 — LTEE Clonal Interference (Good et al. 2017).
//!
//! Validates clonal interference dynamics in asexual populations:
//!   - Fixation probability decreases with population size
//!   - Fixation probability exceeds neutral (1/N) for beneficial mutations
//!   - Adaptation rate (log-fitness) scales sublinearly with N in the
//!     interference regime
//!   - Small-N fixation approaches the Haldane sieve (2s)
//!
//! LTEE `GuideStone` B3 | `lithoSpore` module 3.

use groundspring::cast::usize_f64;
use groundspring::prng::Xorshift64;
use groundspring::validate::ValidationHarness;
use groundspring_validate::{f64_field, parse_benchmark, print_provenance_header, usize_field};

const BENCHMARK: &str =
    include_str!("../../../control/ltee_clonal_interference/benchmark_ltee_clonal.json");

fn main() {
    std::process::exit(run());
}

struct Lineage {
    frequency: f64,
    selective_advantage: f64,
}

fn simulate_clonal_interference(
    pop_size: usize,
    n_gens: usize,
    u_b: f64,
    mean_s: f64,
    rng: &mut Xorshift64,
) -> (usize, usize, f64) {
    let n = pop_size;
    let nf = usize_f64(n);
    let mut lineages: Vec<Lineage> = Vec::new();
    let mut fixation_count: usize = 0;
    let mut total_mutations: usize = 0;
    let mut log_fitness = 0.0_f64;

    for _ in 0..n_gens {
        let n_new = poisson(nf * u_b, rng);
        for _ in 0..n_new {
            let s_i = exponential(mean_s, rng);
            lineages.push(Lineage {
                frequency: 1.0 / nf,
                selective_advantage: s_i,
            });
            total_mutations += 1;
        }

        let mut surviving = Vec::with_capacity(lineages.len());
        for lin in &lineages {
            let mean_freq = lin.frequency * (1.0 + lin.selective_advantage)
                / lin.frequency.mul_add(lin.selective_advantage, 1.0);
            let clamped = mean_freq.min(1.0);
            let n_copies = binomial(n, clamped, rng);
            let new_freq = usize_f64(n_copies) / nf;

            if new_freq >= 1.0 {
                fixation_count += 1;
                log_fitness += lin.selective_advantage.ln_1p();
            } else if new_freq > 0.0 {
                surviving.push(Lineage {
                    frequency: new_freq,
                    selective_advantage: lin.selective_advantage,
                });
            }
        }
        lineages = surviving;
    }

    (fixation_count, total_mutations, log_fitness)
}

fn poisson(lambda: f64, rng: &mut Xorshift64) -> usize {
    if lambda <= 0.0 {
        return 0;
    }
    let l = (-lambda).exp();
    let mut k: usize = 0;
    let mut p = 1.0_f64;
    loop {
        k += 1;
        p *= rng.next_f64();
        if p < l {
            break;
        }
    }
    k - 1
}

fn exponential(mean: f64, rng: &mut Xorshift64) -> f64 {
    let u = rng.next_f64();
    -mean * (1.0 - u).ln()
}

fn binomial(n: usize, p: f64, rng: &mut Xorshift64) -> usize {
    if p <= 0.0 {
        return 0;
    }
    if p >= 1.0 {
        return n;
    }
    if n <= 30 {
        let mut count = 0;
        for _ in 0..n {
            if rng.next_f64() < p {
                count += 1;
            }
        }
        return count;
    }
    let nf = usize_f64(n);
    let mean = nf * p;
    let std = (nf * p * (1.0 - p)).sqrt();
    let z = normal_approx(rng);
    let val = (mean + std * z).round();
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "clamped to [0, N] — safe"
    )]
    {
        val.clamp(0.0, nf) as usize
    }
}

fn normal_approx(rng: &mut Xorshift64) -> f64 {
    let u1 = rng.next_f64().max(1e-15);
    let u2 = rng.next_f64();
    (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
}

struct PopResult {
    total_fixations: usize,
    total_mutations: usize,
    fixation_probability: f64,
    mean_log_fitness: f64,
    adaptation_rate: f64,
}

fn run_replicates(
    pop_size: usize,
    n_gens: usize,
    u_b: f64,
    mean_s: f64,
    n_reps: usize,
    rng: &mut Xorshift64,
) -> PopResult {
    let mut total_fix = 0_usize;
    let mut total_mut = 0_usize;
    let mut log_fitness_sum = 0.0_f64;

    for _ in 0..n_reps {
        let (fixes, muts, log_fit) =
            simulate_clonal_interference(pop_size, n_gens, u_b, mean_s, rng);
        total_fix += fixes;
        total_mut += muts;
        log_fitness_sum += log_fit;
    }

    let fix_prob = if total_mut > 0 {
        usize_f64(total_fix) / usize_f64(total_mut)
    } else {
        0.0
    };
    let mean_log_fit = log_fitness_sum / usize_f64(n_reps);
    let adapt_rate = mean_log_fit / usize_f64(n_gens);

    PopResult {
        total_fixations: total_fix,
        total_mutations: total_mut,
        fixation_probability: fix_prob,
        mean_log_fitness: mean_log_fit,
        adaptation_rate: adapt_rate,
    }
}

fn run() -> i32 {
    let bench = parse_benchmark(BENCHMARK);
    let mut h = ValidationHarness::from_args(
        "Rust Validation: LTEE Clonal Interference (Good et al. 2017 B3)",
    );
    print_provenance_header(&bench, "LTEE Clonal Interference");

    let model = &bench["model"];
    let exp = &bench["expected_results"];

    let pop_sizes: Vec<usize> = model["pop_sizes"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_u64().and_then(|x| usize::try_from(x).ok()))
                .collect()
        })
        .unwrap_or_default();
    let n_gens = usize_field(model, "n_generations");
    let u_b = f64_field(model, "beneficial_mutation_rate");
    let mean_s = f64_field(model, "mean_selective_advantage");
    let n_reps = usize_field(model, "n_replicates");
    let seed = model["seed"].as_u64().unwrap_or(2017);

    let mut rng = Xorshift64::new(seed);

    let mut results: Vec<(usize, PopResult)> = Vec::new();
    for &n in &pop_sizes {
        let res = run_replicates(n, n_gens, u_b, mean_s, n_reps, &mut rng);
        results.push((n, res));
    }

    // Check 1: fixation probability decreases with N
    let fix_probs: Vec<f64> = results
        .iter()
        .map(|(_, r)| r.fixation_probability)
        .collect();
    let decreasing = fix_probs.windows(2).all(|w| w[0] >= w[1]);
    h.check_true("Fixation probability decreases with N", decreasing);

    // Check 2: fixation prob > neutral (1/N) for all sizes
    let above_neutral = results
        .iter()
        .all(|(n, r)| r.fixation_probability > 1.0 / usize_f64(*n));
    h.check_true(
        "Fixation probability > neutral (1/N) for all N",
        above_neutral,
    );

    // Check 3: CI ratio N=1000 vs N=100
    let r100 = results.iter().find(|(n, _)| *n == 100).map(|(_, r)| r);
    let r1000 = results.iter().find(|(n, _)| *n == 1000).map(|(_, r)| r);
    if let (Some(r100), Some(r1000)) = (r100, r1000) {
        let ci_ratio = if r100.fixation_probability > 0.0 {
            r1000.fixation_probability / r100.fixation_probability
        } else {
            0.0
        };
        let range = &exp["clonal_interference_ratio_N1000_vs_N100"];
        let lo = range[0].as_f64().unwrap_or(0.3);
        let hi = range[1].as_f64().unwrap_or(0.9);
        h.check_true(
            "CI ratio (N=1000/N=100) in expected range",
            (lo..=hi).contains(&ci_ratio),
        );
    }

    // Check 4: mean fitness increases for all sizes
    let all_increase = results.iter().all(|(_, r)| r.mean_log_fitness > 0.0);
    h.check_true(
        "Mean fitness increases for all population sizes",
        all_increase,
    );

    // Check 5: adaptation rate scales sublinearly (N=100000 vs N=10000)
    let r10000 = results.iter().find(|(n, _)| *n == 10_000).map(|(_, r)| r);
    let r100000 = results.iter().find(|(n, _)| *n == 100_000).map(|(_, r)| r);
    if let (Some(r10k), Some(r100k)) = (r10000, r100000) {
        let rate_ratio = if r10k.adaptation_rate > 0.0 {
            r100k.adaptation_rate / r10k.adaptation_rate
        } else {
            0.0
        };
        let range = &exp["adaptation_rate_ratio_N100000_vs_N10000"];
        let lo = range[0].as_f64().unwrap_or(1.0);
        let hi = range[1].as_f64().unwrap_or(100.0);
        h.check_true(
            "Adaptation rate sublinear in interference regime",
            (lo..=hi).contains(&rate_ratio),
        );
    }

    // Check 6: small-N approaches Haldane sieve
    if let Some(r100) = r100 {
        let haldane = 2.0 * mean_s;
        h.check_true(
            "Small-N fixation approaches Haldane sieve (2s)",
            r100.fixation_probability > 0.5 * haldane,
        );
    }

    // Check 7: determinism
    let mut rng2 = Xorshift64::new(seed);
    let res2 = run_replicates(pop_sizes[0], n_gens, u_b, mean_s, n_reps, &mut rng2);
    h.check_true(
        "Deterministic (same seed → same counts)",
        res2.total_fixations == results[0].1.total_fixations
            && res2.total_mutations == results[0].1.total_mutations,
    );

    h.summary()
}
