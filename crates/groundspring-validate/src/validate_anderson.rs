// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Validation binary for Experiment 008: Anderson Localization.
//!
//! Computes Lyapunov exponents for 1D Anderson model across disorder
//! strengths, verifying localization theory and Thouless scaling.
//!
//! All tolerances loaded from the benchmark JSON `expected_results` block.
//! Stochastic tolerances are justified by the large chain length (10 000
//! sites × 20 realizations) which yields sub-percent statistical error.
//!
//! Reference: Anderson (1958) Phys Rev 109:1492,
//!            Bourgain & Kachkovskiy (2018) GAFA 29:3-43

use groundspring::anderson::{
    anderson_potential, localization_length, lyapunov_averaged, lyapunov_exponent,
};
use groundspring::validate::ValidationHarness;
use serde_json::Value;

const BENCHMARK: &str =
    include_str!("../../../control/anderson_localization/benchmark_anderson_localization.json");

fn f64_field(v: &Value, key: &str) -> f64 {
    v[key]
        .as_f64()
        .unwrap_or_else(|| panic!("missing f64 field: {key}"))
}

/// Disorder sweep: compute Lyapunov exponents across disorder strengths.
fn disorder_sweep(
    h: &mut ValidationHarness,
    disorders: &[f64],
    n_sites: usize,
    energy: f64,
    n_real: usize,
    exp: &Value,
) -> Vec<(f64, f64)> {
    println!("\n--- Part 2: Disorder Sweep ---");

    let mut gammas: Vec<(f64, f64)> = Vec::new();
    for &w in disorders {
        let g = lyapunov_averaged(n_sites, w, energy, n_real, 42);
        let xi = localization_length(g);
        let xi_str = if xi < 1e6 {
            format!("{xi:.1}")
        } else {
            "∞".to_string()
        };
        println!("  W={w:.1}: γ={g:.6}, ξ={xi_str}");
        gammas.push((w, g));
    }

    let nonzero_gammas: Vec<f64> = gammas
        .iter()
        .filter(|(w, _)| *w > 0.0)
        .map(|(_, g)| *g)
        .collect();

    h.check_true(
        "All disordered states have γ > 0",
        nonzero_gammas.iter().all(|g| *g > 0.0),
    );
    h.check_true(
        "γ increases monotonically with W",
        nonzero_gammas.windows(2).all(|w| w[0] <= w[1]),
    );

    let gamma_8 = gammas
        .iter()
        .find(|(w, _)| (*w - 8.0).abs() < 0.01)
        .unwrap()
        .1;
    h.check_min(
        "Strong disorder (W=8) γ",
        gamma_8,
        f64_field(exp, "strong_disorder_lyapunov_min"),
    );

    gammas
}

/// Thouless scaling and localization-length checks.
fn thouless_and_localization(h: &mut ValidationHarness, gammas: &[(f64, f64)], exp: &Value) {
    println!("\n--- Part 3: Thouless Scaling ---");

    let gamma_1 = gammas
        .iter()
        .find(|(w, _)| (*w - 1.0).abs() < 0.01)
        .unwrap()
        .1;
    let xi_1 = localization_length(gamma_1);
    println!("  At W=1: ξ={xi_1:.1}, C = ξ·W² = {xi_1:.1}");

    let c_range = exp["thouless_ratio_range"].as_array().expect("C range");
    h.check_range(
        "Thouless coefficient C",
        xi_1,
        c_range[0].as_f64().unwrap(),
        c_range[1].as_f64().unwrap(),
    );

    println!("\n--- Part 4: Localization Length vs Disorder ---");

    let nonzero: Vec<(f64, f64)> = gammas.iter().copied().filter(|(w, _)| *w > 0.0).collect();
    let xi_values: Vec<f64> = nonzero
        .iter()
        .map(|(_, g)| localization_length(*g))
        .collect();
    for (i, (w, _)) in nonzero.iter().enumerate() {
        let xi_str = if xi_values[i] < 1e6 {
            format!("{:.1}", xi_values[i])
        } else {
            "∞".to_string()
        };
        println!("  W={w:.1}: ξ={xi_str}");
    }

    h.check_true(
        "ξ decreases with increasing W",
        xi_values.windows(2).all(|w| w[0] >= w[1]),
    );
}

#[expect(clippy::cast_possible_truncation, clippy::float_cmp)]
fn main() {
    let bench: Value = serde_json::from_str(BENCHMARK).expect("valid benchmark JSON");
    let mut h = ValidationHarness::stdout("Rust Validation: Anderson Localization");

    let model = &bench["model"];
    let pred = &bench["analytical_predictions"];
    let exp = &bench["expected_results"];

    let n_sites = model["n_sites"].as_u64().expect("n_sites") as usize;
    let n_real = model["n_realizations"].as_u64().expect("n_realizations") as usize;
    let energy = f64_field(model, "energy");

    let disorders: Vec<f64> = model["disorder_strengths"]
        .as_array()
        .expect("disorder array")
        .iter()
        .map(|v| v.as_f64().expect("disorder f64"))
        .collect();

    println!("{}", "=".repeat(72));
    println!("groundSpring Rust Validation: Anderson Localization");
    println!("  Model: 1D tight-binding, {n_sites} sites, {n_real} realizations");
    println!("{}", "=".repeat(72));

    // ── Part 1: Clean system ──────────────────────────────────────────
    println!("\n--- Part 1: Clean System (W=0) ---");

    let gamma_clean = lyapunov_averaged(n_sites, 0.0, energy, 1, 42);
    println!("  Lyapunov exponent (W=0): {gamma_clean:.6}");

    h.check_approx(
        "Clean system γ ≈ 0",
        gamma_clean,
        f64_field(pred, "clean_lyapunov"),
        f64_field(exp, "clean_lyapunov_tol"),
    );

    let gammas = disorder_sweep(&mut h, &disorders, n_sites, energy, n_real, exp);
    thouless_and_localization(&mut h, &gammas, exp);

    // ── Part 5: Determinism ───────────────────────────────────────────
    println!("\n--- Part 5: Determinism ---");

    let p1 = anderson_potential(1000, 2.0, 12345);
    let p2 = anderson_potential(1000, 2.0, 12345);
    h.check_true("Potential deterministic", p1 == p2);

    let g1 = lyapunov_exponent(&p1, 0.0);
    let g2 = lyapunov_exponent(&p2, 0.0);
    h.check_true("Lyapunov deterministic", g1 == g2);

    let exit_code = h.summary();
    std::process::exit(exit_code);
}
