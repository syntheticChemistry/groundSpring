// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Echo State Network (ESN) regime classifier for Anderson localization.
//!
//! Provides a thin wrapper around `barracuda::esn_v2::ESN` for classifying
//! localization regimes (extended / critical / localized) from spectral
//! features. This is the cross-spring realization of hotSpring's ESN work
//! applied to groundSpring's uncertainty quantification domain.
//!
//! # Cross-spring lineage
//!
//! - **hotSpring S26** — `EchoStateNetwork` for MD plasma regime classification.
//!   Reservoir dynamics track temporal correlations in observables (pressure,
//!   energy, diffusion coefficient) to classify WDM phase state without
//!   explicit order parameter computation.
//! - **hotSpring Exp015/022** — ESN regime detection for Anderson localization
//!   transitions. Training data: `(W, ⟨r⟩)` disorder sweeps. Prediction:
//!   regime label from single `⟨r⟩(W)` measurement.
//! - **`ToadStool` S59** — Absorbed as `barracuda::esn_v2::ESN` with GPU reservoir
//!   update via `esn_reservoir_update_f64.wgsl` (wetSpring → hotSpring provenance).
//!   Ridge regression readout via `barracuda::linalg::ridge_regression`.
//! - **groundSpring** — Wraps for Anderson transition regime classification.
//!   Complements the NPU path (Exp 028, AKD1000) with a GPU alternative.
//!
//! # Architecture
//!
//! ```text
//! Input features ─→ Reservoir (sparse random RNN) ─→ Readout (ridge regression) ─→ Label
//!     [W, ⟨r⟩]          500–1000 neurons                   3-class softmax
//! ```
//!
//! # barracuda delegation
//!
//! This module requires the `barracuda-gpu` feature. All heavy computation
//! (reservoir update, readout training) runs on GPU via barracuda's ESN.

/// Multi-head disagreement measurement for epistemic uncertainty.
///
/// When multiple ESN heads (or classifiers) make predictions, the spread
/// across their outputs measures how *uncertain* the classification is.
/// High disagreement at a parameter value indicates a regime boundary —
/// exactly the concept edges from the Nautilus Shell.
///
/// # Cross-spring lineage
///
/// `HeadGroupDisagreement` from hotSpring `reservoir.rs` (Gen 2 multi-head ESN).
/// The Nautilus Shell's concept edge detection (LOO cross-validation) identifies
/// the same boundaries from a different angle. Together they provide both
/// intra-model (disagreement) and inter-model (LOO) uncertainty quantification.
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
        .filter(|&&p| p > 1e-15)
        .map(|&p| p * p.log2())
        .sum::<f64>();

    let mut sorted = probs;
    sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
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
/// Returns `(disorder_value, loo_error)` pairs for detected edges.
///
/// # Cross-spring lineage
///
/// Concept from `bingoCube/nautilus/brain.rs` (`detect_concept_edges`).
/// The Nautilus Shell uses this for QCD phase boundary detection in lattice
/// gauge theory. groundSpring applies it to Anderson localization transitions.
#[must_use]
pub fn detect_concept_edges(
    disorder_values: &[f64],
    features: &[[f64; 3]],
    regime_labels: &[RegimeLabel],
    threshold: f64,
) -> Vec<(f64, f64)> {
    if features.len() < 4 || features.len() != regime_labels.len() {
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

        // Simple nearest-neighbor prediction as lightweight LOO
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
            edges.push((disorder_values[hold_out], error));
        }
    }

    edges
}

/// Localization regime labels for Anderson model classification.
///
/// The three regimes correspond to distinct spectral statistics:
/// - **Extended**: level repulsion, GOE statistics (`⟨r⟩ ≈ 0.53`)
/// - **Critical**: multifractal wavefunctions, intermediate statistics
/// - **Localized**: Poisson statistics (`⟨r⟩ ≈ 0.39`), exponential decay
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegimeLabel {
    /// Delocalized phase — wavefunctions span the full system.
    Extended,
    /// Critical point — fractal wavefunctions, scale-invariant.
    Critical,
    /// Localized phase — wavefunctions decay exponentially.
    Localized,
}

impl std::fmt::Display for RegimeLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Extended => write!(f, "extended"),
            Self::Critical => write!(f, "critical"),
            Self::Localized => write!(f, "localized"),
        }
    }
}

/// GOE level spacing ratio (extended phase, Random Matrix Theory).
pub const GOE_R: f64 = 0.5307;

/// Poisson level spacing ratio (localized phase).
pub const POISSON_R: f64 = 0.3863;

