// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Validation binary for Experiment 014: Drift vs Selection.
//!
//! Wright-Fisher simulation testing when stochastic drift dominates
//! over deterministic selection in finite populations.
//!
//! Reference: Anderson (2022) mBio 13:e00354-22

use groundspring::drift::{
    kimura_fixation_prob, neutral_diversity_trajectory, wright_fisher_fixation,
};
use groundspring::validate::ValidationHarness;
use groundspring_validate::{f64_field, f64_range, print_provenance_header, usize_field};
use serde_json::Value;

const BENCHMARK: &str =
    include_str!("../../../control/drift_selection/benchmark_drift_selection.json");

#[expect(
    clippy::too_many_lines,
    reason = "validation harness with neutral + selection sweep checks"
)]
fn run() -> i32 {
    let Ok(bench) = serde_json::from_str::<Value>(BENCHMARK) else {
        eprintln!("FATAL: invalid benchmark JSON");
        return 1;
    };
    let mut h = ValidationHarness::stdout("Rust Validation: Drift vs Selection");

    print_provenance_header(&bench, "Drift vs Selection");

    let model = &bench["model"];
    let exp = &bench["expected_results"];

    let s_coeff = f64_field(model, "selection_coefficient");
    let p0 = f64_field(model, "initial_frequency");
    let n_trials = usize_field(model, "n_trials");
    let base_seed = model["base_seed"].as_u64().expect("base_seed");

    let pop_sizes: Vec<usize> = bench["model"]["population_sizes"]
        .as_array()
        .expect("population_sizes")
        .iter()
        .map(|v| {
            #[expect(
                clippy::cast_possible_truncation,
                reason = "JSON population sizes ≤ 10000, fits usize"
            )]
            let n = v.as_u64().expect("u64") as usize;
            n
        })
        .collect();

    // Part 1: Neutral fixation
    println!("\n--- Part 1: Neutral Fixation (s=0) ---");
    let n_neutral = 100;
    let neutral_fixes: usize = (0..n_trials)
        .filter(|&i| wright_fisher_fixation(n_neutral, 0.0, p0, base_seed + i as u64))
        .count();

    #[expect(
        clippy::cast_precision_loss,
        reason = "fix count and n_trials ≤ 1000 ≪ 2^53"
    )]
    let neutral_rate = neutral_fixes as f64 / n_trials as f64;
    println!("  N={n_neutral}, s=0: P_fix = {neutral_rate:.3} (expected ~{p0})");

    let (nlo, nhi) = f64_range(&exp["neutral_fixation_range"]);
    h.check_range("Neutral fixation ≈ p₀", neutral_rate, nlo, nhi);

    // Part 2: Selection across population sizes
    println!("\n--- Part 2: Selection Across Population Sizes ---");
    let mut fix_rates = Vec::new();

    for &n_pop in &pop_sizes {
        let fixes: usize = (0..n_trials)
            .filter(|&i| {
                let seed = base_seed + 10000 + (n_pop as u64) * 1000 + i as u64;
                wright_fisher_fixation(n_pop, s_coeff, p0, seed)
            })
            .count();

        #[expect(
            clippy::cast_precision_loss,
            reason = "fix count and n_trials ≤ 1000 ≪ 2^53"
        )]
        let rate = fixes as f64 / n_trials as f64;
        fix_rates.push(rate);

        let kimura = kimura_fixation_prob(n_pop, s_coeff, p0);
        #[expect(clippy::cast_precision_loss, reason = "n_pop ≤ 10000 ≪ 2^53")]
        let ns = n_pop as f64 * s_coeff;
        let regime = if ns < 1.0 { "DRIFT" } else { "SELECTION" };
        println!("  N={n_pop:4}, N×s={ns:5.2} ({regime:9}): P_fix={rate:.3} (Kimura={kimura:.3})");
    }

    let drift_tol = f64_field(exp, "drift_regime_fixation_near_neutral_tol");
    h.check_range(
        &format!("Drift regime (N={}) near neutral", pop_sizes[0]),
        fix_rates[0],
        p0 - drift_tol,
        p0 + drift_tol,
    );

    let sel_min = f64_field(exp, "strong_selection_fixation_min");
    h.check_true(
        &format!(
            "Selection regime (N={}) > 60%",
            pop_sizes[pop_sizes.len() - 1]
        ),
        *fix_rates.last().unwrap_or(&0.0) >= sel_min,
    );

    h.check_true(
        "Fixation increases with N",
        *fix_rates.last().unwrap_or(&0.0) > fix_rates[0],
    );

    // Part 3: Kimura accuracy
    println!("\n--- Part 3: Kimura Formula ---");
    for (i, &n_pop) in pop_sizes.iter().enumerate() {
        let kimura = kimura_fixation_prob(n_pop, s_coeff, p0);
        let diff = (fix_rates[i] - kimura).abs();
        println!(
            "  N={n_pop:4}: obs={:.3}, Kimura={kimura:.3}, diff={diff:.3}",
            fix_rates[i]
        );
    }

    // Part 4: Neutral diversity decay
    println!("\n--- Part 4: Neutral Diversity Decay ---");
    let n_sp = usize_field(model, "n_species_neutral");
    let n_gen = usize_field(model, "n_generations_diversity");

    let div_small = neutral_diversity_trajectory(n_sp, 50, n_gen, base_seed + 90000);
    let div_large = neutral_diversity_trajectory(n_sp, 500, n_gen, base_seed + 91000);

    let h0s = *div_small.first().expect("non-empty small-pop trajectory");
    let hes = *div_small.last().expect("non-empty small-pop trajectory");
    let h0l = *div_large.first().expect("non-empty large-pop trajectory");
    let hel = *div_large.last().expect("non-empty large-pop trajectory");

    println!("  N=50:  H(0)={h0s:.4} → H({n_gen})={hes:.4}");
    println!("  N=500: H(0)={h0l:.4} → H({n_gen})={hel:.4}");

    h.check_true("Diversity declines (N=50)", hes < h0s);
    h.check_true("Small pop loses more diversity", hes < hel);

    // Part 5: Determinism
    println!("\n--- Part 5: Determinism ---");
    let r1 = wright_fisher_fixation(100, 0.01, 0.5, 99999);
    let r2 = wright_fisher_fixation(100, 0.01, 0.5, 99999);
    h.check_true("WF deterministic", r1 == r2);

    h.summary()
}

fn main() {
    std::process::exit(run());
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    #[test]
    fn validation_passes() {
        assert_eq!(super::run(), 0);
    }
}
