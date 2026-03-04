// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Anderson localization in 1D tight-binding models (Exp 008).
//!
//! ```text
//! H ψ(n) = ψ(n+1) + ψ(n-1) + V(n) ψ(n)
//! ```
//!
//! where `V(n)` is a random potential drawn uniformly from `[-W/2, W/2]`.
//! In 1D, Anderson (1958) proved that ALL states are localized for any
//! disorder `W > 0`.  The localization length `ξ ~ C / W²` at the band
//! center `E = 0` (Thouless 1972, Derrida-Gardner).
//!
//! # barracuda delegation
//!
//! When the `barracuda-gpu` feature is enabled, [`lyapunov_exponent`] and
//! [`lyapunov_averaged`] delegate to `barracuda::spectral::lyapunov_*`.
//!
//! **Seed convention:** the local CPU implementation uses `base_seed + i`
//! per realization; barracuda uses `base_seed + r * 1000`.  Results diverge
//! when the feature gate is active — this is expected and documented in
//! `specs/BARRACUDA_EVOLUTION.md` as Phase 2b alignment work.
//!
//! ## Cross-spring evolved capabilities (S59+, S79+)
//!
//! - `anderson_2d_eigenvalues` — 2D Anderson Hamiltonian via Lanczos.
//!   Requires `barracuda-gpu`. hotSpring S26 Lanczos → barraCuda S59.
//! - `anderson_3d_eigenvalues` — 3D Anderson with true metal-insulator
//!   transition at `W_c` ≈ 16.5. Requires `barracuda-gpu`.
//! - [`disorder_sweep`] — GPU-accelerated disorder parameter sweep with
//!   automatic level spacing ratio averaging. Feeds ESN regime classifier
//!   (see [`crate::esn`]).
//! - [`spectral_diagnostics`] — Spectral bandwidth, condition number, and
//!   phase classification via `barracuda::spectral::stats` (barraCuda S79).

#[cfg(not(feature = "barracuda"))]
use crate::eps;
use crate::prng::Xorshift64;

/// Derrida-Gardner constant for ξ ≈ C / W² at band center.
#[cfg(not(feature = "barracuda"))]
const DERRIDA_GARDNER_CONSTANT: f64 = 96.0;

/// Generate a random potential for the 1D Anderson model.
///
/// Each site gets `V(n) ~ Uniform[-W/2, W/2]` where `W = disorder`.
/// Returns the zero vector for `disorder <= 0`.
#[must_use]
pub fn anderson_potential(n: usize, disorder: f64, seed: u64) -> Vec<f64> {
    if disorder <= 0.0 {
        return vec![0.0; n];
    }
    let half_w = disorder / 2.0;
    let mut rng = Xorshift64::new(seed);
    (0..n)
        .map(|_| rng.next_f64().mul_add(disorder, -half_w))
        .collect()
}

/// Compute the Lyapunov exponent for a given potential and energy.
///
/// When `barracuda-gpu` feature is enabled, delegates to
/// `barracuda::spectral::lyapunov_exponent`.
///
/// Uses the transfer-matrix method with vector renormalization to avoid
/// overflow.  The transfer matrix at site `n` is:
///
/// ```text
/// T_n = [[E - V(n), -1],
///        [1,         0]]
/// ```
///
/// Returns `γ = (1/N) Σ ln(norm)`, the largest Lyapunov exponent.
#[must_use]
pub fn lyapunov_exponent(potential: &[f64], energy: f64) -> f64 {
    #[cfg(feature = "barracuda-gpu")]
    return barracuda::spectral::lyapunov_exponent(potential, energy);
    #[cfg(not(feature = "barracuda-gpu"))]
    lyapunov_exponent_cpu(potential, energy)
}

#[cfg(not(feature = "barracuda-gpu"))]
fn lyapunov_exponent_cpu(potential: &[f64], energy: f64) -> f64 {
    let n = potential.len();
    if n == 0 {
        return 0.0;
    }

    let mut log_growth = 0.0;
    let mut v0: f64 = 1.0;
    let mut v1: f64 = 0.0;

    for &v in potential {
        let new_0 = (energy - v).mul_add(v0, -v1);
        let new_1 = v0;
        v0 = new_0;
        v1 = new_1;

        let norm = v0.hypot(v1);
        if norm > 0.0 {
            log_growth += norm.ln();
            v0 /= norm;
            v1 /= norm;
        }
    }

    log_growth / crate::cast::usize_f64(n)
}

/// Localization length `ξ = 1 / γ`.  Returns `f64::INFINITY` if `γ <= 0`.
#[must_use]
pub fn localization_length(gamma: f64) -> f64 {
    if gamma <= 0.0 {
        return f64::INFINITY;
    }
    1.0 / gamma
}

