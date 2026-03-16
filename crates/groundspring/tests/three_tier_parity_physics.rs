// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Three-tier parity tests — physics and numerical primitives.
//!
//! Validates that Anderson localization, Almost-Mathieu, band structure,
//! transport, seismic, freeze-out, spectral reconstruction, WDM,
//! and FAO-56 functions produce identical results regardless of
//! feature mode (default / barracuda / barracuda-gpu).

use groundspring::tol;

// ── anderson ───────────────────────────────────────────────────────

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

// ── almost_mathieu ─────────────────────────────────────────────────

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

// ── band_structure ──────────────────────────────────────────────────

#[test]
fn band_structure_free_lattice_parity() {
    let edges = groundspring::band_structure::find_band_edges(&[0.0], 1.0, -4.0, 4.0, 2000);
    assert_eq!(edges.len(), 2, "free lattice: 2 band edges");
    // 0.05: Brent bisection on a 2000-point grid; Δε = 8/2000 = 0.004,
    // but edge detection uses sign changes in the transfer-matrix trace
    // which round to nearest grid point. 0.05 ≈ 12× grid spacing.
    assert!((edges[0] - (-2.0)).abs() < 0.05);
    assert!((edges[1] - 2.0).abs() < 0.05);
}

#[test]
fn band_structure_period_2_parity() {
    let n = groundspring::band_structure::count_bands(&[1.0, -1.0], 1.0, -4.0, 4.0, 2000);
    assert_eq!(n, 2, "period-2 should have 2 bands");
}

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

// ── transport ───────────────────────────────────────────────────────

#[test]
fn transport_eigh_parity_2x2() {
    let (vals, _vecs) = groundspring::transport::tridiag_eigh(&[0.0, 0.0], &[1.0]).expect("2x2");
    assert!((vals[0] - (-1.0)).abs() < tol::EXACT);
    assert!((vals[1] - 1.0).abs() < tol::EXACT);
}

// ── seismic ─────────────────────────────────────────────────────────

#[test]
fn seismic_haversine_parity() {
    // NYC (40.7128°N, 74.0060°W) → London (51.5074°N, 0.1278°W) ≈ 5570 km.
    // Reference: great-circle distance via WGS-84 ellipsoid ≈ 5570 km.
    // Tolerance 50 km: haversine uses spherical Earth (R = 6371 km).
    let d = groundspring::seismic::haversine_km(40.7128, -74.0060, 51.5074, -0.1278);
    assert!((d - 5570.0).abs() < 50.0);
}

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

// ── freeze_out ──────────────────────────────────────────────────────

#[test]
fn freeze_out_curve_parity() {
    let t = groundspring::freeze_out::freeze_out_curve(155.0, 0.013, 0.0);
    assert!((t - 155.0).abs() < tol::EXACT, "T_f(0) = T0");
}

#[test]
fn freeze_out_chi2_parity() {
    let obs = [1.0, 2.0, 3.0];
    let pred = [1.0, 2.0, 3.0];
    let c2 = groundspring::freeze_out::chi_squared(&obs, &pred, 1.0).unwrap();
    assert!(c2.abs() < tol::STRICT, "zero residual");
}

