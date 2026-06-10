// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Capability registration with the NUCLEUS (`capability.register` / `capability.deregister`).

use std::path::Path;

use serde_json::Value;

use super::routing::capability_call_value;
use super::{
    BiomeOsError, FAMILY_ID, MEASUREMENT_CAPABILITIES, MEASUREMENT_DOMAIN, MEASUREMENT_MAPPINGS,
    Result, server,
};

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
        Ok(_) => tracing::info!("registered measurement domain"),
        Err(e) => tracing::warn!("domain registration failed (non-fatal): {e}"),
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
                tracing::warn!("failed to register {cap}: {e}");
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

/// Register groundSpring's methods with biomeOS v3.51 `method.register`.
///
/// Unlike [`register_capabilities`] which uses the older `capability.register`
/// endpoint, this calls `method.register` (GAP-09) to dynamically register
/// `measurement.*` methods into the semantic routing layer. biomeOS extracts
/// the `measurement` domain and registers each operation for direct IPC routing.
///
/// # Errors
///
/// Returns `Err` if the socket is unavailable or `method.register` is not
/// supported by the running biomeOS version.
pub fn register_methods(socket: &Path) -> Result<usize> {
    let socket_path = server::socket_path();
    let socket_str = socket_path.to_string_lossy().to_string();

    let methods: Vec<&str> = MEASUREMENT_CAPABILITIES.to_vec();

    let params = serde_json::json!({
        "primal": FAMILY_ID,
        "transport": socket_str,
        "methods": methods,
        "source": "groundspring.startup",
    });

    let request = super::protocol::build_request("method.register", &params);
    let response = super::transport::rpc_call(socket, &request)?;
    let result = super::protocol::extract_rpc_result(&response)?;

    let registered = result["registered"]
        .as_u64()
        .unwrap_or(0)
        .try_into()
        .unwrap_or(0);

    if registered > 0 {
        tracing::info!("method.register: {registered} methods registered via biomeOS v3.51");
    } else {
        tracing::warn!("method.register returned 0 registered methods");
    }

    Ok(registered)
}

/// Announce groundSpring to the NUCLEUS via `primal.announce`, falling back
/// to the legacy `capability.register` + `method.register` pattern if the
/// orchestrator does not support the announce protocol.
///
/// This implements the Wave 17 signal adoption standard: a single
/// `primal.announce` call replaces the 3-call registration pattern.
///
/// # Errors
///
/// Returns `Err` if all registration paths fail.
pub fn announce_or_register(socket: &Path) -> Result<usize> {
    let socket_path = server::socket_path();
    let socket_str = socket_path.to_string_lossy().to_string();

    let methods: Vec<&str> = MEASUREMENT_CAPABILITIES.to_vec();

    let announce_params = serde_json::json!({
        "primal": FAMILY_ID,
        "socket": socket_str,
        "methods": methods,
        "capabilities": [MEASUREMENT_DOMAIN],
        "version": env!("CARGO_PKG_VERSION"),
        "lifecycle": { "state": "running" },
    });

    let request = super::protocol::build_request("primal.announce", &announce_params);
    match super::transport::rpc_call(socket, &request) {
        Ok(response) => {
            if let Ok(result) = super::protocol::extract_rpc_result(&response) {
                let registered = result["methods_registered"]
                    .as_u64()
                    .or_else(|| result["registered"].as_u64())
                    .unwrap_or(methods.len() as u64);
                tracing::info!(
                    count = registered,
                    "primal.announce: registered via signal protocol"
                );
                return Ok(registered as usize);
            }
            tracing::info!("primal.announce accepted (no count in response)");
            Ok(methods.len())
        }
        Err(e) => {
            tracing::info!(
                error = %e,
                "primal.announce not available — falling back to legacy registration"
            );
            let cap_result = register_capabilities(socket);
            let method_result = register_methods(socket);
            let cap_count = cap_result.unwrap_or_else(|e| {
                tracing::warn!(error = %e, "legacy capability registration failed");
                0
            });
            let method_count = method_result.unwrap_or_else(|e| {
                tracing::warn!(error = %e, "legacy method registration failed");
                0
            });
            let total = cap_count.max(method_count);
            if total == 0 {
                return Err(BiomeOsError::Registration(
                    "announce and legacy registration both failed".to_string(),
                ));
            }
            Ok(total)
        }
    }
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
        match capability_call_value(socket, "capability.deregister", &args) {
            Ok(_) => deregistered += 1,
            Err(e) => tracing::debug!(capability = cap, error = %e, "deregister skipped"),
        }
    }
    Ok(deregistered)
}

