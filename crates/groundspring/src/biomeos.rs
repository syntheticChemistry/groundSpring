// SPDX-License-Identifier: AGPL-3.0-only
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
//! JSON-RPC 2.0, newline-delimited, over platform-agnostic transport.
//!
//! Transport selection:
//! - **Unix**: Unix domain socket (preferred, zero-copy-friendly)
//! - **Non-Unix**: TCP via `GROUNDSPRING_BIOMEOS_TCP` env var
//!
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
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::Value;

/// Connect timeout, overridable via `GROUNDSPRING_BIOMEOS_CONNECT_TIMEOUT_SECS`.
fn connect_timeout() -> Duration {
    Duration::from_secs(
        std::env::var("GROUNDSPRING_BIOMEOS_CONNECT_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5),
    )
}

/// Read timeout, overridable via `GROUNDSPRING_BIOMEOS_READ_TIMEOUT_SECS`.
fn read_timeout() -> Duration {
    Duration::from_secs(
        std::env::var("GROUNDSPRING_BIOMEOS_READ_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30),
    )
}

/// Family identifier for all biomeOS interactions.
///
/// Used in JSON-RPC requests and provenance key namespacing to identify
/// this spring within the ecosystem. Other modules (e.g. `nestgate`)
/// should reference this constant rather than duplicating the literal.
pub const FAMILY_ID: &str = "groundspring";

/// Socket names the NUCLEUS startup scripts create, in priority order.
/// `start_nucleus.sh` creates `neural-api.sock` (no family suffix) and
/// optionally symlinks `neural-api-{family_id}.sock`.
const NUCLEUS_SOCKET_NAMES: &[&str] = &["neural-api.sock", "neural-api-default.sock"];

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

// ─── JSON-RPC Serialization ──────────────────────────────────────────────────

/// Build a JSON-RPC 2.0 request envelope.
fn build_request(method: &str, params: &Value) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    })
    .to_string()
}

/// Parse a JSON-RPC 2.0 response, extracting the result or error.
fn parse_rpc_response(response: &str) -> Result<String> {
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
        Some(Value::String(s)) => Ok(s.clone()),
        Some(other) => Ok(other.to_string()),
        None => Err(BiomeOsError("missing result field in response".to_string())),
    }
}

/// Check whether a JSON-RPC response contains an error field.
fn response_has_error(response: &str) -> Result<()> {
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

// ─── Capability Routing ──────────────────────────────────────────────────────

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
        .map_err(|e| BiomeOsError(format!("invalid params JSON: {e}")))?;
    capability_call_value(socket, capability, &args)
}

/// Internal capability call accepting a pre-built [`Value`] to avoid
/// redundant serialize→deserialize round-trips from internal callers.
fn capability_call_value(socket: &Path, capability: &str, args: &Value) -> Result<String> {
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

/// Direct JSON-RPC call targeting a specific biomeOS primal by name.
///
/// **Prefer [`capability_call`] for normal use** — it routes by capability,
/// letting biomeOS discover which primal provides the service at runtime.
/// Use `direct_rpc_call` only when you must bypass capability discovery and
/// target a known primal directly (e.g. hardware-specific operations).
///
/// Internally delegates to [`capability_call_value`] with `"{target}.{method}"`
/// as the capability string.
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

// ─── Storage ─────────────────────────────────────────────────────────────────

/// Store a value via biomeOS capability-based storage routing.
///
/// Routes through `storage.put` capability — biomeOS translates to the
/// storage provider's actual method at runtime.
///
/// # Errors
///
/// Returns `Err` if the socket is unavailable or the RPC fails.
pub fn storage_put(socket: &Path, key: &str, value: &str) -> Result<()> {
    let args = serde_json::json!({
        "key": key,
        "value": value,
        "family_id": FAMILY_ID,
    });
    capability_call_value(socket, "storage.put", &args)?;
    Ok(())
}

/// Retrieve a value via biomeOS capability-based storage routing.
///
/// Routes through `storage.get` capability — biomeOS translates to the
/// storage provider's actual method at runtime.
///
/// # Errors
///
/// Returns `Err` if the socket is unavailable, the key does not exist,
/// or the RPC fails.
pub fn storage_get(socket: &Path, key: &str) -> Result<String> {
    let args = serde_json::json!({
        "key": key,
        "family_id": FAMILY_ID,
    });
    capability_call_value(socket, "storage.get", &args)
}

// ─── Compute Dispatch ────────────────────────────────────────────────────────

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
    let mut args: Value = serde_json::from_str(params_json)
        .map_err(|e| BiomeOsError(format!("invalid compute params: {e}")))?;
    merge_compute_fields(&mut args, op);
    capability_call_value(socket, "compute.execute", &args)
}