#[test]
fn freeze_out_grid_fit_recovers_noiseless() {
    use groundspring::freeze_out::{GridFitConfig, freeze_out_curve, grid_fit_2d};
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
    use groundspring::freeze_out::{GridFitConfig, freeze_out_curve, grid_fit_2d};
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

// ── spectral_recon ──────────────────────────────────────────────────

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
    assert!(
        rel_err < tol::LITERATURE,
        "relative error {rel_err:.6} exceeds 0.1%"
    );
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

// ── anderson disorder_sweep (S59 cross-spring) ──────────────────────

#[test]
fn anderson_disorder_sweep_parity() {
    let sweep = groundspring::anderson::disorder_sweep(500, 1.0, 3.0, 3, 5, 42);
    assert_eq!(sweep.len(), 3);
    assert!(sweep[0].disorder < sweep[2].disorder);
    for p in &sweep {
        assert!(
            p.mean_ratio > 0.0,
            "Lyapunov should be positive: {}",
            p.mean_ratio
        );
    }
}

#[test]
fn anderson_disorder_sweep_deterministic() {
    let s1 = groundspring::anderson::disorder_sweep(500, 1.0, 3.0, 3, 3, 42);
    let s2 = groundspring::anderson::disorder_sweep(500, 1.0, 3.0, 3, 3, 42);
    for (a, b) in s1.iter().zip(s2.iter()) {
        assert_eq!(
            a.mean_ratio.to_bits(),
            b.mean_ratio.to_bits(),
            "sweep should be deterministic"
        );
    }
}

// ── freeze_out chi2_analysis (S59 cross-spring) ─────────────────────

#[test]
fn chi2_analysis_agrees_with_chi_squared() {
    let t0 = 155.0;
    let k2 = 0.013;
    let sigma = 0.5;
    let mu_b: Vec<f64> = (0..9).map(|i| f64::from(i) * 50.0).collect();
    let obs: Vec<f64> = mu_b
        .iter()
        .map(|&m| groundspring::freeze_out::freeze_out_curve(t0, k2, m) + 0.1)
        .collect();
    let pred: Vec<f64> = mu_b
        .iter()
        .map(|&m| groundspring::freeze_out::freeze_out_curve(t0, k2, m))
        .collect();

    let basic = groundspring::freeze_out::chi_squared(&obs, &pred, sigma).unwrap();
    let analysis = groundspring::freeze_out::chi2_analysis(&obs, &pred, sigma, 0).unwrap();

    assert!(
        (analysis.chi2_total - basic).abs() < tol::ANALYTICAL,
        "chi2_analysis.total={} vs chi_squared={}",
        analysis.chi2_total,
        basic
    );
    assert_eq!(analysis.dof, 9);
    assert_eq!(analysis.residuals.len(), 9);
    assert_eq!(analysis.pulls.len(), 9);
    assert_eq!(analysis.contributions.len(), 9);
}

// ── esn regime classification ───────────────────────────────────────

#[test]
fn esn_classify_extended_phase() {
    let n = 200;
    let coupling = 0.5;
    let alpha = 0.618_033_988_749_894_9;
    let mut eigs = groundspring::almost_mathieu::eigenvalues(n, coupling, alpha, 0.0);
    let [r, _bw, _kurt] = groundspring::esn::spectral_features(&mut eigs);
    let label = groundspring::esn::classify_by_spacing_ratio(r, 0.03);
    assert_ne!(
        label,
        groundspring::esn::RegimeLabel::Localized,
        "λ=0.5 should be extended, got {label} (r={r})"
    );
}

#[test]
fn esn_classify_localized_phase() {
    let n = 200;
    let coupling = 4.0;
    let alpha = 0.618_033_988_749_894_9;
    let mut eigs = groundspring::almost_mathieu::eigenvalues(n, coupling, alpha, 0.0);
    let [r, _bw, _kurt] = groundspring::esn::spectral_features(&mut eigs);
    let label = groundspring::esn::classify_by_spacing_ratio(r, 0.03);
    assert_ne!(
        label,
        groundspring::esn::RegimeLabel::Extended,
        "λ=4.0 should be localized, got {label} (r={r})"
    );
}

// ── cross-spring lineage sentinel ───────────────────────────────────

#[test]
fn cross_spring_lineage_anderson_sweep_exists() {
    let sweep = groundspring::anderson::disorder_sweep(100, 2.0, 2.0, 1, 2, 42);
    assert!(!sweep.is_empty(), "disorder_sweep should return results");
}

#[cfg(feature = "barracuda-gpu")]
#[test]
fn cross_spring_anderson_2d_eigenvalues() {
    let eigs = groundspring::anderson::anderson_2d_eigenvalues(4, 4, 2.0, 8, 42);
    assert_eq!(
        eigs.len(),
        8,
        "should return 8 eigenvalues from 4×4 lattice"
    );
}

#[cfg(feature = "barracuda-gpu")]
#[test]
fn cross_spring_anderson_3d_eigenvalues() {
    let eigs = groundspring::anderson::anderson_3d_eigenvalues(3, 3, 3, 2.0, 8, 42);
    assert_eq!(
        eigs.len(),
        8,
        "should return 8 eigenvalues from 3×3×3 lattice"
    );
}
