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
#[cfg_attr(
    not(feature = "barracuda-gpu"),
    expect(dead_code, reason = "SSA GPU batch path behind barracuda-gpu feature")
)]
pub const SSA_FLOOR: f64 = 1e-15;

/// Near-zero guard for log/entropy computations.
///
/// Probabilities below this threshold are treated as zero in entropy
/// sums to avoid `-0 × log(0)` NaN. Also used for coefficient-of-variation
/// denominators in multi-head uncertainty measurement.
pub const LOG_FLOOR: f64 = 1e-15;

/// Underflow guard for condition number / matrix element magnitude.
///
/// Used by [`crate::linalg`] QL iteration to detect near-zero off-diagonal
/// elements that would cause division overflow in implicit shift computation.
pub const UNDERFLOW: f64 = 1e-300;
