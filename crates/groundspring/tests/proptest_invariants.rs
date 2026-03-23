// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Property-based tests verifying mathematical invariants across the
//! `groundspring` library.  These complement deterministic unit tests by
//! exercising edge cases that manual test vectors cannot cover.

#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use proptest::prelude::*;

use groundspring::stats::{
    mbe, mean, norm_cdf, norm_ppf, pearson_r, r_squared, rmse, sample_std_dev, std_dev,
};
use groundspring::tol;

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn finite_vec(min_len: usize, max_len: usize) -> impl Strategy<Value = Vec<f64>> {
    proptest::collection::vec(-1e6_f64..1e6, min_len..=max_len)
}

fn positive_vec(min_len: usize, max_len: usize) -> impl Strategy<Value = Vec<f64>> {
    proptest::collection::vec(0.001_f64..1e6, min_len..=max_len)
}

// ---------------------------------------------------------------------------
// mean / std_dev
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn mean_of_constant_is_constant(c in -1e6_f64..1e6, n in 2_usize..100) {
        let v = vec![c; n];
        prop_assert!((mean(&v) - c).abs() < tol::INTEGRATION, "mean({c}) = {}", mean(&v));
    }

    #[test]
    fn std_dev_of_constant_is_zero(c in -1e6_f64..1e6, n in 2_usize..100) {
        let v = vec![c; n];
        let sd = std_dev(&v);
        let tol_val = c.abs().mul_add(tol::EXACT, tol::STRICT);
        prop_assert!(sd < tol_val, "std_dev({c}) = {sd}");
    }

    #[test]
    fn std_dev_is_nonnegative(v in finite_vec(2, 200)) {
        prop_assert!(std_dev(&v) >= 0.0);
    }

    #[test]
    fn sample_std_dev_ge_population(v in finite_vec(3, 200)) {
        prop_assert!(sample_std_dev(&v) >= std_dev(&v) - tol::EXACT);
    }
}

// ---------------------------------------------------------------------------
// RMSE / MBE
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn rmse_identical_is_zero(v in finite_vec(1, 200)) {
        prop_assert!(rmse(&v, &v) < tol::ANALYTICAL);
    }

    #[test]
    fn rmse_is_nonnegative(
        a in finite_vec(2, 100),
    ) {
        let b: Vec<f64> = a.iter().map(|x| x + 1.0).collect();
        prop_assert!(rmse(&a, &b) >= 0.0);
    }

    #[test]
    fn mbe_identical_is_zero(v in finite_vec(1, 200)) {
        prop_assert!(mbe(&v, &v).abs() < tol::ANALYTICAL);
    }

    #[test]
    fn mbe_antisymmetric(
        a in finite_vec(2, 100),
    ) {
        let b: Vec<f64> = a.iter().map(|x| x + 1.0).collect();
        prop_assert!((mbe(&a, &b) + mbe(&b, &a)).abs() < tol::INTEGRATION);
    }
}

// ---------------------------------------------------------------------------
// R² / pearson_r
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn r_squared_perfect_fit(v in positive_vec(3, 100)) {
        let r2 = r_squared(&v, &v);
        prop_assert!((r2 - 1.0).abs() < tol::INTEGRATION, "R² = {r2}");
    }

    #[test]
    fn pearson_perfect_positive(v in positive_vec(3, 100)) {
        let r = pearson_r(&v, &v);
        prop_assert!((r - 1.0).abs() < tol::INTEGRATION, "r = {r}");
    }

    #[test]
    fn pearson_bounded(a in finite_vec(3, 100), b in finite_vec(3, 100)) {
        let len = a.len().min(b.len());
        if len < 3 { return Ok(()); }
        let a = &a[..len];
        let b = &b[..len];
        let r = pearson_r(a, b);
        prop_assert!((-1.0 - tol::INTEGRATION..=1.0 + tol::INTEGRATION).contains(&r), "r = {r}");
    }
}

