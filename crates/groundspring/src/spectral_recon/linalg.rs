// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

/// Aᵀ · B where A is `m × k` and B is `m × n`, result is `k × n` (row-major).
///
/// CPU fallback for when GPU GEMM is unavailable. Used by
/// [`super::tikhonov_solve_cpu`] and as fallback in [`super::tikhonov_solve`].
#[expect(
    clippy::many_single_char_names,
    reason = "standard linear algebra notation (m × k × n)"
)]
pub fn mat_transpose_mul(a: &[f64], b: &[f64], m: usize, k: usize, n: usize) -> Vec<f64> {
    let mut c = vec![0.0; k * n];
    for i in 0..k {
        for j in 0..n {
            let mut s = 0.0;
            for l in 0..m {
                s = a[l * k + i].mul_add(b[l * n + j], s);
            }
            c[i * n + j] = s;
        }
    }
    c
}

/// Aᵀ · v where A is `m × n`, v is length `m`, result is length `n`.
///
/// CPU fallback — see [`mat_transpose_mul`].
#[expect(
    clippy::many_single_char_names,
    reason = "standard linear algebra notation (m × n)"
)]
pub fn mat_transpose_vec(a: &[f64], v: &[f64], m: usize, n: usize) -> Vec<f64> {
    let mut r = vec![0.0; n];
    for i in 0..n {
        let mut s = 0.0;
        for l in 0..m {
            s = a[l * n + i].mul_add(v[l], s);
        }
        r[i] = s;
    }
    r
}

/// Cholesky decomposition and solve for SPD system `Ax = b`.
///
/// Returns x. Panics if A is not positive definite.
#[expect(
    clippy::many_single_char_names,
    reason = "standard Cholesky notation (L, i, j, k, n)"
)]
pub fn cholesky_solve(a: &[f64], b: &[f64], n: usize) -> Vec<f64> {
    let mut l: Vec<f64> = vec![0.0; n * n];

    for i in 0..n {
        for j in 0..=i {
            let mut s: f64 = 0.0;
            for k in 0..j {
                s = l[i * n + k].mul_add(l[j * n + k], s);
            }
            if i == j {
                let diag = a[i * n + i] - s;
                assert!(diag > 0.0, "Matrix not positive definite at row {i}");
                l[i * n + j] = diag.sqrt();
            } else {
                l[i * n + j] = (a[i * n + j] - s) / l[j * n + j];
            }
        }
    }

    let mut y = vec![0.0; n];
    for i in 0..n {
        let mut s: f64 = 0.0;
        for j in 0..i {
            s = l[i * n + j].mul_add(y[j], s);
        }
        y[i] = (b[i] - s) / l[i * n + i];
    }

    let mut x = vec![0.0; n];
    for i in (0..n).rev() {
        let mut s: f64 = 0.0;
        for j in (i + 1)..n {
            s = l[j * n + i].mul_add(x[j], s);
        }
        x[i] = (y[i] - s) / l[i * n + i];
    }

    x
}
