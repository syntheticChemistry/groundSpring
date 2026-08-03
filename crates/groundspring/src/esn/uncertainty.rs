// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Multi-head and classification uncertainty quantification for ESN outputs.
//!
//! Softmax entropy, confidence margins, and inter-head disagreement measure
//! epistemic uncertainty at regime boundaries.

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
