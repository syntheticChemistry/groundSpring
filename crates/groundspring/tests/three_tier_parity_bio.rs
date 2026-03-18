// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Three-tier parity tests — ecological and biological primitives.
//!
//! Validates that drift, jackknife, rare-biosphere, quasispecies,
//! rarefaction, Gillespie, kinetics, and ODE derivative functions
//! produce identical results regardless of feature mode
//! (default / barracuda / barracuda-gpu).

use groundspring::tol;

// ── drift ───────────────────────────────────────────────────────────

#[test]
fn drift_kimura_parity_neutral() {
    let p = groundspring::drift::kimura_fixation_prob(100, 0.0, 0.5);
    assert!(
        (p - 0.5).abs() < tol::ANALYTICAL,
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
        (p - expected).abs() < tol::EXACT,
        "Kimura known value: got {p}, expected {expected}"
    );
}

#[test]
fn drift_wf_parity_deterministic() {
    let a = groundspring::drift::wright_fisher_fixation(100, 0.01, 0.5, 42).unwrap();
    let b = groundspring::drift::wright_fisher_fixation(100, 0.01, 0.5, 42).unwrap();
    assert_eq!(a, b);
}

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
    #[expect(clippy::cast_precision_loss, reason = "count/trials ≤ N ≪ 2^53")]
    let observed = fix_count as f64 / n_trials as f64;
    assert!(
        (observed - kimura).abs() < 0.15,
        "batch fixation fraction {observed} vs Kimura {kimura}"
    );
}

// ── jackknife ───────────────────────────────────────────────────────

#[test]
fn jackknife_parity_small_sample() {
    let data = [1.0, 2.0, 3.0, 4.0, 5.0];
    let r = groundspring::jackknife::jackknife_mean_variance(&data).expect("jackknife on [1..5]");
    assert!((r.estimate - 3.0).abs() < tol::EXACT);
    assert!(r.variance > 0.0);
    assert!(r.std_error > 0.0);
}

#[test]
fn jackknife_parity_bitwise_deterministic() {
    let data: Vec<f64> = (0..100).map(|i| f64::from(i).mul_add(0.7, 1.5)).collect();
    let r1 = groundspring::jackknife::jackknife_mean_variance(&data)
        .expect("jackknife determinism run 1");
    let r2 = groundspring::jackknife::jackknife_mean_variance(&data)
        .expect("jackknife determinism run 2");
    assert_eq!(r1.estimate.to_bits(), r2.estimate.to_bits());
    assert_eq!(r1.variance.to_bits(), r2.variance.to_bits());
}

#[test]
fn jackknife_parity_known_variance() {
    let data = [1.0, 3.0];
    let r =
        groundspring::jackknife::jackknife_mean_variance(&data).expect("jackknife on [1.0, 3.0]");
    // N=2: leave-one-out means are [3.0, 1.0], grand mean 2.0
    // JK var = (2-1)/2 * ((3-2)^2 + (1-2)^2) = 0.5 * 2 = 1.0
    assert!((r.estimate - 2.0).abs() < tol::EXACT);
    assert!((r.variance - 1.0).abs() < tol::EXACT);
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
    assert!((est - expected).abs() < tol::ANALYTICAL);
}

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

// ── quasispecies ────────────────────────────────────────────────────

