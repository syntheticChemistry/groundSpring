// SPDX-License-Identifier: AGPL-3.0-only
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
#[expect(
    clippy::cast_precision_loss,
    reason = "n_steps ≤ 10 000; no precision loss at this scale"
)]
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
        assert_eq!(a, b);
    }

    /// Exponential decay: dy/dt = -y, y(0) = 1  →  y(t) = e^{-t}.
    ///
    /// RK4 with small dt should match the analytical solution to ~O(dt^4).
    #[test]
    fn exponential_decay_matches_analytical() {
        let state = [1.0, 0.0, 0.0, 0.0, 0.0];
        let dt = 0.001;
        let n_steps = 1000; // integrate to t = 1.0
        let result = integrate(&state, dt, n_steps, |s| {
            let mut d = [0.0; 5];
            d[0] = -s[0];
            d
        });
        let expected = (-1.0_f64).exp();
        // RK4 global error is O(dt^4) ≈ 1e-12 for dt=0.001, 1000 steps
        assert!(
            (result[0] - expected).abs() < 1e-10,
            "exponential decay: got {}, expected {expected}",
            result[0]
        );
    }

    /// Simple harmonic oscillator: dx/dt = v, dv/dt = -x.
    ///
    /// Analytical: x(t) = cos(t), v(t) = -sin(t) with x(0)=1, v(0)=0.
    #[test]
    fn harmonic_oscillator_energy_conservation() {
        let state = [1.0, 0.0, 0.0, 0.0, 0.0]; // x=1, v=0, rest unused
        let dt = 0.001;
        let n_steps = 6283; // ≈ 2π ≈ one full period
        let result = integrate(&state, dt, n_steps, |s| {
            let mut d = [0.0; 5];
            d[0] = s[1]; // dx/dt = v
            d[1] = -s[0]; // dv/dt = -x
            d
        });
        let t = dt * n_steps as f64;
        let expected_x = t.cos();
        let expected_v = -t.sin();
        assert!(
            (result[0] - expected_x).abs() < 1e-6,
            "SHO x: got {}, expected {expected_x}",
            result[0]
        );
        assert!(
            (result[1] - expected_v).abs() < 1e-6,
            "SHO v: got {}, expected {expected_v}",
            result[1]
        );
        // Energy conservation: E = ½(x² + v²) should remain ≈ 0.5
        let energy = 0.5 * result[0].mul_add(result[0], result[1] * result[1]);
        assert!(
            (energy - 0.5).abs() < 1e-8,
            "SHO energy drift: got {energy}, expected 0.5"
        );
    }

    /// Coupled linear system: d/dt [x, y] = [[0, 1], [-1, 0]] [x, y]
    /// with x(0)=0, y(0)=1  →  x(t)=sin(t), y(t)=cos(t).
    #[test]
    fn coupled_rotation_matches_analytical() {
        let state = [0.0, 1.0, 0.0, 0.0, 0.0]; // x=0, y=1
        let dt = 0.0005;
        let n_steps = 2000; // t = 1.0
        let result = integrate(&state, dt, n_steps, |s| {
            let mut d = [0.0; 5];
            d[0] = s[1]; // dx/dt = y
            d[1] = -s[0]; // dy/dt = -x
            d
        });
        let t = dt * n_steps as f64;
        assert!(
            (result[0] - t.sin()).abs() < 1e-10,
            "rotation x: got {}, expected {}",
            result[0],
            t.sin()
        );
        assert!(
            (result[1] - t.cos()).abs() < 1e-10,
            "rotation y: got {}, expected {}",
            result[1],
            t.cos()
        );
    }

    /// Logistic growth: dy/dt = y(1 - y), y(0) = 0.1.
    ///
    /// Analytical: y(t) = 1 / (1 + 9·e^{-t}).
    #[test]
    fn logistic_growth_matches_analytical() {
        let y0 = 0.1;
        let state = [y0, 0.0, 0.0, 0.0, 0.0];
        let dt = 0.001;
        let n_steps = 5000; // t = 5.0
        let result = integrate(&state, dt, n_steps, |s| {
            let mut d = [0.0; 5];
            d[0] = s[0] * (1.0 - s[0]);
            d
        });
        let t = dt * n_steps as f64;
        let expected = 1.0 / 9.0f64.mul_add((-t).exp(), 1.0);
        assert!(
            (result[0] - expected).abs() < 1e-8,
            "logistic: got {}, expected {expected}",
            result[0]
        );
    }
}
