// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Spectral function reconstruction via Tikhonov regularization.
//!
//! Reconstructs a spectral function ρ(ω) from a noisy Euclidean correlator
//! G(τ) using the Laplace-transform kernel K(τ,ω) = exp(−τω).
//!
//! The Tikhonov solution minimizes ‖Kρ − G‖² + λ‖ρ‖² and is given by
//! `ρ = (KᵀK + λI)⁻¹ KᵀG`, solved via Cholesky factorization.
//!
//! Reference: Bazavov et al. (2025) arXiv 2501.12259
//!
//! # `barracuda` delegation
//!
//! `build_kernel` and `tikhonov_solve` are natural GPU candidates:
//! kernel construction is embarrassingly parallel, and the Cholesky solve
//! maps to batched dense linear algebra.

/// Build the Laplace-transform kernel matrix (row-major, `n_tau × n_omega`).
///
/// `K[i,j] = exp(−τ_i ω_j) Δω`
#[must_use]
pub fn build_kernel(tau: &[f64], omega: &[f64]) -> Vec<f64> {
    let n_tau = tau.len();
    let n_omega = omega.len();
    let d_omega = if n_omega > 1 {
        omega[1] - omega[0]
    } else {
        1.0
    };
    let mut k = vec![0.0; n_tau * n_omega];
    for (i, &t) in tau.iter().enumerate() {
        for (j, &w) in omega.iter().enumerate() {
            k[i * n_omega + j] = (-t * w).exp() * d_omega;
        }
    }
    k
}

/// Forward correlator: G = K · ρ  (matrix-vector product).
#[must_use]
pub fn forward_correlator(kernel: &[f64], rho: &[f64], n_tau: usize, n_omega: usize) -> Vec<f64> {
    let mut g = vec![0.0; n_tau];
    for i in 0..n_tau {
        let mut s = 0.0;
        for j in 0..n_omega {
            s = kernel[i * n_omega + j].mul_add(rho[j], s);
        }
        g[i] = s;
    }
    g
}

/// Gaussian spectral peak: `ρ(ω) = A / (σ√(2π)) exp(−(ω−ω₀)²/(2σ²))`.
#[must_use]
pub fn gaussian_peak(omega: &[f64], center: f64, width: f64, amplitude: f64) -> Vec<f64> {
    let norm = amplitude / (width * std::f64::consts::TAU.sqrt());
    omega
        .iter()
        .map(|&w| {
            let z = (w - center) / width;
            norm * (-0.5 * z * z).exp()
        })
        .collect()
}

/// Tikhonov-regularized reconstruction.
///
/// Solves `(KᵀK + λI) ρ = KᵀG` via Cholesky decomposition.
///
/// # Panics
///
/// Panics if dimensions are inconsistent or Cholesky fails.
#[must_use]
pub fn tikhonov_solve(
    kernel: &[f64],
    data: &[f64],
    lambda: f64,
    n_tau: usize,
    n_omega: usize,
) -> Vec<f64> {
    let ktk = mat_transpose_mul(kernel, kernel, n_tau, n_omega, n_omega);
    let ktg = mat_transpose_vec(kernel, data, n_tau, n_omega);

    let mut a = ktk;
    for i in 0..n_omega {
        a[i * n_omega + i] += lambda;
    }

    cholesky_solve(&a, &ktg, n_omega)
}

/// Find the omega index with maximum reconstructed value.
#[must_use]
pub fn peak_index(rho: &[f64]) -> usize {
    rho.iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map_or(0, |(i, _)| i)
}

/// RMSE between two vectors.
///
/// Delegates to [`crate::stats::rmse`], which in turn delegates to
/// `barracuda::stats::rmse` when the `barracuda` feature is enabled.
///
/// # Panics
///
/// Panics if `a` and `b` have different lengths.
#[must_use]
pub fn rmse(a: &[f64], b: &[f64]) -> f64 {
    crate::stats::rmse(a, b)
}

