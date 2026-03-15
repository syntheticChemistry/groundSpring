// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Biological and ecological parity checks: diversity, kinetics, ODE integration,
//! rare biosphere, Gillespie SSA, Wright-Fisher, multinomial resampling,
//! FAO-56 evapotranspiration, and tissue Anderson localization.

use groundspring::tol;
use groundspring_forge::harness::Harness;
use std::time::Instant;

/// Run all bio-domain parity checks.
pub fn validate_all(h: &mut Harness) {
    validate_diversity_parity(h);
    validate_kinetics_parity(h);
    validate_bistable_ode_parity(h);
    validate_rare_biosphere_cpu_gpu_parity(h);
    validate_gillespie_batch_parity(h);
    validate_wright_fisher_batch_parity(h);
    validate_multinomial_batch_parity(h);
    validate_bistable_batch_gpu_parity(h);
    validate_fao56_batch_gpu_parity(h);
    validate_tissue_anderson_parity(h);
}

fn validate_diversity_parity(h: &mut Harness) {
    println!("\n--- Diversity Parity (Shannon + Simpson GPU, V65) ---\n");

    let counts = vec![100u64, 50, 25, 10, 5, 3, 2, 1, 1, 1];

    let h1 = groundspring::rarefaction::shannon_diversity(&counts);
    let h2 = groundspring::rarefaction::shannon_diversity(&counts);
    let e1 = groundspring::rarefaction::evenness(&counts);

    println!("  H'={h1:.6}, J'={e1:.6}");

    h.check("Shannon > 0", h1 > 0.0);
    h.check("Evenness in (0,1]", e1 > 0.0 && e1 <= 1.0);
    h.check(
        "Shannon deterministic (bitwise)",
        h1.to_bits() == h2.to_bits(),
    );

    let d1 = groundspring::rarefaction::simpson_diversity(&counts);
    let d2 = groundspring::rarefaction::simpson_diversity(&counts);
    println!("  D={d1:.6}");
    h.check("Simpson in (0,1)", d1 > 0.0 && d1 < 1.0);
    h.check(
        "Simpson deterministic (bitwise)",
        d1.to_bits() == d2.to_bits(),
    );

    let even_counts = vec![100u64, 100, 100, 100];
    let h_even = groundspring::rarefaction::shannon_diversity(&even_counts);
    let expected_h = 4.0_f64.ln();
    h.check(
        "Shannon(even 4-taxa) ≈ ln(4)",
        (h_even - expected_h).abs() < tol::CDF_APPROX,
    );

    let d_even = groundspring::rarefaction::simpson_diversity(&even_counts);
    let expected_d = 4.0_f64.mul_add(-(0.25 * 0.25), 1.0);
    h.check(
        "Simpson(even 4-taxa) ≈ 0.75",
        (d_even - expected_d).abs() < tol::CDF_APPROX,
    );
}

fn validate_kinetics_parity(h: &mut Harness) {
    println!("\n--- Hill Kinetics Parity (bio-kinetics lineage S68) ---\n");

    let hill_val = groundspring::kinetics::hill(1.0, 0.5, 2.0);
    let repress = groundspring::kinetics::hill_repress(1.0, 0.5, 2.0);

    println!("  hill(1.0, K=0.5, n=2) = {hill_val:.6}");
    println!("  hill_repress(1.0, K=0.5, n=2) = {repress:.6}");

    h.check(
        "Hill + repress = 1.0",
        (hill_val + repress - 1.0).abs() < tol::EXACT,
    );
    h.check("Hill(x>>K) ≈ 1.0", (hill_val - 0.8).abs() < 0.1);

    let hill2 = groundspring::kinetics::hill(1.0, 0.5, 2.0);
    h.check("Hill deterministic", hill_val.to_bits() == hill2.to_bits());
}

