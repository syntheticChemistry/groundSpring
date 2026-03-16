// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Validation binary for Experiments 033-034: Tissue Anderson Localization.
//!
//! Exp 033: Cytokine Anderson lattice — multi-layer skin model with
//! quasi-2D epidermis coupled to 3D dermis, barrier disruption sweep,
//! and dimensional promotion quantification.
//!
//! Exp 034: Geometry-aware drug scoring — Anderson-augmented repurposing
//! score with tissue penetration factor for the AD drug panel.
//!
//! Reference: Paper 12 — Anderson Localization in Immunological Signaling

use groundspring::tissue_anderson::{
    DeliveryRoute, DrugCandidate, SkinLayer, TissueState, ad_drug_panel, barrier_disruption_sweep,
    dimensional_duality_sweep, effective_disorder, geometry_drug_score, healthy_dermis,
    healthy_epidermis, inflamed_dermis, pielou_evenness, score_drug_panel, simulate_tissue,
};
use groundspring::validate::ValidationHarness;

// ─── Tissue Anderson Thresholds ──────────────────────────────────────────────
//
// Provenance: Paper 12 — "Anderson Localization in Immunological Signaling"
// (Strandgate 2026). Analytical invariants from Anderson theory extended to
// multi-compartment tissue geometry with known dimensionality (d=2 epidermis,
// d=3 dermis).
//
// These are structural invariants, not empirical fits — they follow from:
// - d=2: all states localize for any disorder (Abrahams et al. 1979)
// - d=3: Anderson transition at W_c ≈ 16.5 (Slevin & Ohtsuki 1999)
// - Barrier disruption: continuous breach_fraction sweep [0,1]

/// Pielou evenness lower bound for inflamed dermis.
///
/// High cell-type diversity (many immune infiltrates) produces J' > 0.8.
/// Healthy epidermis (dominated by keratinocytes) has lower evenness.
const MIN_INFLAMED_EVENNESS: f64 = 0.8;

/// System effective dimensionality upper bound when barrier is intact.
///
/// Epidermis (d=2) coupled to dermis (d=3) with intact barrier should
/// yield `d_eff` < 2.5 (barrier limits signal propagation to quasi-2D).
const MAX_HEALTHY_D_EFF: f64 = 2.5;

/// 3D Anderson transition critical disorder (Slevin & Ohtsuki 1999).
///
/// Below `W_c`, states in 3D are extended; above, localized. Inflamed
/// dermis should remain below this threshold (signals still propagate).
const ANDERSON_3D_W_C: f64 = 16.5;

/// Barrier disruption sweep: minimum breach fraction for transition.
///
/// The barrier → breached transition should occur between these bounds,
/// reflecting the gradual loss of tight-junction integrity in the
/// stratum corneum (Paper 12 §3.2).
const MIN_BARRIER_TRANSITION: f64 = 0.4;
/// Barrier disruption sweep: maximum breach fraction for transition.
const MAX_BARRIER_TRANSITION: f64 = 0.8;

/// Minimum topical penetration factor for large biologics (intact barrier).
///
/// Monoclonal antibodies (~150 kDa) cannot cross intact stratum corneum;
/// penetration factor should be < 0.15 (Paper 12 Table 3).
const MAX_TOPICAL_MAB_PENETRATION: f64 = 0.15;

/// Minimum systemic penetration for small molecules reaching dermis.
const MIN_SYSTEMIC_PENETRATION: f64 = 0.8;

/// Minimum composite drug score for a "good candidate" (systemic delivery).
const MIN_GOOD_COMPOSITE_SCORE: f64 = 0.5;

fn main() {
    std::process::exit(run());
}

