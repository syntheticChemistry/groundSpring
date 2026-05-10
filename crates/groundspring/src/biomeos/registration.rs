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
