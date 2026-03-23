// SPDX-License-Identifier: AGPL-3.0-or-later
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

/// Structured outcome of a JSON-RPC dispatch.
///
/// Distinguishes protocol-level errors (invalid request, method not found,
/// timeout) from application-level errors (the target primal processed the
/// request but returned a domain error). Callers can retry protocol errors
/// but should propagate application errors.
///
/// Pattern source: biomeOS v2.46 `DispatchOutcome`.
#[derive(Debug, Clone)]
pub(super) enum DispatchOutcome {
    /// The RPC succeeded and returned a result.
    Ok(String),
    /// Protocol-level error (JSON-RPC spec codes -32700 to -32600).
    ProtocolError { code: i64, message: String },
    /// Application-level error (code >= -32000 or non-standard).
    ApplicationError {
        #[allow(dead_code)]
        // Classified in `classify_rpc_error`; non-test handlers only surface `message`.
        code: i64,
        message: String,
    },
}

impl DispatchOutcome {
    /// Whether this is a `-32601 Method not found` protocol error.
    #[must_use]
    pub(super) const fn is_method_not_found(&self) -> bool {
        matches!(self, Self::ProtocolError { code: -32601, .. })
    }
}

/// Standard JSON-RPC 2.0 error code range for protocol errors.
const JSONRPC_PROTOCOL_ERROR_MIN: i64 = -32700;
const JSONRPC_PROTOCOL_ERROR_MAX: i64 = -32600;

/// Extract structured error from a JSON-RPC 2.0 response.
///
/// Returns `Some((code, message))` if the response contains an `error` field,
/// `None` if no error is present. Handles both standard JSON-RPC error objects
/// (`{"code": -32600, "message": "..."}`) and bare string errors.
///
/// Pattern source: wetSpring V123 / healthSpring V30 centralized RPC error extraction.
pub(super) fn extract_rpc_error(v: &Value) -> Option<(i64, String)> {
    let error = v.get("error")?;
    let code = error.get("code").and_then(Value::as_i64).unwrap_or(-32000);
    let message = error
        .get("message")
        .and_then(Value::as_str)
        .map(String::from)
        .or_else(|| error.as_str().map(String::from))
        .unwrap_or_else(|| "unknown RPC error".to_string());
    Some((code, message))
}

/// Classify an RPC error code as protocol or application level.
fn classify_rpc_error(code: i64, message: String) -> DispatchOutcome {
    if (JSONRPC_PROTOCOL_ERROR_MIN..=JSONRPC_PROTOCOL_ERROR_MAX).contains(&code) {
        DispatchOutcome::ProtocolError { code, message }
    } else {
        DispatchOutcome::ApplicationError { code, message }
    }
}

/// Parse a JSON-RPC 2.0 response into a [`DispatchOutcome`].
pub(super) fn parse_rpc_dispatch(
    response: &str,
) -> std::result::Result<DispatchOutcome, BiomeOsError> {
    let v: Value = serde_json::from_str(response)
        .map_err(|e| BiomeOsError::Protocol(format!("invalid JSON-RPC response: {e}")))?;

    if let Some((code, message)) = extract_rpc_error(&v) {
        return Ok(classify_rpc_error(code, message));
    }

    match v.get("result") {
        Some(Value::String(s)) => Ok(DispatchOutcome::Ok(s.to_owned())),
        Some(other) => Ok(DispatchOutcome::Ok(other.to_string())),
        None => Err(BiomeOsError::Protocol(
            "missing result field in response".to_string(),
        )),
    }
}

/// Parse a JSON-RPC 2.0 response, extracting the result or error.
pub(super) fn parse_rpc_response(response: &str) -> Result<String> {
    match parse_rpc_dispatch(response)? {
        DispatchOutcome::Ok(result) => Ok(result),
        DispatchOutcome::ProtocolError { message, .. }
        | DispatchOutcome::ApplicationError { message, .. } => Err(BiomeOsError::Protocol(message)),
    }
}

/// Check whether a JSON-RPC response contains an error field.
pub(super) fn response_has_error(response: &str) -> Result<()> {
    match parse_rpc_dispatch(response)? {
        DispatchOutcome::Ok(_) => Ok(()),
        DispatchOutcome::ProtocolError { message, .. }
        | DispatchOutcome::ApplicationError { message, .. } => Err(BiomeOsError::Protocol(message)),
    }
}