fn run() -> i32 {
    let mut h = ValidationHarness::stdout("Exp 033-034: Tissue Anderson (Paper 12)");

    println!("{}", "=".repeat(72));
    println!("groundSpring Rust Validation: Tissue Anderson (Paper 12)");
    println!("  Source: Exp 033–034 — Anderson Localization in Immunological Signaling");
    println!("  Provenance: analytical — tissue geometry constraints from Paper 12");
    println!("  Baseline: Structural invariants from Anderson theory extended to");
    println!("            multi-compartment tissue with known dimensionality (d=2, d=3).");
    println!("  Note: No benchmark JSON — validates theoretical predictions:");
    println!("        W_epi < W_derm (fewer cell types), barrier transition in [0.4, 0.8],");
    println!("        dimensional duality (Paper 06 ↔ Paper 12), drug geometry scoring.");
    println!("{}", "=".repeat(72));

    validate_compartment_disorder(&mut h);
    validate_pielou_evenness(&mut h);
    validate_healthy_skin(&mut h);
    validate_inflamed_dermis(&mut h);
    validate_barrier_disruption(&mut h);
    validate_dimensional_duality(&mut h);
    validate_drug_scoring_systemic(&mut h);
    validate_drug_scoring_topical(&mut h);
    validate_ad_drug_panel(&mut h);
    validate_barrier_effect_on_drug(&mut h);

    h.summary()
}

fn validate_compartment_disorder(h: &mut ValidationHarness) {
    println!("\n--- Part 1: Compartment Disorder (W) ---\n");

    let epi = healthy_epidermis();
    let w_epi = effective_disorder(&epi.cell_composition);

    let derm = healthy_dermis();
    let w_derm = effective_disorder(&derm.cell_composition);

    let inflamed = inflamed_dermis();
    let w_inflamed = effective_disorder(&inflamed.cell_composition);

    println!("  Healthy epidermis W: {w_epi:.4}");
    println!("  Healthy dermis W:    {w_derm:.4}");
    println!("  Inflamed dermis W:   {w_inflamed:.4}");

    h.check_true("Epidermis has lower disorder than dermis", w_epi < w_derm);
    h.check_true(
        "Inflamed dermis has higher disorder than healthy",
        w_inflamed > w_derm,
    );
    h.check_max("Epidermis W < 1.0 (low heterogeneity)", w_epi, 1.0);
}

fn validate_pielou_evenness(h: &mut ValidationHarness) {
    println!("\n--- Part 2: Pielou Evenness (J') ---\n");

    let epi = healthy_epidermis();
    let j_epi = pielou_evenness(&epi.cell_composition);

    let inflamed = inflamed_dermis();
    let j_inflamed = pielou_evenness(&inflamed.cell_composition);

    println!("  Healthy epidermis J': {j_epi:.4}");
    println!("  Inflamed dermis J':   {j_inflamed:.4}");

    h.check_true(
        "Epidermis J' < inflamed dermis J' (less even)",
        j_epi < j_inflamed,
    );
    h.check_min(
        "Inflamed dermis J' > 0.8 (high evenness)",
        j_inflamed,
        MIN_INFLAMED_EVENNESS,
    );
}

fn validate_healthy_skin(h: &mut ValidationHarness) {
    println!("\n--- Part 3: Healthy Skin Simulation ---\n");

    let epi = healthy_epidermis();
    let derm = healthy_dermis();
    let result = simulate_tissue(&[epi, derm], 20, 42);

    println!("  Epidermis γ: {:.6}", result.gamma_per_compartment[0]);
    println!("  Dermis γ:    {:.6}", result.gamma_per_compartment[1]);
    println!("  Epidermis ξ: {:.2}", result.xi_per_compartment[0]);
    println!("  Dermis ξ:    {:.2}", result.xi_per_compartment[1]);
    println!("  Barrier intact: {}", !result.barrier_breached);
    println!("  System d_eff: {:.1}", result.d_eff_system);

    h.check_true("Healthy skin barrier intact", !result.barrier_breached);
    h.check_min(
        "Epidermis has positive γ (signals localize in 2D)",
        result.gamma_per_compartment[0],
        0.0,
    );
    h.check_min(
        "Dermis has positive γ",
        result.gamma_per_compartment[1],
        0.0,
    );
    h.check_max(
        "System d_eff < 2.5 (barrier limits)",
        result.d_eff_system,
        MAX_HEALTHY_D_EFF,
    );
}

fn validate_inflamed_dermis(h: &mut ValidationHarness) {
    println!("\n--- Part 4: Inflamed Dermis ---\n");

    let derm = inflamed_dermis();
    let result = simulate_tissue(&[derm], 20, 42);

    println!(
        "  Inflamed dermis γ: {:.6}",
        result.gamma_per_compartment[0]
    );
    println!("  Inflamed dermis ξ: {:.2}", result.xi_per_compartment[0]);
    println!("  Inflamed dermis W: {:.2}", result.w_per_compartment[0]);

    h.check_max(
        "Inflamed dermis W < W_c=16.5 (signals still propagate in 3D)",
        result.w_per_compartment[0],
        ANDERSON_3D_W_C,
    );
    h.check_true(
        "Inflamed dermis has finite ξ",
        result.xi_per_compartment[0].is_finite(),
    );
}

