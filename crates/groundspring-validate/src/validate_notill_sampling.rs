// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Validation binary for Experiment 023: No-Till vs Tilled 16S Sampling Design.
//!
//! Compares rarefaction curves for high-diversity (no-till) vs low-diversity
//! (tilled) synthetic soil communities. Determines minimum sequencing depth
//! to distinguish soil management regimes.
//!
//! Reference: Anderson, Sogin, Baross (2015) FEMS Microbiol Ecol 91:fiu016

use groundspring::prng::Xorshift64;
use groundspring::rare_biosphere::chao1;
use groundspring::rarefaction::{multinomial_sample_batch, shannon_diversity};
use groundspring::validate::ValidationHarness;
use groundspring_validate::{f64_field, f64_range, print_provenance_header, usize_field};
use serde_json::Value;

const BENCHMARK: &str =
    include_str!("../../../control/notill_sampling/benchmark_notill_sampling.json");

fn generate_community(n_genera: usize, mu: f64, sigma: f64, seed: u64) -> Vec<f64> {
    let mut rng = Xorshift64::new(seed);
    let mut raw: Vec<f64> = (0..n_genera)
        .map(|_| {
            let z = rng.next_normal();
            sigma.mul_add(z, mu).exp().max(groundspring::tol::EXACT)
        })
        .collect();
    let total: f64 = raw.iter().sum();
    for v in &mut raw {
        *v /= total;
    }
    raw
}

fn true_shannon(abundances: &[f64]) -> f64 {
    abundances
        .iter()
        .filter(|&&p| p > 0.0)
        .map(|&p| -p * p.ln())
        .sum()
}

struct RarefactionResult {
    shannon_mean: f64,
    chao1_mean: f64,
}

fn rarefy_at_depth(
    community: &[f64],
    depth: u64,
    n_reps: usize,
    base_seed: u64,
) -> RarefactionResult {
    let batch_seed = base_seed.wrapping_add(depth);
    let batch = multinomial_sample_batch(community, depth, n_reps, batch_seed);
    let mut shannon_sum = 0.0;
    let mut chao1_sum = 0.0;
    for counts in &batch {
        shannon_sum += shannon_diversity(counts);
        chao1_sum += chao1(counts);
    }
    #[expect(clippy::cast_precision_loss, reason = "n_reps ≤ 20")]
    let n = n_reps as f64;
    RarefactionResult {
        shannon_mean: shannon_sum / n,
        chao1_mean: chao1_sum / n,
    }
}

fn find_saturation_depth(results: &[(u64, f64)], true_h: f64, threshold_pct: f64) -> f64 {
    for &(depth, h) in results {
        if true_h > 0.0 {
            let pct_diff = ((h - true_h) / true_h).abs() * 100.0;
            if pct_diff <= threshold_pct {
                #[expect(clippy::cast_precision_loss, reason = "depth ≤ 50000")]
                return depth as f64;
            }
        }
    }
    -1.0
}

struct CurveData {
    curve: Vec<(u64, f64)>,
    high: RarefactionResult,
    shannon_1k: f64,
}

fn build_curve(
    community: &[f64],
    depths: &[u64],
    n_reps: usize,
    base_seed: u64,
    label: &str,
) -> CurveData {
    let max_depth = *depths.last().expect("non-empty depths");
    let mut curve = Vec::with_capacity(depths.len());
    let mut high = RarefactionResult {
        shannon_mean: 0.0,
        chao1_mean: 0.0,
    };
    let mut shannon_1k = 0.0_f64;

    for &depth in depths {
        let r = rarefy_at_depth(community, depth, n_reps, base_seed);
        println!(
            "  {label} D={depth:6}: H'={:.4}, Chao1={:.1}",
            r.shannon_mean, r.chao1_mean
        );
        curve.push((depth, r.shannon_mean));
        if depth == 1000 {
            shannon_1k = r.shannon_mean;
        }
        if depth == max_depth {
            high = r;
        }
    }
    CurveData {
        curve,
        high,
        shannon_1k,
    }
}

