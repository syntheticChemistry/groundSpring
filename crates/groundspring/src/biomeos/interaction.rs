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

use serde_json::Value;

use super::protocol::{build_request, parse_rpc_response, response_has_error};
use super::transport::rpc_call;
use super::{BiomeOsError, Result};

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
    for primal in &primals {
        let request = build_request("capability.list", &serde_json::json!({}));
        if let Ok(ref response) = rpc_call(&primal.socket, &request)
            && let Ok(body) = parse_rpc_response(response)
        {
            let caps = extract_capabilities(&body);
            if caps.iter().any(|c| c == capability) {
                return Ok(primal.clone());
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
    Vec::new()
}

fn extract_capabilities_from_value(value: &Value) -> Vec<String> {
    let mut caps = Vec::new();
    match value {
        Value::Array(arr) => {
            for item in arr {
                match item {
                    Value::String(s) => caps.push(s.clone()),
                    Value::Object(obj) => {
                        if let Some(Value::String(name)) = obj.get("name") {
                            caps.push(name.clone());
                        } else if let Some(Value::String(cap)) = obj.get("capability") {
                            caps.push(cap.clone());
                        }
                    }
                    _ => {}
                }
            }
        }
        Value::Object(obj) => {
            if let Some(Value::Array(arr)) = obj.get("capabilities") {
                caps.extend(extract_capabilities_from_value(&Value::Array(arr.clone())));
            }
        }
        _ => {}
    }
    caps
}

// ─── toadStool compute.dispatch.* Direct Dispatch ────────────────────────────

/// Submit a GPU compute job directly to toadStool via `compute.dispatch.submit`.
///
/// Discovers the `compute.dispatch.submit` provider at runtime (toadStool)
/// and submits a workload for GPU execution. Returns a job handle (ID or
/// inline result depending on toadStool configuration).
///
/// This bypasses Neural API capability routing for sub-frame latency
/// on GPU dispatch paths (ludoSpring V22 pattern).
///
/// # Errors
///
/// Returns `Err` if no `compute.dispatch.submit` provider is found or the RPC fails.
/// Callers should fall back to CPU-local computation.
pub fn dispatch_submit(op: &str, params: &serde_json::Value) -> Result<String> {
    let provider = discover_by_capability("compute.dispatch.submit")?;
    let mut args = params.clone();
    if let Some(obj) = args.as_object_mut() {
        obj.insert("op".to_string(), Value::String(op.to_string()));
    }
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
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
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
}
