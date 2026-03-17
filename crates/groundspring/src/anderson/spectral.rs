// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Spectral diagnostics for Anderson localization eigenvalue analysis.
//!
//! Random Matrix Theory (RMT) tools: Marchenko-Pastur bounds, empirical
//! spectral density histograms, spectral phase classification, and
//! transition detection via peak finding on disorder sweeps.
//!
//! Cross-spring lineage: neuralSpring V69 spectral phase classification →
//! barraCuda S79 `spectral::stats` → barraCuda `stats::spectral_density` →
//! groundSpring Anderson RMT diagnostics.

#[cfg(not(feature = "barracuda"))]
use crate::eps;

use super::SweepPoint;

/// Outlier fraction threshold below which spectrum is classified as Bulk (< 5%).
#[cfg(not(feature = "barracuda"))]
const BULK_PHASE_OUTLIER_THRESHOLD: f64 = 0.05;
/// Outlier fraction threshold for `EdgeOfChaos` (5–20%).
#[cfg(not(feature = "barracuda"))]
const EDGE_OF_CHAOS_OUTLIER_THRESHOLD: f64 = 0.20;
/// Minimum prominence for peak detection in disorder sweep transition.
#[cfg(feature = "barracuda-gpu")]
const PEAK_DETECT_MIN_PROMINENCE: f64 = 0.001;

/// Spectral diagnostics for eigenvalue analysis.
///
/// Wraps `barracuda::spectral::stats` (barraCuda S79) to provide
/// bandwidth, condition number, and phase classification for Anderson
/// localization eigenvalue spectra.
#[derive(Debug, Clone)]
pub struct SpectralDiagnostics {
    /// Spectral bandwidth (max − min eigenvalue).
    pub bandwidth: f64,
    /// Spectral condition number (max|λ| / min|λ|).
    pub condition_number: f64,
    /// Phase classification based on Marchenko-Pastur outlier fraction.
    pub phase: SpectralPhaseLabel,
}

/// Spectral phase label matching barracuda's `SpectralPhase`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpectralPhaseLabel {
    /// < 5 % outliers beyond Marchenko-Pastur upper bound.
    Bulk,
    /// 5–20 % outliers.
    EdgeOfChaos,
    /// > 20 % outliers.
    Chaotic,
}

