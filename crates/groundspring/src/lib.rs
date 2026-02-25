// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! # groundSpring
//!
//! Measurement noise characterization primitives for the ecoPrimals ecosystem.
//!
//! groundSpring provides the statistical building blocks for decomposing
//! measurement error into correctable bias and irreducible noise across
//! scientific domains.
//!
//! ## Modules
//!
//! - [`stats`] — Core statistical metrics (RMSE, MBE, R², IA, hit rate)
//! - [`decompose`] — Bias-variance error decomposition
//! - [`prng`] — Deterministic pseudo-random number generation
//! - [`rarefaction`] — Multinomial rarefaction for sequencing noise analysis
//! - [`seismic`] — Travel-time computation and source inversion
//! - [`fao56`] — FAO-56 Penman-Monteith reference evapotranspiration
//! - [`validate`] — Validation harness (pass/fail with counters)

pub mod decompose;
pub mod fao56;
pub mod prng;
pub mod rarefaction;
pub mod seismic;
pub mod stats;
pub mod validate;

/// Centralized numeric cast helpers.
///
/// `usize` and `u64` → `f64` conversions are unavoidable in numerical code
/// (Rust has no `From<usize>` for `f64` because usize may be 64-bit).
/// These helpers document the safety argument once and keep cast lints
/// targeted rather than blanket-allowed.
pub(crate) mod cast {
    /// Convert a collection length (`usize`) to `f64`.
    ///
    /// Exact for lengths up to 2^53 (≈ 9 × 10¹⁵), far beyond practical memory.
    #[inline]
    #[expect(clippy::cast_precision_loss)]
    pub const fn usize_f64(n: usize) -> f64 {
        n as f64
    }

    /// Convert a `u64` count to `f64`.
    ///
    /// Exact for values up to 2^53.  Used in rarefaction and PRNG where
    /// counts are sequencing depths or taxonomic totals.
    #[inline]
    #[expect(clippy::cast_precision_loss)]
    pub const fn u64_f64(n: u64) -> f64 {
        n as f64
    }

    /// Convert a non-negative `f64` to `usize` (truncating toward zero).
    ///
    /// Used for index computation from floating-point rank/position values.
    #[inline]
    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub const fn f64_usize(x: f64) -> usize {
        x as usize
    }
}