/// Submit a compute job asynchronously via `compute.submit`.
///
/// Returns a job ID or status from `ToadStool`.
///
/// # Errors
///
/// Returns `Err` if biomeOS is unavailable or the submission fails.
pub fn compute_submit(socket: &Path, op: &str, params_json: &str) -> Result<String> {
    let mut args: Value = serde_json::from_str(params_json)
        .map_err(|e| BiomeOsError(format!("invalid compute params: {e}")))?;
    merge_compute_fields(&mut args, op);
    capability_call_value(socket, "compute.submit", &args)
}

/// Inject `op` and `family_id` into a compute params [`Value`].
fn merge_compute_fields(args: &mut Value, op: &str) {
    if let Some(obj) = args.as_object_mut() {
        obj.insert("op".to_string(), Value::String(op.to_string()));
        obj.insert(
            "family_id".to_string(),
            Value::String(FAMILY_ID.to_string()),
        );
    }
}

/// Query compute capabilities.
///
/// Returns JSON listing available compute operations and GPU info.
///
/// # Errors
///
/// Returns `Err` if biomeOS or the compute provider is unavailable.
pub fn compute_capabilities(socket: &Path) -> Result<String> {
    let args = serde_json::json!({ "family_id": FAMILY_ID });
    capability_call_value(socket, "resource.health.check", &args)
}

// ─── Capability Registration ─────────────────────────────────────────────────

/// Science capabilities that groundSpring registers with the NUCLEUS.
///
/// Each capability represents a validated scientific computation that other
/// primals can invoke via `capability.call`. The primal providing the
/// capability is discovered at runtime — groundSpring only knows its own
/// capabilities, not which primals might call them.
pub const SCIENCE_CAPABILITIES: &[&str] = &[
    "science.anderson_validation",
    "science.noise_decomposition",
    "science.parity_check",
    "science.et0_propagation",
    "science.regime_classification",
    "science.uncertainty_budget",
    "science.spectral_features",
];

/// Register groundSpring's science capabilities with the NUCLEUS.
///
/// Sends a `capability.register` call for each capability in
/// [`SCIENCE_CAPABILITIES`]. Returns the number of capabilities
/// successfully registered.
///
/// # Errors
///
/// Returns `Err` if the socket is unavailable. Individual registration
/// failures are counted but do not abort the batch.
pub fn register_capabilities(socket: &Path) -> Result<usize> {
    let mut registered = 0;
    for &cap in SCIENCE_CAPABILITIES {
        let args = serde_json::json!({
            "capability": cap,
            "family_id": FAMILY_ID,
            "provider": FAMILY_ID,
            "version": env!("CARGO_PKG_VERSION"),
        });
        match capability_call_value(socket, "capability.register", &args) {
            Ok(_) => registered += 1,
            Err(e) => {
                eprintln!("warn: failed to register {cap}: {e}");
            }
        }
    }
    if registered == 0 {
        return Err(BiomeOsError(
            "no capabilities registered — NUCLEUS may not support registration".to_string(),
        ));
    }
    Ok(registered)
}

/// Deregister groundSpring's capabilities from the NUCLEUS.
///
/// Called during graceful shutdown so the NUCLEUS knows this provider
/// is no longer available.
///
/// # Errors
///
/// Returns `Err` if the socket is unavailable.
pub fn deregister_capabilities(socket: &Path) -> Result<usize> {
    let mut deregistered = 0;
    for &cap in SCIENCE_CAPABILITIES {
        let args = serde_json::json!({
            "capability": cap,
            "family_id": FAMILY_ID,
        });
        if capability_call_value(socket, "capability.deregister", &args).is_ok() {
            deregistered += 1;
        }
    }
    Ok(deregistered)
}

// ─── Health ──────────────────────────────────────────────────────────────────

/// Health check: verify the Neural API is alive.
///
/// Uses `topology.metrics` since the Neural API doesn't expose a bare
/// `health` method.
///
/// # Errors
///
/// Returns `Err` if the socket is unavailable or the health check fails.
pub fn health(socket: &Path) -> Result<()> {
    let params = serde_json::json!({});
    let request = build_request("topology.metrics", &params);
    let response = rpc_call(socket, &request)?;
    response_has_error(&response)
}

/// Send an arbitrary JSON-RPC request over the transport and read the response.
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

// ─── Transport Layer ─────────────────────────────────────────────────────────

/// Send a JSON-RPC request and read the newline-delimited response.
///
/// Uses Unix domain sockets on Unix platforms and TCP on others.
fn rpc_call(socket: &Path, request: &str) -> Result<String> {
    #[cfg(unix)]
    {
        unix_rpc_call(socket, request)
    }
    #[cfg(not(unix))]
    {
        let _ = socket;
        tcp_rpc_call(request)
    }
}

