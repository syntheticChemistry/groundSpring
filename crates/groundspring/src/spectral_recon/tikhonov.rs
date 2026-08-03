// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

use super::linalg::{cholesky_solve, mat_transpose_mul, mat_transpose_vec};

/// Tikhonov regularization strength for noisy correlators (λ = 1e-6).
///
/// Balances noise suppression against reconstruction bias. Suitable for
/// synthetic benchmarks with O(1%) noise. Used by validation binaries
/// and GPU parity checks.
///
/// Provenance: grid-search over λ ∈ [1e-8, 1e-2] on Exp 019 synthetic
/// correlator; 1e-6 minimizes RMSE on the noiseless→noisy degradation
/// curve. See `control/spectral_recon/spectral_reconstruction.py`.
pub const LAMBDA_NOISY: f64 = 1e-6;

/// Tikhonov regularization strength for CPU↔GPU parity checks (λ = 1e-8).
///
/// Weaker regularization than [`LAMBDA_NOISY`] to stress-test numerical
/// agreement between Cholesky solvers on different substrates.
pub const LAMBDA_PARITY: f64 = 1e-8;

/// CPU-only Tikhonov-regularized reconstruction.
///
/// Always uses the local Cholesky solver — never dispatches to barracuda.
/// Useful for cross-substrate parity comparisons where a known-good
/// CPU reference is needed independently of the dispatch path.
///
/// # Panics
///
/// Panics if dimensions are inconsistent or Cholesky fails.
#[must_use]
pub fn tikhonov_solve_cpu(
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

/// Tikhonov-regularized reconstruction.
///
/// Solves `(KᵀK + λI) ρ = KᵀG` via Cholesky decomposition.
///
/// When the `barracuda-gpu` feature is enabled, first tries GPU Cholesky
/// via `barracuda::linalg::cholesky_f64`, then falls back to
/// `barracuda::linalg::solve_f64_cpu` (Gauss-Jordan), then to the local
/// Cholesky solver.
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
    #[cfg(feature = "barracuda-gpu")]
    let (mut a, ktg) = gemm_setup_gpu(kernel, data, n_tau, n_omega).unwrap_or_else(|| {
        let ktk = mat_transpose_mul(kernel, kernel, n_tau, n_omega, n_omega);
        let ktg = mat_transpose_vec(kernel, data, n_tau, n_omega);
        (ktk, ktg)
    });

    #[cfg(not(feature = "barracuda-gpu"))]
    let (mut a, ktg) = {
        let ktk = mat_transpose_mul(kernel, kernel, n_tau, n_omega, n_omega);
        let ktg = mat_transpose_vec(kernel, data, n_tau, n_omega);
        (ktk, ktg)
    };

    for i in 0..n_omega {
        a[i * n_omega + i] += lambda;
    }

    #[cfg(feature = "barracuda-gpu")]
    {
        if let Some(solution) = tikhonov_solve_gpu(&a, &ktg, n_omega) {
            return solution;
        }
        if let Ok(solution) = barracuda::linalg::solve_f64_cpu(&a, &ktg, n_omega) {
            return solution;
        }
    }

    cholesky_solve(&a, &ktg, n_omega)
}

/// GPU Cholesky path: decompose the SPD system on GPU, then solve.
#[cfg(feature = "barracuda-gpu")]
fn tikhonov_solve_gpu(a: &[f64], b: &[f64], n: usize) -> Option<Vec<f64>> {
    let device = crate::gpu::get_device()?;
    let decomp = barracuda::linalg::cholesky_f64(device, a, n).ok()?;
    decomp.solve(b).ok()
}

/// GPU matrix setup: compute `KᵀK` and `KᵀG` via `GemmF64::execute_gemm_ex`.
///
/// Returns `None` if the device is unavailable or the GEMM dispatch fails.
/// barraCuda v0.3.5: `execute_gemm_ex` supports `trans_a`/`trans_b` flags.
#[cfg(feature = "barracuda-gpu")]
fn gemm_setup_gpu(
    kernel: &[f64],
    data: &[f64],
    n_tau: usize,
    n_omega: usize,
) -> Option<(Vec<f64>, Vec<f64>)> {
    let device = crate::gpu::get_device()?;
    // KᵀK: A=[n_tau × n_omega], B=[n_tau × n_omega], result=[n_omega × n_omega]
    let ktk = barracuda::ops::linalg::GemmF64::execute_gemm_ex(
        device.clone(),
        kernel,
        kernel,
        n_omega, // m (rows of result)
        n_tau,   // k (contraction dim)
        n_omega, // n (cols of result)
        1,       // batch_size
        1.0,     // alpha
        0.0,     // beta
        true,    // trans_a → Kᵀ
        false,   // trans_b → K
    )
    .ok()?;
    // KᵀG: A=[n_tau × n_omega], B=[n_tau × 1], result=[n_omega × 1]
    let ktg = barracuda::ops::linalg::GemmF64::execute_gemm_ex(
        device, kernel, data, n_omega, // m
        n_tau,   // k
        1,       // n
        1,       // batch_size
        1.0,     // alpha
        0.0,     // beta
        true,    // trans_a → Kᵀ
        false,   // trans_b
    )
    .ok()?;
    Some((ktk, ktg))
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test")]
mod tests {
    use super::*;
    use crate::spectral_recon::{build_kernel, forward_correlator, gaussian_peak};
    use crate::cast::usize_f64;

    #[test]
    fn tikhonov_solve_cpu_recovers_gaussian_peak() {
        let n_tau = 20;
        let n_omega = 40;
        let tau: Vec<f64> = (1..=n_tau)
            .map(|i| usize_f64(i) * 2.0 / usize_f64(n_tau))
            .collect();
        let omega: Vec<f64> = (1..=n_omega)
            .map(|i| usize_f64(i) * 8.0 / usize_f64(n_omega))
            .collect();
        let rho_true = gaussian_peak(&omega, 3.0, 0.5, 1.0);
        let kernel = build_kernel(&tau, &omega);
        let g = forward_correlator(&kernel, &rho_true, n_tau, n_omega);
        let rho_rec = tikhonov_solve_cpu(&kernel, &g, 1e-12, n_tau, n_omega);
        let peak_idx = rho_rec
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .unwrap()
            .0;
        assert!((omega[peak_idx] - 3.0).abs() < 1.0);
    }
}
