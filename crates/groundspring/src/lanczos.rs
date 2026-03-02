// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Lanczos eigensolver for large sparse systems.
//!
//! Provides Lanczos iteration for extracting eigenvalues from large sparse
//! Hamiltonians that are too big for dense diagonalization. Primary use case:
//! Anderson 2D/3D localization (100×100 → 10 000 × 10 000 Hamiltonian).
//!
//! # Cross-spring lineage
//!
//! - **hotSpring S26** — Lanczos tridiagonalization for nuclear structure
//!   (harmonic oscillator many-body, Exp 001). Validated against LAPACK
//!   `dstebz`/`dstein` to machine precision.
//! - **`ToadStool` S59** — Absorbed as `barracuda::spectral::lanczos` with
//!   GPU-accelerated `SpMV` kernel (`spmv_csr_f64.wgsl`). The Lanczos
//!   vectors stay on-device; only tridiagonal scalars (α, β) return to host.
//! - **groundSpring** — Wraps for Anderson 2D/3D localization studies
//!   where the Hamiltonian dimension (L² or L³) exceeds dense solver limits.
//!
//! # barracuda delegation
//!
//! This module requires the `barracuda-gpu` feature. The Lanczos iteration
//! is inherently sequential (each step depends on the previous), but the
//! sparse matrix-vector product within each step is GPU-accelerated via
//! `barracuda::spectral::lanczos`.

/// Compute eigenvalues of a sparse symmetric matrix via Lanczos iteration.
///
/// Takes the CSR (Compressed Sparse Row) components of a sparse matrix
/// and returns approximate eigenvalues. The number of Lanczos iterations
/// controls how many eigenvalues are resolved (more iterations → more
/// eigenvalues, up to `n`).
///
/// # Arguments
///
/// * `n` — Matrix dimension (n × n)
/// * `row_ptr` — CSR row pointers (length n + 1)
/// * `col_idx` — CSR column indices
/// * `values` — CSR non-zero values
/// * `n_iterations` — Number of Lanczos iterations (controls eigenvalue count)
/// * `seed` — PRNG seed for the initial random vector
#[cfg(feature = "barracuda-gpu")]
#[must_use]
pub fn sparse_eigenvalues(
    n: usize,
    row_ptr: &[usize],
    col_idx: &[usize],
    values: &[f64],
    n_iterations: usize,
    seed: u64,
) -> Vec<f64> {
    let csr = barracuda::spectral::SpectralCsrMatrix {
        n,
        row_ptr: row_ptr.to_vec(),
        col_idx: col_idx.to_vec(),
        values: values.to_vec(),
    };
    let tridiag = barracuda::spectral::lanczos(&csr, n_iterations, seed);
    barracuda::spectral::lanczos_eigenvalues(&tridiag)
}

/// Compute eigenvalues of a sparse symmetric matrix from pre-built CSR.
///
/// This is the internal entry point used by [`crate::anderson`] for 2D/3D
/// Hamiltonians. Avoids the CSR copy overhead when the caller already has
/// a `SpectralCsrMatrix`.
#[cfg(feature = "barracuda-gpu")]
pub(crate) fn eigenvalues_from_csr(
    csr: &barracuda::spectral::SpectralCsrMatrix,
    n_iterations: usize,
    seed: u64,
) -> Vec<f64> {
    let tridiag = barracuda::spectral::lanczos(csr, n_iterations, seed);
    barracuda::spectral::lanczos_eigenvalues(&tridiag)
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "barracuda-gpu")]
    use super::*;

    #[test]
    #[cfg(feature = "barracuda-gpu")]
    fn lanczos_2x2_identity() {
        let row_ptr = vec![0, 1, 2];
        let col_idx = vec![0, 1];
        let values = vec![1.0, 1.0];
        let eigs = sparse_eigenvalues(2, &row_ptr, &col_idx, &values, 2, 42);
        assert_eq!(eigs.len(), 2);
        for &e in &eigs {
            assert!((e - 1.0).abs() < 1e-10, "identity eigenvalue = {e}");
        }
    }

    #[test]
    #[cfg(feature = "barracuda-gpu")]
    fn lanczos_tridiagonal_known() {
        let n = 5;
        let mut row_ptr = vec![0usize];
        let mut col_idx = Vec::new();
        let mut values = Vec::new();
        for i in 0..n {
            if i > 0 {
                col_idx.push(i - 1);
                values.push(1.0);
            }
            col_idx.push(i);
            values.push(0.0);
            if i + 1 < n {
                col_idx.push(i + 1);
                values.push(1.0);
            }
            row_ptr.push(col_idx.len());
        }
        let eigs = sparse_eigenvalues(n, &row_ptr, &col_idx, &values, n, 42);
        assert_eq!(eigs.len(), n);
        let mut sorted = eigs;
        sorted.sort_by(f64::total_cmp);
        assert!(
            sorted[0] > -2.1 && sorted[0] < -1.5,
            "min eigenvalue plausible"
        );
        assert!(
            sorted[n - 1] > 1.5 && sorted[n - 1] < 2.1,
            "max eigenvalue plausible"
        );
    }
}
