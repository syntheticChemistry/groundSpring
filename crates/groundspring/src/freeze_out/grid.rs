// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! 2D grid search over (T₀, κ₂) with L-BFGS refinement.
//!
//! GPU promotion via `barracuda-gpu` dispatches as a 2D workgroup with
//! per-point chi-squared reduction, then refines via L-BFGS with
//! numerical gradient (absorbed from airSpring V035 → barraCuda S84).

use crate::cast::{f64_usize, usize_f64};

#[cfg(feature = "barracuda-gpu")]
use crate::cast::u32_usize;

use super::curve::{chi_squared_per_dof, chi2_freeze_out};
use super::{GridFitConfig, GridFitResult};

#[cfg(feature = "barracuda-gpu")]
const LBFGS_MEMORY: usize = 5;
#[cfg(feature = "barracuda-gpu")]
const LBFGS_MAX_ITER: usize = 200;
#[cfg(feature = "barracuda-gpu")]
const LBFGS_GTOL: f64 = crate::tol::EXACT;
#[cfg(feature = "barracuda-gpu")]
const LBFGS_FTOL: f64 = crate::tol::DETERMINISM;
#[cfg(feature = "barracuda-gpu")]
const LBFGS_C1: f64 = crate::tol::RECONSTRUCTION;
#[cfg(feature = "barracuda-gpu")]
const LBFGS_C2: f64 = 0.9;
#[cfg(feature = "barracuda-gpu")]
const LBFGS_MAX_LINESEARCH: usize = 40;

/// 2D grid search over (T₀, κ₂) minimizing chi-squared.
///
/// Evaluates the freeze-out model on a regular grid and returns the
/// parameters with lowest chi-squared.
///
/// # Errors
///
/// Returns [`crate::error::InputError::LengthMismatch`] if
/// `config.observed` and `config.mu_b` have different lengths.
pub fn grid_fit_2d(config: &GridFitConfig<'_>) -> Result<GridFitResult, crate::error::InputError> {
    super::validate_config_lengths(config)?;
    #[cfg(feature = "barracuda-gpu")]
    {
        if let Some(result) = grid_fit_2d_gpu(config) {
            return Ok(lbfgs_refine(config, result));
        }
    }
    let coarse = grid_fit_2d_cpu(config);
    Ok(lbfgs_refine(config, coarse))
}

#[cfg_attr(
    not(feature = "barracuda-gpu"),
    expect(
        clippy::missing_const_for_fn,
        reason = "const only in non-barracuda-gpu builds; runtime dispatch with GPU"
    )
)]
fn lbfgs_refine(
    #[cfg_attr(
        not(feature = "barracuda-gpu"),
        expect(
            unused,
            reason = "config only used in barracuda-gpu L-BFGS refinement path"
        )
    )]
    config: &GridFitConfig<'_>,
    coarse: GridFitResult,
) -> GridFitResult {
    #[cfg(feature = "barracuda-gpu")]
    {
        if let Some(refined) = lbfgs_refine_barracuda(config, &coarse) {
            return refined;
        }
    }
    coarse
}

#[cfg(feature = "barracuda-gpu")]
fn lbfgs_refine_barracuda(
    config: &GridFitConfig<'_>,
    coarse: &GridFitResult,
) -> Option<GridFitResult> {
    use barracuda::optimize::{LbfgsConfig, lbfgs_numerical};

    let n_data = config.observed.len();
    let inv_sigma2 = 1.0 / (config.sigma * config.sigma);
    let observed = config.observed;
    let mu_b = config.mu_b;

    let objective = |x: &[f64]| -> f64 { chi2_freeze_out(observed, mu_b, x[0], x[1], inv_sigma2) };

    let lbfgs_config = LbfgsConfig {
        memory: LBFGS_MEMORY,
        max_iter: LBFGS_MAX_ITER,
        gtol: LBFGS_GTOL,
        ftol: LBFGS_FTOL,
        c1: LBFGS_C1,
        c2: LBFGS_C2,
        max_linesearch: LBFGS_MAX_LINESEARCH,
    };

    let x0 = [coarse.t0, coarse.kappa2];
    let result = lbfgs_numerical(objective, &x0, &lbfgs_config).ok()?;

    if result.f_val < coarse.chi_squared {
        Some(GridFitResult {
            t0: result.x[0],
            kappa2: result.x[1],
            chi_squared: result.f_val,
            chi2_per_dof: chi_squared_per_dof(result.f_val, n_data, 2),
        })
    } else {
        None
    }
}