/// Extract the result value from a JSON-RPC 2.0 response, or the error.
///
/// Convenience wrapper that parses the raw JSON and returns the `"result"`
/// field as a [`Value`], or converts the `"error"` field into a typed
/// [`BiomeOsError`].
///
/// Absorbed from ludoSpring V23 / healthSpring V30 `extract_rpc_result()`.
pub(super) fn extract_rpc_result(response: &str) -> Result<Value> {
    let v: Value = serde_json::from_str(response)
        .map_err(|e| BiomeOsError::Protocol(format!("invalid JSON-RPC response: {e}")))?;

    if let Some((code, message)) = extract_rpc_error(&v) {
        return if (JSONRPC_PROTOCOL_ERROR_MIN..=JSONRPC_PROTOCOL_ERROR_MAX).contains(&code) {
            Err(BiomeOsError::Protocol(message))
        } else {
            Err(BiomeOsError::Other(message))
        };
    }

    v.get("result")
        .cloned()
        .ok_or_else(|| BiomeOsError::Protocol("missing result field in response".to_string()))
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

    #[test]
    fn extract_rpc_error_standard() {
        let v: Value =
            serde_json::from_str(r#"{"error":{"code":-32601,"message":"method not found"}}"#)
                .unwrap();
        let (code, msg) = extract_rpc_error(&v).unwrap();
        assert_eq!(code, -32601);
        assert_eq!(msg, "method not found");
    }

    #[test]
    fn extract_rpc_error_bare_string() {
        let v: Value = serde_json::from_str(r#"{"error":"something went wrong"}"#).unwrap();
        let (code, msg) = extract_rpc_error(&v).unwrap();
        assert_eq!(code, -32000);
        assert_eq!(msg, "something went wrong");
    }

    #[test]
    fn extract_rpc_error_none_when_no_error() {
        let v: Value = serde_json::from_str(r#"{"result":"ok"}"#).unwrap();
        assert!(extract_rpc_error(&v).is_none());
    }

    #[test]
    fn extract_rpc_error_is_method_not_found() {
        let v: Value =
            serde_json::from_str(r#"{"error":{"code":-32601,"message":"method not found"}}"#)
                .unwrap();
        let (code, _) = extract_rpc_error(&v).unwrap();
        assert_eq!(code, -32601);
    }

    #[test]
    fn dispatch_outcome_protocol_error() {
        let resp =
            r#"{"jsonrpc":"2.0","error":{"code":-32601,"message":"method not found"},"id":1}"#;
        let outcome = parse_rpc_dispatch(resp).unwrap();
        assert!(matches!(
            outcome,
            DispatchOutcome::ProtocolError { code: -32601, .. }
        ));
        assert!(outcome.is_method_not_found());
    }

    #[test]
    fn dispatch_outcome_application_error() {
        let resp = r#"{"jsonrpc":"2.0","error":{"code":-32000,"message":"compute failed"},"id":1}"#;
        let outcome = parse_rpc_dispatch(resp).unwrap();
        assert!(matches!(
            outcome,
            DispatchOutcome::ApplicationError { code: -32000, .. }
        ));
        assert!(!outcome.is_method_not_found());
    }

    #[test]
    fn dispatch_outcome_success() {
        let resp = r#"{"jsonrpc":"2.0","result":"done","id":1}"#;
        let outcome = parse_rpc_dispatch(resp).unwrap();
        assert!(matches!(outcome, DispatchOutcome::Ok(ref s) if s == "done"));
    }

    #[test]
    fn dispatch_outcome_classify_boundary() {
        let outcome = classify_rpc_error(-32700, "parse error".into());
        assert!(matches!(outcome, DispatchOutcome::ProtocolError { .. }));

        let outcome = classify_rpc_error(-32600, "invalid request".into());
        assert!(matches!(outcome, DispatchOutcome::ProtocolError { .. }));

        let outcome = classify_rpc_error(-32599, "custom error".into());
        assert!(matches!(outcome, DispatchOutcome::ApplicationError { .. }));
    }
}
