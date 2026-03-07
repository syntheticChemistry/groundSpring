// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Gillespie stochastic simulation algorithm (SSA) for chemical kinetics.
//!
//! Implements the direct method (Gillespie 1977) for a birth-death process
//! modelling enzymatic signal vs noise — specifically, c-di-GMP dynamics
//! with competing DGC (synthesis) and PDE (degradation) enzymes.
//!
//! # barracuda delegation
//!
//! [`birth_death_ssa`] stays local on CPU — SSA is inherently serial
//! (next event depends on current state). GPU promotion is via
//! [`birth_death_ssa_batch`], which runs many independent trajectories
//! in parallel using `barracuda::ops::bio::GillespieGpu` when the
//! `barracuda-gpu` feature is enabled, falling back to a sequential CPU
//! loop otherwise. [`steady_state_mean`] and [`time_averaged_mean`]
//! are scalar reductions, Stays Local tier.

#[cfg(feature = "barracuda-gpu")]
use crate::eps;
use crate::prng::Xorshift64;

/// A recorded trajectory from a Gillespie SSA run.
#[derive(Debug, Clone)]
pub struct Trajectory {
    /// Event times (including t=0).
    pub times: Vec<f64>,
    /// Species count at each event time.
    pub states: Vec<u64>,
}

/// Run Gillespie SSA for a single-species birth-death process.
///
/// Each DGC contributes a zero-order synthesis rate.  Degradation is
/// first-order: `total_deg_rate * current_count`.
///
/// Returns a [`Trajectory`] recording every state change.
///
/// # Panics
///
/// Panics if `synthesis_rates` is empty.
#[must_use]
pub fn birth_death_ssa(
    synthesis_rates: &[f64],
    total_deg_rate: f64,
    initial: u64,
    t_max: f64,
    seed: u64,
) -> Trajectory {
    assert!(
        !synthesis_rates.is_empty(),
        "need at least one synthesis rate"
    );

    let total_syn: f64 = synthesis_rates.iter().sum();
    let mut rng = Xorshift64::new(seed);
    let mut t = 0.0;
    let mut s = initial;
    let mut times = vec![0.0];
    let mut states = vec![s];

    #[expect(
        clippy::while_float,
        reason = "SSA termination by continuous time is intentional"
    )]
    while t < t_max {
        let deg = total_deg_rate * crate::cast::u64_f64(s);
        let total_rate = total_syn + deg;
        if total_rate <= 0.0 {
            break;
        }

        let dt = -rng.next_f64().ln() / total_rate;
        t += dt;
        if t > t_max {
            break;
        }

        if rng.next_f64() < total_syn / total_rate {
            s += 1;
        } else {
            s = s.saturating_sub(1);
        }

        times.push(t);
        states.push(s);
    }

    Trajectory { times, states }
}

/// Result from a batch of Gillespie SSA trajectories.
#[derive(Debug, Clone)]
pub struct BatchResult {
    /// Time-averaged mean across all trajectories.
    pub mean: f64,
    /// Time-averaged variance across all trajectories.
    pub variance: f64,
    /// Number of trajectories that completed.
    pub n_trajectories: usize,
}

/// Run many independent birth-death SSA trajectories and return summary statistics.
///
/// When the `barracuda-gpu` feature is enabled and a GPU is available,
/// dispatches all trajectories to `GillespieGpu` in a single batch.
/// Falls back to sequential CPU execution otherwise.
///
/// Each trajectory uses a deterministic seed derived from `base_seed`.
#[must_use]
pub fn birth_death_ssa_batch(
    synthesis_rates: &[f64],
    total_deg_rate: f64,
    initial: u64,
    t_max: f64,
    n_trajectories: usize,
    t_burnin: f64,
    base_seed: u64,
) -> BatchResult {
    #[cfg(feature = "barracuda-gpu")]
    {
        if let Some(result) = birth_death_ssa_batch_gpu(
            synthesis_rates,
            total_deg_rate,
            initial,
            t_max,
            n_trajectories,
            t_burnin,
            base_seed,
        ) {
            return result;
        }
    }
    birth_death_ssa_batch_cpu(
        synthesis_rates,
        total_deg_rate,
        initial,
        t_max,
        n_trajectories,
        t_burnin,
        base_seed,
    )
}

fn birth_death_ssa_batch_cpu(
    synthesis_rates: &[f64],
    total_deg_rate: f64,
    initial: u64,
    t_max: f64,
    n_trajectories: usize,
    t_burnin: f64,
    base_seed: u64,
) -> BatchResult {
    let mut means = Vec::with_capacity(n_trajectories);
    for i in 0..n_trajectories {
        let seed = base_seed.wrapping_add(i as u64);
        let traj = birth_death_ssa(synthesis_rates, total_deg_rate, initial, t_max, seed);
        means.push(time_averaged_mean(&traj, t_burnin));
    }
    let (grand_mean, std) = crate::stats::mean_and_std_dev(&means);
    BatchResult {
        mean: grand_mean,
        variance: std * std,
        n_trajectories,
    }
}