fn validate_barrier_disruption(h: &mut ValidationHarness) {
    println!("\n--- Part 5: Barrier Disruption Sweep ---\n");

    let sweep = barrier_disruption_sweep(11, 10, 42);

    for p in &sweep {
        println!(
            "  breach={:.1}  d_eff={:.2}  γ_epi={:.4}  ξ_epi={:.1}  breached={}",
            p.breach_fraction,
            p.d_eff_epidermis,
            p.gamma_epidermis,
            p.xi_epidermis,
            p.barrier_breached,
        );
    }

    h.check_true("Sweep has 11 points", sweep.len() == 11);
    h.check_true("Healthy endpoint not breached", !sweep[0].barrier_breached);
    h.check_true(
        "Fully disrupted endpoint breached",
        sweep[10].barrier_breached,
    );

    let transition_idx = sweep.iter().position(|p| p.barrier_breached);
    if let Some(idx) = transition_idx {
        println!(
            "\n  Barrier transition at breach_fraction={:.2}",
            sweep[idx].breach_fraction
        );
        h.check_range(
            "Transition between 0.4 and 0.8",
            sweep[idx].breach_fraction,
            MIN_BARRIER_TRANSITION,
            MAX_BARRIER_TRANSITION,
        );
    } else {
        h.check_true("Barrier transition found", false);
    }
}

fn validate_dimensional_duality(h: &mut ValidationHarness) {
    println!("\n--- Part 6: Dimensional Duality (Paper 06 ↔ Paper 12) ---\n");

    let sweep = dimensional_duality_sweep(11, 10, 42);

    for p in &sweep {
        println!(
            "  param={:+.2}  d_eff={:.2}  γ={:.4}  ξ={:.1}  regime={:<10}  context={}",
            p.parameter, p.d_eff, p.gamma, p.xi, p.regime, p.context,
        );
    }

    h.check_true("Duality sweep has 11 points", sweep.len() == 11);
    h.check_true(
        "Negative param → collapse context",
        sweep[0].context.contains("collapse"),
    );
    h.check_true(
        "Positive param → promotion context",
        sweep[10].context.contains("promotion"),
    );
    h.check_true(
        "All γ positive (1D always localizes)",
        sweep.iter().all(|p| p.gamma > 0.0),
    );
}

fn validate_drug_scoring_systemic(h: &mut ValidationHarness) {
    println!("\n--- Part 7: Drug Scoring — Systemic Delivery ---\n");

    let tissue = TissueState {
        barrier_disruption: 0.0,
        d_eff_epidermis: 2.0,
        w_dermis: 3.0,
    };

    let oclacitinib = DrugCandidate {
        name: "Oclacitinib".into(),
        pathway_score: 0.92,
        molecular_weight_da: 337.0,
        delivery: DeliveryRoute::Systemic,
        target_compartment: SkinLayer::Dermis,
    };
    let score = geometry_drug_score(&oclacitinib, &tissue);

    println!("  Oclacitinib (systemic, small molecule, dermis target):");
    println!("    pathway:     {:.3}", score.pathway_score);
    println!("    penetration: {:.3}", score.penetration_factor);
    println!("    anderson:    {:.3}", score.anderson_factor);
    println!("    composite:   {:.3}", score.composite_score);
    println!("    reaches:     {}", score.reaches_target);

    h.check_true(
        "Systemic small molecule reaches dermis",
        score.reaches_target,
    );
    h.check_min(
        "High penetration for systemic delivery",
        score.penetration_factor,
        MIN_SYSTEMIC_PENETRATION,
    );
    h.check_min(
        "Composite score > 0.5 (good candidate)",
        score.composite_score,
        MIN_GOOD_COMPOSITE_SCORE,
    );
}

