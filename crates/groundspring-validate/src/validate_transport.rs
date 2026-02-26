// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Validation binary for Experiment 012: Spin Chain Transport.
//!
//! Wavepacket dynamics in the 1D Almost-Mathieu tight-binding model:
//! ballistic transport (β≈1) for λ<2, localized (β≈0) for λ>2.
//!
//! Reference: Kachkovskiy (2016) Comm Math Phys 345:659-673

use groundspring::almost_mathieu;
use groundspring::anderson::lyapunov_exponent;
use groundspring::transport::{transport_exponent, tridiag_eigh, wavepacket_msd};
use groundspring::validate::ValidationHarness;
use groundspring_validate::{f64_field, f64_range, print_provenance_header, usize_field};
use serde_json::Value;

const BENCHMARK: &str =
    include_str!("../../../control/spin_transport/benchmark_spin_transport.json");

fn run() -> i32 {
    let bench: Value = serde_json::from_str(BENCHMARK).expect("valid benchmark JSON");
    let mut h = ValidationHarness::stdout("Rust Validation: Spin Chain Transport");

    println!("{}", "=".repeat(72));
    println!("groundSpring Rust Validation: Spin Chain Transport (Exp 012)");
    println!("{}", "=".repeat(72));
    print_provenance_header(&bench, "Spin Chain Transport");

    let model = &bench["model"];
    let exp = &bench["expected_results"];

    let n_sites = usize_field(model, "n_sites");
    let alpha = f64_field(model, "alpha");
    let theta = f64_field(model, "theta");
    let init_site = usize_field(model, "init_site");
    let lyap_n = usize_field(model, "lyapunov_n_sites");
    let lyap_e = f64_field(model, "lyapunov_energy");
    let norm_tol = f64_field(exp, "normalization_tolerance");

    let couplings: Vec<f64> = bench["model"]["coupling_strengths"]
        .as_array()
        .expect("coupling_strengths array")
        .iter()
        .map(|v| v.as_f64().expect("f64 coupling"))
        .collect();

    let times: Vec<f64> = bench["model"]["times"]
        .as_array()
        .expect("times array")
        .iter()
        .map(|v| v.as_f64().expect("f64 time"))
        .collect();

    let mut betas = Vec::new();

    for &lam in &couplings {
        println!("\n--- Coupling λ = {lam:.1} ---");

        let potential = almost_mathieu::potential(n_sites, lam, alpha, theta);
        let offdiag = vec![1.0; n_sites - 1];
        let (eigenvalues, eigenvectors) = tridiag_eigh(&potential, &offdiag);

        let mut msds_at_t = Vec::with_capacity(times.len());
        for (ti, &t) in times.iter().enumerate() {
            let (msd, norm) = wavepacket_msd(&eigenvalues, &eigenvectors, init_site, t);
            msds_at_t.push(msd);

            if ti == 0 || ti == times.len() - 1 {
                h.check_approx(&format!("Norm λ={lam:.1} t={t:.0}"), norm, 1.0, norm_tol);
            }
        }

        let beta = transport_exponent(&times, &msds_at_t);
        betas.push(beta);

        let sigma_final = msds_at_t.last().copied().unwrap_or(0.0).sqrt();
        println!(
            "  MSD(t={:.0}) = {:.4}, σ = {sigma_final:.4}",
            times.last().unwrap_or(&0.0),
            msds_at_t.last().unwrap_or(&0.0)
        );
        println!("  Transport exponent β = {beta:.4}");
    }

    // Part 1: Ballistic transport (λ < 2)
    println!("\n--- Validation: Transport Exponents ---");
    let (ballo, balhi) = f64_range(&exp["ballistic_beta_range"]);

    if let Some(i) = couplings.iter().position(|&c| (c - 0.5).abs() < 0.01) {
        h.check_range("Ballistic β (λ=0.5)", betas[i], ballo, balhi);
    }
    if let Some(i) = couplings.iter().position(|&c| (c - 1.0).abs() < 0.01) {
        h.check_range("Ballistic β (λ=1.0)", betas[i], ballo, balhi);
    }

    // Part 2: Localized transport (λ > 2)
    let loc_max = f64_field(exp, "localized_beta_max");
    if let Some(i) = couplings.iter().position(|&c| (c - 4.0).abs() < 0.01) {
        h.check_max("Localized β (λ=4.0)", betas[i], loc_max);
    }

    let msd_bound = f64_field(exp, "msd_localized_bounded_max");
    // Re-compute final MSD for λ=4.0 for the bound check
    if let Some(idx) = couplings.iter().position(|&c| (c - 4.0).abs() < 0.01) {
        let potential = almost_mathieu::potential(n_sites, 4.0, alpha, theta);
        let offdiag = vec![1.0; n_sites - 1];
        let (evals, evecs) = tridiag_eigh(&potential, &offdiag);
        let t_final = *times.last().unwrap_or(&1.0);
        let (msd_final, _) = wavepacket_msd(&evals, &evecs, init_site, t_final);
        h.check_max("Localized MSD bounded (λ=4.0)", msd_final, msd_bound);
        let _ = idx;
    }

    // Part 3: Critical point (λ = 2)
    let (crit_lo, crit_hi) = f64_range(&exp["critical_beta_range"]);
    if let Some(i) = couplings.iter().position(|&c| (c - 2.0).abs() < 0.01) {
        h.check_range("Critical β (λ=2.0)", betas[i], crit_lo, crit_hi);
    }

    // Part 4: Lyapunov cross-check
    println!("\n--- Lyapunov Cross-Check ---");
    let lyap_ext_max = f64_field(exp, "lyapunov_extended_max");
    let lyap_loc_min = f64_field(exp, "lyapunov_localized_min");

    let pot_ext = almost_mathieu::potential(lyap_n, 1.0, alpha, theta);
    let gamma_ext = lyapunov_exponent(&pot_ext, lyap_e);
    println!("  Lyapunov γ (λ=1.0): {gamma_ext:.6}");
    h.check_max("Lyapunov extended γ ≈ 0", gamma_ext, lyap_ext_max);

    let pot_loc = almost_mathieu::potential(lyap_n, 4.0, alpha, theta);
    let gamma_loc = lyapunov_exponent(&pot_loc, lyap_e);
    println!("  Lyapunov γ (λ=4.0): {gamma_loc:.6}");
    h.check_true("Lyapunov localized γ > threshold", gamma_loc > lyap_loc_min);

    // Part 5: Monotonicity
    println!("\n--- Monotonicity ---");
    // Tol 0.15: in deeply-localized regime (λ>>2) both β≈0 and small MSD
    // fluctuations make β slightly negative with noise on the order of 0.1.
    let monotonic = (0..betas.len().saturating_sub(1)).all(|i| betas[i] >= betas[i + 1] - 0.15);
    h.check_true("β decreases with λ", monotonic);

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
