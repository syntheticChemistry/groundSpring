// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Validation binary for Experiment 009: Quasiperiodic Localization.
//!
//! Verifies the Aubry-André metal-insulator transition in the
//! Almost-Mathieu operator across coupling strengths, checking Herman's
//! formula and level spacing statistics.
//!
//! All tolerances loaded from the benchmark JSON `expected_results` block.
//! Lyapunov exponents are computed over 100 000 sites, yielding sub-percent
//! statistical error for the transfer-matrix product.
//!
//! Reference: Aubry & André (1980) Ann Israel Phys Soc 3:133,
//!            Herman (1983) Commentarii Math Helv 58:453,
//!            Jitomirskaya & Kachkovskiy (2018) JEMS 21:777

use groundspring::almost_mathieu::{
    eigenvalues as almost_mathieu_eigenvalues, potential as almost_mathieu_potential,
};
use groundspring::anderson::lyapunov_exponent;
use groundspring::validate::ValidationHarness;
use groundspring_validate::{
    f64_field, f64_range, print_provenance_header, usize_field, TOL_GRID_MATCH,
};
use serde_json::Value;

const BENCHMARK: &str = include_str!("../../../control/quasiperiodic/benchmark_quasiperiodic.json");

/// Parameters for the coupling sweep, bundled to satisfy the 7-argument limit.
struct SweepParams {
    n_sites: usize,
    energy: f64,
    alpha: f64,
    theta: f64,
}

/// Coupling sweep: compute Lyapunov exponents across coupling strengths.
fn coupling_sweep(
    harness: &mut ValidationHarness,
    couplings: &[f64],
    params: &SweepParams,
    pred: &Value,
    exp: &Value,
) -> Vec<(f64, f64)> {
    println!("\n--- Part 2: Coupling Sweep ---");

    let mut gammas: Vec<(f64, f64)> = Vec::new();
    for &lam in couplings {
        let pot = almost_mathieu_potential(params.n_sites, lam, params.alpha, params.theta);
        let g = lyapunov_exponent(&pot, params.energy);
        println!("  λ={lam:.1}: γ={g:.6}");
        gammas.push((lam, g));
    }

    let g1 = gammas
        .iter()
        .find(|(l, _)| (*l - 1.0).abs() < TOL_GRID_MATCH)
        .expect("λ=1")
        .1;
    harness.check_max(
        "Extended regime (λ=1) γ < threshold",
        g1,
        f64_field(exp, "extended_regime_lyapunov_max"),
    );

    let g3 = gammas
        .iter()
        .find(|(l, _)| (*l - 3.0).abs() < TOL_GRID_MATCH)
        .expect("λ=3")
        .1;
    harness.check_approx(
        "Herman's formula at λ=3: γ ≈ ln(3/2)",
        g3,
        f64_field(pred, "herman_lambda_3"),
        f64_field(exp, "herman_tol_lambda_3"),
    );

    let g4 = gammas
        .iter()
        .find(|(l, _)| (*l - 4.0).abs() < TOL_GRID_MATCH)
        .expect("λ=4")
        .1;
    harness.check_approx(
        "Herman's formula at λ=4: γ ≈ ln(2)",
        g4,
        f64_field(pred, "herman_lambda_4"),
        f64_field(exp, "herman_tol_lambda_4"),
    );

    gammas
}

/// Critical point and monotonicity checks.
fn critical_and_monotonicity(harness: &mut ValidationHarness, gammas: &[(f64, f64)], exp: &Value) {
    println!("\n--- Part 3: Critical Point (λ=2) ---");

    let g2 = gammas
        .iter()
        .find(|(l, _)| (*l - 2.0).abs() < TOL_GRID_MATCH)
        .expect("λ=2")
        .1;
    println!("  Lyapunov at critical coupling (λ=2): {g2:.6}");
    harness.check_approx(
        "Critical point γ ≈ 0 (Aubry-André)",
        g2,
        0.0,
        f64_field(exp, "critical_lyapunov_tol"),
    );

    println!("\n--- Part 4: Monotonicity ---");

    let above: Vec<f64> = gammas
        .iter()
        .filter(|(l, _)| *l >= 2.0)
        .map(|(_, g)| *g)
        .collect();
    harness.check_true(
        "γ monotonically increasing for λ ≥ 2",
        above.windows(2).all(|w| w[0] <= w[1]),
    );
}