/// Rule-based regime classification from mean level spacing ratio.
///
/// Uses the established crossover values from Random Matrix Theory:
/// - `⟨r⟩ > GOE_R − margin` → Extended
/// - `⟨r⟩ < POISSON_R + margin` → Localized
/// - Otherwise → Critical
///
/// The `margin` parameter controls the width of the critical window.
/// Default suggestion: 0.02 (validated against finite-size scaling
/// in hotSpring Exp015).
#[must_use]
pub fn classify_by_spacing_ratio(mean_r: f64, margin: f64) -> RegimeLabel {
    if mean_r > GOE_R - margin {
        RegimeLabel::Extended
    } else if mean_r < POISSON_R + margin {
        RegimeLabel::Localized
    } else {
        RegimeLabel::Critical
    }
}

/// Rule-based regime classification from Lyapunov exponent.
///
/// For 1D Anderson: all states are localized for W > 0, so this
/// classifies by the *strength* of localization relative to the
/// Aubry-André transition at λ = 2 (Almost-Mathieu model).
///
/// For higher dimensions (2D/3D), the Lyapunov exponent distinguishes
/// true extended (γ → 0 as L → ∞) from localized (γ > 0) phases.
#[must_use]
pub fn classify_by_lyapunov(gamma: f64, critical_threshold: f64) -> RegimeLabel {
    if gamma < 0.005 {
        RegimeLabel::Extended
    } else if gamma < critical_threshold {
        RegimeLabel::Critical
    } else {
        RegimeLabel::Localized
    }
}

/// Extract spectral features for regime classification.
///
/// Given a set of eigenvalues, computes features suitable for ESN
/// or rule-based classification:
/// - Mean level spacing ratio `⟨r⟩`
/// - Spectral bandwidth (max − min)
/// - Normalized IPR proxy (kurtosis of eigenvalue density)
///
/// Returns `[mean_r, bandwidth, kurtosis]`.
#[must_use]
pub fn spectral_features(eigenvalues: &mut [f64]) -> [f64; 3] {
    let mean_r = crate::almost_mathieu::level_spacing_ratio(eigenvalues);
    let n = eigenvalues.len();
    if n < 2 {
        return [mean_r, 0.0, 0.0];
    }

    let bandwidth = eigenvalues[n - 1] - eigenvalues[0];

    let mean_e: f64 = eigenvalues.iter().sum::<f64>() / crate::cast::usize_f64(n);
    let var: f64 = eigenvalues
        .iter()
        .map(|&e| (e - mean_e).powi(2))
        .sum::<f64>()
        / crate::cast::usize_f64(n);
    let m4: f64 = eigenvalues
        .iter()
        .map(|&e| (e - mean_e).powi(4))
        .sum::<f64>()
        / crate::cast::usize_f64(n);
    let kurtosis = if var > 0.0 { m4 / (var * var) } else { 0.0 };

    [mean_r, bandwidth, kurtosis]
}

/// ESN-based regime classifier wrapping `barracuda::esn_v2::ESN`.
///
/// Requires training on disorder sweep data before prediction.
/// The ESN learns the non-linear mapping from spectral features to
/// regime labels, capturing finite-size effects that rule-based
/// classifiers miss.
///
/// # Cross-spring lineage
///
/// `esn_reservoir_update_f64.wgsl` — wetSpring bio (microbial community
/// dynamics) → hotSpring MD (plasma regime detection) → `ToadStool` S59
/// absorption → groundSpring Anderson regime classification. The same
/// reservoir update kernel serves three springs, each benefiting from
/// the others' validation (wetSpring tested diversity stability,
/// hotSpring tested energy conservation, groundSpring tests spectral
/// statistics).
#[cfg(feature = "barracuda-gpu")]
pub struct EsnClassifier {
    esn: barracuda::esn_v2::ESN,
}

#[cfg(feature = "barracuda-gpu")]
impl EsnClassifier {
    /// Create a new ESN classifier with sensible defaults for Anderson
    /// regime classification.
    ///
    /// - `reservoir_size`: 500 neurons (sufficient for 3-class separation)
    /// - `spectral_radius`: 0.9 (echo state property margin)
    /// - `connectivity`: 0.1 (sparse reservoir)
    /// - `leak_rate`: 0.3 (moderate memory)
    /// - `regularization`: 1e-6 (mild Tikhonov for readout)
    ///
    /// # Errors
    ///
    /// Returns an error if GPU device initialization fails.
    pub fn new(seed: u64) -> Result<Self, String> {
        let config = barracuda::esn_v2::ESNConfig {
            input_size: 3,
            reservoir_size: 500,
            output_size: 3,
            spectral_radius: 0.9,
            connectivity: 0.1,
            leak_rate: 0.3,
            regularization: 1e-6,
            seed,
        };
        let esn = pollster::block_on(barracuda::esn_v2::ESN::new(config))
            .map_err(|e| format!("ESN init failed: {e}"))?;
        Ok(Self { esn })
    }

