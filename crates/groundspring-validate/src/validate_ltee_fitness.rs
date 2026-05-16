// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team
#![forbid(unsafe_code)]

//! Experiment 036 — LTEE Fitness Dynamics (Wiser et al. 2013).
//!
//! Two-phase validation:
//!   Phase 1: Model selection on noiseless power-law trajectory — verifies
//!            AIC/BIC correctly identifies `power_law` over `hyperbolic`/logarithmic.
//!   Phase 2: Jackknife variance estimation on noisy synthetic populations —
//!            verifies statistical machinery on LTEE-realistic data.
//!
//! LTEE `GuideStone` B2 | `lithoSpore` module 1 | Ecosystem critical path.

use groundspring::cast::usize_f64;
use groundspring::prng::Xorshift64;
use groundspring::stats::{compare_models, fit_all, fit_hyperbolic, fit_power_law};
use groundspring::validate::ValidationHarness;
use groundspring_validate::{
    f64_field, f64_range, parse_benchmark, print_provenance_header, usize_field,
};

const BENCHMARK: &str =
    include_str!("../../../control/ltee_fitness_dynamics/benchmark_ltee_fitness.json");

fn main() {
    std::process::exit(run());
}

fn generate_noisy_pop(gens: &[f64], alpha: f64, beta: f64, sigma: f64, seed: u64) -> Vec<f64> {
    let mut rng = Xorshift64::new(seed);
    let pa = alpha * rng.next_normal().mul_add(0.15, 1.0);
    let pb = (beta * rng.next_normal().mul_add(0.08, 1.0)).clamp(0.1, 0.9);
    gens.iter()
        .map(|&t| {
            if t == 0.0 {
                1.0
            } else {
                let w = (pa.mul_add(t, 1.0)).powf(pb);
                (sigma.mul_add(rng.next_normal(), w)).max(1.0)
            }
        })
        .collect()
}

