// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Validation binary for rarefaction / sequencing noise analysis.
//!
//! Verifies Shannon diversity, multinomial sampling, and convergence
//! properties against analytical known values.

use groundspring::rarefaction::{
    evenness, multinomial_sample, rarefaction_at_depth, shannon_diversity, taxa_detected,
};
use groundspring::validate;

#[allow(clippy::too_many_lines)]
fn main() {
    validate::reset();

    println!("{}", "=".repeat(72));
    println!("groundSpring Rust Validation: Rarefaction & Sequencing Noise");
    println!("{}", "=".repeat(72));

    // ------------------------------------------------------------------
    // Shannon known values
    // ------------------------------------------------------------------
    println!("\n--- Shannon Diversity Known Values ---");

    // Uniform distribution: H' = ln(S)
    let uniform = [100_u64, 100, 100, 100];
    let expected_uniform = (4.0_f64).ln();
    let _ = validate::check_approx(
        "Shannon uniform(4)",
        shannon_diversity(&uniform),
        expected_uniform,
        1e-10,
    );

    // Single species: H' = 0
    let single = [1000_u64, 0, 0, 0];
    let _ = validate::check_approx(
        "Shannon single species",
        shannon_diversity(&single),
        0.0,
        1e-10,
    );

    // Evenness of uniform distribution = 1.0
    let _ = validate::check_approx("Evenness uniform", evenness(&uniform), 1.0, 1e-10);

    // ------------------------------------------------------------------
    // Multinomial sampling properties
    // ------------------------------------------------------------------
    println!("\n--- Multinomial Sampling ---");

    let abundances = [0.5, 0.3, 0.15, 0.05];

    // Determinism
    let r1 = multinomial_sample(&abundances, 10_000, 42);
    let r2 = multinomial_sample(&abundances, 10_000, 42);
    let _ = validate::check_true("Multinomial deterministic (same seed)", r1 == r2);

    // Different seeds differ
    let r3 = multinomial_sample(&abundances, 10_000, 99);
    let _ = validate::check_true("Multinomial differs with different seed", r1 != r3);

    // Total equals depth
    let total: u64 = r1.iter().sum();
    let _ = validate::check_true("Multinomial total == depth", total == 10_000);

    // All taxa detected at high depth
    let _ = validate::check_true("All 4 taxa detected at 10k depth", taxa_detected(&r1) == 4);

    // Proportions roughly match abundances (within 5% for 10k draws)
    for (i, &expected_frac) in abundances.iter().enumerate() {
        let observed_frac = r1[i] as f64 / 10_000.0;
        let _ = validate::check_approx(
            &format!("Taxon {i} proportion"),
            observed_frac,
            expected_frac,
            0.05,
        );
    }

    // ------------------------------------------------------------------
    // Rarefaction convergence
    // ------------------------------------------------------------------
    println!("\n--- Rarefaction Convergence ---");

    // 10 taxa with decreasing abundances
    let community: Vec<f64> = {
        let raw: Vec<f64> = (1..=10).rev().map(f64::from).collect();
        let total: f64 = raw.iter().sum();
        raw.iter().map(|&x| x / total).collect()
    };

    let low = rarefaction_at_depth(&community, 50, 30, 42);
    let high = rarefaction_at_depth(&community, 50_000, 30, 42);

    let _ = validate::check_true(
        "More reads detect more taxa",
        high.genera_mean >= low.genera_mean,
    );
    let _ = validate::check_true(
        "Shannon increases with depth",
        high.shannon_mean >= low.shannon_mean,
    );
    let _ = validate::check_true(
        "Low depth has higher Shannon variance",
        low.shannon_std >= high.shannon_std,
    );

    // At high depth, should detect all 10 taxa
    let _ = validate::check_approx("All 10 taxa at 50k depth", high.genera_mean, 10.0, 0.5);

    let exit_code = validate::summary("Rust Validation: Rarefaction");
    std::process::exit(exit_code);
}
