// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Gillespie stochastic simulation algorithm (SSA) for chemical kinetics.
//!
//! Implements the direct method (Gillespie 1977) for a birth-death process
//! modelling enzymatic signal vs noise — specifically, c-di-GMP dynamics
//! with competing DGC (synthesis) and PDE (degradation) enzymes.
//!
//! # barracuda delegation
//!
//! When the `barracuda` feature is enabled, `gillespie_ssa` can delegate to
//! `barracuda::ops::bio::GillespieGpu::simulate()` which uses the same
//! `rate_k` / `stoich_react` / `stoich_net` format.

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
    for i in 0..times.len() - 1 {
        let dt = times[i + 1] - times[i];
        weighted_sum += crate::cast::u64_f64(states[i]) * dt;
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
    for i in 0..times.len() - 1 {
        let dt = times[i + 1] - times[i];
        let dev = crate::cast::u64_f64(states[i]) - mean;
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

    #[test]
    fn steady_state_analytical() {
        let ss = steady_state_mean(40.0, 2.2);
        assert!((ss - 18.182).abs() < 0.01);
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
        assert!((steady_state_mean(10.0, 0.0)).abs() < 1e-12);
        assert!((steady_state_mean(10.0, -1.0)).abs() < 1e-12);
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
        assert!((time_averaged_mean(&traj, 999.0)).abs() < 1e-12);
    }

    #[test]
    fn time_averaged_mean_single_event() {
        let traj = Trajectory {
            times: vec![0.0],
            states: vec![7],
        };
        assert!((time_averaged_mean(&traj, 0.0) - 7.0).abs() < 1e-12);
    }

    #[test]
    fn time_averaged_variance_beyond_burnin() {
        let traj = Trajectory {
            times: vec![0.0, 1.0],
            states: vec![5, 5],
        };
        assert!((time_averaged_variance(&traj, 999.0, 5.0)).abs() < 1e-12);
    }

    #[test]
    fn time_averaged_variance_single_event() {
        let traj = Trajectory {
            times: vec![0.0],
            states: vec![7],
        };
        assert!((time_averaged_variance(&traj, 0.0, 7.0)).abs() < 1e-12);
    }
}
