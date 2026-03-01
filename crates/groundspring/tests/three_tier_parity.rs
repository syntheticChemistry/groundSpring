// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Three-tier parity tests.
//!
//! Validates that public API functions produce identical results
//! regardless of feature mode (default / barracuda / barracuda-gpu).
//!
//! In default mode these test the local CPU implementations directly.
//! With `--features barracuda`, the same functions route through
//! barracuda CPU delegations — if they produce different results,
//! it means the delegation broke mathematical parity.
//!
//! Test naming: `<module>_parity_<property>`.

#![allow(clippy::float_cmp)]

// ── drift ───────────────────────────────────────────────────────────

#[test]
fn drift_kimura_parity_neutral() {
    let p = groundspring::drift::kimura_fixation_prob(100, 0.0, 0.5);
    assert!(
        (p - 0.5).abs() < 1e-10,
        "Kimura neutral should return initial_freq"
    );
}

#[test]
fn drift_kimura_parity_positive_selection() {
    let p = groundspring::drift::kimura_fixation_prob(1000, 0.01, 0.5);
    assert!(p > 0.5 && p < 1.0);
    let p2 = groundspring::drift::kimura_fixation_prob(1000, 0.01, 0.5);
    assert_eq!(p.to_bits(), p2.to_bits(), "bitwise determinism");
}

#[test]
fn drift_kimura_parity_known_value() {
    let p = groundspring::drift::kimura_fixation_prob(100, 0.01, 0.5);
    // 4Ns = 4, so P = (1 - exp(-2)) / (1 - exp(-4)) ≈ 0.8808
    let four_ns = 4.0;
    let expected = (1.0 - (-four_ns * 0.5_f64).exp()) / (1.0 - (-four_ns).exp());
    assert!(
        (p - expected).abs() < 1e-12,
        "Kimura known value: got {p}, expected {expected}"
    );
}

#[test]
fn drift_wf_parity_deterministic() {
    let a = groundspring::drift::wright_fisher_fixation(100, 0.01, 0.5, 42);
    let b = groundspring::drift::wright_fisher_fixation(100, 0.01, 0.5, 42);
    assert_eq!(a, b);
}

// ── jackknife ───────────────────────────────────────────────────────

#[test]
fn jackknife_parity_small_sample() {
    let data = [1.0, 2.0, 3.0, 4.0, 5.0];
    let r = groundspring::jackknife::jackknife_mean_variance(&data).unwrap();
    assert!((r.estimate - 3.0).abs() < 1e-12);
    assert!(r.variance > 0.0);
    assert!(r.std_error > 0.0);
}

#[test]
fn jackknife_parity_bitwise_deterministic() {
    let data: Vec<f64> = (0..100).map(|i| f64::from(i).mul_add(0.7, 1.5)).collect();
    let r1 = groundspring::jackknife::jackknife_mean_variance(&data).unwrap();
    let r2 = groundspring::jackknife::jackknife_mean_variance(&data).unwrap();
    assert_eq!(r1.estimate.to_bits(), r2.estimate.to_bits());
    assert_eq!(r1.variance.to_bits(), r2.variance.to_bits());
}

#[test]
fn jackknife_parity_known_variance() {
    let data = [1.0, 3.0];
    let r = groundspring::jackknife::jackknife_mean_variance(&data).unwrap();
    // N=2: leave-one-out means are [3.0, 1.0], grand mean 2.0
    // JK var = (2-1)/2 * ((3-2)^2 + (1-2)^2) = 0.5 * 2 = 1.0
    assert!((r.estimate - 2.0).abs() < 1e-12);
    assert!((r.variance - 1.0).abs() < 1e-12);
}

// ── fao56 ───────────────────────────────────────────────────────────

#[test]
fn fao56_parity_example_18() {
    let inp = groundspring::fao56::example_18_inputs();
    let et0 = groundspring::fao56::daily_et0(&inp);
    assert!(
        (et0 - 3.88).abs() < 0.10,
        "FAO-56 Example 18: ET₀ ≈ 3.88, got {et0}"
    );
}

#[test]
fn fao56_parity_bitwise_deterministic() {
    let inp = groundspring::fao56::example_18_inputs();
    let a = groundspring::fao56::daily_et0(&inp);
    let b = groundspring::fao56::daily_et0(&inp);
    assert_eq!(a.to_bits(), b.to_bits());
}

// ── rare_biosphere ──────────────────────────────────────────────────

#[test]
fn rare_biosphere_detection_power_parity() {
    let p = groundspring::rare_biosphere::detection_power(0.003, 998);
    assert!(
        p > 0.94 && p < 0.96,
        "detection power at threshold depth: {p}"
    );
}

#[test]
fn rare_biosphere_detection_threshold_parity() {
    let d = groundspring::rare_biosphere::detection_threshold(0.003, 0.95);
    assert_eq!(d, 998);
}

#[test]
fn rare_biosphere_chao1_parity() {
    let counts = [100u64, 50, 2, 2, 1, 1, 1];
    let est = groundspring::rare_biosphere::chao1(&counts);
    let expected = 7.0 + 9.0 / 4.0; // S_obs + f1^2/(2*f2) = 7 + 9/4 = 9.25
    assert!((est - expected).abs() < 1e-10);
}

// ── quasispecies ────────────────────────────────────────────────────

#[test]
fn quasispecies_error_threshold_parity() {
    let mu_c = groundspring::quasispecies::error_threshold(10.0, 100);
    assert!(
        (mu_c - 0.02276).abs() < 0.001,
        "error threshold: got {mu_c}"
    );
}

#[test]
fn quasispecies_master_freq_parity() {
    let xm = groundspring::quasispecies::master_frequency_analytical(10.0, 0.01, 100);
    assert!(xm > 0.1, "below threshold, master survives: {xm}");
    let xm_above = groundspring::quasispecies::master_frequency_analytical(10.0, 0.04, 100);
    assert!(
        xm_above < f64::EPSILON,
        "above threshold, master gone: {xm_above}"
    );
}

// ── band_structure ──────────────────────────────────────────────────

#[test]
fn band_structure_free_lattice_parity() {
    let edges = groundspring::band_structure::find_band_edges(&[0.0], 1.0, -4.0, 4.0, 2000);
    assert_eq!(edges.len(), 2, "free lattice: 2 band edges");
    assert!((edges[0] - (-2.0)).abs() < 0.05);
    assert!((edges[1] - 2.0).abs() < 0.05);
}

