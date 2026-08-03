// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

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
///
/// When the `barracuda-gpu` feature is enabled, delegates to
/// `GemmF64::execute_gemm_ex` for the mat-vec product (K × ρ viewed
/// as an `n_tau × n_omega` by `n_omega × 1` GEMM). Falls back to a
/// local `mul_add` loop if no GPU is present or the dispatch fails.
#[must_use]
pub fn forward_correlator(kernel: &[f64], rho: &[f64], n_tau: usize, n_omega: usize) -> Vec<f64> {
    #[cfg(feature = "barracuda-gpu")]
    if let Some(g) = forward_correlator_gpu(kernel, rho, n_tau, n_omega) {
        return g;
    }
    forward_correlator_cpu(kernel, rho, n_tau, n_omega)
}

fn forward_correlator_cpu(kernel: &[f64], rho: &[f64], n_tau: usize, n_omega: usize) -> Vec<f64> {
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

/// GPU-delegated forward correlator via barraCuda GEMM.
#[cfg(feature = "barracuda-gpu")]
fn forward_correlator_gpu(
    kernel: &[f64],
    rho: &[f64],
    n_tau: usize,
    n_omega: usize,
) -> Option<Vec<f64>> {
    let device = crate::gpu::get_device()?;
    barracuda::ops::linalg::GemmF64::execute_gemm_ex(
        device, kernel, rho, n_tau,   // m (rows of result)
        n_omega, // k (contraction dim)
        1,       // n (columns of result — vector)
        1,       // batch_size
        1.0,     // alpha
        0.0,     // beta
        false,   // trans_a — K as-is
        false,   // trans_b — ρ as column
    )
    .ok()
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
