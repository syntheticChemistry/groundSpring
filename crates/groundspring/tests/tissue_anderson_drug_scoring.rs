// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Property-based tests for [`groundspring::tissue_anderson`] drug scoring
//! (Exp 034 geometry-aware repurposing scores).

#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use proptest::prelude::*;

use groundspring::tissue_anderson::{
    DeliveryRoute, DrugCandidate, SkinLayer, TissueState, geometry_drug_score, score_drug_panel,
};

// ---------------------------------------------------------------------------
// strategies
// ---------------------------------------------------------------------------

fn skin_layer() -> impl Strategy<Value = SkinLayer> {
    prop_oneof![
        Just(SkinLayer::StratumCorneum),
        Just(SkinLayer::Epidermis),
        Just(SkinLayer::Dermis),
    ]
}

fn delivery_route() -> impl Strategy<Value = DeliveryRoute> {
    prop_oneof![Just(DeliveryRoute::Systemic), Just(DeliveryRoute::Topical)]
}

fn tissue_state() -> impl Strategy<Value = TissueState> {
    (0.0_f64..=1.0, 1.0_f64..=4.0, 0.01_f64..=50.0).prop_map(
        |(barrier_disruption, d_eff_epidermis, w_dermis)| TissueState {
            barrier_disruption,
            d_eff_epidermis,
            w_dermis,
        },
    )
}

fn drug_candidate() -> impl Strategy<Value = DrugCandidate> {
    (
        delivery_route(),
        skin_layer(),
        0.0_f64..=1.0,
        1.0_f64..=200_000.0,
    )
        .prop_map(
            |(delivery, target_compartment, pathway_score, molecular_weight_da)| DrugCandidate {
                name: "prop-test".into(),
                pathway_score,
                molecular_weight_da,
                delivery,
                target_compartment,
            },
        )
}

