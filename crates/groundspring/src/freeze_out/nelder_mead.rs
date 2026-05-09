// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Multi-start Nelder-Mead refinement via `barracuda::optimize`.
//!
//! `barracuda::optimize::batched_nelder_mead_gpu` (barraCuda S80) enables
//! multi-start derivative-free optimization for non-smooth landscapes.

use super::GridFitConfig;

/// Multi-start Nelder-Mead refinement result.
///
/// Returned by [`nelder_mead_multi_start`] when the `barracuda-gpu`
/// feature is enabled and a GPU device is available.
#[derive(Debug, Clone)]
pub struct NelderMeadMultiStartResult {
    /// Best-fit T₀ parameter.
    pub t0: f64,
    /// Best-fit κ₂ parameter.
    pub kappa2: f64,
    /// Chi-squared value at the best fit.
    pub chi_squared: f64,
    /// Number of starts that converged.
    pub converged_count: usize,
}

#[cfg(feature = "barracuda-gpu")]
const NM_MAX_ITERS: usize = 500;
#[cfg(feature = "barracuda-gpu")]
const NM_TOL: f64 = crate::tol::EXACT;
#[cfg(feature = "barracuda-gpu")]
const NM_SEED: u64 = 42;

/// Nelder-Mead simplex perturbation scale relative to the coarse grid step.
///
/// Each non-centroid vertex is offset from the coarse-grid optimum by
/// `N(0, step × NM_SIMPLEX_SCALE)`. A factor of 2 ensures the simplex
/// spans ±2σ of the grid cell, exploring enough of the local landscape
/// to avoid the nearest-grid-point trap.
#[cfg(feature = "barracuda-gpu")]
const NM_SIMPLEX_SCALE: f64 = 2.0;

/// Multi-start Nelder-Mead refinement of the freeze-out fit.
///
/// Dispatches `n_starts` independent Nelder-Mead optimizations on GPU
/// via `barracuda::optimize::batched_nelder_mead_gpu` (barraCuda S80).
/// Each start initializes around the coarse grid-search result with
/// random perturbations, exploring the landscape for global minima.
///
/// Requires `barracuda-gpu` feature. Returns `None` if no GPU is
/// available or the feature is not enabled.
///
/// # Errors
///
/// Returns [`crate::error::InputError::LengthMismatch`] if
/// `config.observed` and `config.mu_b` differ in length.
pub fn nelder_mead_multi_start(
    config: &GridFitConfig<'_>,
    coarse: &super::GridFitResult,
    n_starts: usize,
) -> Result<Option<NelderMeadMultiStartResult>, crate::error::InputError> {
    super::validate_config_lengths(config)?;
    #[cfg(feature = "barracuda-gpu")]
    {
        Ok(nelder_mead_multi_start_gpu(config, coarse, n_starts))
    }
    #[cfg(not(feature = "barracuda-gpu"))]
    {
        let _ = (coarse, n_starts);
        Ok(None)
    }
}

#[cfg(feature = "barracuda-gpu")]
fn nelder_mead_multi_start_gpu(
    config: &GridFitConfig<'_>,
    coarse: &super::GridFitResult,
    n_starts: usize,
) -> Option<NelderMeadMultiStartResult> {
    use barracuda::optimize::batched_nelder_mead_gpu::{BatchNelderMeadConfig, NelderMeadResult};

    use super::curve::chi2_freeze_out;

    let device = crate::gpu::get_device()?;
    let inv_sigma2 = 1.0 / (config.sigma * config.sigma);

    let nm_config = BatchNelderMeadConfig {
        dims: 2,
        max_iters: NM_MAX_ITERS,
        tol: NM_TOL,
        ..BatchNelderMeadConfig::default()
    };

    let mut rng = crate::prng::DefaultRng::new(NM_SEED);
    let mut simplices = Vec::with_capacity(n_starts * 3 * 2);
    for _ in 0..n_starts {
        for vertex in 0..3 {
            let t0 = coarse.t0
                + if vertex == 0 {
                    0.0
                } else {
                    rng.normal(0.0, config.t0_step * NM_SIMPLEX_SCALE)
                };
            let k2 = coarse.kappa2
                + if vertex == 0 {
                    0.0
                } else {
                    rng.normal(0.0, config.k2_step * NM_SIMPLEX_SCALE)
                };
            simplices.push(t0);
            simplices.push(k2);
        }
    }

    let observed = config.observed.to_vec();
    let mu_b = config.mu_b.to_vec();

    let f_values = |points: &[f64]| -> Vec<f64> {
        points
            .chunks(2)
            .map(|p| chi2_freeze_out(&observed, &mu_b, p[0], p[1], inv_sigma2))
            .collect()
    };

    let results: Vec<NelderMeadResult> = barracuda::device::test_pool::tokio_block_on(async {
        barracuda::optimize::batched_nelder_mead_gpu::batched_nelder_mead_gpu(
            &device, &nm_config, n_starts, &simplices, f_values,
        )
        .await
    })
    .ok()?;

    let converged_count = results.iter().filter(|r| r.converged).count();
    let best = results
        .iter()
        .min_by(|a, b| a.best_value.total_cmp(&b.best_value))?;

    Some(NelderMeadMultiStartResult {
        t0: best.best_point[0],
        kappa2: best.best_point[1],
        chi_squared: best.best_value,
        converged_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::freeze_out::{GridFitConfig, GridFitResult};

    #[test]
    fn nelder_mead_without_gpu_returns_none() {
        let cfg = GridFitConfig {
            observed: &[155.0, 150.0, 140.0],
            mu_b: &[0.0, 100.0, 200.0],
            sigma: 2.0,
            t0_lo: 150.0,
            t0_hi: 160.0,
            t0_step: 1.0,
            k2_lo: 0.001,
            k2_hi: 0.02,
            k2_step: 0.001,
        };
        let coarse = GridFitResult {
            t0: 155.0,
            kappa2: 0.01,
            chi_squared: 5.0,
            chi2_per_dof: 5.0,
        };
        #[expect(clippy::unwrap_used, reason = "test assertions on known-valid inputs")]
        let result = nelder_mead_multi_start(&cfg, &coarse, 4).unwrap();
        if !cfg!(feature = "barracuda-gpu") {
            assert!(result.is_none());
        }
    }

    #[test]
    fn nelder_mead_validates_lengths() {
        let cfg = GridFitConfig {
            observed: &[1.0],
            mu_b: &[1.0, 2.0],
            sigma: 1.0,
            t0_lo: 1.0,
            t0_hi: 2.0,
            t0_step: 0.5,
            k2_lo: 0.0,
            k2_hi: 0.1,
            k2_step: 0.05,
        };
        let coarse = GridFitResult {
            t0: 1.5,
            kappa2: 0.05,
            chi_squared: 1.0,
            chi2_per_dof: 1.0,
        };
        assert!(nelder_mead_multi_start(&cfg, &coarse, 1).is_err());
    }
}
