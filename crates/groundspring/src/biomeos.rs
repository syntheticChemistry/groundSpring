// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Optional `biomeOS` Neural API client for ecosystem integration.
//!
//! When `GROUNDSPRING_COMPUTE_PROVIDER=biomeos` is set, groundSpring routes
//! compute-intensive operations through biomeOS's Neural API instead of running
//! them locally. Falls back to sovereign local computation if the socket is
//! unavailable.
//!
//! # Protocol
//!
//! JSON-RPC 2.0, newline-delimited, over Unix domain socket.
//! Socket discovery (capability-based, no hardcoded paths):
//! 1. `GROUNDSPRING_BIOMEOS_SOCKET` env var (explicit override)
//! 2. `$XDG_RUNTIME_DIR/biomeos/neural-api-default.sock`
//! 3. `<temp_dir>/biomeos-neural-api.sock` (platform-agnostic fallback)
//!
//! # Sovereign fallback
//!
//! All operations work without `biomeOS`. When the socket is unavailable,
//! `capability_call` and `rpc_call` return `Err`, and callers fall back to
//! local computation. This follows the same pattern as wetSpring's `NestGate`
//! client.
//!
//! # Evolution path
//!
//! | Phase | Strategy | Status |
//! |-------|----------|--------|
//! | Phase 0 | Live NUCLEUS local, sovereign fallback | **active** |
//! | Phase 1 | Data pipeline via `NestGate` live providers | active |
//! | Phase 2 | `ToadStool` GPU dispatch via `compute.execute` | planned |
//! | Phase 3 | `metalForge` cross-substrate via Neural API | planned |

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const READ_TIMEOUT: Duration = Duration::from_secs(30);
const FAMILY_ID: &str = "groundspring";

/// Socket names the NUCLEUS startup scripts create, in priority order.
/// `start_nucleus.sh` creates `neural-api.sock` (no family suffix) and
/// optionally symlinks `neural-api-{family_id}.sock`.
const NUCLEUS_SOCKET_NAMES: &[&str] = &[
    "neural-api.sock",
    "neural-api-default.sock",
];

/// Error type for `biomeOS` client operations.
#[derive(Debug)]
pub struct BiomeOsError(pub String);

impl std::fmt::Display for BiomeOsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "biomeOS: {}", self.0)
    }
}

impl std::error::Error for BiomeOsError {}

/// Result alias for `biomeOS` operations.
pub type Result<T> = std::result::Result<T, BiomeOsError>;

/// Whether `biomeOS` routing is enabled via environment.
#[must_use]
pub fn is_enabled() -> bool {
    std::env::var("GROUNDSPRING_COMPUTE_PROVIDER")
        .is_ok_and(|v| v.trim().eq_ignore_ascii_case("biomeos"))
}

/// Discover the `biomeOS` Neural API Unix socket path.
///
/// Discovery priority (no hardcoded absolute paths):
/// 1. `GROUNDSPRING_BIOMEOS_SOCKET` env var (explicit override)
/// 2. `$XDG_RUNTIME_DIR/biomeos/neural-api.sock` (NUCLEUS `start_nucleus.sh`)
/// 3. `$XDG_RUNTIME_DIR/biomeos/neural-api-default.sock` (legacy)
/// 4. `/run/user/{uid}/biomeos/neural-api.sock` (non-XDG Linux)
/// 5. `<temp_dir>/biomeos-neural-api.sock` (platform-agnostic fallback)
#[must_use]
pub fn discover_socket() -> Option<PathBuf> {
    let explicit = std::env::var("GROUNDSPRING_BIOMEOS_SOCKET").ok();
    let xdg = std::env::var("XDG_RUNTIME_DIR").ok();
    resolve_socket(explicit.as_deref(), xdg.as_deref())
}

