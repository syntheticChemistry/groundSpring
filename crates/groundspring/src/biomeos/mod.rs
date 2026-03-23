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
mod interaction;
mod protocol;
pub mod resilience;
pub mod server;
mod transport;

pub use discovery::{auto_connect, discover_socket, is_nucleus_available};
pub use interaction::{
    DiscoveredPrimal, direct_primal_rpc, discover_by_capability, discover_primals,
    dispatch_capabilities, dispatch_result, dispatch_submit, primal_health, proprioception,
    topology,
};

use std::path::Path;
use std::time::Duration;

use serde_json::Value;

use protocol::{
    DispatchOutcome, build_request, extract_rpc_result, parse_rpc_dispatch, parse_rpc_response,
};
use transport::rpc_call;

// ─── Configuration ───────────────────────────────────────────────────────────

/// Default connect timeout in seconds when env var is unset.
const DEFAULT_CONNECT_TIMEOUT_SECS: u64 = 5;

/// Default read timeout in seconds when env var is unset.
const DEFAULT_READ_TIMEOUT_SECS: u64 = 30;

/// Connect timeout, overridable via `GROUNDSPRING_BIOMEOS_CONNECT_TIMEOUT_SECS`.
fn connect_timeout() -> Duration {
    connect_timeout_with_env(|k| std::env::var(k).ok())
}

fn connect_timeout_with_env(env: impl Fn(&str) -> Option<String>) -> Duration {
    Duration::from_secs(
        env("GROUNDSPRING_BIOMEOS_CONNECT_TIMEOUT_SECS")
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_CONNECT_TIMEOUT_SECS),
    )
}

/// Read timeout, overridable via `GROUNDSPRING_BIOMEOS_READ_TIMEOUT_SECS`.
fn read_timeout() -> Duration {
    read_timeout_with_env(|k| std::env::var(k).ok())
}

fn read_timeout_with_env(env: impl Fn(&str) -> Option<String>) -> Duration {
    Duration::from_secs(
        env("GROUNDSPRING_BIOMEOS_READ_TIMEOUT_SECS")
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_READ_TIMEOUT_SECS),
    )
}

/// Family identifier for all biomeOS interactions.
///
/// Delegates to [`crate::niche::NICHE_ID`] — the single source of truth
/// for this spring's identity within the ecosystem.
pub const FAMILY_ID: &str = crate::niche::NICHE_ID;

// ─── Error Type ──────────────────────────────────────────────────────────────

/// Error type for `biomeOS` client operations.
///
/// Typed variants replace the former `BiomeOsError(String)` for better
/// error handling and pattern matching. The `Other` variant handles
/// messages that don't fit a specific category.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum BiomeOsError {
    /// Transport-level failure (connect, read, write, flush, timeout).
    #[error("biomeOS transport: {0}")]
    Transport(String),
    /// JSON-RPC protocol error (invalid response, missing fields, RPC error).
    #[error("biomeOS protocol: {0}")]
    Protocol(String),
    /// Serialization error (invalid params JSON).
    #[error("biomeOS serialization: {0}")]
    Serialization(String),
    /// Capability registration failure.
    #[error("biomeOS registration: {0}")]
    Registration(String),
    /// Primal discovery or health check failure.
    #[error("biomeOS discovery: {0}")]
    Discovery(String),
    /// Data pipeline error (no results, empty response).
    #[error("biomeOS data: {0}")]
    Data(String),
    /// Uncategorized error (migration path from `BiomeOsError(String)`).
    #[error("biomeOS: {0}")]
    Other(String),
}

impl BiomeOsError {
    /// Construct from a plain string (backwards-compatible migration path).
    #[must_use]
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }

    /// Whether this error is transient and the operation may succeed on retry.
    ///
    /// Transport errors and some discovery failures are recoverable.
    /// Protocol, serialization, and data errors are permanent.
    ///
    /// Absorbed from airSpring V0.8.8 / wetSpring V121 `is_recoverable()` pattern.
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        matches!(self, Self::Transport(_) | Self::Discovery(_))
    }

    /// Whether a retry with backoff is appropriate.
    ///
    /// Same as [`is_recoverable`](Self::is_recoverable) — transient errors
    /// should be retried; permanent errors should be surfaced immediately.
    #[must_use]
    pub const fn is_retriable(&self) -> bool {
        self.is_recoverable()
    }

    /// Whether this is a JSON-RPC "method not found" error.
    ///
    /// Useful for fallback chains where the first method may not be
    /// supported by the target primal version.
    #[must_use]
    pub fn is_method_not_found(&self) -> bool {
        match self {
            Self::Protocol(msg) => msg.contains("-32601") || msg.contains("method not found"),
            _ => false,
        }
    }
}