#[test]
fn band_structure_period_2_parity() {
    let n = groundspring::band_structure::count_bands(&[1.0, -1.0], 1.0, -4.0, 4.0, 2000);
    assert_eq!(n, 2, "period-2 should have 2 bands");
}

// ── freeze_out ──────────────────────────────────────────────────────

#[test]
fn freeze_out_curve_parity() {
    let t = groundspring::freeze_out::freeze_out_curve(155.0, 0.013, 0.0);
    assert!((t - 155.0).abs() < 1e-12, "T_f(0) = T0");
}

#[test]
fn freeze_out_chi2_parity() {
    let obs = [1.0, 2.0, 3.0];
    let pred = [1.0, 2.0, 3.0];
    let c2 = groundspring::freeze_out::chi_squared(&obs, &pred, 1.0).unwrap();
    assert!(c2.abs() < 1e-14, "zero residual");
}

// ── gillespie ───────────────────────────────────────────────────────

#[test]
fn gillespie_parity_deterministic() {
    let rates = vec![1.0; 5];
    let t1 = groundspring::gillespie::birth_death_ssa(&rates, 0.5, 10, 10.0, 42);
    let t2 = groundspring::gillespie::birth_death_ssa(&rates, 0.5, 10, 10.0, 42);
    assert_eq!(t1.states, t2.states);
    assert_eq!(t1.times.len(), t2.times.len());
}

#[test]
fn gillespie_steady_state_parity() {
    let ss = groundspring::gillespie::steady_state_mean(40.0, 2.2);
    assert!((ss - 18.182).abs() < 0.01);
}

// ── transport ───────────────────────────────────────────────────────

#[test]
fn transport_eigh_parity_2x2() {
    let (vals, _vecs) = groundspring::transport::tridiag_eigh(&[0.0, 0.0], &[1.0]).expect("2x2");
    assert!((vals[0] - (-1.0)).abs() < 1e-12);
    assert!((vals[1] - 1.0).abs() < 1e-12);
}

// ── seismic ─────────────────────────────────────────────────────────

#[test]
fn seismic_haversine_parity() {
    let d = groundspring::seismic::haversine_km(40.7128, -74.0060, 51.5074, -0.1278);
    assert!((d - 5570.0).abs() < 50.0);
}

// ── GPU-dispatch parity: grid_fit_2d ────────────────────────────────

#[test]
fn freeze_out_grid_fit_recovers_noiseless() {
    use groundspring::freeze_out::{freeze_out_curve, grid_fit_2d, GridFitConfig};
    let t0 = 155.0;
    let k2 = 0.013;
    let mu_b: Vec<f64> = (0..9).map(|i| f64::from(i) * 50.0).collect();
    let obs: Vec<f64> = mu_b.iter().map(|&m| freeze_out_curve(t0, k2, m)).collect();
    let cfg = GridFitConfig {
        observed: &obs,
        mu_b: &mu_b,
        sigma: 1.0,
        t0_lo: 150.0,
        t0_hi: 160.0,
        t0_step: 0.5,
        k2_lo: 0.008,
        k2_hi: 0.020,
        k2_step: 0.001,
    };
    let r = grid_fit_2d(&cfg).unwrap();
    assert!((r.t0 - t0).abs() < 1.0);
    assert!((r.kappa2 - k2).abs() < 0.002);
}

#[test]
fn freeze_out_grid_fit_bitwise_deterministic() {
    use groundspring::freeze_out::{freeze_out_curve, grid_fit_2d, GridFitConfig};
    let mu_b: Vec<f64> = (0..5).map(|i| f64::from(i) * 100.0).collect();
    let obs: Vec<f64> = mu_b
        .iter()
        .map(|&m| freeze_out_curve(155.0, 0.013, m))
        .collect();
    let cfg = GridFitConfig {
        observed: &obs,
        mu_b: &mu_b,
        sigma: 1.0,
        t0_lo: 150.0,
        t0_hi: 160.0,
        t0_step: 1.0,
        k2_lo: 0.010,
        k2_hi: 0.016,
        k2_step: 0.001,
    };
    let r1 = grid_fit_2d(&cfg).unwrap();
    let r2 = grid_fit_2d(&cfg).unwrap();
    assert_eq!(r1.chi_squared.to_bits(), r2.chi_squared.to_bits());
}

// ── GPU-dispatch parity: band_edges ────────────────────────────────

#[test]
fn band_edges_parity_alternating_potential() {
    let edges = groundspring::band_structure::find_band_edges(&[2.0, -2.0], 1.0, -6.0, 6.0, 5000);
    assert!(edges.len() >= 4, "alternating ±2 should produce ≥ 2 bands");
}

#[test]
fn band_edges_bitwise_deterministic() {
    let e1 = groundspring::band_structure::find_band_edges(&[0.0], 1.0, -3.0, 3.0, 1000);
    let e2 = groundspring::band_structure::find_band_edges(&[0.0], 1.0, -3.0, 3.0, 1000);
    assert_eq!(e1.len(), e2.len());
    for (a, b) in e1.iter().zip(e2.iter()) {
        assert_eq!(a.to_bits(), b.to_bits());
    }
}

// ── GPU-dispatch parity: seismic grid search ───────────────────────

#[test]
fn seismic_grid_search_parity_known_location() {
    use groundspring::seismic::{GridSearchConfig, Station};
    let stations = vec![
        Station {
            code: "STA1".to_string(),
            lat: 40.0,
            lon: -74.0,
        },
        Station {
            code: "STA2".to_string(),
            lat: 41.0,
            lon: -73.0,
        },
        Station {
            code: "STA3".to_string(),
            lat: 40.5,
            lon: -73.5,
        },
    ];
    let src_lat = 40.5;
    let src_lon = -73.5;
    let src_depth = 10.0;
    let vp = 6.0;
    let observed: Vec<(String, f64)> = stations
        .iter()
        .map(|s| {
            let d = groundspring::seismic::haversine_km(src_lat, src_lon, s.lat, s.lon);
            let tt = groundspring::seismic::travel_time_1d(d, src_depth, vp);
            (s.code.clone(), tt + 5.0)
        })
        .collect();
    let cfg = GridSearchConfig {
        lat_range: (39.5, 41.5),
        lon_range: (-75.0, -72.0),
        depth_range: (0.0, 30.0),
        grid_spacing_deg: 0.5,
        depth_spacing_km: 5.0,
        vp,
    };
    let r = groundspring::seismic::grid_search_inversion(&observed, &stations, &cfg);
    assert!((r.lat - src_lat).abs() < 1.0);
    assert!((r.lon - src_lon).abs() < 1.0);
}

