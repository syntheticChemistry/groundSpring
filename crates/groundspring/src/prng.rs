// SPDX-License-Identifier: AGPL-3.0-only
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

/// Default PRNG type for groundSpring validation.
///
/// Uses `Xorshift64` for validation baselines. When barracuda GPU promotion
/// requires stream-compatible PRNG, use [`Xoshiro128StarStar`] directly.
pub type DefaultRng = Xorshift64;

/// GPU-aligned PRNG type matching barracuda's `ops::prng_xoshiro_wgsl`.
///
/// Use this type when generating seed/state vectors for GPU dispatch
/// (e.g., `BatchedMultinomialGpu`, `GillespieGpu`, `WrightFisherGpu`).
/// Ensures CPU-side state initialization produces the same stream as
/// the GPU shader.
pub type GpuAlignedRng = Xoshiro128StarStar;

/// Xorshift64 pseudo-random number generator.
///
/// Deterministic, fast, and adequate for Monte Carlo sampling.
/// Not cryptographically secure. This is the validation-baseline PRNG;
/// all existing benchmark JSON values were generated with this generator.
#[derive(Debug, Clone)]
pub struct Xorshift64 {
    state: u64,
}

impl Xorshift64 {
    /// Create a new generator from the given seed.
    ///
    /// A seed of zero is replaced with a non-zero constant because
    /// xorshift requires non-zero state.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut rng = groundspring::prng::Xorshift64::new(42);
    /// let v = rng.next_f64();
    /// assert!((0.0..1.0).contains(&v));
    /// // Deterministic: same seed produces same stream.
    /// let mut rng2 = groundspring::prng::Xorshift64::new(42);
    /// assert_eq!(rng2.next_u64(), groundspring::prng::Xorshift64::new(42).next_u64());
    /// ```
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

    /// Binomial variate: number of successes in `n` trials with probability `p`.
    ///
    /// Uses direct simulation (n Bernoulli trials). Adequate for n ≤ `10_000`;
    /// for larger n, consider a normal approximation.
    #[must_use]
    pub fn binomial(&mut self, n: usize, p: f64) -> u64 {
        let mut successes = 0u64;
        for _ in 0..n {
            if self.next_f64() < p {
                successes += 1;
            }
        }
        successes
    }
}

