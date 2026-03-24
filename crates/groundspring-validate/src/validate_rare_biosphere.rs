// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team
#![forbid(unsafe_code)]

//! Validation binary for Experiment 016: Rare Biosphere Signal Detection.
//!
//! When does a detected rare microbial lineage represent real biological
//! signal vs sequencing artifact?
//!
//! Reference: Anderson, Sogin, Baross (2015) FEMS Microbiol Ecol 91:fiv016

use groundspring::rare_biosphere::{
    abundance_occupancy, detection_power, detection_threshold, mean_chao1_at_depth,
    singleton_fraction, tier_detection_rate,
};
use groundspring::validate::ValidationHarness;
use groundspring_validate::{
    OrExit, TOL_DETERMINISM, array_field, f64_field, get_array, get_f64_range, get_u64,
    parse_benchmark, print_provenance_header, usize_field,
};
use serde_json::Value;

const BENCHMARK: &str =
    include_str!("../../../control/rare_biosphere/benchmark_rare_biosphere.json");

fn validate_chao1(
    h: &mut ValidationHarness,
    community: &[f64],
    depths: &[u64],
    n_reps: usize,
    base_seed: u64,
    exp: &Value,
) {
    println!("\n--- Part 1: Chao1 Richness ---");
    for &depth in depths {
        let (mean_c, mean_s) = mean_chao1_at_depth(community, depth, n_reps, base_seed + depth);
        println!("  D={depth:6}: S_obs={mean_s:.1}, Chao1={mean_c:.1}");
    }

    let (chao1_deep, sobs_deep) =
        mean_chao1_at_depth(community, 50_000, n_reps, base_seed + 50_000);
    let (chao1_shallow, sobs_shallow) =
        mean_chao1_at_depth(community, 100, n_reps, base_seed + 100);

    let (c_lo, c_hi) =
        get_f64_range(&exp["chao1_at_depth_50000_range"]).or_exit("chao1_at_depth_50000_range");

    h.check_range("Chao1 at D=50000 ≈ true richness", chao1_deep, c_lo, c_hi);
    h.check_true("Chao1 > S_obs at D=100", chao1_shallow > sobs_shallow);
    h.check_true(
        "All species detected at D=50000",
        sobs_deep >= f64_field(exp, "sobs_at_depth_50000") - 0.5,
    );
}

fn validate_detection(
    h: &mut ValidationHarness,
    community: &[f64],
    dom: (usize, usize),
    vr: (usize, usize),
    n_reps: usize,
    base_seed: u64,
    exp: &Value,
) {
    println!("\n--- Part 2: Detection Power ---");
    let dom_rate = tier_detection_rate(community, dom.0, dom.1, 100, n_reps, base_seed + 1000);
    let vr_rate_100 = tier_detection_rate(community, vr.0, vr.1, 100, n_reps, base_seed + 2000);
    let vr_rate_5000 = tier_detection_rate(community, vr.0, vr.1, 5000, n_reps, base_seed + 3000);

    println!(
        "  Dominant at D=100: {dom_rate:.3} (theory ≥ {:.3})",
        detection_power(0.06, 100)
    );
    println!(
        "  Very rare at D=100: {vr_rate_100:.3} (theory ≈ {:.3})",
        detection_power(0.003, 100)
    );
    println!(
        "  Very rare at D=5000: {vr_rate_5000:.3} (theory ≈ {:.3})",
        detection_power(0.003, 5000)
    );

    h.check_true(
        "Dominant detected at D=100",
        dom_rate >= f64_field(exp, "detection_rate_dominant_at_100_min"),
    );
    h.check_true(
        "Very rare rarely detected at D=100",
        vr_rate_100 <= f64_field(exp, "detection_rate_very_rare_at_100_max"),
    );
    h.check_true(
        "Very rare detected at D=5000",
        vr_rate_5000 >= f64_field(exp, "detection_rate_very_rare_at_5000_min"),
    );

    println!("\n--- Part 3: Detection Thresholds ---");
    let power_target = f64_field(exp, "detection_power_target");
    let d_003 = detection_threshold(0.003, power_target);
    let d_004 = detection_threshold(0.004, power_target);
    let d_008 = detection_threshold(0.008, power_target);
    let d_030 = detection_threshold(0.030, power_target);
    println!(
        "  p=0.003: D*={d_003}  p=0.004: D*={d_004}  p=0.008: D*={d_008}  p=0.030: D*={d_030}"
    );
    h.check_true(
        "Threshold monotonically decreases with abundance",
        d_003 > d_004 && d_004 > d_008 && d_008 > d_030,
    );
}

struct OccupancyCtx<'a> {
    community: &'a [f64],
    model: &'a Value,
    exp: &'a Value,
    dom: (usize, usize),
    vr: (usize, usize),
    depths: &'a [u64],
    n_reps: usize,
    base_seed: u64,
}

