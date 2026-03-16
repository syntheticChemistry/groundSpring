// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Bistable phenotypic switching in V. cholerae c-di-GMP circuit.
//!
//! 5-variable ODE: `[cell, AI, HapR, c-di-GMP, biofilm]` with positive
//! feedback from biofilm to DGC that creates two stable phenotypes:
//! - **Motile** (low c-di-GMP, low biofilm)
//! - **Sessile** (high c-di-GMP, high biofilm)
//!
//! The model matches `barracuda::numerical::ode_bio::BistableOde` exactly:
//! 18 base QS-biofilm parameters + 3 feedback parameters (`α_fb`, `n_fb`, `K_fb`).
//!
//! # barracuda delegation
//!
//! When the `barracuda` feature is enabled, `bistable_derivative` delegates
//! to `barracuda::numerical::ode_bio::BistableOde::cpu_derivative`.
//!
//! Reference: Fernandez, Waters et al. (2020) PNAS 117:26058-26068

#[cfg(feature = "barracuda")]
use barracuda::numerical::OdeSystem as _;

/// Default value for half-saturation and rate parameters in [`BistableParams`].
const DEFAULT_BISTABLE_HALF_SATURATION_AND_RATE: f64 = 0.5;

/// Parameter set matching `barracuda::BistableParams::default()`.
///
/// 18 base QS-biofilm parameters + 3 positive-feedback parameters.
#[derive(Debug, Clone, Copy)]
pub struct BistableParams {
    /// Maximum growth rate.
    pub mu_max: f64,
    /// Carrying capacity.
    pub k_cap: f64,
    /// Cell death rate.
    pub death_rate: f64,
    /// AI production rate.
    pub k_ai_prod: f64,
    /// AI degradation rate.
    pub d_ai: f64,
    /// Maximum `HapR` production rate.
    pub k_hapr_max: f64,
    /// Half-saturation for `HapR` activation by AI.
    pub k_hapr_ai: f64,
    /// Hill coefficient for `HapR` activation.
    pub n_hapr: f64,
    /// `HapR` degradation rate.
    pub d_hapr: f64,
    /// Basal DGC (diguanylate cyclase) rate.
    pub k_dgc_basal: f64,
    /// `HapR` repression coefficient on DGC.
    pub k_dgc_rep: f64,
    /// Basal PDE (phosphodiesterase) rate.
    pub k_pde_basal: f64,
    /// `HapR` activation coefficient on PDE.
    pub k_pde_act: f64,
    /// c-di-GMP degradation rate.
    pub d_cdg: f64,
    /// Maximum biofilm formation rate.
    pub k_bio_max: f64,
    /// Half-saturation for biofilm from c-di-GMP.
    pub k_bio_cdg: f64,
    /// Hill coefficient for biofilm production.
    pub n_bio: f64,
    /// Biofilm decay rate.
    pub d_bio: f64,
    /// Positive-feedback strength (biofilm → DGC).
    pub alpha_fb: f64,
    /// Hill coefficient for feedback.
    pub n_fb: f64,
    /// Half-saturation for feedback.
    pub k_fb: f64,
}

impl Default for BistableParams {
    fn default() -> Self {
        Self {
            mu_max: 0.8,
            k_cap: 1.0,
            death_rate: 0.02,
            k_ai_prod: 5.0,
            d_ai: 1.0,
            k_hapr_max: 1.0,
            k_hapr_ai: DEFAULT_BISTABLE_HALF_SATURATION_AND_RATE,
            n_hapr: 2.0,
            d_hapr: DEFAULT_BISTABLE_HALF_SATURATION_AND_RATE,
            k_dgc_basal: 2.0,
            k_dgc_rep: 0.3,
            k_pde_basal: DEFAULT_BISTABLE_HALF_SATURATION_AND_RATE,
            k_pde_act: DEFAULT_BISTABLE_HALF_SATURATION_AND_RATE,
            d_cdg: 0.3,
            k_bio_max: 1.0,
            k_bio_cdg: 1.5,
            n_bio: 4.0,
            d_bio: 0.2,
            alpha_fb: 3.0,
            n_fb: 4.0,
            k_fb: 0.6,
        }
    }
}

