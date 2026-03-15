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
//! The matrix products (`KᵀK`, `KᵀG`) remain local: typical spectral
//! reconstruction problems have small dimensions (`n_tau`, `n_omega` ~ 10–100)
//! where GPU dispatch overhead exceeds compute savings.  If problem sizes
//! grow, `barracuda::linalg::GemmF64::execute` provides a batched GPU path.

/// Build the Laplace-transform kernel matrix (row-major, `n_tau × n_omega`).
///
/// `K[i,j] = exp(−τ_i ω_j) Δω`
#[must_use]
pub fn build_kernel(tau: &[f64], omega: &[f64]) -> Vec<f64> {
    let d_omega = if omega.len() > 1 {
        omega[1] - omega[0]
    } else {
        1.0
    };
    tau.iter()
        .flat_map(|&t| omega.iter().map(move |&w| (-t * w).exp() * d_omega))
        .collect()
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
    let ktk = mat_transpose_mul(kernel, kernel, n_tau, n_omega, n_omega);
    let ktg = mat_transpose_vec(kernel, data, n_tau, n_omega);

    let mut a = ktk;
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

/// Compute the power spectrum |FFT(G)|² of a real-valued correlator.
///
/// For lattice correlator analysis (Bazavov 2025): the FFT reveals the
/// frequency content of G(τ) directly, complementing the Tikhonov
/// reconstruction which solves the inverse Laplace transform.
///
/// Returns `(frequencies, power)` where `frequencies[k] = k / (N Δτ)`
/// and `power[k] = |X[k]|²`.  Only the first `N/2 + 1` (positive
/// frequencies) are returned.
///
/// When `barracuda-gpu` is enabled, delegates to `barracuda::ops::fft::Fft1DF64`
/// (GPU Cooley-Tukey radix-2).  Falls back to a CPU DFT otherwise.
///
/// # Panics
///
/// Panics if `correlator` is empty.
#[must_use]
pub fn fft_power_spectrum(correlator: &[f64], d_tau: f64) -> (Vec<f64>, Vec<f64>) {
    assert!(!correlator.is_empty(), "correlator must be non-empty");

    let n = correlator.len().next_power_of_two();
    let n_out = n / 2 + 1;

    let (re, im) = fft_correlator(correlator, n);

    let frequencies: Vec<f64> = (0..n_out)
        .map(|k| crate::cast::usize_f64(k) / (crate::cast::usize_f64(n) * d_tau))
        .collect();

    let power: Vec<f64> = re[..n_out]
        .iter()
        .zip(&im[..n_out])
        .map(|(&r, &i)| r.mul_add(r, i * i))
        .collect();

    (frequencies, power)
}

/// Compute the FFT of a real correlator, returning (re, im) arrays.
///
/// Zero-pads to the next power of two. Returns full N-point transform.
fn fft_correlator(correlator: &[f64], n: usize) -> (Vec<f64>, Vec<f64>) {
    #[cfg(feature = "barracuda-gpu")]
    {
        if let Some(result) = fft_correlator_gpu(correlator, n) {
            return result;
        }
    }
    fft_correlator_cpu(correlator, n)
}

/// CPU DFT: O(N²) naive implementation, sufficient for correlators (N < 1000).
fn fft_correlator_cpu(correlator: &[f64], n: usize) -> (Vec<f64>, Vec<f64>) {
    let mut re = vec![0.0_f64; n];
    let mut im = vec![0.0_f64; n];
    let n_f = crate::cast::usize_f64(n);

    for k in 0..n {
        let mut sum_re = 0.0_f64;
        let mut sum_im = 0.0_f64;
        for (j, &g_j) in correlator.iter().enumerate() {
            let angle = -std::f64::consts::TAU * crate::cast::usize_f64(k * j) / n_f;
            sum_re = angle.cos().mul_add(g_j, sum_re);
            sum_im = angle.sin().mul_add(g_j, sum_im);
        }
        re[k] = sum_re;
        im[k] = sum_im;
    }

    (re, im)
}

/// GPU FFT path via `barracuda::ops::fft::Fft1DF64`.
///
/// Uploads interleaved `[re, im]` f64 data via `Tensor::from_data_pod`,
/// dispatches Cooley-Tukey radix-2 on GPU, then reads back via `to_vec()`
/// (f32 view of raw bytes) and reinterprets as f64 via `bytemuck`.
#[cfg(feature = "barracuda-gpu")]
fn fft_correlator_gpu(correlator: &[f64], n: usize) -> Option<(Vec<f64>, Vec<f64>)> {
    use barracuda::ops::fft::Fft1DF64;
    use barracuda::tensor::Tensor;

    let device = crate::gpu::get_device()?;

    let mut interleaved = vec![0.0_f64; n * 2];
    for (i, &g) in correlator.iter().enumerate() {
        interleaved[i * 2] = g;
    }

    let tensor = Tensor::from_data_pod(&interleaved, vec![n, 2], device).ok()?;

    #[expect(
        clippy::cast_possible_truncation,
        reason = "FFT degree n is always a power of 2 ≤ 2^20, fits u32"
    )]
    let fft = Fft1DF64::new(tensor, n as u32).ok()?;
    let result = barracuda::device::test_pool::tokio_block_on(fft.execute()).ok()?;

    let f32_data = result.to_vec().ok()?;
    let data: &[f64] = bytemuck::cast_slice(&f32_data);

    let re: Vec<f64> = data.iter().step_by(2).copied().collect();
    let im: Vec<f64> = data.iter().skip(1).step_by(2).copied().collect();

    Some((re, im))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cast::usize_f64;
    use crate::tol;

    /// Tikhonov regularization strengths (λ) for tests.
    /// Stronger regularisation → smoother but biased; weaker → faithful but noisy.
    const LAMBDA_NOISELESS: f64 = 1e-12;
    const LAMBDA_NOISY: f64 = 1e-6;
    const LAMBDA_PARITY: f64 = 1e-8;

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
