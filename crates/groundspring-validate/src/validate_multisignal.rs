// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Validation binary for Experiment 011: Multi-Signal QS Integration.
//!
//! Integrates the 7-variable dual-signal QS ODE and compares dual-signal
//! vs single-signal phenotypic responses, verifying that signal integration
//! produces a sharper regulatory response.
//!
//! Reference: Srivastava, Waters et al. (2011) J Bacteriology 194:122-136

use groundspring::multisignal::{MultiSignalParams, integrate, stochastic_integrate};
use groundspring::validate::ValidationHarness;
use groundspring_validate::{array_field, f64_field, print_provenance_header};
use serde_json::Value;

const BENCHMARK: &str = include_str!("../../../control/multisignal_qs/benchmark_multisignal.json");

fn params_from_json(model: &Value) -> MultiSignalParams {
    let p = &model["parameters"];
    MultiSignalParams {
        mu_max: f64_field(p, "mu_max"),
        k_cap: f64_field(p, "k_cap"),
        death_rate: f64_field(p, "death_rate"),
        k_cai1_prod: f64_field(p, "k_cai1_prod"),
        d_cai1: f64_field(p, "d_cai1"),
        k_cqs: f64_field(p, "k_cqs"),
        k_ai2_prod: f64_field(p, "k_ai2_prod"),
        d_ai2: f64_field(p, "d_ai2"),
        k_luxpq: f64_field(p, "k_luxpq"),
        k_luxo_phos: f64_field(p, "k_luxo_phos"),
        d_luxo_p: f64_field(p, "d_luxo_p"),
        k_hapr_max: f64_field(p, "k_hapr_max"),
        n_repress: f64_field(p, "n_repress"),
        k_repress: f64_field(p, "k_repress"),
        d_hapr: f64_field(p, "d_hapr"),
        k_dgc_basal: f64_field(p, "k_dgc_basal"),
        k_dgc_rep: f64_field(p, "k_dgc_rep"),
        k_pde_basal: f64_field(p, "k_pde_basal"),
        k_pde_act: f64_field(p, "k_pde_act"),
        d_cdg: f64_field(p, "d_cdg"),
        k_bio_max: f64_field(p, "k_bio_max"),
        k_bio_cdg: f64_field(p, "k_bio_cdg"),
        n_bio: f64_field(p, "n_bio"),
        d_bio: f64_field(p, "d_bio"),
    }
}

fn ic_from_json(model: &Value) -> [f64; 7] {
    let arr = array_field(model, "initial_state");
    let mut ic = [0.0; 7];
    for (i, val) in ic.iter_mut().enumerate() {
        *val = arr[i].as_f64().expect("IC element must be a valid f64");
    }
    ic
}

