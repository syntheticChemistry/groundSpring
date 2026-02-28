// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Core statistical primitives shared across all groundSpring experiments.
//!
//! Organised into three domain-focused submodules so each stays well under
//! the 1 000-line quality gate while maintaining clear single-responsibility:
//!
//! | Submodule | Domain |
//! |-----------|--------|
//! | `metrics` | Error metrics (RMSE, MAE, MBE, NSE, R², IA), descriptive stats, hit rate |
//! | `correlation` | Pearson / Spearman correlation, covariance |
//! | `distributions` | Normal CDF / PPF, chi-squared statistic |
//!
//! All functions operate on `&[f64]` slices for zero-copy usage from any
//! data source.  When the `barracuda` feature is enabled, several functions
//! delegate to GPU-ready implementations in the shared crate.

mod correlation;
mod distributions;
mod metrics;
mod regression;

pub use correlation::{covariance, pearson_r, spearman_r};
pub use distributions::{chi2_statistic, norm_cdf, norm_ppf};
pub use metrics::{
    hit_rate, index_of_agreement, mae, mbe, mean, nash_sutcliffe, percentile, r_squared, rmse,
    sample_std_dev, std_dev,
};
pub use regression::{
    fit_exponential, fit_linear, fit_logarithmic, fit_quadratic, LinearFit, NonlinearFit,
};