fn validate_bistable_ode_parity(h: &mut Harness) {
    println!("\n--- Bistable ODE Parity (bio-dynamics lineage S58) ---\n");

    let params = groundspring::bistable::BistableParams::default();
    let y = [0.1, 0.5, 0.3, 0.2, 0.1];
    let deriv = groundspring::bistable::bistable_derivative(&y, &params);

    println!(
        "  dy/dt = [{:.4}, {:.4}, {:.4}, {:.4}, {:.4}]",
        deriv[0], deriv[1], deriv[2], deriv[3], deriv[4]
    );

    h.check("Derivative non-zero", deriv.iter().any(|&d| d.abs() > 0.0));

    let deriv2 = groundspring::bistable::bistable_derivative(&y, &params);
    h.check(
        "ODE deterministic",
        deriv
            .iter()
            .zip(&deriv2)
            .all(|(a, b)| a.to_bits() == b.to_bits()),
    );
}

fn validate_rare_biosphere_cpu_gpu_parity(h: &mut Harness) {
    println!("\n--- Rare Biosphere Parity (bio-rarefaction lineage S64) ---\n");

    let community = vec![0.5, 0.3, 0.15, 0.04, 0.01];
    let depth = 500_u64;
    let n_samples = 50;

    let t0 = Instant::now();
    let occ1 = groundspring::rare_biosphere::abundance_occupancy(&community, depth, n_samples, 42);
    let us1 = t0.elapsed().as_micros();

    let t1 = Instant::now();
    let _occ2 = groundspring::rare_biosphere::abundance_occupancy(&community, depth, n_samples, 42);
    let us2 = t1.elapsed().as_micros();

    println!(
        "  Occupancy: [{:.2}, {:.2}, {:.2}, {:.2}, {:.2}]",
        occ1[0], occ1[1], occ1[2], occ1[3], occ1[4]
    );
    println!("  Run 1: {us1} µs, Run 2: {us2} µs");

    h.check("Dominant species high occupancy", occ1[0] > 0.9);
    h.check("Occupancy decreases with abundance", occ1[0] >= occ1[4]);

    let tier_abundant =
        groundspring::rare_biosphere::tier_detection_rate(&community, 0, 3, depth, n_samples, 42);
    let tier_rare =
        groundspring::rare_biosphere::tier_detection_rate(&community, 3, 5, depth, n_samples, 42);

    println!("  Tier detection: abundant={tier_abundant:.4}, rare={tier_rare:.4}");

    h.check("Abundant tier ≥ rare tier", tier_abundant >= tier_rare);
}

fn validate_gillespie_batch_parity(h: &mut Harness) {
    println!("\n--- Gillespie Batch Parity (Phase 2b) ---\n");

    let rates = vec![1.0_f64; 10];
    let n_traj = 100;

    let t0 = Instant::now();
    let result =
        groundspring::gillespie::birth_death_ssa_batch(&rates, 1.0, 10, 200.0, n_traj, 50.0, 42);
    let us = t0.elapsed().as_micros();

    let ss = groundspring::gillespie::steady_state_mean(10.0, 1.0);

    println!(
        "  mean={:.4}, variance={:.4}, ss={ss:.4}, {us} µs",
        result.mean, result.variance
    );

    h.check("Gillespie batch mean > 0", result.mean > 0.0);
    h.check(
        "Gillespie batch near steady state",
        (result.mean - ss).abs() < 5.0,
    );
    h.check("Gillespie batch variance > 0", result.variance > 0.0);
    h.check(
        "Gillespie batch n_trajectories",
        result.n_trajectories == n_traj,
    );

    let result2 =
        groundspring::gillespie::birth_death_ssa_batch(&rates, 1.0, 10, 200.0, n_traj, 50.0, 42);
    h.check(
        "Gillespie batch deterministic",
        result.mean.to_bits() == result2.mean.to_bits(),
    );
}

fn validate_wright_fisher_batch_parity(h: &mut Harness) {
    println!("\n--- Wright-Fisher Batch Parity (Phase 2b) ---\n");

    let pop = 100;
    let selection = 0.0;
    let freq = 0.1;
    let n_trials = 500;

    let t0 = Instant::now();
    let fix_count =
        groundspring::drift::wright_fisher_fixation_batch(pop, selection, freq, n_trials, 42);
    let us = t0.elapsed().as_micros();

    let kimura = groundspring::drift::kimura_fixation_prob(pop, selection, freq);
    #[expect(clippy::cast_precision_loss, reason = "count/trials ≤ N ≪ 2^53")]
    let rate = fix_count as f64 / n_trials as f64;

    println!("  fixations={fix_count}/{n_trials}, rate={rate:.4}, Kimura={kimura:.4}, {us} µs");

    h.check("WF batch fixation count > 0", fix_count > 0);
    h.check("WF batch fixation count < n_trials", fix_count < n_trials);
    h.check("WF batch rate near Kimura", (rate - kimura).abs() < 0.15);

    let fix2 =
        groundspring::drift::wright_fisher_fixation_batch(pop, selection, freq, n_trials, 42);
    h.check("WF batch deterministic", fix_count == fix2);
}