fn validate_drug_scoring_topical(h: &mut ValidationHarness) {
    println!("\n--- Part 8: Drug Scoring — Topical mAb vs Small Molecule ---\n");

    let tissue = TissueState {
        barrier_disruption: 0.0,
        d_eff_epidermis: 2.0,
        w_dermis: 3.0,
    };

    let topical_mab = DrugCandidate {
        name: "Topical mAb".into(),
        pathway_score: 0.95,
        molecular_weight_da: 150_000.0,
        delivery: DeliveryRoute::Topical,
        target_compartment: SkinLayer::Dermis,
    };

    let topical_small = DrugCandidate {
        name: "Crisaborole (topical)".into(),
        pathway_score: 0.55,
        molecular_weight_da: 251.0,
        delivery: DeliveryRoute::Topical,
        target_compartment: SkinLayer::Epidermis,
    };

    let score_mab = geometry_drug_score(&topical_mab, &tissue);
    let score_small = geometry_drug_score(&topical_small, &tissue);

    println!("  Topical mAb → dermis:");
    println!(
        "    penetration: {:.3}  composite: {:.3}",
        score_mab.penetration_factor, score_mab.composite_score
    );
    println!("  Topical small molecule → epidermis:");
    println!(
        "    penetration: {:.3}  composite: {:.3}",
        score_small.penetration_factor, score_small.composite_score
    );

    h.check_max(
        "Topical mAb has low penetration (intact barrier)",
        score_mab.penetration_factor,
        MAX_TOPICAL_MAB_PENETRATION,
    );
    h.check_true(
        "Topical small molecule has better penetration",
        score_small.penetration_factor > score_mab.penetration_factor,
    );
}

fn validate_ad_drug_panel(h: &mut ValidationHarness) {
    println!("\n--- Part 9: Full AD Drug Panel Scoring ---\n");

    let tissue = TissueState {
        barrier_disruption: 0.2,
        d_eff_epidermis: 2.2,
        w_dermis: 4.0,
    };

    let panel = ad_drug_panel();
    let scores = score_drug_panel(&panel, &tissue);

    for (drug, score) in panel.iter().zip(scores.iter()) {
        println!(
            "  {:<30}  path={:.2}  pen={:.2}  and={:.2}  comp={:.3}  reach={}",
            drug.name,
            score.pathway_score,
            score.penetration_factor,
            score.anderson_factor,
            score.composite_score,
            if score.reaches_target { "YES" } else { "NO " },
        );
    }

    h.check_true("Panel has 6 drugs", scores.len() == 6);
    h.check_true(
        "All composite scores in [0, 1]",
        scores
            .iter()
            .all(|s| (0.0..=1.0).contains(&s.composite_score)),
    );

    let systemic_reach_count = scores.iter().filter(|s| s.reaches_target).count();
    h.check_true(
        "At least 4 drugs reach target (systemic delivery)",
        systemic_reach_count >= 4,
    );
}

fn validate_barrier_effect_on_drug(h: &mut ValidationHarness) {
    println!("\n--- Part 10: Barrier Effect on Drug Scoring ---\n");

    let tissue_intact = TissueState {
        barrier_disruption: 0.0,
        d_eff_epidermis: 2.0,
        w_dermis: 3.0,
    };

    let tissue_disrupted = TissueState {
        barrier_disruption: 0.8,
        d_eff_epidermis: 2.8,
        w_dermis: 5.0,
    };

    let topical_small = DrugCandidate {
        name: "Crisaborole".into(),
        pathway_score: 0.55,
        molecular_weight_da: 251.0,
        delivery: DeliveryRoute::Topical,
        target_compartment: SkinLayer::Dermis,
    };

    let score_intact = geometry_drug_score(&topical_small, &tissue_intact);
    let score_disrupted = geometry_drug_score(&topical_small, &tissue_disrupted);

    println!(
        "  Crisaborole → dermis (intact barrier):    pen={:.3}",
        score_intact.penetration_factor
    );
    println!(
        "  Crisaborole → dermis (disrupted barrier): pen={:.3}",
        score_disrupted.penetration_factor
    );

    h.check_true(
        "Disrupted barrier increases topical penetration",
        score_disrupted.penetration_factor > score_intact.penetration_factor,
    );
    h.check_true(
        "Anderson factor accounts for higher disorder",
        score_disrupted.anderson_factor <= score_intact.anderson_factor,
    );
}
