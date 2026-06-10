// SPDX-License-Identifier: AGPL-3.0-or-later

//! IPC interface for skunkBat audit logging.
//!
//! skunkBat provides ecosystem-wide audit event recording. groundSpring
//! uses skunkBat for:
//! - Forwarding validation events to the audit trail
//! - Recording certification results for provenance
//! - Deploy graph audit logging (JH-5 forwarding)
//!
//! When Phase 3 ships, audit events propagate to rhizoCrypt DAG +
//! sweetGrass braid for cross-primal audit forwarding.
//!
//! # Capability surface
//!
//! - `security.audit_log` — query/append audit events

/// Audit service traits via skunkBat (tarpc path).
#[tarpc::service]
pub trait AuditService {
    /// Query audit events from the ring buffer.
    async fn audit_log(since_seq: u64, limit: u64) -> Result<String, String>;
}

/// Emit an audit event via JSON-RPC to skunkBat.
///
/// Sends a `security.audit_log` call with structured event data.
/// Non-blocking in deploy graphs (`fallback = "skip"`); this function
/// returns `Err` if the socket is unreachable rather than panicking.
///
/// # Errors
///
/// Returns `BiomeOsError` if skunkBat is not discovered or the IPC call fails.
#[cfg(feature = "biomeos")]
pub fn emit_audit_event(
    socket: &std::path::Path,
    event_type: &str,
    source: &str,
    payload: &serde_json::Value,
) -> crate::biomeos::Result<serde_json::Value> {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "security.audit_log",
        "params": {
            "event_type": event_type,
            "source": source,
            "payload": payload,
            "timestamp": unix_timestamp(),
        },
        "id": 1
    })
    .to_string();
    let response = crate::biomeos::raw_rpc_call(socket, &request)?;
    crate::biomeos::protocol::extract_rpc_result(&response)
}

/// Emit a validation audit event (convenience wrapper).
///
/// Records the outcome of a validation run (experiment count, pass/fail).
///
/// # Errors
///
/// Returns `BiomeOsError` if skunkBat is not discovered or the IPC call fails.
#[cfg(feature = "biomeos")]
pub fn emit_validation_event(
    socket: &std::path::Path,
    experiment_id: u32,
    passed: u32,
    failed: u32,
) -> crate::biomeos::Result<serde_json::Value> {
    emit_audit_event(
        socket,
        "validation",
        crate::primal_names::SELF_ID,
        &serde_json::json!({
            "experiment_id": experiment_id,
            "passed": passed,
            "failed": failed,
        }),
    )
}

/// Emit a certification audit event (convenience wrapper).
///
/// Records the outcome of a guidestone/certification run.
///
/// # Errors
///
/// Returns `BiomeOsError` if skunkBat is not discovered or the IPC call fails.
#[cfg(feature = "biomeos")]
pub fn emit_certification_event(
    socket: &std::path::Path,
    tier: u8,
    passed: u32,
    failed: u32,
    skipped: u32,
) -> crate::biomeos::Result<serde_json::Value> {
    emit_audit_event(
        socket,
        "certification",
        crate::primal_names::SELF_ID,
        &serde_json::json!({
            "tier": tier,
            "passed": passed,
            "failed": failed,
            "skipped": skipped,
        }),
    )
}

/// Attempt to discover skunkBat and emit an audit event.
///
/// Returns `Ok(None)` if skunkBat is not available (graceful degradation).
/// Returns `Ok(Some(response))` on successful emission.
/// Returns `Err` only on transport/protocol errors after discovery succeeds.
///
/// # Errors
///
/// Returns `BiomeOsError` if the IPC call fails after successful discovery.
#[cfg(feature = "biomeos")]
pub fn try_emit_audit_event(
    event_type: &str,
    source: &str,
    payload: &serde_json::Value,
) -> crate::biomeos::Result<Option<serde_json::Value>> {
    match crate::primal_names::discover_socket(crate::primal_names::roles::AUDIT) {
        Some(socket) => emit_audit_event(&socket, event_type, source, payload).map(Some),
        None => {
            tracing::debug!("skunkBat not discovered — audit event skipped");
            Ok(None)
        }
    }
}

fn unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unix_timestamp_is_reasonable() {
        let ts = unix_timestamp();
        assert!(ts > 1_700_000_000, "timestamp {ts} seems too old");
    }

    #[test]
    fn tarpc_trait_compiles() {
        fn _assert_service<T: AuditService>() {}
    }

    #[test]
    fn audit_role_is_skunkbat() {
        assert_eq!(crate::primal_names::roles::AUDIT, "skunkbat");
    }
}