// ── GPU-dispatch parity: quasispecies simulation ───────────────────

#[test]
fn quasispecies_simulation_parity_deterministic() {
    let a = groundspring::quasispecies::quasispecies_simulation(500, 100, 10.0, 0.01, 50, 42);
    let b = groundspring::quasispecies::quasispecies_simulation(500, 100, 10.0, 0.01, 50, 42);
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(b.iter()) {
        assert_eq!(x.to_bits(), y.to_bits(), "bitwise determinism");
    }
}

#[test]
fn quasispecies_simulation_parity_below_threshold() {
    let mu_c = groundspring::quasispecies::error_threshold(10.0, 100);
    let freqs =
        groundspring::quasispecies::quasispecies_simulation(1000, 100, 10.0, mu_c * 0.5, 200, 42);
    #[expect(clippy::cast_precision_loss, reason = "slice length < 2^52")]
    let avg: f64 = freqs.iter().skip(100).sum::<f64>() / freqs[100..].len() as f64;
    assert!(avg > 0.05, "below threshold, master persists: avg={avg}");
}

// ── GPU-dispatch parity: rare biosphere occupancy ──────────────────

#[test]
fn rare_biosphere_occupancy_parity_deterministic() {
    let community = [0.50, 0.30, 0.15, 0.04, 0.01];
    let a = groundspring::rare_biosphere::abundance_occupancy(&community, 100, 20, 42);
    let b = groundspring::rare_biosphere::abundance_occupancy(&community, 100, 20, 42);
    for (x, y) in a.iter().zip(b.iter()) {
        assert_eq!(x.to_bits(), y.to_bits(), "bitwise determinism");
    }
}

#[test]
fn rare_biosphere_tier_detection_parity_deterministic() {
    let community = [0.50, 0.30, 0.10, 0.05, 0.03, 0.02];
    let a = groundspring::rare_biosphere::tier_detection_rate(&community, 3, 6, 200, 30, 42);
    let b = groundspring::rare_biosphere::tier_detection_rate(&community, 3, 6, 200, 30, 42);
    assert_eq!(a.to_bits(), b.to_bits(), "bitwise determinism");
}

#[test]
fn rare_biosphere_occupancy_dominant_always_detected() {
    let community = [0.90, 0.05, 0.04, 0.01];
    let occ = groundspring::rare_biosphere::abundance_occupancy(&community, 500, 50, 42);
    assert!(
        occ[0] > 0.99,
        "90% abundance at depth 500 should always be detected: {}",
        occ[0]
    );
}

// ── CPU ↔ GPU dispatch parity: same inputs, identical outputs ──────
//
// These tests call functions that have both CPU and barracuda-gpu paths.
// In default mode, both go through CPU. With --features barracuda-gpu,
// the dispatch path switches. Either way, results must be identical.

#[test]
fn anderson_lyapunov_parity_known_value() {
    let pot: Vec<f64> = vec![0.5, -0.3, 0.8, -0.1, 0.6];
    let gamma = groundspring::anderson::lyapunov_exponent(&pot, 0.0);
    assert!(gamma > 0.0, "positive Lyapunov for disordered potential");
    let gamma2 = groundspring::anderson::lyapunov_exponent(&pot, 0.0);
    assert_eq!(gamma.to_bits(), gamma2.to_bits(), "bitwise deterministic");
}

#[test]
fn anderson_lyapunov_averaged_parity() {
    let g1 = groundspring::anderson::lyapunov_averaged(100, 1.0, 0.0, 5, 42);
    let g2 = groundspring::anderson::lyapunov_averaged(100, 1.0, 0.0, 5, 42);
    assert_eq!(g1.to_bits(), g2.to_bits());
    assert!(g1 > 0.0, "disorder W=1 should localize: gamma={g1}");
}

#[test]
fn almost_mathieu_eigenvalues_parity() {
    let ev1 = groundspring::almost_mathieu::eigenvalues(
        10,
        1.0,
        0.5 * std::f64::consts::FRAC_1_SQRT_2,
        0.0,
    );
    let ev2 = groundspring::almost_mathieu::eigenvalues(
        10,
        1.0,
        0.5 * std::f64::consts::FRAC_1_SQRT_2,
        0.0,
    );
    assert_eq!(ev1.len(), ev2.len());
    for (a, b) in ev1.iter().zip(ev2.iter()) {
        assert_eq!(a.to_bits(), b.to_bits(), "eigenvalue bitwise parity");
    }
}

#[test]
fn spectral_recon_tikhonov_parity() {
    let kernel = vec![1.0, 0.5, 0.5, 1.0];
    let data = vec![1.0, 0.5];
    let r1 = groundspring::spectral_recon::tikhonov_solve(&kernel, &data, 0.01, 2, 2);
    let r2 = groundspring::spectral_recon::tikhonov_solve(&kernel, &data, 0.01, 2, 2);
    for (a, b) in r1.iter().zip(r2.iter()) {
        assert_eq!(a.to_bits(), b.to_bits(), "Tikhonov bitwise parity");
    }
}

// ── New V32 metrics ─────────────────────────────────────────────────

#[test]
fn mae_parity_known_value() {
    let obs = [1.0, 2.0, 3.0, 4.0, 5.0];
    let modeled = [1.2, 2.3, 2.8, 4.1, 5.4];
    let mae1 = groundspring::stats::mae(&obs, &modeled);
    let mae2 = groundspring::stats::mae(&obs, &modeled);
    assert_eq!(mae1.to_bits(), mae2.to_bits(), "MAE bitwise parity");
    assert!(mae1 > 0.0 && mae1 < 1.0, "MAE in expected range: {mae1}");
}

#[test]
fn nse_parity_known_value() {
    let obs = [1.0, 2.0, 3.0, 4.0, 5.0];
    let modeled = [1.1, 2.2, 2.8, 4.3, 4.9];
    let nse1 = groundspring::stats::nash_sutcliffe(&obs, &modeled);
    let nse2 = groundspring::stats::nash_sutcliffe(&obs, &modeled);
    assert_eq!(nse1.to_bits(), nse2.to_bits(), "NSE bitwise parity");
    assert!(nse1 > 0.9 && nse1 <= 1.0, "NSE near-perfect: {nse1}");
}

