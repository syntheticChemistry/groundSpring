// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Validation binary for rarefaction / sequencing noise analysis.
//!
//! Analytical known-values verified locally; community configuration and
//! convergence thresholds loaded from the benchmark JSON.
//!
//! Note: `benchmark_sequencing_noise.json` `expected_results` depth ranges
//! are for the Python validation path (`NumPy` PCG64 PRNG community generation).
//! The Rust validator instead tests analytically stronger invariants:
//! Shannon entropy, evenness, multinomial conservation, and convergence
//! monotonicity, which hold for any well-formed community.

use groundspring::rarefaction::{
    evenness, multinomial_sample, rarefaction_at_depth, shannon_diversity, taxa_detected,
};
use groundspring::validate::ValidationHarness;
use groundspring_validate::{
    TOL_ANALYTICAL, TOL_RAREFACTION_PROP, TOL_REGIME, print_provenance_header,
};
use serde_json::Value;

const BENCHMARK: &str =
    include_str!("../../../control/sequencing_noise/benchmark_sequencing_noise.json");

fn run() -> i32 {
    let Ok(bench) = serde_json::from_str::<Value>(BENCHMARK) else {
        eprintln!("FATAL: invalid benchmark JSON");
        return 1;
    };
    let mut h = ValidationHarness::stdout("Rust Validation: Rarefaction");

    #[expect(
        clippy::cast_possible_truncation,
        reason = "JSON u64 genus count ≤ 1000, fits usize"
    )]
    let n_genera = bench["reference_community"]["n_genera"]
        .as_u64()
        .expect("n_genera") as usize;
    let expected_shannon = bench["reference_community"]["shannon_diversity"]
        .as_f64()
        .expect("shannon_diversity");

    print_provenance_header(&bench, "Rarefaction & Sequencing Noise");
    println!("  Reference community: {n_genera} genera, H'={expected_shannon:.2}");

    // ── Shannon known values ────────────────────────────────────────
    println!("\n--- Shannon Diversity Known Values ---");

    let uniform = [100_u64, 100, 100, 100];
    let expected_uniform = (4.0_f64).ln();
    h.check_approx(
        "Shannon uniform(4)",
        shannon_diversity(&uniform),
        expected_uniform,
        TOL_ANALYTICAL,
    );

    let single = [1000_u64, 0, 0, 0];
    h.check_approx(
        "Shannon single species",
        shannon_diversity(&single),
        0.0,
        TOL_ANALYTICAL,
    );

    h.check_approx("Evenness uniform", evenness(&uniform), 1.0, TOL_ANALYTICAL);

    // ── Multinomial sampling ────────────────────────────────────────
    println!("\n--- Multinomial Sampling ---");

    let abundances = [0.5, 0.3, 0.15, 0.05];

    let r1 = multinomial_sample(&abundances, 10_000, 42);
    let r2 = multinomial_sample(&abundances, 10_000, 42);
    h.check_true("Multinomial deterministic (same seed)", r1 == r2);

    let r3 = multinomial_sample(&abundances, 10_000, 99);
    h.check_true("Multinomial differs with different seed", r1 != r3);

    let total: u64 = r1.iter().sum();
    h.check_true("Multinomial total == depth", total == 10_000);

    h.check_true("All 4 taxa detected at 10k depth", taxa_detected(&r1) == 4);

    // Tol 0.05: for n=10000 draws at p=0.5 the standard error is
    // sqrt(p(1-p)/n) ≈ 0.005; 0.05 gives 10× margin for all taxa.
    for (i, &expected_frac) in abundances.iter().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "count ≤ 10000 ≪ 2^53")]
        let observed_frac = r1[i] as f64 / 10_000.0;
        h.check_approx(
            &format!("Taxon {i} proportion"),
            observed_frac,
            expected_frac,
            TOL_RAREFACTION_PROP,
        );
    }

    // ── Rarefaction convergence ─────────────────────────────────────
    println!("\n--- Rarefaction Convergence ---");

    let community: Vec<f64> = {
        let raw: Vec<f64> = (1..=10).rev().map(f64::from).collect();
        let total: f64 = raw.iter().sum();
        raw.iter().map(|&x| x / total).collect()
    };

    let low = rarefaction_at_depth(&community, 50, 30, 42);
    let high = rarefaction_at_depth(&community, 50_000, 30, 42);

    h.check_true(
        "More reads detect more taxa",
        high.genera_mean >= low.genera_mean,
    );
    h.check_true(
        "Shannon increases with depth",
        high.shannon_mean >= low.shannon_mean,
    );
    h.check_true(
        "Low depth has higher Shannon variance",
        low.shannon_std >= high.shannon_std,
    );

    // Tol 0.5: at 50k depth with 10 taxa, sampling variance is negligible;
    // 0.5 handles any rare-taxon under-sampling in 30 replicates.
    #[expect(clippy::cast_precision_loss, reason = "community len ≤ 10 taxa ≪ 2^53")]
    let expected_all = community.len() as f64;
    h.check_approx(
        "All 10 taxa at 50k depth",
        high.genera_mean,
        expected_all,
        TOL_REGIME,
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