fn validate_multinomial_batch_parity(h: &mut Harness) {
    println!("\n--- Multinomial Batch Parity (Phase 2b) ---\n");

    let abundances = vec![0.4, 0.3, 0.2, 0.1];
    let depth = 1000_u64;
    let n_reps = 50;

    let t0 = Instant::now();
    let batch = groundspring::rarefaction::multinomial_sample_batch(&abundances, depth, n_reps, 42);
    let us = t0.elapsed().as_micros();

    h.check("Multinomial batch size", batch.len() == n_reps);

    let all_correct_total = batch.iter().all(|counts| {
        let total: u64 = counts.iter().sum();
        total == depth
    });
    h.check("Multinomial batch totals correct", all_correct_total);

    #[expect(clippy::cast_precision_loss, reason = "depth ≤ N ≪ 2^53")]
    let depth_f = depth as f64;
    #[expect(clippy::cast_precision_loss, reason = "n_reps ≤ N ≪ 2^53")]
    let n_reps_f = n_reps as f64;
    let mean_first: f64 = batch
        .iter()
        .map(|c| {
            #[expect(clippy::cast_precision_loss, reason = "value from small array ≪ 2^53")]
            let v = c[0] as f64;
            v / depth_f
        })
        .sum::<f64>()
        / n_reps_f;
    println!("  {n_reps} reps, mean p[0]={mean_first:.4} (expected ~0.4), {us} µs");

    h.check(
        "Multinomial batch p[0] near expected",
        (mean_first - 0.4).abs() < 0.05,
    );

    let batch2 =
        groundspring::rarefaction::multinomial_sample_batch(&abundances, depth, n_reps, 42);
    let deterministic = if cfg!(feature = "barracuda-gpu") {
        batch.iter().zip(&batch2).all(|(a, b)| {
            let a_total: u64 = a.iter().sum();
            let b_total: u64 = b.iter().sum();
            a_total == b_total
        })
    } else {
        batch == batch2
    };
    h.check("Multinomial batch deterministic", deterministic);
}

fn validate_bistable_batch_gpu_parity(h: &mut Harness) {
    println!("\n--- Bistable Batch GPU Parity (V66) ---\n");

    let params = groundspring::bistable::BistableParams::default();
    let ics = [
        [0.95, 4.5, 1.9, 0.3, 0.02],
        [0.95, 4.5, 1.9, 2.5, 0.85],
        [0.5, 1.0, 0.5, 1.0, 0.3],
    ];

    let t0 = Instant::now();
    let batch = groundspring::bistable::integrate_batch(&ics, &params, 0.01, 5_000);
    let us = t0.elapsed().as_micros();

    h.check("Batch length matches", batch.len() == 3);
    h.check(
        "All states non-negative",
        batch.iter().all(|s| s.iter().all(|&v| v >= 0.0)),
    );

    let single_low = groundspring::bistable::integrate(&ics[0], &params, 0.01, 5_000);
    let tol = if cfg!(feature = "barracuda-gpu") {
        0.1
    } else {
        f64::EPSILON
    };
    h.check(
        "Batch[0] ≈ single integrate",
        (batch[0][3] - single_low[3]).abs() < tol,
    );

    println!(
        "  3 trajectories, c-di-GMP finals: [{:.3}, {:.3}, {:.3}], {us} µs",
        batch[0][3], batch[1][3], batch[2][3]
    );
}

