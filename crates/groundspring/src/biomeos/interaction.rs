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
    if let Ok(dir) = std::env::var("BIOMEOS_SOCKET_DIR") {
        return Some(std::path::PathBuf::from(dir));
    }
    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
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
