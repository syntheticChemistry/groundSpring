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
//! - **barraCuda S59** — Absorbed as `barracuda::esn_v2::ESN` with GPU reservoir
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
//! # Module structure
//!
//! - `brain` — Concept edge detection ([`detect_concept_edges`], [`DriftAction`], [`multi_head_uncertainty`])
//! - `classifier` — Rule-based ([`classify_by_lyapunov`], [`classify_by_spacing_ratio`]) and ESN-based regime classification
//!
//! # barracuda delegation
//!
//! This module requires the `barracuda-gpu` feature for ESN GPU inference.
//! Brain architecture types and rule-based classifiers work without GPU.

mod brain;
mod classifier;

pub use brain::{
    ClassificationUncertainty, ConceptEdge, DriftAction, MultiHeadUncertainty,
    classification_uncertainty, detect_concept_edges, multi_head_uncertainty, seed_around_edges,
};
#[cfg(feature = "barracuda-gpu")]
pub use classifier::EsnClassifier;
pub use classifier::{
    GOE_R, POISSON_R, classify_by_lyapunov, classify_by_spacing_ratio, spectral_features,
};

/// Typed error for ESN classifier operations.
///
/// Preserves the operation context (init / train / predict) and the
/// source error from the barraCuda GPU layer instead of flattening
/// to an opaque `String`.
#[cfg(feature = "barracuda-gpu")]
#[derive(Debug, thiserror::Error)]
pub enum EsnError {
    /// GPU device initialization or ESN reservoir creation failed.
    #[error("ESN init failed: {0}")]
    Init(#[source] barracuda::error::BarracudaError),
    /// Readout weight training failed.
    #[error("ESN training failed: {0}")]
    Train(#[source] barracuda::error::BarracudaError),
    /// Prediction (forward pass) failed.
    #[error("ESN predict failed: {0}")]
    Predict(#[source] barracuda::error::BarracudaError),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regime_label_display() {
        assert_eq!(format!("{}", RegimeLabel::Extended), "extended");
        assert_eq!(format!("{}", RegimeLabel::Critical), "critical");
        assert_eq!(format!("{}", RegimeLabel::Localized), "localized");
    }

    #[test]
    fn regime_labels_are_distinct() {
        assert_ne!(RegimeLabel::Extended, RegimeLabel::Critical);
        assert_ne!(RegimeLabel::Critical, RegimeLabel::Localized);
        assert_ne!(RegimeLabel::Extended, RegimeLabel::Localized);
    }

    #[test]
    fn goe_and_poisson_are_distinct() {
        assert!(GOE_R > POISSON_R);
    }
}
