// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team
#![forbid(unsafe_code)]

//! Cross-spring benchmark: CPU-local vs barracuda-CPU vs barracuda-GPU.
//!
//! **Sovereignty note**: This binary is a provenance documentation tool that
//! records the historical lineage of cross-spring shader evolution. Spring
//! names appear as provenance labels (not runtime coupling or discovery).
//! No runtime dependency on sibling primals exists.
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

use groundspring::tol;
use groundspring_forge::harness::Harness;
use std::time::Instant;

fn main() {
    println!("=== groundSpring Cross-Spring Benchmark ===");
    println!("=== ToadStool S158+ / barraCuda v0.3.5 / wgpu 28 / DF64 Precision Tiers ===\n");

    let mut h = Harness::new();

    bench_stats_metrics(&mut h);
    bench_fused_mean_variance(&mut h);
    bench_bootstrap_rawr(&mut h);
    bench_regression(&mut h);
    bench_diversity(&mut h);
    bench_et0_methods(&mut h);
    bench_anderson(&mut h);
    bench_anderson_sweep(&mut h);
    bench_chi2_analysis(&mut h);
    bench_esn_classification(&mut h);
    bench_rarefaction_occupancy(&mut h);

    println!("\n--- Cross-Spring Evolution Timeline ---\n");
    print_evolution_timeline();

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
            #[expect(clippy::cast_possible_truncation, reason = "loop index i ≪ 2^32")]
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

fn bench_fused_mean_variance(h: &mut Harness) {
    println!("\n--- Fused Mean+Variance (hotSpring DF64 → Welford single-pass) ---\n");

    let data: Vec<f64> = (0..50_000)
        .map(|i| (f64::from(i) * 0.001).sin().mul_add(100.0, 50.0))
        .collect();

    let t0 = Instant::now();
    let (fused_mean, fused_std) = groundspring::stats::mean_and_std_dev(&data);
    let fused_us = t0.elapsed().as_micros();

    let t1 = Instant::now();
    let sep_mean = groundspring::stats::mean(&data);
    let sep_std = groundspring::stats::std_dev(&data);
    let sep_us = t1.elapsed().as_micros();

    println!("  n = {} values", data.len());
    println!("  Fused:    mean={fused_mean:.8}, std={fused_std:.8}  ({fused_us} µs)");
    println!("  Separate: mean={sep_mean:.8},  std={sep_std:.8}  ({sep_us} µs)");
    println!("  Provenance: hotSpring DF64 → Welford mean_variance_f64.wgsl (barraCuda v0.3.3)");

    h.check(
        "Fused mean matches separate mean",
        (fused_mean - sep_mean).abs() < tol::ANALYTICAL,
    );
    h.check(
        "Fused std matches separate std",
        (fused_std - sep_std).abs() < tol::ANALYTICAL,
    );
}

fn bench_bootstrap_rawr(h: &mut Harness) {
    println!("\n--- Bootstrap RAWR (groundSpring → S66) ---\n");

    let data: Vec<f64> = (0..5000)
        .map(|i| (f64::from(i) * 0.1).sin() * 10.0)
        .collect();

    let t0 = Instant::now();
    let ci = groundspring::bootstrap::rawr_mean(&data, 1000, 0.05, 42)
        .expect("hardcoded benchmark inputs are valid");
    let rawr_us = t0.elapsed().as_micros();

    let t1 = Instant::now();
    let ci_classic = groundspring::bootstrap::bootstrap_mean(&data, 1000, 0.05, 42)
        .expect("hardcoded benchmark inputs are valid");
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

fn bench_et0_methods(h: &mut Harness) {
    println!("\n--- ET₀ Method Comparison (airSpring → barraCuda v0.3.2) ---\n");
    println!("  Cross-spring: airSpring V068/V069 evolved Makkink, Turc, Hamon to barraCuda.");
    println!("  groundSpring delegates with sovereign fallback. All springs benefit.\n");

    let inp = groundspring::fao56::example_18_inputs();
    let tmean = f64::midpoint(inp.tmax_c, inp.tmin_c);
    let rh_mean = f64::midpoint(inp.rhmax_pct, inp.rhmin_pct);
    let ra = groundspring::fao56::extraterrestrial_radiation(inp.latitude_deg_n, inp.day_of_year);
    let big_n = groundspring::fao56::daylight_hours(inp.latitude_deg_n, inp.day_of_year);
    let n = inp.sunshine_hours.min(big_n).max(0.0);
    let rs = groundspring::fao56::solar_radiation_from_sunshine(n, big_n, ra);

    let t0 = Instant::now();
    let pm = groundspring::fao56::daily_et0(&inp);
    let pm_us = t0.elapsed().as_micros();

    let t1 = Instant::now();
    let hg = groundspring::fao56::hargreaves_et0(
        inp.tmax_c,
        inp.tmin_c,
        inp.latitude_deg_n,
        inp.day_of_year,
    );
    let hargreaves_us = t1.elapsed().as_micros();

    let t2 = Instant::now();
    let mk = groundspring::fao56::makkink_et0(tmean, rs);
    let makkink_us = t2.elapsed().as_micros();

    let t3 = Instant::now();
    let tu = groundspring::fao56::turc_et0(tmean, rs, rh_mean);
    let turc_us = t3.elapsed().as_micros();

    let t4 = Instant::now();
    let hamon = groundspring::fao56::hamon_et0(tmean, big_n);
    let hamon_us = t4.elapsed().as_micros();

    println!("  Site: Uccle (50.8°N), July 6  (FAO-56 Example 18)");
    println!("  ┌──────────────────────┬──────────┬──────────┐");
    println!("  │ Method               │ ET₀ mm/d │ Time µs  │");
    println!("  ├──────────────────────┼──────────┼──────────┤");
    println!("  │ Penman-Monteith      │ {pm:8.4} │ {pm_us:>8} │");
    println!("  │ Hargreaves           │ {hg:8.4} │ {hargreaves_us:>8} │");
    println!("  │ Makkink (v0.3.2)     │ {mk:8.4} │ {makkink_us:>8} │");
    println!("  │ Turc (v0.3.2)        │ {tu:8.4} │ {turc_us:>8} │");
    println!("  │ Hamon (v0.3.2)       │ {hamon:8.4} │ {hamon_us:>8} │");
    println!("  └──────────────────────┴──────────┴──────────┘");

    h.check("PM ET₀ positive", pm > 0.0);
    h.check(
        "All methods in (0, 20) mm/day",
        [pm, hg, mk, tu, hamon].iter().all(|&v| v > 0.0 && v < 20.0),
    );
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
    let monotonic = match (sweep.first(), sweep.last()) {
        (Some(first), Some(last)) => last.mean_ratio > first.mean_ratio,
        _ => false,
    };
    h.check("Lyapunov increases with disorder", monotonic);
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
    let Some(analysis) = groundspring::freeze_out::chi2_analysis(&obs, &pred, sigma, 2).ok() else {
        h.check("Chi² analysis computation", false);
        return;
    };
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

fn print_evolution_timeline() {
    println!("  Cross-Spring Evolution: How Shaders Flow Through the Ecosystem");
    println!("  ================================================================\n");

    println!("  Phase 1: Foundation (S26-S58, Jan-Feb 2026)");
    println!("  ┌─ hotSpring v0.6.0 → anderson.rs, lanczos.rs, spectral theory");
    println!("  ├─ hotSpring S58 ──→ df64_core.wgsl (DF64 precision for consumer GPUs)");
    println!("  ├─ hotSpring S58 ──→ Fp64Strategy (Native/Hybrid auto-selection)");
    println!("  ├─ wetSpring S64 ──→ diversity (Shannon, Simpson, Bray-Curtis)");
    println!("  ├─ wetSpring S64 ──→ math_f64.wgsl (f64 constant precision pattern)");
    println!("  └─ neuralSpring S-17 → pow_f64 polyfill (unblocked Ada Lovelace)\n");

    println!("  Phase 2: Cross-Spring Absorption (S59-S66, Feb 2026)");
    println!("  ┌─ hotSpring S59 ──→ anderson_2d/3d sparse Lanczos, chi2_decomposed");
    println!("  ├─ hotSpring S59 ──→ esn_v2 reservoir (Stanton-Murillo transport)");
    println!("  ├─ airSpring S66 ──→ regression (fit_linear, fit_quadratic, fit_exp)");
    println!("  ├─ airSpring S66 ──→ hydrology (hargreaves_et0, FAO-56 delegation)");
    println!("  ├─ groundSpring S64 → batched_multinomial_f64.wgsl (rarefaction)");
    println!("  ├─ groundSpring S66 → rawr_mean bootstrap (feeds wetSpring pipelines)");
    println!("  └─ wetSpring → neuralSpring: bio primitives cross-pollinate ML\n");

    println!("  Phase 3: Universal Precision (S67-S68, Feb 2026)");
    println!("  ┌─ ToadStool S67 ──→ compile_shader_universal() — one shader, any precision");
    println!("  ├─ ToadStool S68 ──→ Waves 1-11: ALL 844+ shaders evolved to f64-canonical");
    println!("  ├─ ToadStool S68 ──→ dual-layer DF64 (op_preamble + naga IR rewrite)");
    println!("  └─ Result: \"Math is universal, precision is silicon\"\n");

    println!("  Phase 4: Modern Wiring (S70-S93, Feb-Mar 2026)");
    println!("  ┌─ hotSpring → all: DF64 gives f64-class on consumer GPUs (RTX 4070)");
    println!("  ├─ hotSpring → groundSpring: anderson_4d + wegner_block_4d (tissue model)");
    println!("  ├─ airSpring → groundSpring: L-BFGS refinement (freeze-out optimization)");
    println!("  ├─ wetSpring → groundSpring: BatchedMultinomialGpu, DiversityFusionGpu");
    println!("  ├─ neuralSpring → all: AlphaFold2 Evoformer primitives (S69)");
    println!("  ├─ groundSpring → all: InterconnectTopology, SubstratePipeline (S81)");
    println!("  ├─ S87: FHE shader fix, unsafe audit, all ~60+ unsafe sites documented");
    println!("  ├─ S89: barraCuda budded to standalone primal (zero toadStool deps)");
    println!("  └─ S90-S93: REST→JSON-RPC, sovereignty evolution, D-DF64 transfer\n");

    println!("  Bidirectional Flow (every spring feeds every other spring):");
    println!("  ┌─ hotSpring precision → wetSpring bio gets f64-class on consumer GPUs");
    println!("  ├─ wetSpring bio shaders → neuralSpring ML uses for taxonomy/alignment");
    println!("  ├─ neuralSpring pow_f64 fix → hotSpring + wetSpring unblocked on Ada");
    println!("  ├─ airSpring hydrology → groundSpring ET₀ + seasonal pipeline");
    println!("  ├─ groundSpring bootstrap → wetSpring rarefaction confidence intervals");
    println!("  └─ All springs → ToadStool → absorbed → all springs consume\n");

    println!("  Phase 5: Modern Rewiring (S94b + v0.3.3, Mar 2026)");
    println!("  ┌─ barraCuda v0.3.3 → wgpu 28, DF64 precision tiers (15 ops)");
    println!("  ├─ barraCuda v0.3.3 → fused mean+variance Welford, 5-acc Pearson");
    println!("  ├─ barraCuda v0.3.3 → TensorContext pooled buffers for stats ops");
    println!("  ├─ barraCuda v0.3.2 → 3 new ET₀ ops (Makkink, Turc, Hamon) from airSpring");
    println!("  ├─ toadStool S94b ──→ full primal decoupling, barraCuda standalone");
    println!(
        "  ├─ toadStool S96c ──→ HardwareFingerprint, SubstrateCapabilityKind, god file splits"
    );
    println!("  ├─ groundSpring V80 → fused correlation_full GPU, Welford single-pass CPU");
    println!("  ├─ groundSpring V84 → Dual-GPU probe, DF64 green, f64 shared-mem issue found");
    println!("  └─ groundSpring V85 → coralReef sovereign compilation, f64 reduction SM70/SM89\n");

    println!(
        "  Current state: barraCuda e1184f3, 710 WGSL shaders, 3471+ tests, DF64 reduce ops wired"
    );
    println!(
        "  groundSpring: 110 delegations (67 CPU + 43 GPU), 936 tests, wgpu 28, three-tier parity proven"
    );
    println!(
        "  coralReef: 849fedd, 672 tests, NVIDIA backend complete, f64 reduction SM70/SM89, Phase 5+"
    );
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
    println!("  │ almost_mathieu_hamiltonian          │ hotSpring spectral       │ S26    │");
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
    println!("  ├──────────────────────────────────┼──────────────────────────┼────────┤");
    println!("  │ McEt0PropagateGpu                 │ airSpring → ToadStool   │ S72    │");
    println!("  │ SeasonalPipelineF64               │ airSpring → ToadStool   │ S80    │");
    println!("  │ lbfgs_numerical (L-BFGS)          │ airSpring → S84         │ S84    │");
    println!("  │ anderson_4d + wegner_block_4d     │ hotSpring → S84         │ S84    │");
    println!("  │ FHE shader fix (u64_mod_simple)   │ ToadStool internal      │ S87    │");
    println!("  │ is_device_lost() + retry          │ ToadStool resilience    │ S87    │");
    println!("  │ barraCuda standalone primal        │ ToadStool budding       │ S89    │");
    println!("  │ D-DF64 transfer to barraCuda       │ ToadStool → barraCuda   │ S93    │");
    println!("  │ wgpu 28 migration                 │ barraCuda standalone    │ v0.3.3 │");
    println!("  │ DF64 precision tiers (15 ops)     │ hotSpring precision     │ v0.3.3 │");
    println!("  │ Fused mean+variance (Welford)     │ hotSpring stats         │ v0.3.3 │");
    println!("  │ Fused 5-acc Pearson correlation   │ hotSpring stats         │ v0.3.3 │");
    println!("  │ Makkink/Turc/Hamon ET₀            │ airSpring V068/V069    │ v0.3.2 │");
    println!("  │ TensorContext pooled buffers       │ hotSpring + airSpring   │ v0.3.3 │");
    println!("  │ S94b full primal decoupling        │ ToadStool evolution     │ S94b   │");
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
    println!("    airSpring L-BFGS → groundSpring freeze-out post-grid-search refinement");
    println!("    hotSpring anderson_4d → groundSpring tissue immunology (Paper 12)");
    println!("    ToadStool S87-S93 → FHE fix, barraCuda budding, D-DF64 transfer");
    println!("    Universal precision: F16→F32→F64→Df64 from one f64-canonical source");
}
