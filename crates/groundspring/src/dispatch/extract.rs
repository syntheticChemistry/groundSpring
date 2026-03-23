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
