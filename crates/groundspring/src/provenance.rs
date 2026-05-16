// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Provenance Trio integration for groundSpring server lifecycle.
//!
//! Convenience wrappers around `capability_call` for the Provenance Trio
//! (`rhizoCrypt` + `LoamSpine` + `sweetGrass`). All interactions are
//! capability-based — groundSpring never names the trio primals directly.
//! biomeOS routes `provenance.*` and `contribution.*` to whichever primals
//! provide those capabilities at runtime.
//!
//! Used by the `groundspring server` binary during startup (session create)
//! and shutdown (dehydrate + attribute). Deploy graphs can also wire these
//! as graph nodes with `fallback = "skip"`.

use std::path::Path;

use crate::biomeos::{self, Result};

/// Start a provenance session for a groundSpring server run.
///
/// Calls `provenance.session_create` via capability routing. Returns the
/// session ID on success.
///
/// # Errors
///
/// Returns `Err` if the Provenance Trio is unavailable (non-fatal for
/// sovereign operation).
pub fn start_session(socket: &Path, experiment_id: &str) -> Result<String> {
    let params = serde_json::json!({
        "agent": biomeos::FAMILY_ID,
        "experiment_id": experiment_id,
        "version": env!("CARGO_PKG_VERSION"),
        "family_id": biomeos::FAMILY_ID,
    })
    .to_string();
    biomeos::capability_call(socket, "provenance.session_create", &params)
}

/// Commit (dehydrate) a provenance session after validation completes.
///
/// Calls `provenance.session_dehydrate` which triggers rhizoCrypt to
/// dehydrate the session, `LoamSpine` to append to permanent storage,
/// and sweetGrass to record attribution.
///
/// # Errors
///
/// Returns `Err` if the Provenance Trio is unavailable.
pub fn commit_session(socket: &Path, session_id: &str, summary_json: &str) -> Result<String> {
    let params = serde_json::json!({
        "session_id": session_id,
        "summary": summary_json,
        "agent": biomeos::FAMILY_ID,
        "family_id": biomeos::FAMILY_ID,
    })
    .to_string();
    biomeos::capability_call(socket, "provenance.session_dehydrate", &params)
}

/// Record attribution for a dehydrated session.
///
/// Calls `contribution.recordDehydration` so sweetGrass can track which
/// agent contributed what to the provenance chain.
///
/// # Errors
///
/// Returns `Err` if the attribution service is unavailable.
pub fn record_attribution(socket: &Path, session_id: &str, contribution: &str) -> Result<String> {
    let params = serde_json::json!({
        "session_id": session_id,
        "contribution": contribution,
        "agent": biomeos::FAMILY_ID,
        "family_id": biomeos::FAMILY_ID,
    })
    .to_string();
    biomeos::capability_call(socket, "contribution.recordDehydration", &params)
}

/// Store a validation result in the provenance chain via `storage.put`.
///
/// Routes through capability-based storage — biomeOS resolves to the
/// active `NestGate` or equivalent storage provider at runtime.
///
/// # Errors
///
/// Returns `Err` if the storage provider is unavailable.
pub fn store_result(socket: &Path, key: &str, result_json: &str) -> Result<()> {
    biomeos::storage_put(socket, key, result_json)
}

/// Execute a complete provenance lifecycle for a validation run.
///
/// Prefers `nest.store` signal dispatch (Wave 17) which collapses the
/// 4-step sequence into a single orchestrated call. Falls back to the
/// legacy sequential pattern if signal dispatch is unavailable.
///
/// Signal path: `dispatch("nest.store", { content, author, metadata })`
/// Legacy path: session → store → commit → attribution (4 calls)
///
/// Each step gracefully degrades: if the trio is unavailable, the validation
/// still succeeds locally.
///
/// # Errors
///
/// Returns `Err` if session creation fails (both signal and legacy paths).
pub fn run_lifecycle(socket: &Path, experiment_id: &str, result_json: &str) -> Result<String> {
    #[cfg(feature = "biomeos")]
    {
        let metadata = serde_json::json!({
            "experiment_id": experiment_id,
            "agent": biomeos::FAMILY_ID,
            "version": env!("CARGO_PKG_VERSION"),
        });
        match crate::ipc::nestgate::nest_store_dispatch(
            result_json.as_bytes(),
            Some(biomeos::FAMILY_ID),
            Some(&metadata),
        ) {
            Ok(Some(result)) => {
                let session_id = result
                    .get("session_commit")
                    .and_then(|v| v.as_str())
                    .unwrap_or("signal-dispatched")
                    .to_string();
                tracing::info!(
                    session_id = %session_id,
                    "provenance lifecycle via nest.store signal"
                );
                return Ok(session_id);
            }
            Ok(None) | Err(_) => {
                tracing::debug!("nest.store signal unavailable, using legacy provenance path");
            }
        }
    }

    run_lifecycle_legacy(socket, experiment_id, result_json)
}

/// Legacy 4-step provenance lifecycle (pre-Wave 17).
fn run_lifecycle_legacy(socket: &Path, experiment_id: &str, result_json: &str) -> Result<String> {
    let session_id = start_session(socket, experiment_id)?;

    let result_key = format!("{}/{experiment_id}", biomeos::FAMILY_ID);
    if let Err(e) = store_result(socket, &result_key, result_json) {
        tracing::warn!("provenance store failed (non-fatal): {e}");
    }

    if let Err(e) = commit_session(socket, &session_id, result_json) {
        tracing::warn!("provenance commit failed (non-fatal): {e}");
    }

    let contribution = format!("measurement validation: {experiment_id}");
    if let Err(e) = record_attribution(socket, &session_id, &contribution) {
        tracing::warn!("provenance attribution failed (non-fatal): {e}");
    }

    Ok(session_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_session_fails_without_socket() {
        let path = std::env::temp_dir().join("groundspring_test_prov_nonexistent.sock");
        let err = start_session(&path, "exp001");
        assert!(err.is_err());
    }

    #[test]
    fn commit_session_fails_without_socket() {
        let path = std::env::temp_dir().join("groundspring_test_prov_commit.sock");
        let err = commit_session(&path, "sess-123", "{}");
        assert!(err.is_err());
    }

    #[test]
    fn record_attribution_fails_without_socket() {
        let path = std::env::temp_dir().join("groundspring_test_prov_attr.sock");
        let err = record_attribution(&path, "sess-123", "measurement validation");
        assert!(err.is_err());
    }

    #[test]
    fn store_result_fails_without_socket() {
        let path = std::env::temp_dir().join("groundspring_test_prov_store.sock");
        let err = store_result(&path, "test/key", r#"{"pass":true}"#);
        assert!(err.is_err());
    }

    #[test]
    fn run_lifecycle_fails_without_socket() {
        let path = std::env::temp_dir().join("groundspring_test_prov_lifecycle.sock");
        let err = run_lifecycle(&path, "exp001", r#"{"pass":true}"#);
        assert!(err.is_err());
    }
}
