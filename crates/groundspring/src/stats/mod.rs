// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Core statistical primitives shared across all groundSpring experiments.
//!
//! Organised into domain-focused submodules so each stays well under
//! the 1 000-line quality gate while maintaining clear single-responsibility:
//!
//! | Submodule | Domain |
//! |-----------|--------|
//! | `agreement` | Paired-observation error/agreement metrics (RMSE, MAE, MBE, NSE, R², IA, hit rate) |
//! | `metrics` | Descriptive statistics (mean, `std_dev`, percentile) |
//! | `correlation` | Pearson / Spearman correlation, covariance |
//! | `distributions` | Normal CDF / PPF, chi-squared statistic |
//! | `regression` | Linear, exponential, logarithmic, quadratic fits |
//! | `moving_window` | Sliding window mean, variance, min, max |
//!
//! All functions operate on `&[f64]` slices for zero-copy usage from any
//! data source.  When the `barracuda` feature is enabled, several functions
//! delegate to GPU-ready implementations in the shared crate.

mod agreement;
mod correlation;
mod distributions;
mod metrics;
pub mod moving_window;
mod regression;

pub use agreement::{hit_rate, index_of_agreement, mae, mbe, nash_sutcliffe, r_squared, rmse};
pub use correlation::{covariance, pearson_r, spearman_r};
pub use distributions::{chi2_statistic, norm_cdf, norm_ppf};
pub use metrics::{mean, percentile, sample_std_dev, std_dev};
pub use moving_window::{moving_window_stats, MovingWindowResult};
pub use regression::{
    fit_exponential, fit_linear, fit_logarithmic, fit_quadratic, LinearFit, NonlinearFit,
};
