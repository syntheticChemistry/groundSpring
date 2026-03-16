// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Linear algebra primitives shared across groundSpring modules.
//!
//! This module contains general-purpose numerical linear algebra that multiple
//! physics modules depend on. Extracting these from domain-specific modules
//! (`transport`, `band_structure`) makes the dependency explicit and allows
//! independent evolution.
//!
//! # Contents
//!
//! - [`tridiag_eigh`] — Eigendecomposition of symmetric tridiagonal matrices
//!   via implicit QL with Wilkinson shifts. O(n²) for tridiagonal vs O(n³)
//!   for dense Jacobi, with higher precision (1e-10 residuals vs 1e-5).
//!
//! # barracuda delegation
//!
//! `tridiag_eigh` stays local: the implicit QL algorithm outperforms
//! barracuda's dense Jacobi `eigh_f64` for tridiagonal matrices.
//! GPU promotion requires a dedicated `BatchedTridiagEigh` — candidate
//! for `ToadStool` absorption.
//!
//! `tridiag_eigh_barracuda` provides a barracuda-delegated path that
//! constructs the dense matrix and calls `barracuda::linalg::eigh_f64`
//! (Jacobi rotation). This is slower than QL for single decompositions
//! but provides a validation cross-check and the foundation for future
//! GPU-batched eigensolvers.

/// Maximum QL iterations before convergence failure.
///
/// 30 iterations is sufficient for all tridiagonal matrices up to n = 10 000
/// in practice. The Wilkinson shift guarantees cubic convergence, so each
/// off-diagonal element converges in O(1) iterations.
/// Reference: Golub & Van Loan (2013) Matrix Computations, §8.3.
const QL_MAX_ITERATIONS: usize = 30;

/// Error type for eigendecomposition failures.
#[derive(Debug, Clone, PartialEq, Eq)]
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
                let threshold = (eps * (d[m].abs() + d[m + 1].abs())).max(crate::eps::UNDERFLOW);
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

