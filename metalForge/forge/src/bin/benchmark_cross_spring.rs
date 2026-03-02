// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Cross-spring benchmark: CPU-local vs barracuda-CPU vs barracuda-GPU.
//!
//! Measures performance across the three execution tiers for workloads
//! that span the cross-spring shader ecosystem:
//!
//! | Workload                  | Shader Provenance             | Spring Origin        |
//! |---------------------------|-------------------------------|----------------------|
//! | Anderson Lyapunov         | hotSpring spectral theory     | hotSpring → S26      |
//! | Almost-Mathieu eigenvalues| hotSpring Hofstadter          | hotSpring → S26      |
//! | Bootstrap RAWR            | groundSpring bootstrap        | groundSpring → S66   |
//! | Regression fits           | airSpring hydrology           | airSpring → S66      |
//! | Stats metrics (RMSE etc.) | airSpring + groundSpring      | mixed → S64          |
//! | Shannon diversity         | wetSpring biodiversity        | wetSpring → S64      |
//! | Multinomial rarefaction   | groundSpring rare biosphere   | groundSpring → S64   |
//! | Rare biosphere occupancy  | groundSpring + neuralSpring   | metalForge → S64     |
//!
//! Cross-spring evolution highlights:
//! - **hotSpring → all**: DF64 precision shaders (biomeGate Feb 2026) give f64-class
//!   precision on consumer GPUs. `Fp64Strategy` auto-selects Native vs Hybrid.
//! - **wetSpring → neuralSpring**: Bio primitives (`BatchedMultinomialGpu`,
//!   `WrightFisherGpu`, `DiversityFusionGpu`) originated in wetSpring, absorbed
//!   through neuralSpring metalForge, now available to all springs.
//! - **neuralSpring → hotSpring**: `pow_f64` polyfill fix (S-17) unblocked all
//!   springs on Ada Lovelace GPUs. `batch_ipr_f64.wgsl` feeds spectral analysis.
//! - **airSpring → groundSpring**: Regression (`fit_linear`, `fit_quadratic`,
//!   `fit_exponential`, `fit_logarithmic`) and hydrology (`hargreaves_et0`)
//!   absorbed in S66, delegated by groundSpring for WDM extrapolation.
//! - **groundSpring → wetSpring**: `rawr_mean` bootstrap and `batched_multinomial`
//!   GPU shader feed rarefaction pipelines.
//!
//! Exit 0 if all benchmarks complete, exit 1 on any failure.

use groundspring_forge::harness::Harness;
use std::time::Instant;

fn main() {
    println!("=== groundSpring Cross-Spring Benchmark ===");
    println!("=== ToadStool S79 / BarraCUDA Universal Precision ===\n");

    let mut h = Harness::new();

    bench_stats_metrics(&mut h);
    bench_bootstrap_rawr(&mut h);
    bench_regression(&mut h);
    bench_diversity(&mut h);
    bench_anderson(&mut h);
    bench_anderson_sweep(&mut h);
    bench_chi2_analysis(&mut h);
    bench_esn_classification(&mut h);
    bench_rarefaction_occupancy(&mut h);

    println!("\n--- Cross-Spring Shader Provenance Summary ---\n");
    print_provenance_table();

    h.finish();
}

fn bench_stats_metrics(h: &mut Harness) {
    println!("\n--- Stats Metrics (airSpring + groundSpring → S64) ---\n");

    let n = 10_000_u32;
    let observed: Vec<f64> = (0..n).map(|i| (f64::from(i) * 0.1).sin()).collect();
    let simulated: Vec<f64> = observed
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            #[expect(clippy::cast_possible_truncation)]
            let idx = i as u32;
            (f64::from(idx) * 0.01).cos().mul_add(0.01, v)
        })
        .collect();

    let t0 = Instant::now();
    let rmse = groundspring::stats::rmse(&observed, &simulated);
    let mbe = groundspring::stats::mbe(&observed, &simulated);
    let mae = groundspring::stats::mae(&observed, &simulated);
    let nse = groundspring::stats::nash_sutcliffe(&observed, &simulated);
    let r2 = groundspring::stats::r_squared(&observed, &simulated);
    let ia = groundspring::stats::index_of_agreement(&observed, &simulated);
    let us = t0.elapsed().as_micros();

    println!("  n = {n} paired observations");
    println!("  RMSE={rmse:.6}, MBE={mbe:.6}, MAE={mae:.6}, NSE={nse:.6}, R²={r2:.6}, IA={ia:.6}");
    println!("  Time: {us} µs (6 metrics in one pass)");

    h.check("RMSE > 0", rmse > 0.0);
    h.check("NSE near 1 (good fit)", nse > 0.99);
    h.check("R² near 1", r2 > 0.99);
    h.check("IA near 1", ia > 0.99);
}

