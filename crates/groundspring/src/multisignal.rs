// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Multi-signal QS integration in V. cholerae.
//!
//! 7-variable ODE: `[cell, CAI-1, AI-2, LuxO~P, HapR, c-di-GMP, biofilm]`
//! modeling how two quorum sensing signals (CAI-1 and AI-2) converge
//! through `LuxO`/`HapR` to control c-di-GMP and biofilm formation.
//!
//! Key biology: both signals dephosphorylate `LuxO~P`, releasing
//! repression of `HapR`. Higher `HapR` at high cell density *represses*
//! DGC and activates PDE, reducing c-di-GMP and thus biofilm —
//! integrating two signals gives a sharper, more specific response.
//!
//! The model matches `barracuda::numerical::ode_bio::MultiSignalOde`
//! exactly: 24 parameters from Srivastava et al. (2011).
//!
//! # barracuda delegation
//!
//! When the `barracuda` feature is enabled, `multisignal_derivative`
//! delegates to `barracuda::numerical::ode_bio::MultiSignalOde::cpu_derivative`.
//!
//! Reference: Srivastava, Waters et al. (2011) J Bacteriology 194:122-136

#[cfg(feature = "barracuda")]
use barracuda::numerical::OdeSystem as _;

/// Parameter set matching `barracuda::MultiSignalParams::default()`.
#[derive(Debug, Clone, Copy)]
pub struct MultiSignalParams {
    /// Maximum growth rate.
    pub mu_max: f64,
    /// Carrying capacity.
    pub k_cap: f64,
    /// Cell death rate.
    pub death_rate: f64,
    /// CAI-1 production rate.
    pub k_cai1_prod: f64,
    /// CAI-1 degradation rate.
    pub d_cai1: f64,
    /// `CqsS` half-saturation for CAI-1 sensing.
    pub k_cqs: f64,
    /// AI-2 production rate.
    pub k_ai2_prod: f64,
    /// AI-2 degradation rate.
    pub d_ai2: f64,
    /// `LuxPQ` half-saturation for AI-2 sensing.
    pub k_luxpq: f64,
    /// `LuxO` phosphorylation rate.
    pub k_luxo_phos: f64,
    /// `LuxO~P` dephosphorylation/degradation rate.
    pub d_luxo_p: f64,
    /// Maximum `HapR` production rate.
    pub k_hapr_max: f64,
    /// Hill coefficient for `LuxO~P` repression of `HapR`.
    pub n_repress: f64,
    /// Half-saturation for `LuxO~P` repression.
    pub k_repress: f64,
    /// `HapR` degradation rate.
    pub d_hapr: f64,
    /// Basal DGC rate.
    pub k_dgc_basal: f64,
    /// `HapR` repression coefficient on DGC.
    pub k_dgc_rep: f64,
    /// Basal PDE rate.
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
}

impl Default for MultiSignalParams {
    fn default() -> Self {
        Self {
            mu_max: 0.8,
            k_cap: 1.0,
            death_rate: 0.02,
            k_cai1_prod: 3.0,
            d_cai1: 1.0,
            k_cqs: 0.5,
            k_ai2_prod: 3.0,
            d_ai2: 1.0,
            k_luxpq: 0.5,
            k_luxo_phos: 2.0,
            d_luxo_p: 0.5,
            k_hapr_max: 1.0,
            n_repress: 2.0,
            k_repress: 0.5,
            d_hapr: 0.5,
            k_dgc_basal: 2.0,
            k_dgc_rep: 0.8,
            k_pde_basal: 0.5,
            k_pde_act: 2.0,
            d_cdg: 0.3,
            k_bio_max: 1.0,
            k_bio_cdg: 1.5,
            n_bio: 2.0,
            d_bio: 0.2,
        }
    }
}

