// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Unit tests for the biomeOS client (kept out of `mod.rs` for line budget).

use serde_json::Value;

use super::protocol::build_request;
use super::{
    FAMILY_ID, MEASUREMENT_CAPABILITIES, MEASUREMENT_MAPPINGS, capability_call, compute_execute,
    compute_submit, deregister_capabilities, direct_rpc_call, health, is_enabled,
    register_capabilities, storage_get, storage_put,
};

use tempfile::tempdir;

use std::sync::atomic::{AtomicU64, Ordering};

/// Atomic counter for unique test socket paths (ludoSpring V28 pattern).
/// Prevents CI flakiness from parallel test socket collisions.
static TEST_SOCKET_ID: AtomicU64 = AtomicU64::new(0);

fn unique_test_socket(label: &str) -> std::path::PathBuf {
    let id = TEST_SOCKET_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("gs_test_{label}_{id}.sock"))
}

#[test]
fn is_enabled_unset_is_false() {
    temp_env::with_var("GROUNDSPRING_COMPUTE_PROVIDER", None::<&str>, || {
        assert!(!is_enabled());
    });
}

#[test]
fn is_enabled_accepts_biomeos_true_and_one() {
    for val in ["biomeos", "BIOMEOS", "true", "TRUE", "1"] {
        temp_env::with_var("GROUNDSPRING_COMPUTE_PROVIDER", Some(val), || {
            assert!(is_enabled(), "expected enabled for {val:?}");
        });
    }
}

#[test]
fn is_enabled_rejects_false_zero_and_other() {
    for val in ["false", "0", "other"] {
        temp_env::with_var("GROUNDSPRING_COMPUTE_PROVIDER", Some(val), || {
            assert!(!is_enabled(), "expected disabled for {val:?}");
        });
    }
}

#[test]
fn capability_call_request_format() {
    let cap = "measurement.anderson_validation";
    let (cap_part, op_part) = cap.split_once('.').unwrap();
    let request = build_request(
        "capability.call",
        &serde_json::json!({
            "capability": cap_part,
            "operation": op_part,
            "args": {"n_sites": 10000},
            "family_id": FAMILY_ID,
        }),
    );
    let v: Value = serde_json::from_str(&request).unwrap();
    assert_eq!(v["method"], "capability.call");
    assert_eq!(v["params"]["capability"], "measurement");
    assert_eq!(v["params"]["operation"], "anderson_validation");
    assert_eq!(v["params"]["family_id"], "groundspring");
}

#[test]
fn measurement_capabilities_non_empty() {
    assert!(!MEASUREMENT_CAPABILITIES.is_empty());
    for cap in MEASUREMENT_CAPABILITIES {
        assert!(
            cap.starts_with("measurement."),
            "all caps should be in measurement namespace: {cap}"
        );
    }
}

#[test]
fn measurement_mappings_complete() {
    assert_eq!(
        MEASUREMENT_MAPPINGS.len(),
        MEASUREMENT_CAPABILITIES.len(),
        "every capability needs a semantic mapping"
    );
}

#[test]
fn register_capabilities_nonexistent_socket_errors() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("register_missing.sock");
    let err = register_capabilities(&path).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("no capabilities registered") || msg.contains("biomeOS connect"),
        "should fail with clear message: {msg}"
    );
}

#[test]
fn deregister_capabilities_nonexistent_socket_returns_zero() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("deregister_missing.sock");
    let count = deregister_capabilities(&path).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn health_nonexistent_socket_errors() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("health_missing.sock");
    let err = health(&path).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("biomeOS connect") || msg.contains("invalid socket"));
}

#[test]
fn storage_put_nonexistent_socket_errors() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("storage_put_missing.sock");
    let err = storage_put(&path, "k", "v").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("biomeOS connect") || msg.contains("invalid socket"));
}

#[test]
fn storage_get_nonexistent_socket_errors() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("storage_get_missing.sock");
    let err = storage_get(&path, "k").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("biomeOS connect") || msg.contains("invalid socket"));
}

#[test]
fn compute_execute_nonexistent_socket_errors() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("compute_exec_missing.sock");
    let err = compute_execute(&path, "op", "{}").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("biomeOS connect") || msg.contains("invalid socket"));
}

#[test]
fn compute_submit_nonexistent_socket_errors() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("compute_submit_missing.sock");
    let err = compute_submit(&path, "op", "{}").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("biomeOS connect") || msg.contains("invalid socket"));
}

#[test]
fn capability_call_nonexistent_socket_errors() {
    let path = unique_test_socket("cap");
    let err = capability_call(&path, "science.test", "{}").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("biomeOS connect") || msg.contains("invalid socket"));
}

#[test]
fn direct_rpc_call_nonexistent_socket_errors() {
    let path = unique_test_socket("rpc");
    let err = direct_rpc_call(&path, "compute", "health", "{}").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("biomeOS connect") || msg.contains("invalid socket"));
}

#[test]
fn unique_test_socket_paths_are_unique() {
    let a = unique_test_socket("uniq");
    let b = unique_test_socket("uniq");
    assert_ne!(a, b, "each call should produce a distinct path");
}