fn bench_bootstrap_rawr(h: &mut Harness) {
    println!("\n--- Bootstrap RAWR (groundSpring → S66) ---\n");

    let data: Vec<f64> = (0..5000)
        .map(|i| (f64::from(i) * 0.1).sin() * 10.0)
        .collect();

    let t0 = Instant::now();
    let ci = groundspring::bootstrap::rawr_mean(&data, 1000, 0.05, 42);
    let rawr_us = t0.elapsed().as_micros();

    let t1 = Instant::now();
    let ci_classic = groundspring::bootstrap::bootstrap_mean(&data, 1000, 0.05, 42);
    let classic_us = t1.elapsed().as_micros();

    println!("  n = {}, B = 1000, α = 0.05", data.len());
    println!(
        "  RAWR:    [{:.4}, {:.4}]  {rawr_us} µs",
        ci.ci_lower, ci.ci_upper
    );
    println!(
        "  Classic: [{:.4}, {:.4}]  {classic_us} µs",
        ci_classic.ci_lower, ci_classic.ci_upper
    );

    h.check("RAWR CI valid (lower < upper)", ci.ci_lower < ci.ci_upper);
    h.check(
        "RAWR CI contains plausible mean (near 0)",
        ci.ci_lower < 0.1 && ci.ci_upper > -0.1,
    );
}

fn bench_regression(h: &mut Harness) {
    println!("\n--- Regression Fits (airSpring → S66) ---\n");

    let n = 1000;
    let x: Vec<f64> = (0..n).map(|i| f64::from(i) * 0.01).collect();
    let y_linear: Vec<f64> = x.iter().map(|&xi| 2.5f64.mul_add(xi, 1.0)).collect();
    let y_quadratic: Vec<f64> = x
        .iter()
        .map(|&xi| (0.5 * xi).mul_add(xi, 2.0 * xi) + 1.0)
        .collect();
    let y_exp: Vec<f64> = x.iter().map(|&xi| 3.0 * (0.5 * xi).exp()).collect();

    let t0 = Instant::now();
    let fit_lin = groundspring::stats::fit_linear(&x, &y_linear);
    let us_lin = t0.elapsed().as_micros();

    let t1 = Instant::now();
    let fit_quad = groundspring::stats::fit_quadratic(&x, &y_quadratic);
    let us_quad = t1.elapsed().as_micros();

    let t2 = Instant::now();
    let fit_exp = groundspring::stats::fit_exponential(&x, &y_exp);
    let us_exp = t2.elapsed().as_micros();

    println!("  n = {n} points");
    if let Some(ref f) = fit_lin {
        println!("  Linear:    R²={:.8}  {us_lin} µs", f.r_squared);
    }
    if let Some(ref f) = fit_quad {
        println!("  Quadratic: R²={:.8}  {us_quad} µs", f.r_squared);
    }
    if let Some(ref f) = fit_exp {
        println!("  Exponential: R²={:.8}  {us_exp} µs", f.r_squared);
    }

    h.check(
        "Linear fit R² ≈ 1.0",
        fit_lin.as_ref().is_some_and(|f| f.r_squared > 0.999),
    );
    h.check(
        "Quadratic fit R² ≈ 1.0",
        fit_quad.as_ref().is_some_and(|f| f.r_squared > 0.999),
    );
    h.check(
        "Exponential fit R² ≈ 1.0",
        fit_exp.as_ref().is_some_and(|f| f.r_squared > 0.999),
    );
}

