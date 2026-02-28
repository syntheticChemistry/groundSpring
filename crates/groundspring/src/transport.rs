// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Wavepacket transport in 1D tight-binding models.
//!
//! Computes the mean square displacement (MSD) of a wavepacket evolving under
//! Schrödinger dynamics in a tridiagonal Hamiltonian. The transport exponent β
//! extracted from σ(t) ~ t^β distinguishes ballistic (β=1), diffusive (β=0.5),
//! and localized (β=0) regimes.
//!
//! # References
//!
//! - Kachkovskiy (2016) Comm Math Phys 345:659-673
//! - Jitomirskaya & Kachkovskiy (2018) JEMS 21:777-795
//!
//! # Linear algebra
//!
//! The tridiagonal eigensolver ([`tridiag_eigh`] / [`EighError`]) lives in
//! [`crate::linalg`] and is re-exported here for backward compatibility.
//! It is also used by [`crate::band_structure`].

use crate::cast::usize_f64;

pub use crate::linalg::{tridiag_eigh, EighError};

/// Minimum MSD threshold for log-log regression.
///
/// Values below this are excluded from log-log regression to avoid
/// `ln(0)` and numerical noise dominating the fit. 1e-20 is ~44 orders
/// below typical MSD values and safely above the f64 denormal range.
const MSD_MIN_THRESHOLD: f64 = 1e-20;

/// Compute the mean square displacement of a wavepacket at a given time.
///
/// Given the eigendecomposition (eigenvalues, flat `n×n` eigenvector matrix)
/// of the Hamiltonian, evolves an initial delta-function wavepacket at
/// `init_site` to time `t` and returns `(msd, normalization)`.
///
/// The eigenvector matrix is row-major: element `(row, col)` is at
/// `eigenvectors[row * n + col]`.
///
/// # Theory
///
/// `ψ_j(t) = Σ_k U_{j,k} U_{n₀,k} exp(-i E_k t)`
/// `P_j(t) = |ψ_j(t)|²`
/// `σ²(t) = Σ_j (j - n₀)² P_j(t)`
///
/// # Panics
///
/// Panics if `init_site >= eigenvalues.len()`.
#[must_use]
pub fn wavepacket_msd(
    eigenvalues: &[f64],
    eigenvectors: &[f64],
    init_site: usize,
    time: f64,
) -> (f64, f64) {
    let n = eigenvalues.len();
    assert!(init_site < n, "init_site must be < n");

    let coeffs: Vec<f64> = (0..n).map(|k| eigenvectors[init_site * n + k]).collect();

    let mut msd = 0.0;
    let mut norm = 0.0;
    let init_f = usize_f64(init_site);

    for j in 0..n {
        let mut re = 0.0;
        let mut im = 0.0;
        for k in 0..n {
            let u_jk = eigenvectors[j * n + k];
            let c_k = coeffs[k];
            let phase = eigenvalues[k] * time;
            re += u_jk * c_k * phase.cos();
            im -= u_jk * c_k * phase.sin();
        }
        let prob = re.mul_add(re, im * im);
        let displacement = usize_f64(j) - init_f;
        msd += displacement * displacement * prob;
        norm += prob;
    }

    (msd, norm)
}

