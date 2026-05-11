// SPDX-License-Identifier: AGPL-3.0-or-later
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
pub(crate) mod metrics;
pub mod model_selection;
pub mod moving_window;
mod regression;

pub use agreement::{hit_rate, index_of_agreement, mae, mbe, nash_sutcliffe, r_squared, rmse};
pub use correlation::{CorrelationFull, covariance, pearson_full, pearson_r, spearman_r};
pub use distributions::{chi2_statistic, norm_cdf, norm_ppf};
pub use metrics::{mean, mean_and_std_dev, percentile, sample_std_dev, std_dev};
pub use model_selection::{ModelComparison, aic, bic, compare_models, rss};
pub use moving_window::{MovingWindowResult, moving_window_stats};
pub use regression::{
    LinearFit, NonlinearFit, fit_all, fit_exponential, fit_hyperbolic, fit_linear, fit_logarithmic,
    fit_power_law, fit_quadratic,
};