#[cfg(feature = "barracuda-gpu")]
fn grid_fit_2d_gpu(config: &GridFitConfig<'_>) -> Option<GridFitResult> {
    let device = crate::gpu::get_device()?;

    let n_data = config.observed.len();
    let inv_sigma2 = 1.0 / (config.sigma * config.sigma);

    let nt0 = f64_usize(((config.t0_hi - config.t0_lo) / config.t0_step).ceil()) + 1;
    let nk2 = f64_usize(((config.k2_hi - config.k2_lo) / config.k2_step).ceil()) + 1;

    let t0_grid: Vec<f64> = (0..nt0)
        .map(|i| usize_f64(i).mul_add(config.t0_step, config.t0_lo))
        .collect();
    let k2_grid: Vec<f64> = (0..nk2)
        .map(|i| usize_f64(i).mul_add(config.k2_step, config.k2_lo))
        .collect();
    let z_grid = vec![0.0_f64];

    let mut chi2_values = Vec::with_capacity(nt0 * nk2);

    for &t0_val in &t0_grid {
        for &k2_val in &k2_grid {
            chi2_values.push(chi2_freeze_out(
                config.observed,
                config.mu_b,
                t0_val,
                k2_val,
                inv_sigma2,
            ));
        }
    }

    let result =
        barracuda::ops::grid::grid_search_3d(&device, &t0_grid, &k2_grid, &z_grid, &chi2_values)
            .ok()?;

    Some(GridFitResult {
        t0: t0_grid[u32_usize(result.min_ix)],
        kappa2: k2_grid[u32_usize(result.min_iy)],
        chi_squared: result.min_value,
        chi2_per_dof: chi_squared_per_dof(result.min_value, n_data, 2),
    })
}

#[expect(
    clippy::similar_names,
    reason = "t0/k2 and lo/hi are domain-standard names"
)]
pub(super) fn grid_fit_2d_cpu(config: &GridFitConfig<'_>) -> GridFitResult {
    let n_data = config.observed.len();
    let inv_sigma2 = 1.0 / (config.sigma * config.sigma);

    let mut best_chi2 = f64::INFINITY;
    let mut best_t0 = config.t0_lo;
    let mut best_k2 = config.k2_lo;

    let n_t0 = f64_usize(((config.t0_hi - config.t0_lo) / config.t0_step).ceil()) + 1;
    let n_k2 = f64_usize(((config.k2_hi - config.k2_lo) / config.k2_step).ceil()) + 1;

    for it in 0..n_t0 {
        let t0 = usize_f64(it).mul_add(config.t0_step, config.t0_lo);
        for ik in 0..n_k2 {
            let k2 = usize_f64(ik).mul_add(config.k2_step, config.k2_lo);
            let c2 = chi2_freeze_out(config.observed, config.mu_b, t0, k2, inv_sigma2);
            if c2 < best_chi2 {
                best_chi2 = c2;
                best_t0 = t0;
                best_k2 = k2;
            }
        }
    }

    let n_params = 2;
    GridFitResult {
        t0: best_t0,
        kappa2: best_k2,
        chi_squared: best_chi2,
        chi2_per_dof: chi_squared_per_dof(best_chi2, n_data, n_params),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::freeze_out::GridFitConfig;

    fn test_config() -> GridFitConfig<'static> {
        static OBS: [f64; 4] = [155.0, 150.0, 140.0, 120.0];
        static MU_B: [f64; 4] = [0.0, 100.0, 200.0, 400.0];
        GridFitConfig {
            observed: &OBS,
            mu_b: &MU_B,
            sigma: 2.0,
            t0_lo: 150.0,
            t0_hi: 160.0,
            t0_step: 1.0,
            k2_lo: 0.001,
            k2_hi: 0.02,
            k2_step: 0.001,
        }
    }

    #[test]
    fn grid_fit_cpu_returns_finite() {
        let cfg = test_config();
        let result = grid_fit_2d_cpu(&cfg);
        assert!(result.t0.is_finite());
        assert!(result.kappa2.is_finite());
        assert!(result.chi_squared >= 0.0);
    }

    #[test]
    fn grid_fit_cpu_t0_in_range() {
        let cfg = test_config();
        let result = grid_fit_2d_cpu(&cfg);
        assert!(result.t0 >= cfg.t0_lo && result.t0 <= cfg.t0_hi);
        assert!(result.kappa2 >= cfg.k2_lo && result.kappa2 <= cfg.k2_hi);
    }

    #[test]
    fn grid_fit_2d_validates_lengths() {
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
        assert!(grid_fit_2d(&cfg).is_err());
    }
}