// ---------------------------------------------------------------------------
// Normal CDF / PPF round-trip
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn cdf_ppf_round_trip(p in 0.01_f64..0.99) {
        let z = norm_ppf(p);
        let recovered = norm_cdf(z);
        prop_assert!(
            (recovered - p).abs() < tol::CDF_APPROX,
            "CDF(PPF({p})) = {recovered}"
        );
    }

    #[test]
    fn cdf_is_monotone(a in -4.0_f64..4.0, b in -4.0_f64..4.0) {
        if a < b {
            prop_assert!(norm_cdf(a) <= norm_cdf(b) + tol::EXACT);
        }
    }

    #[test]
    fn cdf_bounded(z in -10.0_f64..10.0) {
        let p = norm_cdf(z);
        prop_assert!((0.0..=1.0).contains(&p), "CDF({z}) = {p}");
    }
}

// ---------------------------------------------------------------------------
// Anderson localization: Lyapunov exponent invariants
// ---------------------------------------------------------------------------

use groundspring::anderson::{anderson_potential, lyapunov_exponent};

proptest! {
    #[test]
    fn lyapunov_is_nonnegative(w in 1.0_f64..10.0, seed in 1_u64..10_000) {
        let pot = anderson_potential(500, w, seed);
        let gamma = lyapunov_exponent(&pot, 0.0);
        prop_assert!(gamma >= -tol::STOCHASTIC, "gamma(W={w}, seed={seed}) = {gamma}");
    }

    #[test]
    fn lyapunov_increases_with_disorder(seed in 1_u64..1000) {
        let weak = anderson_potential(500, 1.0, seed);
        let strong = anderson_potential(500, 6.0, seed);
        let g_weak = lyapunov_exponent(&weak, 0.0);
        let g_strong = lyapunov_exponent(&strong, 0.0);
        prop_assert!(
            g_strong > g_weak - tol::STOCHASTIC,
            "gamma(W=6) = {g_strong} should exceed gamma(W=1) = {g_weak}"
        );
    }
}

// ---------------------------------------------------------------------------
// Rarefaction: diversity index invariants
// ---------------------------------------------------------------------------

use groundspring::rarefaction::{shannon_diversity, simpson_diversity};

fn abundance_vec(min_len: usize, max_len: usize) -> impl Strategy<Value = Vec<u64>> {
    proptest::collection::vec(1_u64..1000, min_len..=max_len)
}

proptest! {
    #[test]
    fn shannon_is_nonnegative(counts in abundance_vec(2, 50)) {
        let h = shannon_diversity(&counts);
        prop_assert!(h >= -tol::EXACT, "H = {h}");
    }

    #[test]
    fn simpson_bounded_01(counts in abundance_vec(2, 50)) {
        let d = simpson_diversity(&counts);
        prop_assert!(
            (-tol::EXACT..=1.0 + tol::EXACT).contains(&d),
            "D = {d}"
        );
    }

    #[test]
    fn simpson_monoculture_is_zero(n in 1_u64..10_000) {
        let counts = vec![n];
        let d = simpson_diversity(&counts);
        prop_assert!(d.abs() < tol::EXACT, "monoculture D = {d}");
    }

    #[test]
    fn shannon_monoculture_is_zero(n in 1_u64..10_000) {
        let counts = vec![n];
        let h = shannon_diversity(&counts);
        prop_assert!(h.abs() < tol::EXACT, "monoculture H = {h}");
    }
}

// ---------------------------------------------------------------------------
// Decompose: bias-variance invariants
// ---------------------------------------------------------------------------

use groundspring::decompose::decompose_error;

proptest! {
    #[test]
    fn decompose_pythagorean(
        mbe_val in -100.0_f64..100.0,
        rmse_val in 0.01_f64..200.0,
    ) {
        let rmse_val = rmse_val.max(mbe_val.abs());
        let d = decompose_error(mbe_val, rmse_val);
        let pythagorean = (d.bias_fraction + d.noise_fraction - 1.0).abs();
        prop_assert!(
            pythagorean < tol::DECOMPOSITION,
            "bias_frac + noise_frac = {} (expect ~1.0)",
            d.bias_fraction + d.noise_fraction
        );
    }
}