fn run() -> i32 {
    let bench: Value = serde_json::from_str(BENCHMARK).expect("valid benchmark JSON");
    let mut h = ValidationHarness::stdout("Rust Validation: Multi-Signal QS Integration");

    print_provenance_header(&bench, "Multi-Signal QS Integration");

    let model = &bench["model"];
    let pred = &bench["analytical_predictions"];
    let exp = &bench["expected_results"];

    let params = params_from_json(model);
    let dt = f64_field(model, "dt");
    let t_final = f64_field(model, "t_final");

    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "t_final/dt positive, ODE steps fit usize"
    )]
    let n_steps = (t_final / dt) as usize;

    let ic = ic_from_json(model);

    println!("  Model: 7-variable multi-signal QS ODE, dt={dt}, t_final={t_final}");

    // ── Part 1: Dual-signal steady state ──────────────────────────────
    println!("\n--- Part 1: Dual-Signal Steady State ---");

    let final_dual = integrate(&ic, &params, dt, n_steps);
    println!(
        "  Cell={:.3}, HapR={:.3}, cdg={:.3}, bio={:.3}",
        final_dual[0], final_dual[4], final_dual[5], final_dual[6]
    );

    h.check_approx(
        "Cell reaches capacity",
        final_dual[0],
        f64_field(pred, "cell_steady_state"),
        f64_field(exp, "cell_reaches_capacity_tol"),
    );
    h.check_min(
        "HapR > 0 at steady state",
        final_dual[4],
        f64_field(exp, "hapr_min_at_steady_state"),
    );
    h.check_min(
        "Biofilm > 0 at steady state",
        final_dual[6],
        f64_field(exp, "biofilm_min_at_steady_state"),
    );

    // ── Part 2: Single-signal comparisons ─────────────────────────────
    println!("\n--- Part 2: Single-Signal vs Dual ---");

    let mut p_cai1 = params;
    p_cai1.k_ai2_prod = 0.0;
    let final_cai1 = integrate(&ic, &p_cai1, dt, n_steps);

    let mut p_ai2 = params;
    p_ai2.k_cai1_prod = 0.0;
    let final_ai2 = integrate(&ic, &p_ai2, dt, n_steps);

    println!(
        "  CAI-1 only: HapR={:.3}, bio={:.3}",
        final_cai1[4], final_cai1[6]
    );
    println!(
        "  AI-2 only:  HapR={:.3}, bio={:.3}",
        final_ai2[4], final_ai2[6]
    );

    h.check_true("Dual HapR > CAI-1 only", final_dual[4] > final_cai1[4]);
    h.check_true("Dual HapR > AI-2 only", final_dual[4] > final_ai2[4]);
    h.check_true(
        "Dual HapR represses biofilm (less bio than single)",
        final_dual[6] < final_cai1[6].max(final_ai2[6]),
    );

    // ── Part 3: Determinism ───────────────────────────────────────────
    println!("\n--- Part 3: Determinism ---");

    let repeat = integrate(&ic, &params, dt, n_steps);
    let det_tol = f64_field(exp, "determinism_tolerance");
    h.check_true(
        "Deterministic trajectories agree",
        (0..7).all(|i| (final_dual[i] - repeat[i]).abs() < det_tol),
    );

    // ── Part 4: SNR — dual-signal has lower variance ───────────────────
    println!("\n--- Part 4: Dual-Signal Variance Advantage ---");

    let n_stoch_trials = 20;
    let noise_amp = 0.1;

    let dual_cdgs: Vec<f64> = (0..n_stoch_trials)
        .map(|i| stochastic_integrate(&ic, &params, dt, n_steps, noise_amp, 200 + i)[5])
        .collect();
    let cai1_cdgs: Vec<f64> = (0..n_stoch_trials)
        .map(|i| stochastic_integrate(&ic, &p_cai1, dt, n_steps, noise_amp, 300 + i)[5])
        .collect();

    let dual_std = groundspring::stats::std_dev(&dual_cdgs);
    let cai1_std = groundspring::stats::std_dev(&cai1_cdgs);
    let var_ratio_max = f64_field(exp, "dual_signal_variance_ratio_max");
    println!("  Dual σ(c-di-GMP)={dual_std:.4}, CAI-1 only σ={cai1_std:.4}");
    h.check_true(
        &format!("Dual-signal σ ≤ {var_ratio_max}× single-signal σ"),
        dual_std <= cai1_std * var_ratio_max,
    );

    // ── Part 5: Low noise agreement ───────────────────────────────────
    println!("\n--- Part 5: Low Noise Agreement ---");

    let stoch = stochastic_integrate(&ic, &params, dt, n_steps, 0.01, 99);
    let diff = (final_dual[5] - stoch[5]).abs();
    println!(
        "  Deterministic cdg={:.3}, stochastic={:.3}",
        final_dual[5], stoch[5]
    );
    h.check_max(
        "Low noise c-di-GMP agrees with deterministic",
        diff,
        f64_field(exp, "low_noise_cdg_agreement_tol"),
    );

    h.summary()
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