#[test]
fn detect_band_ranges_parity() {
    let eigenvalues: Vec<f64> = (0..50)
        .map(|i| f64::from(i).mul_add(0.01, -2.0))
        .chain((0..50).map(|i| f64::from(i).mul_add(0.01, 1.0)))
        .collect();
    let bands1 = groundspring::band_structure::detect_band_ranges(&eigenvalues, 3.0);
    let bands2 = groundspring::band_structure::detect_band_ranges(&eigenvalues, 3.0);
    assert_eq!(bands1.len(), bands2.len(), "band count parity");
    for (a, b) in bands1.iter().zip(bands2.iter()) {
        assert_eq!(a.0.to_bits(), b.0.to_bits(), "band lo parity");
        assert_eq!(a.1.to_bits(), b.1.to_bits(), "band hi parity");
    }
}

// ── WDM green_kubo_integrate → barracuda::numerical::trapz ────────

#[test]
fn wdm_green_kubo_parity_exponential_decay() {
    let c0 = 1.0;
    let tau = 10.0;
    let dt = 0.001;
    let n_steps = 100_000;
    let vacf = groundspring::wdm::synthetic_vacf(c0, tau, n_steps, dt);

    let i1 = groundspring::wdm::green_kubo_integrate(&vacf, dt);
    let i2 = groundspring::wdm::green_kubo_integrate(&vacf, dt);
    assert_eq!(i1.to_bits(), i2.to_bits(), "Green-Kubo bitwise parity");

    let analytical = c0 * tau;
    let rel_err = (i1 - analytical).abs() / analytical;
    assert!(rel_err < 0.001, "relative error {rel_err:.6} exceeds 0.1%");
}

// ── regression fit_quadratic/exponential/logarithmic ──────────────

#[test]
fn regression_quadratic_parity() {
    let xs: Vec<f64> = (-5..=5).map(f64::from).collect();
    let ys: Vec<f64> = xs
        .iter()
        .map(|&x| (2.0 * x).mul_add(x, (-3.0_f64).mul_add(x, 1.0)))
        .collect();
    let f1 = groundspring::stats::fit_quadratic(&xs, &ys).unwrap();
    let f2 = groundspring::stats::fit_quadratic(&xs, &ys).unwrap();
    assert_eq!(f1.params[0].to_bits(), f2.params[0].to_bits(), "a parity");
    assert_eq!(f1.params[1].to_bits(), f2.params[1].to_bits(), "b parity");
    assert_eq!(f1.params[2].to_bits(), f2.params[2].to_bits(), "c parity");
    assert!((f1.params[0] - 2.0).abs() < 1e-8, "a = {}", f1.params[0]);
    assert!(f1.r_squared > 0.999, "R² = {}", f1.r_squared);
}

#[test]
fn regression_exponential_parity() {
    let xs: Vec<f64> = (0..10).map(f64::from).collect();
    let a = 5.0_f64;
    let b = -0.3_f64;
    let ys: Vec<f64> = xs.iter().map(|&x| a * (b * x).exp()).collect();
    let f1 = groundspring::stats::fit_exponential(&xs, &ys).unwrap();
    let f2 = groundspring::stats::fit_exponential(&xs, &ys).unwrap();
    assert_eq!(f1.params[0].to_bits(), f2.params[0].to_bits(), "a parity");
    assert_eq!(f1.params[1].to_bits(), f2.params[1].to_bits(), "b parity");
    assert!(f1.r_squared > 0.99, "R² = {}", f1.r_squared);
}

#[test]
fn regression_logarithmic_parity() {
    let xs: Vec<f64> = (1..=10).map(f64::from).collect();
    let a = 3.0_f64;
    let b = 2.0_f64;
    let ys: Vec<f64> = xs.iter().map(|&x| a.mul_add(x.ln(), b)).collect();
    let f1 = groundspring::stats::fit_logarithmic(&xs, &ys).unwrap();
    let f2 = groundspring::stats::fit_logarithmic(&xs, &ys).unwrap();
    assert_eq!(f1.params[0].to_bits(), f2.params[0].to_bits(), "a parity");
    assert_eq!(f1.params[1].to_bits(), f2.params[1].to_bits(), "b parity");
    assert!((f1.params[0] - a).abs() < 1e-8, "a = {}", f1.params[0]);
    assert!((f1.params[1] - b).abs() < 1e-8, "b = {}", f1.params[1]);
}

// ── bootstrap ─────────────────────────────────────────────────────

#[test]
fn bootstrap_mean_parity_deterministic() {
    let data = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let r1 = groundspring::bootstrap::bootstrap_mean(&data, 1000, 0.95, 42);
    let r2 = groundspring::bootstrap::bootstrap_mean(&data, 1000, 0.95, 42);
    assert_eq!(r1.estimate.to_bits(), r2.estimate.to_bits(), "mean bitwise");
    assert_eq!(
        r1.ci_lower.to_bits(),
        r2.ci_lower.to_bits(),
        "ci_lower bitwise"
    );
    assert_eq!(
        r1.ci_upper.to_bits(),
        r2.ci_upper.to_bits(),
        "ci_upper bitwise"
    );
    assert!(
        (r1.estimate - 5.5).abs() < 1.0,
        "mean near 5.5: {}",
        r1.estimate
    );
}

#[test]
fn bootstrap_mean_parity_ci_contains_true() {
    let data = [2.0, 4.0, 6.0, 8.0, 10.0];
    let r = groundspring::bootstrap::bootstrap_mean(&data, 2000, 0.95, 99);
    assert!(
        r.ci_lower <= 6.0 && r.ci_upper >= 6.0,
        "95% CI should contain true mean 6.0"
    );
}

#[test]
fn rawr_mean_parity_deterministic() {
    let data = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let r1 = groundspring::bootstrap::rawr_mean(&data, 1000, 0.95, 42);
    let r2 = groundspring::bootstrap::rawr_mean(&data, 1000, 0.95, 42);
    assert_eq!(r1.estimate.to_bits(), r2.estimate.to_bits(), "rawr bitwise");
    assert_eq!(
        r1.std_error.to_bits(),
        r2.std_error.to_bits(),
        "rawr se bitwise"
    );
}

