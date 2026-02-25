// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Deterministic pseudo-random number generation.
//!
//! Provides [`Xorshift64`] for reproducible sampling in Monte Carlo
//! simulations and rarefaction analysis.  Uses xorshift64 with shifts
//! (13, 7, 17) per Marsaglia (2003).
//!
//! # GPU Evolution
//!
//! `BarraCUDA` uses xoshiro128\*\* (`ops::prng_xoshiro_wgsl`).  When
//! groundSpring enables the `barracuda` feature gate the PRNG will
//! align with the GPU kernel.  This CPU xorshift64 implementation is
//! retained as the validation reference.

use std::f64::consts::TAU;

/// Xorshift64 pseudo-random number generator.
///
/// Deterministic, fast, and adequate for Monte Carlo sampling.
/// Not cryptographically secure.
#[derive(Debug, Clone)]
pub struct Xorshift64 {
    state: u64,
}

impl Xorshift64 {
    /// Create a new generator from the given seed.
    ///
    /// A seed of zero is replaced with a non-zero constant because
    /// xorshift requires non-zero state.
    #[must_use]
    pub const fn new(seed: u64) -> Self {
        let state = if seed == 0 {
            0x9E37_79B9_7F4A_7C15
        } else {
            seed
        };
        Self { state }
    }

    /// Advance state and return the next raw `u64`.
    pub const fn next_u64(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    /// Uniform random value in \[0, 1).
    pub fn next_f64(&mut self) -> f64 {
        crate::cast::u64_f64(self.next_u64()) / crate::cast::u64_f64(u64::MAX)
    }

    /// Standard normal variate via Box-Muller transform.
    pub fn next_normal(&mut self) -> f64 {
        let u1 = self.next_f64().max(f64::MIN_POSITIVE);
        let u2 = self.next_f64();
        (-2.0 * u1.ln()).sqrt() * (TAU * u2).cos()
    }

    /// Normal variate with given mean and standard deviation.
    pub fn normal(&mut self, mean: f64, std_dev: f64) -> f64 {
        std_dev.mul_add(self.next_normal(), mean)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_same_seed() {
        let mut a = Xorshift64::new(42);
        let mut b = Xorshift64::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seeds_diverge() {
        let mut a = Xorshift64::new(42);
        let mut b = Xorshift64::new(99);
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn zero_seed_does_not_stick() {
        let mut rng = Xorshift64::new(0);
        let v1 = rng.next_u64();
        let v2 = rng.next_u64();
        assert_ne!(v1, 0);
        assert_ne!(v1, v2);
    }

    #[test]
    fn next_f64_in_unit_interval() {
        let mut rng = Xorshift64::new(42);
        for _ in 0..1_000 {
            let u = rng.next_f64();
            assert!((0.0..1.0).contains(&u), "got {u}");
        }
    }

    #[test]
    fn normal_has_roughly_zero_mean() {
        let mut rng = Xorshift64::new(42);
        let n = 10_000_i32;
        let sum: f64 = (0..n).map(|_| rng.next_normal()).sum();
        let mean = sum / f64::from(n);
        assert!(
            mean.abs() < 0.1,
            "mean of {n} standard normals should be near 0, got {mean}"
        );
    }

    #[test]
    fn normal_with_mean_and_std() {
        let mut rng = Xorshift64::new(42);
        let n = 10_000_i32;
        let target_mean = 5.0;
        let target_std = 2.0;
        let sum: f64 = (0..n).map(|_| rng.normal(target_mean, target_std)).sum();
        let mean = sum / f64::from(n);
        assert!(
            (mean - target_mean).abs() < 0.2,
            "mean should be near {target_mean}, got {mean}"
        );
    }

    #[test]
    fn normal_deterministic_bitwise() {
        let mut a = Xorshift64::new(42);
        let mut b = Xorshift64::new(42);
        for _ in 0..1_000 {
            assert_eq!(a.next_normal().to_bits(), b.next_normal().to_bits());
        }
    }

    #[test]
    fn backward_compatible_with_inline_xorshift() {
        let seed: u64 = 42;

        let mut rng = Xorshift64::new(seed);
        let rng_val = rng.next_f64();

        let mut state = seed;
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let inline_val = crate::cast::u64_f64(state) / crate::cast::u64_f64(u64::MAX);

        assert!(
            (rng_val - inline_val).abs() < f64::EPSILON,
            "Xorshift64 struct must match inline xorshift: {rng_val} vs {inline_val}"
        );
    }
}