/// Extract the transport exponent β from MSD data via log-log linear regression.
///
/// Fits log(σ(t)) = β log(t) + const, where σ = √MSD.
/// Returns the slope β.
///
/// # Panics
///
/// Panics if `times` and `msds` have different lengths.
#[must_use]
pub fn transport_exponent(times: &[f64], msds: &[f64]) -> f64 {
    assert_eq!(times.len(), msds.len(), "times and msds must match");

    let (log_t, log_sigma): (Vec<f64>, Vec<f64>) = times
        .iter()
        .zip(msds.iter())
        .filter(|(&t, &m)| t > 0.0 && m > MSD_MIN_THRESHOLD)
        .map(|(&t, &m)| (t.ln(), 0.5 * m.ln()))
        .unzip();

    crate::stats::fit_linear(&log_t, &log_sigma).map_or(0.0, |f| f.slope)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn almost_mathieu_diag_offdiag(
        n: usize,
        coupling: f64,
        alpha: f64,
        theta: f64,
    ) -> (Vec<f64>, Vec<f64>) {
        let diag: Vec<f64> = (0..n)
            .map(|i| {
                coupling
                    * (2.0 * std::f64::consts::PI * alpha)
                        .mul_add(usize_f64(i), theta)
                        .cos()
            })
            .collect();
        let offdiag = vec![1.0; n - 1];
        (diag, offdiag)
    }

    #[test]
    fn ballistic_transport() {
        let n = 101;
        let (diag, offdiag) = almost_mathieu_diag_offdiag(n, 0.5, 0.618_033_988_749_894_9, 0.0);
        let (evals, evecs) = tridiag_eigh(&diag, &offdiag).expect("ballistic");

        let times = [1.0, 2.0, 5.0, 10.0, 20.0];
        let msds: Vec<f64> = times
            .iter()
            .map(|&t| wavepacket_msd(&evals, &evecs, 50, t).0)
            .collect();

        let beta = transport_exponent(&times, &msds);
        // Ballistic phase has β∈[0.8,1.0]; 0.5 is a conservative lower bound separating ballistic from diffusive.
        assert!(beta > 0.5, "ballistic β should be > 0.5, got {beta}");
    }

    #[test]
    fn localized_transport() {
        let n = 101;
        let (diag, offdiag) = almost_mathieu_diag_offdiag(n, 4.0, 0.618_033_988_749_894_9, 0.0);
        let (evals, evecs) = tridiag_eigh(&diag, &offdiag).expect("localized");

        let times = [1.0, 5.0, 10.0, 20.0, 40.0];
        let msds: Vec<f64> = times
            .iter()
            .map(|&t| wavepacket_msd(&evals, &evecs, 50, t).0)
            .collect();

        let beta = transport_exponent(&times, &msds);
        // Localized phase has β≈0; 0.3 separates from diffusive (β=0.5).
        assert!(beta < 0.3, "localized β should be < 0.3, got {beta}");
    }

    #[test]
    fn normalization_preserved() {
        let n = 51;
        let (diag, offdiag) = almost_mathieu_diag_offdiag(n, 1.0, 0.618_033_988_749_894_9, 0.0);
        let (evals, evecs) = tridiag_eigh(&diag, &offdiag).expect("normalization");

        for &t in &[0.0, 1.0, 5.0, 20.0] {
            let (_, norm) = wavepacket_msd(&evals, &evecs, 25, t);
            // Unitary evolution preserves norm to machine precision; 1e-8 absorbs accumulated rounding in n=51 sum.
            assert!((norm - 1.0).abs() < 1e-8, "normalization at t={t}: {norm}");
        }
    }

    #[test]
    fn transport_exponent_linear() {
        let times = [1.0, 2.0, 4.0, 8.0, 16.0];
        let msds: Vec<f64> = times.iter().map(|&t| t * t).collect();
        let beta = transport_exponent(&times, &msds);
        // Regression on exact σ²=t² data should give β=1.0 within floating-point regression error.
        assert!(
            (beta - 1.0).abs() < 0.01,
            "β for σ²~t² should be 1.0, got {beta}"
        );
    }

    #[test]
    fn transport_exponent_constant() {
        let times = [1.0, 2.0, 4.0, 8.0, 16.0];
        let msds = [5.0; 5];
        let beta = transport_exponent(&times, &msds);
        // Same as transport_exponent_linear: regression on constant MSD gives β=0.0 within floating-point error.
        assert!(
            beta.abs() < 0.01,
            "β for constant MSD should be 0.0, got {beta}"
        );
    }

    #[test]
    fn transport_exponent_insufficient_data() {
        assert!(transport_exponent(&[1.0], &[1.0]).abs() < f64::EPSILON);
        assert!(transport_exponent(&[], &[]).abs() < f64::EPSILON);
    }

    #[test]
    fn transport_exponent_filters_nonpositive_time() {
        let times = [0.0, -1.0, 1.0, 2.0, 4.0];
        let msds = [1.0, 1.0, 1.0, 4.0, 16.0];
        let beta = transport_exponent(&times, &msds);
        assert!(beta > 0.5, "should still compute from valid entries");
    }

    #[test]
    fn transport_exponent_filters_tiny_msd() {
        let times = [1.0, 2.0, 4.0, 8.0];
        let msds = [1e-30, 1e-25, 4.0, 16.0];
        let beta = transport_exponent(&times, &msds);
        assert!(beta.is_finite());
    }

    #[test]
    fn transport_exponent_identical_times() {
        let times = [1.0, 1.0, 1.0];
        let msds = [1.0, 2.0, 3.0];
        let beta = transport_exponent(&times, &msds);
        assert!(beta.abs() < f64::EPSILON);
    }

    #[test]
    fn msd_at_time_zero() {
        let n = 21;
        let (diag, offdiag) = almost_mathieu_diag_offdiag(n, 1.0, 0.618_033_988_749_894_9, 0.0);
        let (evals, evecs) = tridiag_eigh(&diag, &offdiag).expect("msd at t=0");
        let (msd, norm) = wavepacket_msd(&evals, &evecs, 10, 0.0);
        assert!(msd.abs() < 1e-10, "MSD at t=0 should be 0, got {msd}");
        assert!((norm - 1.0).abs() < 1e-10);
    }
}
