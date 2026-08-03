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
//! # barracuda delegation
//!
//! When the `barracuda-gpu` feature is enabled, [`tikhonov_solve`] delegates
//! the linear system solve to `barracuda::linalg::solve_f64_cpu` (Gauss–Jordan
//! with partial pivoting). Falls back to the local Cholesky solver on error.
//!
//! GPU path: the Cholesky solve maps to `barracuda::linalg::cholesky_f64`.
//! V113: matrix products (`KᵀK`, `KᵀG`) are delegated to
//! `barracuda::ops::linalg::GemmF64::execute_gemm_ex` when GPU is available,
//! using `trans_a=true` for the `Kᵀ` operation. Falls back to local CPU
//! implementations for small problems or when no GPU is present.

mod fft;
mod kernel;
mod linalg;
mod tikhonov;

pub use fft::fft_power_spectrum;
pub use kernel::{build_kernel, forward_correlator, gaussian_peak};
pub use tikhonov::{LAMBDA_NOISY, LAMBDA_PARITY, tikhonov_solve, tikhonov_solve_cpu};

/// Find the omega index with maximum reconstructed value.
#[must_use]
pub fn peak_index(rho: &[f64]) -> usize {
    rho.iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
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

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use crate::cast::usize_f64;
    use crate::tol;

    use super::linalg::cholesky_solve;

    /// Noiseless (near-zero) regularization for analytical roundtrip tests.
    const LAMBDA_NOISELESS: f64 = 1e-12;

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

        let rho_rec = tikhonov_solve(&kernel, &g, LAMBDA_NOISELESS, n_tau, n_omega);
        let g_rec = forward_correlator(&kernel, &rho_rec, n_tau, n_omega);
        let r = rmse(&g, &g_rec);
        assert!(r < tol::CDF_APPROX, "noiseless roundtrip RMSE = {r}");
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
        let rho_rec = tikhonov_solve(&kernel, &g, LAMBDA_NOISY, n_tau, n_omega);
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
        assert!((x[0] - 3.0).abs() < tol::STRICT);
        assert!((x[1] - 7.0).abs() < tol::STRICT);
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
            (integral - 1.0).abs() < tol::NORM_2PCT,
            "Gaussian peak should integrate to ~1.0, got {integral}"
        );
    }

    #[test]
    fn fft_single_cosine() {
        let n = 64;
        let d_tau = 0.01;
        let freq_hz = 10.0;
        let correlator: Vec<f64> = (0..n)
            .map(|i| {
                let t = usize_f64(i) * d_tau;
                (std::f64::consts::TAU * freq_hz * t).cos()
            })
            .collect();

        let (frequencies, power) = fft_power_spectrum(&correlator, d_tau);
        let peak_idx = power
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .unwrap()
            .0;

        assert!(
            (frequencies[peak_idx] - freq_hz).abs() < 2.0 / (usize_f64(n) * d_tau),
            "FFT peak at {}, expected ~{freq_hz}",
            frequencies[peak_idx]
        );
    }

    #[test]
    fn fft_dc_signal() {
        let correlator = vec![1.0; 32];
        let (_, power) = fft_power_spectrum(&correlator, 0.1);
        assert!(
            power[0] > power[1..].iter().copied().fold(0.0, f64::max),
            "DC signal should have peak at k=0"
        );
    }

    #[test]
    fn fft_output_length() {
        let correlator = vec![1.0, 0.0, -1.0, 0.0, 0.5, -0.5];
        let (freqs, power) = fft_power_spectrum(&correlator, 0.1);
        let n_padded = correlator.len().next_power_of_two();
        assert_eq!(freqs.len(), n_padded / 2 + 1);
        assert_eq!(power.len(), n_padded / 2 + 1);
    }

    #[test]
    fn tikhonov_cholesky_parity() {
        let n_tau = 15;
        let n_omega = 20;
        let tau: Vec<f64> = (1..=n_tau)
            .map(|i| usize_f64(i) * 2.0 / usize_f64(n_tau))
            .collect();
        let omega: Vec<f64> = (1..=n_omega)
            .map(|i| usize_f64(i) * 6.0 / usize_f64(n_omega))
            .collect();
        let rho_true = gaussian_peak(&omega, 2.5, 0.4, 1.0);
        let kernel = build_kernel(&tau, &omega);
        let g = forward_correlator(&kernel, &rho_true, n_tau, n_omega);

        let rho_rec = tikhonov_solve(&kernel, &g, LAMBDA_PARITY, n_tau, n_omega);
        let g_rec = forward_correlator(&kernel, &rho_rec, n_tau, n_omega);
        let r = rmse(&g, &g_rec);
        assert!(r < tol::RECONSTRUCTION, "Tikhonov roundtrip RMSE = {r}");

        let pi = peak_index(&rho_rec);
        assert!(
            (omega[pi] - 2.5).abs() < 1.0,
            "peak at ω={}, expected ~2.5",
            omega[pi]
        );
    }
}
