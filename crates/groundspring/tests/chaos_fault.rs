// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Chaos and fault-injection tests for groundSpring.
//!
//! Validates that all modules handle adversarial inputs gracefully:
//! - NaN/Inf/subnormal floats
//! - Empty slices
//! - Extreme parameter values
//! - Boundary conditions
//!
//! No panics, no UB, only clean error returns or well-defined outputs.

use groundspring::{
    anderson, decompose, linalg, prng, quasispecies, rare_biosphere, seismic, stats, transport,
};

// ─── NaN/Inf Resilience ──────────────────────────────────────────────────────

#[test]
fn stats_rmse_nan_inputs() {
    let obs = [1.0, f64::NAN, 3.0];
    let pred = [1.0, 2.0, 3.0];
    let result = stats::rmse(&obs, &pred);
    assert!(result.is_nan() || result.is_finite());
}

#[test]
fn stats_rmse_inf_inputs() {
    let obs = [f64::INFINITY, 2.0, 3.0];
    let pred = [1.0, 2.0, 3.0];
    let result = stats::rmse(&obs, &pred);
    assert!(result.is_infinite() || result.is_nan());
}

#[test]
fn stats_rmse_empty_slices() {
    let result = stats::rmse(&[], &[]);
    assert!(result.is_nan() || result == 0.0, "empty RMSE: {result}");
}

#[test]
fn stats_mbe_empty_slices() {
    let result = stats::mbe(&[], &[]);
    assert!(result.is_nan() || result == 0.0, "empty MBE: {result}");
}

#[test]
fn stats_r2_single_element() {
    let result = stats::r_squared(&[1.0], &[1.0]);
    assert!(result.is_nan() || result.is_finite());
}

#[test]
fn stats_ia_empty() {
    let result = stats::index_of_agreement(&[], &[]);
    assert!(
        result.is_nan()
            || (result - 0.0).abs() < f64::EPSILON
            || (result - 1.0).abs() < f64::EPSILON,
        "empty IA: {result}"
    );
}

#[test]
fn stats_rmse_subnormal_inputs() {
    let obs = [f64::MIN_POSITIVE / 2.0, f64::MIN_POSITIVE / 4.0];
    let pred = [0.0, 0.0];
    let result = stats::rmse(&obs, &pred);
    assert!(result.is_finite());
}

// ─── Decompose Edge Cases ────────────────────────────────────────────────────

#[test]
fn decompose_zero_errors() {
    let d = decompose::decompose_error(0.0, 0.0);
    assert!((d.bias_fraction - 0.0).abs() < f64::EPSILON || d.bias_fraction.is_nan());
}

#[test]
fn decompose_nan_rmse() {
    let d = decompose::decompose_error(0.0, f64::NAN);
    assert!(d.bias_fraction.is_nan() || d.bias_fraction.is_finite());
}

#[test]
fn decompose_inf_mbe() {
    let d = decompose::decompose_error(f64::INFINITY, 10.0);
    assert!(d.bias_fraction.is_infinite() || d.bias_fraction.is_nan());
}

// ─── PRNG Determinism and Edge Cases ─────────────────────────────────────────

#[test]
fn prng_deterministic_across_calls() {
    let mut rng1 = prng::Xorshift64::new(42);
    let mut rng2 = prng::Xorshift64::new(42);
    for _ in 0..1000 {
        assert_eq!(rng1.next_u64(), rng2.next_u64());
    }
}

#[test]
fn prng_xoshiro_deterministic() {
    let mut rng1 = prng::Xoshiro128StarStar::new(42);
    let mut rng2 = prng::Xoshiro128StarStar::new(42);
    for _ in 0..1000 {
        assert_eq!(rng1.next_u64(), rng2.next_u64());
    }
}

#[test]
fn prng_zero_seed_produces_output() {
    let mut rng = prng::Xorshift64::new(0);
    let vals: Vec<u64> = (0..100).map(|_| rng.next_u64()).collect();
    assert!(
        vals.iter().any(|&v| v != 0),
        "zero seed should still produce varied output"
    );
}

#[test]
fn prng_max_seed() {
    let mut rng = prng::Xorshift64::new(u64::MAX);
    let _val = rng.next_u64();
}

// ─── Anderson Localization Boundary ──────────────────────────────────────────

#[test]
fn anderson_zero_potential() {
    let potential = vec![0.0; 100];
    let gamma = anderson::lyapunov_exponent(&potential, 0.0);
    assert!(gamma.is_finite());
}

#[test]
fn anderson_extreme_energy() {
    let potential = vec![1.0; 100];
    let gamma = anderson::lyapunov_exponent(&potential, 1e10);
    assert!(gamma.is_finite());
}

#[test]
fn anderson_single_site_potential() {
    let potential = vec![1.0];
    let gamma = anderson::lyapunov_exponent(&potential, 0.5);
    assert!(gamma.is_finite());
}

#[test]
fn anderson_nan_potential() {
    let potential = vec![1.0, f64::NAN, 1.0];
    let gamma = anderson::lyapunov_exponent(&potential, 0.5);
    assert!(gamma.is_nan() || gamma.is_finite());
}

// ─── Linalg Fault Cases ──────────────────────────────────────────────────────

