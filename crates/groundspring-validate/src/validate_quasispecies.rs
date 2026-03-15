// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Validation binary for Experiment 017: Quasispecies Error Threshold.
//!
//! At what mutation rate does noise destroy self-replicating information?
//!
//! References:
//! - Dolson et al. (2023) J R Soc Interface 20(208)
//! - Eigen (1971) Naturwiss 58:465-523

use groundspring::quasispecies::{
    error_threshold, master_frequency_analytical, mean_fitness, quasispecies_simulation,
};
use groundspring::validate::ValidationHarness;
use groundspring_validate::{
    TOL_RAREFACTION_PROP, f64_field, f64_range, print_provenance_header, usize_field,
};
use serde_json::Value;

const BENCHMARK: &str =
    include_str!("../../../control/quasispecies_threshold/benchmark_quasispecies.json");

fn run() -> i32 {
    let bench: Value = serde_json::from_str(BENCHMARK).expect("valid benchmark JSON");
    let mut h = ValidationHarness::stdout("Rust Validation: Quasispecies Error Threshold");

    println!("{}", "=".repeat(72));
    println!("groundSpring Rust Validation: Quasispecies (Exp 017)");
    println!("{}", "=".repeat(72));
    print_provenance_header(&bench, "Quasispecies Error Threshold");

    let model = &bench["model"];
    let exp = &bench["expected_results"];

    let pop_size = usize_field(model, "population_size");
    let genome_length = usize_field(model, "genome_length");
    let sigma = f64_field(model, "master_fitness");
    let n_gen = usize_field(model, "n_generations");
    let base_seed =
        groundspring_validate::get_u64(model, "base_seed").expect("benchmark base_seed");

    let mutation_rates = groundspring_validate::get_f64_vec(model, "mutation_rates")
        .expect("benchmark mutation_rates");

    let mu_c = error_threshold(sigma, genome_length);

    // Part 1: Analytical predictions
    println!("\n--- Part 1: Analytical ---");
    for &mu in &mutation_rates {
        let x_m = master_frequency_analytical(sigma, mu, genome_length);
        let regime = if mu < mu_c { "BELOW" } else { "ABOVE" };
        println!("  μ={mu:.3} ({regime:5} threshold): x_m = {x_m:.4}");
    }

    let (thr_lo, thr_hi) = f64_range(&exp["error_threshold_observed_range"]);
    h.check_range("Error threshold in expected range", mu_c, thr_lo, thr_hi);

    // Part 2: Below threshold — signal survives
    println!("\n--- Part 2: Below Threshold ---");
    let mu_below = mutation_rates[1];
    let freqs_below =
        quasispecies_simulation(pop_size, genome_length, sigma, mu_below, n_gen, base_seed);
    let steady_below = tail_mean(&freqs_below, n_gen / 2);
    let x_m_theory = master_frequency_analytical(sigma, mu_below, genome_length);
    println!("  μ={mu_below}: steady x_m = {steady_below:.4} (theory {x_m_theory:.4})");
    h.check_true(
        "Master survives below threshold",
        steady_below >= f64_field(exp, "master_freq_below_threshold_min"),
    );

    // Part 3: Above threshold — noise wins
    println!("\n--- Part 3: Above Threshold ---");
    let mu_above = mutation_rates[5];
    let freqs_above = quasispecies_simulation(
        pop_size,
        genome_length,
        sigma,
        mu_above,
        n_gen,
        base_seed + 1000,
    );
    let steady_above = tail_mean(&freqs_above, n_gen / 2);
    println!("  μ={mu_above}: steady x_m = {steady_above:.4}");
    h.check_true(
        "Master lost above threshold",
        steady_above <= f64_field(exp, "master_freq_above_threshold_max"),
    );

    // Part 4: Mutation rate sweep
    println!("\n--- Part 4: Sweep ---");
    let mut steady_states = Vec::new();
    let mut fitnesses = Vec::new();
    for (i, &mu) in mutation_rates.iter().enumerate() {
        let freqs = quasispecies_simulation(
            pop_size,
            genome_length,
            sigma,
            mu,
            n_gen,
            base_seed + 5000 + (i as u64) * 100,
        );
        let ss = tail_mean(&freqs, n_gen / 2);
        let mf = mean_fitness(sigma, ss);
        steady_states.push(ss);
        fitnesses.push(mf);
        let regime = if mu < mu_c { "SIGNAL" } else { "NOISE" };
        println!("  μ={mu:.3}: x_m={ss:.4}, fitness={mf:.3} [{regime}]");
    }

    // Fitness drops at threshold
    let below_idx = mutation_rates.iter().rposition(|&mu| mu < mu_c);
    let above_idx = mutation_rates.iter().position(|&mu| mu > mu_c);
    if let (Some(bi), Some(ai)) = (below_idx, above_idx) {
        h.check_true(
            "Mean fitness drops at threshold",
            fitnesses[bi] > fitnesses[ai],
        );
    }

    // Part 5: Monotonicity
    println!("\n--- Part 5: Monotonicity ---");
    let decreasing = steady_states
        .windows(2)
        .all(|w| w[0] >= w[1] - TOL_RAREFACTION_PROP);
    h.check_true("Master frequency decreases with μ", decreasing);

    // Part 6: Determinism
    println!("\n--- Part 6: Determinism ---");
    let f1 = quasispecies_simulation(pop_size, genome_length, sigma, 0.01, 100, 99999);
    let f2 = quasispecies_simulation(pop_size, genome_length, sigma, 0.01, 100, 99999);
    h.check_true("Simulation deterministic", f1 == f2);

    h.summary()
}

fn tail_mean(freqs: &[f64], skip: usize) -> f64 {
    let tail = &freqs[skip..];
    if tail.is_empty() {
        return 0.0;
    }
    groundspring::stats::mean(tail)
}

fn main() {
    std::process::exit(run());
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    #[test]
    fn validation_passes() {
        assert_eq!(super::run(), 0);
    }
}