/// Compute spectral diagnostics for an eigenvalue spectrum.
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::spectral::{spectral_bandwidth, spectral_condition_number,
/// classify_spectral_phase}` (barraCuda S79, provenance: neuralSpring V69).
///
/// `marchenko_upper` is the Marchenko-Pastur upper bound for the
/// eigenvalue distribution of the associated random matrix; eigenvalues
/// exceeding this bound are counted as outliers for phase classification.
#[must_use]
pub fn spectral_diagnostics(eigenvalues: &[f64], marchenko_upper: f64) -> SpectralDiagnostics {
    #[cfg(feature = "barracuda")]
    return spectral_diagnostics_barracuda(eigenvalues, marchenko_upper);
    #[cfg(not(feature = "barracuda"))]
    spectral_diagnostics_cpu(eigenvalues, marchenko_upper)
}

#[cfg(feature = "barracuda")]
fn spectral_diagnostics_barracuda(
    eigenvalues: &[f64],
    marchenko_upper: f64,
) -> SpectralDiagnostics {
    let bandwidth = barracuda::spectral::spectral_bandwidth(eigenvalues);
    let condition_number = barracuda::spectral::spectral_condition_number(eigenvalues);
    let phase = barracuda::spectral::classify_spectral_phase(eigenvalues, marchenko_upper);
    SpectralDiagnostics {
        bandwidth,
        condition_number,
        phase: match phase {
            barracuda::spectral::SpectralPhase::Bulk => SpectralPhaseLabel::Bulk,
            barracuda::spectral::SpectralPhase::EdgeOfChaos => SpectralPhaseLabel::EdgeOfChaos,
            barracuda::spectral::SpectralPhase::Chaotic => SpectralPhaseLabel::Chaotic,
        },
    }
}

#[cfg(not(feature = "barracuda"))]
fn spectral_diagnostics_cpu(eigenvalues: &[f64], marchenko_upper: f64) -> SpectralDiagnostics {
    let (min, max, min_abs, max_abs) = eigenvalues.iter().fold(
        (f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, 0.0_f64),
        |(lo, hi, lo_abs, hi_abs), &x| {
            (
                lo.min(x),
                hi.max(x),
                lo_abs.min(x.abs()),
                hi_abs.max(x.abs()),
            )
        },
    );
    let bandwidth = if eigenvalues.is_empty() {
        0.0
    } else {
        max - min
    };
    let condition_number = if min_abs < eps::UNDERFLOW {
        f64::INFINITY
    } else {
        max_abs / min_abs
    };

    let outlier_frac = if eigenvalues.is_empty() {
        0.0
    } else {
        let outliers = eigenvalues.iter().filter(|&&x| x > marchenko_upper).count();
        crate::cast::usize_f64(outliers) / crate::cast::usize_f64(eigenvalues.len())
    };
    let phase = if outlier_frac < BULK_PHASE_OUTLIER_THRESHOLD {
        SpectralPhaseLabel::Bulk
    } else if outlier_frac <= EDGE_OF_CHAOS_OUTLIER_THRESHOLD {
        SpectralPhaseLabel::EdgeOfChaos
    } else {
        SpectralPhaseLabel::Chaotic
    };

    SpectralDiagnostics {
        bandwidth,
        condition_number,
        phase,
    }
}

/// Compute Marchenko-Pastur upper bound for a random matrix with aspect ratio γ.
///
/// For an N×M random matrix with γ = N/M, the eigenvalue distribution of
/// M^T M / N converges to the Marchenko-Pastur law with upper support
/// \( \lambda_{\max} = (1 + \sqrt{\gamma})^2 \). This is the cutoff used by
/// [`spectral_diagnostics`] for outlier-based phase classification.
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::stats::spectral_density::marchenko_pastur_bounds`.
#[must_use]
pub fn marchenko_pastur_upper(gamma: f64) -> f64 {
    #[cfg(feature = "barracuda")]
    {
        let (_lower, upper) = barracuda::stats::spectral_density::marchenko_pastur_bounds(gamma);
        upper
    }
    #[cfg(not(feature = "barracuda"))]
    {
        let sq = gamma.sqrt();
        (1.0 + sq).powi(2)
    }
}

/// Compute full spectral diagnostics with automatic Marchenko-Pastur bound.
///
/// Convenience wrapper that computes the Marchenko-Pastur upper bound from
/// `gamma` (aspect ratio N/M of the original random matrix) and then calls
/// [`spectral_diagnostics`].
#[must_use]
pub fn spectral_diagnostics_auto(eigenvalues: &[f64], gamma: f64) -> SpectralDiagnostics {
    spectral_diagnostics(eigenvalues, marchenko_pastur_upper(gamma))
}

/// Empirical spectral density (histogram) of eigenvalues.
///
/// Returns `(bin_centers, bin_counts)` where `bin_counts` are normalized
/// so that `Σ counts × Δbin = 1`.
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::stats::spectral_density::empirical_spectral_density`.
///
/// Cross-spring lineage: neuralSpring V69 spectral analysis →
/// barraCuda `stats::spectral_density` → groundSpring Anderson RMT diagnostics.
#[must_use]
pub fn empirical_spectral_density(eigenvalues: &[f64], n_bins: usize) -> (Vec<f64>, Vec<f64>) {
    #[cfg(feature = "barracuda")]
    {
        barracuda::stats::spectral_density::empirical_spectral_density(eigenvalues, n_bins)
    }
    #[cfg(not(feature = "barracuda"))]
    empirical_spectral_density_cpu(eigenvalues, n_bins)
}

#[cfg(not(feature = "barracuda"))]
fn empirical_spectral_density_cpu(eigenvalues: &[f64], n_bins: usize) -> (Vec<f64>, Vec<f64>) {
    if eigenvalues.is_empty() || n_bins == 0 {
        return (vec![], vec![]);
    }
    let min = eigenvalues.iter().copied().fold(f64::INFINITY, f64::min);
    let max = eigenvalues
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let range = max - min;
    if range < f64::EPSILON {
        return (vec![min], vec![1.0]);
    }
    let bin_width = range / crate::cast::usize_f64(n_bins);
    let mut counts = vec![0_usize; n_bins];
    for &e in eigenvalues {
        let idx = crate::cast::f64_usize(((e - min) / bin_width).floor());
        counts[idx.min(n_bins - 1)] += 1;
    }
    let n_f = crate::cast::usize_f64(eigenvalues.len());
    let centers: Vec<f64> = (0..n_bins)
        .map(|i| (crate::cast::usize_f64(i) + 0.5).mul_add(bin_width, min))
        .collect();
    let densities: Vec<f64> = counts
        .iter()
        .map(|&c| crate::cast::usize_f64(c) / (n_f * bin_width))
        .collect();
    (centers, densities)
}

/// Detect transition points in a disorder sweep using peak detection.
///
/// Analyzes the derivative of ⟨r⟩ vs W to find the steepest descent,
/// which marks the localization transition at `W_c`.
///
/// When `barracuda-gpu` is enabled, delegates to
/// `barracuda::ops::peak_detect_f64::PeakDetectF64` for GPU-accelerated
/// peak finding on the negative derivative signal. Falls back to CPU
/// argmax on the finite-difference derivative.
///
/// Returns the disorder strength `W_c` at the detected transition, or
/// `None` if no clear transition is found.
#[must_use]
pub fn detect_transition(sweep: &[SweepPoint]) -> Option<f64> {
    if sweep.len() < 3 {
        return None;
    }
    let deriv: Vec<f64> = sweep
        .windows(2)
        .map(|w| (w[1].mean_ratio - w[0].mean_ratio).abs() / (w[1].disorder - w[0].disorder))
        .collect();

    #[cfg(feature = "barracuda-gpu")]
    {
        if let Some(w_c) = detect_transition_gpu(&deriv, sweep) {
            return Some(w_c);
        }
    }

    let peak_idx = deriv
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(i, _)| i)?;
    Some(f64::midpoint(
        sweep[peak_idx].disorder,
        sweep[peak_idx + 1].disorder,
    ))
}

