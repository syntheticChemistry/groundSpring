// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! JSON-RPC parameter extraction helpers for dispatch methods.
//!
//! Typed extractors that convert `serde_json::Value` parameters into
//! Rust types, returning [`DispatchError`] on missing or malformed inputs.

use serde_json::Value;

use crate::error::DispatchError;

/// Extract a required `Vec<f64>` from a JSON array parameter.
pub(super) fn extract_f64_array(params: &Value, key: &str) -> Result<Vec<f64>, DispatchError> {
    params
        .get(key)
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(Value::as_f64).collect::<Vec<_>>())
        .ok_or_else(|| DispatchError::MissingParam(key.into()))
}

/// Extract a required `Vec<u64>` from a JSON array parameter.
pub(super) fn extract_u64_array(params: &Value, key: &str) -> Result<Vec<u64>, DispatchError> {
    params
        .get(key)
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(Value::as_u64).collect::<Vec<_>>())
        .ok_or_else(|| DispatchError::MissingParam(key.into()))
}

/// Extract an optional `f64` parameter with a default.
pub(super) fn extract_f64(params: &Value, key: &str, default: f64) -> f64 {
    params.get(key).and_then(Value::as_f64).unwrap_or(default)
}

/// Extract a required `f64` parameter.
pub(super) fn require_f64(params: &Value, key: &str) -> Result<f64, DispatchError> {
    params
        .get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| DispatchError::MissingParam(key.into()))
}

/// Extract an optional `u64` parameter with a default.
pub(super) fn extract_u64(params: &Value, key: &str, default: u64) -> u64 {
    params.get(key).and_then(Value::as_u64).unwrap_or(default)
}

/// Extract an optional `usize` parameter with a `u64` default.
///
/// Returns [`DispatchError::InvalidParam`] if the value exceeds platform `usize`.
pub(super) fn extract_usize(
    params: &Value,
    key: &str,
    default: u64,
) -> Result<usize, DispatchError> {
    let v = extract_u64(params, key, default);
    usize::try_from(v)
        .map_err(|_| DispatchError::InvalidParam(format!("{key} too large for usize")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_f64_array_ok() {
        let v = serde_json::json!({"data": [1.0, 2.0, 3.0]});
        let arr = extract_f64_array(&v, "data").unwrap();
        assert_eq!(arr, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn extract_f64_array_missing() {
        let v = serde_json::json!({});
        assert!(extract_f64_array(&v, "data").is_err());
    }

    #[test]
    fn extract_f64_uses_default() {
        let v = serde_json::json!({});
        assert_eq!(extract_f64(&v, "x", 42.0), 42.0);
    }

    #[test]
    fn extract_f64_overrides_default() {
        let v = serde_json::json!({"x": 7.5});
        assert_eq!(extract_f64(&v, "x", 42.0), 7.5);
    }

    #[test]
    fn require_f64_missing() {
        let v = serde_json::json!({});
        assert!(require_f64(&v, "x").is_err());
    }

    #[test]
    fn require_f64_present() {
        let v = serde_json::json!({"x": 3.14});
        assert!((require_f64(&v, "x").unwrap() - 3.14).abs() < 1e-10);
    }

    #[test]
    fn extract_usize_default() {
        let v = serde_json::json!({});
        assert_eq!(extract_usize(&v, "n", 100).unwrap(), 100);
    }
}
