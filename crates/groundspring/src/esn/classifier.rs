// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Regime classification for Anderson localization transitions.
//!
//! Rule-based and ESN-based classifiers that map spectral features
//! (level spacing ratio, bandwidth, kurtosis) to localization regimes
//! (extended / critical / localized).
//!
//! # Cross-spring lineage
//!
//! - **hotSpring Exp015/022** — ESN regime detection for Anderson transitions
//! - **barraCuda S59** — `barracuda::esn_v2::ESN` with GPU reservoir update
//! - **groundSpring** — Wraps for Anderson transition regime classification

use super::RegimeLabel;

/// Tikhonov regularization for ESN readout weight computation.
///
/// Mild regularization (1e-6) prevents ill-conditioned readout matrices
/// without biasing the classification. Validated against hotSpring
/// Exp 015/022 Anderson transition detection.
#[cfg(feature = "barracuda-gpu")]
const ESN_READOUT_REGULARIZATION: f32 = 1e-6;

/// Default ESN reservoir size for Anderson regime classification.
///
/// 500 neurons provides sufficient capacity for 3-class separation
/// (extended / critical / localized) without overfitting. Validated
/// against hotSpring Exp 015/022 disorder sweeps.
#[cfg(feature = "barracuda-gpu")]
const ESN_RESERVOIR_SIZE: usize = 500;

/// Default ESN spectral radius (echo state property margin).
///
/// 0.9 < 1.0 ensures the echo state property holds: reservoir dynamics
/// decay, preventing chaotic amplification of input noise.
#[cfg(feature = "barracuda-gpu")]
const ESN_SPECTRAL_RADIUS: f32 = 0.9;

/// Default ESN connectivity (fraction of non-zero reservoir weights).
///
/// 10% connectivity yields a sparse reservoir that is computationally
/// efficient while preserving sufficient coupling for regime separation.
#[cfg(feature = "barracuda-gpu")]
const ESN_CONNECTIVITY: f32 = 0.1;

/// Default ESN leak rate (exponential smoothing of reservoir state).
///
/// 0.3 provides moderate memory: new inputs blend with 70% prior state,
/// suitable for disorder-sweep time series where adjacent points are
/// correlated but not redundant.
#[cfg(feature = "barracuda-gpu")]
const ESN_LEAK_RATE: f32 = 0.3;

/// GOE level spacing ratio (extended phase, Random Matrix Theory).
pub const GOE_R: f64 = 0.5307;

/// Poisson level spacing ratio (localized phase).
pub const POISSON_R: f64 = 0.3863;

/// Lyapunov exponent below which a state is classified as extended.
///
/// In higher-dimensional Anderson models (2D/3D) the Lyapunov exponent
/// γ → 0 as L → ∞ in the extended phase. This threshold captures the
/// practical residual from finite-size systems: γ < 0.005 indicates
/// essentially ballistic transport. Validated against Almost-Mathieu
/// λ = 0.5 (deeply extended) benchmark in Exp 009.
const LYAPUNOV_EXTENDED_THRESHOLD: f64 = 0.005;

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
    if gamma < LYAPUNOV_EXTENDED_THRESHOLD {
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
/// dynamics) → hotSpring MD (plasma regime detection) → barraCuda S59
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
            reservoir_size: ESN_RESERVOIR_SIZE,
            output_size: 3,
            spectral_radius: ESN_SPECTRAL_RADIUS,
            connectivity: ESN_CONNECTIVITY,
            leak_rate: ESN_LEAK_RATE,
            regularization: ESN_READOUT_REGULARIZATION,
            seed,
            ..barracuda::esn_v2::ESNConfig::default()
        };
        let esn = barracuda::device::test_pool::tokio_block_on(barracuda::esn_v2::ESN::new(config))
            .map_err(|e| format!("ESN init failed: {e}"))?;
        Ok(Self { esn })
    }

    /// Train the ESN on disorder sweep data.
    ///
    /// `features` — one `[mean_r, bandwidth, kurtosis]` per sweep point
    /// `labels` — one-hot encoded regime labels per sweep point
    ///   (Extended = \[1,0,0\], Critical = \[0,1,0\], Localized = \[0,0,1\])
    ///
    /// Returns the training RMSE.
    ///
    /// # Errors
    ///
    /// Returns an error if training fails (insufficient data, GPU error).
    pub fn train(&mut self, features: &[Vec<f32>], labels: &[Vec<f32>]) -> Result<f32, String> {
        barracuda::device::test_pool::tokio_block_on(self.esn.train(features, labels))
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
        let output = barracuda::device::test_pool::tokio_block_on(self.esn.predict(features))
            .map_err(|e| format!("ESN predict failed: {e}"))?;

        let (max_idx, _) = output
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .unwrap_or((0, &0.0));

        Ok(match max_idx {
            0 => RegimeLabel::Extended,
            1 => RegimeLabel::Critical,
            _ => RegimeLabel::Localized,
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
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
}
