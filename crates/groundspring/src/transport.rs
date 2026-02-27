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
//! # Future GPU path
//!
//! Future: `tridiag_eigh` could delegate to barracuda's eigenvector primitives
//! when available.

use crate::cast::usize_f64;

/// Maximum QL iterations before convergence failure.
const QL_MAX_ITERATIONS: usize = 30;
/// Minimum MSD threshold for log-log regression (avoids log(0)).
const MSD_MIN_THRESHOLD: f64 = 1e-20;
/// Denominator epsilon for regression singularity check.
const REGRESSION_EPSILON: f64 = 1e-30;

/// Error type for eigendecomposition failures.
#[derive(Debug)]
pub enum EighError {
    /// The matrix was empty.
    EmptyMatrix,
    /// Off-diagonal length did not match `diag.len() - 1`.
    DimensionMismatch {
        /// Length of diagonal.
        diag_len: usize,
        /// Length of off-diagonal.
        offdiag_len: usize,
    },
    /// QL algorithm failed to converge within the iteration budget.
    ConvergenceFailure {
        /// Index of the sub-diagonal element that did not converge.
        index: usize,
        /// Maximum iterations attempted.
        max_iterations: usize,
    },
}

impl std::fmt::Display for EighError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyMatrix => write!(f, "matrix must be non-empty"),
            Self::DimensionMismatch {
                diag_len,
                offdiag_len,
            } => write!(
                f,
                "offdiag length {offdiag_len} must equal diag length {diag_len} minus 1"
            ),
            Self::ConvergenceFailure {
                index,
                max_iterations,
            } => write!(
                f,
                "QL algorithm failed to converge at index {index} after {max_iterations} iterations"
            ),
        }
    }
}

impl std::error::Error for EighError {}

/// Eigendecomposition of a symmetric tridiagonal matrix via implicit QL
/// algorithm with Wilkinson shifts.
///
/// Returns `(eigenvalues, eigenvectors_flat)` where `eigenvectors_flat` is
/// an `n × n` row-major flat buffer. Row `i` of the matrix holds `U[i][:]`,
/// so the j-th component of eigenvector `k` is at `eigenvectors_flat[j * n + k]`.
///
/// # Errors
///
/// Returns [`EighError`] if the matrix is empty, dimensions mismatch, or the
/// QL algorithm fails to converge.
pub fn tridiag_eigh(diag: &[f64], offdiag: &[f64]) -> Result<(Vec<f64>, Vec<f64>), EighError> {
    let n = diag.len();
    if n == 0 {
        return Err(EighError::EmptyMatrix);
    }
    if offdiag.len() != n - 1 {
        return Err(EighError::DimensionMismatch {
            diag_len: n,
            offdiag_len: offdiag.len(),
        });
    }

    if n == 1 {
        return Ok((vec![diag[0]], vec![1.0]));
    }

    let mut d = diag.to_vec();
    let mut e = vec![0.0; n];
    for (i, &val) in offdiag.iter().enumerate() {
        e[i] = val;
    }

    let mut z = vec![0.0; n * n];
    for i in 0..n {
        z[i * n + i] = 1.0;
    }

    implicit_ql(&mut d, &mut e, &mut z, n)?;

    sort_eigenpairs(&mut d, &mut z, n);

    Ok((d, z))
}

/// Implicit QL algorithm for symmetric tridiagonal eigenvalue/eigenvector
/// computation. Modifies `d` (diagonal → eigenvalues), `e` (off-diagonal →
/// zeros), and `z` (flat `n×n` identity → eigenvector columns).
#[expect(
    clippy::many_single_char_names,
    reason = "standard QL algorithm notation (LAPACK dsteqr convention)"
)]
fn implicit_ql(d: &mut [f64], e: &mut [f64], z: &mut [f64], n: usize) -> Result<(), EighError> {
    let eps = f64::EPSILON;

    for l in 0..n {
        let mut iter_count = 0;
        loop {
            let mut m = l;
            while m < n - 1 {
                let threshold = eps * (d[m].abs() + d[m + 1].abs());
                if e[m].abs() <= threshold {
                    break;
                }
                m += 1;
            }

            if m == l {
                break;
            }

            if iter_count >= QL_MAX_ITERATIONS {
                return Err(EighError::ConvergenceFailure {
                    index: l,
                    max_iterations: QL_MAX_ITERATIONS,
                });
            }
            iter_count += 1;

            let g0 = (d[l + 1] - d[l]) / (2.0 * e[l]);
            let r0 = g0.hypot(1.0);
            let mut g = d[m] - d[l] + e[l] / (g0 + r0.copysign(g0));

            let mut c = 1.0;
            let mut s = 1.0;
            let mut p = 0.0;

            for i in (l..m).rev() {
                let f = s * e[i];
                let b = c * e[i];

                let r_rot;
                if f.abs() >= g.abs() {
                    c = g / f;
                    r_rot = c.hypot(1.0);
                    e[i + 1] = f * r_rot;
                    s = 1.0 / r_rot;
                    c *= s;
                } else {
                    s = f / g;
                    r_rot = s.hypot(1.0);
                    e[i + 1] = g * r_rot;
                    c = 1.0 / r_rot;
                    s *= c;
                }

                let gi = d[i + 1] - p;
                let r_val = (d[i] - gi).mul_add(s, 2.0 * c * b);
                p = s * r_val;
                d[i + 1] = gi + p;
                g = c.mul_add(r_val, -b);

                for row_idx in 0..n {
                    let f_z = z[row_idx * n + i + 1];
                    z[row_idx * n + i + 1] = s.mul_add(z[row_idx * n + i], c * f_z);
                    z[row_idx * n + i] = c.mul_add(z[row_idx * n + i], -(s * f_z));
                }
            }

            d[l] -= p;
            e[l] = g;
            e[m] = 0.0;
        }
    }
    Ok(())
}