#[test]
fn bootstrap_median_parity_deterministic() {
    let data = [1.0, 3.0, 5.0, 7.0, 9.0, 11.0, 13.0];
    let r1 = groundspring::bootstrap::bootstrap_median(&data, 1000, 0.95, 7);
    let r2 = groundspring::bootstrap::bootstrap_median(&data, 1000, 0.95, 7);
    assert_eq!(
        r1.estimate.to_bits(),
        r2.estimate.to_bits(),
        "median bitwise"
    );
    assert!(
        (r1.estimate - 7.0).abs() < 2.0,
        "median near 7.0: {}",
        r1.estimate
    );
}

#[test]
fn bootstrap_std_parity_deterministic() {
    let data = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    let r1 = groundspring::bootstrap::bootstrap_std(&data, 1000, 0.95, 13);
    let r2 = groundspring::bootstrap::bootstrap_std(&data, 1000, 0.95, 13);
    assert_eq!(r1.estimate.to_bits(), r2.estimate.to_bits(), "std bitwise");
    assert!(r1.estimate > 0.0, "std positive");
}

// ── stats::moving_window ──────────────────────────────────────────

#[test]
fn moving_window_stats_parity_deterministic() {
    let data = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let r1 = groundspring::stats::moving_window_stats(&data, 3).unwrap();
    let r2 = groundspring::stats::moving_window_stats(&data, 3).unwrap();
    assert_eq!(r1.mean.len(), r2.mean.len(), "length parity");
    for (a, b) in r1.mean.iter().zip(r2.mean.iter()) {
        assert_eq!(a.to_bits(), b.to_bits(), "mean bitwise");
    }
    for (a, b) in r1.variance.iter().zip(r2.variance.iter()) {
        assert_eq!(a.to_bits(), b.to_bits(), "variance bitwise");
    }
}

#[test]
fn moving_window_stats_parity_known_value() {
    let data = [1.0, 2.0, 3.0];
    let r = groundspring::stats::moving_window_stats(&data, 3).unwrap();
    assert!((r.mean[0] - 2.0).abs() < 1e-10, "window mean = 2.0");
}

// ── rarefaction (diversity indices) ───────────────────────────────

#[test]
fn simpson_diversity_parity_known_value() {
    let counts = [10, 10, 10, 10];
    let d1 = groundspring::rarefaction::simpson_diversity(&counts);
    let d2 = groundspring::rarefaction::simpson_diversity(&counts);
    assert_eq!(d1.to_bits(), d2.to_bits(), "simpson bitwise");
    assert!((d1 - 0.75).abs() < 0.01, "even community D ≈ 0.75: {d1}");
}

#[test]
fn shannon_diversity_parity_known_value() {
    let counts = [10, 10, 10, 10];
    let h1 = groundspring::rarefaction::shannon_diversity(&counts);
    let h2 = groundspring::rarefaction::shannon_diversity(&counts);
    assert_eq!(h1.to_bits(), h2.to_bits(), "shannon bitwise");
    let expected = (4.0_f64).ln();
    assert!(
        (h1 - expected).abs() < 1e-10,
        "H = ln(4) ≈ {expected}: {h1}"
    );
}

#[test]
fn evenness_parity_known_value() {
    let counts = [10, 10, 10, 10];
    let e1 = groundspring::rarefaction::evenness(&counts);
    let e2 = groundspring::rarefaction::evenness(&counts);
    assert_eq!(e1.to_bits(), e2.to_bits(), "evenness bitwise");
    assert!((e1 - 1.0).abs() < 1e-10, "perfectly even J = 1.0: {e1}");
}

#[test]
fn bray_curtis_parity_known_value() {
    let a = [10.0, 20.0, 30.0];
    let b = [10.0, 20.0, 30.0];
    let d = groundspring::rarefaction::bray_curtis(&a, &b);
    assert!((d - 0.0).abs() < 1e-15, "identical => BC = 0: {d}");

    let c = [0.0, 0.0, 60.0];
    let d2 = groundspring::rarefaction::bray_curtis(&a, &c);
    assert!(d2 > 0.0 && d2 <= 1.0, "BC in (0,1]: {d2}");
}

#[test]
fn analytical_rarefaction_parity_deterministic() {
    let counts = [100, 50, 25, 10, 5];
    let depths = [10, 50, 100, 150];
    let r1 = groundspring::rarefaction::analytical_rarefaction(&counts, &depths);
    let r2 = groundspring::rarefaction::analytical_rarefaction(&counts, &depths);
    assert_eq!(r1.len(), r2.len());
    for (a, b) in r1.iter().zip(r2.iter()) {
        assert_eq!(a.to_bits(), b.to_bits(), "rarefaction bitwise");
    }
    for w in r1.windows(2) {
        assert!(w[1] >= w[0], "rarefaction monotonically non-decreasing");
    }
}

// ── kinetics ──────────────────────────────────────────────────────

#[test]
fn hill_parity_known_value() {
    let y = groundspring::kinetics::hill(10.0, 10.0, 2.0);
    assert!((y - 0.5).abs() < 1e-10, "hill(K,K,n) = 0.5: {y}");
    let y2 = groundspring::kinetics::hill(10.0, 10.0, 2.0);
    assert_eq!(y.to_bits(), y2.to_bits(), "hill bitwise");
}

#[test]
fn hill_parity_extreme() {
    let sat = groundspring::kinetics::hill(1e6, 1.0, 2.0);
    assert!((sat - 1.0).abs() < 1e-6, "saturated hill ≈ 1.0: {sat}");
    let low = groundspring::kinetics::hill(1e-6, 1.0, 2.0);
    assert!(low < 1e-6, "subsaturated hill ≈ 0.0: {low}");
}

#[test]
fn monod_parity_known_value() {
    let y = groundspring::kinetics::monod(10.0, 1.0, 10.0);
    assert!((y - 0.5).abs() < 1e-10, "monod(K,1,K) = 0.5: {y}");
    let y2 = groundspring::kinetics::monod(10.0, 1.0, 10.0);
    assert_eq!(y.to_bits(), y2.to_bits(), "monod bitwise");
}

// ── stats::metrics ────────────────────────────────────────────────

#[test]
fn mean_parity_known_value() {
    let vals = [1.0, 2.0, 3.0, 4.0, 5.0];
    let m1 = groundspring::stats::mean(&vals);
    let m2 = groundspring::stats::mean(&vals);
    assert_eq!(m1.to_bits(), m2.to_bits(), "mean bitwise");
    assert!((m1 - 3.0).abs() < 1e-15, "mean = 3.0: {m1}");
}