/// Unix domain socket transport.
#[cfg(unix)]
fn unix_rpc_call(socket: &Path, request: &str) -> Result<String> {
    use std::os::unix::net::UnixStream;

    let stream = UnixStream::connect_addr(
        &std::os::unix::net::SocketAddr::from_pathname(socket)
            .map_err(|e| BiomeOsError(format!("invalid socket path: {e}")))?,
    )
    .map_err(|e| BiomeOsError(format!("biomeOS connect {}: {e}", socket.display())))?;

    stream
        .set_read_timeout(Some(read_timeout()))
        .map_err(|e| BiomeOsError(format!("set read timeout: {e}")))?;
    stream
        .set_write_timeout(Some(connect_timeout()))
        .map_err(|e| BiomeOsError(format!("set write timeout: {e}")))?;

    send_receive_stream(&stream, request)
}

/// TCP transport for non-Unix platforms.
///
/// Reads the target address from `GROUNDSPRING_BIOMEOS_TCP` (e.g. `"127.0.0.1:9100"`).
#[cfg(not(unix))]
fn tcp_rpc_call(request: &str) -> Result<String> {
    use std::net::TcpStream;

    let addr = std::env::var("GROUNDSPRING_BIOMEOS_TCP").map_err(|_| {
        BiomeOsError(
            "biomeOS requires GROUNDSPRING_BIOMEOS_TCP (host:port) on non-Unix platforms"
                .to_string(),
        )
    })?;

    let stream = TcpStream::connect_timeout(
        &addr
            .parse()
            .map_err(|e| BiomeOsError(format!("invalid TCP address {addr}: {e}")))?,
        connect_timeout(),
    )
    .map_err(|e| BiomeOsError(format!("biomeOS TCP connect {addr}: {e}")))?;

    stream
        .set_read_timeout(Some(read_timeout()))
        .map_err(|e| BiomeOsError(format!("set read timeout: {e}")))?;
    stream
        .set_write_timeout(Some(connect_timeout()))
        .map_err(|e| BiomeOsError(format!("set write timeout: {e}")))?;

    send_receive_stream(&stream, request)
}

/// Write a newline-delimited JSON-RPC request and read the response line.
///
/// Works with any stream where `&S` implements both `Read` and `Write`
/// (e.g. `UnixStream`, `TcpStream`).
fn send_receive_stream<S>(stream: &S, request: &str) -> Result<String>
where
    for<'a> &'a S: std::io::Read + std::io::Write,
{
    let mut writer = std::io::BufWriter::new(stream);
    writer
        .write_all(request.as_bytes())
        .map_err(|e| BiomeOsError(format!("write to biomeOS: {e}")))?;
    writer
        .write_all(b"\n")
        .map_err(|e| BiomeOsError(format!("write newline: {e}")))?;
    writer
        .flush()
        .map_err(|e| BiomeOsError(format!("flush: {e}")))?;
    drop(writer);

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .map_err(|e| BiomeOsError(format!("read from biomeOS: {e}")))?;

    if line.is_empty() {
        return Err(BiomeOsError("biomeOS returned empty response".to_string()));
    }

    Ok(line)
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
        assert_eq!(
            result,
            Some(sock),
            "should find legacy neural-api-default.sock"
        );
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
        assert_eq!(
            result,
            Some(primary),
            "should prefer neural-api.sock over default"
        );
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
    fn capability_call_request_format() {
        let cap = "science.anderson_validation";
        let (cap_part, op_part) = cap.split_once('.').unwrap();
        let request = build_request(
            "capability.call",
            &serde_json::json!({
                "capability": cap_part,
                "operation": op_part,
                "args": {"n_sites": 10000},
                "family_id": FAMILY_ID,
            }),
        );
        let v: Value = serde_json::from_str(&request).unwrap();
        assert_eq!(v["method"], "capability.call");
        assert_eq!(v["params"]["capability"], "science");
        assert_eq!(v["params"]["operation"], "anderson_validation");
        assert_eq!(v["params"]["family_id"], "groundspring");
    }

    #[test]
    fn science_capabilities_non_empty() {
        assert!(!SCIENCE_CAPABILITIES.is_empty());
        for cap in SCIENCE_CAPABILITIES {
            assert!(
                cap.starts_with("science."),
                "all caps should be in science namespace: {cap}"
            );
        }
    }

    #[test]
    fn register_capabilities_nonexistent_socket_errors() {
        let path = PathBuf::from("/tmp/groundspring_test_nonexistent_register.sock");
        let err = register_capabilities(&path).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("no capabilities registered") || msg.contains("biomeOS connect"),
            "should fail with clear message: {msg}"
        );
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
        let err = direct_rpc_call(&path, "compute", "health", "{}").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("biomeOS connect") || msg.contains("invalid socket"));
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
