// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team
#![forbid(unsafe_code)]

//! Shared helpers for groundSpring validation binaries.
//!
//! Provides typed accessors for benchmark JSON fields and a standard
//! provenance header printer. Each validation binary loads its benchmark
//! via `include_str!` and parses it with `serde_json`; these helpers
//! eliminate the repeated boilerplate across binaries.

use serde_json::Value;
use std::fmt;

// ── Tolerances ───────────────────────────────────────────────────────────
//
// Re-export from `groundspring::tol` where values match, and define
// validation-specific tolerances alongside.

/// f64 identity — values computed by the same deterministic path on
/// identical inputs.  Only IEEE 754 rounding distinguishes them.
pub const TOL_EXACT: f64 = groundspring::tol::EXACT;

/// Exact arithmetic (add / mul / div) on f64 inputs with at most one
/// transcendental (sqrt, ln) introducing ~1 ULP accumulated error.
pub const TOL_ANALYTICAL: f64 = groundspring::tol::ANALYTICAL;

/// Literature values reported to 3–4 significant decimals (e.g.
/// Dong et al. 2020 sensor MBE/RMSE calibrations).
pub const TOL_LITERATURE: f64 = groundspring::tol::LITERATURE;

/// Bias–variance decomposition fractions where the Pythagorean identity
/// RMSE² = MBE² + σ² amplifies rounding near the fourth decimal.
pub const TOL_DECOMPOSITION: f64 = groundspring::tol::DECOMPOSITION;

/// Finite-sample mean estimators from stochastic algorithms (Gillespie,
/// Monte Carlo) where sampling noise is O(1/√N).
pub const TOL_STOCHASTIC_MEAN: f64 = groundspring::tol::STOCHASTIC;

/// ODE equilibrium values and meteorological parameters where physical
/// measurement precision is ~0.1 unit.
pub const TOL_EQUILIBRIUM: f64 = groundspring::tol::EQUILIBRIUM;

/// Deterministic rerun tolerance — same code, same inputs, same seed.
/// Stricter than `TOL_EXACT` because no algorithmic variation is expected.
pub const TOL_DETERMINISM: f64 = groundspring::tol::DETERMINISM;

// ── Validation-specific tolerances (no library counterpart) ──────────

/// Rarefaction taxon proportions at moderate sequencing depth — multinomial
/// sampling variance at N ≈ 50 000.
pub const TOL_RAREFACTION_PROP: f64 = 0.05;

/// Coarse stochastic regime classification (e.g. "all taxa detected")
/// tolerating ±0.5 in count-like quantities.
pub const TOL_REGIME: f64 = 0.5;

/// Grid-search matching tolerance for locating a disorder/coupling value
/// in a sweep array (e.g. `(w - target).abs() < TOL_GRID_MATCH`).
pub const TOL_GRID_MATCH: f64 = 0.01;

/// Monotonicity slack for physical quantities that should decrease but
/// may exhibit minor non-monotonicity from finite sampling.
pub const TOL_MONOTONIC_SLACK: f64 = 0.15;

/// Threshold for strong model performance: R² ≥ 0.95.
/// Statistical regression fit quality — 95% of variance explained.
pub const THRESHOLD_GOOD_R2: f64 = 0.95;

/// Threshold for strong model agreement: IA ≥ 0.9.
/// Willmott Index of Agreement (d) — 0.9 indicates excellent agreement
/// between modeled and observed values.
pub const THRESHOLD_GOOD_IA: f64 = 0.9;

/// Anderson localization: Lyapunov exponent threshold for strong disorder.
/// γ > 0.3 indicates exponential localization in 1D disordered systems.
pub const THRESHOLD_LARGE_GAMMA: f64 = 0.3;

/// Division-safe epsilon to avoid NaN in `x / y.max(EPS_SAFE_DIV)`.
pub const EPS_SAFE_DIV: f64 = 1e-10;

/// Strict division-safe epsilon for quantities where physical floor is ~1e-15
/// (e.g. diffusion coefficients in m²/s). Below any physically meaningful value.
pub const EPS_SAFE_DIV_STRICT: f64 = 1e-20;