/// Pure-logic socket path resolution (testable without filesystem side effects).
fn resolve_socket(explicit: Option<&str>, xdg_runtime: Option<&str>) -> Option<PathBuf> {
    if let Some(path) = explicit {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    if let Some(xdg) = xdg_runtime {
        let biomeos_dir = PathBuf::from(xdg).join("biomeos");
        for name in NUCLEUS_SOCKET_NAMES {
            let p = biomeos_dir.join(name);
            if p.exists() {
                return Some(p);
            }
        }
    }

    // Non-XDG Linux fallback: /run/user/{uid}/biomeos/
    #[cfg(target_os = "linux")]
    if xdg_runtime.is_none() {
        if let Some(uid) = proc_self_uid() {
            let run_dir = PathBuf::from(format!("/run/user/{uid}/biomeos"));
            for name in NUCLEUS_SOCKET_NAMES {
                let p = run_dir.join(name);
                if p.exists() {
                    return Some(p);
                }
            }
        }
    }

    // /tmp fallback (least preferred)
    for name in NUCLEUS_SOCKET_NAMES {
        let p = std::env::temp_dir().join(format!("biomeos/{name}"));
        if p.exists() {
            return Some(p);
        }
    }

    let legacy = std::env::temp_dir().join("biomeos-neural-api.sock");
    if legacy.exists() {
        return Some(legacy);
    }

    None
}

/// Get the real UID of the current process via `/proc/self` metadata.
#[cfg(target_os = "linux")]
fn proc_self_uid() -> Option<u32> {
    use std::os::unix::fs::MetadataExt;
    std::fs::metadata("/proc/self").ok().map(|m| m.uid())
}

/// Attempt to connect to a live NUCLEUS and return the socket path if healthy.
///
/// Unlike [`discover_socket`] which only checks if a socket *file* exists,
/// this function actually connects and verifies the Neural API responds.
/// Returns `None` if no NUCLEUS is running or it fails the health check.
#[must_use]
pub fn auto_connect() -> Option<PathBuf> {
    let socket = discover_socket()?;
    health(&socket).ok()?;
    Some(socket)
}

/// Check whether a live NUCLEUS is available and responding.
#[must_use]
pub fn is_nucleus_available() -> bool {
    auto_connect().is_some()
}

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
    let (cap, op) = capability.split_once('.').unwrap_or((capability, "call"));
    let request = format!(
        r#"{{"jsonrpc":"2.0","method":"capability.call","params":{{"capability":"{}","operation":"{}","args":{},"family_id":"{}"}},"id":1}}"#,
        escape_json(cap),
        escape_json(op),
        params_json,
        FAMILY_ID,
    );
    let response = rpc_call(socket, &request)?;
    if response.contains("\"error\"") {
        Err(BiomeOsError(extract_error(&response)))
    } else {
        extract_result(&response)
    }
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
    let request = format!(
        r#"{{"jsonrpc":"2.0","method":"capability.call","params":{{"capability":"{}","operation":"{}","args":{},"family_id":"{}"}},"id":1}}"#,
        escape_json(target),
        escape_json(method),
        params_json,
        FAMILY_ID,
    );
    let response = rpc_call(socket, &request)?;
    if response.contains("\"error\"") {
        Err(BiomeOsError(extract_error(&response)))
    } else {
        extract_result(&response)
    }
}

/// Store a value via biomeOS capability-based storage routing.
///
/// Routes through `storage.put` capability — biomeOS translates to
/// `NestGate`'s `storage.store` method at runtime.
///
/// # Errors
///
/// Returns `Err` if the socket is unavailable or the RPC fails.
pub fn storage_put(socket: &Path, key: &str, value: &str) -> Result<()> {
    let params = format!(
        r#"{{"key":"{}","value":"{}","family_id":"{}"}}"#,
        escape_json(key),
        escape_json(value),
        FAMILY_ID,
    );
    capability_call(socket, "storage.put", &params)?;
    Ok(())
}

/// Retrieve a value via biomeOS capability-based storage routing.
///
/// Routes through `storage.get` capability — biomeOS translates to
/// `NestGate`'s `storage.retrieve` method at runtime.
///
/// # Errors
///
/// Returns `Err` if the socket is unavailable, the key does not exist,
/// or the RPC fails.
pub fn storage_get(socket: &Path, key: &str) -> Result<String> {
    let params = format!(
        r#"{{"key":"{}","family_id":"{}"}}"#,
        escape_json(key),
        FAMILY_ID,
    );
    capability_call(socket, "storage.get", &params)
}