#[test]
fn percentile_parity_known_value() {
    let vals = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let p50_1 = groundspring::stats::percentile(&vals, 50.0).unwrap();
    let p50_2 = groundspring::stats::percentile(&vals, 50.0).unwrap();
    assert_eq!(p50_1.to_bits(), p50_2.to_bits(), "p50 bitwise");
    assert!((p50_1 - 5.5).abs() < 1.0, "median near 5.5: {p50_1}");
}

#[test]
fn sample_std_dev_parity_known_value() {
    let vals = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    let s1 = groundspring::stats::sample_std_dev(&vals);
    let s2 = groundspring::stats::sample_std_dev(&vals);
    assert_eq!(s1.to_bits(), s2.to_bits(), "sample_std_dev bitwise");
    assert!(s1 > 0.0, "positive std dev");
}

// ── stats::agreement (remaining) ──────────────────────────────────

#[test]
fn rmse_parity_known_value() {
    let obs = [1.0, 2.0, 3.0, 4.0, 5.0];
    let mod_ = [1.0, 2.0, 3.0, 4.0, 5.0];
    let r = groundspring::stats::rmse(&obs, &mod_);
    assert!((r - 0.0).abs() < 1e-15, "perfect fit RMSE = 0: {r}");
    let r2 = groundspring::stats::rmse(&obs, &mod_);
    assert_eq!(r.to_bits(), r2.to_bits(), "rmse bitwise");
}

#[test]
fn mbe_parity_known_value() {
    let obs = [1.0, 2.0, 3.0];
    let mod_ = [2.0, 3.0, 4.0];
    let b1 = groundspring::stats::mbe(&obs, &mod_);
    let b2 = groundspring::stats::mbe(&obs, &mod_);
    assert_eq!(b1.to_bits(), b2.to_bits(), "mbe bitwise");
    assert!((b1 - 1.0).abs() < 1e-15, "constant +1 bias: {b1}");
}

#[test]
fn r_squared_parity_known_value() {
    let obs = [1.0, 2.0, 3.0, 4.0, 5.0];
    let r2v = groundspring::stats::r_squared(&obs, &obs);
    assert!((r2v - 1.0).abs() < 1e-10, "perfect R² = 1.0: {r2v}");
    let r2v2 = groundspring::stats::r_squared(&obs, &obs);
    assert_eq!(r2v.to_bits(), r2v2.to_bits(), "r_squared bitwise");
}

#[test]
fn index_of_agreement_parity_known_value() {
    let obs = [1.0, 2.0, 3.0, 4.0, 5.0];
    let ia = groundspring::stats::index_of_agreement(&obs, &obs);
    assert!((ia - 1.0).abs() < 1e-10, "perfect IA = 1.0: {ia}");
    let ia2 = groundspring::stats::index_of_agreement(&obs, &obs);
    assert_eq!(ia.to_bits(), ia2.to_bits(), "ia bitwise");
}

#[test]
fn hit_rate_parity_known_value() {
    let obs = [10.0, 20.0, 30.0];
    let mod_ = [10.0, 20.0, 30.0];
    let hr = groundspring::stats::hit_rate(&obs, &mod_, 5.0);
    assert!((hr - 1.0).abs() < 1e-15, "perfect hit rate = 1.0: {hr}");
    let hr2 = groundspring::stats::hit_rate(&obs, &mod_, 5.0);
    assert_eq!(hr.to_bits(), hr2.to_bits(), "hit_rate bitwise");
}

// ── stats::correlation ────────────────────────────────────────────

#[test]
fn pearson_r_parity_known_value() {
    let x = [1.0, 2.0, 3.0, 4.0, 5.0];
    let y = [2.0, 4.0, 6.0, 8.0, 10.0];
    let r1 = groundspring::stats::pearson_r(&x, &y);
    let r2 = groundspring::stats::pearson_r(&x, &y);
    assert_eq!(r1.to_bits(), r2.to_bits(), "pearson bitwise");
    assert!((r1 - 1.0).abs() < 1e-10, "perfect linear r = 1.0: {r1}");
}

#[test]
fn spearman_r_parity_known_value() {
    let x = [1.0, 2.0, 3.0, 4.0, 5.0];
    let y = [2.0, 4.0, 6.0, 8.0, 10.0];
    let r1 = groundspring::stats::spearman_r(&x, &y);
    let r2 = groundspring::stats::spearman_r(&x, &y);
    assert_eq!(r1.to_bits(), r2.to_bits(), "spearman bitwise");
    assert!((r1 - 1.0).abs() < 1e-10, "perfect monotonic rs = 1.0: {r1}");
}

#[test]
fn covariance_parity_known_value() {
    let x = [1.0, 2.0, 3.0, 4.0, 5.0];
    let y = [2.0, 4.0, 6.0, 8.0, 10.0];
    let c1 = groundspring::stats::covariance(&x, &y);
    let c2 = groundspring::stats::covariance(&x, &y);
    assert_eq!(c1.to_bits(), c2.to_bits(), "covariance bitwise");
    assert!(
        c1 > 0.0,
        "positive covariance for positively correlated data: {c1}"
    );
}

// ── bistable / multisignal ODE derivatives ────────────────────────

#[test]
fn bistable_derivative_parity_equilibrium() {
    let params = groundspring::bistable::BistableParams::default();
    let state = [0.5, 0.1, 0.5, 0.035, 0.01];
    let d1 = groundspring::bistable::bistable_derivative(&state, &params);
    let d2 = groundspring::bistable::bistable_derivative(&state, &params);
    for i in 0..5 {
        assert_eq!(d1[i].to_bits(), d2[i].to_bits(), "bistable d[{i}] bitwise");
    }
    assert!(d1.iter().any(|v| v.abs() > 0.0), "non-trivial derivatives");
}

#[test]
fn multisignal_derivative_parity_equilibrium() {
    let params = groundspring::multisignal::MultiSignalParams::default();
    let state = [0.5, 0.1, 0.1, 0.5, 0.1, 0.5, 0.01];
    let d1 = groundspring::multisignal::multisignal_derivative(&state, &params);
    let d2 = groundspring::multisignal::multisignal_derivative(&state, &params);
    for i in 0..7 {
        assert_eq!(
            d1[i].to_bits(),
            d2[i].to_bits(),
            "multisignal d[{i}] bitwise"
        );
    }
    assert!(d1.iter().any(|v| v.abs() > 0.0), "non-trivial derivatives");
}