/// Rust vs Python ET₀ method-comparison tolerance.
///
/// Same equations, small rounding diffs from trig intermediates (Ra),
/// Kelvin convention (273.0 vs 273.16), `mul_add` vs multiply-then-add.
/// Hargreaves amplifies Ra differences.
///
/// Provenance: `control/et0_methods/et0_methods.py` (commit `a29480fd`,
/// 2026-03-05) — `python3 control/et0_methods/et0_methods.py`.
/// Observed max delta: PM 0.002, HG 0.004, MK 0.001, TU 0.001, HA 0.001.
/// 0.005 provides 1.25× margin over worst case (Hargreaves 0.004).
pub const TOL_ET0: f64 = 0.005;

/// Error returned when a benchmark JSON field is missing or has the wrong type.
#[derive(Debug, Clone)]
pub struct BenchFieldError {
    field: String,
    expected: &'static str,
}

impl fmt::Display for BenchFieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "benchmark field '{}': expected {}",
            self.field, self.expected
        )
    }
}

impl std::error::Error for BenchFieldError {}

/// Alias for benchmark-field extraction results.
pub type BenchResult<T> = Result<T, BenchFieldError>;

/// Try to extract an `f64` from a JSON object.
///
/// # Errors
///
/// Returns [`BenchFieldError`] if the field is absent or not numeric.
pub fn get_f64(v: &Value, key: &str) -> BenchResult<f64> {
    v.get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| BenchFieldError {
            field: key.into(),
            expected: "f64",
        })
}

/// Try to extract a `usize` from a JSON object.
///
/// # Errors
///
/// Returns [`BenchFieldError`] if the field is absent or not an integer.
#[expect(
    clippy::cast_possible_truncation,
    reason = "JSON u64 → usize; benchmark values are small enough"
)]
pub fn get_usize(v: &Value, key: &str) -> BenchResult<usize> {
    v.get(key)
        .and_then(Value::as_u64)
        .map(|n| n as usize)
        .ok_or_else(|| BenchFieldError {
            field: key.into(),
            expected: "u64 (usize)",
        })
}

/// Try to extract a `u64` from a JSON object.
///
/// # Errors
///
/// Returns [`BenchFieldError`] if the field is absent or not an integer.
pub fn get_u64(v: &Value, key: &str) -> BenchResult<u64> {
    v.get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| BenchFieldError {
            field: key.into(),
            expected: "u64",
        })
}

/// Try to extract a string from a JSON object.
///
/// # Errors
///
/// Returns [`BenchFieldError`] if the field is absent or not a string.
pub fn get_str<'a>(v: &'a Value, key: &str) -> BenchResult<&'a str> {
    v.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| BenchFieldError {
            field: key.into(),
            expected: "string",
        })
}

/// Try to extract a `bool` from a JSON object.
///
/// # Errors
///
/// Returns [`BenchFieldError`] if the field is absent or not a boolean.
pub fn get_bool(v: &Value, key: &str) -> BenchResult<bool> {
    v.get(key)
        .and_then(Value::as_bool)
        .ok_or_else(|| BenchFieldError {
            field: key.into(),
            expected: "bool",
        })
}

/// Try to extract a JSON array from a JSON object.
///
/// # Errors
///
/// Returns [`BenchFieldError`] if the field is absent or not an array.
pub fn get_array<'a>(v: &'a Value, key: &str) -> BenchResult<&'a Vec<Value>> {
    v.get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| BenchFieldError {
            field: key.into(),
            expected: "array",
        })
}

/// Try to extract a `Vec<f64>` from a JSON array field.
///
/// # Errors
///
/// Returns [`BenchFieldError`] if the field is absent, not an array,
/// or contains non-numeric elements.
pub fn get_f64_vec(v: &Value, key: &str) -> BenchResult<Vec<f64>> {
    let arr = get_array(v, key)?;
    arr.iter()
        .enumerate()
        .map(|(i, el)| {
            el.as_f64().ok_or_else(|| BenchFieldError {
                field: format!("{key}[{i}]"),
                expected: "f64",
            })
        })
        .collect()
}

/// Try to extract a two-element `[lo, hi]` range from a JSON array.
///
/// # Errors
///
/// Returns [`BenchFieldError`] if the value is not an array or its
/// first two elements are not numeric.
pub fn get_f64_range(arr: &Value) -> BenchResult<(f64, f64)> {
    let a = arr.as_array().ok_or_else(|| BenchFieldError {
        field: "(range)".into(),
        expected: "array",
    })?;
    let lo = a
        .first()
        .and_then(Value::as_f64)
        .ok_or_else(|| BenchFieldError {
            field: "(range)[0]".into(),
            expected: "f64",
        })?;
    let hi = a
        .get(1)
        .and_then(Value::as_f64)
        .ok_or_else(|| BenchFieldError {
            field: "(range)[1]".into(),
            expected: "f64",
        })?;
    Ok((lo, hi))
}

