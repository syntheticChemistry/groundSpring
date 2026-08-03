// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Neural API health checks and composition status.
//!
//! - [`health`]: liveness probe via `neural_api.get_metrics` / `topology.metrics`
//! - [`composition_status`]: biomeOS v3.51 `composition.status` — returns
//!   `{ active_users, primal_health, resource_pressure }` for adaptive monitoring

use std::path::Path;

use serde_json::Value;

use super::protocol::{DispatchOutcome, build_request, extract_rpc_result, parse_rpc_dispatch};
use super::transport::rpc_call;
use super::{BiomeOsError, Result};

/// Composition health snapshot from `composition.status`.
///
/// Fields mirror the biomeOS v3.51 response:
/// - `active_users`: count of primals in Active state
/// - `primal_health`: per-primal health objects
/// - `resource_pressure`: host CPU / memory / disk from `/proc`
/// - `total_primals`: total primals in the composition
/// - `topology_version`: current topology revision
#[derive(Debug, Clone)]
pub struct CompositionStatus {
    /// Number of primals in Active state.
    pub active_users: u64,
    /// Per-primal health snapshots (raw JSON objects).
    pub primal_health: Vec<Value>,
    /// Host resource pressure `{ cpu, memory, disk }`.
    pub resource_pressure: Value,
    /// Total primals in the composition.
    pub total_primals: u64,
    /// Topology version counter.
    pub topology_version: u64,
}

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
                    tracing::debug!("health {method}: method not found, trying next");
                }
                Ok(DispatchOutcome::ApplicationError { message, .. }) => {
                    tracing::debug!("health {method}: application error: {message}");
                }
                Ok(DispatchOutcome::ProtocolError { message, .. }) => {
                    tracing::debug!("health {method}: protocol error: {message}");
                }
                Err(e) => {
                    tracing::trace!("health {method}: response parse failed: {e}");
                }
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

/// Query biomeOS `composition.status` for adaptive health monitoring.
///
/// biomeOS v3.51 returns `{ active_users, primal_health, resource_pressure }`
/// so springs can drive adaptive daemons without scraping infrastructure.
///
/// # Errors
///
/// Returns `Err` if the socket is unavailable or the method is not supported.
pub fn composition_status(socket: &Path) -> Result<CompositionStatus> {
    let request = build_request("composition.status", &serde_json::json!({}));
    let response = rpc_call(socket, &request)?;
    let result = extract_rpc_result(&response)?;

    let active_users = result["active_users"].as_u64().unwrap_or(0);
    let total_primals = result["total_primals"].as_u64().unwrap_or(0);
    let topology_version = result["topology_version"].as_u64().unwrap_or(0);

    let primal_health = result["primal_health"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let resource_pressure = result
        .get("resource_pressure")
        .cloned()
        .unwrap_or(Value::Null);

    Ok(CompositionStatus {
        active_users,
        primal_health,
        resource_pressure,
        total_primals,
        topology_version,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_on_nonexistent_socket_returns_error() {
        let path = std::path::Path::new("/tmp/nonexistent_groundspring_test.sock");
        assert!(health(path).is_err());
    }

    #[test]
    fn composition_status_on_nonexistent_socket_returns_error() {
        let path = std::path::Path::new("/tmp/nonexistent_groundspring_test.sock");
        assert!(composition_status(path).is_err());
    }
}
