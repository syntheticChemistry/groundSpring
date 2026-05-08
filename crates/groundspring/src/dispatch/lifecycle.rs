// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Health, capability, and lifecycle methods for the groundSpring primal.
//!
//! These are infrastructure methods required by the biomeOS primal protocol —
//! they do not depend on any domain-specific library code.

use serde_json::Value;

static START_TIME: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();

/// Initialize the start time. Call once at server startup.
pub fn init_start_time() {
    START_TIME.get_or_init(std::time::Instant::now);
}

fn uptime_secs() -> u64 {
    START_TIME
        .get()
        .map_or(0, |start| start.elapsed().as_secs())
}

/// `health.check` — full health status including capabilities and uptime.
pub(super) fn health_check() -> Value {
    serde_json::json!({
        "status": "healthy",
        "primal": crate::biomeos::FAMILY_ID,
        "version": env!("CARGO_PKG_VERSION"),
        "capabilities": crate::biomeos::MEASUREMENT_CAPABILITIES,
        "uptime_seconds": uptime_secs(),
    })
}

/// `capability.list` — advertise the measurement domain and capabilities.
pub(super) fn capability_list() -> Value {
    serde_json::json!({
        "domain": crate::biomeos::MEASUREMENT_DOMAIN,
        "capabilities": crate::biomeos::MEASUREMENT_CAPABILITIES,
    })
}

/// `lifecycle.status` — identity, version, and operational state.
pub(super) fn lifecycle_status() -> Value {
    serde_json::json!({
        "name": crate::biomeos::FAMILY_ID,
        "family_id": crate::biomeos::FAMILY_ID,
        "version": env!("CARGO_PKG_VERSION"),
        "capabilities": crate::biomeos::MEASUREMENT_CAPABILITIES,
        "uptime_seconds": uptime_secs(),
    })
}

/// Liveness probe — answers immediately if the process is alive.
///
/// Absorbed from wetSpring V121 / airSpring V0.8.8 / healthSpring V30
/// health probe pattern. biomeOS uses this to distinguish "process is
/// alive" from "process is ready to serve requests".
pub(super) fn health_liveness() -> Value {
    serde_json::json!({
        "status": "alive",
        "primal": crate::biomeos::FAMILY_ID,
    })
}

/// Readiness probe — confirms the server can accept and process requests.
///
/// Returns capability count and uptime. A positive capability count
/// indicates all dispatch routes are wired and ready.
pub(super) fn health_readiness() -> Value {
    serde_json::json!({
        "status": "ready",
        "primal": crate::biomeos::FAMILY_ID,
        "version": env!("CARGO_PKG_VERSION"),
        "capabilities_ready": crate::biomeos::MEASUREMENT_CAPABILITIES.len(),
        "uptime_seconds": uptime_secs(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_check_is_healthy() {
        init_start_time();
        let v = health_check();
        assert_eq!(v["status"], "healthy");
    }

    #[test]
    fn capability_list_has_domain() {
        let v = capability_list();
        assert!(v["domain"].is_string());
        assert!(v["capabilities"].is_array());
    }

    #[test]
    fn lifecycle_status_has_version() {
        init_start_time();
        let v = lifecycle_status();
        assert!(v["version"].is_string());
    }

    #[test]
    fn liveness_is_alive() {
        let v = health_liveness();
        assert_eq!(v["status"], "alive");
    }

    #[test]
    fn readiness_is_ready() {
        init_start_time();
        let v = health_readiness();
        assert_eq!(v["status"], "ready");
    }
}
