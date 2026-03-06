// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Freeze-out curve fitting and chi-squared inverse problem.
//!
//! Implements the polynomial forward model and 2D grid-search chi-squared
//! minimization from Bazavov et al. (2016) Phys Rev D 93, 014512.
//!
//! The freeze-out curve parameterizes the QCD transition temperature as
//! a function of baryon chemical potential:
//! `T_f(μ_B) = T₀ (1 - κ₂ (μ_B/T₀)²)`
//!
//! # barracuda delegation
//!
//! [`grid_fit_2d`] is embarrassingly parallel — each (T₀, κ₂) grid
//! point evaluates independently. GPU promotion via `barracuda-gpu`
//! dispatches as a 2D workgroup with per-point chi-squared reduction.
//! When `barracuda` is enabled, the grid-search result is refined via
//! `barracuda::optimize::lbfgs_numerical` (L-BFGS with numerical gradient,
//! absorbed from airSpring V035 → barraCuda S84).
//! [`chi_squared`] and [`freeze_out_curve`] stay local (scalar ops).
//!
//! ## S80 evolution: batched Nelder-Mead GPU
//!
//! `barracuda::optimize::batched_nelder_mead_gpu` (barraCuda S80) enables
//! multi-start derivative-free optimization. [`nelder_mead_multi_start`]
//! exposes this as an alternative to L-BFGS for non-smooth landscapes.

use crate::cast::usize_f64;

// ── L-BFGS refinement configuration ──────────────────────────────────────
// These defaults produce robust convergence for the 2-parameter freeze-out
// landscape across the Bazavov et al. benchmark grid.

/// L-BFGS history window: number of past iterations retained.
#[cfg(feature = "barracuda-gpu")]
const LBFGS_MEMORY: usize = 5;
/// L-BFGS maximum iterations.
#[cfg(feature = "barracuda-gpu")]
const LBFGS_MAX_ITER: usize = 200;
/// L-BFGS gradient tolerance for convergence.
#[cfg(feature = "barracuda-gpu")]
const LBFGS_GTOL: f64 = 1e-12;
/// L-BFGS function-value tolerance for convergence.
#[cfg(feature = "barracuda-gpu")]
const LBFGS_FTOL: f64 = 1e-15;
/// L-BFGS Wolfe condition c₁ (sufficient decrease).
#[cfg(feature = "barracuda-gpu")]
const LBFGS_C1: f64 = 1e-4;
/// L-BFGS Wolfe condition c₂ (curvature).
#[cfg(feature = "barracuda-gpu")]
const LBFGS_C2: f64 = 0.9;
/// L-BFGS maximum line-search steps per iteration.
#[cfg(feature = "barracuda-gpu")]
const LBFGS_MAX_LINESEARCH: usize = 40;

/// Nelder-Mead maximum iterations for batched GPU multi-start.
#[cfg(feature = "barracuda-gpu")]
const NM_MAX_ITERS: usize = 500;
/// Nelder-Mead convergence tolerance (function-value).
#[cfg(feature = "barracuda-gpu")]
const NM_TOL: f64 = 1e-12;
/// Deterministic PRNG seed for Nelder-Mead simplex initialization.
///
/// The optimization result is seed-independent; this just ensures
/// reproducibility across runs.
#[cfg(feature = "barracuda-gpu")]
const NM_SEED: u64 = 42;

/// Result of a 2D grid-search chi-squared fit.
#[derive(Debug, Clone)]
pub struct GridFitResult {
    /// Best-fit T₀ parameter.
    pub t0: f64,
    /// Best-fit κ₂ parameter.
    pub kappa2: f64,
    /// Chi-squared value at the best fit.
    pub chi_squared: f64,
    /// Chi-squared per degree of freedom.
    pub chi2_per_dof: f64,
}

/// Evaluate the freeze-out curve at a single `μ_B`.
///
/// `T_f(μ_B) = T₀ (1 - κ₂ (μ_B/T₀)²)`
#[inline]
#[must_use]
pub fn freeze_out_curve(t0: f64, kappa2: f64, mu_b: f64) -> f64 {
    let r = mu_b / t0;
    (-kappa2).mul_add(r * r, 1.0) * t0
}

