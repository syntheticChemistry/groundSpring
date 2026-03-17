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
use groundspring_validate::{
    OrExit, TOL_STOCHASTIC_MEAN, f64_field, f64_range, get_array, get_u64, parse_benchmark,
    print_provenance_header, usize_field,
};
use serde_json::Value;

const BENCHMARK: &str =
    include_str!("../../../control/signal_specificity/benchmark_signal_specificity.json");

/// Enzyme network parameters extracted from the benchmark JSON.
struct EnzymeNetwork {
    n_dgc: usize,
    k_syn: f64,
    total_deg: f64,
    basal_rates: Vec<f64>,
}

/// Simulation parameters extracted from the benchmark JSON.
struct SimConfig {
    t_max: f64,
    t_burnin: f64,
    n_reps: usize,
    seed: u64,
}

/// Validate analytical steady-state predictions.
///
/// Tol 0.01: birth-death steady state is `S* = total_syn/total_deg`;
/// for `n_dgc`=40, `k_syn`=1, `n_pde`=22, `k_deg`=0.1 the analytical
/// values are exact fractions. 0.01 absorbs f64 rounding only.
fn validate_analytical(h: &mut ValidationHarness, net: &EnzymeNetwork, pred: &Value) -> (f64, f64) {
    println!("\n--- Part 1: Analytical Steady State ---");

    #[expect(clippy::cast_precision_loss, reason = "n_dgc ≤ 100 ≪ 2^53")]
    let total_syn_basal = net.n_dgc as f64 * net.k_syn;
    let ss_mean = steady_state_mean(total_syn_basal, net.total_deg);
    let ss_std = ss_mean.sqrt();

    h.check_approx(
        "Analytical mean",
        ss_mean,
        f64_field(pred, "steady_state_mean"),
        TOL_STOCHASTIC_MEAN,
    );
    h.check_approx(
        "Analytical std",
        ss_std,
        f64_field(pred, "steady_state_std"),
        TOL_STOCHASTIC_MEAN,
    );

    (ss_mean, ss_std)
}

/// Validate Gillespie SSA basal state against analytical predictions.
///
/// Tol: `steady_state_mean_tol` and `steady_state_std_tol` from JSON;
/// with 200 replicates of `t_max`=500 (burnin=100), the ensemble mean
/// converges to within ±1 of the analytical value.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "ss_mean as u64 and loop index i as u64"
)]
fn validate_gillespie_basal(
    h: &mut ValidationHarness,
    net: &EnzymeNetwork,
    sim: &SimConfig,
    ss_mean: f64,
    exp: &Value,
) -> (f64, f64) {
    println!("\n--- Part 2: Gillespie SSA Basal ---");

    let mut basal_means = Vec::with_capacity(sim.n_reps);
    let mut basal_vars = Vec::with_capacity(sim.n_reps);

    for i in 0..sim.n_reps {
        let traj = birth_death_ssa(
            &net.basal_rates,
            net.total_deg,
            ss_mean as u64,
            sim.t_max,
            sim.seed + i as u64,
        );
        let m = time_averaged_mean(&traj, sim.t_burnin);
        let v = time_averaged_variance(&traj, sim.t_burnin, m);
        basal_means.push(m);
        basal_vars.push(v);
    }

    let ensemble_mean = groundspring::stats::mean(&basal_means);
    let mean_var = groundspring::stats::mean(&basal_vars);
    let basal_std = mean_var.sqrt();

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

    (ensemble_mean, basal_std)
}