impl BistableParams {
    /// Pack into the flat 21-element array matching barracuda's layout.
    #[must_use]
    pub const fn to_flat(&self) -> [f64; 21] {
        [
            self.mu_max,
            self.k_cap,
            self.death_rate,
            self.k_ai_prod,
            self.d_ai,
            self.k_hapr_max,
            self.k_hapr_ai,
            self.n_hapr,
            self.d_hapr,
            self.k_dgc_basal,
            self.k_dgc_rep,
            self.k_pde_basal,
            self.k_pde_act,
            self.d_cdg,
            self.k_bio_max,
            self.k_bio_cdg,
            self.n_bio,
            self.d_bio,
            self.alpha_fb,
            self.n_fb,
            self.k_fb,
        ]
    }
}

#[cfg(not(feature = "barracuda"))]
use crate::kinetics::hill;

/// Compute the derivative for the 5-variable bistable ODE.
///
/// State: `[cell, AI, HapR, c-di-GMP, biofilm]`.
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::numerical::ode_bio::BistableOde::cpu_derivative`.
#[must_use]
pub fn bistable_derivative(state: &[f64; 5], params: &BistableParams) -> [f64; 5] {
    #[cfg(feature = "barracuda")]
    {
        let flat = params.to_flat();
        let result = barracuda::numerical::ode_bio::BistableOde::cpu_derivative(
            0.0,
            state.as_slice(),
            &flat,
        );
        [result[0], result[1], result[2], result[3], result[4]]
    }
    #[cfg(not(feature = "barracuda"))]
    bistable_derivative_cpu(state, params)
}

#[cfg(not(feature = "barracuda"))]
fn bistable_derivative_cpu(state: &[f64; 5], p: &BistableParams) -> [f64; 5] {
    let cell = state[0].max(0.0);
    let ai = state[1].max(0.0);
    let hapr = state[2].max(0.0);
    let cdg = state[3].max(0.0);
    let bio = state[4].max(0.0);

    let d_cell = (p.mu_max * cell).mul_add(1.0 - cell / p.k_cap, -(p.death_rate * cell));
    let d_ai = p.k_ai_prod.mul_add(cell, -p.d_ai * ai);
    let d_hapr = p
        .k_hapr_max
        .mul_add(hill(ai, p.k_hapr_ai, p.n_hapr), -p.d_hapr * hapr);

    let basal_dgc = p.k_dgc_basal * p.k_dgc_rep.mul_add(-hapr, 1.0).max(0.0);
    let feedback_dgc = p.alpha_fb * hill(bio, p.k_fb, p.n_fb);
    let pde_rate = p.k_pde_act.mul_add(hapr, p.k_pde_basal);
    let d_cdg = p
        .d_cdg
        .mul_add(-cdg, basal_dgc + feedback_dgc - pde_rate * cdg);

    let bio_promote = p.k_bio_max * hill(cdg, p.k_bio_cdg, p.n_bio);
    let d_bio = bio_promote.mul_add(1.0 - bio, -(p.d_bio * bio));

    [d_cell, d_ai, d_hapr, d_cdg, d_bio]
}

/// RK4 integration step (delegates to [`crate::ode::rk4_step`]).
#[must_use]
pub fn rk4_step(state: &[f64; 5], params: &BistableParams, dt: f64) -> [f64; 5] {
    crate::ode::rk4_step(state, dt, |s| bistable_derivative(s, params))
}

/// Integrate the bistable ODE from `state0` for `n_steps` of size `dt`.
#[must_use]
pub fn integrate(state0: &[f64; 5], params: &BistableParams, dt: f64, n_steps: usize) -> [f64; 5] {
    crate::ode::integrate(state0, dt, n_steps, |s| bistable_derivative(s, params))
}

/// Euler-Maruyama integration with additive noise on c-di-GMP (index 3).
#[must_use]
pub fn stochastic_integrate(
    state0: &[f64; 5],
    params: &BistableParams,
    dt: f64,
    n_steps: usize,
    noise_level: f64,
    seed: u64,
) -> [f64; 5] {
    let sqrt_dt = dt.sqrt();
    let mut rng = crate::prng::Xorshift64::new(seed);
    let mut state = *state0;
    for _ in 0..n_steps {
        let deriv = bistable_derivative(&state, params);
        for i in 0..5 {
            state[i] += dt * deriv[i];
        }
        state[3] += noise_level * sqrt_dt * rng.next_normal();
        for s in &mut state {
            *s = s.max(0.0);
        }
    }
    state
}

/// Batch-integrate the bistable ODE for multiple initial conditions.
///
/// When `barracuda-gpu` is enabled and a GPU is available, uses
/// `BatchedOdeRK4F64` for parallel integration of all trajectories.
/// Falls back to sequential CPU integration otherwise.
///
/// Returns the final state `[f64; 5]` for each initial condition.
#[must_use]
pub fn integrate_batch(
    initial_conditions: &[[f64; 5]],
    params: &BistableParams,
    dt: f64,
    n_steps: usize,
) -> Vec<[f64; 5]> {
    #[cfg(feature = "barracuda-gpu")]
    {
        if let Some(results) = integrate_batch_gpu(initial_conditions, params, dt, n_steps) {
            return results;
        }
    }
    initial_conditions
        .iter()
        .map(|ic| integrate(ic, params, dt, n_steps))
        .collect()
}

#[cfg(feature = "barracuda-gpu")]
#[expect(
    clippy::cast_possible_truncation,
    reason = "batch count bounded by test setup"
)]
fn integrate_batch_gpu(
    initial_conditions: &[[f64; 5]],
    params: &BistableParams,
    dt: f64,
    n_steps: usize,
) -> Option<Vec<[f64; 5]>> {
    use barracuda::ops::batched_ode_rk4::{BatchedOdeRK4F64, BatchedRk4Config};

    let device = crate::gpu::get_device()?;
    let config = BatchedRk4Config {
        n_batches: initial_conditions.len() as u32,
        n_steps: n_steps as u32,
        h: dt,
        ..BatchedRk4Config::default()
    };
    let integrator = BatchedOdeRK4F64::new(device, config);

    let flat_states: Vec<f64> = initial_conditions
        .iter()
        .flat_map(|s| s.iter().copied())
        .collect();
    let flat_params = params.to_flat();
    let batch_params: Vec<f64> = initial_conditions
        .iter()
        .flat_map(|_| flat_params[..BatchedOdeRK4F64::N_PARAMS].iter().copied())
        .collect();

    let result = integrator.integrate(&flat_states, &batch_params).ok()?;
    Some(
        result
            .chunks_exact(5)
            .map(|c| [c[0], c[1], c[2], c[3], c[4]])
            .collect(),
    )
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::tol;

    fn default_params() -> BistableParams {
        BistableParams::default()
    }

    #[test]
    fn derivative_at_zero_state() {
        let state = [0.0; 5];
        let d = bistable_derivative(&state, &default_params());
        assert!(
            d[0].abs() < f64::EPSILON,
            "d_cell should be 0 at zero state"
        );
    }

    #[test]
    fn cell_growth_positive_from_low() {
        let state = [0.1, 0.0, 0.0, 0.0, 0.0];
        let d = bistable_derivative(&state, &default_params());
        assert!(d[0] > 0.0, "cell should grow from low density");
    }

    #[test]
    fn bistability_two_attractors() {
        let p = default_params();
        let dt = 0.01;
        let n_steps = 20_000;

        let ic_low = [0.95, 4.5, 1.9, 0.3, 0.02];
        let ic_high = [0.95, 4.5, 1.9, 2.5, 0.85];

        let final_low = integrate(&ic_low, &p, dt, n_steps);
        let final_high = integrate(&ic_high, &p, dt, n_steps);

        assert!(
            final_low[3] < 1.0,
            "low IC should converge to low cdg, got {}",
            final_low[3]
        );
        assert!(
            final_high[3] > 1.0,
            "high IC should converge to high cdg, got {}",
            final_high[3]
        );
    }

    #[test]
    fn monostable_when_no_feedback() {
        let mut p = default_params();
        p.alpha_fb = 0.0;

        let dt = 0.01;
        let n_steps = 20_000;

        let ic_low = [0.95, 4.5, 1.9, 0.3, 0.02];
        let ic_high = [0.95, 4.5, 1.9, 2.5, 0.85];

        let final_low = integrate(&ic_low, &p, dt, n_steps);
        let final_high = integrate(&ic_high, &p, dt, n_steps);

        let diff = (final_low[3] - final_high[3]).abs();
        assert!(
            diff < 0.5,
            "monostable system: cdg should agree, diff={diff}"
        );
    }

    #[test]
    #[expect(clippy::float_cmp, reason = "bitwise determinism test")]
    fn rk4_deterministic() {
        let p = default_params();
        let state = [0.5, 1.0, 0.5, 1.0, 0.3];
        let a = rk4_step(&state, &p, 0.01);
        let b = rk4_step(&state, &p, 0.01);
        assert_eq!(a, b);
    }

    #[test]
    fn integrate_cell_reaches_capacity() {
        let p = default_params();
        let ic = [0.1, 0.0, 0.0, 0.0, 0.0];
        let final_state = integrate(&ic, &p, 0.01, 20_000);
        assert!(
            (final_state[0] - 0.975).abs() < tol::EQUILIBRIUM,
            "cell should reach carrying capacity, got {}",
            final_state[0]
        );
    }

    #[test]
    fn flat_params_roundtrip() {
        // flat[0] = mu_max (0.8), flat[18] = alpha_fb (3.0) from Default impl
        let p = default_params();
        let flat = p.to_flat();
        assert_eq!(flat.len(), 21);
        assert!((flat[0] - 0.8).abs() < f64::EPSILON);
        assert!((flat[18] - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn stochastic_integrate_deterministic_same_seed() {
        let p = default_params();
        let ic = [0.5, 1.0, 0.5, 1.0, 0.3];
        let a = stochastic_integrate(&ic, &p, 0.01, 1000, 0.1, 42);
        let b = stochastic_integrate(&ic, &p, 0.01, 1000, 0.1, 42);
        for (x, y) in a.iter().zip(b.iter()) {
            assert_eq!(x.to_bits(), y.to_bits(), "bitwise determinism");
        }
    }

    #[test]
    fn stochastic_integrate_different_seed_diverges() {
        let p = default_params();
        let ic = [0.5, 1.0, 0.5, 1.0, 0.3];
        let a = stochastic_integrate(&ic, &p, 0.01, 1000, 0.1, 42);
        let b = stochastic_integrate(&ic, &p, 0.01, 1000, 0.1, 99);
        assert!(
            a.iter()
                .zip(b.iter())
                .any(|(x, y)| x.to_bits() != y.to_bits())
        );
    }

    #[test]
    fn stochastic_integrate_low_noise_near_deterministic() {
        let p = default_params();
        let ic = [0.5, 1.0, 0.5, 1.0, 0.3];
        let det = integrate(&ic, &p, 0.01, 2000);
        let stoch = stochastic_integrate(&ic, &p, 0.01, 2000, 0.001, 42);
        let cdg_diff = (det[3] - stoch[3]).abs();
        assert!(
            cdg_diff < 0.5,
            "low noise should be near deterministic, cdg diff={cdg_diff}"
        );
    }

    #[test]
    fn stochastic_integrate_states_non_negative() {
        let p = default_params();
        let ic = [0.01, 0.01, 0.01, 0.01, 0.01];
        let result = stochastic_integrate(&ic, &p, 0.01, 5000, 1.0, 42);
        for (i, &val) in result.iter().enumerate() {
            assert!(
                val >= 0.0,
                "state[{i}] = {val} < 0 after stochastic integration"
            );
        }
    }

    #[test]
    fn derivative_components_bounded() {
        let p = default_params();
        let state = [1.0, 5.0, 2.0, 2.0, 0.5];
        let d = bistable_derivative(&state, &p);
        for (i, &val) in d.iter().enumerate() {
            assert!(val.is_finite(), "derivative[{i}] is not finite: {val}");
        }
    }
}
