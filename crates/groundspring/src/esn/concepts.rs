// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Concept edge detection and drift actions for regime boundary identification.
//!
//! LOO cross-validation on disorder sweeps locates parameter values where
//! predictive power breaks down. Drift actions counteract genetic drift at
//! detected phase boundaries.

use super::RegimeLabel;

/// Action taken by the drift monitor when population health is at risk.
///
/// Mirrors `bingoCube/nautilus/src/constraints.rs::DriftAction`. When the
/// effective population size `N_e` times the selection coefficient `s` drops
/// below the drift boundary, the evolutionary process is dominated by
/// genetic drift rather than selection. These actions counteract that.
///
/// # Cross-spring lineage
///
/// Nautilus Shell `constraints.rs` → hotSpring Exp 029/030 → groundSpring V63.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DriftAction {
    /// Population health is fine — no action needed.
    None,
    /// Increase selection pressure (halve elite survivors or grow tournament).
    IncreaseSelection,
    /// Grow population by the given factor with fresh random individuals.
    IncreasePop {
        /// Multiplicative growth factor (e.g. 1.5 = grow by 50%).
        factor: f64,
    },
}

impl std::fmt::Display for DriftAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::IncreaseSelection => write!(f, "increase_selection"),
            Self::IncreasePop { factor } => write!(f, "increase_pop({factor:.1}x)"),
        }
    }
}

/// A detected concept edge — a parameter value where the model's
/// predictive power breaks down, indicating a physical phase boundary.
///
/// Structured replacement for the `(f64, f64)` tuples previously returned
/// by `detect_concept_edges`. Carries the edge location, prediction error,
/// and optional drift action recommendation.
///
/// # Cross-spring lineage
///
/// Nautilus Shell `brain.rs::detect_concept_edges` → hotSpring Exp 028/030.
#[derive(Debug, Clone)]
pub struct ConceptEdge {
    /// The parameter value (e.g. disorder strength W) where the edge occurs.
    pub parameter: f64,
    /// LOO prediction error at this point — higher = sharper boundary.
    pub loo_error: f64,
    /// Recommended drift action if evolution is active at this edge.
    pub drift_action: DriftAction,
}

/// Detect concept edges via leave-one-out cross-validation on disorder sweep data.
///
/// For each point in the sweep, trains on all other points and measures
/// prediction error at the held-out point. Points where the LOO error exceeds
/// `threshold` are regime boundaries — the model cannot generalize across them.
///
/// Returns structured [`ConceptEdge`] values with error magnitude and drift
/// action recommendations. The drift action follows the Nautilus Shell pattern:
/// high-error edges recommend `IncreaseSelection` (sharpen around the boundary),
/// moderate edges recommend `IncreasePop` (explore the boundary region).
///
/// # Cross-spring lineage
///
/// Original: `bingoCube/nautilus/brain.rs` (`detect_concept_edges`).
/// Self-regulation: `bingoCube/nautilus/constraints.rs` (`DriftAction`).
/// The Nautilus Shell uses this for QCD phase boundary detection in lattice
/// gauge theory. groundSpring applies it to Anderson localization transitions.
#[must_use]
pub fn detect_concept_edges(
    disorder_values: &[f64],
    features: &[[f64; 3]],
    regime_labels: &[RegimeLabel],
    threshold: f64,
) -> Vec<ConceptEdge> {
    if features.len() < 4
        || features.len() != regime_labels.len()
        || disorder_values.len() != features.len()
    {
        return Vec::new();
    }
    let n = features.len();

    let label_to_vec = |l: &RegimeLabel| -> [f64; 3] {
        match l {
            RegimeLabel::Extended => [1.0, 0.0, 0.0],
            RegimeLabel::Critical => [0.0, 1.0, 0.0],
            RegimeLabel::Localized => [0.0, 0.0, 1.0],
        }
    };

    let mut edges = Vec::new();

    for hold_out in 0..n {
        let train_feat: Vec<[f64; 3]> = features
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != hold_out)
            .map(|(_, f)| *f)
            .collect();
        let train_labels: Vec<[f64; 3]> = regime_labels
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != hold_out)
            .map(|(_, l)| label_to_vec(l))
            .collect();

        let test_feat = features[hold_out];
        let mut best_dist = f64::MAX;
        let mut best_idx = 0;
        for (i, f) in train_feat.iter().enumerate() {
            let dist = (0..3).map(|k| (f[k] - test_feat[k]).powi(2)).sum::<f64>();
            if dist < best_dist {
                best_dist = dist;
                best_idx = i;
            }
        }

        let pred = &train_labels[best_idx];
        let actual = label_to_vec(&regime_labels[hold_out]);
        let error: f64 = (0..3).map(|k| (pred[k] - actual[k]).powi(2)).sum::<f64>();
        let error = error.sqrt();

        if error > threshold {
            let drift_action = drift_action_for_edge(error, threshold);
            edges.push(ConceptEdge {
                parameter: disorder_values[hold_out],
                loo_error: error,
                drift_action,
            });
        }
    }

    edges
}

/// Error-to-threshold ratio above which drift action escalates to selection pressure.
///
/// Nautilus Shell convention: errors above 2× the threshold indicate a sharp
/// phase boundary requiring tighter selection, not broader exploration.
const SHARP_BOUNDARY_RATIO: f64 = 2.0;

/// Population growth factor when error exceeds the threshold but is below
/// the sharp-boundary ratio. Expands the evolutionary population by 50%
/// to explore the boundary neighbourhood.
const BOUNDARY_EXPLORE_FACTOR: f64 = 1.5;

/// Recommend a [`DriftAction`] based on edge error magnitude.
///
/// The heuristic mirrors Nautilus Shell `constraints.rs`:
/// - Error > 2× threshold → `IncreaseSelection` (sharp boundary)
/// - Error > threshold → `IncreasePop` by 1.5× (explore boundary)
pub(crate) fn drift_action_for_edge(error: f64, threshold: f64) -> DriftAction {
    if error > threshold * SHARP_BOUNDARY_RATIO {
        DriftAction::IncreaseSelection
    } else {
        DriftAction::IncreasePop {
            factor: BOUNDARY_EXPLORE_FACTOR,
        }
    }
}

/// Seed additional sampling points around detected concept edges.
///
/// For each edge, generates `n_seeds` disorder values within ±`radius`
/// of the edge parameter. Used to focus evolutionary exploration around
/// phase boundaries, matching the Nautilus Shell's `EdgeSeeder` pattern.
///
/// # Cross-spring lineage
///
/// `bingoCube/nautilus/constraints.rs::EdgeSeeder` → hotSpring Exp 030 adaptive β.
#[must_use]
pub fn seed_around_edges(edges: &[ConceptEdge], n_seeds: usize, radius: f64) -> Vec<f64> {
    let mut seeds = Vec::with_capacity(edges.len() * n_seeds);
    for edge in edges {
        for i in 0..n_seeds {
            let frac = if n_seeds <= 1 {
                0.0
            } else {
                let fi = crate::cast::usize_f64(i);
                let fn_max = crate::cast::usize_f64(n_seeds - 1);
                (fi / fn_max).mul_add(2.0, -1.0)
            };
            seeds.push(frac.mul_add(radius, edge.parameter));
        }
    }
    seeds
}