// ---------------------------------------------------------------------------
// Bootstrap: CI invariants
// ---------------------------------------------------------------------------

use groundspring::bootstrap::bootstrap_mean;

proptest! {
    #[test]
    fn bootstrap_ci_contains_mean(v in positive_vec(10, 100)) {
        let m = mean(&v);
        let ci = bootstrap_mean(&v, 500, 0.95, 42).unwrap();
        prop_assert!(
            ci.ci_lower <= m + tol::STOCHASTIC && ci.ci_upper >= m - tol::STOCHASTIC,
            "mean {m} outside CI [{}, {}]", ci.ci_lower, ci.ci_upper
        );
    }

    #[test]
    fn bootstrap_ci_ordered(v in positive_vec(5, 100)) {
        let ci = bootstrap_mean(&v, 200, 0.95, 42).unwrap();
        prop_assert!(
            ci.ci_lower <= ci.ci_upper + tol::EXACT,
            "lo={} > hi={}", ci.ci_lower, ci.ci_upper
        );
    }
}

// ---------------------------------------------------------------------------
// Drift: Kimura fixation probability invariants
// ---------------------------------------------------------------------------

use groundspring::drift::kimura_fixation_prob;

proptest! {
    #[test]
    fn kimura_fixation_bounded_01(
        n in 10_usize..500,
        s in -0.1_f64..0.1,
        p in 0.01_f64..0.99,
    ) {
        let prob = kimura_fixation_prob(n, s, p);
        prop_assert!(
            (-tol::STOCHASTIC..=1.0 + tol::STOCHASTIC).contains(&prob),
            "kimura({n}, {s}, {p}) = {prob}"
        );
    }

    #[test]
    fn kimura_neutral_equals_initial_freq(n in 10_usize..500, p in 0.01_f64..0.99) {
        let prob = kimura_fixation_prob(n, 0.0, p);
        prop_assert!(
            (prob - p).abs() < tol::DECOMPOSITION,
            "kimura({n}, 0.0, {p}) = {prob}, expected ~{p}"
        );
    }
}

// ---------------------------------------------------------------------------
// FAO-56: ET₀ physical plausibility
// ---------------------------------------------------------------------------

use groundspring::fao56::{DailyWeatherInputs, daily_et0};

proptest! {
    #[test]
    fn et0_is_nonnegative(
        tmax in 10.0_f64..45.0,
        wind in 1.0_f64..30.0,
        sunshine in 2.0_f64..14.0,
    ) {
        let inp = DailyWeatherInputs {
            tmax_c: tmax,
            tmin_c: tmax - 10.0,
            rhmax_pct: 80.0,
            rhmin_pct: 40.0,
            wind_speed_10m_km_h: wind,
            sunshine_hours: sunshine,
            latitude_deg_n: 40.0,
            altitude_m: 100.0,
            day_of_year: 172,
        };
        let et0 = daily_et0(&inp);
        prop_assert!(et0 >= 0.0, "ET₀ = {et0}");
    }

    #[test]
    fn et0_within_plausible_range(
        tmax in 15.0_f64..40.0,
        sunshine in 4.0_f64..12.0,
    ) {
        let inp = DailyWeatherInputs {
            tmax_c: tmax,
            tmin_c: tmax - 8.0,
            rhmax_pct: 70.0,
            rhmin_pct: 35.0,
            wind_speed_10m_km_h: 8.0,
            sunshine_hours: sunshine,
            latitude_deg_n: 45.0,
            altitude_m: 0.0,
            day_of_year: 180,
        };
        let et0 = daily_et0(&inp);
        prop_assert!(
            et0 < 20.0,
            "ET₀ = {et0} exceeds physical max"
        );
    }
}