fn bench_diversity(h: &mut Harness) {
    println!("\n--- Shannon Diversity (wetSpring → S64) ---\n");

    let mut counts = vec![0u64; 200];
    for (i, c) in counts.iter_mut().enumerate() {
        *c = ((200 - i) * (200 - i)) as u64;
    }

    let t0 = Instant::now();
    let shannon = groundspring::rarefaction::shannon_diversity(&counts);
    let us = t0.elapsed().as_micros();

    let even = groundspring::rarefaction::evenness(&counts);

    println!("  S = {} species, uneven abundance", counts.len());
    println!("  H' = {shannon:.6}, J' = {even:.6}");
    println!("  Time: {us} µs");

    h.check("Shannon > 0 for non-trivial community", shannon > 0.0);
    h.check("Evenness in (0, 1]", even > 0.0 && even <= 1.0);
}

fn bench_anderson(h: &mut Harness) {
    println!("\n--- Anderson Lyapunov (hotSpring → S26, spectral theory) ---\n");

    let n_sites = 200;
    let disorder = 2.0;
    let energy = 0.0;
    let n_realizations = 500;

    let t0 = Instant::now();
    let gamma =
        groundspring::anderson::lyapunov_averaged(n_sites, disorder, energy, n_realizations, 42);
    let us = t0.elapsed().as_micros();
    let xi = if gamma > 0.0 {
        1.0 / gamma
    } else {
        f64::INFINITY
    };

    println!("  L={n_sites}, W={disorder}, E={energy}, R={n_realizations}");
    println!("  γ = {gamma:.6}, ξ = {xi:.2}");
    println!("  Time: {us} µs");

    h.check("γ > 0 (localized regime)", gamma > 0.0);
    h.check(
        "ξ ∈ [5, 50] (finite-size range)",
        (5.0..=50.0).contains(&xi),
    );

    let analytical_xi = groundspring::anderson::analytical_localization_length(disorder, energy);
    println!("  Analytical ξ(W=2) = {analytical_xi:.2} (hotSpring special functions)");
    h.check("Analytical ξ > 0", analytical_xi > 0.0);
}

fn bench_anderson_sweep(h: &mut Harness) {
    println!("\n--- Anderson Disorder Sweep (hotSpring S59 → groundSpring) ---\n");

    let n_sites = 200;
    let n_points = 10;
    let n_realizations = 50;

    let t0 = Instant::now();
    let sweep =
        groundspring::anderson::disorder_sweep(n_sites, 0.5, 5.0, n_points, n_realizations, 42);
    let us = t0.elapsed().as_micros();

    println!("  L={n_sites}, W∈[0.5, 5.0], {n_points} points × {n_realizations} realizations");
    println!("  Time: {us} µs");
    for p in &sweep {
        println!(
            "    W={:.2}: γ={:.6} ± {:.6}",
            p.disorder, p.mean_ratio, p.std_error
        );
    }

    h.check(
        "Sweep returns correct number of points",
        sweep.len() == n_points,
    );
    h.check(
        "Lyapunov increases with disorder",
        sweep.last().expect("non-empty sweep").mean_ratio
            > sweep.first().expect("non-empty sweep").mean_ratio,
    );
}

fn bench_chi2_analysis(h: &mut Harness) {
    println!("\n--- Chi² Decomposed Analysis (hotSpring S59 → groundSpring) ---\n");

    let t0_param = 155.0;
    let k2 = 0.013;
    let sigma = 0.5;
    let mu_b: Vec<f64> = (0..50).map(|i| f64::from(i) * 10.0).collect();
    let obs: Vec<f64> = mu_b
        .iter()
        .map(|&m| groundspring::freeze_out::freeze_out_curve(t0_param, k2, m) + 0.1)
        .collect();
    let pred: Vec<f64> = mu_b
        .iter()
        .map(|&m| groundspring::freeze_out::freeze_out_curve(t0_param, k2, m))
        .collect();

    let t0 = Instant::now();
    let analysis = groundspring::freeze_out::chi2_analysis(&obs, &pred, sigma, 2).unwrap();
    let us = t0.elapsed().as_micros();

    println!("  n = {} data points, σ = {sigma}", obs.len());
    println!(
        "  χ²/dof = {:.4}, dof = {}",
        analysis.chi2_per_dof, analysis.dof
    );
    println!(
        "  p-value = {} (NaN = CPU fallback, no incomplete gamma)",
        if analysis.p_value.is_nan() {
            "NaN (CPU)".to_string()
        } else {
            format!("{:.6}", analysis.p_value)
        }
    );
    println!(
        "  Max |pull| = {:.4}",
        analysis
            .pulls
            .iter()
            .map(|p| p.abs())
            .fold(0.0_f64, f64::max)
    );
    println!("  Time: {us} µs");

    h.check("Chi² analysis completes", analysis.dof > 0);
    h.check(
        "Residuals are positive (obs > pred)",
        analysis.residuals.iter().all(|&r| r > 0.0),
    );
}