#[cfg(feature = "barracuda-gpu")]
fn birth_death_ssa_batch_gpu(
    synthesis_rates: &[f64],
    total_deg_rate: f64,
    initial: u64,
    t_max: f64,
    n_trajectories: usize,
    t_burnin: f64,
    base_seed: u64,
) -> Option<BatchResult> {
    use barracuda::ops::bio::gillespie::{GillespieGpu, GillespieModel};

    let device = crate::gpu::get_device_f64_safe()?;

    let total_syn: f64 = synthesis_rates.iter().sum();

    // Birth-death → 2-reaction, 1-species network:
    //   R0: ∅ → X  (birth, rate = total_syn)
    //   R1: X → ∅  (death, rate = total_deg * X)
    let rate_k = [total_syn, total_deg_rate];
    let stoich_react: [u32; 2] = [0, 1];
    let stoich_net: [i32; 2] = [1, -1];

    let initial_states: Vec<f64> = vec![crate::cast::u64_f64(initial); n_trajectories];

    let mut prng_seeds = Vec::with_capacity(n_trajectories * 4);
    let mut rng = crate::prng::Xorshift64::new(base_seed);
    for _ in 0..n_trajectories {
        for _ in 0..4 {
            #[expect(
                clippy::cast_possible_truncation,
                reason = "RNG u64 → u32 seed; high bits discarded intentionally"
            )]
            prng_seeds.push(rng.next_u64() as u32);
        }
    }

    let gpu = GillespieGpu::new(&device);
    let model = GillespieModel {
        rate_k: &rate_k,
        stoich_react: &stoich_react,
        stoich_net: &stoich_net,
    };
    let config = barracuda::ops::bio::gillespie::GillespieConfig {
        t_max,
        max_steps: 1_000_000,
    };

    let result = gpu
        .simulate(
            &model,
            &initial_states,
            &prng_seeds,
            n_trajectories,
            &config,
        )
        .ok()?;

    // GPU returns final state per trajectory — approximate time-averaged mean
    // using analytical steady-state weighted by burn-in fraction.
    // Guard: SSA_FLOOR prevents division-by-zero in the steady-state mean
    // when degradation rate is negligible (e.g. pure synthesis regime).
    // The resulting ss_mean saturates at a large but finite value rather than Inf.
    let ss_mean = total_syn / total_deg_rate.max(eps::SSA_FLOOR);
    let burnin_fraction = (t_burnin / t_max).clamp(0.0, 1.0);
    let post_burnin_weight = 1.0 - burnin_fraction;

    let means: Vec<f64> = result
        .states
        .iter()
        .map(|&s| s.mul_add(post_burnin_weight, ss_mean * burnin_fraction))
        .collect();

    let (grand_mean, std) = crate::stats::mean_and_std_dev(&means);
    Some(BatchResult {
        mean: grand_mean,
        variance: std * std,
        n_trajectories,
    })
}

/// Analytical steady-state mean for a birth-death process.
///
/// `S* = total_synthesis / total_degradation_rate`
#[must_use]
pub fn steady_state_mean(total_synthesis: f64, total_degradation_rate: f64) -> f64 {
    if total_degradation_rate <= 0.0 {
        return 0.0;
    }
    total_synthesis / total_degradation_rate
}

/// Time-weighted average of a trajectory after a burn-in period.
#[must_use]
pub fn time_averaged_mean(traj: &Trajectory, t_start: f64) -> f64 {
    let n = traj.times.len();
    let start_idx = traj.times.iter().position(|&t| t >= t_start).unwrap_or(n);

    if start_idx >= n {
        return 0.0;
    }

    let times = &traj.times[start_idx..];
    let states = &traj.states[start_idx..];

    if times.len() < 2 {
        return crate::cast::u64_f64(states[0]);
    }

    let mut weighted_sum = 0.0;
    let mut total_dt = 0.0;
    for (pair, &s) in times.windows(2).zip(states.iter()) {
        let (t0, t1) = (pair[0], pair[1]);
        let dt = t1 - t0;
        weighted_sum += crate::cast::u64_f64(s) * dt;
        total_dt += dt;
    }

    if total_dt <= 0.0 {
        return crate::cast::u64_f64(states[0]);
    }
    weighted_sum / total_dt
}

