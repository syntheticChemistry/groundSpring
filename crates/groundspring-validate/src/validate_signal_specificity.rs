// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Validation binary for Experiment 006: Enzymatic Signal Specificity.
//!
//! Models c-di-GMP signaling with Gillespie SSA (birth-death process).
//! Validates against analytical steady-state predictions and benchmarked
//! SNR values.
//!
//! Reference: Massie et al. (2012) PNAS 109:12746-51

use groundspring::gillespie::{
    birth_death_ssa, steady_state_mean, time_averaged_mean, time_averaged_variance,
};
use groundspring::validate::ValidationHarness;
use serde_json::Value;

const BENCHMARK: &str =
    include_str!("../../../control/signal_specificity/benchmark_signal_specificity.json");

fn f64_field(v: &Value, key: &str) -> f64 {
    v[key]
        .as_f64()
        .unwrap_or_else(|| panic!("missing f64 field: {key}"))
}

#[expect(clippy::cast_possible_truncation)]
fn usize_field(v: &Value, key: &str) -> usize {
    v[key]
        .as_u64()
        .unwrap_or_else(|| panic!("missing u64 field: {key}")) as usize
}

fn f64_range(arr: &Value) -> (f64, f64) {
    let a = arr.as_array().expect("expected JSON array for range");
    (
        a[0].as_f64().expect("range lower bound"),
        a[1].as_f64().expect("range upper bound"),
    )
}