    /// Train the ESN on disorder sweep data.
    ///
    /// `features` — one `[mean_r, bandwidth, kurtosis]` per sweep point
    /// `labels` — one-hot encoded regime labels per sweep point
    ///   (Extended = [1,0,0], Critical = [0,1,0], Localized = [0,0,1])
    ///
    /// Returns the training RMSE.
    ///
    /// # Errors
    ///
    /// Returns an error if training fails (insufficient data, GPU error).
    pub fn train(&mut self, features: &[Vec<f32>], labels: &[Vec<f32>]) -> Result<f32, String> {
        pollster::block_on(self.esn.train(features, labels))
            .map_err(|e| format!("ESN training failed: {e}"))
    }

    /// Classify a single observation.
    ///
    /// Returns the predicted [`RegimeLabel`] based on the ESN output
    /// (argmax of 3-class softmax).
    ///
    /// # Errors
    ///
    /// Returns an error if prediction fails.
    pub fn classify(&mut self, features: &[f32; 3]) -> Result<RegimeLabel, String> {
        let output = pollster::block_on(self.esn.predict(features))
            .map_err(|e| format!("ESN predict failed: {e}"))?;

        let (max_idx, _) = output
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or((0, &0.0));

        Ok(match max_idx {
            0 => RegimeLabel::Extended,
            1 => RegimeLabel::Critical,
            _ => RegimeLabel::Localized,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_extended_by_ratio() {
        assert_eq!(classify_by_spacing_ratio(0.55, 0.02), RegimeLabel::Extended);
    }

    #[test]
    fn classify_localized_by_ratio() {
        assert_eq!(
            classify_by_spacing_ratio(0.38, 0.02),
            RegimeLabel::Localized
        );
    }

    #[test]
    fn classify_critical_by_ratio() {
        assert_eq!(classify_by_spacing_ratio(0.45, 0.02), RegimeLabel::Critical);
    }

    #[test]
    fn classify_by_lyapunov_extended() {
        assert_eq!(classify_by_lyapunov(0.001, 0.05), RegimeLabel::Extended);
    }

    #[test]
    fn classify_by_lyapunov_localized() {
        assert_eq!(classify_by_lyapunov(0.3, 0.05), RegimeLabel::Localized);
    }

    #[test]
    fn classify_by_lyapunov_critical() {
        assert_eq!(classify_by_lyapunov(0.02, 0.05), RegimeLabel::Critical);
    }

    #[test]
    fn regime_label_display() {
        assert_eq!(RegimeLabel::Extended.to_string(), "extended");
        assert_eq!(RegimeLabel::Critical.to_string(), "critical");
        assert_eq!(RegimeLabel::Localized.to_string(), "localized");
    }

    #[test]
    fn spectral_features_uniform_spacing() {
        let mut eigs: Vec<f64> = (0..100).map(|i| f64::from(i) * 0.1).collect();
        let [r, bw, kurt] = spectral_features(&mut eigs);
        assert!((r - 1.0).abs() < 0.01, "uniform spacing → r ≈ 1.0, got {r}");
        assert!((bw - 9.9).abs() < 0.01, "bandwidth = 9.9, got {bw}");
        assert!(kurt > 0.0, "kurtosis should be positive");
    }

    #[test]
    fn spectral_features_empty() {
        let mut eigs: Vec<f64> = vec![];
        let [r, bw, kurt] = spectral_features(&mut eigs);
        assert!(r.abs() < f64::EPSILON);
        assert!(bw.abs() < f64::EPSILON);
        assert!(kurt.abs() < f64::EPSILON);
    }

    #[test]
    fn goe_poisson_constants_ordered() {
        const _: () = assert!(GOE_R > POISSON_R);
    }

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
        // Simulate a disorder sweep crossing the Anderson transition
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
        // Edges should be near the transition points (W≈6, W≈10)
        let edge_disorders: Vec<f64> = edges.iter().map(|(w, _)| *w).collect();
        assert!(
            edge_disorders.iter().any(|&w| w > 4.0 && w < 12.0),
            "edges should be in transition region: {edge_disorders:?}"
        );
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
}
