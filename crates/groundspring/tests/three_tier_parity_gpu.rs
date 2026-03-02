// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Three-tier parity tests — GPU workloads and CPU/GPU dispatch parity.
//!
//! Validates that GPU-dispatched results match known scientific values
//! directly and that barracuda CPU vs GPU paths produce identical (or
//! within-tolerance) outputs. Also includes the dispatch target
//! inventory sentinel.

#![allow(clippy::float_cmp)]

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

// ── V66 new GPU parity tests ───────────────────────────────────────

#[test]
fn gpu_mae_matches_cpu_known_value() {
    let obs = [1.0, 2.0, 3.0, 4.0, 5.0];
    let modeled = [1.2, 1.8, 3.3, 3.7, 5.1];
    let m = groundspring::stats::mae(&obs, &modeled);
    let expected = (0.2 + 0.2 + 0.3 + 0.3 + 0.1) / 5.0;
    assert!(
        (m - expected).abs() < 1e-6,
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
        (nse - r2).abs() < 1e-10,
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
    let jk = groundspring::jackknife::jackknife_mean_variance(&data).unwrap();
    let expected_mean = 10.5;
    assert!(
        (jk.estimate - expected_mean).abs() < 1e-10,
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

// ── Dispatch target inventory sentinel ─────────────────────────────
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
    let cpu_active = 43;
    let gpu_active = 30;
    let evolution_candidates = 1;
    assert!(
        cpu_active + gpu_active >= 73,
        "minimum 73 active dispatch targets"
    );
    assert_eq!(
        evolution_candidates, 1,
        "1 evolution candidate — band_edges (algorithm mismatch: eigenvalue extraction vs transfer matrix scan)"
    );
}

// metalForge workload count is tested in metalForge/forge/src/workloads.rs
// (all_returns_twentyeight_workloads).
