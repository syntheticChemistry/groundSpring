// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Determinism tests: verify that identical seeds produce bitwise-identical
//! results across reruns. Any failure here indicates non-deterministic
//! floating-point paths (thread scheduling, unordered reductions, etc.).

// Bitwise determinism: tests intentionally compare exact f64 bits across runs.
#![expect(
    clippy::float_cmp,
    reason = "determinism tests require bitwise f64 equality"
)]

use groundspring::almost_mathieu::{eigenvalues, level_spacing_ratio, potential};
use groundspring::anderson::lyapunov_exponent;
use groundspring::bistable::{BistableParams, integrate as bistable_integrate};
use groundspring::bootstrap::{bootstrap_mean, rawr_mean};
use groundspring::drift::wright_fisher_fixation;
use groundspring::gillespie::birth_death_ssa;
use groundspring::multisignal::{MultiSignalParams, integrate as multi_integrate};
use groundspring::prng::Xorshift64;
use groundspring::rarefaction::{multinomial_sample, rarefaction_at_depth};
use groundspring::transport::{tridiag_eigh, wavepacket_msd};

const GOLDEN: f64 = 0.618_033_988_749_894_9;

#[test]
fn prng_deterministic() {
    let mut a = Xorshift64::new(42);
    let mut b = Xorshift64::new(42);
    for _ in 0..1000 {
        assert_eq!(a.next_u64(), b.next_u64());
    }
}

#[test]
fn bootstrap_deterministic() {
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let r1 = bootstrap_mean(&data, 500, 0.95, 42);
    let r2 = bootstrap_mean(&data, 500, 0.95, 42);
    assert_eq!(r1.estimate, r2.estimate);
    assert_eq!(r1.ci_lower, r2.ci_lower);
    assert_eq!(r1.ci_upper, r2.ci_upper);
}

#[test]
fn rawr_deterministic() {
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let r1 = rawr_mean(&data, 500, 0.95, 42);
    let r2 = rawr_mean(&data, 500, 0.95, 42);
    assert_eq!(r1.estimate, r2.estimate);
    assert_eq!(r1.ci_lower, r2.ci_lower);
    assert_eq!(r1.ci_upper, r2.ci_upper);
}

#[test]
fn multinomial_deterministic() {
    let probs = [0.5, 0.3, 0.15, 0.05];
    let r1 = multinomial_sample(&probs, 10_000, 42);
    let r2 = multinomial_sample(&probs, 10_000, 42);
    assert_eq!(r1, r2);
}

#[test]
fn rarefaction_deterministic() {
    let community: Vec<f64> = (1..=10).rev().map(|x| f64::from(x) / 55.0).collect();
    let r1 = rarefaction_at_depth(&community, 5000, 30, 42);
    let r2 = rarefaction_at_depth(&community, 5000, 30, 42);
    assert_eq!(r1.genera_mean, r2.genera_mean);
    assert_eq!(r1.shannon_mean, r2.shannon_mean);
}

#[test]
fn anderson_lyapunov_deterministic() {
    let pot1 = potential(100_000, 3.0, GOLDEN, 0.0);
    let pot2 = potential(100_000, 3.0, GOLDEN, 0.0);
    assert_eq!(pot1, pot2);
    let g1 = lyapunov_exponent(&pot1, 0.0);
    let g2 = lyapunov_exponent(&pot2, 0.0);
    assert_eq!(g1, g2);
}

#[test]
fn eigenvalue_deterministic() {
    let e1 = eigenvalues(50, 2.0, GOLDEN, 0.0);
    let e2 = eigenvalues(50, 2.0, GOLDEN, 0.0);
    assert_eq!(e1, e2);
}

#[test]
fn level_spacing_deterministic() {
    let mut e1 = eigenvalues(50, 1.0, GOLDEN, 0.0);
    let mut e2 = e1.clone();
    assert_eq!(level_spacing_ratio(&mut e1), level_spacing_ratio(&mut e2));
}

#[test]
fn bistable_ode_deterministic() {
    let p = BistableParams::default();
    let ic = [0.95, 4.5, 1.9, 0.3, 0.02];
    let r1 = bistable_integrate(&ic, &p, 0.01, 5_000);
    let r2 = bistable_integrate(&ic, &p, 0.01, 5_000);
    assert_eq!(r1, r2);
}

#[test]
fn multisignal_ode_deterministic() {
    let p = MultiSignalParams::default();
    let ic = [0.1, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0];
    let r1 = multi_integrate(&ic, &p, 0.01, 5_000);
    let r2 = multi_integrate(&ic, &p, 0.01, 5_000);
    assert_eq!(r1, r2);
}

#[test]
fn gillespie_deterministic() {
    let rates = [1.0; 10];
    let r1 = birth_death_ssa(&rates, 0.5, 10, 50.0, 42);
    let r2 = birth_death_ssa(&rates, 0.5, 10, 50.0, 42);
    assert_eq!(r1.times, r2.times);
    assert_eq!(r1.states, r2.states);
}

#[test]
fn wright_fisher_deterministic() {
    let r1 = wright_fisher_fixation(100, 0.5, 0.0, 42);
    let r2 = wright_fisher_fixation(100, 0.5, 0.0, 42);
    assert_eq!(r1, r2);
}

#[test]
fn transport_deterministic() {
    let pot = potential(51, 1.0, GOLDEN, 0.0);
    let offdiag = vec![1.0; 50];
    let (evals1, evecs1) = tridiag_eigh(&pot, &offdiag).expect("eigh run 1");
    let (evals2, evecs2) = tridiag_eigh(&pot, &offdiag).expect("eigh run 2");
    assert_eq!(evals1, evals2);
    assert_eq!(evecs1, evecs2);
    let (msd1, norm1) = wavepacket_msd(&evals1, &evecs1, 25, 5.0);
    let (msd2, norm2) = wavepacket_msd(&evals2, &evecs2, 25, 5.0);
    assert_eq!(msd1, msd2);
    assert_eq!(norm1, norm2);
}
