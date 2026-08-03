// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

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

    let fft = Fft1DF64::new(tensor, crate::cast::u64_u32_truncate(n as u64)).ok()?;
    let result = barracuda::device::test_pool::tokio_block_on(fft.execute()).ok()?;

    let f32_data = result.to_vec().ok()?;
    let data: &[f64] = bytemuck::cast_slice(&f32_data);

    let re: Vec<f64> = data.iter().step_by(2).copied().collect();
    let im: Vec<f64> = data.iter().skip(1).step_by(2).copied().collect();

    Some((re, im))
}