/// Register groundSpring as an AI-routable provider with Squirrel.
///
/// Squirrel's `provider.register` enables AI coordination routing to
/// groundSpring's measurement capabilities. Unlike NUCLEUS registration
/// (which handles IPC routing), Squirrel registration enables AI workload
/// routing via the Model Context Protocol coordination layer.
///
/// Discovery: looks for `SQUIRREL_SOCKET` env var, then falls back to
/// `$XDG_RUNTIME_DIR/biomeos/squirrel.sock`.
///
/// # Errors
///
/// Returns `Err` if the Squirrel socket is unavailable. This is non-fatal
/// for normal operation (graceful degradation — AI routing unavailable).
pub fn register_with_squirrel() -> Result<()> {
    let squirrel_socket = discover_squirrel_socket()?;
    let socket_path = server::socket_path();
    let socket_str = socket_path.to_string_lossy().to_string();

    let methods: Vec<&str> = MEASUREMENT_CAPABILITIES.to_vec();

    let params = serde_json::json!({
        "provider_id": FAMILY_ID,
        "socket": socket_str,
        "capabilities": methods,
        "version": env!("CARGO_PKG_VERSION"),
        "domain": MEASUREMENT_DOMAIN,
        "priority": 128_u8,
    });

    let request = super::protocol::build_request("provider.register", &params);
    match super::transport::rpc_call(&squirrel_socket, &request) {
        Ok(response) => {
            if super::protocol::extract_rpc_result(&response).is_ok() {
                tracing::info!(
                    provider_id = FAMILY_ID,
                    capabilities = methods.len(),
                    "registered with Squirrel AI coordinator"
                );
                Ok(())
            } else {
                tracing::warn!("Squirrel provider.register returned error in response");
                Err(BiomeOsError::Registration(
                    "Squirrel provider.register rejected".to_string(),
                ))
            }
        }
        Err(e) => {
            tracing::debug!(error = %e, "Squirrel not available — AI routing disabled");
            Err(e)
        }
    }
}

/// Discover the Squirrel UDS path using the standard resolution chain.
fn discover_squirrel_socket() -> Result<std::path::PathBuf> {
    if let Ok(path) = std::env::var("SQUIRREL_SOCKET") {
        let p = std::path::PathBuf::from(path);
        if p.exists() {
            return Ok(p);
        }
    }

    let biomeos_dir = super::discovery::biomeos_runtime_dir();
    let squirrel_sock = format!("{}.sock", crate::primal_names::roles::ASSISTANT);
    let default_path = biomeos_dir.join(squirrel_sock);
    if default_path.exists() {
        return Ok(default_path);
    }

    Err(BiomeOsError::Discovery(
        "Squirrel socket not found (set SQUIRREL_SOCKET or ensure squirrel is running)".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measurement_capabilities_non_empty() {
        assert!(!MEASUREMENT_CAPABILITIES.is_empty());
    }

    #[test]
    fn measurement_domain_is_measurement() {
        assert_eq!(MEASUREMENT_DOMAIN, "measurement");
    }

    #[test]
    fn mappings_cover_all_capabilities() {
        for &cap in MEASUREMENT_CAPABILITIES {
            assert!(
                MEASUREMENT_MAPPINGS
                    .iter()
                    .any(|(_, method)| *method == cap),
                "capability {cap} not found in mappings"
            );
        }
    }

    #[test]
    fn register_methods_on_nonexistent_socket_returns_error() {
        let path = std::path::Path::new("/tmp/nonexistent_groundspring_test.sock");
        assert!(register_methods(path).is_err());
    }
}
