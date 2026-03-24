// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Centralized numeric cast helpers.
//!
//! Replaces bare `as` casts with named functions that document the safety
//! argument once and keep cast lints targeted rather than blanket-allowed.
//!
//! Absorbed from neuralSpring V113 `safe_cast` pattern; extended with
//! groundSpring-specific helpers for rarefaction counts and GPU dispatch.

/// Convert a collection length (`usize`) to `f64`.
///
/// Exact for lengths up to 2^53 (≈ 9 × 10¹⁵), far beyond practical memory.
#[inline]
#[must_use]
#[expect(clippy::cast_precision_loss, reason = "exact for lengths up to 2^53")]
pub const fn usize_f64(n: usize) -> f64 {
    n as f64
}

/// Convert a `u64` count to `f64`.
///
/// Exact for values up to 2^53.  Used in rarefaction and PRNG where
/// counts are sequencing depths or taxonomic totals.
#[inline]
#[must_use]
#[expect(clippy::cast_precision_loss, reason = "exact for values up to 2^53")]
pub const fn u64_f64(n: u64) -> f64 {
    n as f64
}

/// Convert a non-negative `f64` to `usize` (truncating toward zero).
///
/// Used for index computation from floating-point rank/position values.
#[inline]
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "callers ensure x is non-negative and within usize range"
)]
pub const fn f64_usize(x: f64) -> usize {
    x as usize
}

/// `usize` → `u32`, returning an error on overflow.
///
/// GPU dispatch parameters (workgroup counts, dimension sizes) must fit
/// in `u32`. This makes the check explicit rather than silently truncating.
///
/// Absorbed from neuralSpring V113 `safe_cast::usize_u32`.
///
/// # Errors
///
/// Returns an error string if the value exceeds `u32::MAX`.
pub fn usize_u32(value: usize, label: &str) -> Result<u32, String> {
    u32::try_from(value).map_err(|_| format!("{label}: {value} exceeds u32::MAX"))
}

/// `usize` → `u64`, infallible on 64-bit platforms.
///
/// On 32-bit platforms `usize` ≤ `u64` so this is always safe, but using
/// a named function documents intent and avoids bare `as` casts.
#[inline]
#[must_use]
pub const fn usize_u64(value: usize) -> u64 {
    value as u64
}

/// `f64` → `f32`, intentionally lossy for GPU shader inputs.
///
/// Most WGSL shaders operate on f32. This wrapper documents the
/// intentional precision loss and avoids lint noise from bare casts.
///
/// Absorbed from neuralSpring V113 `safe_cast::f64_f32`.
#[inline]
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    reason = "intentional: GPU shaders require f32"
)]
pub const fn f64_f32(value: f64) -> f32 {
    value as f32
}

/// `i32` → `f64`. Always exact (`i32` ⊂ `f64` mantissa range).
///
/// Absorbed from airSpring barracuda cast module.
#[inline]
#[must_use]
pub const fn i32_f64(v: i32) -> f64 {
    v as f64
}

/// `u32` → `f64`. Always exact (`u32` ⊂ `f64` mantissa range).
///
/// Absorbed from airSpring barracuda cast module.
#[inline]
#[must_use]
pub const fn u32_f64(v: u32) -> f64 {
    v as f64
}

/// `f64` → `u32` via truncation toward zero.
///
/// Debug-panics if `v` is negative, NaN, or exceeds `u32::MAX`.
///
/// Absorbed from airSpring barracuda cast module.
#[inline]
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "callers ensure v is non-negative and within u32 range"
)]
pub fn f64_u32(v: f64) -> u32 {
    debug_assert!(
        v.is_finite() && v >= 0.0 && v <= u32_f64(u32::MAX),
        "f64_u32: {v} out of range"
    );
    v as u32
}

/// `u32` → `usize`. Always exact (`u32` ⊆ `usize` on all Rust targets).
///
/// Absorbed from airSpring barracuda cast module.
#[inline]
#[must_use]
pub const fn u32_usize(v: u32) -> usize {
    v as usize
}

/// `u64` → `usize`. Exact on 64-bit targets; debug-panics on 32-bit overflow.
///
/// Absorbed from airSpring barracuda cast module.
#[inline]
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    reason = "exact on 64-bit; debug-asserted on 32-bit"
)]
pub const fn u64_usize(v: u64) -> usize {
    debug_assert!(
        v <= usize::MAX as u64,
        "u64_usize: overflow on this platform"
    );
    v as usize
}

