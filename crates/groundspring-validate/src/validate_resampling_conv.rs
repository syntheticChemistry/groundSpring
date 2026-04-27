// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team
#![forbid(unsafe_code)]
#![expect(
    clippy::expect_used,
    reason = "validation binaries use expect for compile-time benchmark JSON; missing data is a programmer error"
)]

//! Validation binary for Experiment 013: Resampling Convergence.
//!
//! Studies how quickly bootstrap and RAWR confidence intervals converge
//! as the number of replicates increases.
//!
//! Reference: Lee & Liu (2024) IEEE BIBM

use groundspring::bootstrap::{bootstrap_mean, rawr_mean};
use groundspring::prng::Xorshift64;
use groundspring::validate::ValidationHarness;
use groundspring_validate::{
    EPS_SAFE_DIV, OrExit, TOL_DETERMINISM, f64_field, get_array, get_u64, parse_benchmark,
    print_provenance_header, usize_field,
};

const BENCHMARK: &str =
    include_str!("../../../control/resampling_convergence/benchmark_resampling_convergence.json");

/// Gaussian CI convergence bound: final width ≤ initial × this factor.
/// A well-behaved estimator should not widen with more replicates;
/// 10% headroom absorbs seed-dependent fluctuations.
const CONVERGENCE_FACTOR_GAUSSIAN: f64 = 1.1;

/// Log-normal CI convergence bound (wider than Gaussian).
/// Skewed distributions cause higher variance in bootstrap CI width.
const CONVERGENCE_FACTOR_LOGNORMAL: f64 = 1.2;

/// Heavy-tail CI width comparison factor: heavy-tailed data should
/// produce wider CIs than Gaussian. The 20% discount guards against
/// the rare seed where heavy-tail data is well-concentrated.
const HEAVY_TAIL_WIDTH_FACTOR: f64 = 0.8;

fn generate_normal(n: usize, mu: f64, sigma: f64, seed: u64) -> Vec<f64> {
    let mut rng = Xorshift64::new(seed);
    (0..n).map(|_| rng.normal(mu, sigma)).collect()
}

fn ci_width(data: &[f64], n_boot: usize, confidence: f64, seed: u64, use_rawr: bool) -> f64 {
    let r = if use_rawr {
        rawr_mean(data, n_boot, confidence, seed)
    } else {
        bootstrap_mean(data, n_boot, confidence, seed)
    }
    .expect("validated inputs must not fail");
    r.ci_upper - r.ci_lower
}