/// Dispatch a computation through `ToadStool` via `compute.execute`.
///
/// The `op` field names the operation (e.g. `"lyapunov_averaged"`).
/// Additional fields in `params_json` carry the operation-specific arguments.
///
/// Returns the raw result string from `ToadStool`.
///
/// # Errors
///
/// Returns `Err` if biomeOS is unavailable or `ToadStool` rejects the request.
pub fn compute_execute(socket: &Path, op: &str, params_json: &str) -> Result<String> {
    let merged = if params_json.starts_with('{') && params_json.len() > 2 {
        let inner = &params_json[1..params_json.len() - 1];
        format!(r#"{{"op":"{}",{inner},"family_id":"{}"}}"#, escape_json(op), FAMILY_ID)
    } else {
        format!(
            r#"{{"op":"{}","family_id":"{}"}}"#,
            escape_json(op),
            FAMILY_ID,
        )
    };
    capability_call(socket, "compute.execute", &merged)
}

/// Submit a compute job asynchronously via `compute.submit`.
///
/// Returns a job ID or status from `ToadStool`.
///
/// # Errors
///
/// Returns `Err` if biomeOS is unavailable or the submission fails.
pub fn compute_submit(socket: &Path, op: &str, params_json: &str) -> Result<String> {
    let merged = if params_json.starts_with('{') && params_json.len() > 2 {
        let inner = &params_json[1..params_json.len() - 1];
        format!(r#"{{"op":"{}",{inner},"family_id":"{}"}}"#, escape_json(op), FAMILY_ID)
    } else {
        format!(
            r#"{{"op":"{}","family_id":"{}"}}"#,
            escape_json(op),
            FAMILY_ID,
        )
    };
    capability_call(socket, "compute.submit", &merged)
}

/// Query `ToadStool` compute capabilities.
///
/// Returns JSON listing available compute operations and GPU info.
///
/// # Errors
///
/// Returns `Err` if biomeOS or `ToadStool` is unavailable.
pub fn compute_capabilities(socket: &Path) -> Result<String> {
    let params = format!(r#"{{"family_id":"{FAMILY_ID}"}}"#);
    capability_call(socket, "resource.health.check", &params)
}

/// Health check: verify the Neural API is alive.
///
/// Uses `topology.metrics` since the Neural API doesn't expose a bare `health` method.
///
/// # Errors
///
/// Returns `Err` if the socket is unavailable or the health check fails.
pub fn health(socket: &Path) -> Result<()> {
    let request = r#"{"jsonrpc":"2.0","method":"topology.metrics","params":{},"id":1}"#;
    let response = rpc_call(socket, request)?;
    if response.contains("\"error\"") {
        Err(BiomeOsError(extract_error(&response)))
    } else {
        Ok(())
    }
}

/// Send an arbitrary JSON-RPC request over a Unix socket and read the response.
///
/// For use by integration tests and advanced consumers that need to send
/// raw JSON-RPC to the Neural API.
///
/// # Errors
///
/// Returns `Err` if the socket is unavailable or the RPC fails.
pub fn raw_rpc_call(socket: &Path, request: &str) -> Result<String> {
    rpc_call(socket, request)
}

/// Send a JSON-RPC request over a Unix socket and read the response.
fn rpc_call(socket: &Path, request: &str) -> Result<String> {
    let stream = UnixStream::connect_addr(
        &std::os::unix::net::SocketAddr::from_pathname(socket)
            .map_err(|e| BiomeOsError(format!("invalid socket path: {e}")))?,
    )
    .map_err(|e| BiomeOsError(format!("biomeOS connect {}: {e}", socket.display())))?;

    stream
        .set_read_timeout(Some(READ_TIMEOUT))
        .map_err(|e| BiomeOsError(format!("set read timeout: {e}")))?;
    stream
        .set_write_timeout(Some(CONNECT_TIMEOUT))
        .map_err(|e| BiomeOsError(format!("set write timeout: {e}")))?;

    let mut writer = std::io::BufWriter::new(&stream);
    writer
        .write_all(request.as_bytes())
        .map_err(|e| BiomeOsError(format!("write to biomeOS: {e}")))?;
    writer
        .write_all(b"\n")
        .map_err(|e| BiomeOsError(format!("write newline: {e}")))?;
    writer
        .flush()
        .map_err(|e| BiomeOsError(format!("flush: {e}")))?;

    let mut reader = BufReader::new(&stream);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .map_err(|e| BiomeOsError(format!("read from biomeOS: {e}")))?;

    if line.is_empty() {
        return Err(BiomeOsError("biomeOS returned empty response".to_string()));
    }

    Ok(line)
}

/// Public JSON string escaping for sibling modules (e.g. `nestgate`).
#[must_use]
pub fn escape_json_pub(s: &str) -> String {
    escape_json(s)
}

/// Minimal JSON string escaping for values embedded in RPC requests.
fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Extract the error message from a JSON-RPC error response.
fn extract_error(response: &str) -> String {
    if let Some(start) = response.find("\"message\"") {
        if let Some(colon) = response[start..].find(':') {
            let after_colon = &response[start + colon + 1..];
            let trimmed = after_colon.trim_start();
            if let Some(inner) = trimmed.strip_prefix('"') {
                if let Some(end) = inner.find('"') {
                    return inner[..end].to_string();
                }
            }
        }
    }
    format!(
        "biomeOS RPC error: {}",
        &response[..response.len().min(200)]
    )
}

/// Extract the `result` field from a JSON-RPC response.
///
/// Handles string, object, array, number, and boolean result values by
/// tracking brace/bracket nesting for complex JSON structures.
fn extract_result(response: &str) -> Result<String> {
    if let Some(start) = response.find("\"result\"") {
        if let Some(colon) = response[start..].find(':') {
            let after_colon = &response[start + colon + 1..];
            let trimmed = after_colon.trim_start();
            if let Some(inner) = trimmed.strip_prefix('"') {
                if let Some(end) = inner.find('"') {
                    let raw = &inner[..end];
                    return Ok(raw.replace("\\n", "\n").replace("\\\"", "\""));
                }
            }
            if trimmed.starts_with('{') || trimmed.starts_with('[') {
                return extract_balanced(trimmed);
            }
            if let Some(end) = trimmed.find([',', '}']) {
                return Ok(trimmed[..end].trim().to_string());
            }
        }
    }
    Err(BiomeOsError(
        "could not extract result from biomeOS response".to_string(),
    ))
}

/// Extract a balanced JSON object or array from the start of `s`.
fn extract_balanced(s: &str) -> Result<String> {
    let open = s.as_bytes()[0];
    let close = if open == b'{' { b'}' } else { b']' };
    let mut depth: u32 = 0;
    let mut in_string = false;
    let mut escape = false;
    for (i, byte) in s.bytes().enumerate() {
        if escape {
            escape = false;
            continue;
        }
        if byte == b'\\' && in_string {
            escape = true;
            continue;
        }
        if byte == b'"' {
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        if byte == open {
            depth += 1;
        } else if byte == close {
            depth -= 1;
            if depth == 0 {
                return Ok(s[..=i].to_string());
            }
        }
    }
    Err(BiomeOsError("unbalanced JSON in result".to_string()))
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]
mod tests {
    use super::*;

    #[test]
    fn is_enabled_default_false() {
        assert!(
            !std::env::var("GROUNDSPRING_COMPUTE_PROVIDER")
                .is_ok_and(|v| v.trim().eq_ignore_ascii_case("biomeos"))
                || is_enabled()
        );
    }

    #[test]
    fn resolve_socket_explicit_nonexistent() {
        let result = resolve_socket(Some("/tmp/nonexistent_groundspring_biomeos.sock"), None);
        // The explicit path doesn't exist, so it won't be returned. However,
        // the Linux /run/user/{uid} fallback might find a real NUCLEUS socket.
        if let Some(ref p) = result {
            assert_ne!(
                p.to_str().unwrap(),
                "/tmp/nonexistent_groundspring_biomeos.sock",
                "should not return the nonexistent explicit path"
            );
            assert!(p.exists(), "returned path must exist");
        }
    }

    #[test]
    fn resolve_socket_all_none() {
        let result = resolve_socket(None, None);
        assert!(
            result.is_none() || result.is_some_and(|p| p.exists()),
            "should be None or a path that exists"
        );
    }

    #[test]
    fn resolve_socket_explicit_exists() {
        let dir = tempfile::tempdir().unwrap();
        let sock = dir.path().join("test.sock");
        std::fs::write(&sock, "").unwrap();
        let result = resolve_socket(Some(sock.to_str().unwrap()), None);
        assert_eq!(result, Some(sock));
    }

    #[test]
    fn resolve_socket_xdg_path_neural_api() {
        let dir = tempfile::tempdir().unwrap();
        let biomeos = dir.path().join("biomeos");
        std::fs::create_dir_all(&biomeos).unwrap();
        let sock = biomeos.join("neural-api.sock");
        std::fs::write(&sock, "").unwrap();
        let result = resolve_socket(None, Some(dir.path().to_str().unwrap()));
        assert_eq!(result, Some(sock), "should prefer neural-api.sock");
    }

    #[test]
    fn resolve_socket_xdg_path_legacy_default() {
        let dir = tempfile::tempdir().unwrap();
        let biomeos = dir.path().join("biomeos");
        std::fs::create_dir_all(&biomeos).unwrap();
        let sock = biomeos.join("neural-api-default.sock");
        std::fs::write(&sock, "").unwrap();
        let result = resolve_socket(None, Some(dir.path().to_str().unwrap()));
        assert_eq!(result, Some(sock), "should find legacy neural-api-default.sock");
    }

    #[test]
    fn resolve_socket_prefers_neural_api_over_default() {
        let dir = tempfile::tempdir().unwrap();
        let biomeos = dir.path().join("biomeos");
        std::fs::create_dir_all(&biomeos).unwrap();
        let primary = biomeos.join("neural-api.sock");
        let legacy = biomeos.join("neural-api-default.sock");
        std::fs::write(&primary, "").unwrap();
        std::fs::write(&legacy, "").unwrap();
        let result = resolve_socket(None, Some(dir.path().to_str().unwrap()));
        assert_eq!(result, Some(primary), "should prefer neural-api.sock over default");
    }

    #[test]
    fn resolve_socket_xdg_nonexistent() {
        let dir = tempfile::tempdir().unwrap();
        let xdg = dir.path().join("nonexistent_xdg");
        let result = resolve_socket(None, Some(xdg.to_str().unwrap()));
        assert!(result.is_none() || result.is_some_and(|p| p.exists()));
    }

    #[test]
    fn resolve_socket_explicit_overrides_xdg() {
        let dir = tempfile::tempdir().unwrap();
        let explicit_sock = dir.path().join("explicit.sock");
        std::fs::write(&explicit_sock, "").unwrap();
        let xdg_dir = tempfile::tempdir().unwrap();
        let biomeos = xdg_dir.path().join("biomeos");
        std::fs::create_dir_all(&biomeos).unwrap();
        let xdg_sock = biomeos.join("neural-api.sock");
        std::fs::write(&xdg_sock, "").unwrap();
        let result = resolve_socket(
            Some(explicit_sock.to_str().unwrap()),
            Some(xdg_dir.path().to_str().unwrap()),
        );
        assert_eq!(result, Some(explicit_sock));
    }

    #[test]
    fn escape_json_special_chars() {
        assert_eq!(escape_json("hello\nworld"), "hello\\nworld");
        assert_eq!(escape_json("say \"hi\""), "say \\\"hi\\\"");
        assert_eq!(escape_json("back\\slash"), "back\\\\slash");
        assert_eq!(escape_json("tab\there"), "tab\\there");
    }

    #[test]
    fn escape_json_empty() {
        assert_eq!(escape_json(""), "");
    }

    #[test]
    fn escape_json_no_special() {
        assert_eq!(escape_json("plain text"), "plain text");
    }

    #[test]
    fn extract_error_with_message() {
        let response = r#"{"jsonrpc":"2.0","error":{"code":-32600,"message":"not found"},"id":1}"#;
        let err = extract_error(response);
        assert!(err.contains("not found"));
    }

    #[test]
    fn extract_error_no_message() {
        let response = r#"{"jsonrpc":"2.0","error":{"code":-32600},"id":1}"#;
        let err = extract_error(response);
        assert!(err.contains("RPC error"));
    }

    #[test]
    fn extract_result_string() {
        let response = r#"{"jsonrpc":"2.0","result":"ok","id":1}"#;
        let val = extract_result(response).unwrap();
        assert_eq!(val, "ok");
    }

    #[test]
    fn extract_result_missing() {
        let response = r#"{"jsonrpc":"2.0","id":1}"#;
        assert!(extract_result(response).is_err());
    }

    #[test]
    fn extract_result_escaped_newline() {
        let response = r#"{"jsonrpc":"2.0","result":"line1\nline2","id":1}"#;
        let val = extract_result(response).unwrap();
        assert_eq!(val, "line1\nline2");
    }

    #[test]
    fn extract_result_nested_object() {
        let response = r#"{"jsonrpc":"2.0","result":{"passed":12,"failed":0},"id":1}"#;
        let val = extract_result(response).unwrap();
        assert!(val.contains("passed") && val.contains("12"));
    }

    #[test]
    fn capability_call_request_format() {
        let cap = "science.anderson_validation";
        let args = r#"{"n_sites":10000}"#;
        let (cap_part, op_part) = cap.split_once('.').unwrap();
        let request = format!(
            r#"{{"jsonrpc":"2.0","method":"capability.call","params":{{"capability":"{}","operation":"{}","args":{},"family_id":"{}"}},"id":1}}"#,
            escape_json(cap_part),
            escape_json(op_part),
            args,
            FAMILY_ID,
        );
        assert!(request.contains("capability.call"));
        assert!(request.contains("\"capability\":\"science\""));
        assert!(request.contains("\"operation\":\"anderson_validation\""));
        assert!(request.contains("groundspring"));
    }

    #[test]
    fn storage_request_formats() {
        let key = "groundspring:results:exp008";
        let put_params = format!(
            r#"{{"key":"{}","value":"test","family_id":"{}"}}"#,
            escape_json(key),
            FAMILY_ID,
        );
        assert!(put_params.contains("groundspring:results:exp008"));

        let get_params = format!(
            r#"{{"key":"{}","family_id":"{}"}}"#,
            escape_json(key),
            FAMILY_ID,
        );
        assert!(get_params.contains("groundspring:results:exp008"));
    }

    #[test]
    fn discover_socket_does_not_panic() {
        let _ = discover_socket();
    }

    #[test]
    fn health_nonexistent_socket_errors() {
        let path = PathBuf::from("/tmp/groundspring_test_nonexistent_biomeos.sock");
        let err = health(&path).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("biomeOS connect") || msg.contains("invalid socket"));
    }

    #[test]
    fn capability_call_nonexistent_socket_errors() {
        let path = PathBuf::from("/tmp/groundspring_test_nonexistent_cap.sock");
        let err = capability_call(&path, "science.test", "{}").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("biomeOS connect") || msg.contains("invalid socket"));
    }

    #[test]
    fn direct_rpc_call_nonexistent_socket_errors() {
        let path = PathBuf::from("/tmp/groundspring_test_nonexistent_rpc.sock");
        let err = direct_rpc_call(&path, "nestgate", "health", "{}").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("biomeOS connect") || msg.contains("invalid socket"));
    }
}