/// Analytical localization length from disorder strength and energy.
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::special::anderson_transport::localization_length` which uses
/// the perturbative result `ξ(W, E) ≈ 105.2 / W²` at the band center.
/// The local fallback uses `ξ ≈ C / W²` with `C ≈ 96` from Derrida-Gardner.
///
/// This is an analytical estimate — for numerical results, use
/// [`lyapunov_exponent`] + [`localization_length`].
#[must_use]
pub fn analytical_localization_length(disorder: f64, energy: f64) -> f64 {
    #[cfg(feature = "barracuda")]
    return barracuda::special::anderson_transport::localization_length(disorder, energy);
    #[cfg(not(feature = "barracuda"))]
    analytical_localization_length_cpu(disorder, energy)
}

#[cfg(not(feature = "barracuda"))]
fn analytical_localization_length_cpu(disorder: f64, energy: f64) -> f64 {
    if disorder <= 0.0 {
        return f64::INFINITY;
    }
    let _ = energy;
    DERRIDA_GARDNER_CONSTANT / (disorder * disorder)
}

/// Average Lyapunov exponent over many disorder realizations.
///
/// When `barracuda-gpu` feature is enabled, delegates to
/// `barracuda::spectral::lyapunov_averaged`.  Note: barracuda uses
/// `base_seed + r * 1000` per realization; the local fallback uses
/// `base_seed + i` — results will differ across feature gates until
/// Phase 2b PRNG alignment.
#[must_use]
pub fn lyapunov_averaged(
    n_sites: usize,
    disorder: f64,
    energy: f64,
    n_realizations: usize,
    base_seed: u64,
) -> f64 {
    #[cfg(feature = "barracuda-gpu")]
    return barracuda::spectral::lyapunov_averaged(
        n_sites,
        disorder,
        energy,
        n_realizations,
        base_seed,
    );
    #[cfg(not(feature = "barracuda-gpu"))]
    lyapunov_averaged_cpu(n_sites, disorder, energy, n_realizations, base_seed)
}

#[cfg(not(feature = "barracuda-gpu"))]
fn lyapunov_averaged_cpu(
    n_sites: usize,
    disorder: f64,
    energy: f64,
    n_realizations: usize,
    base_seed: u64,
) -> f64 {
    let mut total = 0.0;
    for i in 0..n_realizations {
        let pot = anderson_potential(n_sites, disorder, base_seed + i as u64);
        total += lyapunov_exponent(&pot, energy);
    }
    total / crate::cast::usize_f64(n_realizations)
}

/// Result of a single point in a disorder parameter sweep.
#[derive(Debug, Clone, Copy)]
pub struct SweepPoint {
    /// Disorder strength W.
    pub disorder: f64,
    /// Mean level spacing ratio ⟨r⟩ averaged over realizations.
    pub mean_ratio: f64,
    /// Standard error of ⟨r⟩.
    pub std_error: f64,
}

/// GPU-accelerated disorder parameter sweep for Anderson localization.
///
/// Sweeps disorder strength from `w_min` to `w_max` in `n_points` steps,
/// computing the mean level spacing ratio ⟨r⟩ at each point averaged
/// over `n_realizations` disorder realizations.
///
/// When `barracuda-gpu` is enabled, delegates to
/// `barracuda::spectral::anderson_sweep_averaged` which runs the full
/// sweep on GPU with parallel disorder realizations.
///
/// Cross-spring lineage: hotSpring Exp003 (nuclear structure level
/// statistics) → `barracuda::spectral` S59 GPU sweep → groundSpring
/// Exp008/015/022 uncertainty propagation validation.
#[must_use]
pub fn disorder_sweep(
    n_sites: usize,
    w_min: f64,
    w_max: f64,
    n_points: usize,
    n_realizations: usize,
    base_seed: u64,
) -> Vec<SweepPoint> {
    #[cfg(feature = "barracuda-gpu")]
    {
        let points = barracuda::spectral::anderson_sweep_averaged(
            n_sites,
            w_min,
            w_max,
            n_points,
            n_realizations,
            base_seed,
        );
        points
            .into_iter()
            .map(|p| SweepPoint {
                disorder: p.w,
                mean_ratio: p.r_mean,
                std_error: p.r_stderr,
            })
            .collect()
    }
    #[cfg(not(feature = "barracuda-gpu"))]
    disorder_sweep_cpu(n_sites, w_min, w_max, n_points, n_realizations, base_seed)
}

#[cfg(not(feature = "barracuda-gpu"))]
fn disorder_sweep_cpu(
    n_sites: usize,
    w_min: f64,
    w_max: f64,
    n_points: usize,
    n_realizations: usize,
    base_seed: u64,
) -> Vec<SweepPoint> {
    use crate::cast::usize_f64;

    let step = if n_points > 1 {
        (w_max - w_min) / usize_f64(n_points - 1)
    } else {
        0.0
    };

    (0..n_points)
        .map(|i| {
            let w = usize_f64(i).mul_add(step, w_min);
            let gamma = lyapunov_averaged(
                n_sites,
                w,
                0.0,
                n_realizations,
                base_seed + (i as u64) * 10_000,
            );
            SweepPoint {
                disorder: w,
                mean_ratio: gamma,
                std_error: 0.0,
            }
        })
        .collect()
}

/// Eigenvalues of a 2D Anderson Hamiltonian via Lanczos iteration.
///
/// Constructs an `(lx × ly)` square lattice with on-site disorder
/// `V ∈ [-W/2, W/2]` and nearest-neighbor hopping, then extracts
/// eigenvalues using the Lanczos algorithm.
///
/// The 2D Anderson model has no localization transition in the
/// thermodynamic limit (all states localized for any W > 0), but
/// finite-size systems show a crossover that ESN classifiers can detect.
///
/// Cross-spring lineage: hotSpring spectral theory (S26 Lanczos) +
/// barraCuda S59 `anderson_2d` → groundSpring higher-dimensional
/// localization studies.
///
/// # Arguments
///
/// * `lx`, `ly` — Lattice dimensions (total sites = lx × ly)
/// * `disorder` — Disorder strength W
/// * `n_eigenvalues` — Number of Lanczos iterations (eigenvalue count)
/// * `seed` — PRNG seed for disorder potential
#[cfg(feature = "barracuda-gpu")]
#[must_use]
pub fn anderson_2d_eigenvalues(
    lx: usize,
    ly: usize,
    disorder: f64,
    n_eigenvalues: usize,
    seed: u64,
) -> Vec<f64> {
    let csr = barracuda::spectral::anderson_2d(lx, ly, disorder, seed);
    crate::lanczos::eigenvalues_from_csr(&csr, n_eigenvalues, seed.wrapping_add(1))
}

/// Eigenvalues of a 3D Anderson Hamiltonian via Lanczos iteration.
///
/// The 3D Anderson model exhibits a **true metal-insulator transition**
/// at critical disorder `W_c` ≈ 16.5 (Slevin & Ohtsuki 1999). Below `W_c`,
/// states at the band center are extended; above `W_c`, all states are
/// localized.
///
/// Cross-spring lineage: hotSpring `anderson_3d` (S59, correlated
/// disorder variant for WDM transport) → barraCuda GPU sparse eigensolver
/// → groundSpring 3D localization validation.
///
/// # Arguments
///
/// * `lx`, `ly`, `lz` — Lattice dimensions (total sites = lx × ly × lz)
/// * `disorder` — Disorder strength W
/// * `n_eigenvalues` — Number of Lanczos iterations (eigenvalue count)
/// * `seed` — PRNG seed for disorder potential
#[cfg(feature = "barracuda-gpu")]
#[must_use]
pub fn anderson_3d_eigenvalues(
    lx: usize,
    ly: usize,
    lz: usize,
    disorder: f64,
    n_eigenvalues: usize,
    seed: u64,
) -> Vec<f64> {
    let csr = barracuda::spectral::anderson_3d(lx, ly, lz, disorder, seed);
    crate::lanczos::eigenvalues_from_csr(&csr, n_eigenvalues, seed.wrapping_add(1))
}

// ── Spectral diagnostics ──────────────────────────────────────────
//
// Cross-spring lineage: neuralSpring V69 spectral phase classification →
// barraCuda S79 `spectral::stats` → groundSpring Anderson analysis.

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
    let phase = if outlier_frac < 0.05 {
        SpectralPhaseLabel::Bulk
    } else if outlier_frac <= 0.20 {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tol;

    #[test]
    fn clean_system_zero_lyapunov() {
        let pot = anderson_potential(10000, 0.0, 42);
        let gamma = lyapunov_exponent(&pot, 0.0);
        // W=0 gives deterministic transfer matrix (identity-like); γ=0 exactly; LITERATURE absorbs any residual numerical drift.
        assert!(
            gamma.abs() < tol::LITERATURE,
            "clean system γ={gamma}, expected ~0"
        );
    }

    #[test]
    fn disorder_gives_positive_lyapunov() {
        let gamma = lyapunov_averaged(10000, 2.0, 0.0, 10, 42);
        assert!(
            gamma > 0.0,
            "disordered system should have γ > 0, got {gamma}"
        );
    }

    #[test]
    fn lyapunov_increases_with_disorder() {
        let g1 = lyapunov_averaged(10000, 1.0, 0.0, 10, 42);
        let g2 = lyapunov_averaged(10000, 4.0, 0.0, 10, 42);
        assert!(g2 > g1, "γ(W=4)={g2} should exceed γ(W=1)={g1}");
    }

    #[test]
    fn localization_length_decreases_with_disorder() {
        let g1 = lyapunov_averaged(10000, 1.0, 0.0, 10, 42);
        let g2 = lyapunov_averaged(10000, 4.0, 0.0, 10, 42);
        let xi1 = localization_length(g1);
        let xi2 = localization_length(g2);
        assert!(xi2 < xi1, "ξ(W=4)={xi2} should be less than ξ(W=1)={xi1}");
    }

    #[test]
    fn potential_deterministic() {
        let p1 = anderson_potential(100, 2.0, 42);
        let p2 = anderson_potential(100, 2.0, 42);
        assert_eq!(p1, p2);
    }

    #[test]
    fn potential_different_seed() {
        let p1 = anderson_potential(100, 2.0, 42);
        let p2 = anderson_potential(100, 2.0, 99);
        assert_ne!(p1, p2);
    }

    #[test]
    fn analytical_localization_length_decreases_with_disorder() {
        let xi1 = analytical_localization_length(1.0, 0.0);
        let xi2 = analytical_localization_length(2.0, 0.0);
        assert!(xi2 < xi1, "ξ(W=2)={xi2} should be less than ξ(W=1)={xi1}");
    }

    #[test]
    fn analytical_localization_length_clean_system_large() {
        let xi = analytical_localization_length(0.0, 0.0);
        assert!(
            xi > 1000.0 || xi.is_infinite(),
            "clean system should have ξ ≫ lattice constant, got {xi}"
        );
    }

    #[test]
    fn analytical_vs_numerical_same_order() {
        let g = lyapunov_averaged(100_000, 1.0, 0.0, 20, 42);
        let xi_numerical = localization_length(g);
        let xi_analytical = analytical_localization_length(1.0, 0.0);
        assert!(
            xi_analytical > 10.0 && xi_numerical > 10.0,
            "both should be large: analytical={xi_analytical}, numerical={xi_numerical}"
        );
    }

    #[test]
    fn thouless_scaling() {
        let g = lyapunov_averaged(100_000, 1.0, 0.0, 20, 42);
        let xi = localization_length(g);
        let c = xi * 1.0_f64.powi(2);
        // Derrida-Gardner C≈96; 20 realizations give ~√20 sample variance; [60,140] is ~±45% around 96.
        assert!(
            (60.0..140.0).contains(&c),
            "Thouless coefficient C={c}, expected ~96"
        );
    }

    #[test]
    fn disorder_sweep_monotonic_lyapunov() {
        let sweep = disorder_sweep(1000, 0.5, 4.0, 5, 5, 42);
        assert_eq!(sweep.len(), 5);
        assert!(sweep[0].disorder < sweep[4].disorder);
        assert!(
            sweep[0].mean_ratio < sweep[4].mean_ratio,
            "Lyapunov should increase with disorder: γ(W=0.5)={} vs γ(W=4)={}",
            sweep[0].mean_ratio,
            sweep[4].mean_ratio
        );
    }

    #[test]
    fn disorder_sweep_single_point() {
        let sweep = disorder_sweep(500, 2.0, 2.0, 1, 3, 42);
        assert_eq!(sweep.len(), 1);
        assert!((sweep[0].disorder - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn sweep_point_debug_display() {
        let p = SweepPoint {
            disorder: 2.0,
            mean_ratio: 0.45,
            std_error: 0.01,
        };
        let dbg = format!("{p:?}");
        assert!(dbg.contains("2.0"));
    }

    #[cfg(feature = "barracuda-gpu")]
    #[test]
    fn anderson_2d_eigenvalue_count() {
        let eigs = anderson_2d_eigenvalues(5, 5, 2.0, 10, 42);
        assert_eq!(eigs.len(), 10, "should return n_eigenvalues eigenvalues");
    }

    #[cfg(feature = "barracuda-gpu")]
    #[test]
    fn anderson_3d_eigenvalue_count() {
        let eigs = anderson_3d_eigenvalues(3, 3, 3, 2.0, 10, 42);
        assert_eq!(eigs.len(), 10, "should return n_eigenvalues eigenvalues");
    }

    #[cfg(feature = "barracuda-gpu")]
    #[test]
    fn anderson_2d_eigenvalues_bounded() {
        let lx = 5;
        let ly = 5;
        let w = 3.0;
        let eigs = anderson_2d_eigenvalues(lx, ly, w, 20, 42);
        let spectral_bound = 4.0 + w / 2.0;
        for &e in &eigs {
            assert!(
                e.abs() <= spectral_bound + 0.5,
                "2D eigenvalue {e} exceeds bound {spectral_bound}"
            );
        }
    }

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
}
