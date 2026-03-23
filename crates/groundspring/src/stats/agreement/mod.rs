// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Paired-observation agreement and error metrics.
//!
//! All functions take `(observed, modeled)` slice pairs and quantify how
//! well the model reproduces the observations. When the `barracuda`
//! feature is enabled, each metric delegates to `barracuda::stats`.

mod coefficient;
mod efficiency;
mod error_metrics;
mod hit_rate;
mod willmott;

pub use efficiency::{nash_sutcliffe, r_squared};
pub use error_metrics::{mae, mbe, rmse};
pub use hit_rate::hit_rate;
pub use willmott::index_of_agreement;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tol;

    #[test]
    fn rmse_identical_is_zero() {
        let x = [1.0, 2.0, 3.0];
        assert!((rmse(&x, &x)).abs() < tol::EXACT);
    }

    #[test]
    fn rmse_known_value() {
        let obs = [1.0, 2.0, 3.0];
        let modeled = [1.1, 2.1, 3.1];
        assert!((rmse(&obs, &modeled) - 0.1).abs() < tol::ANALYTICAL);
    }

    #[test]
    fn rmse_empty() {
        let empty: [f64; 0] = [];
        assert!(rmse(&empty, &empty).abs() < tol::EXACT);
    }

    #[test]
    fn mbe_overestimate_positive() {
        let obs = [1.0, 2.0, 3.0];
        let modeled = [1.5, 2.5, 3.5];
        assert!((mbe(&obs, &modeled) - 0.5).abs() < tol::EXACT);
    }

    #[test]
    fn mbe_empty() {
        let empty: [f64; 0] = [];
        assert!(mbe(&empty, &empty).abs() < tol::EXACT);
    }

    #[test]
    fn r2_perfect() {
        let x = [1.0, 2.0, 3.0, 4.0];
        assert!((r_squared(&x, &x) - 1.0).abs() < tol::EXACT);
    }

    #[test]
    fn r2_mean_model_is_zero() {
        let obs = [1.0, 2.0, 3.0];
        let modeled = [2.0, 2.0, 2.0];
        assert!(r_squared(&obs, &modeled).abs() < tol::EXACT);
    }

    #[test]
    fn r2_empty() {
        let empty: [f64; 0] = [];
        assert!(r_squared(&empty, &empty).abs() < tol::EXACT);
    }

    #[test]
    fn r2_constant_observation() {
        let obs = [3.0, 3.0, 3.0];
        let modeled = [3.1, 2.9, 3.0];
        assert!(r_squared(&obs, &modeled).abs() < tol::EXACT);
    }

    #[test]
    fn ia_perfect() {
        let x = [1.0, 2.0, 3.0, 4.0];
        assert!((index_of_agreement(&x, &x) - 1.0).abs() < tol::EXACT);
    }

    #[test]
    fn ia_empty() {
        let empty: [f64; 0] = [];
        assert!(index_of_agreement(&empty, &empty).abs() < tol::EXACT);
    }

    #[test]
    fn ia_constant_denominator_zero() {
        let obs = [5.0, 5.0, 5.0];
        let modeled = [5.0, 5.0, 5.0];
        assert!((index_of_agreement(&obs, &modeled)).abs() < tol::EXACT);
    }

    #[test]
    fn hit_rate_perfect() {
        let obs = [0.0, 5.0, 0.0, 3.0];
        assert!((hit_rate(&obs, &obs, 0.1) - 1.0).abs() < tol::EXACT);
    }

    #[test]
    fn hit_rate_known_value() {
        let obs = [0.0, 5.0, 0.0, 3.0];
        let modeled = [0.0, 4.0, 0.0, 0.0];
        assert!((hit_rate(&obs, &modeled, 0.1) - 0.75).abs() < tol::EXACT);
    }

    #[test]
    fn hit_rate_empty() {
        let empty: [f64; 0] = [];
        assert!(hit_rate(&empty, &empty, 0.1).abs() < tol::EXACT);
    }

    #[test]
    fn mae_identical_is_zero() {
        let x = [1.0, 2.0, 3.0];
        assert!(mae(&x, &x).abs() < tol::EXACT);
    }

    #[test]
    fn mae_known_value() {
        let obs = [1.0, 2.0, 3.0];
        let modeled = [1.5, 2.5, 3.5];
        assert!((mae(&obs, &modeled) - 0.5).abs() < tol::EXACT);
    }

    #[test]
    fn mae_empty() {
        let empty: [f64; 0] = [];
        assert!(mae(&empty, &empty).abs() < tol::EXACT);
    }

    #[test]
    fn nse_perfect() {
        let x = [1.0, 2.0, 3.0, 4.0];
        assert!((nash_sutcliffe(&x, &x) - 1.0).abs() < tol::EXACT);
    }

    #[test]
    fn nse_mean_model_is_zero() {
        let obs = [1.0, 2.0, 3.0];
        let modeled = [2.0, 2.0, 2.0];
        assert!(nash_sutcliffe(&obs, &modeled).abs() < tol::EXACT);
    }

    #[test]
    fn nse_empty() {
        let empty: [f64; 0] = [];
        assert!(nash_sutcliffe(&empty, &empty).abs() < tol::EXACT);
    }

    #[test]
    fn nse_equals_r2_for_same_inputs() {
        let obs = [1.0, 2.0, 3.0, 4.0, 5.0];
        let modeled = [1.1, 2.2, 2.8, 4.3, 4.9];
        let nse = nash_sutcliffe(&obs, &modeled);
        let r2 = r_squared(&obs, &modeled);
        assert!(
            (nse - r2).abs() < tol::ANALYTICAL,
            "NSE should equal R² for the same inputs: nse={nse}, r2={r2}"
        );
    }
}
