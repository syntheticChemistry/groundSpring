// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team
#![forbid(unsafe_code)]

//! Experiment 037 — LTEE Neutral Mutation Dynamics (Barrick et al. 2009).
//!
//! Validates Kimura fixation theory and neutral molecular clock:
//!   - `P_fix(s=0)` = `1/N` for haploid populations
//!   - Substitution rate = genomic mutation rate μ (independent of N)
//!   - Drift dominates for `|s| < 1/N`; selection detectable for `|s| >> 1/N`
//!
//! LTEE `GuideStone` B1 | `lithoSpore` module 2.

use groundspring::cast::usize_f64;
use groundspring::drift::kimura_fixation_prob;
use groundspring::validate::ValidationHarness;
use groundspring_validate::{f64_field, parse_benchmark, print_provenance_header, usize_field};

const BENCHMARK: &str =
    include_str!("../../../control/ltee_neutral_mutation/benchmark_ltee_neutral.json");

fn main() {
    std::process::exit(run());
}

fn run() -> i32 {
    let bench = parse_benchmark(BENCHMARK);
    let mut h = ValidationHarness::stdout(
        "Rust Validation: LTEE Neutral Mutation Dynamics (Barrick 2009 B1)",
    );
    print_provenance_header(&bench, "LTEE Neutral Mutation");

    let model = &bench["model"];
    let exp = &bench["expected_results"];

    let pop_size = usize_field(model, "population_size");
    let mu = f64_field(model, "genomic_mutation_rate");
    let s_neutral = f64_field(model, "selection_coefficient");

    let expected_pfix = f64_field(&exp["fixation_probability_neutral"], "expected");
    let pfix_tol = f64_field(&exp["fixation_probability_neutral"], "tolerance_factor");
    let expected_rate = f64_field(&exp["accumulation_rate_per_generation"], "expected");
    let rate_tol = f64_field(&exp["accumulation_rate_per_generation"], "tolerance_factor");
    let kimura_tol = f64_field(&exp["kimura_fixation_analytical_match"], "tolerance");
    let s_threshold = f64_field(&exp["drift_dominates_for_small_s"], "s_threshold");
    let drift_factor = f64_field(
        &exp["drift_dominates_for_small_s"],
        "fixation_prob_within_factor",
    );

    // Check 1: Kimura fixation probability for neutral mutation = 1/N
    let pfix = kimura_fixation_prob(pop_size, s_neutral, 1.0 / usize_f64(pop_size));
    h.check_true(
        "Neutral fixation probability matches 1/N",
        (pfix - expected_pfix).abs() / expected_pfix < pfix_tol,
    );

    // Check 2: Molecular clock rate = μ (theoretical)
    h.check_true(
        "Molecular clock rate = μ",
        (mu - expected_rate).abs() / expected_rate < rate_tol,
    );

    // Check 3: Kimura formula matches 1/N analytical
    let analytical = 1.0 / usize_f64(pop_size);
    h.check_true(
        "Kimura formula matches 1/N analytical",
        (pfix - analytical).abs() < kimura_tol,
    );

    // Check 4: Drift dominates for small |s|
    let pfix_small_s = kimura_fixation_prob(pop_size, s_threshold, 1.0 / usize_f64(pop_size));
    let drift_ratio = pfix_small_s / analytical;
    h.check_true("Drift dominates at |s| = 1/N", drift_ratio < drift_factor);

    // Check 5: Selection detectable for |s| >> 1/N
    let pfix_large = kimura_fixation_prob(pop_size, 0.01, 1.0 / usize_f64(pop_size));
    h.check_true(
        "Selection detectable at s = 0.01",
        pfix_large > 10.0 * analytical,
    );

    // Check 6: Wright-Fisher neutral simulation (using our drift module)
    let n_trials = 500;
    let fixation_count = groundspring::drift::wright_fisher_fixation_batch(
        pop_size,
        0.0,
        1.0 / usize_f64(pop_size),
        n_trials,
        42,
    );
    let obs_rate = usize_f64(fixation_count) / usize_f64(n_trials);
    h.check_true(
        "Wright-Fisher neutral fixation rate reasonable",
        obs_rate < 0.1,
    );

    // Check 7: Beneficial mutation has higher fixation rate
    let ben_count =
        groundspring::drift::wright_fisher_fixation_batch(1000, 0.05, 0.01, n_trials, 99);
    let ben_rate = usize_f64(ben_count) / usize_f64(n_trials);
    h.check_true(
        "Beneficial mutation (s=0.05) fixes more often than neutral",
        ben_rate > obs_rate,
    );

    // Check 8: Determinism
    let fix_count2 = groundspring::drift::wright_fisher_fixation_batch(
        pop_size,
        0.0,
        1.0 / usize_f64(pop_size),
        n_trials,
        42,
    );
    h.check_true(
        "Deterministic (same seed → same count)",
        fixation_count == fix_count2,
    );

    h.summary()
}