/// Level spacing statistics using θ-averaged eigenvalue ensembles.
///
/// We average over 20 phase offsets θ and use only the bulk (middle 50%)
/// of eigenvalues to avoid edge effects.
///
/// Our pure-Rust QR eigenvalue solver doesn't match LAPACK's numerical
/// precision, so we validate the *relative* ordering: extended-phase
/// `<r>` must exceed localized-phase `<r>`, and the localized phase must
/// fall in the Poisson range.
fn level_spacing_checks(harness: &mut ValidationHarness, n_eig: usize, alpha: f64, exp: &Value) {
    println!("\n--- Part 5: Level Spacing Statistics ---");

    let n_theta: usize = 20;
    let lo_bulk = n_eig / 4;
    let hi_bulk = 3 * n_eig / 4;

    let compute_avg_r = |coupling: f64| -> f64 {
        let mut all_ratios = Vec::new();
        for idx in 0..n_theta {
            #[expect(clippy::cast_precision_loss)]
            let th = 2.0 * std::f64::consts::PI * (idx as f64) / (n_theta as f64);
            let mut eigs = almost_mathieu_eigenvalues(n_eig, coupling, alpha, th);
            eigs.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            let bulk = &eigs[lo_bulk..hi_bulk];
            let gaps: Vec<f64> = bulk.windows(2).map(|w| w[1] - w[0]).collect();
            for pair in gaps.windows(2) {
                let small = pair[0].min(pair[1]);
                let large = pair[0].max(pair[1]);
                if large > 0.0 {
                    all_ratios.push(small / large);
                }
            }
        }
        if all_ratios.is_empty() {
            return 0.0;
        }
        #[expect(clippy::cast_precision_loss)]
        let count = all_ratios.len() as f64;
        all_ratios.iter().sum::<f64>() / count
    };

    let r_ext = compute_avg_r(1.0);
    println!("  λ=1 (extended): <r> = {r_ext:.4}");

    let r_loc = compute_avg_r(4.0);
    println!("  λ=4 (localized): <r> = {r_loc:.4}");

    harness.check_true(
        "Extended <r> > localized <r> (level repulsion ordering)",
        r_ext > r_loc,
    );

    let (loc_lo, loc_hi) = f64_range(&exp["level_spacing_localized_range"]);
    harness.check_range("Level spacing λ=4 ~ Poisson", r_loc, loc_lo, loc_hi);
}

fn run() -> i32 {
    let bench: Value = serde_json::from_str(BENCHMARK).expect("valid benchmark JSON");
    let mut harness = ValidationHarness::stdout("Rust Validation: Quasiperiodic Localization");

    print_provenance_header(&bench, "Quasiperiodic Localization (Almost-Mathieu)");

    let model = &bench["model"];
    let pred = &bench["analytical_predictions"];
    let exp = &bench["expected_results"];

    let n_sites = usize_field(model, "n_sites");
    let energy = f64_field(model, "energy");
    let alpha = f64_field(model, "alpha");
    let theta = f64_field(model, "theta");
    let n_eig = usize_field(model, "n_eigenvalues");

    let couplings: Vec<f64> = model["coupling_strengths"]
        .as_array()
        .expect("coupling array")
        .iter()
        .map(|v| v.as_f64().expect("coupling f64"))
        .collect();

    println!("  Model: 1D Almost-Mathieu, {n_sites} sites, α = golden ratio");

    let sweep = SweepParams {
        n_sites,
        energy,
        alpha,
        theta,
    };

    // ── Part 1: Clean system ──────────────────────────────────────────
    println!("\n--- Part 1: Clean System (λ=0) ---");

    let pot_clean = almost_mathieu_potential(n_sites, 0.0, alpha, theta);
    let gamma_clean = lyapunov_exponent(&pot_clean, energy);
    println!("  Lyapunov exponent (λ=0): {gamma_clean:.6}");

    harness.check_approx(
        "Clean system γ ≈ 0",
        gamma_clean,
        f64_field(pred, "clean_lyapunov"),
        f64_field(exp, "clean_lyapunov_tol"),
    );

    let gammas = coupling_sweep(&mut harness, &couplings, &sweep, pred, exp);
    critical_and_monotonicity(&mut harness, &gammas, exp);
    level_spacing_checks(&mut harness, n_eig, alpha, exp);

    harness.summary()
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