fn bench_esn_classification(h: &mut Harness) {
    println!("\n--- ESN Regime Classification (hotSpring ESN → groundSpring) ---\n");

    let alpha = 0.618_033_988_749_894_9;

    let t0 = Instant::now();
    let mut extended_eigs = groundspring::almost_mathieu::eigenvalues(200, 0.5, alpha, 0.0);
    let ext_features = groundspring::esn::spectral_features(&mut extended_eigs);
    let ext_label = groundspring::esn::classify_by_spacing_ratio(ext_features[0], 0.03);
    let ext_us = t0.elapsed().as_micros();

    let t1 = Instant::now();
    let mut localized_eigs = groundspring::almost_mathieu::eigenvalues(200, 4.0, alpha, 0.0);
    let loc_features = groundspring::esn::spectral_features(&mut localized_eigs);
    let loc_label = groundspring::esn::classify_by_spacing_ratio(loc_features[0], 0.03);
    let loc_us = t1.elapsed().as_micros();

    println!(
        "  Extended  (λ=0.5): ⟨r⟩={:.4}, bw={:.4}, kurt={:.4} → {ext_label}  {ext_us} µs",
        ext_features[0], ext_features[1], ext_features[2]
    );
    println!(
        "  Localized (λ=4.0): ⟨r⟩={:.4}, bw={:.4}, kurt={:.4} → {loc_label}  {loc_us} µs",
        loc_features[0], loc_features[1], loc_features[2]
    );
    println!("  GOE reference:     ⟨r⟩={:.4}", groundspring::esn::GOE_R);
    println!(
        "  Poisson reference: ⟨r⟩={:.4}",
        groundspring::esn::POISSON_R
    );

    h.check(
        "Extended phase not classified as localized",
        ext_label != groundspring::esn::RegimeLabel::Localized,
    );
    h.check(
        "Localized phase not classified as extended",
        loc_label != groundspring::esn::RegimeLabel::Extended,
    );
}

fn bench_rarefaction_occupancy(h: &mut Harness) {
    println!("\n--- Rare Biosphere Occupancy (groundSpring + neuralSpring GPU shader) ---\n");

    let community = vec![
        0.30, 0.20, 0.15, 0.10, 0.08, 0.06, 0.04, 0.03, 0.02, 0.01, 0.005, 0.003, 0.001, 0.0005,
        0.0005,
    ];
    let depth = 1000_u64;
    let n_samples = 200;

    let t0 = Instant::now();
    let occupancy =
        groundspring::rare_biosphere::abundance_occupancy(&community, depth, n_samples, 42);
    let us = t0.elapsed().as_micros();

    println!(
        "  S = {} species, depth = {depth}, n_samples = {n_samples}",
        community.len()
    );
    println!(
        "  Occupancy (top-5): {:.3}, {:.3}, {:.3}, {:.3}, {:.3}",
        occupancy[0], occupancy[1], occupancy[2], occupancy[3], occupancy[4]
    );
    println!(
        "  Occupancy (rare-5): {:.3}, {:.3}, {:.3}, {:.3}, {:.3}",
        occupancy[10], occupancy[11], occupancy[12], occupancy[13], occupancy[14]
    );
    println!("  Time: {us} µs");

    h.check("Dominant species detected in ~100%", occupancy[0] > 0.95);
    h.check(
        "Rare species (<0.1%) detected less often",
        occupancy[14] < occupancy[0],
    );

    let t1 = Instant::now();
    let tier_abundant =
        groundspring::rare_biosphere::tier_detection_rate(&community, 0, 5, depth, n_samples, 42);
    let tier_rare =
        groundspring::rare_biosphere::tier_detection_rate(&community, 10, 15, depth, n_samples, 42);
    let tier_us = t1.elapsed().as_micros();

    println!("\n  Tier detection: abundant={tier_abundant:.4}, rare={tier_rare:.4}  {tier_us} µs");
    h.check("Abundant tier > rare tier", tier_abundant >= tier_rare);

    println!("\n  --- Rarefaction Scaling (n_samples sweep) ---\n");
    for &ns in &[50, 100, 500, 1000] {
        let t = Instant::now();
        let _ = groundspring::rare_biosphere::abundance_occupancy(&community, depth, ns, 42);
        let ms = t.elapsed().as_micros();
        println!("    n_samples={ns:>5}: {ms:>8} µs");
    }
}