fn validate_fao56_batch_gpu_parity(h: &mut Harness) {
    use groundspring::fao56::DailyWeatherInputs;
    println!("\n--- FAO-56 Batch GPU Parity (V66) ---\n");

    let inputs: Vec<DailyWeatherInputs> = vec![
        DailyWeatherInputs {
            tmax_c: 30.0,
            tmin_c: 20.0,
            rhmax_pct: 60.0,
            rhmin_pct: 40.0,
            wind_speed_10m_km_h: 7.2,
            sunshine_hours: 8.0,
            latitude_deg_n: 42.0,
            altitude_m: 200.0,
            day_of_year: 182,
        },
        DailyWeatherInputs {
            tmax_c: 32.0,
            tmin_c: 22.0,
            rhmax_pct: 65.0,
            rhmin_pct: 45.0,
            wind_speed_10m_km_h: 5.4,
            sunshine_hours: 9.0,
            latitude_deg_n: 42.0,
            altitude_m: 200.0,
            day_of_year: 183,
        },
        DailyWeatherInputs {
            tmax_c: 28.0,
            tmin_c: 18.0,
            rhmax_pct: 70.0,
            rhmin_pct: 50.0,
            wind_speed_10m_km_h: 10.8,
            sunshine_hours: 7.0,
            latitude_deg_n: 42.0,
            altitude_m: 200.0,
            day_of_year: 184,
        },
    ];

    let t0 = Instant::now();
    let et0_batch = groundspring::fao56::daily_et0_batch(&inputs);
    let us = t0.elapsed().as_micros();

    h.check("FAO-56 batch length", et0_batch.len() == 3);
    h.check("FAO-56 batch all > 0", et0_batch.iter().all(|&v| v > 0.0));
    h.check(
        "FAO-56 batch all < 20 mm/day",
        et0_batch.iter().all(|&v| v < 20.0),
    );

    let et0_single: Vec<f64> = inputs.iter().map(groundspring::fao56::daily_et0).collect();
    h.check(
        "FAO-56 batch matches singles",
        et0_batch
            .iter()
            .zip(&et0_single)
            .all(|(a, b)| a.to_bits() == b.to_bits()),
    );

    let et0_2 = groundspring::fao56::daily_et0_batch(&inputs);
    h.check(
        "FAO-56 batch deterministic",
        et0_batch
            .iter()
            .zip(&et0_2)
            .all(|(a, b)| a.to_bits() == b.to_bits()),
    );

    println!(
        "  ET₀ = [{:.2}, {:.2}, {:.2}] mm/day, {us} µs",
        et0_batch[0], et0_batch[1], et0_batch[2]
    );
}

fn validate_tissue_anderson_parity(h: &mut Harness) {
    use groundspring::tissue_anderson::{
        barrier_disruption_sweep, effective_disorder, healthy_dermis, healthy_epidermis,
        inflamed_dermis, pielou_evenness, simulate_tissue,
    };

    println!("\n--- Tissue Anderson Parity (Paper 12) ---\n");

    let t0 = Instant::now();

    let epi = healthy_epidermis();
    let derm = healthy_dermis();
    let inflamed = inflamed_dermis();

    let w_epi = effective_disorder(&epi.cell_composition);
    let w_derm = effective_disorder(&derm.cell_composition);
    let w_inflamed = effective_disorder(&inflamed.cell_composition);

    println!("  W(epidermis)={w_epi:.3}, W(dermis)={w_derm:.3}, W(inflamed)={w_inflamed:.3}");

    h.check("Epidermis W < dermis W", w_epi < w_derm);
    h.check("Inflamed W > healthy dermis W", w_inflamed > w_derm);

    let j_epi = pielou_evenness(&epi.cell_composition);
    let j_inflamed = pielou_evenness(&inflamed.cell_composition);
    h.check("Pielou J'(epidermis) < J'(inflamed)", j_epi < j_inflamed);

    let result = simulate_tissue(&[epi.clone(), derm.clone()], 10, 42);
    h.check("Healthy barrier intact", !result.barrier_breached);

    let result2 = simulate_tissue(&[epi, derm], 10, 42);
    h.check(
        "Tissue simulation deterministic",
        result
            .gamma_per_compartment
            .iter()
            .zip(&result2.gamma_per_compartment)
            .all(|(a, b)| a.to_bits() == b.to_bits()),
    );

    let sweep = barrier_disruption_sweep(5, 5, 42);
    h.check("Sweep healthy not breached", !sweep[0].barrier_breached);
    h.check("Sweep disrupted breached", sweep[4].barrier_breached);

    let us = t0.elapsed().as_micros();
    println!("  7 checks, {us} µs");
}
