// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team
#![forbid(unsafe_code)]
#![deny(clippy::expect_used, clippy::unwrap_used)]

//! Shared helpers for groundSpring validation binaries.
//!
//! Provides typed accessors for benchmark JSON fields and a standard
//! provenance header printer. Each validation binary loads its benchmark
//! via `include_str!` and parses it with `serde_json`; these helpers
//! eliminate the repeated boilerplate across binaries.

pub mod provenance;
pub mod tolerances;
pub use provenance::*;
pub use tolerances::*;

use serde_json::Value;
use std::fmt;

/// Zero-panic exit trait for validation binaries.
///
/// Replaces the `let Ok(v) = expr else { eprintln!("FATAL: ..."); return 1; }`
/// boilerplate in every validation binary with a clean `.or_exit(msg)` call.
///
/// Pattern source: wetSpring V123 / healthSpring V31 `OrExit<T>`.
pub trait OrExit<T> {
    /// Unwrap the value or print `msg` to stderr and exit with code 1.
    fn or_exit(self, msg: &str) -> T;
}

impl<T, E: fmt::Display> OrExit<T> for Result<T, E> {
    fn or_exit(self, msg: &str) -> T {
        match self {
            Ok(v) => v,
            Err(e) => {
                eprintln!("FATAL: {msg}: {e}");
                std::process::exit(exit_code::GENERAL_ERROR);
            }
        }
    }
}

impl<T> OrExit<T> for Option<T> {
    fn or_exit(self, msg: &str) -> T {
        self.unwrap_or_else(|| {
            eprintln!("FATAL: {msg}");
            std::process::exit(exit_code::GENERAL_ERROR);
        })
    }
}

/// Standardized exit codes per `UNIBIN_ARCHITECTURE_STANDARD`.
///
/// Pattern source: sweetGrass v0.7.19 `exit_code` module.
pub mod exit_code {
    /// Successful execution.
    pub const SUCCESS: i32 = 0;
    /// General runtime failure.
    pub const GENERAL_ERROR: i32 = 1;
    /// Configuration or benchmark parsing error.
    pub const CONFIG_ERROR: i32 = 78;
    /// Network or IPC error.
    pub const NETWORK_ERROR: i32 = 76;
}

/// Parse a benchmark JSON string, exiting on failure.
///
/// Replaces the repeated `let Ok(bench) = serde_json::from_str::<Value>(s)
/// else { eprintln!("FATAL: ..."); return 1; }` pattern in every validation binary.
#[must_use]
pub fn parse_benchmark(json_str: &str) -> Value {
    serde_json::from_str::<Value>(json_str).or_exit("invalid benchmark JSON")
}

/// Error returned when a benchmark JSON field is missing or has the wrong type.
#[derive(Debug, Clone, thiserror::Error)]
#[error("benchmark field '{field}': expected {expected}")]
pub struct BenchFieldError {
    field: String,
    expected: &'static str,
}

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
#[expect(
    clippy::expect_used,
    reason = "compile-time JSON; missing field is a programmer error"
)]
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
#[expect(
    clippy::expect_used,
    reason = "compile-time JSON; missing field is a programmer error"
)]
pub fn usize_field(v: &Value, key: &str) -> usize {
    get_usize(v, key).expect("benchmark usize field")
}

/// Extract a `u64` from a JSON object.
///
/// # Panics
///
/// Panics if `v[key]` is absent or not representable as `u64`.
#[must_use]
#[expect(
    clippy::expect_used,
    reason = "compile-time JSON; missing field is a programmer error"
)]
pub fn u64_field(v: &Value, key: &str) -> u64 {
    get_u64(v, key).expect("benchmark u64 field")
}

/// Extract a two-element `[lo, hi]` range from a JSON array.
///
/// # Panics
///
/// Panics if the value is not a JSON array with at least two numeric elements.
#[must_use]
#[expect(
    clippy::expect_used,
    reason = "compile-time JSON; missing field is a programmer error"
)]
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
#[expect(
    clippy::expect_used,
    reason = "compile-time JSON; missing field is a programmer error"
)]
pub fn str_field<'a>(v: &'a Value, key: &str) -> &'a str {
    get_str(v, key).expect("benchmark str field")
}

/// Extract a JSON array field from a JSON object.
///
/// # Panics
///
/// Panics if `v[key]` is absent or not an array.
#[must_use]
#[expect(
    clippy::expect_used,
    reason = "compile-time JSON; missing field is a programmer error"
)]
pub fn array_field<'a>(v: &'a Value, key: &str) -> &'a Vec<Value> {
    get_array(v, key).expect("benchmark array field")
}

/// Extract a `Vec<f64>` from a JSON array field.
///
/// # Panics
///
/// Panics if `v[key]` is absent, not an array, or contains non-numeric elements.
#[must_use]
#[expect(
    clippy::expect_used,
    reason = "compile-time JSON; missing field is a programmer error"
)]
pub fn f64_vec(v: &Value, key: &str) -> Vec<f64> {
    get_f64_vec(v, key).expect("benchmark f64 vec")
}

