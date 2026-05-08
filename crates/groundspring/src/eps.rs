// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Production epsilon guards (division safety, underflow, SSA floor).
//!
//! Test tolerances live in [`crate::tol`].

/// Division-safe epsilon (avoid NaN in `x / y.max(eps::SAFE_DIV)`).
pub const SAFE_DIV: f64 = 1e-10;

/// Gillespie SSA steady-state guard (~10× `f64::EPSILON`).
///
/// Prevents division-by-zero in steady-state mean computation when
/// degradation rate is negligible. The resulting value saturates at a
/// large but finite number rather than infinity.
pub const SSA_FLOOR: f64 = 1e-15;

/// Near-zero guard for log/entropy computations.
///
/// Probabilities below this threshold are treated as zero in entropy
/// sums to avoid `-0 × log(0)` NaN. Also used for coefficient-of-variation
/// denominators in multi-head uncertainty measurement.
pub const LOG_FLOOR: f64 = 1e-15;

/// Strict near-zero guard for quantities with very small physical floors.
///
/// Used for log-log regression filters (MSD), diffusion coefficients
/// (~1e-15 m²/s), and other quantities where `SAFE_DIV` is too generous.
/// Matches `groundspring_validate::tolerances::EPS_SAFE_DIV_STRICT`.
pub const SAFE_DIV_STRICT: f64 = 1e-20;

/// Underflow guard for condition number / matrix element magnitude.
///
/// Used by [`crate::linalg`] QL iteration to detect near-zero off-diagonal
/// elements that would cause division overflow in implicit shift computation.
pub const UNDERFLOW: f64 = 1e-300;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_div_prevents_nan() {
        let result = 1.0 / 0.0_f64.max(SAFE_DIV);
        assert!(result.is_finite());
    }

    #[test]
    fn log_floor_prevents_log_nan() {
        let p = 0.0_f64.max(LOG_FLOOR);
        assert!(p.ln().is_finite());
    }

    #[test]
    fn constants_are_positive() {
        assert!(SAFE_DIV > 0.0);
        assert!(SSA_FLOOR > 0.0);
        assert!(LOG_FLOOR > 0.0);
        assert!(SAFE_DIV_STRICT > 0.0);
        assert!(UNDERFLOW > 0.0);
    }

    #[test]
    fn strict_is_stricter_than_safe() {
        assert!(SAFE_DIV_STRICT < SAFE_DIV);
    }
}
