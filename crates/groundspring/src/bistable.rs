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
            k_hapr_ai: 0.5,
            n_hapr: 2.0,
            d_hapr: 0.5,
            k_dgc_basal: 2.0,
            k_dgc_rep: 0.3,
            k_pde_basal: 0.5,
            k_pde_act: 0.5,
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
        return [result[0], result[1], result[2], result[3], result[4]];
    }
    #[cfg(not(feature = "barracuda"))]
    bistable_derivative_cpu(state, params)
}

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

/// RK4 integration step.
#[must_use]
pub fn rk4_step(state: &[f64; 5], params: &BistableParams, dt: f64) -> [f64; 5] {
    let half_dt = 0.5 * dt;
    let sixth_dt = dt / 6.0;

    let k1 = bistable_derivative(state, params);
    let mut s1 = [0.0; 5];
    for i in 0..5 {
        s1[i] = half_dt.mul_add(k1[i], state[i]);
    }
    let k2 = bistable_derivative(&s1, params);
    let mut s2 = [0.0; 5];
    for i in 0..5 {
        s2[i] = half_dt.mul_add(k2[i], state[i]);
    }
    let k3 = bistable_derivative(&s2, params);
    let mut s3 = [0.0; 5];
    for i in 0..5 {
        s3[i] = dt.mul_add(k3[i], state[i]);
    }
    let k4 = bistable_derivative(&s3, params);
    let mut result = [0.0; 5];
    for i in 0..5 {
        let slope = 2.0f64.mul_add(k2[i] + k3[i], k1[i] + k4[i]);
        result[i] = sixth_dt.mul_add(slope, state[i]);
    }
    result
}

/// Integrate the bistable ODE from `state0` for `n_steps` of size `dt`.
#[must_use]
pub fn integrate(state0: &[f64; 5], params: &BistableParams, dt: f64, n_steps: usize) -> [f64; 5] {
    let mut state = *state0;
    for _ in 0..n_steps {
        state = rk4_step(&state, params, dt);
    }
    state
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

#[cfg(test)]
mod tests {
    use super::*;

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
            (final_state[0] - 0.975).abs() < 0.1,
            "cell should reach carrying capacity, got {}",
            final_state[0]
        );
    }

    #[test]
    fn flat_params_roundtrip() {
        let p = default_params();
        let flat = p.to_flat();
        assert_eq!(flat.len(), 21);
        assert!((flat[0] - 0.8).abs() < f64::EPSILON);
        assert!((flat[18] - 3.0).abs() < f64::EPSILON);
    }
}