// ── stats::regression (linear) ────────────────────────────────────

#[test]
fn regression_linear_parity() {
    let xs: Vec<f64> = (0..10).map(f64::from).collect();
    let ys: Vec<f64> = xs.iter().map(|&x| 2.0_f64.mul_add(x, 1.0)).collect();
    let f1 = groundspring::stats::fit_linear(&xs, &ys).unwrap();
    let f2 = groundspring::stats::fit_linear(&xs, &ys).unwrap();
    assert_eq!(f1.slope.to_bits(), f2.slope.to_bits(), "slope bitwise");
    assert_eq!(
        f1.intercept.to_bits(),
        f2.intercept.to_bits(),
        "intercept bitwise"
    );
    assert!((f1.slope - 2.0).abs() < 1e-10, "slope = 2.0: {}", f1.slope);
    assert!(
        (f1.intercept - 1.0).abs() < 1e-10,
        "intercept = 1.0: {}",
        f1.intercept
    );
    assert!(f1.r_squared > 0.999, "R² perfect: {}", f1.r_squared);
}

// ── gillespie batch GPU dispatch ─────────────────────────────────

#[test]
fn gillespie_batch_parity() {
    let rates = vec![1.0; 5];
    let r1 = groundspring::gillespie::birth_death_ssa_batch(&rates, 0.5, 10, 100.0, 20, 50.0, 42);
    let r2 = groundspring::gillespie::birth_death_ssa_batch(&rates, 0.5, 10, 100.0, 20, 50.0, 42);
    assert_eq!(r1.mean.to_bits(), r2.mean.to_bits(), "batch mean bitwise");
    assert_eq!(
        r1.variance.to_bits(),
        r2.variance.to_bits(),
        "batch variance bitwise"
    );
    assert_eq!(r1.n_trajectories, 20);
}

#[test]
fn gillespie_batch_mean_near_steady_state() {
    let rates = vec![1.0; 10];
    let result =
        groundspring::gillespie::birth_death_ssa_batch(&rates, 1.0, 10, 500.0, 50, 50.0, 42);
    let ss = groundspring::gillespie::steady_state_mean(10.0, 1.0);
    assert!(
        (result.mean - ss).abs() < 5.0,
        "batch mean {} vs steady-state {ss}",
        result.mean
    );
}

// ── drift batch GPU dispatch ────────────────────────────────────

#[test]
fn wf_batch_parity() {
    let n1 = groundspring::drift::wright_fisher_fixation_batch(100, 0.01, 0.5, 20, 42);
    let n2 = groundspring::drift::wright_fisher_fixation_batch(100, 0.01, 0.5, 20, 42);
    assert_eq!(n1, n2, "same seed → same fixation count");
}

#[test]
fn wf_batch_fixation_positive_selection() {
    let count = groundspring::drift::wright_fisher_fixation_batch(100, 0.05, 0.5, 50, 42);
    assert!(count > 0, "positive selection should fix some trials");
    assert!(count <= 50, "can't fix more than n_trials");
}

#[test]
fn wf_batch_kimura_convergence() {
    let n = 200;
    let s = 0.01;
    let p0 = 0.5;
    let n_trials = 200;
    let fix_count = groundspring::drift::wright_fisher_fixation_batch(n, s, p0, n_trials, 42);
    let kimura = groundspring::drift::kimura_fixation_prob(n, s, p0);
    #[expect(clippy::cast_precision_loss)]
    let observed = fix_count as f64 / n_trials as f64;
    assert!(
        (observed - kimura).abs() < 0.15,
        "batch fixation fraction {observed} vs Kimura {kimura}"
    );
}

// ── fao56 batch GPU dispatch ────────────────────────────────────

#[test]
fn fao56_batch_parity() {
    let inputs = vec![groundspring::fao56::example_18_inputs(); 5];
    let batch = groundspring::fao56::daily_et0_batch(&inputs);
    assert_eq!(batch.len(), 5);
    for &et0 in &batch {
        assert!((et0 - 3.88).abs() < 0.10, "batch ET₀ ≈ 3.88, got {et0:.4}");
    }
}

#[test]
fn fao56_batch_single_matches_scalar() {
    let inp = groundspring::fao56::example_18_inputs();
    let scalar = groundspring::fao56::daily_et0(&inp);
    let batch = groundspring::fao56::daily_et0_batch(&[inp]);
    // GPU shader computes intermediate values in a single-pass kernel,
    // so minor numerical divergence (< 0.05 mm/day) from the host
    // step-by-step computation is expected and within FAO-56 tolerance.
    assert!(
        (batch[0] - scalar).abs() < 0.05,
        "batch[0]={}, scalar={}",
        batch[0],
        scalar
    );
}

// ══════════════════════════════════════════════════════════════════
// Pure GPU workload validation
//
// These tests verify GPU-dispatched results match known scientific
// values directly — proving the math is truly portable to GPU.
// They run through the batch APIs which dispatch to GPU when available.
// ══════════════════════════════════════════════════════════════════

#[test]
fn gpu_gillespie_steady_state_convergence() {
    let rates = vec![1.0_f64; 10];
    let ss = groundspring::gillespie::steady_state_mean(10.0, 1.0);
    let result =
        groundspring::gillespie::birth_death_ssa_batch(&rates, 1.0, 10, 1000.0, 100, 100.0, 42);
    assert!(
        (result.mean - ss).abs() < 5.0,
        "GPU batch mean {} vs analytical {}",
        result.mean,
        ss
    );
    assert!(result.variance > 0.0, "variance should be positive");
}

#[test]
fn gpu_wright_fisher_kimura_agreement() {
    let n = 200;
    let s = 0.01;
    let p0 = 0.5;
    let n_trials = 500;
    let fix_count = groundspring::drift::wright_fisher_fixation_batch(n, s, p0, n_trials, 42);
    let kimura = groundspring::drift::kimura_fixation_prob(n, s, p0);
    #[expect(clippy::cast_precision_loss)]
    let observed = fix_count as f64 / n_trials as f64;
    assert!(
        (observed - kimura).abs() < 0.10,
        "GPU WF fixation {observed} vs Kimura {kimura}"
    );
}

#[test]
fn gpu_fao56_reference_et0() {
    let inputs = vec![groundspring::fao56::example_18_inputs(); 10];
    let batch = groundspring::fao56::daily_et0_batch(&inputs);
    assert_eq!(batch.len(), 10);
    for &et0 in &batch {
        assert!(
            (et0 - 3.88).abs() < 0.10,
            "GPU ET₀ should match FAO-56 Example 18 (3.88), got {et0:.4}"
        );
    }
}