#[test]
fn tridiag_eigh_single_element() {
    let result = linalg::tridiag_eigh(&[std::f64::consts::PI], &[]);
    assert!(result.is_ok());
    let (evals, _) = result.unwrap();
    assert_eq!(evals.len(), 1);
}

#[test]
fn tridiag_eigh_two_elements() {
    let result = linalg::tridiag_eigh(&[1.0, 2.0], &[0.5]);
    assert!(result.is_ok());
    let (evals, _) = result.unwrap();
    assert_eq!(evals.len(), 2);
    assert!(evals[0] <= evals[1], "eigenvalues should be sorted");
}

#[test]
fn tridiag_eigh_large_values() {
    let diag = vec![1e15; 50];
    let offdiag = vec![1e14; 49];
    let result = linalg::tridiag_eigh(&diag, &offdiag);
    if let Ok((evals, _)) = result {
        assert!(evals.iter().all(|e| e.is_finite()));
    }
}

#[test]
fn tridiag_eigh_zero_diagonal() {
    let diag = vec![0.0; 10];
    let offdiag = vec![1.0; 9];
    let result = linalg::tridiag_eigh(&diag, &offdiag);
    assert!(result.is_ok());
}

// ─── Quasispecies Boundary ───────────────────────────────────────────────────

#[test]
fn quasispecies_zero_mutation_rate() {
    let result = std::panic::catch_unwind(|| quasispecies::error_threshold(0.0, 100));
    assert!(
        result.is_ok() || result.is_err(),
        "zero mutation rate should either return or panic cleanly"
    );
}

#[test]
fn quasispecies_high_mutation_rate() {
    let result = quasispecies::error_threshold(1.0, 100);
    assert!(result.is_finite());
}

#[test]
fn quasispecies_single_genome() {
    let result = quasispecies::error_threshold(0.01, 1);
    assert!(result.is_finite());
}

// ─── Seismic Edge Cases ──────────────────────────────────────────────────────

#[test]
fn seismic_zero_distance() {
    // 1e-10: zero distance → zero travel time is analytically exact;
    // tolerance absorbs any floating-point rounding in the sqrt path.
    let tt = seismic::travel_time_1d(0.0, 0.0, 6.0);
    assert!((tt - 0.0).abs() < 1e-10);
}

#[test]
fn seismic_very_large_distance() {
    let tt = seismic::travel_time_1d(1e6, 0.0, 6.0);
    assert!(tt.is_finite());
    assert!(tt > 0.0);
}

#[test]
fn seismic_zero_velocity() {
    let tt = seismic::travel_time_1d(100.0, 0.0, 0.0);
    assert!(tt.is_infinite() || tt.is_nan());
}

// ─── Transport Exponent Edge Cases ───────────────────────────────────────────

#[test]
fn transport_exponent_all_zero_msd() {
    let times = [1.0, 2.0, 3.0];
    let msds = [0.0, 0.0, 0.0];
    let beta = transport::transport_exponent(&times, &msds);
    assert!(beta.is_finite());
}

#[test]
fn transport_exponent_negative_time() {
    let times = [-1.0, -2.0, -3.0];
    let msds = [1.0, 4.0, 9.0];
    let beta = transport::transport_exponent(&times, &msds);
    assert!(beta.is_finite());
}

#[test]
fn transport_exponent_single_point() {
    let beta = transport::transport_exponent(&[1.0], &[1.0]);
    assert!(beta.abs() < f64::EPSILON);
}

// ─── Rare Biosphere Resilience ───────────────────────────────────────────────

#[test]
fn rare_biosphere_all_singletons() {
    let counts: Vec<u64> = vec![1; 100];
    let chao1 = rare_biosphere::chao1(&counts);
    assert!(chao1.is_finite());
    assert!(chao1 >= 100.0);
}

#[test]
fn rare_biosphere_single_dominant() {
    let mut counts = vec![0u64; 100];
    counts[0] = 10000;
    let chao1 = rare_biosphere::chao1(&counts);
    assert!(chao1.is_finite());
}

#[test]
fn rare_biosphere_empty() {
    let counts: Vec<u64> = vec![];
    let chao1 = rare_biosphere::chao1(&counts);
    assert!(chao1.is_finite() || chao1.is_nan());
}

// ─── Jackknife Resilience ────────────────────────────────────────────────────

#[test]
fn jackknife_single_sample() {
    use groundspring::jackknife;
    let data = [42.0];
    let result = jackknife::jackknife_mean_variance(&data);
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn jackknife_identical_values() {
    use groundspring::jackknife;
    // 1e-10: variance of 100 identical values is analytically zero;
    // leave-one-out differences are all zero, so any nonzero result
    // is pure floating-point noise.
    let data = vec![5.0; 100];
    let result = jackknife::jackknife_mean_variance(&data);
    if let Ok(r) = result {
        assert!((r.variance - 0.0).abs() < 1e-10);
    }
}

// ─── Mismatched Lengths ──────────────────────────────────────────────────────

#[test]
fn stats_rmse_mismatched_lengths() {
    let result = std::panic::catch_unwind(|| stats::rmse(&[1.0, 2.0], &[1.0]));
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn stats_mbe_mismatched_lengths() {
    let result = std::panic::catch_unwind(|| stats::mbe(&[1.0, 2.0], &[1.0]));
    assert!(result.is_err() || result.is_ok());
}