// ── Legacy convenience API (panicking wrappers) ─────────────────────
//
// These delegate to the `get_*` functions and panic on `Err`. Suitable
// for validation binaries where the JSON is `include_str!`-ed at compile
// time and a missing field is a programmer error, not a runtime condition.

/// Try to extract an `f64` from a JSON object, returning `None` on
/// missing or non-numeric fields.
#[must_use]
pub fn try_f64_field(v: &Value, key: &str) -> Option<f64> {
    get_f64(v, key).ok()
}

/// Extract an `f64` from a JSON object.
///
/// # Panics
///
/// Panics if `v[key]` is absent or not representable as `f64`.
#[must_use]
pub fn f64_field(v: &Value, key: &str) -> f64 {
    get_f64(v, key).expect("benchmark f64 field")
}

/// Try to extract a `usize` from a JSON object, returning `None` on
/// missing or non-integer fields.
#[must_use]
pub fn try_usize_field(v: &Value, key: &str) -> Option<usize> {
    get_usize(v, key).ok()
}

/// Extract a `usize` from a JSON object.
///
/// # Panics
///
/// Panics if `v[key]` is absent or not representable as `u64`.
#[must_use]
pub fn usize_field(v: &Value, key: &str) -> usize {
    get_usize(v, key).expect("benchmark usize field")
}

/// Extract a `u64` from a JSON object.
///
/// # Panics
///
/// Panics if `v[key]` is absent or not representable as `u64`.
#[must_use]
pub fn u64_field(v: &Value, key: &str) -> u64 {
    get_u64(v, key).expect("benchmark u64 field")
}

/// Extract a two-element `[lo, hi]` range from a JSON array.
///
/// # Panics
///
/// Panics if the value is not a JSON array with at least two numeric elements.
#[must_use]
pub fn f64_range(arr: &Value) -> (f64, f64) {
    get_f64_range(arr).expect("benchmark f64 range")
}

/// Try to extract a string field from a JSON object, returning `None` on
/// missing or non-string fields.
#[must_use]
pub fn try_str_field<'a>(v: &'a Value, key: &str) -> Option<&'a str> {
    get_str(v, key).ok()
}

/// Extract a string field from a JSON object.
///
/// # Panics
///
/// Panics if `v[key]` is absent or not a string.
#[must_use]
pub fn str_field<'a>(v: &'a Value, key: &str) -> &'a str {
    get_str(v, key).expect("benchmark str field")
}

/// Extract a JSON array field from a JSON object.
///
/// # Panics
///
/// Panics if `v[key]` is absent or not an array.
#[must_use]
pub fn array_field<'a>(v: &'a Value, key: &str) -> &'a Vec<Value> {
    get_array(v, key).expect("benchmark array field")
}

/// Extract a `Vec<f64>` from a JSON array field.
///
/// # Panics
///
/// Panics if `v[key]` is absent, not an array, or contains non-numeric elements.
#[must_use]
pub fn f64_vec(v: &Value, key: &str) -> Vec<f64> {
    get_f64_vec(v, key).expect("benchmark f64 vec")
}

/// Extract a `bool` from a JSON object.
///
/// # Panics
///
/// Panics if `v[key]` is absent or not a boolean.
#[must_use]
pub fn bool_field(v: &Value, key: &str) -> bool {
    get_bool(v, key).expect("benchmark bool field")
}