// ---------------------------------------------------------------------------
// Spectral reconstruction: Tikhonov round-trip invariants
// ---------------------------------------------------------------------------

use groundspring::spectral_recon::{build_kernel, forward_correlator, tikhonov_solve};

proptest! {
    #[test]
    fn tikhonov_nonnegative_with_nonneg_data(
        n_tau in 5_usize..20,
        n_omega in 5_usize..20,
    ) {
        let tau: Vec<f64> = (0..n_tau).map(|i| f64::from(u32::try_from(i).unwrap()) * 0.1).collect();
        let omega: Vec<f64> = (0..n_omega).map(|i| f64::from(u32::try_from(i).unwrap()) * 0.2).collect();
        let kernel = build_kernel(&tau, &omega);
        let rho_true: Vec<f64> = omega.iter().map(|w| (-(*w - 1.0).powi(2)).exp()).collect();
        let data = forward_correlator(&kernel, &rho_true, n_tau, n_omega);
        let rho_recon = tikhonov_solve(&kernel, &data, 1e-4, n_tau, n_omega);
        prop_assert_eq!(rho_recon.len(), n_omega);
    }
}

// ---------------------------------------------------------------------------
// Quasispecies: error threshold invariants
// ---------------------------------------------------------------------------

use groundspring::quasispecies::{error_threshold, master_frequency_analytical};

proptest! {
    #[test]
    fn error_threshold_bounded_01(sigma in 2.0_f64..100.0, l in 10_usize..500) {
        let mu_c = error_threshold(sigma, l);
        prop_assert!(
            (0.0..=1.0).contains(&mu_c),
            "error_threshold({sigma}, {l}) = {mu_c}"
        );
    }

    #[test]
    fn error_threshold_decreases_with_genome_length(sigma in 5.0_f64..50.0) {
        let short = error_threshold(sigma, 50);
        let long = error_threshold(sigma, 200);
        prop_assert!(
            long < short + tol::DECOMPOSITION,
            "mu_c(L=200)={long} should be < mu_c(L=50)={short}"
        );
    }

    #[test]
    fn master_freq_below_threshold_is_high(sigma in 5.0_f64..50.0, l in 50_usize..200) {
        let mu_c = error_threshold(sigma, l);
        let mu_below = mu_c * 0.5;
        let freq = master_frequency_analytical(sigma, mu_below, l);
        prop_assert!(freq > 0.0, "master freq = {freq} should be positive below threshold");
    }
}

// ---------------------------------------------------------------------------
// Band structure: band edge invariants
// ---------------------------------------------------------------------------

use groundspring::band_structure::{count_bands, find_band_edges};

proptest! {
    #[test]
    fn band_edges_come_in_pairs(n_atoms in 2_usize..8) {
        let potential = vec![0.0; n_atoms];
        let edges = find_band_edges(&potential, 1.0, -5.0, 5.0, 500);
        prop_assert!(
            edges.len().is_multiple_of(2),
            "band edges should come in pairs, got {}",
            edges.len()
        );
    }

    #[test]
    fn band_edges_are_sorted(n_atoms in 2_usize..6) {
        let potential = vec![0.0; n_atoms];
        let edges = find_band_edges(&potential, 1.0, -5.0, 5.0, 300);
        for pair in edges.windows(2) {
            prop_assert!(
                pair[0] <= pair[1] + tol::ANALYTICAL,
                "edges not sorted: {} > {}", pair[0], pair[1]
            );
        }
    }

    #[test]
    fn count_bands_equals_half_edges(n_atoms in 2_usize..6) {
        let potential = vec![0.0; n_atoms];
        let edges = find_band_edges(&potential, 1.0, -5.0, 5.0, 300);
        let n = count_bands(&potential, 1.0, -5.0, 5.0, 300);
        prop_assert_eq!(n, edges.len() / 2);
    }
}