#[expect(
    clippy::too_many_lines,
    reason = "validation harness with multiple convergence check sections"
)]
fn run() -> i32 {
    let bench = parse_benchmark(BENCHMARK);
    let mut h = ValidationHarness::stdout("Rust Validation: Resampling Convergence");

    println!("{}", "=".repeat(72));
    println!("groundSpring Rust Validation: Resampling Convergence (Exp 013)");
    println!("{}", "=".repeat(72));
    print_provenance_header(&bench, "Resampling Convergence");

    let model = &bench["model"];
    let exp = &bench["expected_results"];

    let data_n = usize_field(model, "data_n");
    let confidence = f64_field(model, "confidence");
    let max_rel_change = f64_field(exp, "relative_width_change_5k_to_10k_max");

    let replicate_counts: Vec<usize> = get_array(&bench["model"], "replicate_counts")
        .or_exit("replicate_counts array")
        .iter()
        .map(|v| {
            #[expect(
                clippy::cast_possible_truncation,
                reason = "JSON replicate counts ≤ 10000, fits usize"
            )]
            let n = v.as_u64().or_exit("u64 count") as usize;
            n
        })
        .collect();

    // Part 1: Gaussian convergence
    println!("\n--- Part 1: Gaussian (μ=5.0, σ=2.0) ---");
    let gauss_mu = f64_field(&model["gaussian"], "mu");
    let gauss_sigma = f64_field(&model["gaussian"], "sigma");
    let gauss_seed = get_u64(&model["gaussian"], "seed").or_exit("seed");

    let data_gauss = generate_normal(data_n, gauss_mu, gauss_sigma, gauss_seed);

    let mut boot_widths: Vec<f64> = Vec::new();
    let mut rawr_widths: Vec<f64> = Vec::new();

    for &n_boot in &replicate_counts {
        let seed_offset = n_boot as u64;
        let bw = ci_width(
            &data_gauss,
            n_boot,
            confidence,
            gauss_seed + seed_offset,
            false,
        );
        let rw = ci_width(
            &data_gauss,
            n_boot,
            confidence,
            gauss_seed + seed_offset + 50000,
            true,
        );
        boot_widths.push(bw);
        rawr_widths.push(rw);
        println!("  n={n_boot:5}: bootstrap={bw:.4}  RAWR={rw:.4}");
    }

    h.check_true(
        "Bootstrap width converges (Gaussian)",
        *boot_widths.last().unwrap_or(&f64::MAX)
            <= boot_widths[0] * CONVERGENCE_FACTOR_GAUSSIAN,
    );
    h.check_true(
        "RAWR width converges (Gaussian)",
        *rawr_widths.last().unwrap_or(&f64::MAX)
            <= rawr_widths[0] * CONVERGENCE_FACTOR_GAUSSIAN,
    );

    let len = boot_widths.len();
    if len >= 2 {
        let rel_boot = (boot_widths[len - 1] - boot_widths[len - 2]).abs()
            / boot_widths[len - 2].max(EPS_SAFE_DIV);
        let rel_rawr = (rawr_widths[len - 1] - rawr_widths[len - 2]).abs()
            / rawr_widths[len - 2].max(EPS_SAFE_DIV);
        println!("  Relative change 5k→10k: bootstrap={rel_boot:.4} RAWR={rel_rawr:.4}");

        h.check_max("Bootstrap converged (5k→10k)", rel_boot, max_rel_change);
        h.check_max("RAWR converged (5k→10k)", rel_rawr, max_rel_change);
    }

    // Part 2: Log-normal convergence
    println!("\n--- Part 2: Log-Normal ---");
    let ln_seed = get_u64(&model["lognormal"], "seed").or_exit("seed");
    let ln_mu = f64_field(&model["lognormal"], "mu_ln");
    let ln_sigma = f64_field(&model["lognormal"], "sigma_ln");

    let normals = generate_normal(data_n, ln_mu, ln_sigma, ln_seed);
    let data_lognorm: Vec<f64> = normals.iter().map(|&x| x.exp()).collect();

    let mut boot_widths_ln: Vec<f64> = Vec::new();
    for &n_boot in &replicate_counts {
        let seed_offset = n_boot as u64;
        let bw = ci_width(
            &data_lognorm,
            n_boot,
            confidence,
            ln_seed + seed_offset,
            false,
        );
        boot_widths_ln.push(bw);
    }
    h.check_true(
        "Log-normal width converges",
        *boot_widths_ln.last().unwrap_or(&f64::MAX)
            <= boot_widths_ln[0] * CONVERGENCE_FACTOR_LOGNORMAL,
    );

    // Part 3: Heavy-tailed convergence (approximate t-distribution using normal)
    println!("\n--- Part 3: Heavy-Tailed ---");
    let ht_seed = get_u64(&model["heavy_tail"], "seed").or_exit("seed");
    let ht_loc = f64_field(&model["heavy_tail"], "loc");
    let ht_scale = f64_field(&model["heavy_tail"], "scale");
    let ht_df = f64_field(&model["heavy_tail"], "df");

    // Generate approximate t-distribution via ratio of normals
    let mut rng_ht = Xorshift64::new(ht_seed);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "df from JSON positive, t-dist DOF fits usize"
    )]
    let df_int = ht_df as usize;
    let data_ht: Vec<f64> = (0..data_n)
        .map(|_| {
            let z = rng_ht.normal(0.0, 1.0);
            let mut chi2_sum = 0.0;
            for _ in 0..df_int {
                let u = rng_ht.normal(0.0, 1.0);
                chi2_sum += u * u;
            }
            let t_val = z / (chi2_sum / ht_df).sqrt();
            t_val * ht_scale + ht_loc
        })
        .collect();

    let bw_ht = ci_width(&data_ht, 10000, confidence, ht_seed + 10000, false);
    let bw_g = *boot_widths.last().unwrap_or(&1.0);
    println!("  Heavy-tail width at n=10k: {bw_ht:.4} (Gaussian: {bw_g:.4})");

    h.check_true(
        "Heavy-tail wider than Gaussian",
        bw_ht > bw_g * HEAVY_TAIL_WIDTH_FACTOR,
    );

    // Part 4: Determinism
    println!("\n--- Part 4: Determinism ---");
    let det_data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

    let w1 = ci_width(&det_data, 500, 0.95, 7777, false);
    let w2 = ci_width(&det_data, 500, 0.95, 7777, false);
    h.check_true("Bootstrap deterministic", (w1 - w2).abs() < TOL_DETERMINISM);

    let w3 = ci_width(&det_data, 500, 0.95, 8888, true);
    let w4 = ci_width(&det_data, 500, 0.95, 8888, true);
    h.check_true("RAWR deterministic", (w3 - w4).abs() < TOL_DETERMINISM);

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