/// `SplitMix64` one-round mixing for seed initialization.
///
/// Reference: Steele, Lea, Flood (2014) — used by java.util.SplittableRandom.
const fn splitmix64(state: u64) -> u64 {
    let z = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    let z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Xoshiro128\*\* pseudo-random number generator (32-bit output).
///
/// Matches the PRNG used in barracuda WGSL compute shaders
/// (`ops::prng_xoshiro_wgsl`). Required for GPU stream-compatible
/// random number generation in Phase 2b.
///
/// Reference: Blackman & Vigna (2021) ACM Trans Math Softw 47:1-32.
///
/// Note: generates `u32` outputs. For `f64` sampling, two outputs are
/// combined to fill the mantissa (53-bit resolution via 32+21 split).
#[derive(Debug, Clone)]
pub struct Xoshiro128StarStar {
    s: [u32; 4],
}

impl Xoshiro128StarStar {
    /// Create a new generator from a 64-bit seed.
    ///
    /// The seed is split into two 32-bit halves and mixed via `SplitMix32`
    /// to initialise the four-word state. A zero seed is replaced.
    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "deliberate u64→u32 seed split"
    )]
    pub const fn new(seed: u64) -> Self {
        let seed = if seed == 0 {
            0x9E37_79B9_7F4A_7C15
        } else {
            seed
        };
        let z0 = splitmix64(seed);
        let z1 = splitmix64(z0);
        let z2 = splitmix64(z1);
        let z3 = splitmix64(z2);
        Self {
            s: [z0 as u32, z1 as u32, z2 as u32, z3 as u32],
        }
    }

    /// Advance state and return the next raw `u32`.
    pub const fn next_u32(&mut self) -> u32 {
        let result = (self.s[1].wrapping_mul(5)).rotate_left(7).wrapping_mul(9);
        let t = self.s[1] << 9;

        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];

        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(11);

        result
    }

    /// Uniform random value in \[0, 1) with 32-bit resolution.
    pub fn next_f64(&mut self) -> f64 {
        f64::from(self.next_u32()) / f64::from(u32::MAX)
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

    /// Combine two 32-bit outputs into a raw `u64`.
    pub fn next_u64(&mut self) -> u64 {
        let hi = u64::from(self.next_u32());
        let lo = u64::from(self.next_u32());
        (hi << 32) | lo
    }

    /// Binomial variate: number of successes in `n` trials with probability `p`.
    #[must_use]
    pub fn binomial(&mut self, n: usize, p: f64) -> u64 {
        let mut successes = 0u64;
        for _ in 0..n {
            if self.next_f64() < p {
                successes += 1;
            }
        }
        successes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tol;

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
            mean.abs() < tol::EQUILIBRIUM,
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

    // --- Xoshiro128StarStar tests ---

    #[test]
    fn xoshiro_deterministic_same_seed() {
        let mut a = Xoshiro128StarStar::new(42);
        let mut b = Xoshiro128StarStar::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u32(), b.next_u32());
        }
    }

    #[test]
    fn xoshiro_different_seeds_diverge() {
        let mut a = Xoshiro128StarStar::new(42);
        let mut b = Xoshiro128StarStar::new(99);
        assert_ne!(a.next_u32(), b.next_u32());
    }

    #[test]
    fn xoshiro_zero_seed_works() {
        let mut rng = Xoshiro128StarStar::new(0);
        let v1 = rng.next_u32();
        let v2 = rng.next_u32();
        assert_ne!(v1, 0);
        assert_ne!(v1, v2);
    }

    #[test]
    fn xoshiro_f64_in_unit_interval() {
        let mut rng = Xoshiro128StarStar::new(42);
        for _ in 0..1_000 {
            let u = rng.next_f64();
            assert!((0.0..1.0).contains(&u), "got {u}");
        }
    }

    #[test]
    fn xoshiro_normal_has_roughly_zero_mean() {
        let mut rng = Xoshiro128StarStar::new(42);
        let n = 10_000_i32;
        let sum: f64 = (0..n).map(|_| rng.next_normal()).sum();
        let mean = sum / f64::from(n);
        assert!(
            mean.abs() < tol::EQUILIBRIUM,
            "mean of {n} standard normals should be near 0, got {mean}"
        );
    }

    #[test]
    fn default_rng_is_xorshift64() {
        let mut a = DefaultRng::new(42);
        let mut b = Xorshift64::new(42);
        assert_eq!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn xoshiro_next_u64_deterministic() {
        let mut a = Xoshiro128StarStar::new(42);
        let mut b = Xoshiro128StarStar::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn xoshiro_binomial_deterministic() {
        let mut a = Xoshiro128StarStar::new(42);
        let mut b = Xoshiro128StarStar::new(42);
        assert_eq!(a.binomial(100, 0.5), b.binomial(100, 0.5));
    }

    #[test]
    fn xoshiro_binomial_mean_near_n_times_p() {
        let mut rng = Xoshiro128StarStar::new(42);
        let n = 1000;
        let p = 0.3;
        let trials = 500;
        let sum: u64 = (0..trials).map(|_| rng.binomial(n, p)).sum();
        #[expect(clippy::cast_precision_loss, reason = "test counter; value < 2^52")]
        let mean = sum as f64 / f64::from(trials);
        #[expect(clippy::cast_precision_loss, reason = "test parameter; value < 2^52")]
        let expected = n as f64 * p;
        assert!(
            (mean - expected).abs() < 15.0,
            "binomial mean {mean} should be near {expected}"
        );
    }

    #[test]
    fn xoshiro_normal_with_mean_and_std() {
        let mut rng = Xoshiro128StarStar::new(42);
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
}