/// Validate activated states and response ratios.
///
/// Response-ratio tolerances from JSON; the ranges [1.1, 1.4] and [1.3, 1.7]
/// for α=10 and α=20 absorb stochastic variation across 200 replicates.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::too_many_arguments,
    reason = "validation orchestration groups related checks with shared state"
)]
fn validate_activated_states(
    h: &mut ValidationHarness,
    net: &EnzymeNetwork,
    sim: &SimConfig,
    ss_mean: f64,
    ensemble_mean: f64,
    basal_std: f64,
    alphas: &[u64],
    exp: &Value,
) {
    println!("\n--- Part 3: Activated States ---");

    let mut activated_means = Vec::new();
    for &alpha in alphas {
        let mut rates = vec![net.k_syn; net.n_dgc];
        #[expect(clippy::cast_precision_loss, reason = "alpha ≤ 20 ≪ 2^53")]
        {
            rates[0] = net.k_syn * alpha as f64;
        }

        let mut act_means = Vec::with_capacity(sim.n_reps);
        for i in 0..sim.n_reps {
            let traj = birth_death_ssa(
                &rates,
                net.total_deg,
                ss_mean as u64,
                sim.t_max,
                sim.seed + 10000 + alpha * 1000 + i as u64,
            );
            let m = time_averaged_mean(&traj, sim.t_burnin);
            act_means.push(m);
        }
        let act_ensemble = groundspring::stats::mean(&act_means);
        activated_means.push((alpha, act_ensemble));
        println!("  α={alpha}: mean={act_ensemble:.3}");
    }

    let get_act_mean = |a: u64| {
        activated_means
            .iter()
            .find(|(al, _)| *al == a)
            .or_exit("activation ratio must be present in results")
            .1
    };

    let rr10 = get_act_mean(10) / ensemble_mean;
    let rr20 = get_act_mean(20) / ensemble_mean;

    let (rr10_lo, rr10_hi) = f64_range(&exp["response_ratio_alpha_10_range"]);
    let (rr20_lo, rr20_hi) = f64_range(&exp["response_ratio_alpha_20_range"]);

    h.check_range("Response ratio α=10", rr10, rr10_lo, rr10_hi);
    h.check_range("Response ratio α=20", rr20, rr20_lo, rr20_hi);

    // ── SNR ─────────────────────────────────────────────────────────
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
            .or_exit("SNR ratio must be present in results")
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
}

fn run() -> i32 {
    let bench = parse_benchmark(BENCHMARK);
    let mut h = ValidationHarness::stdout("Rust Validation: Signal Specificity");

    print_provenance_header(&bench, "Signal Specificity (c-di-GMP)");

    let net_json = &bench["enzyme_network"];
    let sim_json = &bench["simulation"];
    let pred = &bench["analytical_predictions"];
    let exp = &bench["expected_results"];

    let n_dgc = usize_field(net_json, "n_dgc");
    let n_pde = usize_field(net_json, "n_pde");
    let k_syn = f64_field(net_json, "k_syn_per_dgc");
    let k_deg = f64_field(net_json, "k_deg_per_pde");
    #[expect(clippy::cast_precision_loss, reason = "n_pde ≤ 100 ≪ 2^53")]
    let total_deg = n_pde as f64 * k_deg;

    let net = EnzymeNetwork {
        n_dgc,
        k_syn,
        total_deg,
        basal_rates: vec![k_syn; n_dgc],
    };
    let sim = SimConfig {
        t_max: f64_field(sim_json, "t_max"),
        t_burnin: f64_field(sim_json, "t_burnin"),
        n_reps: usize_field(sim_json, "n_replicates"),
        seed: get_u64(sim_json, "seed").or_exit("seed"),
    };

    println!("  Enzyme network: {n_dgc} DGCs, {n_pde} PDEs");

    let (ss_mean, _ss_std) = validate_analytical(&mut h, &net, pred);
    let (ensemble_mean, basal_std) = validate_gillespie_basal(&mut h, &net, &sim, ss_mean, exp);

    let alphas: Vec<u64> = get_array(net_json, "activation_ratios")
        .or_exit("activation_ratios array")
        .iter()
        .map(|v| v.as_u64().or_exit("alpha"))
        .collect();

    validate_activated_states(
        &mut h,
        &net,
        &sim,
        ss_mean,
        ensemble_mean,
        basal_std,
        &alphas,
        exp,
    );

    // ── Part 5: Determinism ───────────────────────────────────────────
    println!("\n--- Part 5: Determinism ---");

    let t1 = birth_death_ssa(&net.basal_rates, net.total_deg, 18, 50.0, 12345);
    let t2 = birth_death_ssa(&net.basal_rates, net.total_deg, 18, 50.0, 12345);
    h.check_true("Deterministic (same seed)", t1.states == t2.states);

    let t3 = birth_death_ssa(&net.basal_rates, net.total_deg, 18, 50.0, 99999);
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
