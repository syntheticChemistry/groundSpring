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
use groundspring_validate::{f64_field, f64_range, print_provenance_header, usize_field};
use serde_json::Value;

const BENCHMARK: &str =
    include_str!("../../../control/anderson_localization/benchmark_anderson_localization.json");

/// Disorder sweep: compute Lyapunov exponents across disorder strengths.
///
/// Tolerances: `strong_disorder_lyapunov_min` from JSON ensures γ > 0.3
/// at W=8, which is the deep-localization regime where the transfer-matrix
/// product converges within 10⁴ sites.
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
        .expect("disorder W=8.0 not in sweep")
        .1;
    h.check_min(
        "Strong disorder (W=8) γ",
        gamma_8,
        f64_field(exp, "strong_disorder_lyapunov_min"),
    );

    gammas
}

/// Thouless scaling and localization-length checks.
///
/// Thouless coefficient C = ξ·W² ≈ 96 (Derrida-Gardner); the range
/// [60, 140] from JSON absorbs finite-size effects at 10⁴ sites.
fn thouless_and_localization(h: &mut ValidationHarness, gammas: &[(f64, f64)], exp: &Value) {
    println!("\n--- Part 3: Thouless Scaling ---");

    let gamma_1 = gammas
        .iter()
        .find(|(w, _)| (*w - 1.0).abs() < 0.01)
        .expect("disorder W=1.0 not in sweep")
        .1;
    let xi_1 = localization_length(gamma_1);
    println!("  At W=1: ξ={xi_1:.1}, C = ξ·W² = {xi_1:.1}");

    let (c_lo, c_hi) = f64_range(&exp["thouless_ratio_range"]);
    h.check_range("Thouless coefficient C", xi_1, c_lo, c_hi);

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

#[expect(clippy::float_cmp)]
fn run() -> i32 {
    let bench: Value = serde_json::from_str(BENCHMARK).expect("valid benchmark JSON");
    let mut h = ValidationHarness::stdout("Rust Validation: Anderson Localization");

    print_provenance_header(&bench, "Anderson Localization");

    let model = &bench["model"];
    let pred = &bench["analytical_predictions"];
    let exp = &bench["expected_results"];

    let n_sites = usize_field(model, "n_sites");
    let n_real = usize_field(model, "n_realizations");
    let energy = f64_field(model, "energy");

    let disorders: Vec<f64> = model["disorder_strengths"]
        .as_array()
        .expect("disorder array")
        .iter()
        .map(|v| v.as_f64().expect("disorder f64"))
        .collect();

    println!("  Model: 1D tight-binding, {n_sites} sites, {n_real} realizations");

    // ── Part 1: Clean system ──────────────────────────────────────────
    // Tol: `clean_lyapunov_tol` from JSON (0.001); for W=0 the transfer
    // matrix is a rotation and γ = 0 analytically. Tolerance absorbs
    // finite-chain drift at 10⁵ sites.
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

    h.summary()
}

fn main() {
    std::process::exit(run());
}

#[cfg(test)]
mod tests {
    #[test]
    fn validation_passes() {
        assert_eq!(super::run(), 0);
    }
}