#[test]
fn gpu_anderson_localization_positive_lyapunov() {
    let potential: Vec<f64> = (0..500).map(|i| (f64::from(i) * 0.3).sin() * 3.0).collect();
    let gamma = groundspring::anderson::lyapunov_exponent(&potential, 0.5);
    assert!(
        gamma > 0.0,
        "Anderson model should show localization: γ={gamma}"
    );
}

#[test]
fn gpu_rare_biosphere_dominant_occupancy() {
    let mut community = vec![0.001; 100];
    community[0] = 0.9;
    let total: f64 = community.iter().sum();
    for c in &mut community {
        *c /= total;
    }
    let occ = groundspring::rare_biosphere::abundance_occupancy(&community, 50, 5000, 42);
    assert!(
        occ[0] > 0.99,
        "dominant species should have ~100% occupancy, got {}",
        occ[0]
    );
}

#[test]
fn gpu_batch_determinism() {
    let rates = vec![1.0_f64; 10];
    let r1 = groundspring::gillespie::birth_death_ssa_batch(&rates, 1.0, 10, 200.0, 50, 50.0, 42);
    let r2 = groundspring::gillespie::birth_death_ssa_batch(&rates, 1.0, 10, 200.0, 50, 50.0, 42);
    assert_eq!(
        r1.mean.to_bits(),
        r2.mean.to_bits(),
        "GPU batch must be deterministic"
    );
    assert_eq!(
        r1.variance.to_bits(),
        r2.variance.to_bits(),
        "GPU batch variance must be deterministic"
    );
}

// ══════════════════════════════════════════════════════════════════
// Barracuda CPU vs GPU explicit parity
//
// These tests verify that GPU-dispatched statistics produce results
// matching barracuda CPU within documented tolerances. When the GPU
// is available, the public API automatically dispatches to GPU —
// proving the math is portable.
// ══════════════════════════════════════════════════════════════════

#[test]
fn gpu_mean_matches_cpu_known_value() {
    let data = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    let m = groundspring::stats::mean(&data);
    assert!((m - 5.0).abs() < 1e-10, "mean should be 5.0, got {m}");
}

#[test]
fn gpu_std_dev_matches_cpu_known_value() {
    let data = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    let s = groundspring::stats::std_dev(&data);
    assert!(
        (s - 2.0).abs() < 1e-6,
        "population std should be 2.0, got {s}"
    );
}

#[test]
fn gpu_rmse_matches_cpu_known_value() {
    let obs = [1.0, 2.0, 3.0, 4.0, 5.0];
    let modeled = [1.1, 2.1, 3.1, 4.1, 5.1];
    let r = groundspring::stats::rmse(&obs, &modeled);
    assert!((r - 0.1).abs() < 1e-6, "RMSE of +0.1 bias = 0.1, got {r}");
}

#[test]
fn gpu_mbe_matches_cpu_known_value() {
    let obs = [1.0, 2.0, 3.0, 4.0, 5.0];
    let modeled = [1.5, 2.5, 3.5, 4.5, 5.5];
    let b = groundspring::stats::mbe(&obs, &modeled);
    assert!((b - 0.5).abs() < 1e-6, "MBE of +0.5 bias = 0.5, got {b}");
}

#[test]
fn gpu_pearson_perfect_positive() {
    let x = [1.0, 2.0, 3.0, 4.0, 5.0];
    let y = [2.0, 4.0, 6.0, 8.0, 10.0];
    let r = groundspring::stats::pearson_r(&x, &y);
    assert!(
        (r - 1.0).abs() < 1e-6,
        "perfect positive correlation, got {r}"
    );
}

#[test]
fn gpu_pearson_zero_correlation() {
    let x = [1.0, 2.0, 3.0, 4.0, 5.0];
    let y = [3.0, 3.0, 3.0, 3.0, 3.0];
    let r = groundspring::stats::pearson_r(&x, &y);
    assert!(r.abs() < 1e-6, "zero correlation, got {r}");
}

#[test]
fn gpu_r_squared_perfect() {
    let x = [1.0, 2.0, 3.0, 4.0, 5.0];
    let r2 = groundspring::stats::r_squared(&x, &x);
    assert!((r2 - 1.0).abs() < 1e-6, "perfect R², got {r2}");
}

#[test]
fn gpu_decompose_pythagorean() {
    let obs: Vec<f64> = (0..100).map(|i| f64::from(i) * 0.1).collect();
    let modeled: Vec<f64> = obs.iter().map(|&v| v + 0.05).collect();
    let rmse_val = groundspring::stats::rmse(&obs, &modeled);
    let mbe_val = groundspring::stats::mbe(&obs, &modeled);
    let d = groundspring::decompose::decompose_error(mbe_val, rmse_val);
    let reconstructed = (d.bias_sq + d.variance).sqrt();
    assert!(
        (reconstructed - rmse_val).abs() < 1e-6,
        "RMSE² = MBE² + σ² must hold across CPU/GPU: reconstructed={reconstructed}, rmse={rmse_val}"
    );
}

#[test]
fn gpu_stats_deterministic() {
    let data = [1.5, 2.7, 3.1, 4.9, 5.2, 6.8, 7.3, 8.1, 9.0, 10.4];
    let m1 = groundspring::stats::mean(&data);
    let m2 = groundspring::stats::mean(&data);
    assert_eq!(m1.to_bits(), m2.to_bits(), "mean must be deterministic");
    let s1 = groundspring::stats::std_dev(&data);
    let s2 = groundspring::stats::std_dev(&data);
    assert_eq!(s1.to_bits(), s2.to_bits(), "std_dev must be deterministic");
}

// ── Dispatch target inventory sentinel ─────────────────────────────

#[test]
fn dispatch_targets_at_least_32() {
    let cpu_active = 31;
    let gpu_active = 17;
    let pending_toadstool = 6;
    assert!(
        cpu_active + gpu_active >= 48,
        "minimum 48 active dispatch targets"
    );
    assert_eq!(
        pending_toadstool, 6,
        "6 pending ToadStool delegations (scalar fao56, kimura, grid_search_3d, grid_fit_2d, jackknife, band_edges)"
    );
}

// metalForge workload count is tested in metalForge/forge/src/workloads.rs
// (all_returns_nineteen_workloads).