/// Barracuda-delegated tridiagonal eigendecomposition.
///
/// Constructs the full dense symmetric matrix from the tridiagonal form
/// and delegates to `barracuda::linalg::eigh_f64` (Jacobi rotation).
/// Returns `(eigenvalues, eigenvectors_flat)` in the same layout as
/// [`tridiag_eigh`].
///
/// # Errors
///
/// Returns [`EighError`] on empty input, dimension mismatch, or barracuda
/// decomposition failure.
#[cfg(feature = "barracuda-gpu")]
pub fn tridiag_eigh_barracuda(
    diag: &[f64],
    offdiag: &[f64],
) -> Result<(Vec<f64>, Vec<f64>), EighError> {
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

    let mut dense = vec![0.0; n * n];
    for i in 0..n {
        dense[i * n + i] = diag[i];
        if i + 1 < n {
            dense[i * n + (i + 1)] = offdiag[i];
            dense[(i + 1) * n + i] = offdiag[i];
        }
    }

    let decomp =
        barracuda::linalg::eigh_f64(&dense, n).map_err(|_| EighError::ConvergenceFailure {
            index: 0,
            max_iterations: QL_MAX_ITERATIONS,
        })?;

    let mut eigenvectors = vec![0.0; n * n];
    for k in 0..n {
        if let Some(ev) = decomp.eigenvector(k) {
            for (row, val) in ev.iter().enumerate() {
                eigenvectors[row * n + k] = *val;
            }
        }
    }

    Ok((decomp.eigenvalues, eigenvectors))
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
    fn trivial_eigh() {
        let (vals, vecs) = tridiag_eigh(&[3.0], &[]).expect("trivial 1x1");
        assert!((vals[0] - 3.0).abs() < tol::EXACT);
        assert!((vecs[0] - 1.0).abs() < tol::EXACT);
    }

    #[test]
    fn two_by_two_eigh() {
        let n = 2;
        let (vals, vecs) = tridiag_eigh(&[0.0, 0.0], &[1.0]).expect("2x2");
        assert!((vals[0] - (-1.0)).abs() < tol::EXACT);
        assert!((vals[1] - 1.0).abs() < tol::EXACT);

        let norm0: f64 = (0..n).map(|row| vecs[row * n] * vecs[row * n]).sum();
        assert!(
            (norm0 - 1.0).abs() < tol::EXACT,
            "eigenvector 0 not normalized"
        );
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
    fn eigh_error_display() {
        let e = EighError::EmptyMatrix;
        assert_eq!(e.to_string(), "matrix must be non-empty");

        let e = EighError::DimensionMismatch {
            diag_len: 3,
            offdiag_len: 5,
        };
        assert!(e.to_string().contains('5'));

        let e = EighError::ConvergenceFailure {
            index: 2,
            max_iterations: 30,
        };
        assert!(e.to_string().contains("30"));
    }

    #[test]
    fn eigh_error_derives() {
        let e1 = EighError::EmptyMatrix;
        let e2 = e1.clone();
        assert_eq!(e1, e2);
        assert_eq!(format!("{e1:?}"), "EmptyMatrix");
    }

    #[test]
    fn orthogonality() {
        let n = 20_usize;
        let diag: Vec<f64> = (0..20_i32).map(|i| f64::from(i) * 0.3).collect();
        let offdiag = vec![1.0; n - 1];
        let (_, vecs) = tridiag_eigh(&diag, &offdiag).expect("20x20");

        for i in 0..n {
            for j in 0..n {
                let dot: f64 = (0..n).map(|k| vecs[k * n + i] * vecs[k * n + j]).sum();
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!(
                    (dot - expected).abs() < tol::ANALYTICAL,
                    "dot({i},{j}) = {dot}, expected {expected}"
                );
            }
        }
    }

    #[test]
    fn eigenvalue_reconstruction() {
        let n = 10_usize;
        let diag: Vec<f64> = (0..10_i32).map(|i| f64::from(i) * 0.5).collect();
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
                let diff = vals[k].mul_add(-vecs[j * n + k], hv[j]).abs();
                assert!(
                    diff < tol::ANALYTICAL,
                    "H*v != λ*v at k={k}, j={j}: diff={diff}"
                );
            }
        }
    }

    #[cfg(feature = "barracuda-gpu")]
    mod barracuda_tests {
        use super::*;

        #[test]
        fn barracuda_eigh_eigenvalue_parity() {
            let n = 10_usize;
            let diag: Vec<f64> = (0..10_i32).map(|i| f64::from(i) * 0.5).collect();
            let offdiag = vec![0.3; n - 1];

            let (vals_ql, _) = tridiag_eigh(&diag, &offdiag).expect("QL");
            let (vals_bc, _) = tridiag_eigh_barracuda(&diag, &offdiag).expect("barracuda");

            for (ql, bc) in vals_ql.iter().zip(&vals_bc) {
                assert!(
                    (ql - bc).abs() < tol::ANALYTICAL,
                    "eigenvalue mismatch: QL={ql}, barracuda={bc}"
                );
            }
        }

        #[test]
        fn barracuda_eigh_orthogonality() {
            let n = 8_usize;
            let diag: Vec<f64> = (0..8_i32).map(|i| f64::from(i) * 0.4).collect();
            let offdiag = vec![1.0; n - 1];
            let (_, vecs) = tridiag_eigh_barracuda(&diag, &offdiag).expect("barracuda");

            for i in 0..n {
                for j in 0..n {
                    let dot: f64 = (0..n).map(|k| vecs[k * n + i] * vecs[k * n + j]).sum();
                    let expected = if i == j { 1.0 } else { 0.0 };
                    // Jacobi rotation has lower precision than QL; LITERATURE for cross-implementation parity.
                    assert!(
                        (dot - expected).abs() < tol::LITERATURE,
                        "barracuda eigenvector dot({i},{j}) = {dot}"
                    );
                }
            }
        }

        #[test]
        fn barracuda_eigh_trivial() {
            let (vals, vecs) = tridiag_eigh_barracuda(&[5.0], &[]).expect("1x1");
            assert!((vals[0] - 5.0).abs() < tol::EXACT);
            assert!((vecs[0] - 1.0).abs() < tol::EXACT);
        }

        #[test]
        fn barracuda_eigh_empty_errors() {
            assert!(tridiag_eigh_barracuda(&[], &[]).is_err());
        }
    }
}