#[cfg(feature = "barracuda-gpu")]
fn detect_transition_gpu(deriv: &[f64], sweep: &[SweepPoint]) -> Option<f64> {
    let device = crate::gpu::get_device()?;
    let peaks = barracuda::ops::peak_detect_f64::PeakDetectF64::new(deriv, 1)
        .prominence(PEAK_DETECT_MIN_PROMINENCE)
        .execute(&device)
        .ok()?;
    let best = peaks
        .iter()
        .max_by(|a, b| a.prominence.total_cmp(&b.prominence))?;
    Some(f64::midpoint(
        sweep[best.index].disorder,
        sweep[best.index + 1].disorder,
    ))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use crate::tol;

    #[test]
    fn spectral_diagnostics_empty() {
        let d = spectral_diagnostics(&[], 5.0);
        assert!(d.bandwidth.abs() < f64::EPSILON);
        assert_eq!(d.phase, SpectralPhaseLabel::Bulk);
    }

    #[test]
    fn spectral_diagnostics_known_spectrum() {
        let eigs = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let d = spectral_diagnostics(&eigs, 10.0);
        assert!(
            (d.bandwidth - 4.0).abs() < tol::EXACT,
            "bandwidth = max - min"
        );
        assert!((d.condition_number - 5.0).abs() < tol::EXACT, "κ = 5/1");
        assert_eq!(d.phase, SpectralPhaseLabel::Bulk, "all below MP upper");
    }

    #[test]
    fn spectral_diagnostics_chaotic() {
        let mut eigs = vec![0.1; 10];
        for e in &mut eigs[..8] {
            *e = 100.0;
        }
        let d = spectral_diagnostics(&eigs, 0.5);
        assert_eq!(d.phase, SpectralPhaseLabel::Chaotic, "80% outliers");
    }

    #[test]
    fn marchenko_pastur_upper_square() {
        let upper = marchenko_pastur_upper(1.0);
        assert!(
            (upper - 4.0).abs() < tol::EXACT,
            "γ=1 → λ_max=4, got {upper}"
        );
    }

    #[test]
    fn marchenko_pastur_upper_rectangular() {
        let upper = marchenko_pastur_upper(0.25);
        assert!(
            (upper - 2.25).abs() < tol::EXACT,
            "γ=0.25 → λ_max=2.25, got {upper}"
        );
    }

    #[test]
    fn spectral_diagnostics_auto_matches_manual() {
        let eigs = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let gamma = 1.0;
        let auto = spectral_diagnostics_auto(&eigs, gamma);
        let manual = spectral_diagnostics(&eigs, marchenko_pastur_upper(gamma));
        assert!((auto.bandwidth - manual.bandwidth).abs() < f64::EPSILON);
        assert!((auto.condition_number - manual.condition_number).abs() < f64::EPSILON);
        assert_eq!(auto.phase, manual.phase);
    }

    #[test]
    fn esd_uniform_eigenvalues() {
        let eigs: Vec<f64> = (0..100).map(|i| crate::cast::usize_f64(i) * 0.1).collect();
        let (centers, densities) = empirical_spectral_density(&eigs, 10);
        assert_eq!(centers.len(), 10);
        assert_eq!(densities.len(), 10);
        for d in &densities {
            assert!(
                *d > 0.0,
                "all bins should have positive density for uniform eigenvalues"
            );
        }
    }

    #[test]
    fn esd_empty_returns_empty() {
        let (c, d) = empirical_spectral_density(&[], 10);
        assert!(c.is_empty());
        assert!(d.is_empty());
    }

    #[test]
    fn detect_transition_monotone_sweep() {
        let sweep: Vec<SweepPoint> = (0..20)
            .map(|i| {
                let w = crate::cast::usize_f64(i).mul_add(0.25, 0.5);
                let r = 1.0 / (1.0 + (0.5 * (w - 3.0)).exp());
                SweepPoint {
                    disorder: w,
                    mean_ratio: r,
                    std_error: 0.01,
                }
            })
            .collect();
        let w_c = detect_transition(&sweep);
        assert!(w_c.is_some(), "should detect a transition");
        let w_c = w_c.unwrap();
        assert!(
            (w_c - 3.0).abs() < 1.0,
            "transition at W_c={w_c}, expected ~3.0"
        );
    }

    #[test]
    fn detect_transition_too_short() {
        let sweep = vec![SweepPoint {
            disorder: 1.0,
            mean_ratio: 0.5,
            std_error: 0.01,
        }];
        assert!(detect_transition(&sweep).is_none());
    }
}
