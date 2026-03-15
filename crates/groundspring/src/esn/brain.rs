// SPDX-License-Identifier: AGPL-3.0-only
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

/// Multi-head disagreement measurement for epistemic uncertainty.
///
/// When multiple ESN heads (or classifiers) make predictions, the spread
/// across their outputs measures how *uncertain* the classification is.
/// High disagreement at a parameter value indicates a regime boundary —
/// exactly the concept edges from the Nautilus Shell.
///
/// # Cross-spring lineage
///
/// `HeadGroupDisagreement` from hotSpring `reservoir.rs` (Gen 2 multi-head ESN,
/// 15 heads in v0.6.15). The Nautilus Shell's concept edge detection (LOO
/// cross-validation) identifies the same boundaries from a different angle.
/// Together they provide both intra-model (disagreement) and inter-model
/// (LOO) uncertainty quantification.
#[derive(Debug, Clone, Copy)]
pub struct ClassificationUncertainty {
    /// Maximum softmax output (confidence of the winning class).
    pub confidence: f64,
    /// Entropy of the softmax distribution (bits).
    /// Low entropy → confident; high entropy → uncertain.
    pub entropy: f64,
    /// Margin between top-1 and top-2 softmax probabilities.
    /// Small margin → boundary region.
    pub margin: f64,
}

/// Multi-observable uncertainty from multiple ESN heads.
///
/// When N heads each predict a different observable (e.g. bias, variance,
/// spectral width, localization length), this struct captures per-observable
/// uncertainty and the inter-head disagreement.
///
/// # Cross-spring lineage
///
/// hotSpring v0.6.15 15-head ESN: heads 1-3 predict plaquette/CG/acceptance,
/// heads 4-11 predict phase/therm/convergence/anomaly, heads 12-15 predict
/// Anderson proxies. groundSpring maps this to uncertainty observables.
#[derive(Debug, Clone)]
pub struct MultiHeadUncertainty {
    /// Per-observable mean prediction across heads.
    pub means: Vec<f64>,
    /// Per-observable standard deviation across heads (epistemic uncertainty).
    pub std_devs: Vec<f64>,
    /// Maximum inter-head disagreement (max of per-observable CV).
    pub max_disagreement: f64,
    /// Number of heads that contributed predictions.
    pub n_heads: usize,
}

/// Compute multi-head uncertainty from a matrix of head predictions.
///
/// `predictions` is `[n_heads][n_observables]` — each head's prediction
/// for each observable. Returns aggregated uncertainty across heads.
#[must_use]
pub fn multi_head_uncertainty(predictions: &[Vec<f64>]) -> MultiHeadUncertainty {
    if predictions.is_empty() || predictions[0].is_empty() {
        return MultiHeadUncertainty {
            means: Vec::new(),
            std_devs: Vec::new(),
            max_disagreement: 0.0,
            n_heads: 0,
        };
    }

    let n_heads = predictions.len();
    let n_obs = predictions[0].len();
    let nh = crate::cast::usize_f64(n_heads);

    let mut means = vec![0.0; n_obs];
    let mut std_devs = vec![0.0; n_obs];

    for obs in 0..n_obs {
        let sum: f64 = predictions.iter().map(|h| h[obs]).sum();
        means[obs] = sum / nh;
    }

    for obs in 0..n_obs {
        let var: f64 = predictions
            .iter()
            .map(|h| (h[obs] - means[obs]).powi(2))
            .sum::<f64>()
            / nh;
        std_devs[obs] = var.sqrt();
    }

    let max_disagreement = means
        .iter()
        .zip(std_devs.iter())
        .map(|(&m, &s)| {
            if m.abs() > crate::eps::LOG_FLOOR {
                s / m.abs()
            } else {
                s
            }
        })
        .fold(0.0_f64, f64::max);

    MultiHeadUncertainty {
        means,
        std_devs,
        max_disagreement,
        n_heads,
    }
}

impl ClassificationUncertainty {
    /// Whether this classification is near a regime boundary.
    ///
    /// Returns `true` if both confidence is low AND margin is small,
    /// indicating the classifier cannot clearly distinguish regimes.
    #[must_use]
    pub fn is_boundary(&self, confidence_threshold: f64, margin_threshold: f64) -> bool {
        self.confidence < confidence_threshold && self.margin < margin_threshold
    }
}

/// Compute classification uncertainty from raw softmax-like outputs.
///
/// `outputs` — raw (unnormalized) scores from a classifier, one per class.
/// Returns uncertainty metrics after softmax normalization.
#[must_use]
pub fn classification_uncertainty(outputs: &[f64]) -> ClassificationUncertainty {
    if outputs.is_empty() {
        return ClassificationUncertainty {
            confidence: 0.0,
            entropy: 0.0,
            margin: 0.0,
        };
    }

    let max_val = outputs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let exp_sum: f64 = outputs.iter().map(|&x| (x - max_val).exp()).sum();
    let probs: Vec<f64> = outputs
        .iter()
        .map(|&x| (x - max_val).exp() / exp_sum)
        .collect();

    let confidence = probs.iter().copied().fold(0.0_f64, f64::max);

    let entropy = -probs
        .iter()
        .filter(|&&p| p > crate::eps::LOG_FLOOR)
        .map(|&p| p * p.log2())
        .sum::<f64>();

    let mut sorted = probs;
    sorted.sort_by(|a, b| b.total_cmp(a));
    let margin = if sorted.len() >= 2 {
        sorted[0] - sorted[1]
    } else {
        1.0
    };

    ClassificationUncertainty {
        confidence,
        entropy,
        margin,
    }
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

/// Recommend a [`DriftAction`] based on edge error magnitude.
///
/// The heuristic mirrors Nautilus Shell `constraints.rs`:
/// - Error > 2× threshold → `IncreaseSelection` (sharp boundary)
/// - Error > threshold → `IncreasePop` by 1.5× (explore boundary)
fn drift_action_for_edge(error: f64, threshold: f64) -> DriftAction {
    if error > threshold * 2.0 {
        DriftAction::IncreaseSelection
    } else {
        DriftAction::IncreasePop { factor: 1.5 }
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
        let labels: Vec<RegimeLabel> = disorders
            .iter()
            .map(|&w| {
                if w < 6.0 {
                    RegimeLabel::Extended
                } else if w < 10.0 {
                    RegimeLabel::Critical
                } else {
                    RegimeLabel::Localized
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
            &[RegimeLabel::Extended; 2],
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
        let action = super::drift_action_for_edge(1.5, 0.5);
        assert_eq!(action, DriftAction::IncreaseSelection);
    }

    #[test]
    fn drift_action_for_edge_moderate_boundary() {
        let action = super::drift_action_for_edge(0.7, 0.5);
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