/// Print the standard provenance header shared by all validation binaries.
///
/// Displays source, baseline commit/date, and (when present) the script,
/// command, and author that generated the baseline — full chain of custody.
///
/// # Panics
///
/// Panics if the benchmark JSON is missing `_source` or if `_provenance`
/// is missing `baseline_commit` or `baseline_date`.
pub fn print_provenance_header(bench: &Value, title: &str) {
    println!("{}", "=".repeat(72));
    println!("groundSpring Rust Validation: {title}");
    println!(
        "  Source: {}",
        bench["_source"]
            .as_str()
            .expect("benchmark JSON missing _source")
    );
    let prov = &bench["_provenance"];
    println!(
        "  Provenance: commit {}, {}",
        prov["baseline_commit"]
            .as_str()
            .expect("provenance missing baseline_commit"),
        prov["baseline_date"]
            .as_str()
            .expect("provenance missing baseline_date"),
    );
    if let Some(script) = prov["validation_script"]
        .as_str()
        .or_else(|| bench["validation_script"].as_str())
    {
        println!("  Script: {script}");
    }
    if let Some(cmd) = prov["command"]
        .as_str()
        .or_else(|| bench["command"].as_str())
    {
        println!("  Command: {cmd}");
    }
    if let Some(author) = prov["generated_by"]
        .as_str()
        .or_else(|| bench["generated_by"].as_str())
    {
        println!("  Author: {author}");
    }
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
    #[should_panic(expected = "benchmark f64 field")]
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
    #[should_panic(expected = "benchmark usize field")]
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
    #[should_panic(expected = "benchmark u64 field")]
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
    #[should_panic(expected = "benchmark f64 range")]
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
    #[should_panic(expected = "benchmark JSON missing _source")]
    fn print_provenance_header_panics_on_missing_source() {
        let bench = json!({"_source": null, "_provenance": {}});
        print_provenance_header(&bench, "Fallback");
    }

    #[test]
    fn try_f64_field_returns_some() {
        let v = json!({"temp": 20.5});
        assert_eq!(try_f64_field(&v, "temp"), Some(20.5));
    }

    #[test]
    fn try_f64_field_returns_none_on_missing() {
        let v = json!({"temp": 20.5});
        assert_eq!(try_f64_field(&v, "absent"), None);
    }

    #[test]
    fn try_usize_field_returns_some() {
        let v = json!({"n": 100});
        assert_eq!(try_usize_field(&v, "n"), Some(100));
    }

    #[test]
    fn try_usize_field_returns_none_on_missing() {
        let v = json!({"n": 100});
        assert_eq!(try_usize_field(&v, "absent"), None);
    }

    #[test]
    fn try_str_field_returns_some() {
        let v = json!({"name": "exp008"});
        assert_eq!(try_str_field(&v, "name"), Some("exp008"));
    }

    #[test]
    fn try_str_field_returns_none_on_missing() {
        let v = json!({"name": "exp008"});
        assert_eq!(try_str_field(&v, "absent"), None);
    }

    #[test]
    fn f64_range_extracts_from_longer_array() {
        let v = json!([1.0, 2.0, 3.0]);
        let (lo, hi) = f64_range(&v);
        assert!((lo - 1.0).abs() < f64::EPSILON);
        assert!((hi - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn get_f64_returns_ok() {
        let v = json!({"temp": 20.5});
        assert!((get_f64(&v, "temp").unwrap() - 20.5).abs() < f64::EPSILON);
    }

    #[test]
    fn get_f64_returns_err_on_missing() {
        let v = json!({"temp": 20.5});
        assert!(get_f64(&v, "absent").is_err());
    }

    #[test]
    fn get_usize_returns_ok() {
        let v = json!({"n": 100});
        assert_eq!(get_usize(&v, "n").unwrap(), 100);
    }

    #[test]
    fn get_usize_returns_err_on_missing() {
        let v = json!({"n": 100});
        assert!(get_usize(&v, "absent").is_err());
    }

    #[test]
    fn get_f64_vec_returns_ok() {
        let v = json!({"data": [1.0, 2.0, 3.0]});
        let vec = get_f64_vec(&v, "data").unwrap();
        assert_eq!(vec.len(), 3);
        assert!((vec[0] - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn get_f64_vec_returns_err_on_non_numeric() {
        let v = json!({"data": [1.0, "two"]});
        assert!(get_f64_vec(&v, "data").is_err());
    }

    #[test]
    fn get_f64_range_returns_ok() {
        let v = json!([1.5, 3.7]);
        let (lo, hi) = get_f64_range(&v).unwrap();
        assert!((lo - 1.5).abs() < f64::EPSILON);
        assert!((hi - 3.7).abs() < f64::EPSILON);
    }

    #[test]
    fn get_f64_range_returns_err_on_non_array() {
        let v = json!(42.0);
        assert!(get_f64_range(&v).is_err());
    }

    #[test]
    fn bench_field_error_display_is_informative() {
        let v = json!({"x": "not_a_number"});
        let err = get_f64(&v, "x").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains('x'));
        assert!(msg.contains("f64"));
    }
}