#[test]
fn quasispecies_error_threshold_parity() {
    let mu_c = groundspring::quasispecies::error_threshold(10.0, 100);
    assert!(
        (mu_c - 0.02276).abs() < tol::LITERATURE,
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

#[test]
fn quasispecies_simulation_parity_deterministic() {
    let a =
        groundspring::quasispecies::quasispecies_simulation(500, 100, 10.0, 0.01, 50, 42).unwrap();
    let b =
        groundspring::quasispecies::quasispecies_simulation(500, 100, 10.0, 0.01, 50, 42).unwrap();
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(b.iter()) {
        assert_eq!(x.to_bits(), y.to_bits(), "bitwise determinism");
    }
}

#[test]
fn quasispecies_simulation_parity_below_threshold() {
    let mu_c = groundspring::quasispecies::error_threshold(10.0, 100);
    let freqs =
        groundspring::quasispecies::quasispecies_simulation(1000, 100, 10.0, mu_c * 0.5, 200, 42)
            .unwrap();
    #[expect(clippy::cast_precision_loss, reason = "slice length < 2^52")]
    let avg: f64 = freqs.iter().skip(100).sum::<f64>() / freqs[100..].len() as f64;
    assert!(avg > 0.05, "below threshold, master persists: avg={avg}");
}

// ── rarefaction (diversity indices) ───────────────────────────────

#[test]
fn simpson_diversity_parity_known_value() {
    let counts = [10, 10, 10, 10];
    let d1 = groundspring::rarefaction::simpson_diversity(&counts);
    let d2 = groundspring::rarefaction::simpson_diversity(&counts);
    assert_eq!(d1.to_bits(), d2.to_bits(), "simpson bitwise");
    assert!(
        (d1 - 0.75).abs() < tol::STOCHASTIC,
        "even community D ≈ 0.75: {d1}"
    );
}

#[test]
fn shannon_diversity_parity_known_value() {
    let counts = [10, 10, 10, 10];
    let h1 = groundspring::rarefaction::shannon_diversity(&counts);
    let h2 = groundspring::rarefaction::shannon_diversity(&counts);
    assert_eq!(h1.to_bits(), h2.to_bits(), "shannon bitwise");
    let expected = (4.0_f64).ln();
    assert!(
        (h1 - expected).abs() < tol::ANALYTICAL,
        "H = ln(4) ≈ {expected}: {h1}"
    );
}

#[test]
fn evenness_parity_known_value() {
    let counts = [10, 10, 10, 10];
    let e1 = groundspring::rarefaction::evenness(&counts);
    let e2 = groundspring::rarefaction::evenness(&counts);
    assert_eq!(e1.to_bits(), e2.to_bits(), "evenness bitwise");
    assert!(
        (e1 - 1.0).abs() < tol::ANALYTICAL,
        "perfectly even J = 1.0: {e1}"
    );
}

#[test]
fn bray_curtis_parity_known_value() {
    let a = [10.0, 20.0, 30.0];
    let b = [10.0, 20.0, 30.0];
    let d = groundspring::rarefaction::bray_curtis(&a, &b);
    assert!(
        (d - 0.0).abs() < tol::DETERMINISM,
        "identical => BC = 0: {d}"
    );

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
    assert!((y - 0.5).abs() < tol::ANALYTICAL, "hill(K,K,n) = 0.5: {y}");
    let y2 = groundspring::kinetics::hill(10.0, 10.0, 2.0);
    assert_eq!(y.to_bits(), y2.to_bits(), "hill bitwise");
}

#[test]
fn hill_parity_extreme() {
    let sat = groundspring::kinetics::hill(1e6, 1.0, 2.0);
    assert!(
        (sat - 1.0).abs() < tol::CDF_APPROX,
        "saturated hill ≈ 1.0: {sat}"
    );
    let low = groundspring::kinetics::hill(1e-6, 1.0, 2.0);
    assert!(low < 1e-6, "subsaturated hill ≈ 0.0: {low}");
}

#[test]
fn monod_parity_known_value() {
    let y = groundspring::kinetics::monod(10.0, 1.0, 10.0);
    assert!((y - 0.5).abs() < tol::ANALYTICAL, "monod(K,1,K) = 0.5: {y}");
    let y2 = groundspring::kinetics::monod(10.0, 1.0, 10.0);
    assert_eq!(y.to_bits(), y2.to_bits(), "monod bitwise");
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
    assert!((ss - 18.182).abs() < tol::STOCHASTIC);
}

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

// ── Shannon / Simpson GPU parity ────────────────────────────────

#[test]
fn shannon_diversity_gpu_parity_known_value() {
    let counts = vec![100_u64, 100, 100, 100];
    let h = groundspring::rarefaction::shannon_diversity(&counts);
    let expected = 4.0_f64.ln();
    assert!(
        (h - expected).abs() < tol::CDF_APPROX,
        "4 even taxa: H'={h}, expected {expected}"
    );
}

#[test]
fn shannon_diversity_gpu_parity_single_species() {
    let counts = vec![1000_u64, 0, 0, 0];
    let h = groundspring::rarefaction::shannon_diversity(&counts);
    assert!(
        h.abs() < tol::ANALYTICAL,
        "single species H' should be 0: {h}"
    );
}

#[test]
fn simpson_diversity_gpu_parity_known_value() {
    let counts = vec![100_u64, 100, 100, 100];
    let d = groundspring::rarefaction::simpson_diversity(&counts);
    let expected = 4.0_f64.mul_add(-(0.25 * 0.25), 1.0);
    assert!(
        (d - expected).abs() < tol::CDF_APPROX,
        "4 even taxa: D={d}, expected {expected}"
    );
}

#[test]
fn simpson_diversity_gpu_parity_single_species() {
    let counts = vec![1000_u64, 0, 0, 0];
    let d = groundspring::rarefaction::simpson_diversity(&counts);
    assert!(
        d.abs() < tol::ANALYTICAL,
        "single species D should be 0: {d}"
    );
}

#[test]
fn shannon_simpson_determinism() {
    let counts = vec![50_u64, 30, 15, 5];
    let h1 = groundspring::rarefaction::shannon_diversity(&counts);
    let h2 = groundspring::rarefaction::shannon_diversity(&counts);
    assert_eq!(h1.to_bits(), h2.to_bits(), "Shannon bitwise determinism");
    let d1 = groundspring::rarefaction::simpson_diversity(&counts);
    let d2 = groundspring::rarefaction::simpson_diversity(&counts);
    assert_eq!(d1.to_bits(), d2.to_bits(), "Simpson bitwise determinism");
}

// ── tissue Anderson parity ─────────────────────────────────────

#[test]
fn tissue_anderson_healthy_parity() {
    let epi = groundspring::tissue_anderson::healthy_epidermis();
    let derm = groundspring::tissue_anderson::healthy_dermis();
    let r1 = groundspring::tissue_anderson::simulate_tissue(&[epi.clone(), derm.clone()], 5, 42);
    let r2 = groundspring::tissue_anderson::simulate_tissue(&[epi, derm], 5, 42);
    for (a, b) in r1
        .gamma_per_compartment
        .iter()
        .zip(&r2.gamma_per_compartment)
    {
        assert_eq!(a.to_bits(), b.to_bits(), "tissue gamma determinism");
    }
    assert!(!r1.barrier_breached, "healthy skin should not breach");
}

#[test]
fn tissue_anderson_disruption_monotonic() {
    let sweep = groundspring::tissue_anderson::barrier_disruption_sweep(5, 3, 42);
    for i in 1..sweep.len() {
        assert!(
            sweep[i].d_eff_epidermis >= sweep[i - 1].d_eff_epidermis - tol::EQUILIBRIUM,
            "d_eff should be non-decreasing across barrier disruption"
        );
    }
}
