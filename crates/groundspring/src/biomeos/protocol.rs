// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! JSON-RPC 2.0 serialization and response parsing.

use serde_json::Value;

use super::{BiomeOsError, Result};

/// Build a JSON-RPC 2.0 request envelope.
pub(super) fn build_request(method: &str, params: &Value) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    })
    .to_string()
}

/// Parse a JSON-RPC 2.0 response, extracting the result or error.
pub(super) fn parse_rpc_response(response: &str) -> Result<String> {
    let v: Value = serde_json::from_str(response)
        .map_err(|e| BiomeOsError(format!("invalid JSON-RPC response: {e}")))?;

    if let Some(error) = v.get("error") {
        let msg = error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("unknown RPC error");
        return Err(BiomeOsError(msg.to_string()));
    }

    match v.get("result") {
        Some(Value::String(s)) => Ok(s.to_owned()),
        Some(other) => Ok(other.to_string()),
        None => Err(BiomeOsError("missing result field in response".to_string())),
    }
}

/// Check whether a JSON-RPC response contains an error field.
pub(super) fn response_has_error(response: &str) -> Result<()> {
    let v: Value = serde_json::from_str(response)
        .map_err(|e| BiomeOsError(format!("invalid JSON-RPC response: {e}")))?;

    if let Some(error) = v.get("error") {
        let msg = error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("unknown RPC error");
        return Err(BiomeOsError(msg.to_string()));
    }

    Ok(())
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]
mod tests {
    use super::*;

    #[test]
    fn parse_rpc_response_string_result() {
        let resp = r#"{"jsonrpc":"2.0","result":"ok","id":1}"#;
        assert_eq!(parse_rpc_response(resp).unwrap(), "ok");
    }

    #[test]
    fn parse_rpc_response_object_result() {
        let resp = r#"{"jsonrpc":"2.0","result":{"passed":12,"failed":0},"id":1}"#;
        let val = parse_rpc_response(resp).unwrap();
        assert!(val.contains("passed") && val.contains("12"));
    }

    #[test]
    fn parse_rpc_response_error() {
        let resp = r#"{"jsonrpc":"2.0","error":{"code":-32600,"message":"not found"},"id":1}"#;
        let err = parse_rpc_response(resp).unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn parse_rpc_response_missing_result() {
        let resp = r#"{"jsonrpc":"2.0","id":1}"#;
        assert!(parse_rpc_response(resp).is_err());
    }

    #[test]
    fn parse_rpc_response_numeric_result() {
        let resp = r#"{"jsonrpc":"2.0","result":42,"id":1}"#;
        assert_eq!(parse_rpc_response(resp).unwrap(), "42");
    }

    #[test]
    fn parse_rpc_response_array_result() {
        let resp = r#"{"jsonrpc":"2.0","result":[1,2,3],"id":1}"#;
        let val = parse_rpc_response(resp).unwrap();
        assert!(val.contains("[1,2,3]"));
    }

    #[test]
    fn build_request_is_valid_json() {
        let req = build_request("test.method", &serde_json::json!({"key": "value"}));
        let v: Value = serde_json::from_str(&req).unwrap();
        assert_eq!(v["jsonrpc"], "2.0");
        assert_eq!(v["method"], "test.method");
        assert_eq!(v["params"]["key"], "value");
    }

    #[test]
    fn response_has_error_ok() {
        let resp = r#"{"jsonrpc":"2.0","result":"ok","id":1}"#;
        assert!(response_has_error(resp).is_ok());
    }

    #[test]
    fn response_has_error_with_error() {
        let resp = r#"{"jsonrpc":"2.0","error":{"code":-32600,"message":"bad"},"id":1}"#;
        assert!(response_has_error(resp).is_err());
    }
}
