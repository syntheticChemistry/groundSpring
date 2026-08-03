// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Runtime primal discovery and direct interaction.
//!
//! Scans the biomeOS socket directory for live primal sockets and provides
//! direct RPC and health-check methods that bypass Neural API capability
//! routing. Use [`super::capability_call`] for normal operation; these
//! functions exist for diagnostics and direct primal-to-primal queries.
//!
//! All discovery is runtime-only — groundSpring has no compile-time
//! knowledge of which primals exist.

use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde_json::Value;

use super::protocol::{build_request, parse_rpc_response, response_has_error};
use super::transport::rpc_call;
use super::{BiomeOsError, Result};

struct DiscoveryCache {
    primals: Vec<DiscoveredPrimal>,
    last_refresh: Instant,
}

static DISCOVERY_CACHE: OnceLock<Mutex<Option<DiscoveryCache>>> = OnceLock::new();

const DISCOVERY_TTL: Duration = Duration::from_secs(30);

fn discovery_cache() -> &'static Mutex<Option<DiscoveryCache>> {
    DISCOVERY_CACHE.get_or_init(|| Mutex::new(None))
}

fn discovery_ttl() -> Duration {
    std::env::var("GROUNDSPRING_DISCOVERY_TTL_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .map(Duration::from_secs)
        .unwrap_or(DISCOVERY_TTL)
}

/// A primal socket discovered in the biomeOS socket directory.
#[derive(Debug, Clone)]
pub struct DiscoveredPrimal {
    /// Primal name derived from socket filename at runtime.
    pub name: String,
    /// Path to the primal's Unix domain socket.
    pub socket: std::path::PathBuf,
}

/// Discover live primal sockets in the biomeOS runtime directory.
///
/// Scans `$XDG_RUNTIME_DIR/biomeos/` for `.sock` files and returns
/// the list of primals found, excluding symlinks and Neural API itself.
///
/// Results are cached process-wide for 30 seconds by default (override via
/// `GROUNDSPRING_DISCOVERY_TTL_SECS`). Call [`refresh_discovered_primals`]
/// to force a rescan before the TTL expires.
#[must_use]
pub fn discover_primals() -> Vec<DiscoveredPrimal> {
    let ttl = discovery_ttl();
    {
        let cache = discovery_cache().lock().unwrap_or_else(|e| e.into_inner());
        if let Some(ref cached) = *cache {
            if cached.last_refresh.elapsed() < ttl {
                return cached.primals.clone();
            }
        }
    }

    refresh_discovered_primals()
}

/// Force a rescan of the biomeOS socket directory and refresh the discovery cache.
#[must_use]
pub fn refresh_discovered_primals() -> Vec<DiscoveredPrimal> {
    let primals = scan_primals();
    let mut cache = discovery_cache().lock().unwrap_or_else(|e| e.into_inner());
    *cache = Some(DiscoveryCache {
        primals: primals.clone(),
        last_refresh: Instant::now(),
    });
    primals
}

fn scan_primals() -> Vec<DiscoveredPrimal> {
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
        if !is_neural_api_socket(name) && path.extension().is_some_and(|e| e == "sock") {
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

/// Whether a socket filename matches the Neural API orchestrator.
///
/// The Neural API socket should be excluded when scanning for individual
/// primal sockets. Uses [`crate::primal_names::NEURAL_API_SOCKET_NAMES`]
/// as the single source of truth.
fn is_neural_api_socket(name: &str) -> bool {
    crate::primal_names::NEURAL_API_SOCKET_NAMES
        .iter()
        .any(|n| name.contains(n.trim_end_matches(".sock")))
}

/// Resolve the biomeOS socket directory from environment.
fn biomeos_socket_dir() -> Option<std::path::PathBuf> {
    biomeos_socket_dir_with_env(|k| std::env::var(k).ok())
}

fn biomeos_socket_dir_with_env(env: impl Fn(&str) -> Option<String>) -> Option<std::path::PathBuf> {
    if let Some(dir) = env("BIOMEOS_SOCKET_DIR") {
        return Some(std::path::PathBuf::from(dir));
    }
    if let Some(xdg) = env("XDG_RUNTIME_DIR") {
        let dir = std::path::PathBuf::from(xdg).join(crate::primal_names::BIOMEOS_SOCKET_DIR);
        if dir.is_dir() {
            return Some(dir);
        }
    }
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::MetadataExt;
        if let Ok(meta) = std::fs::metadata("/proc/self") {
            let dir = std::path::PathBuf::from(format!(
                "/run/user/{}/{}",
                meta.uid(),
                crate::primal_names::BIOMEOS_SOCKET_DIR
            ));
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
/// `health` for the security provider (which responds to the bare method).
///
/// # Errors
///
/// Returns `Err` if the primal is not found or the health check fails.
pub fn primal_health(primal_name: &str) -> Result<String> {
    let primals = discover_primals();
    let primal = primals
        .iter()
        .find(|p| p.name == primal_name)
        .ok_or_else(|| BiomeOsError::Discovery(format!("primal not found: {primal_name}")))?;

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

    Err(BiomeOsError::Discovery(format!(
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
        .ok_or_else(|| BiomeOsError::Discovery(format!("primal not found: {primal_name}")))?;

    let args: Value = serde_json::from_str(params)
        .map_err(|e| BiomeOsError::Serialization(format!("invalid params JSON: {e}")))?;
    let request = build_request(method, &args);
    let response = rpc_call(&primal.socket, &request)?;
    parse_rpc_response(&response)
}

/// Query Neural API proprioception (self-awareness and deployment status).
///
/// # Errors
///
/// Returns `Err` if the Neural API is unavailable.
pub fn proprioception(socket: &std::path::Path) -> Result<String> {
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
pub fn topology(socket: &std::path::Path) -> Result<String> {
    let params = serde_json::json!({});
    let request = build_request("neural_api.get_topology", &params);
    let response = rpc_call(socket, &request)?;
    parse_rpc_response(&response)
}

// ─── Capability-Based Discovery ──────────────────────────────────────────────

/// Discover a primal by capability rather than by name.
///
/// Queries each discovered primal's `capability.list` method and returns
/// the first one advertising the requested capability. This is the
/// sovereign discovery pattern: groundSpring never assumes which primal
/// provides a capability.
///
/// Handles both capability response formats:
/// - **Flat array** (pre-S156): `["compute.execute", "compute.submit"]`
/// - **Nested objects** (S156+): `[{"name": "compute.execute", ...}, ...]`
///
/// # Errors
///
/// Returns `Err` if no primal advertising `capability` is found.
pub fn discover_by_capability(capability: &str) -> Result<DiscoveredPrimal> {
    let primals = discover_primals();
    for primal in primals {
        let request = build_request("capability.list", &serde_json::json!({}));
        if let Ok(ref response) = rpc_call(&primal.socket, &request)
            && let Ok(body) = parse_rpc_response(response)
        {
            let caps = extract_capabilities(&body);
            if caps.iter().any(|c| c == capability) {
                return Ok(primal);
            }
        }
    }
    Err(BiomeOsError::Discovery(format!(
        "no primal found advertising capability: {capability}"
    )))
}

/// Extract capability names from either flat-array or nested-object formats.
///
/// Pre-S156 format: `["compute.execute", "storage.put"]`
/// S156+ format:    `[{"name": "compute.execute", "version": "1.0"}, ...]`
///
/// Falls back to substring search if JSON parsing fails (raw text response).
fn extract_capabilities(body: &str) -> Vec<String> {
    if let Ok(parsed) = serde_json::from_str::<Value>(body) {
        return extract_capabilities_from_value(&parsed);
    }
    if let Ok(parsed) = serde_json::from_str::<Value>(&format!("[{body}]")) {
        return extract_capabilities_from_value(&parsed);
    }
    tracing::debug!(
        body_len = body.len(),
        "capability list body unparseable; returning empty capability set"
    );
    Vec::new()
}

fn extract_capabilities_from_value(value: &Value) -> Vec<String> {
    if let Value::Object(obj) = value {
        for wrapper_key in ["capabilities", "result", "methods"] {
            if let Some(inner) = obj.get(wrapper_key) {
                return extract_capabilities_from_value(inner);
            }
        }
    }

    match value {
        Value::Array(arr) => arr
            .iter()
            .filter_map(|v| match v {
                Value::String(s) => Some(s.clone()),
                Value::Object(obj) => extract_method_name_from_object(obj),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// Extract a method/capability name from a JSON object, supporting all
/// known capability advertisement formats:
///
/// - **Format A** (flat string) — handled by caller
/// - **Format B** (`name`/`capability` key): `{"name": "compute.execute", ...}`
/// - **Format C** (`method_info`): `{"method": "compute.execute", "description": "..."}`
/// - **Format D** (`semantic_mappings`): `{"semantic_method": "compute.execute", ...}`
///   or `{"method_name": "compute.execute", ...}`
fn extract_method_name_from_object(obj: &serde_json::Map<String, Value>) -> Option<String> {
    for key in [
        "name",
        "capability",
        "method",
        "semantic_method",
        "method_name",
    ] {
        if let Some(s) = obj.get(key).and_then(Value::as_str) {
            return Some(s.to_owned());
        }
    }
    None
}

// ─── compute.dispatch.* Direct Dispatch ─────────────────────────────────────

/// Submit a GPU compute job via `compute.dispatch.submit`.
///
/// Discovers the `compute.dispatch.submit` provider at runtime (the compute
/// dispatch provider) and submits a workload for GPU execution. Returns a job
/// handle (ID or inline result depending on provider configuration).
///
/// This bypasses Neural API capability routing for sub-frame latency
/// on GPU dispatch paths (ludoSpring V22 pattern).
///
/// # Errors
///
/// Returns `Err` if no `compute.dispatch.submit` provider is found or the RPC fails.
/// Callers should fall back to CPU-local computation.
pub fn dispatch_submit(op: &str, params: serde_json::Value) -> Result<String> {
    let provider = discover_by_capability("compute.dispatch.submit")?;
    let args = match params {
        Value::Object(mut obj) => {
            obj.insert("op".to_string(), Value::String(op.to_string()));
            Value::Object(obj)
        }
        other => serde_json::json!({ "op": op, "params": other }),
    };
    let request = build_request("compute.dispatch.submit", &args);
    let response = rpc_call(&provider.socket, &request)?;
    parse_rpc_response(&response)
}

/// Poll for a dispatched compute job's result via `compute.dispatch.result`.
///
/// # Errors
///
/// Returns `Err` if the provider is unavailable or the job ID is unknown.
pub fn dispatch_result(job_id: &str) -> Result<String> {
    let provider = discover_by_capability("compute.dispatch.result")?;
    let args = serde_json::json!({ "job_id": job_id });
    let request = build_request("compute.dispatch.result", &args);
    let response = rpc_call(&provider.socket, &request)?;
    parse_rpc_response(&response)
}

/// Query available GPU dispatch capabilities via `compute.dispatch.capabilities`.
///
/// Returns JSON describing supported operations, GPU info, and limits.
///
/// # Errors
///
/// Returns `Err` if no compute dispatch provider is available.
pub fn dispatch_capabilities() -> Result<String> {
    let provider = discover_by_capability("compute.dispatch.capabilities")?;
    let request = build_request("compute.dispatch.capabilities", &serde_json::json!({}));
    let response = rpc_call(&provider.socket, &request)?;
    parse_rpc_response(&response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_capabilities_flat_array() {
        let body = r#"["compute.execute", "storage.put", "storage.get"]"#;
        let caps = extract_capabilities(body);
        assert_eq!(caps, vec!["compute.execute", "storage.put", "storage.get"]);
    }

    #[test]
    fn extract_capabilities_nested_objects_name() {
        let body = r#"[{"name": "compute.dispatch.submit", "version": "1.0"}, {"name": "compute.dispatch.result"}]"#;
        let caps = extract_capabilities(body);
        assert_eq!(
            caps,
            vec!["compute.dispatch.submit", "compute.dispatch.result"]
        );
    }

    #[test]
    fn extract_capabilities_nested_objects_capability_key() {
        let body = r#"[{"capability": "compute.execute", "provider": "toadstool"}]"#;
        let caps = extract_capabilities(body);
        assert_eq!(caps, vec!["compute.execute"]);
    }

    #[test]
    fn extract_capabilities_wrapped_object() {
        let body = r#"{"capabilities": ["compute.execute", "storage.put"]}"#;
        let caps = extract_capabilities(body);
        assert_eq!(caps, vec!["compute.execute", "storage.put"]);
    }

    #[test]
    fn extract_capabilities_empty() {
        let caps = extract_capabilities("[]");
        assert!(caps.is_empty());
    }

    #[test]
    fn extract_capabilities_invalid_json() {
        let caps = extract_capabilities("not json at all");
        assert!(caps.is_empty());
    }

    #[test]
    fn extract_capabilities_result_wrapper() {
        let body = r#"{"result": ["health", "data.weather"]}"#;
        let caps = extract_capabilities(body);
        assert_eq!(caps, vec!["health", "data.weather"]);
    }

    #[test]
    fn extract_capabilities_double_nested() {
        let body = r#"{"capabilities": {"capabilities": ["health", "compute.dispatch"]}}"#;
        let caps = extract_capabilities(body);
        assert_eq!(caps, vec!["health", "compute.dispatch"]);
    }

    #[test]
    fn extract_capabilities_result_with_objects() {
        let body = r#"{"result": [{"name": "compute.execute"}, {"capability": "storage.put"}]}"#;
        let caps = extract_capabilities(body);
        assert_eq!(caps, vec!["compute.execute", "storage.put"]);
    }

    #[test]
    fn extract_capabilities_format_c_method_info() {
        let body = r#"[
            {"method": "compute.execute", "description": "Execute GPU compute workload"},
            {"method": "compute.status", "description": "Query job status"}
        ]"#;
        let caps = extract_capabilities(body);
        assert_eq!(caps, vec!["compute.execute", "compute.status"]);
    }

    #[test]
    fn extract_capabilities_format_d_semantic_mappings() {
        let body = r#"[
            {"semantic_method": "measurement.noise_decomposition", "provider": "groundSpring"},
            {"method_name": "measurement.bootstrap", "provider": "groundSpring"}
        ]"#;
        let caps = extract_capabilities(body);
        assert_eq!(
            caps,
            vec!["measurement.noise_decomposition", "measurement.bootstrap"]
        );
    }

    #[test]
    fn extract_capabilities_methods_wrapper() {
        let body = r#"{"methods": ["health.check", "compute.dispatch.submit"]}"#;
        let caps = extract_capabilities(body);
        assert_eq!(caps, vec!["health.check", "compute.dispatch.submit"]);
    }

    #[test]
    fn extract_capabilities_mixed_formats() {
        let body = r#"{"capabilities": [
            "health",
            {"name": "compute.execute"},
            {"method": "data.query"},
            {"semantic_method": "analysis.run"}
        ]}"#;
        let caps = extract_capabilities(body);
        assert_eq!(
            caps,
            vec!["health", "compute.execute", "data.query", "analysis.run"]
        );
    }
}