/// Time-weighted variance of a trajectory after burn-in.
#[must_use]
pub fn time_averaged_variance(traj: &Trajectory, t_start: f64, mean: f64) -> f64 {
    let n = traj.times.len();
    let start_idx = traj.times.iter().position(|&t| t >= t_start).unwrap_or(n);

    if start_idx >= n {
        return 0.0;
    }

    let times = &traj.times[start_idx..];
    let states = &traj.states[start_idx..];

    if times.len() < 2 {
        return 0.0;
    }

    let mut weighted_sum = 0.0;
    let mut total_dt = 0.0;
    for (pair, &s) in times.windows(2).zip(states.iter()) {
        let (t0, t1) = (pair[0], pair[1]);
        let dt = t1 - t0;
        let dev = crate::cast::u64_f64(s) - mean;
        weighted_sum += dev * dev * dt;
        total_dt += dt;
    }

    if total_dt <= 0.0 {
        return 0.0;
    }
    weighted_sum / total_dt
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tol;

    #[test]
    fn steady_state_analytical() {
        let ss = steady_state_mean(40.0, 2.2);
        assert!((ss - 18.182).abs() < tol::STOCHASTIC);
    }

    #[test]
    fn birth_death_deterministic() {
        let rates = vec![1.0; 5];
        let t1 = birth_death_ssa(&rates, 0.5, 10, 10.0, 42);
        let t2 = birth_death_ssa(&rates, 0.5, 10, 10.0, 42);
        assert_eq!(t1.states, t2.states);
    }

    #[test]
    fn birth_death_different_seed() {
        let rates = vec![1.0; 5];
        let t1 = birth_death_ssa(&rates, 0.5, 10, 10.0, 42);
        let t2 = birth_death_ssa(&rates, 0.5, 10, 10.0, 99);
        assert_ne!(t1.states, t2.states);
    }

    #[test]
    fn time_averaged_mean_converges() {
        let rates = vec![1.0; 10];
        let traj = birth_death_ssa(&rates, 1.0, 10, 500.0, 42);
        let m = time_averaged_mean(&traj, 50.0);
        assert!((m - 10.0).abs() < 3.0, "mean={m}, expected ~10.0");
    }

    #[test]
    fn time_averaged_variance_positive() {
        let rates = vec![1.0; 10];
        let traj = birth_death_ssa(&rates, 1.0, 10, 500.0, 42);
        let m = time_averaged_mean(&traj, 50.0);
        let v = time_averaged_variance(&traj, 50.0, m);
        assert!(v > 0.0, "variance should be positive");
    }

    #[test]
    fn steady_state_zero_degradation() {
        assert!((steady_state_mean(10.0, 0.0)).abs() < tol::EXACT);
        assert!((steady_state_mean(10.0, -1.0)).abs() < tol::EXACT);
    }

    #[test]
    fn zero_rate_terminates() {
        let traj = birth_death_ssa(&[0.0], 0.0, 0, 10.0, 42);
        assert_eq!(
            traj.states.len(),
            1,
            "zero rates should terminate immediately"
        );
    }

    #[test]
    fn time_averaged_mean_beyond_burnin() {
        let traj = Trajectory {
            times: vec![0.0, 1.0],
            states: vec![5, 5],
        };
        assert!((time_averaged_mean(&traj, 999.0)).abs() < tol::EXACT);
    }

    #[test]
    fn time_averaged_mean_single_event() {
        let traj = Trajectory {
            times: vec![0.0],
            states: vec![7],
        };
        assert!((time_averaged_mean(&traj, 0.0) - 7.0).abs() < tol::EXACT);
    }

    #[test]
    fn time_averaged_variance_beyond_burnin() {
        let traj = Trajectory {
            times: vec![0.0, 1.0],
            states: vec![5, 5],
        };
        assert!((time_averaged_variance(&traj, 999.0, 5.0)).abs() < tol::EXACT);
    }

    #[test]
    fn time_averaged_variance_single_event() {
        let traj = Trajectory {
            times: vec![0.0],
            states: vec![7],
        };
        assert!((time_averaged_variance(&traj, 0.0, 7.0)).abs() < tol::EXACT);
    }

    #[test]
    fn batch_parity_cpu_sequential_vs_dispatch() {
        let rates = vec![1.0; 10];
        let n_traj = 50;
        let t_max = 200.0;
        let t_burnin = 50.0;
        let base_seed = 42;

        let batch = birth_death_ssa_batch(&rates, 1.0, 10, t_max, n_traj, t_burnin, base_seed);
        let cpu = birth_death_ssa_batch_cpu(&rates, 1.0, 10, t_max, n_traj, t_burnin, base_seed);

        assert_eq!(batch.n_trajectories, cpu.n_trajectories);
        let tol = if cfg!(feature = "barracuda-gpu") {
            5.0
        } else {
            f64::EPSILON
        };
        assert!(
            (batch.mean - cpu.mean).abs() < tol,
            "mean mismatch: batch={}, cpu={}",
            batch.mean,
            cpu.mean
        );
    }

    #[test]
    fn batch_result_reasonable() {
        let rates = vec![1.0; 10];
        let batch = birth_death_ssa_batch(&rates, 1.0, 10, 500.0, 100, 100.0, 42);
        let ss = steady_state_mean(10.0, 1.0);
        assert!(
            (batch.mean - ss).abs() < 5.0,
            "batch mean {} should be near steady state {}",
            batch.mean,
            ss
        );
        assert!(batch.variance > 0.0);
        assert_eq!(batch.n_trajectories, 100);
    }
}