/// Aᵀ · B where A is `m × k` and B is `m × n`, result is `k × n` (row-major).
#[expect(
    clippy::many_single_char_names,
    reason = "standard linear algebra notation (m × k × n)"
)]
fn mat_transpose_mul(a: &[f64], b: &[f64], m: usize, k: usize, n: usize) -> Vec<f64> {
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
#[expect(
    clippy::many_single_char_names,
    reason = "standard linear algebra notation (m × n)"
)]
fn mat_transpose_vec(a: &[f64], v: &[f64], m: usize, n: usize) -> Vec<f64> {
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
fn cholesky_solve(a: &[f64], b: &[f64], n: usize) -> Vec<f64> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cast::usize_f64;

    #[test]
    fn kernel_shape() {
        let tau = vec![0.1, 0.5, 1.0];
        let omega = vec![1.0, 2.0, 3.0, 4.0];
        let k = build_kernel(&tau, &omega);
        assert_eq!(k.len(), 12);
    }

    #[test]
    fn forward_noiseless_roundtrip() {
        let n_tau = 20;
        let n_omega = 40;
        let tau: Vec<f64> = (1..=n_tau)
            .map(|i| usize_f64(i) * 2.0 / usize_f64(n_tau))
            .collect();
        let omega: Vec<f64> = (1..=n_omega)
            .map(|i| usize_f64(i) * 8.0 / usize_f64(n_omega))
            .collect();
        let rho = gaussian_peak(&omega, 3.0, 0.5, 1.0);
        let kernel = build_kernel(&tau, &omega);
        let g = forward_correlator(&kernel, &rho, n_tau, n_omega);

        let rho_rec = tikhonov_solve(&kernel, &g, 1e-12, n_tau, n_omega);
        let g_rec = forward_correlator(&kernel, &rho_rec, n_tau, n_omega);
        let r = rmse(&g, &g_rec);
        assert!(r < 1e-6, "noiseless roundtrip RMSE = {r}");
    }

    #[test]
    fn peak_detected() {
        let n_tau = 20;
        let n_omega = 40;
        let tau: Vec<f64> = (1..=n_tau)
            .map(|i| usize_f64(i) * 2.0 / usize_f64(n_tau))
            .collect();
        let omega: Vec<f64> = (1..=n_omega)
            .map(|i| usize_f64(i) * 8.0 / usize_f64(n_omega))
            .collect();
        let rho = gaussian_peak(&omega, 3.0, 0.5, 1.0);
        let kernel = build_kernel(&tau, &omega);
        let g = forward_correlator(&kernel, &rho, n_tau, n_omega);
        let rho_rec = tikhonov_solve(&kernel, &g, 1e-6, n_tau, n_omega);
        let pi = peak_index(&rho_rec);
        assert!(
            (omega[pi] - 3.0).abs() < 1.0,
            "peak at ω={}, expected ~3.0",
            omega[pi]
        );
    }

    #[test]
    fn cholesky_identity() {
        let a = vec![1.0, 0.0, 0.0, 1.0];
        let b = vec![3.0, 7.0];
        let x = cholesky_solve(&a, &b, 2);
        assert!((x[0] - 3.0).abs() < 1e-14);
        assert!((x[1] - 7.0).abs() < 1e-14);
    }

    #[test]
    fn gaussian_peak_normalized() {
        let n = 1000;
        let omega: Vec<f64> = (1..=n)
            .map(|i| usize_f64(i) * 10.0 / usize_f64(n))
            .collect();
        let dw = omega[1] - omega[0];
        let rho = gaussian_peak(&omega, 5.0, 1.0, 1.0);
        let integral: f64 = rho.iter().map(|&r| r * dw).sum();
        assert!(
            (integral - 1.0).abs() < 0.02,
            "Gaussian peak should integrate to ~1.0, got {integral}"
        );
    }
}
