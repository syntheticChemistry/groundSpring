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

mod discovery;
mod protocol;
mod transport;

pub use discovery::{auto_connect, discover_socket, is_nucleus_available};

use std::path::Path;
use std::time::Duration;

use serde_json::Value;

use protocol::{build_request, parse_rpc_response, response_has_error};
use transport::rpc_call;

// ─── Configuration ───────────────────────────────────────────────────────────

/// Default connect timeout in seconds when env var is unset.
const DEFAULT_CONNECT_TIMEOUT_SECS: u64 = 5;

/// Default read timeout in seconds when env var is unset.
const DEFAULT_READ_TIMEOUT_SECS: u64 = 30;

/// Connect timeout, overridable via `GROUNDSPRING_BIOMEOS_CONNECT_TIMEOUT_SECS`.
fn connect_timeout() -> Duration {
    Duration::from_secs(
        std::env::var("GROUNDSPRING_BIOMEOS_CONNECT_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_CONNECT_TIMEOUT_SECS),
    )
}

/// Read timeout, overridable via `GROUNDSPRING_BIOMEOS_READ_TIMEOUT_SECS`.
fn read_timeout() -> Duration {
    Duration::from_secs(
        std::env::var("GROUNDSPRING_BIOMEOS_READ_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_READ_TIMEOUT_SECS),
    )
}

/// Family identifier for all biomeOS interactions.
///
/// Used in JSON-RPC requests and provenance key namespacing to identify
/// this spring within the ecosystem. Other modules (e.g. `nestgate`)
/// should reference this constant rather than duplicating the literal.
pub const FAMILY_ID: &str = "groundspring";

// ─── Error Type ──────────────────────────────────────────────────────────────

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

// ─── Feature Detection ───────────────────────────────────────────────────────

/// Whether `biomeOS` routing is enabled via environment.
#[must_use]
pub fn is_enabled() -> bool {
    std::env::var("GROUNDSPRING_COMPUTE_PROVIDER")
        .is_ok_and(|v| v.trim().eq_ignore_ascii_case("biomeos"))
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
                log::warn!("failed to register {cap}: {e}");
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
/// Tries the evolved `neural_api.get_metrics` method first (current
/// live biomeOS binary), then falls back to the aliased
/// `topology.metrics` for forward compatibility with newer versions.
///
/// # Errors
///
/// Returns `Err` if the socket is unavailable or all health methods fail.
pub fn health(socket: &Path) -> Result<()> {
    let params = serde_json::json!({});

    for method in &["neural_api.get_metrics", "topology.metrics"] {
        let request = build_request(method, &params);
        match rpc_call(socket, &request) {
            Ok(ref response) if response_has_error(response).is_ok() => return Ok(()),
            Ok(_) => {}
            Err(_) => {
                return Err(BiomeOsError(format!(
                    "biomeOS connect {}",
                    socket.display()
                )));
            }
        }
    }

    Err(BiomeOsError(
        "Neural API did not respond to any known health method".to_string(),
    ))
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

// ─── Direct Primal Interaction ──────────────────────────────────────────────

/// A primal socket discovered in the biomeOS socket directory.
#[derive(Debug, Clone)]
pub struct DiscoveredPrimal {
    /// Primal name derived from socket filename (e.g. `"beardog"`, `"toadstool"`).
    pub name: String,
    /// Path to the primal's Unix domain socket.
    pub socket: std::path::PathBuf,
}

/// Discover live primal sockets in the biomeOS runtime directory.
///
/// Scans `$XDG_RUNTIME_DIR/biomeos/` for `.sock` files and returns
/// the list of primals found, excluding symlinks and Neural API itself.
#[must_use]
pub fn discover_primals() -> Vec<DiscoveredPrimal> {
    let Some(socket_dir) = biomeos_socket_dir() else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(&socket_dir) else {
        return Vec::new();
    };
    let mut primals = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path
            .symlink_metadata()
            .is_ok_and(|m| m.file_type().is_symlink())
        {
            continue;
        }
        let Some(name) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if !name.contains("neural-api") && path.extension().is_some_and(|e| e == "sock") {
            let clean_name = name.replace(".jsonrpc", "");
            if primals
                .iter()
                .any(|p: &DiscoveredPrimal| p.name == clean_name)
            {
                continue;
            }
            primals.push(DiscoveredPrimal {
                name: clean_name,
                socket: path,
            });
        }
    }
    primals.sort_by(|a, b| a.name.cmp(&b.name));
    primals
}

/// Resolve the biomeOS socket directory from environment.
fn biomeos_socket_dir() -> Option<std::path::PathBuf> {
    if let Ok(dir) = std::env::var("BIOMEOS_SOCKET_DIR") {
        return Some(std::path::PathBuf::from(dir));
    }
    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        let dir = std::path::PathBuf::from(xdg).join("biomeos");
        if dir.is_dir() {
            return Some(dir);
        }
    }
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::MetadataExt;
        if let Ok(meta) = std::fs::metadata("/proc/self") {
            let dir = std::path::PathBuf::from(format!("/run/user/{}/biomeos", meta.uid()));
            if dir.is_dir() {
                return Some(dir);
            }
        }
    }
    None
}

/// Health-check a specific primal by name via its discovered socket.
///
/// Uses the primal-specific health method: `{primal}.health` for most,
/// `health` for `BearDog` (which responds to the bare method).
///
/// # Errors
///
/// Returns `Err` if the primal is not found or the health check fails.
pub fn primal_health(primal_name: &str) -> Result<String> {
    let primals = discover_primals();
    let primal = primals
        .iter()
        .find(|p| p.name == primal_name)
        .ok_or_else(|| BiomeOsError(format!("primal not found: {primal_name}")))?;

    let qualified = format!("{primal_name}.health");
    let methods: Vec<&str> = vec![&qualified, "health"];

    for method in &methods {
        let request = build_request(method, &serde_json::json!({}));
        if let Ok(ref response) = rpc_call(&primal.socket, &request)
            && response_has_error(response).is_ok()
        {
            return parse_rpc_response(response);
        }
    }

    Err(BiomeOsError(format!(
        "{primal_name} did not respond to any known health method"
    )))
}

/// Send a JSON-RPC call directly to a discovered primal's socket.
///
/// Bypasses Neural API routing. Use when the Neural API doesn't support
/// `capability.call` routing or when targeting a specific primal for
/// diagnosis.
///
/// # Errors
///
/// Returns `Err` if the primal is not found or the RPC fails.
pub fn direct_primal_rpc(primal_name: &str, method: &str, params: &str) -> Result<String> {
    let primals = discover_primals();
    let primal = primals
        .iter()
        .find(|p| p.name == primal_name)
        .ok_or_else(|| BiomeOsError(format!("primal not found: {primal_name}")))?;

    let args: Value = serde_json::from_str(params)
        .map_err(|e| BiomeOsError(format!("invalid params JSON: {e}")))?;
    let request = build_request(method, &args);
    let response = rpc_call(&primal.socket, &request)?;
    parse_rpc_response(&response)
}

/// Query Neural API proprioception (self-awareness and deployment status).
///
/// # Errors
///
/// Returns `Err` if the Neural API is unavailable.
pub fn proprioception(socket: &Path) -> Result<String> {
    let params = serde_json::json!({});
    let request = build_request("neural_api.get_proprioception", &params);
    let response = rpc_call(socket, &request)?;
    parse_rpc_response(&response)
}

/// Query Neural API topology (primal connections).
///
/// # Errors
///
/// Returns `Err` if the Neural API is unavailable.
pub fn topology(socket: &Path) -> Result<String> {
    let params = serde_json::json!({});
    let request = build_request("neural_api.get_topology", &params);
    let response = rpc_call(socket, &request)?;
    parse_rpc_response(&response)
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
        let path = std::env::temp_dir().join("groundspring_test_nonexistent_register.sock");
        let err = register_capabilities(&path).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("no capabilities registered") || msg.contains("biomeOS connect"),
            "should fail with clear message: {msg}"
        );
    }

    #[test]
    fn health_nonexistent_socket_errors() {
        let path = std::env::temp_dir().join("groundspring_test_nonexistent_biomeos.sock");
        let err = health(&path).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("biomeOS connect") || msg.contains("invalid socket"));
    }

    #[test]
    fn capability_call_nonexistent_socket_errors() {
        let path = std::env::temp_dir().join("groundspring_test_nonexistent_cap.sock");
        let err = capability_call(&path, "science.test", "{}").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("biomeOS connect") || msg.contains("invalid socket"));
    }

    #[test]
    fn direct_rpc_call_nonexistent_socket_errors() {
        let path = std::env::temp_dir().join("groundspring_test_nonexistent_rpc.sock");
        let err = direct_rpc_call(&path, "compute", "health", "{}").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("biomeOS connect") || msg.contains("invalid socket"));
    }
}
