// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Validation binary for Experiment 010: Bistable Phenotypic Switching.
//!
//! Integrates the 5-variable bistable ODE from two initial conditions
//! and verifies that positive feedback creates two distinct attractors.
//!
//! All tolerances loaded from the benchmark JSON `expected_results` block.
//!
//! Reference: Fernandez, Waters et al. (2020) PNAS 117:26058-26068

use groundspring::bistable::{BistableParams, integrate, stochastic_integrate};
use groundspring::validate::ValidationHarness;
use groundspring_validate::{
    TOL_ANALYTICAL, array_field, f64_field, f64_range, print_provenance_header,
};
use serde_json::Value;

const BENCHMARK: &str = include_str!("../../../control/bistable_switching/benchmark_bistable.json");

fn params_from_json(model: &Value) -> BistableParams {
    let p = &model["parameters"];
    BistableParams {
        mu_max: f64_field(p, "mu_max"),
        k_cap: f64_field(p, "k_cap"),
        death_rate: f64_field(p, "death_rate"),
        k_ai_prod: f64_field(p, "k_ai_prod"),
        d_ai: f64_field(p, "d_ai"),
        k_hapr_max: f64_field(p, "k_hapr_max"),
        k_hapr_ai: f64_field(p, "k_hapr_ai"),
        n_hapr: f64_field(p, "n_hapr"),
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
        alpha_fb: f64_field(p, "alpha_fb"),
        n_fb: f64_field(p, "n_fb"),
        k_fb: f64_field(p, "k_fb"),
    }
}

#[expect(
    clippy::expect_used,
    reason = "validation harness: malformed benchmark config is a fatal infrastructure error"
)]
fn ic_from_json(model: &Value, key: &str) -> [f64; 5] {
    let arr = array_field(model, key);
    let mut ic = [0.0; 5];
    for (i, val) in ic.iter_mut().enumerate() {
        *val = arr[i].as_f64().expect("IC element must be a valid f64");
    }
    ic
}

struct SimCtx<'a> {
    ic_low: &'a [f64; 5],
    params: &'a BistableParams,
    dt: f64,
    n_steps: usize,
    final_low: &'a [f64; 5],
    final_high_cdg: f64,
}

fn validate_stochastic(h: &mut ValidationHarness, ctx: &SimCtx<'_>, exp: &Value) {
    // ── Part 5: Stochastic switching ──────────────────────────────────
    println!("\n--- Part 5: Stochastic Switching ---");

    let n_trials = 50;
    let threshold = f64::midpoint(ctx.final_low[3], ctx.final_high_cdg);
    let mut crossings = 0u32;
    for trial in 0..n_trials {
        let s = stochastic_integrate(ctx.ic_low, ctx.params, ctx.dt, ctx.n_steps, 0.5, 42 + trial);
        if s[3] > threshold {
            crossings += 1;
        }
    }
    #[expect(
        clippy::cast_precision_loss,
        reason = "crossings ≤ 50, n_trials=50 ≪ 2^53"
    )]
    let rate = f64::from(crossings) / n_trials as f64;
    println!("  Switching rate: {crossings}/{n_trials} = {rate:.3}");

    let (r_lo, r_hi) = f64_range(&exp["stochastic_switching_rate_range"]);
    h.check_range("Stochastic switching rate", rate, r_lo, r_hi);

    // ── Part 6: Low noise agreement ──────────────────────────────────
    println!("\n--- Part 6: Low Noise Agreement ---");

    let low_noise = stochastic_integrate(ctx.ic_low, ctx.params, ctx.dt, ctx.n_steps, 0.01, 99);
    let cdg_diff = (ctx.final_low[3] - low_noise[3]).abs();
    println!(
        "  Deterministic cdg={:.3}, low-noise stochastic={:.3}",
        ctx.final_low[3], low_noise[3]
    );
    h.check_max(
        "Low noise c-di-GMP agrees with deterministic",
        cdg_diff,
        f64_field(exp, "low_noise_agreement_tol"),
    );
}

fn run() -> i32 {
    let Ok(bench) = serde_json::from_str::<Value>(BENCHMARK) else {
        eprintln!("FATAL: invalid benchmark JSON");
        return 1;
    };
    let mut h = ValidationHarness::stdout("Rust Validation: Bistable Switching");

    print_provenance_header(&bench, "Bistable Phenotypic Switching");

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

    let ic_low = ic_from_json(model, "initial_low_cdg");
    let ic_high = ic_from_json(model, "initial_high_cdg");

    println!("  Model: 5-variable bistable ODE, dt={dt}, t_final={t_final}");

    // ── Part 1: Cell reaches carrying capacity ────────────────────────
    println!("\n--- Part 1: Cell Dynamics ---");

    let final_low = integrate(&ic_low, &params, dt, n_steps);
    let final_high = integrate(&ic_high, &params, dt, n_steps);

    println!(
        "  Low IC final:  cell={:.3}, cdg={:.3}, bio={:.3}",
        final_low[0], final_low[3], final_low[4]
    );
    println!(
        "  High IC final: cell={:.3}, cdg={:.3}, bio={:.3}",
        final_high[0], final_high[3], final_high[4]
    );

    h.check_approx(
        "Cell reaches capacity (low IC)",
        final_low[0],
        f64_field(pred, "cell_steady_state"),
        f64_field(exp, "cell_reaches_capacity_tol"),
    );
    h.check_approx(
        "Cell reaches capacity (high IC)",
        final_high[0],
        f64_field(pred, "cell_steady_state"),
        f64_field(exp, "cell_reaches_capacity_tol"),
    );

    // ── Part 2: Two distinct attractors ───────────────────────────────
    println!("\n--- Part 2: Attractor Separation ---");

    h.check_max(
        "Low IC → low c-di-GMP",
        final_low[3],
        f64_field(exp, "low_cdg_attractor_max"),
    );
    h.check_min(
        "High IC → high c-di-GMP",
        final_high[3],
        f64_field(exp, "high_cdg_attractor_min"),
    );
    h.check_max(
        "Low IC → low biofilm",
        final_low[4],
        f64_field(exp, "biofilm_low_attractor_max"),
    );
    h.check_min(
        "High IC → high biofilm",
        final_high[4],
        f64_field(exp, "biofilm_high_attractor_min"),
    );

    // ── Part 3: Monostable control ────────────────────────────────────
    println!("\n--- Part 3: Monostable Control ---");

    let mut mono_params = params;
    mono_params.alpha_fb = 0.0;
    let mono_low = integrate(&ic_low, &mono_params, dt, n_steps);
    let mono_high = integrate(&ic_high, &mono_params, dt, n_steps);

    let cdg_diff = (mono_low[3] - mono_high[3]).abs();
    println!(
        "  Mono low cdg={:.3}, mono high cdg={:.3}",
        mono_low[3], mono_high[3]
    );
    h.check_max(
        "Monostable: both ICs agree",
        cdg_diff,
        f64_field(exp, "monostable_attractors_agree_tol"),
    );

    // ── Part 4: Determinism ───────────────────────────────────────────
    println!("\n--- Part 4: Determinism ---");
    let repeat = integrate(&ic_low, &params, dt, n_steps);
    h.check_true(
        "Deterministic trajectories agree",
        (0..5).all(|i| (final_low[i] - repeat[i]).abs() < TOL_ANALYTICAL),
    );

    validate_stochastic(
        &mut h,
        &SimCtx {
            ic_low: &ic_low,
            params: &params,
            dt,
            n_steps,
            final_low: &final_low,
            final_high_cdg: final_high[3],
        },
        exp,
    );
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
