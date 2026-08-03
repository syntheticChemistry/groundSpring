// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Brain architecture types for evolutionary uncertainty quantification.
//!
//! Provides concept edge detection, drift actions, and multi-head uncertainty
//! measurement. These types originate from the Nautilus Shell evolutionary
//! reservoir computing framework and are applied here to Anderson localization
//! phase boundary detection.
//!
//! # Cross-spring lineage
//!
//! - **bingoCube/nautilus** — `DriftAction`, `ConceptEdge`, `EdgeSeeder`
//! - **hotSpring Exp 028-030** — Multi-head ESN disagrement, concept edges for
//!   QCD phase transitions and Anderson localization
//! - **groundSpring V63** — Brain architecture integration for uncertainty
//!   quantification across localization regimes

pub use super::concepts::{ConceptEdge, DriftAction, detect_concept_edges, seed_around_edges};
pub use super::uncertainty::{
    ClassificationUncertainty, MultiHeadUncertainty, classification_uncertainty,
    multi_head_uncertainty,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tol;

    #[test]
    fn uncertainty_confident_classification() {
        let outputs = [5.0, 0.1, 0.1];
        let u = classification_uncertainty(&outputs);
        assert!(
            u.confidence > 0.95,
            "confidence should be high: {}",
            u.confidence
        );
        assert!(u.margin > 0.9, "margin should be large: {}", u.margin);
        assert!(u.entropy < 0.5, "entropy should be low: {}", u.entropy);
        assert!(!u.is_boundary(0.6, 0.3));
    }

    #[test]
    fn uncertainty_boundary_classification() {
        let outputs = [1.0, 0.9, 0.1];
        let u = classification_uncertainty(&outputs);
        assert!(
            u.confidence < 0.55,
            "confidence should be moderate: {}",
            u.confidence
        );
        assert!(u.margin < 0.15, "margin should be small: {}", u.margin);
        assert!(u.is_boundary(0.6, 0.3));
    }

    #[test]
    fn uncertainty_empty() {
        let u = classification_uncertainty(&[]);
        assert!((u.confidence).abs() < f64::EPSILON);
        assert!((u.entropy).abs() < f64::EPSILON);
    }

    #[test]
    fn concept_edge_detects_transition() {
        let disorders: Vec<f64> = (0..12).map(|i| f64::from(i).mul_add(1.5, 1.0)).collect();
        let features: Vec<[f64; 3]> = disorders
            .iter()
            .map(|&w| {
                let r = if w < 8.0 {
                    (w - 1.0).mul_add(-0.005, 0.53)
                } else {
                    (16.5 - w).mul_add(0.002, 0.39)
                };
                [r, w.mul_add(-0.1, 4.0), w.mul_add(0.05, 3.0)]
            })
            .collect();
        let labels: Vec<crate::esn::RegimeLabel> = disorders
            .iter()
            .map(|&w| {
                if w < 6.0 {
                    crate::esn::RegimeLabel::Extended
                } else if w < 10.0 {
                    crate::esn::RegimeLabel::Critical
                } else {
                    crate::esn::RegimeLabel::Localized
                }
            })
            .collect();

        let edges = detect_concept_edges(&disorders, &features, &labels, 0.5);
        assert!(
            !edges.is_empty(),
            "should detect edges at regime transitions"
        );
        let edge_params: Vec<f64> = edges.iter().map(|e| e.parameter).collect();
        assert!(
            edge_params.iter().any(|&w| w > 4.0 && w < 12.0),
            "edges should be in transition region: {edge_params:?}"
        );
        for edge in &edges {
            assert!(edge.loo_error > 0.5, "edges should exceed threshold");
            assert_ne!(
                edge.drift_action,
                DriftAction::None,
                "edge should recommend an action"
            );
        }
    }

    #[test]
    fn concept_edge_too_few_points() {
        let edges = detect_concept_edges(
            &[1.0, 2.0],
            &[[0.5, 1.0, 2.0]; 2],
            &[crate::esn::RegimeLabel::Extended; 2],
            0.5,
        );
        assert!(edges.is_empty(), "need >= 4 points for LOO");
    }

    #[test]
    fn drift_action_display() {
        assert_eq!(DriftAction::None.to_string(), "none");
        assert_eq!(
            DriftAction::IncreaseSelection.to_string(),
            "increase_selection"
        );
        assert_eq!(
            DriftAction::IncreasePop { factor: 1.5 }.to_string(),
            "increase_pop(1.5x)"
        );
    }

    #[test]
    fn drift_action_for_edge_sharp_boundary() {
        let action = crate::esn::concepts::drift_action_for_edge(1.5, 0.5);
        assert_eq!(action, DriftAction::IncreaseSelection);
    }

    #[test]
    fn drift_action_for_edge_moderate_boundary() {
        let action = crate::esn::concepts::drift_action_for_edge(0.7, 0.5);
        assert_eq!(action, DriftAction::IncreasePop { factor: 1.5 });
    }

    #[test]
    fn seed_around_edges_basic() {
        let edges = vec![ConceptEdge {
            parameter: 5.0,
            loo_error: 1.0,
            drift_action: DriftAction::IncreaseSelection,
        }];
        let seeds = seed_around_edges(&edges, 5, 0.5);
        assert_eq!(seeds.len(), 5);
        assert!(
            (seeds[0] - 4.5).abs() < tol::ANALYTICAL,
            "first seed at -radius"
        );
        assert!(
            (seeds[4] - 5.5).abs() < tol::ANALYTICAL,
            "last seed at +radius"
        );
        assert!(
            (seeds[2] - 5.0).abs() < tol::ANALYTICAL,
            "middle seed at center"
        );
    }

    #[test]
    fn seed_around_edges_empty() {
        let seeds = seed_around_edges(&[], 5, 0.5);
        assert!(seeds.is_empty());
    }

    #[test]
    fn multi_head_uncertainty_basic() {
        let preds = vec![
            vec![1.0, 2.0, 3.0],
            vec![1.1, 2.2, 2.8],
            vec![0.9, 1.8, 3.2],
        ];
        let u = multi_head_uncertainty(&preds);
        assert_eq!(u.n_heads, 3);
        assert_eq!(u.means.len(), 3);
        assert_eq!(u.std_devs.len(), 3);
        assert!((u.means[0] - 1.0).abs() < tol::STOCHASTIC);
        assert!(u.max_disagreement > 0.0);
    }

    #[test]
    fn multi_head_uncertainty_empty() {
        let u = multi_head_uncertainty(&[]);
        assert_eq!(u.n_heads, 0);
        assert!(u.means.is_empty());
    }

    #[test]
    fn multi_head_uncertainty_single_head() {
        let preds = vec![vec![1.0, 2.0]];
        let u = multi_head_uncertainty(&preds);
        assert_eq!(u.n_heads, 1);
        assert!(
            (u.std_devs[0]).abs() < tol::STRICT,
            "single head → zero std dev"
        );
    }
}
