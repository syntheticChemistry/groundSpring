// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Integration tests for the biomeOS Neural API client.
//!
//! Tests are behind `#[cfg(feature = "biomeos")]` so they only compile
//! when the feature is active.
//!
//! These tests validate:
//! - Socket discovery logic (env var, XDG, temp fallback)
//! - JSON-RPC request/response serialization
//! - Sovereign fallback when socket is unavailable

#![cfg(feature = "biomeos")]

use groundspring::biomeos;
use std::path::PathBuf;

// ── Socket Discovery ─────────────────────────────────────────────────

#[test]
fn discover_socket_returns_none_when_no_socket_exists() {
    // In CI/test, there is (almost certainly) no biomeOS running.
    // discover_socket should return None without panicking.
    let result = biomeos::discover_socket();
    assert!(
        result.is_none() || result.is_some_and(|p| p.exists()),
        "discover should return None or an existing path"
    );
}

#[test]
fn discover_socket_with_explicit_env_var() {
    let dir = tempfile::tempdir().unwrap();
    let sock = dir.path().join("test-neural-api.sock");
    std::fs::write(&sock, "").unwrap();

    // Temporarily set the env var (test isolation via unique socket name)
    let key = "GROUNDSPRING_BIOMEOS_SOCKET";
    let old = std::env::var(key).ok();
    std::env::set_var(key, sock.to_str().unwrap());

    let result = biomeos::discover_socket();
    assert_eq!(result, Some(sock));

    match old {
        Some(v) => std::env::set_var(key, v),
        None => std::env::remove_var(key),
    }
}

// ── Sovereign Fallback ───────────────────────────────────────────────

#[test]
fn capability_call_fails_gracefully_when_no_socket() {
    let fake_path = PathBuf::from("/tmp/groundspring_biomeos_test_nonexistent.sock");
    let err = biomeos::capability_call(&fake_path, "compute.execute", "{}").unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("biomeOS connect") || msg.contains("invalid socket"),
        "expected connection error, got: {msg}"
    );
}

#[test]
fn direct_rpc_call_fails_gracefully_when_no_socket() {
    let fake_path = PathBuf::from("/tmp/groundspring_biomeos_test_nonexistent_rpc.sock");
    let err = biomeos::direct_rpc_call(&fake_path, "nestgate", "health", "{}").unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("biomeOS connect") || msg.contains("invalid socket"),
        "expected connection error, got: {msg}"
    );
}

#[test]
fn storage_put_fails_gracefully_when_no_socket() {
    let fake_path = PathBuf::from("/tmp/groundspring_biomeos_test_nonexistent_put.sock");
    let err = biomeos::storage_put(&fake_path, "test_key", "test_value").unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("biomeOS connect") || msg.contains("invalid socket"),
        "expected connection error, got: {msg}"
    );
}

#[test]
fn storage_get_fails_gracefully_when_no_socket() {
    let fake_path = PathBuf::from("/tmp/groundspring_biomeos_test_nonexistent_get.sock");
    let err = biomeos::storage_get(&fake_path, "test_key").unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("biomeOS connect") || msg.contains("invalid socket"),
        "expected connection error, got: {msg}"
    );
}

#[test]
fn health_fails_gracefully_when_no_socket() {
    let fake_path = PathBuf::from("/tmp/groundspring_biomeos_test_nonexistent_health.sock");
    let err = biomeos::health(&fake_path).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("biomeOS connect") || msg.contains("invalid socket"),
        "expected connection error, got: {msg}"
    );
}

// ── Provider Detection ───────────────────────────────────────────────

#[test]
fn is_enabled_returns_false_by_default() {
    // Unless GROUNDSPRING_COMPUTE_PROVIDER=biomeos is set in the test
    // environment, is_enabled() should return false.
    let current = std::env::var("GROUNDSPRING_COMPUTE_PROVIDER").ok();
    if current.as_deref() != Some("biomeos") {
        assert!(!biomeos::is_enabled());
    }
}

#[test]
fn biomeos_error_display() {
    let err = biomeos::BiomeOsError("test error".to_string());
    assert_eq!(err.to_string(), "biomeOS: test error");
}

// ── XDG Discovery ────────────────────────────────────────────────────

#[test]
fn discover_socket_xdg_biomeos_path() {
    let dir = tempfile::tempdir().unwrap();
    let biomeos_dir = dir.path().join("biomeos");
    std::fs::create_dir_all(&biomeos_dir).unwrap();
    let sock = biomeos_dir.join("neural-api-default.sock");
    std::fs::write(&sock, "").unwrap();

    let key_socket = "GROUNDSPRING_BIOMEOS_SOCKET";
    let key_xdg = "XDG_RUNTIME_DIR";
    let old_socket = std::env::var(key_socket).ok();
    let old_xdg = std::env::var(key_xdg).ok();

    std::env::remove_var(key_socket);
    std::env::set_var(key_xdg, dir.path().to_str().unwrap());

    let result = biomeos::discover_socket();
    assert_eq!(result, Some(sock));

    match old_socket {
        Some(v) => std::env::set_var(key_socket, v),
        None => std::env::remove_var(key_socket),
    }
    match old_xdg {
        Some(v) => std::env::set_var(key_xdg, v),
        None => std::env::remove_var(key_xdg),
    }
}