/// Chi-squared statistic for uniform errors.
///
/// `χ² = Σ((obs_i - pred_i) / σ)²`
///
/// # Errors
///
/// Returns [`crate::error::InputError::LengthMismatch`] if `observed` and
/// `predicted` have different lengths.
pub fn chi_squared(
    observed: &[f64],
    predicted: &[f64],
    sigma: f64,
) -> Result<f64, crate::error::InputError> {
    if observed.len() != predicted.len() {
        return Err(crate::error::InputError::LengthMismatch {
            first: "observed",
            first_len: observed.len(),
            second: "predicted",
            second_len: predicted.len(),
        });
    }
    let inv_sigma2 = 1.0 / (sigma * sigma);
    Ok(observed
        .iter()
        .zip(predicted.iter())
        .map(|(&o, &p)| (o - p).powi(2) * inv_sigma2)
        .sum())
}

/// Chi-squared per degree of freedom.
#[inline]
#[must_use]
pub fn chi_squared_per_dof(chi2: f64, n_data: usize, n_params: usize) -> f64 {
    chi2 / usize_f64(n_data - n_params)
}

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
    validate_config_lengths(config)?;
    #[cfg(feature = "barracuda-gpu")]
    {
        if let Some(result) = grid_fit_2d_gpu(config) {
            return Ok(lbfgs_refine(config, result));
        }
    }
    let coarse = grid_fit_2d_cpu(config);
    Ok(lbfgs_refine(config, coarse))
}

/// Refine grid-search result via L-BFGS with numerical gradient.
///
/// Cross-spring lineage: airSpring V035 parameter fitting →
/// barraCuda S84 `barracuda::optimize::lbfgs_numerical` →
/// groundSpring freeze-out refinement.
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

    let objective = |x: &[f64]| -> f64 {
        let t0 = x[0];
        let k2 = x[1];
        observed
            .iter()
            .zip(mu_b.iter())
            .map(|(&o, &mu)| (o - freeze_out_curve(t0, k2, mu)).powi(2) * inv_sigma2)
            .sum()
    };

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

/// GPU-accelerated freeze-out grid fit: pre-evaluate chi-squared on CPU,
/// then use barracuda's `grid_search_3d` for parallel argmin over the 2D
/// parameter space (z-dimension = 1).
///
/// Cross-spring lineage: `grid_search_3d_f64.wgsl` — groundSpring forward
/// model (Bazavov freeze-out polynomial) + barracuda `ComputeDispatch` (absorbed S71+++).
#[cfg(feature = "barracuda-gpu")]
fn grid_fit_2d_gpu(config: &GridFitConfig<'_>) -> Option<GridFitResult> {
    use crate::cast::f64_usize;

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
    let mut pred = vec![0.0; n_data];

    for &t0_val in &t0_grid {
        for &k2_val in &k2_grid {
            for (j, &mu) in config.mu_b.iter().enumerate() {
                pred[j] = freeze_out_curve(t0_val, k2_val, mu);
            }
            let c2: f64 = config
                .observed
                .iter()
                .zip(pred.iter())
                .map(|(&o, &p)| (o - p).powi(2) * inv_sigma2)
                .sum();
            chi2_values.push(c2);
        }
    }

    let result =
        barracuda::ops::grid::grid_search_3d(&device, &t0_grid, &k2_grid, &z_grid, &chi2_values)
            .ok()?;

    Some(GridFitResult {
        t0: t0_grid[result.min_ix as usize],
        kappa2: k2_grid[result.min_iy as usize],
        chi_squared: result.min_value,
        chi2_per_dof: chi_squared_per_dof(result.min_value, n_data, 2),
    })
}