/// Sort eigenvalues in ascending order and permute eigenvectors accordingly.
fn sort_eigenpairs(d: &mut [f64], z: &mut [f64], n: usize) {
    let mut indices: Vec<usize> = (0..n).collect();
    indices.sort_unstable_by(|&a, &b| d[a].total_cmp(&d[b]));

    let d_sorted: Vec<f64> = indices.iter().map(|&i| d[i]).collect();
    d.copy_from_slice(&d_sorted);

    let z_copy = z.to_vec();
    for row_idx in 0..n {
        for (new_col, &old_col) in indices.iter().enumerate() {
            z[row_idx * n + new_col] = z_copy[row_idx * n + old_col];
        }
    }
}

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

    let valid: Vec<(f64, f64)> = times
        .iter()
        .zip(msds.iter())
        .filter(|(&t, &m)| t > 0.0 && m > MSD_MIN_THRESHOLD)
        .map(|(&t, &m)| (t.ln(), 0.5 * m.ln()))
        .collect();

    if valid.len() < 2 {
        return 0.0;
    }

    let n = usize_f64(valid.len());
    let sx: f64 = valid.iter().map(|(x, _)| x).sum();
    let sy: f64 = valid.iter().map(|(_, y)| y).sum();
    let sxx: f64 = valid.iter().map(|(x, _)| x * x).sum();
    let sxy: f64 = valid.iter().map(|(x, y)| x * y).sum();

    let denom = n.mul_add(sxx, -(sx * sx));
    if denom.abs() < REGRESSION_EPSILON {
        return 0.0;
    }

    n.mul_add(sxy, -(sx * sy)) / denom
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
    fn trivial_eigh() {
        let (vals, vecs) = tridiag_eigh(&[3.0], &[]).expect("trivial 1x1");
        assert!((vals[0] - 3.0).abs() < 1e-14);
        assert!((vecs[0] - 1.0).abs() < 1e-14);
    }

    #[test]
    fn two_by_two_eigh() {
        let n = 2;
        let (vals, vecs) = tridiag_eigh(&[0.0, 0.0], &[1.0]).expect("2x2");
        assert!((vals[0] - (-1.0)).abs() < 1e-12);
        assert!((vals[1] - 1.0).abs() < 1e-12);

        let norm0: f64 = (0..n).map(|row| vecs[row * n] * vecs[row * n]).sum();
        assert!((norm0 - 1.0).abs() < 1e-12, "eigenvector 0 not normalized");
    }

    #[test]
    fn empty_matrix_returns_error() {
        assert!(tridiag_eigh(&[], &[]).is_err());
    }

    #[test]
    fn dimension_mismatch_returns_error() {
        assert!(tridiag_eigh(&[1.0, 2.0], &[]).is_err());
    }

    #[test]
    fn orthogonality() {
        let n = 20;
        let (diag, offdiag) = almost_mathieu_diag_offdiag(n, 1.0, 0.618_033_988_749_894_9, 0.0);
        let (_, vecs) = tridiag_eigh(&diag, &offdiag).expect("20x20");

        for i in 0..n {
            for j in 0..n {
                let dot: f64 = (0..n).map(|k| vecs[k * n + i] * vecs[k * n + j]).sum();
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!(
                    (dot - expected).abs() < 1e-8,
                    "dot({i},{j}) = {dot}, expected {expected}"
                );
            }
        }
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
        assert_eq!(transport_exponent(&[1.0], &[1.0]), 0.0);
        assert_eq!(transport_exponent(&[], &[]), 0.0);
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
        assert_eq!(beta, 0.0);
    }

    #[test]
    fn eigh_error_display() {
        let e = EighError::EmptyMatrix;
        assert_eq!(format!("{e}"), "matrix must be non-empty");

        let e = EighError::DimensionMismatch {
            diag_len: 3,
            offdiag_len: 5,
        };
        assert!(format!("{e}").contains("5"));

        let e = EighError::ConvergenceFailure {
            index: 2,
            max_iterations: 30,
        };
        assert!(format!("{e}").contains("30"));
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

    #[test]
    fn eigenvalue_reconstruction() {
        let n = 10;
        let diag: Vec<f64> = (0..n).map(|i| usize_f64(i) * 0.5).collect();
        let offdiag = vec![0.3; n - 1];
        let (vals, vecs) = tridiag_eigh(&diag, &offdiag).expect("recon");

        for k in 0..n {
            let mut hv = vec![0.0; n];
            for j in 0..n {
                hv[j] += diag[j] * vecs[j * n + k];
                if j > 0 {
                    hv[j] += offdiag[j - 1] * vecs[(j - 1) * n + k];
                }
                if j + 1 < n {
                    hv[j] += offdiag[j] * vecs[(j + 1) * n + k];
                }
            }
            for j in 0..n {
                let diff = (hv[j] - vals[k] * vecs[j * n + k]).abs();
                assert!(diff < 1e-10, "H*v != λ*v at k={k}, j={j}: diff={diff}");
            }
        }
    }
}