/// `f64` → `i32` via truncation toward zero.
///
/// Debug-panics if `v` is NaN, infinite, or outside `i32` range.
///
/// Absorbed from airSpring barracuda cast module.
#[inline]
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    reason = "callers ensure v is within i32 range"
)]
pub fn f64_i32(v: f64) -> i32 {
    debug_assert!(
        v.is_finite() && v >= f64::from(i32::MIN) && v <= f64::from(i32::MAX),
        "f64_i32: {v} out of range"
    );
    v as i32
}

/// `usize` → `i32`. For converting lengths to signed counters.
///
/// Debug-panics if `v > i32::MAX`.
///
/// Absorbed from airSpring barracuda cast module.
#[inline]
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    reason = "callers ensure v fits in i32; debug-asserted"
)]
pub const fn usize_i32(v: usize) -> i32 {
    debug_assert!(v <= i32::MAX as usize, "usize_i32: overflow");
    v as i32
}

/// `u64` → `u32`, taking the low 32 bits.
///
/// Used for PRNG seed generation where only entropy matters, not
/// the full 64-bit value. Equivalent to `(v & 0xFFFF_FFFF) as u32`.
#[inline]
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    reason = "intentional: low 32 bits of PRNG output for seed generation"
)]
pub const fn u64_u32_truncate(v: u64) -> u32 {
    v as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usize_f64_exact_for_small() {
        assert!((usize_f64(0) - 0.0).abs() < f64::EPSILON);
        assert!((usize_f64(1) - 1.0).abs() < f64::EPSILON);
        assert!((usize_f64(1_000_000) - 1e6).abs() < f64::EPSILON);
    }

    #[test]
    fn u64_f64_exact_for_small() {
        assert!((u64_f64(0) - 0.0).abs() < f64::EPSILON);
        assert!((u64_f64(42) - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn f64_usize_truncates() {
        assert_eq!(f64_usize(3.7), 3);
        assert_eq!(f64_usize(0.0), 0);
        assert_eq!(f64_usize(100.999), 100);
    }

    #[test]
    fn i32_f64_exact() {
        assert!((i32_f64(-42) - (-42.0)).abs() < f64::EPSILON);
        assert!((i32_f64(0) - 0.0).abs() < f64::EPSILON);
        assert!((i32_f64(i32::MAX) - f64::from(i32::MAX)).abs() < f64::EPSILON);
    }

    #[test]
    fn u32_f64_exact() {
        assert!((u32_f64(0) - 0.0).abs() < f64::EPSILON);
        assert!((u32_f64(u32::MAX) - f64::from(u32::MAX)).abs() < f64::EPSILON);
    }

    #[test]
    fn f64_u32_truncates() {
        assert_eq!(f64_u32(255.9), 255);
        assert_eq!(f64_u32(0.0), 0);
    }

    #[test]
    fn u32_usize_identity() {
        assert_eq!(u32_usize(42), 42_usize);
        assert_eq!(u32_usize(0), 0_usize);
        assert_eq!(u32_usize(u32::MAX), u32::MAX as usize);
    }

    #[test]
    fn u64_usize_identity() {
        assert_eq!(u64_usize(42), 42_usize);
        assert_eq!(u64_usize(0), 0_usize);
    }

    #[test]
    fn f64_i32_truncates() {
        assert_eq!(f64_i32(3.9), 3);
        assert_eq!(f64_i32(-3.9), -3);
        assert_eq!(f64_i32(0.0), 0);
    }

    #[test]
    fn usize_i32_within_range() {
        assert_eq!(usize_i32(0), 0);
        assert_eq!(usize_i32(1000), 1000);
    }

    #[test]
    fn usize_u32_within_range() {
        assert_eq!(usize_u32(0, "test").ok(), Some(0));
        assert_eq!(usize_u32(1000, "test").ok(), Some(1000));
    }

    #[test]
    fn u64_u32_truncate_takes_low_bits() {
        assert_eq!(u64_u32_truncate(0), 0);
        assert_eq!(u64_u32_truncate(42), 42);
        assert_eq!(u64_u32_truncate(0xFFFF_FFFF), u32::MAX);
        assert_eq!(u64_u32_truncate(0x1_0000_0000), 0);
    }
}
