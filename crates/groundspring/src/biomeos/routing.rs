// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Neural API `capability.call` routing and related direct RPC helpers.

use std::path::Path;

use serde_json::Value;

use super::protocol::{build_request, extract_rpc_result, parse_rpc_response};
use super::transport::rpc_call;
use super::{BiomeOsError, FAMILY_ID, Result};

/// Route a request through biomeOS Neural API `capability.call`.
///
/// The Neural API uses semantic routing: `capability` is the base category
/// (e.g. `"compute"`, `"crypto"`) and `operation` is the specific method
/// (e.g. `"health"`, `"execute"`). The translation registry maps
/// `capability.operation` to the target primal's actual RPC method.
///
/// # Errors
///
/// Returns `Err` if the socket is unavailable or the RPC fails.
pub fn capability_call(socket: &Path, capability: &str, params_json: &str) -> Result<String> {
    let args: Value = serde_json::from_str(params_json)
        .map_err(|e| BiomeOsError::Serialization(format!("invalid params JSON: {e}")))?;
    capability_call_value(socket, capability, &args)
}

/// Internal capability call accepting a pre-built [`Value`] to avoid
/// redundant serialize→deserialize round-trips from internal callers.
pub fn capability_call_value(socket: &Path, capability: &str, args: &Value) -> Result<String> {
    let (cap, op) = capability.split_once('.').unwrap_or((capability, "call"));
    let params = serde_json::json!({
        "capability": cap,
        "operation": op,
        "args": args,
        "family_id": FAMILY_ID,
    });
    let request = build_request("capability.call", &params);
    let response = rpc_call(socket, &request)?;
    parse_rpc_response(&response)
}

/// Route a request through biomeOS and return the parsed JSON result.
///
/// Like [`capability_call`] but returns the structured `serde_json::Value`
/// directly instead of a `String`, avoiding a redundant serialization
/// round-trip for callers that consume the result as JSON.
///
/// # Errors
///
/// Returns `Err` if the socket is unavailable, the RPC fails, or the
/// response contains a JSON-RPC error.
pub fn capability_call_typed(socket: &Path, capability: &str, params_json: &str) -> Result<Value> {
    let args: Value = serde_json::from_str(params_json)
        .map_err(|e| BiomeOsError::Serialization(format!("invalid params JSON: {e}")))?;
    let (cap, op) = capability.split_once('.').unwrap_or((capability, "call"));
    let params = serde_json::json!({
        "capability": cap,
        "operation": op,
        "args": args,
        "family_id": FAMILY_ID,
    });
    let request = build_request("capability.call", &params);
    let response = rpc_call(socket, &request)?;
    extract_rpc_result(&response)
}

/// Direct JSON-RPC call targeting a specific biomeOS primal by name.
///
/// **Prefer [`capability_call`] for normal use** — it routes by capability,
/// letting biomeOS discover which primal provides the service at runtime.
/// Use `direct_rpc_call` only when you must bypass capability discovery and
/// target a known primal directly (e.g. hardware-specific operations).
///
/// # Errors
///
/// Returns `Err` if the socket is unavailable or the RPC fails.
pub fn direct_rpc_call(
    socket: &Path,
    target: &str,
    method: &str,
    params_json: &str,
) -> Result<String> {
    let capability = format!("{target}.{method}");
    capability_call(socket, &capability, params_json)
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]
mod tests {
    use serde_json::json;

    use tempfile::tempdir;

    use super::{capability_call, capability_call_value};
    use crate::biomeos::BiomeOsError;

    #[test]
    fn capability_call_value_nonexistent_socket_errors() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("missing.sock");
        let err = capability_call_value(&path, "compute.call", &json!({})).unwrap_err();
        assert!(matches!(err, BiomeOsError::Transport(_)));
    }

    #[test]
    fn capability_call_invalid_params_json_errors() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("missing.sock");
        let err = capability_call(&path, "x.y", "not json").unwrap_err();
        assert!(matches!(err, BiomeOsError::Serialization(_)));
    }
}