// ---------------------------------------------------------------------------
// properties
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn tissue_anderson_drug_scores_non_negative(drug in drug_candidate(), tissue in tissue_state()) {
        let score = geometry_drug_score(&drug, &tissue);
        prop_assert!(score.pathway_score >= 0.0);
        prop_assert!(score.penetration_factor >= 0.0);
        prop_assert!(score.anderson_factor > 0.0);
        prop_assert!(score.composite_score >= 0.0);
    }

    #[test]
    fn tissue_anderson_drug_penetration_in_unit_interval(drug in drug_candidate(), tissue in tissue_state()) {
        let score = geometry_drug_score(&drug, &tissue);
        prop_assert!(score.penetration_factor <= 1.0);
    }

    #[test]
    fn tissue_anderson_drug_composite_within_bounds(drug in drug_candidate(), tissue in tissue_state()) {
        let score = geometry_drug_score(&drug, &tissue);
        // pathway ∈ [0, 1]; penetration ∈ [0, 1]; anderson ∈ (0, 1].
        prop_assert!(score.composite_score <= 1.0);
    }

    #[test]
    fn tissue_anderson_drug_composite_is_product(drug in drug_candidate(), tissue in tissue_state()) {
        let score = geometry_drug_score(&drug, &tissue);
        let expected = score.pathway_score * score.penetration_factor * score.anderson_factor;
        prop_assert!((score.composite_score - expected).abs() < 1e-12);
    }

    #[test]
    fn tissue_anderson_drug_scoring_deterministic(drug in drug_candidate(), tissue in tissue_state()) {
        let a = geometry_drug_score(&drug, &tissue);
        let b = geometry_drug_score(&drug, &tissue);
        prop_assert_eq!(a.pathway_score, b.pathway_score);
        prop_assert_eq!(a.penetration_factor, b.penetration_factor);
        prop_assert_eq!(a.anderson_factor, b.anderson_factor);
        prop_assert_eq!(a.composite_score, b.composite_score);
        prop_assert_eq!(a.reaches_target, b.reaches_target);
    }

    #[test]
    fn tissue_anderson_drug_zero_pathway_zero_composite(
        delivery in delivery_route(),
        target in skin_layer(),
        mw in 1.0_f64..200_000.0,
        tissue in tissue_state(),
    ) {
        let drug = DrugCandidate {
            name: "zero-pathway".into(),
            pathway_score: 0.0,
            molecular_weight_da: mw,
            delivery,
            target_compartment: target,
        };
        let score = geometry_drug_score(&drug, &tissue);
        prop_assert_eq!(score.composite_score, 0.0);
    }

    #[test]
    fn tissue_anderson_drug_reaches_target_threshold(
        drug in drug_candidate(),
        tissue in tissue_state(),
    ) {
        let score = geometry_drug_score(&drug, &tissue);
        prop_assert_eq!(score.reaches_target, score.penetration_factor > 0.3);
    }

    #[test]
    fn tissue_anderson_drug_pathway_monotonic(
        delivery in delivery_route(),
        target in skin_layer(),
        p_low in 0.0_f64..=0.5,
        p_high in 0.5_f64..=1.0,
        mw in 1.0_f64..200_000.0,
        tissue in tissue_state(),
    ) {
        let low = DrugCandidate {
            name: "low-pathway".into(),
            pathway_score: p_low,
            molecular_weight_da: mw,
            delivery,
            target_compartment: target,
        };
        let high = DrugCandidate {
            name: "high-pathway".into(),
            pathway_score: p_high,
            molecular_weight_da: mw,
            delivery,
            target_compartment: target,
        };
        let s_low = geometry_drug_score(&low, &tissue);
        let s_high = geometry_drug_score(&high, &tissue);
        prop_assert!(s_high.composite_score >= s_low.composite_score);
    }

    #[test]
    fn tissue_anderson_drug_barrier_disruption_monotonic(
        mw in 1.0_f64..200_000.0,
        pathway in 0.0_f64..=1.0,
        d_low in 0.0_f64..=0.4,
        d_high in 0.6_f64..=1.0,
        d_eff in 1.0_f64..=4.0,
        w_dermis in 0.01_f64..=50.0,
    ) {
        let drug = DrugCandidate {
            name: "topical-dermis".into(),
            pathway_score: pathway,
            molecular_weight_da: mw,
            delivery: DeliveryRoute::Topical,
            target_compartment: SkinLayer::Dermis,
        };
        let tissue_low = TissueState {
            barrier_disruption: d_low,
            d_eff_epidermis: d_eff,
            w_dermis,
        };
        let tissue_high = TissueState {
            barrier_disruption: d_high,
            d_eff_epidermis: d_eff,
            w_dermis,
        };
        let p_low = geometry_drug_score(&drug, &tissue_low).penetration_factor;
        let p_high = geometry_drug_score(&drug, &tissue_high).penetration_factor;
        prop_assert!(p_high >= p_low);
    }

    #[test]
    fn tissue_anderson_drug_mw_penetration_monotonic(
        delivery in Just(DeliveryRoute::Topical),
        target in prop_oneof![Just(SkinLayer::Epidermis), Just(SkinLayer::Dermis)],
        mw_small in 1.0_f64..500.0,
        mw_large in 10_001.0_f64..200_000.0,
        tissue in tissue_state(),
    ) {
        let small = DrugCandidate {
            name: "small-molecule".into(),
            pathway_score: 0.8,
            molecular_weight_da: mw_small,
            delivery,
            target_compartment: target,
        };
        let large = DrugCandidate {
            name: "large-molecule".into(),
            pathway_score: 0.8,
            molecular_weight_da: mw_large,
            delivery,
            target_compartment: target,
        };
        let p_small = geometry_drug_score(&small, &tissue).penetration_factor;
        let p_large = geometry_drug_score(&large, &tissue).penetration_factor;
        prop_assert!(p_small >= p_large);
    }

    #[test]
    fn tissue_anderson_drug_panel_length(drugs in proptest::collection::vec(drug_candidate(), 0..=8), tissue in tissue_state()) {
        let scores = score_drug_panel(&drugs, &tissue);
        prop_assert_eq!(scores.len(), drugs.len());
    }

    #[test]
    fn tissue_anderson_drug_panel_matches_individual(
        drugs in proptest::collection::vec(drug_candidate(), 1..=6),
        tissue in tissue_state(),
    ) {
        let panel = score_drug_panel(&drugs, &tissue);
        for (drug, score) in drugs.iter().zip(panel.iter()) {
            let individual = geometry_drug_score(drug, &tissue);
            prop_assert_eq!(score.composite_score, individual.composite_score);
            prop_assert_eq!(score.penetration_factor, individual.penetration_factor);
            prop_assert_eq!(score.anderson_factor, individual.anderson_factor);
            prop_assert_eq!(score.reaches_target, individual.reaches_target);
        }
    }
}
