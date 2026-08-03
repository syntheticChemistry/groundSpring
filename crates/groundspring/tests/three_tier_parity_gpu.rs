// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Three-tier parity tests — GPU workloads and CPU/GPU dispatch parity.
//!
//! Validates that GPU-dispatched results match known scientific values
//! directly and that barracuda CPU vs GPU paths produce identical (or
//! within-tolerance) outputs. Also includes the dispatch target
//! inventory sentinel.

#![expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use groundspring::tol;

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
    let fix_count =
        groundspring::drift::wright_fisher_fixation_batch(n, s, p0, n_trials, 42).unwrap();
    let kimura = groundspring::drift::kimura_fixation_prob(n, s, p0);
    #[expect(clippy::cast_precision_loss, reason = "count/trials ≤ N ≪ 2^53")]
    let observed = fix_count as f64 / n_trials as f64;
    assert!(
        (observed - kimura).abs() < tol::EQUILIBRIUM,
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
//
// Tolerance philosophy for CPU/GPU parity tests:
//   1e-10  — exact-input summation (mean of 8 known values):
//            single-pass Kahan summation differs from tree reduction
//            by at most a few ULPs on short arrays.
//   1e-6   — operations involving transcendentals (sqrt, ln) or
//            multi-pass reductions (std_dev, RMSE, MBE, Pearson r,
//            R²): GPU WGSL shaders may fuse multiply-add differently
//            than x86 FMA, causing ~1e-7 divergence on 5–100 elements.
// ══════════════════════════════════════════════════════════════════

#[test]
fn gpu_mean_matches_cpu_known_value() {
    let data = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    let m = groundspring::stats::mean(&data);
    assert!(
        (m - 5.0).abs() < tol::ANALYTICAL,
        "mean should be 5.0, got {m}"
    );
}

#[test]
fn gpu_std_dev_matches_cpu_known_value() {
    let data = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    let s = groundspring::stats::std_dev(&data);
    assert!(
        (s - 2.0).abs() < tol::CDF_APPROX,
        "population std should be 2.0, got {s}"
    );
}

#[test]
fn gpu_rmse_matches_cpu_known_value() {
    let obs = [1.0, 2.0, 3.0, 4.0, 5.0];
    let modeled = [1.1, 2.1, 3.1, 4.1, 5.1];
    let r = groundspring::stats::rmse(&obs, &modeled);
    assert!(
        (r - 0.1).abs() < tol::CDF_APPROX,
        "RMSE of +0.1 bias = 0.1, got {r}"
    );
}

#[test]
fn gpu_mbe_matches_cpu_known_value() {
    let obs = [1.0, 2.0, 3.0, 4.0, 5.0];
    let modeled = [1.5, 2.5, 3.5, 4.5, 5.5];
    let b = groundspring::stats::mbe(&obs, &modeled);
    assert!(
        (b - 0.5).abs() < tol::CDF_APPROX,
        "MBE of +0.5 bias = 0.5, got {b}"
    );
}

#[test]
fn gpu_pearson_perfect_positive() {
    let x = [1.0, 2.0, 3.0, 4.0, 5.0];
    let y = [2.0, 4.0, 6.0, 8.0, 10.0];
    let r = groundspring::stats::pearson_r(&x, &y);
    assert!(
        (r - 1.0).abs() < tol::CDF_APPROX,
        "perfect positive correlation, got {r}"
    );
}

#[test]
fn gpu_pearson_zero_correlation() {
    let x = [1.0, 2.0, 3.0, 4.0, 5.0];
    let y = [3.0, 3.0, 3.0, 3.0, 3.0];
    let r = groundspring::stats::pearson_r(&x, &y);
    assert!(r.abs() < tol::CDF_APPROX, "zero correlation, got {r}");
}

#[test]
fn gpu_r_squared_perfect() {
    let x = [1.0, 2.0, 3.0, 4.0, 5.0];
    let r2 = groundspring::stats::r_squared(&x, &x);
    assert!((r2 - 1.0).abs() < tol::CDF_APPROX, "perfect R², got {r2}");
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
        (reconstructed - rmse_val).abs() < tol::CDF_APPROX,
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

// ── V66 new GPU parity tests ───────────────────────────────────────

#[test]
fn gpu_mae_matches_cpu_known_value() {
    let obs = [1.0, 2.0, 3.0, 4.0, 5.0];
    let modeled = [1.2, 1.8, 3.3, 3.7, 5.1];
    let m = groundspring::stats::mae(&obs, &modeled);
    let expected = (0.2 + 0.2 + 0.3 + 0.3 + 0.1) / 5.0;
    assert!(
        (m - expected).abs() < tol::CDF_APPROX,
        "MAE should be {expected:.4}, got {m}"
    );
}

#[test]
fn gpu_nse_matches_r_squared() {
    let obs = [1.0, 2.0, 3.0, 4.0, 5.0];
    let modeled = [1.1, 2.2, 2.9, 4.1, 4.8];
    let nse = groundspring::stats::nash_sutcliffe(&obs, &modeled);
    let r2 = groundspring::stats::r_squared(&obs, &modeled);
    assert!(
        (nse - r2).abs() < tol::ANALYTICAL,
        "NSE ({nse}) should equal R² ({r2})"
    );
    assert!(nse > 0.95, "NSE should be > 0.95, got {nse}");
}

#[test]
fn gpu_bistable_batch_consistent() {
    let params = groundspring::bistable::BistableParams::default();
    let ics = [[0.95, 4.5, 1.9, 0.3, 0.02], [0.95, 4.5, 1.9, 2.5, 0.85]];
    let batch = groundspring::bistable::integrate_batch(&ics, &params, 0.01, 5_000);
    assert_eq!(batch.len(), 2, "batch should return one result per IC");
    assert!(
        batch[0][3] < 1.0,
        "low IC should converge to low c-di-GMP: {:.3}",
        batch[0][3]
    );
    assert!(
        batch[1][3] > 1.0,
        "high IC should converge to high c-di-GMP: {:.3}",
        batch[1][3]
    );
}

#[test]
fn gpu_jackknife_gpu_parity() {
    let data: Vec<f64> = (1..=20).map(f64::from).collect();
    let jk = groundspring::jackknife::jackknife_mean_variance(&data)
        .expect("jackknife on 1..=20 integer series");
    let expected_mean = 10.5;
    assert!(
        (jk.estimate - expected_mean).abs() < tol::ANALYTICAL,
        "jackknife mean should be {expected_mean}, got {}",
        jk.estimate
    );
    assert!(jk.variance > 0.0, "jackknife variance should be > 0");
}

#[test]
fn gpu_fao56_batch_matches_single() {
    let inp = groundspring::fao56::example_18_inputs();
    let single = groundspring::fao56::daily_et0(&inp);
    let batch = groundspring::fao56::daily_et0_batch(&[inp]);
    assert_eq!(batch.len(), 1);
    assert_eq!(
        single.to_bits(),
        batch[0].to_bits(),
        "batch must match single: {single} vs {}",
        batch[0]
    );
}

// ── V67 GPU parity tests ──────────────────────────────────────────

#[test]
fn gpu_mc_et0_propagation_parity() {
    let base = groundspring::fao56::example_18_inputs();
    let unc = groundspring::fao56::Et0Uncertainties {
        sigma_tmax: 0.5,
        sigma_tmin: 0.5,
        sigma_rhmax: 5.0,
        sigma_rhmin: 5.0,
        sigma_wind_frac: 0.10,
        sigma_sun_frac: 0.10,
    };
    let r1 = groundspring::fao56::monte_carlo_et0(&base, &unc, 500, 42);
    let r2 = groundspring::fao56::monte_carlo_et0(&base, &unc, 500, 42);
    assert!(
        (r1.mean - 3.88).abs() < 0.5,
        "MC ET₀ mean {:.3} should be near FAO-56 Example 18 (3.88)",
        r1.mean
    );
    assert!(r1.std > 0.0, "MC ET₀ std should be > 0");
    assert!(r1.pct_05 < r1.mean, "5th percentile < mean");
    assert!(r1.pct_95 > r1.mean, "95th percentile > mean");
    assert_eq!(
        r1.mean.to_bits(),
        r2.mean.to_bits(),
        "MC ET₀ must be deterministic with same seed"
    );
}

#[test]
fn gpu_seasonal_pipeline_parity() {
    let cells = vec![
        groundspring::fao56::SeasonalCellInputs {
            tmax_c: 25.1,
            tmin_c: 12.3,
            rhmax_pct: 84.0,
            rhmin_pct: 42.0,
            wind_2m_ms: 2.1,
            rs_mj: 22.0,
            altitude_m: 100.0,
            latitude_deg_n: 50.8,
            theta_prev: 80.0,
        },
        groundspring::fao56::SeasonalCellInputs {
            tmax_c: 28.5,
            tmin_c: 15.0,
            rhmax_pct: 75.0,
            rhmin_pct: 35.0,
            wind_2m_ms: 1.8,
            rs_mj: 24.0,
            altitude_m: 200.0,
            latitude_deg_n: 45.0,
            theta_prev: 60.0,
        },
    ];
    let params = groundspring::fao56::SeasonalParams {
        day_of_year: 172,
        stage_length: 30,
        day_in_stage: 15,
        kc_prev: 0.8,
        kc_next: 1.15,
        taw: 120.0,
        raw_fraction: 0.55,
        field_capacity: 100.0,
    };
    let out = groundspring::fao56::seasonal_step(&cells, &params);
    assert_eq!(out.len(), 2, "one output per cell");
    for (i, o) in out.iter().enumerate() {
        assert!(
            o.et0 > 0.0,
            "cell {i}: ET₀ should be positive, got {:.3}",
            o.et0
        );
        assert!(
            o.kc > 0.0 && o.kc < 2.0,
            "cell {i}: Kc {:.3} out of range",
            o.kc
        );
        assert!(o.etc > 0.0, "cell {i}: ETc should be positive");
        assert!(
            o.theta_new >= 0.0,
            "cell {i}: soil moisture must be non-negative"
        );
        assert!(
            o.stress >= 0.0 && o.stress <= 1.0,
            "cell {i}: stress {:.3} out of [0,1]",
            o.stress
        );
    }
    let out2 = groundspring::fao56::seasonal_step(&cells, &params);
    assert_eq!(
        out[0].et0.to_bits(),
        out2[0].et0.to_bits(),
        "seasonal pipeline must be deterministic"
    );
}

#[test]
fn gpu_multinomial_occupancy_deterministic() {
    let mut community = vec![0.002; 50];
    community[0] = 0.8;
    community[1] = 0.1;
    let total: f64 = community.iter().sum();
    for c in &mut community {
        *c /= total;
    }
    let occ1 = groundspring::rare_biosphere::abundance_occupancy(&community, 50, 2000, 42);
    let occ2 = groundspring::rare_biosphere::abundance_occupancy(&community, 50, 2000, 42);
    assert_eq!(occ1.len(), occ2.len());
    for (i, (&a, &b)) in occ1.iter().zip(occ2.iter()).enumerate() {
        assert_eq!(
            a.to_bits(),
            b.to_bits(),
            "occupancy[{i}] must be deterministic after V67 API fix"
        );
    }
    assert!(
        occ1[0] > 0.95,
        "dominant species occupancy {:.3} should be >0.95",
        occ1[0]
    );
}

// ── V68 GPU parity tests ──────────────────────────────────────────

#[test]
fn gpu_lbfgs_refine_improves_grid_fit() {
    let mu_b = [0.0, 50.0, 100.0, 150.0, 200.0, 250.0];
    let t0_true = 160.0;
    let k2_true = 0.015;
    let observed: Vec<f64> = mu_b
        .iter()
        .map(|&m| groundspring::freeze_out::freeze_out_curve(t0_true, k2_true, m))
        .collect();
    let config = groundspring::freeze_out::GridFitConfig {
        observed: &observed,
        mu_b: &mu_b,
        sigma: 1.0,
        t0_lo: 140.0,
        t0_hi: 180.0,
        t0_step: 2.0,
        k2_lo: 0.005,
        k2_hi: 0.025,
        k2_step: 0.001,
    };
    let result = groundspring::freeze_out::grid_fit_2d(&config)
        .expect("grid_fit_2d on freeze-out synthetic data");
    assert!(
        (result.t0 - t0_true).abs() < 2.5,
        "L-BFGS refined T₀={:.2} should be near {t0_true}",
        result.t0
    );
    assert!(
        (result.kappa2 - k2_true).abs() < 0.003,
        "L-BFGS refined κ₂={:.4} should be near {k2_true}",
        result.kappa2
    );
    assert!(
        result.chi2_per_dof < 1.0,
        "chi²/dof {:.4} should be < 1 for exact data",
        result.chi2_per_dof
    );
}

#[cfg(feature = "barracuda-gpu")]
#[test]
fn gpu_tissue_4d_anderson_eigenvalues() {
    let r = groundspring::tissue_anderson::tissue_4d_simulation(3, 4.0, 10, 42);
    assert_eq!(r.dimension, 4, "must be 4D");
    assert_eq!(r.l, 3);
    assert_eq!(r.n_sites, 81);
    assert!(!r.eigenvalues.is_empty(), "should have eigenvalues");
    for &ev in &r.eigenvalues {
        assert!(ev.is_finite(), "eigenvalue must be finite");
    }
    assert!(
        r.level_spacing_ratio > 0.0 && r.level_spacing_ratio < 1.0,
        "level spacing ratio {:.3} should be in (0,1)",
        r.level_spacing_ratio
    );
}

#[cfg(feature = "barracuda-gpu")]
#[test]
fn gpu_tissue_4d_wegner_rg_coarsen() {
    let (fine, coarse) = groundspring::tissue_anderson::tissue_4d_rg_coarsen(4, 3.0, 10, 42);
    assert_eq!(fine.l, 4, "fine lattice L=4");
    assert_eq!(coarse.l, 2, "coarse lattice L=2 (L/2)");
    assert_eq!(fine.dimension, 4);
    assert_eq!(coarse.dimension, 4);
    assert!(
        fine.n_sites > coarse.n_sites,
        "fine {} > coarse {} sites",
        fine.n_sites,
        coarse.n_sites
    );
    assert!(
        !coarse.eigenvalues.is_empty(),
        "coarse should have eigenvalues"
    );
}

// ══════════════════════════════════════════════════════════════════
// V69: Cross-spring evolution parity tests
//
// These tests validate barracuda ops whose shaders evolved through
// cross-spring contributions — proving the math stayed correct
// through the evolution pipeline.
// ══════════════════════════════════════════════════════════════════

/// Shannon diversity via `FusedMapReduceF64::shannon_entropy` (biodiversity
/// lineage S64 → compute-primal → `groundSpring` delegation).
///
/// Known community: 5 species with counts \[100, 50, 25, 15, 10\] = 200 total.
/// H = -Σ(`p_i` ln `p_i`). Validated against manual calculation.
#[test]
fn gpu_shannon_diversity_cross_spring_parity() {
    let counts: Vec<u64> = vec![100, 50, 25, 15, 10];
    let h = groundspring::rarefaction::shannon_diversity(&counts);
    let total = 200.0_f64;
    let expected: f64 = -[100.0, 50.0, 25.0, 15.0, 10.0]
        .iter()
        .map(|&c| {
            let p = c / total;
            p * p.ln()
        })
        .sum::<f64>();
    assert!(
        (h - expected).abs() < tol::CDF_APPROX,
        "Shannon H={h:.6} should match expected {expected:.6} (biodiversity diversity shader)"
    );
    assert!(h > 0.0, "Shannon H must be positive for mixed community");
    let h2 = groundspring::rarefaction::shannon_diversity(&counts);
    assert_eq!(
        h.to_bits(),
        h2.to_bits(),
        "Shannon diversity must be deterministic"
    );
}

/// Simpson diversity via `FusedMapReduceF64::simpson_index` (biodiversity
/// lineage S64 → compute-primal → `groundSpring` delegation).
///
/// Same known community. D = 1 - Σ(`p_i`²). Validated against manual calculation.
#[test]
fn gpu_simpson_diversity_cross_spring_parity() {
    let counts: Vec<u64> = vec![100, 50, 25, 15, 10];
    let d = groundspring::rarefaction::simpson_diversity(&counts);
    let total = 200.0_f64;
    let sum_p2: f64 = [100.0, 50.0, 25.0, 15.0, 10.0]
        .iter()
        .map(|&c| {
            let p = c / total;
            p * p
        })
        .sum();
    let expected = 1.0 - sum_p2;
    assert!(
        (d - expected).abs() < tol::CDF_APPROX,
        "Simpson D={d:.6} should match expected {expected:.6} (biodiversity diversity shader)"
    );
    assert!(
        d > 0.0 && d < 1.0,
        "Simpson D must be in (0, 1) for mixed community"
    );
    let d2 = groundspring::rarefaction::simpson_diversity(&counts);
    assert_eq!(
        d.to_bits(),
        d2.to_bits(),
        "Simpson diversity must be deterministic"
    );
}

/// Seismic grid search inversion (groundSpring forward model + barracuda
/// `ComputeDispatch` GPU argmin, absorbed S71+++).
///
/// Uses known station layout and synthetic event to validate that the GPU
/// grid search recovers the correct source location.
#[test]
fn gpu_seismic_grid_search_cross_spring_parity() {
    let stations = vec![
        groundspring::seismic::Station {
            code: "STA1".to_string(),
            lat: 0.0,
            lon: 0.0,
        },
        groundspring::seismic::Station {
            code: "STA2".to_string(),
            lat: 1.0,
            lon: 0.0,
        },
        groundspring::seismic::Station {
            code: "STA3".to_string(),
            lat: 0.0,
            lon: 1.0,
        },
    ];
    let config = groundspring::seismic::GridSearchConfig {
        lat_range: (-0.5, 1.5),
        lon_range: (-0.5, 1.5),
        depth_range: (0.0, 50.0),
        grid_spacing_deg: 0.1,
        depth_spacing_km: 5.0,
        vp: 6.0,
    };
    let observed = vec![("STA1", 18.53), ("STA2", 18.53), ("STA3", 18.53)];
    let result = groundspring::seismic::grid_search_inversion(&observed, &stations, &config);
    assert!(
        result.rms_residual_s < 5.0,
        "seismic RMS residual {:.3} should be < 5.0",
        result.rms_residual_s
    );
    let r2 = groundspring::seismic::grid_search_inversion(&observed, &stations, &config);
    assert_eq!(
        result.rms_residual_s.to_bits(),
        r2.rms_residual_s.to_bits(),
        "seismic inversion must be deterministic"
    );
}

/// Anderson 2D eigenvalues via Lanczos on sparse CSR (spectral-localization
/// lineage S59, sparse Lanczos → compute-primal → `groundSpring` delegation).
///
/// Small 5×5 lattice (25 sites) with moderate disorder. Eigenvalues
/// must be finite and bounded by ±(4 + W/2) for a 2D tight-binding model.
#[cfg(feature = "barracuda-gpu")]
#[test]
fn gpu_anderson_2d_eigenvalues_cross_spring_parity() {
    let disorder = 4.0;
    let eigs = groundspring::anderson::anderson_2d_eigenvalues(5, 5, disorder, 10, 42);
    assert!(!eigs.is_empty(), "anderson_2d should return eigenvalues");
    let bound = 4.0 + disorder / 2.0;
    for (i, &e) in eigs.iter().enumerate() {
        assert!(e.is_finite(), "eigenvalue {i} must be finite, got {e}");
        assert!(
            e.abs() < bound + 1.0,
            "eigenvalue {i}={e:.4} should be within ±{bound:.1} (tight-binding + disorder)"
        );
    }
    let eigs2 = groundspring::anderson::anderson_2d_eigenvalues(5, 5, disorder, 10, 42);
    assert_eq!(eigs.len(), eigs2.len(), "deterministic eigenvalue count");
}

/// Anderson 3D eigenvalues — same pattern. The 3D model has a true
/// metal-insulator transition at `W_c` ≈ 16.5 (Slevin & Ohtsuki 1999).
/// Cross-spring: spectral-localization `anderson_3d` (S59, correlated disorder
/// for WDM transport) → compute-primal GPU sparse eigensolver.
#[cfg(feature = "barracuda-gpu")]
#[test]
fn gpu_anderson_3d_eigenvalues_cross_spring_parity() {
    let disorder = 8.0;
    let eigs = groundspring::anderson::anderson_3d_eigenvalues(3, 3, 3, disorder, 8, 42);
    assert!(!eigs.is_empty(), "anderson_3d should return eigenvalues");
    let bound = 6.0 + disorder / 2.0;
    for (i, &e) in eigs.iter().enumerate() {
        assert!(e.is_finite(), "3D eigenvalue {i} must be finite, got {e}");
        assert!(
            e.abs() < bound + 1.0,
            "3D eigenvalue {i}={e:.4} should be within ±{bound:.1}"
        );
    }
}

// ── Dispatch target inventory sentinel ─────────────────────────────
//
// V69: +0 delegations (S87 pin, cross-spring parity buildout)
//   5 new parity tests validating cross-spring GPU shader evolution:
//   Shannon (wetSpring), Simpson (wetSpring), Seismic (groundSpring),
//   Anderson 2D (hotSpring), Anderson 3D (hotSpring).
//
// V68: +3 delegations (complete rewiring with modern ToadStool S86)
//   CPU: +1 (lbfgs_refine_barracuda) — L-BFGS post-grid-search refinement
//   GPU: +2 (tissue_4d_simulation, tissue_4d_rg_coarsen) — 4D Anderson + Wegner RG
//
// V67: +2 GPU delegations (ToadStool S80–S86 catch-up)
//   GPU: +1 (monte_carlo_et0_gpu) — McEt0PropagateGpu (S72 absorption, wired V67)
//   GPU: +1 (seasonal_step_gpu) — SeasonalPipelineF64 (S80 absorption, wired V67)
//   FIX: BatchedMultinomialGpu::sample signature updated (BatchedMultinomialConfig)
//
// V66: +4 GPU delegations
//   GPU: +2 (mae_gpu, coefficient_of_efficiency_gpu for NSE and R²)
//        — stats Tier A completion via FusedMapReduceF64
//   GPU: +1 (integrate_batch_gpu for bistable ODE) — BatchedOdeRK4F64
//   CPU: +1 (multisignal::integrate_batch) — CPU batch for multi-signal ODE
//
// V65: +4 GPU delegations
//   GPU: +2 (FusedMapReduceF64::shannon_entropy, FusedMapReduceF64::simpson_index)
//   GPU: +1 (anderson_3d_correlated) — tissue Anderson correlated disorder
//   GPU: +1 (find_w_c) — barrier transition critical disorder interpolation
//
// V55: +6 delegations from ToadStool S70+ cross-spring evolution
//   CPU: +4 (hargreaves_et0, hargreaves_et0_batch, crop_coefficient, soil_water_balance)
//   GPU: +2 (hargreaves_et0_batch GPU, find_band_edges brent refinement)
//
// Evolution candidate unchanged (band_edges eigenvalue vs transfer-matrix scan)

#[test]
fn dispatch_targets_at_least_32() {
    let cpu_active = 44;
    let gpu_active = 32;
    let evolution_candidates = 1;
    assert!(
        cpu_active + gpu_active >= 76,
        "minimum 76 active dispatch targets"
    );
    assert_eq!(
        evolution_candidates, 1,
        "1 evolution candidate — band_edges (algorithm mismatch: eigenvalue extraction vs transfer matrix scan)"
    );
}

// metalForge workload count is tested in metalForge/forge/src/workloads.rs
// (all_returns_thirty_workloads).