impl MultiSignalParams {
    /// Pack into the flat 24-element array matching barracuda's layout.
    #[must_use]
    pub const fn to_flat(&self) -> [f64; 24] {
        [
            self.mu_max,
            self.k_cap,
            self.death_rate,
            self.k_cai1_prod,
            self.d_cai1,
            self.k_cqs,
            self.k_ai2_prod,
            self.d_ai2,
            self.k_luxpq,
            self.k_luxo_phos,
            self.d_luxo_p,
            self.k_hapr_max,
            self.n_repress,
            self.k_repress,
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
        ]
    }
}

use crate::kinetics::{hill, hill_repress};

/// Compute the derivative for the 7-variable multi-signal ODE.
///
/// State: `[cell, CAI-1, AI-2, LuxO~P, HapR, c-di-GMP, biofilm]`.
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::numerical::ode_bio::MultiSignalOde::cpu_derivative`.
#[must_use]
pub fn multisignal_derivative(state: &[f64; 7], params: &MultiSignalParams) -> [f64; 7] {
    #[cfg(feature = "barracuda")]
    {
        let flat = params.to_flat();
        let result = barracuda::numerical::ode_bio::MultiSignalOde::cpu_derivative(
            0.0,
            state.as_slice(),
            &flat,
        );
        return [
            result[0], result[1], result[2], result[3], result[4], result[5], result[6],
        ];
    }
    #[cfg(not(feature = "barracuda"))]
    multisignal_derivative_cpu(state, params)
}

fn multisignal_derivative_cpu(state: &[f64; 7], p: &MultiSignalParams) -> [f64; 7] {
    let cell = state[0].max(0.0);
    let cai1 = state[1].max(0.0);
    let ai2 = state[2].max(0.0);
    let luxo_p = state[3].max(0.0);
    let hapr = state[4].max(0.0);
    let cdg = state[5].max(0.0);
    let bio = state[6].max(0.0);

    let d_cell = (p.mu_max * cell).mul_add(1.0 - cell / p.k_cap, -(p.death_rate * cell));
    let d_cai1 = p.k_cai1_prod.mul_add(cell, -p.d_cai1 * cai1);
    let d_ai2 = p.k_ai2_prod.mul_add(cell, -p.d_ai2 * ai2);

    let dephos_cai1 = hill(cai1, p.k_cqs, 2.0);
    let dephos_ai2 = hill(ai2, p.k_luxpq, 2.0);
    let d_luxo_p = (p.d_luxo_p + dephos_cai1 + dephos_ai2).mul_add(-luxo_p, p.k_luxo_phos);

    let d_hapr = p.k_hapr_max.mul_add(
        hill_repress(luxo_p, p.k_repress, p.n_repress),
        -p.d_hapr * hapr,
    );

    let dgc_rate = p.k_dgc_basal * p.k_dgc_rep.mul_add(-hapr, 1.0).max(0.0);
    let pde_rate = p.k_pde_act.mul_add(hapr, p.k_pde_basal);
    let d_cdg = p.d_cdg.mul_add(-cdg, dgc_rate - pde_rate * cdg);

    let bio_promote = p.k_bio_max * hill(cdg, p.k_bio_cdg, p.n_bio);
    let d_bio = bio_promote.mul_add(1.0 - bio, -(p.d_bio * bio));

    [d_cell, d_cai1, d_ai2, d_luxo_p, d_hapr, d_cdg, d_bio]
}

/// RK4 integration step.
#[must_use]
pub fn rk4_step(state: &[f64; 7], params: &MultiSignalParams, dt: f64) -> [f64; 7] {
    let half_dt = 0.5 * dt;
    let sixth_dt = dt / 6.0;

    let k1 = multisignal_derivative(state, params);
    let mut s1 = [0.0; 7];
    for i in 0..7 {
        s1[i] = half_dt.mul_add(k1[i], state[i]);
    }
    let k2 = multisignal_derivative(&s1, params);
    let mut s2 = [0.0; 7];
    for i in 0..7 {
        s2[i] = half_dt.mul_add(k2[i], state[i]);
    }
    let k3 = multisignal_derivative(&s2, params);
    let mut s3 = [0.0; 7];
    for i in 0..7 {
        s3[i] = dt.mul_add(k3[i], state[i]);
    }
    let k4 = multisignal_derivative(&s3, params);
    let mut result = [0.0; 7];
    for i in 0..7 {
        let slope = 2.0f64.mul_add(k2[i] + k3[i], k1[i] + k4[i]);
        result[i] = sixth_dt.mul_add(slope, state[i]);
    }
    result
}

/// Integrate the ODE from `state0` for `n_steps` of size `dt`.
#[must_use]
pub fn integrate(
    state0: &[f64; 7],
    params: &MultiSignalParams,
    dt: f64,
    n_steps: usize,
) -> [f64; 7] {
    let mut state = *state0;
    for _ in 0..n_steps {
        state = rk4_step(&state, params, dt);
    }
    state
}

/// Euler-Maruyama integration with additive noise on c-di-GMP (index 5).
#[must_use]
pub fn stochastic_integrate(
    state0: &[f64; 7],
    params: &MultiSignalParams,
    dt: f64,
    n_steps: usize,
    noise_level: f64,
    seed: u64,
) -> [f64; 7] {
    let sqrt_dt = dt.sqrt();
    let mut rng = crate::prng::Xorshift64::new(seed);
    let mut state = *state0;
    for _ in 0..n_steps {
        let deriv = multisignal_derivative(&state, params);
        for i in 0..7 {
            state[i] += dt * deriv[i];
        }
        state[5] += noise_level * sqrt_dt * rng.next_normal();
        for s in &mut state {
            *s = s.max(0.0);
        }
    }
    state
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_params() -> MultiSignalParams {
        MultiSignalParams::default()
    }

    #[test]
    fn derivative_at_zero_is_bounded() {
        let state = [0.0; 7];
        let d = multisignal_derivative(&state, &default_params());
        assert!(
            d[3] > 0.0,
            "LuxO phosphorylation should be positive at zero state"
        );
    }

    #[test]
    fn cell_growth_from_low_density() {
        let state = [0.1, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0];
        let d = multisignal_derivative(&state, &default_params());
        assert!(d[0] > 0.0, "cell should grow from low density");
    }

    #[test]
    fn dual_signal_more_hapr_than_single() {
        let p = default_params();
        let dt = 0.01;
        let n_steps = 20_000;
        let ic: [f64; 7] = [0.1, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0];

        let final_dual = integrate(&ic, &p, dt, n_steps);

        let mut p_cai1 = p;
        p_cai1.k_ai2_prod = 0.0;
        let final_cai1 = integrate(&ic, &p_cai1, dt, n_steps);

        assert!(
            final_dual[4] > final_cai1[4],
            "dual HapR ({}) should exceed CAI-1 only ({})",
            final_dual[4],
            final_cai1[4]
        );
    }

    #[test]
    fn rk4_deterministic() {
        let p = default_params();
        let state = [0.5, 1.0, 1.0, 0.5, 0.3, 0.5, 0.2];
        let a = rk4_step(&state, &p, 0.01);
        let b = rk4_step(&state, &p, 0.01);
        assert_eq!(a, b);
    }

    #[test]
    fn integrate_cell_reaches_capacity() {
        let p = default_params();
        let ic = [0.1, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0];
        let final_state = integrate(&ic, &p, 0.01, 20_000);
        assert!(
            (final_state[0] - 0.975).abs() < 0.1,
            "cell should reach capacity, got {}",
            final_state[0]
        );
    }

    #[test]
    fn flat_params_roundtrip() {
        let p = default_params();
        let flat = p.to_flat();
        assert_eq!(flat.len(), 24);
        assert!((flat[0] - 0.8).abs() < f64::EPSILON);
        assert!((flat[23] - 0.2).abs() < f64::EPSILON);
    }
}
