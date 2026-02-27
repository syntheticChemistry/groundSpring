// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Shared helpers for groundSpring validation binaries.
//!
//! Provides typed accessors for benchmark JSON fields and a standard
//! provenance header printer. Each validation binary loads its benchmark
//! via `include_str!` and parses it with `serde_json`; these helpers
//! eliminate the repeated boilerplate across binaries.

use serde_json::Value;

/// f64 identity — values computed by the same deterministic path on
/// identical inputs.  Only IEEE 754 rounding distinguishes them.
pub const TOL_EXACT: f64 = 1e-12;

/// Exact arithmetic (add / mul / div) on f64 inputs with at most one
/// transcendental (sqrt, ln) introducing ~1 ULP accumulated error.
pub const TOL_ANALYTICAL: f64 = 1e-10;

/// Literature values reported to 3–4 significant decimals (e.g.
/// Dong et al. 2020 sensor MBE/RMSE calibrations).
pub const TOL_LITERATURE: f64 = 0.001;

/// Bias–variance decomposition fractions where the Pythagorean identity
/// RMSE² = MBE² + σ² amplifies rounding near the fourth decimal.
pub const TOL_DECOMPOSITION: f64 = 0.005;

/// Finite-sample mean estimators from stochastic algorithms (Gillespie,
/// Monte Carlo) where sampling noise is O(1/√N).
pub const TOL_STOCHASTIC_MEAN: f64 = 0.01;

/// ODE equilibrium values and meteorological parameters where physical
/// measurement precision is ~0.1 unit.
pub const TOL_EQUILIBRIUM: f64 = 0.1;

/// Rarefaction taxon proportions at moderate sequencing depth — multinomial
/// sampling variance at N ≈ 50 000.
pub const TOL_RAREFACTION_PROP: f64 = 0.05;

/// Coarse stochastic regime classification (e.g. "all taxa detected")
/// tolerating ±0.5 in count-like quantities.
pub const TOL_REGIME: f64 = 0.5;

/// Extract an `f64` from a JSON object, panicking with a clear message on
/// missing or non-numeric fields.
///
/// # Panics
///
/// Panics if `v[key]` is absent or not representable as `f64`.
#[must_use]
pub fn f64_field(v: &Value, key: &str) -> f64 {
    v[key]
        .as_f64()
        .unwrap_or_else(|| panic!("missing f64 field: {key}"))
}

/// Extract a `usize` from a JSON object.
///
/// # Panics
///
/// Panics if `v[key]` is absent or not representable as `u64`.
#[must_use]
#[expect(clippy::cast_possible_truncation)]
pub fn usize_field(v: &Value, key: &str) -> usize {
    v[key]
        .as_u64()
        .unwrap_or_else(|| panic!("missing u64 field: {key}")) as usize
}

/// Extract a `u64` from a JSON object.
///
/// # Panics
///
/// Panics if `v[key]` is absent or not representable as `u64`.
#[must_use]
pub fn u64_field(v: &Value, key: &str) -> u64 {
    v[key]
        .as_u64()
        .unwrap_or_else(|| panic!("missing u64 field: {key}"))
}

/// Extract a two-element `[lo, hi]` range from a JSON array.
///
/// # Panics
///
/// Panics if the value is not a JSON array with at least two numeric elements.
#[must_use]
pub fn f64_range(arr: &Value) -> (f64, f64) {
    let a = arr.as_array().expect("expected JSON array for range");
    (
        a[0].as_f64().expect("range lower bound"),
        a[1].as_f64().expect("range upper bound"),
    )
}

/// Print the standard provenance header shared by all validation binaries.
pub fn print_provenance_header(bench: &Value, title: &str) {
    println!("{}", "=".repeat(72));
    println!("groundSpring Rust Validation: {title}");
    println!(
        "  Source: {}",
        bench["_source"].as_str().unwrap_or("unknown")
    );
    println!(
        "  Provenance: commit {}, {}",
        bench["_provenance"]["baseline_commit"]
            .as_str()
            .unwrap_or("unknown"),
        bench["_provenance"]["baseline_date"]
            .as_str()
            .unwrap_or("unknown"),
    );
    println!("{}", "=".repeat(72));
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn f64_field_extracts_value() {
        let v = json!({"temp": 20.5});
        assert!((f64_field(&v, "temp") - 20.5).abs() < f64::EPSILON);
    }

    #[test]
    fn f64_field_extracts_integer_as_f64() {
        let v = json!({"count": 42});
        assert!((f64_field(&v, "count") - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    #[should_panic(expected = "missing f64 field: absent")]
    fn f64_field_panics_on_missing_key() {
        let v = json!({"temp": 20.5});
        let _ = f64_field(&v, "absent");
    }

    #[test]
    fn usize_field_extracts_value() {
        let v = json!({"n": 100});
        assert_eq!(usize_field(&v, "n"), 100);
    }

    #[test]
    #[should_panic(expected = "missing u64 field: absent")]
    fn usize_field_panics_on_missing_key() {
        let v = json!({"n": 100});
        let _ = usize_field(&v, "absent");
    }

    #[test]
    fn u64_field_extracts_value() {
        let v = json!({"seed": 42});
        assert_eq!(u64_field(&v, "seed"), 42);
    }

    #[test]
    #[should_panic(expected = "missing u64 field: absent")]
    fn u64_field_panics_on_missing_key() {
        let v = json!({"seed": 42});
        let _ = u64_field(&v, "absent");
    }

    #[test]
    fn f64_range_extracts_bounds() {
        let v = json!([1.5, 3.7]);
        let (lo, hi) = f64_range(&v);
        assert!((lo - 1.5).abs() < f64::EPSILON);
        assert!((hi - 3.7).abs() < f64::EPSILON);
    }

    #[test]
    #[should_panic(expected = "expected JSON array for range")]
    fn f64_range_panics_on_non_array() {
        let v = json!(42.0);
        let _ = f64_range(&v);
    }

    #[test]
    fn print_provenance_header_does_not_panic() {
        let bench = json!({
            "_source": "Test experiment",
            "_provenance": {
                "baseline_commit": "abc1234",
                "baseline_date": "2026-02-27"
            }
        });
        print_provenance_header(&bench, "Test Title");
    }

    #[test]
    fn print_provenance_header_handles_missing_fields() {
        let bench = json!({"_source": null, "_provenance": {}});
        print_provenance_header(&bench, "Fallback");
    }

    #[test]
    fn f64_range_extracts_from_longer_array() {
        let v = json!([1.0, 2.0, 3.0]);
        let (lo, hi) = f64_range(&v);
        assert!((lo - 1.0).abs() < f64::EPSILON);
        assert!((hi - 2.0).abs() < f64::EPSILON);
    }
}
