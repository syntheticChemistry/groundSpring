// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Neural API health checks (`neural_api.get_metrics`, `topology.metrics`).

use std::path::Path;

use super::protocol::{DispatchOutcome, build_request, parse_rpc_dispatch};
use super::transport::rpc_call;
use super::{BiomeOsError, Result};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_on_nonexistent_socket_returns_error() {
        let path = std::path::Path::new("/tmp/nonexistent_groundspring_test.sock");
        assert!(health(path).is_err());
    }
}
