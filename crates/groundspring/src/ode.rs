// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Generic RK4 integration for fixed-size ODE systems.
//!
//! Provides a single implementation of the classical fourth-order
//! Runge-Kutta integrator that works with any state type implementing
//! [`OdeState`]. This eliminates the duplicate RK4 code previously
//! inlined in [`crate::bistable`] and [`crate::multisignal`].

/// Trait for fixed-size ODE state vectors.
///
/// Implementors must support element-wise arithmetic via indexed access
/// and provide their dimensionality at compile time.
pub trait OdeState: Copy {
    /// Number of state variables.
    const DIM: usize;

    /// Create a zero-initialized state.
    fn zero() -> Self;

    /// Get element at index `i`.
    fn get(&self, i: usize) -> f64;

    /// Set element at index `i`.
    fn set(&mut self, i: usize, v: f64);
}

impl OdeState for [f64; 5] {
    const DIM: usize = 5;

    fn zero() -> Self {
        [0.0; 5]
    }

    fn get(&self, i: usize) -> f64 {
        self[i]
    }

    fn set(&mut self, i: usize, v: f64) {
        self[i] = v;
    }
}

impl OdeState for [f64; 7] {
    const DIM: usize = 7;

    fn zero() -> Self {
        [0.0; 7]
    }

    fn get(&self, i: usize) -> f64 {
        self[i]
    }

    fn set(&mut self, i: usize, v: f64) {
        self[i] = v;
    }
}

/// Classical fourth-order Runge-Kutta step.
///
/// Advances `state` by one timestep `dt` using the derivative function `f`.
#[must_use]
pub fn rk4_step<S: OdeState>(state: &S, dt: f64, f: impl Fn(&S) -> S) -> S {
    let half_dt = 0.5 * dt;
    let sixth_dt = dt / 6.0;

    let k1 = f(state);

    let mut s1 = S::zero();
    for i in 0..S::DIM {
        s1.set(i, half_dt.mul_add(k1.get(i), state.get(i)));
    }
    let k2 = f(&s1);

    let mut s2 = S::zero();
    for i in 0..S::DIM {
        s2.set(i, half_dt.mul_add(k2.get(i), state.get(i)));
    }
    let k3 = f(&s2);

    let mut s3 = S::zero();
    for i in 0..S::DIM {
        s3.set(i, dt.mul_add(k3.get(i), state.get(i)));
    }
    let k4 = f(&s3);

    let mut result = S::zero();
    for i in 0..S::DIM {
        let slope = 2.0f64.mul_add(k2.get(i) + k3.get(i), k1.get(i) + k4.get(i));
        result.set(i, sixth_dt.mul_add(slope, state.get(i)));
    }
    result
}

/// Integrate an ODE from `state0` for `n_steps` of size `dt`.
#[must_use]
pub fn integrate<S: OdeState>(state0: &S, dt: f64, n_steps: usize, f: impl Fn(&S) -> S) -> S {
    let mut state = *state0;
    for _ in 0..n_steps {
        state = rk4_step(&state, dt, &f);
    }
    state
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rk4_constant_derivative() {
        let state = [1.0, 2.0, 3.0, 4.0, 5.0];
        let result = rk4_step(&state, 0.1, |_s| [1.0, 0.0, -1.0, 0.0, 0.0]);
        assert!((result[0] - 1.1).abs() < 1e-12);
        assert!((result[2] - 2.9).abs() < 1e-12);
    }

    #[test]
    #[expect(clippy::float_cmp, reason = "bitwise determinism test")]
    fn integrate_deterministic() {
        let state = [0.5, 1.0, 0.5, 1.0, 0.3];
        let a = rk4_step(&state, 0.01, |s| {
            let mut d = [0.0; 5];
            for i in 0..5 {
                d[i] = -s[i];
            }
            d
        });
        let b = rk4_step(&state, 0.01, |s| {
            let mut d = [0.0; 5];
            for i in 0..5 {
                d[i] = -s[i];
            }
            d
        });
        assert_eq!(a, b); // bitwise determinism: same inputs must produce identical f64 outputs
    }
}