fn validate_diversity(
    h: &mut ValidationHarness,
    notill: &CurveData,
    tilled: &CurveData,
    notill_true_h: f64,
    tilled_true_h: f64,
    exp: &Value,
) {
    h.check_true(
        "No-till has higher diversity than tilled at high depth",
        notill.high.shannon_mean > tilled.high.shannon_mean,
    );
    let (nlo, nhi) = f64_range(&exp["notill_shannon_range"]);
    h.check_range(
        "No-till Shannon at high depth",
        notill.high.shannon_mean,
        nlo,
        nhi,
    );
    let (tlo, thi) = f64_range(&exp["tilled_shannon_range"]);
    h.check_range(
        "Tilled Shannon at high depth",
        tilled.high.shannon_mean,
        tlo,
        thi,
    );

    h.check_true(
        "No-till Chao1 higher than tilled at high depth",
        notill.high.chao1_mean > tilled.high.chao1_mean,
    );
    h.check_true(
        "Communities distinguishable at 1000 reads",
        notill.shannon_1k > tilled.shannon_1k,
    );

    let sat_pct = exp
        .get("saturation_threshold_pct")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(5.0);
    let sat_n = find_saturation_depth(&notill.curve, notill_true_h, sat_pct);
    let sat_t = find_saturation_depth(&tilled.curve, tilled_true_h, sat_pct);
    println!("  Saturation: no-till={sat_n}, tilled={sat_t}");

    let notill_sat_range = f64_range(&exp["saturation_depth_notill_range"]);
    h.check_range(
        "No-till saturation depth",
        sat_n,
        notill_sat_range.0,
        notill_sat_range.1,
    );
    let tilled_sat_range = f64_range(&exp["saturation_depth_tilled_range"]);
    h.check_range(
        "Tilled saturation depth",
        sat_t,
        tilled_sat_range.0,
        tilled_sat_range.1,
    );
}

fn run() -> i32 {
    let bench: Value = serde_json::from_str(BENCHMARK).expect("valid benchmark JSON");
    let mut h = ValidationHarness::stdout("Rust Validation: No-Till vs Tilled Sampling");
    print_provenance_header(&bench, "No-Till vs Tilled Sampling (Exp 023)");

    let communities = &bench["communities"];
    let rarefaction = &bench["rarefaction"];
    let exp = &bench["expected"];

    let base_seed = rarefaction["seed"].as_u64().unwrap_or(42);
    let n_reps = usize_field(rarefaction, "n_replicates");
    let depths: Vec<u64> = groundspring_validate::get_array(rarefaction, "depths")
        .expect("benchmark depths array")
        .iter()
        .enumerate()
        .map(|(i, v)| {
            v.as_u64()
                .unwrap_or_else(|| panic!("benchmark depths[{i}]: expected u64"))
        })
        .collect();

    println!("\n--- Part 1: Synthetic Communities ---");
    let notill_cfg = &communities["notill"];
    let tilled_cfg = &communities["tilled"];

    let load_community = |cfg: &Value, fallback_seed: u64| -> Vec<f64> {
        cfg["abundances"].as_array().map_or_else(
            || {
                generate_community(
                    usize_field(cfg, "n_genera"),
                    f64_field(cfg, "log_normal_mu"),
                    f64_field(cfg, "log_normal_sigma"),
                    fallback_seed,
                )
            },
            |arr| arr.iter().map(|v| v.as_f64().unwrap_or(0.0)).collect(),
        )
    };
    let notill_comm = load_community(notill_cfg, base_seed);
    let tilled_comm = load_community(tilled_cfg, base_seed + 1000);
    let notill_true_h = true_shannon(&notill_comm);
    let tilled_true_h = true_shannon(&tilled_comm);
    println!(
        "  No-till: {} genera, H'={notill_true_h:.4}",
        notill_comm.len()
    );
    println!(
        "  Tilled:  {} genera, H'={tilled_true_h:.4}",
        tilled_comm.len()
    );

    println!("\n--- Part 2: Rarefaction ---");
    let notill_data = build_curve(&notill_comm, &depths, n_reps, base_seed, "no-till");
    let tilled_data = build_curve(&tilled_comm, &depths, n_reps, base_seed, "tilled");

    println!("\n--- Part 3: Validate ---");
    validate_diversity(
        &mut h,
        &notill_data,
        &tilled_data,
        notill_true_h,
        tilled_true_h,
        exp,
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