/// Result alias for `biomeOS` operations.
pub type Result<T> = std::result::Result<T, BiomeOsError>;

// ─── Feature Detection ───────────────────────────────────────────────────────

/// Whether `biomeOS` routing is enabled via environment.
#[must_use]
pub fn is_enabled() -> bool {
    std::env::var("GROUNDSPRING_COMPUTE_PROVIDER").is_ok_and(|v| {
        v.trim()
            .eq_ignore_ascii_case(crate::primal_names::roles::ORCHESTRATOR)
    })
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
        .map_err(|e| BiomeOsError::Serialization(format!("invalid params JSON: {e}")))?;
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

/// Dispatch a computation via `compute.execute` capability routing.
///
/// The `op` field names the operation (e.g. `"lyapunov_averaged"`).
/// Additional fields in `params_json` carry the operation-specific arguments.
/// biomeOS routes to whichever primal provides the `compute` capability.
///
/// # Errors
///
/// Returns `Err` if biomeOS is unavailable or the compute provider rejects
/// the request.
pub fn compute_execute(socket: &Path, op: &str, params_json: &str) -> Result<String> {
    let mut args: Value = serde_json::from_str(params_json)
        .map_err(|e| BiomeOsError::Serialization(format!("invalid compute params: {e}")))?;
    merge_compute_fields(&mut args, op);
    capability_call_value(socket, "compute.execute", &args)
}

/// Submit a compute job asynchronously via `compute.submit`.
///
/// Returns a job ID or status from the compute provider.
///
/// # Errors
///
/// Returns `Err` if biomeOS is unavailable or the submission fails.
pub fn compute_submit(socket: &Path, op: &str, params_json: &str) -> Result<String> {
    let mut args: Value = serde_json::from_str(params_json)
        .map_err(|e| BiomeOsError::Serialization(format!("invalid compute params: {e}")))?;
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

// ─── Measurement Domain ──────────────────────────────────────────────────────

/// Capability domain name for groundSpring's measurement validation.
///
/// Delegates to [`crate::niche::DOMAIN`] — the single source of truth.
pub const MEASUREMENT_DOMAIN: &str = crate::niche::DOMAIN;

/// Measurement capabilities that groundSpring registers with the NUCLEUS.
///
/// Delegates to [`crate::niche::CAPABILITIES`] — the single source of truth.
pub const MEASUREMENT_CAPABILITIES: &[&str] = crate::niche::CAPABILITIES;

/// Legacy alias — callers that referenced `SCIENCE_CAPABILITIES` will
/// continue to compile. New code should use [`MEASUREMENT_CAPABILITIES`].
#[deprecated(note = "use MEASUREMENT_CAPABILITIES (measurement.* domain)")]
pub const SCIENCE_CAPABILITIES: &[&str] = MEASUREMENT_CAPABILITIES;

/// Semantic mappings from measurement domain operations to JSON-RPC methods.
///
/// Delegates to [`crate::niche::SEMANTIC_MAPPINGS`] — the single source of truth.
pub const MEASUREMENT_MAPPINGS: &[(&str, &str)] = crate::niche::SEMANTIC_MAPPINGS;

// ─── Capability Registration ─────────────────────────────────────────────────

/// Register groundSpring as a measurement provider with the NUCLEUS.
///
/// Two-phase registration following the Spring-as-Provider pattern:
/// 1. Domain registration: `capability.register` with `measurement` domain
///    and semantic mappings
/// 2. Individual capabilities: per-capability registration for direct routing
///
/// Returns the number of capabilities successfully registered.
///
/// # Errors
///
/// Returns `Err` if the socket is unavailable. Individual registration
/// failures are counted but do not abort the batch.
pub fn register_capabilities(socket: &Path) -> Result<usize> {
    let socket_path = server::socket_path();
    let socket_str = socket_path.to_string_lossy().to_string();

    // Phase 1: Domain registration with semantic mappings
    let mappings: serde_json::Map<String, Value> = MEASUREMENT_MAPPINGS
        .iter()
        .map(|(op, method)| ((*op).to_string(), Value::String((*method).to_string())))
        .collect();

    let domain_args = serde_json::json!({
        "capability": MEASUREMENT_DOMAIN,
        "primal": FAMILY_ID,
        "socket": socket_str,
        "source": "startup",
        "semantic_mappings": mappings,
        "family_id": FAMILY_ID,
        "version": env!("CARGO_PKG_VERSION"),
    });
    match capability_call_value(socket, "capability.register", &domain_args) {
        Ok(_) => log::info!("registered measurement domain"),
        Err(e) => log::warn!("domain registration failed (non-fatal): {e}"),
    }

    // Phase 2: Individual capability registration
    let mut registered = 0;
    for &cap in MEASUREMENT_CAPABILITIES {
        let args = serde_json::json!({
            "capability": cap,
            "primal": FAMILY_ID,
            "socket": socket_str,
            "family_id": FAMILY_ID,
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
        return Err(BiomeOsError::Registration(
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
    for &cap in MEASUREMENT_CAPABILITIES {
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
            Ok(ref response) => match parse_rpc_dispatch(response) {
                Ok(DispatchOutcome::Ok(_)) => return Ok(()),
                Ok(ref outcome) if outcome.is_method_not_found() => {
                    log::debug!("health {method}: method not found, trying next");
                }
                Ok(DispatchOutcome::ApplicationError { message, .. }) => {
                    log::debug!("health {method}: application error: {message}");
                }
                Ok(DispatchOutcome::ProtocolError { message, .. }) => {
                    log::debug!("health {method}: protocol error: {message}");
                }
                Err(_) => {}
            },
            Err(_) => {
                return Err(BiomeOsError::Transport(format!(
                    "biomeOS connect {}",
                    socket.display()
                )));
            }
        }
    }

    Err(BiomeOsError::Discovery(
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

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    /// Atomic counter for unique test socket paths (ludoSpring V28 pattern).
    /// Prevents CI flakiness from parallel test socket collisions.
    static TEST_SOCKET_ID: AtomicU64 = AtomicU64::new(0);

    fn unique_test_socket(label: &str) -> std::path::PathBuf {
        let id = TEST_SOCKET_ID.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("gs_test_{label}_{id}.sock"))
    }

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
        let cap = "measurement.anderson_validation";
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
        assert_eq!(v["params"]["capability"], "measurement");
        assert_eq!(v["params"]["operation"], "anderson_validation");
        assert_eq!(v["params"]["family_id"], "groundspring");
    }

    #[test]
    fn measurement_capabilities_non_empty() {
        assert!(!MEASUREMENT_CAPABILITIES.is_empty());
        for cap in MEASUREMENT_CAPABILITIES {
            assert!(
                cap.starts_with("measurement."),
                "all caps should be in measurement namespace: {cap}"
            );
        }
    }

    #[test]
    fn measurement_mappings_complete() {
        assert_eq!(
            MEASUREMENT_MAPPINGS.len(),
            MEASUREMENT_CAPABILITIES.len(),
            "every capability needs a semantic mapping"
        );
    }

    #[test]
    fn register_capabilities_nonexistent_socket_errors() {
        let path = unique_test_socket("register");
        let err = register_capabilities(&path).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("no capabilities registered") || msg.contains("biomeOS connect"),
            "should fail with clear message: {msg}"
        );
    }

    #[test]
    fn health_nonexistent_socket_errors() {
        let path = unique_test_socket("health");
        let err = health(&path).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("biomeOS connect") || msg.contains("invalid socket"));
    }

    #[test]
    fn capability_call_nonexistent_socket_errors() {
        let path = unique_test_socket("cap");
        let err = capability_call(&path, "science.test", "{}").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("biomeOS connect") || msg.contains("invalid socket"));
    }

    #[test]
    fn direct_rpc_call_nonexistent_socket_errors() {
        let path = unique_test_socket("rpc");
        let err = direct_rpc_call(&path, "compute", "health", "{}").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("biomeOS connect") || msg.contains("invalid socket"));
    }

    #[test]
    fn unique_test_socket_paths_are_unique() {
        let a = unique_test_socket("uniq");
        let b = unique_test_socket("uniq");
        assert_ne!(a, b, "each call should produce a distinct path");
    }
}