/// Extract a `bool` from a JSON object.
///
/// # Panics
///
/// Panics if `v[key]` is absent or not a boolean.
#[must_use]
#[expect(
    clippy::expect_used,
    reason = "compile-time JSON; missing field is a programmer error"
)]
pub fn bool_field(v: &Value, key: &str) -> bool {
    get_bool(v, key).expect("benchmark bool field")
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
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
    fn print_provenance_header_succeeds() {
        let bench = json!({
            "_source": "Test experiment",
            "_provenance": {
                "baseline_commit": "abc1234",
                "baseline_date": "2026-02-27"
            }
        });
        try_print_provenance_header(&bench, "Test Title").unwrap();
    }

    #[test]
    fn try_print_provenance_header_err_on_missing_source() {
        let bench = json!({"_source": null, "_provenance": {}});
        assert!(try_print_provenance_header(&bench, "Fallback").is_err());
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

    /// Provenance registry completeness test (neuralSpring V120 pattern).
    ///
    /// Verifies all 29 benchmark JSONs are present and parseable at compile
    /// time. If a benchmark is added or removed, update `EXPECTED_BENCHMARKS`.
    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "registry test is necessarily long — one entry per benchmark JSON"
    )]
    fn provenance_registry_completeness() {
        const EXPECTED_BENCHMARKS: usize = 29;

        let benchmarks: &[(&str, &str)] = &[
            (
                "sensor_noise",
                include_str!("../../../control/sensor_noise/benchmark_sensor_noise.json"),
            ),
            (
                "observation_gap",
                include_str!("../../../control/observation_gap/benchmark_observation_gap.json"),
            ),
            (
                "error_propagation",
                include_str!("../../../control/error_propagation/benchmark_error_propagation.json"),
            ),
            (
                "sequencing_noise",
                include_str!("../../../control/sequencing_noise/benchmark_sequencing_noise.json"),
            ),
            (
                "seismic",
                include_str!("../../../control/seismic/benchmark_seismic.json"),
            ),
            (
                "signal_specificity",
                include_str!(
                    "../../../control/signal_specificity/benchmark_signal_specificity.json"
                ),
            ),
            (
                "rawr_resampling",
                include_str!("../../../control/rawr_resampling/benchmark_rawr_resampling.json"),
            ),
            (
                "anderson_localization",
                include_str!(
                    "../../../control/anderson_localization/benchmark_anderson_localization.json"
                ),
            ),
            (
                "quasiperiodic",
                include_str!("../../../control/quasiperiodic/benchmark_quasiperiodic.json"),
            ),
            (
                "bistable",
                include_str!("../../../control/bistable_switching/benchmark_bistable.json"),
            ),
            (
                "multisignal",
                include_str!("../../../control/multisignal_qs/benchmark_multisignal.json"),
            ),
            (
                "spin_transport",
                include_str!("../../../control/spin_transport/benchmark_spin_transport.json"),
            ),
            (
                "resampling_convergence",
                include_str!(
                    "../../../control/resampling_convergence/benchmark_resampling_convergence.json"
                ),
            ),
            (
                "drift_selection",
                include_str!("../../../control/drift_selection/benchmark_drift_selection.json"),
            ),
            (
                "uncertainty_bridge",
                include_str!(
                    "../../../control/uncertainty_bridge/benchmark_uncertainty_bridge.json"
                ),
            ),
            (
                "rare_biosphere",
                include_str!("../../../control/rare_biosphere/benchmark_rare_biosphere.json"),
            ),
            (
                "quasispecies",
                include_str!("../../../control/quasispecies_threshold/benchmark_quasispecies.json"),
            ),
            (
                "band_edge",
                include_str!("../../../control/band_edge/benchmark_band_edge.json"),
            ),
            (
                "jackknife",
                include_str!("../../../control/jackknife_estimation/benchmark_jackknife.json"),
            ),
            (
                "freeze_out",
                include_str!("../../../control/freeze_out_inverse/benchmark_freeze_out.json"),
            ),
            (
                "spectral_recon",
                include_str!("../../../control/spectral_recon/benchmark_spectral_recon.json"),
            ),
            (
                "et0_anderson",
                include_str!(
                    "../../../control/et0_anderson_propagation/benchmark_et0_anderson.json"
                ),
            ),
            (
                "notill_sampling",
                include_str!("../../../control/notill_sampling/benchmark_notill_sampling.json"),
            ),
            (
                "aggregate_stability",
                include_str!(
                    "../../../control/aggregate_stability/benchmark_aggregate_stability.json"
                ),
            ),
            (
                "precision_drift",
                include_str!("../../../control/precision_drift/benchmark_precision_drift.json"),
            ),
            (
                "size_convergence",
                include_str!("../../../control/size_convergence/benchmark_size_convergence.json"),
            ),
            (
                "vendor_parity",
                include_str!("../../../control/vendor_parity/benchmark_vendor_parity.json"),
            ),
            (
                "npu_anderson",
                include_str!("../../../control/npu_anderson/benchmark_npu_anderson.json"),
            ),
            (
                "et0_methods",
                include_str!("../../../control/et0_methods/benchmark_et0_methods.json"),
            ),
        ];

        assert_eq!(
            benchmarks.len(),
            EXPECTED_BENCHMARKS,
            "benchmark count mismatch: got {}, expected {EXPECTED_BENCHMARKS}",
            benchmarks.len()
        );

        for (name, json_str) in benchmarks {
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(json_str);
            assert!(
                parsed.is_ok(),
                "benchmark '{name}' failed to parse: {}",
                parsed.unwrap_err()
            );
        }
    }
}