#[expect(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::too_many_lines
)]
fn run() -> i32 {
    let bench: Value = serde_json::from_str(BENCHMARK).expect("valid benchmark JSON");
    let mut h = ValidationHarness::stdout("Rust Validation: Signal Specificity");

    let net = &bench["enzyme_network"];
    let sim = &bench["simulation"];
    let pred = &bench["analytical_predictions"];
    let exp = &bench["expected_results"];

    let n_dgc = usize_field(net, "n_dgc");
    let n_pde = usize_field(net, "n_pde");
    let k_syn = f64_field(net, "k_syn_per_dgc");
    let k_deg = f64_field(net, "k_deg_per_pde");
    let total_deg = n_pde as f64 * k_deg;

    let t_max = f64_field(sim, "t_max");
    let t_burnin = f64_field(sim, "t_burnin");
    let n_reps = usize_field(sim, "n_replicates");
    let seed = sim["seed"].as_u64().expect("seed");

    println!("{}", "=".repeat(72));
    println!("groundSpring Rust Validation: Signal Specificity (c-di-GMP)");
    println!("  Enzyme network: {n_dgc} DGCs, {n_pde} PDEs");
    println!("{}", "=".repeat(72));

    // ── Part 1: Analytical ────────────────────────────────────────────
    println!("\n--- Part 1: Analytical Steady State ---");

    let total_syn_basal = n_dgc as f64 * k_syn;
    let ss_mean = steady_state_mean(total_syn_basal, total_deg);
    let ss_std = ss_mean.sqrt();

    h.check_approx(
        "Analytical mean",
        ss_mean,
        f64_field(pred, "steady_state_mean"),
        0.01,
    );
    h.check_approx(
        "Analytical std",
        ss_std,
        f64_field(pred, "steady_state_std"),
        0.01,
    );

    // ── Part 2: Gillespie basal ───────────────────────────────────────
    println!("\n--- Part 2: Gillespie SSA Basal ---");

    let basal_rates: Vec<f64> = vec![k_syn; n_dgc];

    let mut basal_means = Vec::with_capacity(n_reps);
    let mut basal_vars = Vec::with_capacity(n_reps);

    for i in 0..n_reps {
        let traj = birth_death_ssa(
            &basal_rates,
            total_deg,
            ss_mean as u64,
            t_max,
            seed + i as u64,
        );
        let m = time_averaged_mean(&traj, t_burnin);
        let v = time_averaged_variance(&traj, t_burnin, m);
        basal_means.push(m);
        basal_vars.push(v);
    }

    let ensemble_mean: f64 = basal_means.iter().sum::<f64>() / n_reps as f64;
    let mean_var: f64 = basal_vars.iter().sum::<f64>() / n_reps as f64;
    let basal_std = (basal_vars.iter().sum::<f64>() / n_reps as f64).sqrt();

    println!("  Ensemble mean: {ensemble_mean:.3} (analytical: {ss_mean:.3})");

    h.check_approx(
        "Gillespie mean matches analytical",
        ensemble_mean,
        ss_mean,
        f64_field(exp, "steady_state_mean_tol"),
    );
    h.check_approx(
        "Gillespie variance ~ Poisson",
        mean_var,
        ss_mean,
        f64_field(exp, "steady_state_std_tol").powi(2),
    );

    // ── Part 3: Activated states & response ratios ────────────────────
    println!("\n--- Part 3: Activated States ---");

    let alphas: Vec<u64> = net["activation_ratios"]
        .as_array()
        .expect("activation_ratios array")
        .iter()
        .map(|v| v.as_u64().expect("alpha"))
        .collect();

    let mut activated_means = Vec::new();
    for &alpha in &alphas {
        let mut rates = vec![k_syn; n_dgc];
        rates[0] = k_syn * alpha as f64;

        let mut act_means = Vec::with_capacity(n_reps);
        for i in 0..n_reps {
            let traj = birth_death_ssa(
                &rates,
                total_deg,
                ss_mean as u64,
                t_max,
                seed + 10000 + alpha * 1000 + i as u64,
            );
            let m = time_averaged_mean(&traj, t_burnin);
            act_means.push(m);
        }
        let act_ensemble = act_means.iter().sum::<f64>() / n_reps as f64;
        activated_means.push((alpha, act_ensemble));
        println!("  α={alpha}: mean={act_ensemble:.3}");
    }

    let get_act_mean = |a: u64| {
        activated_means
            .iter()
            .find(|(al, _)| *al == a)
            .unwrap_or_else(|| panic!("no result for activation ratio α={a}"))
            .1
    };

    let rr10 = get_act_mean(10) / ensemble_mean;
    let rr20 = get_act_mean(20) / ensemble_mean;

    let (rr10_lo, rr10_hi) = f64_range(&exp["response_ratio_alpha_10_range"]);
    let (rr20_lo, rr20_hi) = f64_range(&exp["response_ratio_alpha_20_range"]);

    h.check_range("Response ratio α=10", rr10, rr10_lo, rr10_hi);
    h.check_range("Response ratio α=20", rr20, rr20_lo, rr20_hi);

    // ── Part 4: SNR ───────────────────────────────────────────────────
    println!("\n--- Part 4: Signal-to-Noise Ratio ---");

    let mut snr_values: Vec<(u64, f64)> = Vec::new();
    for &(alpha, act_mean) in &activated_means {
        let snr = if basal_std > 0.0 {
            (act_mean - ensemble_mean) / basal_std
        } else {
            0.0
        };
        snr_values.push((alpha, snr));
        println!("  SNR(α={alpha}): {snr:.3}");
    }

    let get_snr = |a: u64| {
        snr_values
            .iter()
            .find(|(al, _)| *al == a)
            .unwrap_or_else(|| panic!("no SNR result for α={a}"))
            .1
    };

    let (snr10_lo, snr10_hi) = f64_range(&exp["snr_alpha_10_range"]);
    let (snr20_lo, snr20_hi) = f64_range(&exp["snr_alpha_20_range"]);

    h.check_range("SNR α=10", get_snr(10), snr10_lo, snr10_hi);
    h.check_range("SNR α=20", get_snr(20), snr20_lo, snr20_hi);

    h.check_true(
        "SNR monotonically increases with α",
        snr_values.windows(2).all(|w| w[0].1 <= w[1].1),
    );
    h.check_true("SNR(α=2) > 0", get_snr(2) > 0.0);

    // ── Part 5: Determinism ───────────────────────────────────────────
    println!("\n--- Part 5: Determinism ---");

    let t1 = birth_death_ssa(&basal_rates, total_deg, 18, 50.0, 12345);
    let t2 = birth_death_ssa(&basal_rates, total_deg, 18, 50.0, 12345);
    h.check_true("Deterministic (same seed)", t1.states == t2.states);

    let t3 = birth_death_ssa(&basal_rates, total_deg, 18, 50.0, 99999);
    h.check_true("Differs (different seed)", t1.states != t3.states);

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