fn run() -> i32 {
    let bench = parse_benchmark(BENCHMARK);
    let mut h =
        ValidationHarness::from_args("Rust Validation: LTEE Fitness Dynamics (Wiser 2013 B2)");
    print_provenance_header(&bench, "LTEE Fitness Dynamics");

    let model = &bench["model"];
    let exp = &bench["expected_results"];
    let n_pop = usize_field(model, "n_populations");
    let alpha = f64_field(&model["power_law_params"], "alpha");
    let beta = f64_field(&model["power_law_params"], "beta");
    let noise_sigma = f64_field(model, "noise_sigma");
    let seed = model["seed"].as_u64().unwrap_or(42);
    let exp_range = f64_range(&exp["power_law_exponent_range"]);
    let jk_se_max = f64_field(exp, "jackknife_exponent_se_max");
    let pl_r2_min = f64_field(exp, "power_law_r_squared_min");

    let gens: Vec<f64> = model["generations"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(serde_json::Value::as_f64)
        .collect();
    let gens_pos: Vec<f64> = gens.iter().copied().filter(|&t| t > 0.0).collect();

    // ── Phase 1: Noiseless model selection ──────────────────────────
    let a_coeff: f64 = 0.004;
    let b_exp: f64 = 0.66;
    let noiseless: Vec<f64> = gens_pos
        .iter()
        .map(|&t| a_coeff.mul_add(t.powf(b_exp), 1.0))
        .collect();

    let fits = fit_all(&gens_pos, &noiseless);
    let comparisons = compare_models(&fits, &gens_pos, &noiseless);

    h.check_true(
        "Phase 1: at least 3 models converge",
        comparisons.len() >= 3,
    );

    if let Some(best) = comparisons.first() {
        h.check_true(
            "Phase 1: best model by AIC is power_law",
            best.model == "power_law",
        );
    }

    let mut by_bic = comparisons.clone();
    by_bic.sort_by(|a, b| {
        a.bic
            .partial_cmp(&b.bic)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    if let Some(best_bic) = by_bic.first() {
        h.check_true(
            "Phase 1: best model by BIC is power_law",
            best_bic.model == "power_law",
        );
    }

    if let Some(pl) = comparisons.iter().find(|c| c.model == "power_law") {
        h.check_true(
            "Phase 1: power-law R² >= threshold",
            pl.r_squared >= pl_r2_min,
        );
    }

    if let (Some(pl), Some(hyp)) = (
        comparisons.iter().find(|c| c.model == "power_law"),
        comparisons.iter().find(|c| c.model == "hyperbolic"),
    ) {
        h.check_true(
            "Phase 1: AIC(power_law) < AIC(hyperbolic)",
            pl.aic < hyp.aic,
        );
    }

    if let Some(pl_fit) = fit_power_law(&gens_pos, &noiseless) {
        h.check_true(
            "Phase 1: exponent in expected range",
            pl_fit.params[1] >= exp_range.0 && pl_fit.params[1] <= exp_range.1,
        );
    }

    if let (Some(pl_fit), Some(hyp_fit)) = (
        fit_power_law(&gens_pos, &noiseless),
        fit_hyperbolic(&gens_pos, &noiseless),
    ) {
        h.check_true(
            "Phase 1: power-law R² > hyperbolic R²",
            pl_fit.r_squared > hyp_fit.r_squared,
        );
    }

    // ── Phase 2: Noisy populations + jackknife ──────────────────────
    let populations: Vec<Vec<f64>> = (0..n_pop)
        .map(|i| generate_noisy_pop(&gens, alpha, beta, noise_sigma, seed + i as u64))
        .collect();

    h.check_true(
        "Phase 2: all populations fitness increasing",
        populations
            .iter()
            .all(|p| p.windows(2).skip(1).all(|w| w[1] >= w[0] - 0.1)),
    );

    let n_sub_f = usize_f64(n_pop - 1);
    let mut jk_exponents = Vec::new();
    for skip in 0..n_pop {
        let subset_mean: Vec<f64> = (0..gens.len())
            .map(|j| {
                populations
                    .iter()
                    .enumerate()
                    .filter(|&(i, _)| i != skip)
                    .map(|(_, p)| p[j])
                    .sum::<f64>()
                    / n_sub_f
            })
            .collect();
        let sm_pos: Vec<f64> = gens
            .iter()
            .zip(&subset_mean)
            .filter(|&(&t, _)| t > 0.0)
            .map(|(&_, &f)| f)
            .collect();
        if let Some(fit) = fit_power_law(&gens_pos, &sm_pos) {
            jk_exponents.push(fit.params[1]);
        }
    }

    if jk_exponents.len() == n_pop {
        let jk_mean = jk_exponents.iter().sum::<f64>() / usize_f64(jk_exponents.len());
        let jk_var = usize_f64(n_pop - 1) / usize_f64(n_pop)
            * jk_exponents
                .iter()
                .map(|&b| (b - jk_mean).powi(2))
                .sum::<f64>();
        h.check_true(
            "Phase 2: jackknife SE(b) within bounds",
            jk_var.sqrt() <= jk_se_max,
        );
    }

    let pop2: Vec<Vec<f64>> = (0..n_pop)
        .map(|i| generate_noisy_pop(&gens, alpha, beta, noise_sigma, seed + i as u64))
        .collect();
    let n_pop_f = usize_f64(n_pop);
    let mean1: Vec<f64> = (0..gens.len())
        .map(|j| populations.iter().map(|p| p[j]).sum::<f64>() / n_pop_f)
        .collect();
    let mean2: Vec<f64> = (0..gens.len())
        .map(|j| pop2.iter().map(|p| p[j]).sum::<f64>() / n_pop_f)
        .collect();
    h.check_true(
        "Phase 2: deterministic (same seed)",
        mean1
            .iter()
            .zip(&mean2)
            .all(|(&a, &b)| (a - b).abs() < 1e-12),
    );

    h.summary()
}