fn validate_occupancy_and_singletons(h: &mut ValidationHarness, ctx: &OccupancyCtx<'_>) {
    println!("\n--- Part 4: Abundance-Occupancy ---");
    let n_samples = usize_field(ctx.model, "n_samples_occupancy");
    let occ_depth = get_u64(ctx.model, "occupancy_depth").or_exit("occupancy_depth");
    let occupancy =
        abundance_occupancy(ctx.community, occ_depth, n_samples, ctx.base_seed + 50_000);

    #[expect(
        clippy::cast_precision_loss,
        reason = "tier span ≤ community size ≪ 2^53"
    )]
    let dom_occ: f64 =
        occupancy[ctx.dom.0..ctx.dom.1].iter().sum::<f64>() / (ctx.dom.1 - ctx.dom.0) as f64;
    #[expect(
        clippy::cast_precision_loss,
        reason = "tier span ≤ community size ≪ 2^53"
    )]
    let vr_occ: f64 =
        occupancy[ctx.vr.0..ctx.vr.1].iter().sum::<f64>() / (ctx.vr.1 - ctx.vr.0) as f64;
    println!("  Dominant occupancy: {dom_occ:.3}, Very rare: {vr_occ:.3}");
    h.check_true("Occupancy correlated with abundance", dom_occ > vr_occ);

    let rho = groundspring::stats::spearman_r(ctx.community, &occupancy);
    let rho_min = f64_field(ctx.exp, "spearman_occupancy_min");
    println!("  Spearman(abundance, occupancy) = {rho:.3}");
    h.check_true(
        &format!("Spearman ρ(abundance, occupancy) > {rho_min}"),
        rho > rho_min,
    );

    println!("\n--- Part 5: Singleton Fraction ---");
    let sf_low = singleton_fraction(
        ctx.community,
        ctx.depths[0],
        ctx.n_reps,
        ctx.base_seed + 60_000,
    );
    let sf_high = singleton_fraction(
        ctx.community,
        *ctx.depths.last().unwrap_or(&50_000),
        ctx.n_reps,
        ctx.base_seed + 61_000,
    );
    println!(
        "  D={}: {sf_low:.3}, D={}: {sf_high:.3}",
        ctx.depths[0],
        ctx.depths.last().unwrap_or(&50_000)
    );
    h.check_true("Singleton fraction decreases with depth", sf_low > sf_high);
}

fn run() -> i32 {
    let bench = parse_benchmark(BENCHMARK);
    let mut h = ValidationHarness::stdout("Rust Validation: Rare Biosphere");

    println!("{}", "=".repeat(72));
    println!("groundSpring Rust Validation: Rare Biosphere (Exp 016)");
    println!("{}", "=".repeat(72));
    print_provenance_header(&bench, "Rare Biosphere Signal Detection");

    let model = &bench["model"];
    let exp = &bench["expected_results"];

    let community: Vec<f64> = get_array(model, "community")
        .or_exit("community")
        .iter()
        .map(|v| v.as_f64().or_exit("f64"))
        .collect();
    let depths: Vec<u64> = get_array(model, "depths")
        .or_exit("depths")
        .iter()
        .map(|v| v.as_u64().or_exit("u64"))
        .collect();
    let n_reps = usize_field(model, "n_replicates");
    let base_seed = get_u64(model, "base_seed").or_exit("base_seed");

    let tiers = &model["tier_boundaries"];
    let tier = |name: &str| -> (usize, usize) {
        let arr = array_field(tiers, name);
        #[expect(
            clippy::cast_possible_truncation,
            reason = "JSON tier indices ≤ community size, fits usize"
        )]
        let lo = arr[0].as_u64().or_exit("lo") as usize;
        #[expect(
            clippy::cast_possible_truncation,
            reason = "JSON tier indices ≤ community size, fits usize"
        )]
        let hi = arr[1].as_u64().or_exit("hi") as usize;
        (lo, hi)
    };
    let dom = tier("dominant");
    let vr = tier("very_rare");

    validate_chao1(&mut h, &community, &depths, n_reps, base_seed, exp);
    validate_detection(&mut h, &community, dom, vr, n_reps, base_seed, exp);
    validate_occupancy_and_singletons(
        &mut h,
        &OccupancyCtx {
            community: &community,
            model,
            exp,
            dom,
            vr,
            depths: &depths,
            n_reps,
            base_seed,
        },
    );

    println!("\n--- Part 6: Determinism ---");
    let counts_a = groundspring::rarefaction::multinomial_sample(&community, 1000, 88888);
    let counts_b = groundspring::rarefaction::multinomial_sample(&community, 1000, 88888);
    h.check_true(
        "Multinomial deterministic (same seed)",
        counts_a == counts_b,
    );

    let (c1, _) = mean_chao1_at_depth(&community, 1000, 10, 77777);
    let (c2, _) = mean_chao1_at_depth(&community, 1000, 10, 77777);
    h.check_true("Chao1 deterministic", (c1 - c2).abs() < TOL_DETERMINISM);

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