#[expect(
    clippy::similar_names,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "t0/k2 and lo/hi are domain-standard names; loop index fits in f64"
)]
fn grid_fit_2d_cpu(config: &GridFitConfig<'_>) -> GridFitResult {
    let n_data = config.observed.len();
    let inv_sigma2 = 1.0 / (config.sigma * config.sigma);

    let mut best_chi2 = f64::INFINITY;
    let mut best_t0 = config.t0_lo;
    let mut best_k2 = config.k2_lo;

    let n_t0 = ((config.t0_hi - config.t0_lo) / config.t0_step).ceil() as usize + 1;
    let n_k2 = ((config.k2_hi - config.k2_lo) / config.k2_step).ceil() as usize + 1;

    let mut pred = vec![0.0; n_data];

    for it in 0..n_t0 {
        let t0 = usize_f64(it).mul_add(config.t0_step, config.t0_lo);
        for ik in 0..n_k2 {
            let k2 = usize_f64(ik).mul_add(config.k2_step, config.k2_lo);
            for (j, &mu) in config.mu_b.iter().enumerate() {
                pred[j] = freeze_out_curve(t0, k2, mu);
            }
            let c2: f64 = config
                .observed
                .iter()
                .zip(pred.iter())
                .map(|(&o, &p)| (o - p).powi(2) * inv_sigma2)
                .sum();
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

/// Decomposed chi-squared analysis with per-datum diagnostics.
///
/// Extends the basic chi-squared statistic with residuals, pulls, and
/// per-datum contributions for detailed goodness-of-fit diagnosis.
///
/// Cross-spring lineage: hotSpring `Chi2Decomposed` (nuclear structure
/// fit quality) → barraCuda S59 `barracuda::stats::chi2` with p-value
/// via regularized incomplete gamma → groundSpring freeze-out analysis.
#[derive(Debug, Clone)]
pub struct Chi2Analysis {
    /// Total chi-squared statistic.
    pub chi2_total: f64,
    /// Chi-squared per data point.
    pub chi2_per_datum: f64,
    /// Chi-squared per degree of freedom.
    pub chi2_per_dof: f64,
    /// Degrees of freedom (`n_data` − `n_params`).
    pub dof: usize,
    /// Per-datum chi-squared contributions.
    pub contributions: Vec<f64>,
    /// Residuals (observed − predicted).
    pub residuals: Vec<f64>,
    /// Pulls (residual / uncertainty).
    pub pulls: Vec<f64>,
    /// P-value from the chi-squared distribution.
    pub p_value: f64,
}

/// Decomposed chi-squared analysis with per-datum diagnostics.
///
/// When the `barracuda` feature is enabled, delegates to
/// `barracuda::stats::chi2::chi2_decomposed_weighted` for the full
/// decomposition including p-value computation via the regularized
/// incomplete gamma function. Falls back to a local implementation
/// that computes everything except the p-value (set to NaN).
///
/// # Errors
///
/// Returns [`crate::error::InputError::LengthMismatch`] if `observed`
/// and `predicted` have different lengths.
pub fn chi2_analysis(
    observed: &[f64],
    predicted: &[f64],
    sigma: f64,
    n_params: usize,
) -> Result<Chi2Analysis, crate::error::InputError> {
    if observed.len() != predicted.len() {
        return Err(crate::error::InputError::LengthMismatch {
            first: "observed",
            first_len: observed.len(),
            second: "predicted",
            second_len: predicted.len(),
        });
    }
    #[cfg(feature = "barracuda")]
    {
        let uncertainties: Vec<f64> = vec![sigma; observed.len()];
        if let Ok(decomposed) = barracuda::stats::chi2::chi2_decomposed_weighted(
            observed,
            predicted,
            &uncertainties,
            n_params,
        ) {
            return Ok(Chi2Analysis {
                chi2_total: decomposed.chi2_total,
                chi2_per_datum: decomposed.chi2_per_datum,
                chi2_per_dof: decomposed.chi2_per_dof,
                dof: decomposed.dof,
                contributions: decomposed.contributions,
                residuals: decomposed.residuals,
                pulls: decomposed.pulls,
                p_value: decomposed.p_value,
            });
        }
    }
    Ok(chi2_analysis_cpu(observed, predicted, sigma, n_params))
}

fn chi2_analysis_cpu(
    observed: &[f64],
    predicted: &[f64],
    sigma: f64,
    n_params: usize,
) -> Chi2Analysis {
    let n = observed.len();
    let inv_sigma = 1.0 / sigma;

    let residuals: Vec<f64> = observed
        .iter()
        .zip(predicted.iter())
        .map(|(&o, &p)| o - p)
        .collect();
    let pulls: Vec<f64> = residuals.iter().map(|&r| r * inv_sigma).collect();
    let contributions: Vec<f64> = pulls.iter().map(|&p| p * p).collect();
    let chi2_total: f64 = contributions.iter().sum();
    let dof = n.saturating_sub(n_params);
    let chi2_per_dof = if dof > 0 {
        chi2_total / usize_f64(dof)
    } else {
        chi2_total
    };

    Chi2Analysis {
        chi2_total,
        chi2_per_datum: if n > 0 {
            chi2_total / usize_f64(n)
        } else {
            0.0
        },
        chi2_per_dof,
        dof,
        contributions,
        residuals,
        pulls,
        p_value: f64::NAN,
    }
}

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
    coarse: &GridFitResult,
    n_starts: usize,
) -> Result<Option<NelderMeadMultiStartResult>, crate::error::InputError> {
    validate_config_lengths(config)?;
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

/// Shared validation for [`GridFitConfig`] slice lengths.
const fn validate_config_lengths(
    config: &GridFitConfig<'_>,
) -> Result<(), crate::error::InputError> {
    if config.observed.len() != config.mu_b.len() {
        return Err(crate::error::InputError::LengthMismatch {
            first: "observed",
            first_len: config.observed.len(),
            second: "mu_b",
            second_len: config.mu_b.len(),
        });
    }
    Ok(())
}

#[cfg(feature = "barracuda-gpu")]
fn nelder_mead_multi_start_gpu(
    config: &GridFitConfig<'_>,
    coarse: &GridFitResult,
    n_starts: usize,
) -> Option<NelderMeadMultiStartResult> {
    use barracuda::optimize::batched_nelder_mead_gpu::{BatchNelderMeadConfig, NelderMeadResult};

    let device = crate::gpu::get_device()?;
    let inv_sigma2 = 1.0 / (config.sigma * config.sigma);

    let nm_config = BatchNelderMeadConfig {
        dims: 2,
        max_iters: NM_MAX_ITERS,
        tol: NM_TOL,
        ..BatchNelderMeadConfig::default()
    };

    let mut rng = crate::prng::Xorshift64::new(NM_SEED);
    let mut simplices = Vec::with_capacity(n_starts * 3 * 2);
    for _ in 0..n_starts {
        for vertex in 0..3 {
            let t0 = coarse.t0
                + if vertex == 0 {
                    0.0
                } else {
                    rng.normal(0.0, config.t0_step * 2.0)
                };
            let k2 = coarse.kappa2
                + if vertex == 0 {
                    0.0
                } else {
                    rng.normal(0.0, config.k2_step * 2.0)
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
            .map(|p| {
                let t0 = p[0];
                let k2 = p[1];
                observed
                    .iter()
                    .zip(mu_b.iter())
                    .map(|(&o, &mu)| (o - freeze_out_curve(t0, k2, mu)).powi(2) * inv_sigma2)
                    .sum()
            })
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

/// Configuration for a 2D grid-search fit.
#[derive(Debug, Clone, Copy)]
pub struct GridFitConfig<'a> {
    /// Observed data points.
    pub observed: &'a [f64],
    /// Corresponding `μ_B` values.
    pub mu_b: &'a [f64],
    /// Measurement uncertainty (uniform σ).
    pub sigma: f64,
    /// T₀ grid lower bound.
    pub t0_lo: f64,
    /// T₀ grid upper bound.
    pub t0_hi: f64,
    /// T₀ grid step size.
    pub t0_step: f64,
    /// κ₂ grid lower bound.
    pub k2_lo: f64,
    /// κ₂ grid upper bound.
    pub k2_hi: f64,
    /// κ₂ grid step size.
    pub k2_step: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tol;

    #[test]
    fn curve_at_zero() {
        let t = freeze_out_curve(155.0, 0.013, 0.0);
        assert!((t - 155.0).abs() < tol::EXACT, "T_f(0) should equal T0");
    }

    #[test]
    fn curve_monotone_decreasing() {
        let t0 = 155.0;
        let k2 = 0.013;
        let prev = freeze_out_curve(t0, k2, 0.0);
        for mu in (50..=400).step_by(50) {
            let t = freeze_out_curve(t0, k2, f64::from(mu));
            assert!(t <= prev + tol::EXACT, "T_f should decrease with mu_B");
        }
    }

    #[test]
    fn chi2_zero_at_truth() {
        let obs = vec![1.0, 2.0, 3.0];
        let pred = vec![1.0, 2.0, 3.0];
        let c2 = chi_squared(&obs, &pred, 1.0).unwrap();
        assert!(c2.abs() < tol::STRICT);
    }

    #[test]
    fn grid_recovers_noiseless() {
        let t0 = 155.0;
        let k2 = 0.013;
        let mu_b: Vec<f64> = (0..9).map(|i| f64::from(i) * 50.0).collect();
        let obs: Vec<f64> = mu_b.iter().map(|&m| freeze_out_curve(t0, k2, m)).collect();

        let cfg = GridFitConfig {
            observed: &obs,
            mu_b: &mu_b,
            sigma: 1.0,
            t0_lo: 150.0,
            t0_hi: 160.0,
            t0_step: 0.5,
            k2_lo: 0.008,
            k2_hi: 0.020,
            k2_step: 0.001,
        };
        let r = grid_fit_2d(&cfg).unwrap();
        assert!((r.t0 - t0).abs() < 1.0, "T0: got {}", r.t0);
        assert!(
            (r.kappa2 - k2).abs() < tol::DECOMPOSITION,
            "k2: got {}",
            r.kappa2
        );
    }

    #[test]
    fn chi2_per_dof_correct() {
        let c = chi_squared_per_dof(14.0, 9, 2);
        assert!((c - 2.0).abs() < tol::EXACT);
    }

    #[test]
    fn chi2_analysis_perfect_fit() {
        let obs = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let pred = obs.clone();
        let a = chi2_analysis(&obs, &pred, 1.0, 2).unwrap();
        assert!(a.chi2_total.abs() < tol::STRICT, "perfect fit → χ²=0");
        assert_eq!(a.dof, 3);
        assert!(a.residuals.iter().all(|&r| r.abs() < tol::STRICT));
        assert!(a.pulls.iter().all(|&p| p.abs() < tol::STRICT));
    }

    #[test]
    fn chi2_analysis_known_value() {
        let obs = vec![1.0, 2.0, 3.0];
        let pred = vec![1.1, 1.9, 3.2];
        let a = chi2_analysis(&obs, &pred, 0.1, 0).unwrap();
        // (0.1/0.1)² + (0.1/0.1)² + (0.2/0.1)² = 1 + 1 + 4 = 6
        let expected_chi2 = 6.0_f64;
        assert!(
            (a.chi2_total - expected_chi2).abs() < tol::ANALYTICAL,
            "χ²={}, expected {expected_chi2}",
            a.chi2_total
        );
        assert_eq!(a.contributions.len(), 3);
        assert_eq!(a.residuals.len(), 3);
    }

    #[test]
    fn chi2_analysis_residual_signs() {
        let obs = vec![5.0, 3.0];
        let pred = vec![4.0, 4.0];
        let a = chi2_analysis(&obs, &pred, 1.0, 0).unwrap();
        assert!(
            (a.residuals[0] - 1.0).abs() < tol::STRICT,
            "obs > pred → positive residual"
        );
        assert!(
            (a.residuals[1] - (-1.0)).abs() < tol::STRICT,
            "obs < pred → negative residual"
        );
    }

    #[test]
    fn chi2_analysis_length_mismatch() {
        assert!(chi2_analysis(&[1.0, 2.0], &[1.0], 1.0, 0).is_err());
    }
}