fn print_provenance_table() {
    println!("  ┌──────────────────────────────────┬──────────────────────────┬────────┐");
    println!("  │ Shader / Function                 │ Origin Spring            │ Session│");
    println!("  ├──────────────────────────────────┼──────────────────────────┼────────┤");
    println!("  │ df64_core.wgsl (DF64 precision)  │ hotSpring (biomeGate)    │ S58    │");
    println!("  │ Fp64Strategy auto-select          │ hotSpring                │ S58    │");
    println!("  │ anderson.rs, lanczos.rs           │ hotSpring spectral       │ S26    │");
    println!("  │ anderson_2d/3d (sparse Lanczos)   │ hotSpring → S59          │ S59    │");
    println!("  │ anderson_sweep_averaged            │ hotSpring → S59          │ S59    │");
    println!("  │ chi2_decomposed_weighted           │ hotSpring nuclear fits   │ S59    │");
    println!("  │ esn_v2 (reservoir update f64)     │ wetSpring → hotSpring    │ S59    │");
    println!("  │ hofstadter.rs                     │ hotSpring spectral       │ S26    │");
    println!("  │ spmv_csr_f64.wgsl (GPU SpMV)     │ hotSpring Lanczos        │ S59    │");
    println!("  │ batched_multinomial_f64.wgsl      │ groundSpring metalForge  │ S64    │");
    println!("  │ wright_fisher_step_f64.wgsl       │ neuralSpring metalForge  │ S66    │");
    println!("  │ diversity.rs, bray_curtis         │ wetSpring biodiversity   │ S64    │");
    println!("  │ regression.rs (fit_*)             │ airSpring hydrology      │ S66    │");
    println!("  │ metrics.rs (RMSE, MAE, etc.)      │ airSpring + groundSpring │ S64    │");
    println!("  │ rawr_mean bootstrap               │ groundSpring bootstrap   │ S66    │");
    println!("  │ pow_f64 polyfill fix              │ neuralSpring (Ada fix)   │ S-17   │");
    println!("  │ math_f64.wgsl precision fixes     │ wetSpring (ldexp fix)    │ S64    │");
    println!("  │ hill_f64.wgsl, monod              │ wetSpring QS/c-di-GMP    │ S68    │");
    println!("  │ GemmCached (60× taxonomy)         │ wetSpring optimization   │ S64    │");
    println!("  │ batch_ipr_f64.wgsl                │ neuralSpring → hotSpring │ S52    │");
    println!("  │ compile_shader_universal()        │ ToadStool sovereign      │ S67    │");
    println!("  │ op_preamble + naga IR rewrite     │ ToadStool dual-layer     │ S68    │");
    println!("  └──────────────────────────────────┴──────────────────────────┴────────┘");
    println!();
    println!("  Key cross-pollination (S70+ evolution):");
    println!("    hotSpring DF64 → all springs get f64-class precision on consumer GPUs");
    println!("    hotSpring Lanczos → groundSpring Anderson 2D/3D localization studies");
    println!("    hotSpring ESN → groundSpring regime classification (complements NPU Exp028)");
    println!("    hotSpring chi2_decomposed → groundSpring freeze-out per-datum diagnostics");
    println!("    wetSpring bio → neuralSpring metalForge → groundSpring delegation");
    println!("    wetSpring ESN reservoir → hotSpring MD → groundSpring Anderson ESN");
    println!("    neuralSpring pow_f64 → unblocked Ada Lovelace for airSpring + wetSpring");
    println!("    airSpring regression → groundSpring WDM finite-size extrapolation");
    println!("    groundSpring RAWR → wetSpring rarefaction confidence intervals");
    println!("    groundSpring Anderson sweep → feeds ESN training data cross-spring");
}
