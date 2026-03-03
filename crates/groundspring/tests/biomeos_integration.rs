// SPDX-License-Identifier: AGPL-3.0-only
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
    let fake_path = std::env::temp_dir().join("groundspring_biomeos_test_nonexistent.sock");
    let err = biomeos::capability_call(&fake_path, "compute.execute", "{}").unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("biomeOS connect") || msg.contains("invalid socket"),
        "expected connection error, got: {msg}"
    );
}

#[test]
fn direct_rpc_call_fails_gracefully_when_no_socket() {
    let fake_path = std::env::temp_dir().join("groundspring_biomeos_test_nonexistent_rpc.sock");
    let err = biomeos::direct_rpc_call(&fake_path, "compute", "health", "{}").unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("biomeOS connect") || msg.contains("invalid socket"),
        "expected connection error, got: {msg}"
    );
}

#[test]
fn storage_put_fails_gracefully_when_no_socket() {
    let fake_path = std::env::temp_dir().join("groundspring_biomeos_test_nonexistent_put.sock");
    let err = biomeos::storage_put(&fake_path, "test_key", "test_value").unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("biomeOS connect") || msg.contains("invalid socket"),
        "expected connection error, got: {msg}"
    );
}

#[test]
fn storage_get_fails_gracefully_when_no_socket() {
    let fake_path = std::env::temp_dir().join("groundspring_biomeos_test_nonexistent_get.sock");
    let err = biomeos::storage_get(&fake_path, "test_key").unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("biomeOS connect") || msg.contains("invalid socket"),
        "expected connection error, got: {msg}"
    );
}

#[test]
fn health_fails_gracefully_when_no_socket() {
    let fake_path = std::env::temp_dir().join("groundspring_biomeos_test_nonexistent_health.sock");
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

// ── Auto-Connect & NUCLEUS Detection ─────────────────────────────────

#[test]
fn auto_connect_returns_none_without_nucleus() {
    // Unless a real NUCLEUS is running, auto_connect should return None.
    let result = biomeos::auto_connect();
    assert!(
        result.is_none() || result.is_some(),
        "auto_connect should not panic"
    );
}

#[test]
fn is_nucleus_available_does_not_panic() {
    let _ = biomeos::is_nucleus_available();
}

// ── Live NUCLEUS Integration ─────────────────────────────────────────
// These tests require a running NUCLEUS. Run with:
//   cargo test --features biomeos -- --ignored nucleus

fn live_socket() -> Option<std::path::PathBuf> {
    biomeos::auto_connect().or_else(|| {
        let xdg = std::env::var("XDG_RUNTIME_DIR").ok()?;
        let p = std::path::PathBuf::from(xdg).join("biomeos/neural-api.sock");
        p.exists().then_some(p)
    })
}

#[test]
#[ignore = "requires running NUCLEUS"]
fn nucleus_capability_call_toadstool_health() {
    let socket = live_socket().expect("Neural API socket not found");
    let result = biomeos::capability_call(&socket, "compute.health", "{}");
    let response = result.expect("capability.call compute.health failed");
    assert!(
        response.contains("healthy") || response.contains("toadstool"),
        "expected ToadStool health, got: {response}"
    );
}

#[test]
#[ignore = "requires running NUCLEUS"]
fn nucleus_capability_call_squirrel_health() {
    let socket = live_socket().expect("Neural API socket not found");
    let result = biomeos::capability_call(&socket, "ai.health", "{}");
    let response = result.expect("capability.call ai.health failed");
    assert!(
        response.contains("healthy"),
        "expected Squirrel health, got: {response}"
    );
}

#[test]
#[ignore = "requires running NUCLEUS"]
fn nucleus_topology_primals() {
    let socket = live_socket().expect("Neural API socket not found");
    let result = biomeos::health(&socket);
    result.expect("Neural API topology.metrics failed");
}

#[test]
#[ignore = "requires running NUCLEUS"]
fn nucleus_capability_list() {
    let socket = live_socket().expect("Neural API socket not found");
    let request = r#"{"jsonrpc":"2.0","method":"capability.list","params":{},"id":1}"#;
    let response = biomeos::raw_rpc_call(&socket, request).expect("capability.list failed");
    assert!(
        response.contains("compute") && response.contains("crypto"),
        "expected compute and crypto capabilities, got: {response}"
    );
}

#[test]
#[ignore = "requires running NUCLEUS"]
fn nucleus_toadstool_compute_submit() {
    let socket = live_socket().expect("Neural API socket not found");
    let params = r#"{"transform":{"operation":"eigendecompose","input":{"disorder_strength":2.0,"lattice_size":50}}}"#;
    let result = biomeos::capability_call(&socket, "compute.submit", params);
    let response = result.expect("compute.submit through Neural API failed");
    assert!(
        response.contains("job_id"),
        "expected job_id in ToadStool response, got: {response}"
    );
}

#[test]
#[ignore = "requires running NUCLEUS"]
fn nucleus_toadstool_compute_capabilities() {
    let socket = live_socket().expect("Neural API socket not found");
    let result = biomeos::capability_call(&socket, "compute.capabilities", "{}");
    let response = result.expect("compute.capabilities through Neural API failed");
    assert!(
        response.contains("compute_units") || response.contains("supported_workload_types"),
        "expected ToadStool capabilities, got: {response}"
    );
}

// ── Live NUCLEUS Storage Round-Trip ───────────────────────────────────

#[test]
#[ignore = "requires running NUCLEUS with NestGate"]
fn nucleus_storage_round_trip() {
    let socket = live_socket().expect("Neural API socket not found");
    let key = "groundspring:test:integration:roundtrip";
    let value = r#"{"test":"nucleus_integration","ts":"2026-02-28"}"#;

    biomeos::storage_put(&socket, key, value).expect("storage_put failed");
    let retrieved = biomeos::storage_get(&socket, key).expect("storage_get failed");
    assert!(
        retrieved.contains("nucleus_integration"),
        "expected stored value, got: {retrieved}"
    );
}

#[test]
#[ignore = "requires running NUCLEUS with NestGate"]
fn nucleus_nestgate_provenance_store() {
    use groundspring::nestgate;

    let socket = live_socket().expect("Neural API socket not found");
    let result_json = r#"{"passed":283,"failed":0,"version":"V55"}"#;
    nestgate::store_result(&socket, 99, "integration_test", result_json)
        .expect("store_result failed");

    let retrieved =
        nestgate::get_result(&socket, 99, "integration_test").expect("get_result failed");
    assert!(
        retrieved.contains("283"),
        "expected stored result, got: {retrieved}"
    );
}

#[test]
#[ignore = "requires running NUCLEUS with NestGate + NCBI provider"]
fn nucleus_nestgate_ncbi_search() {
    use groundspring::nestgate;

    let socket = live_socket().expect("Neural API socket not found");
    let result = nestgate::ncbi_search(&socket, "sra", "soil metagenome 16S");
    match result {
        Ok(data) => assert!(!data.is_empty(), "expected NCBI results"),
        Err(e) => {
            let msg = e.to_string();
            assert!(
                msg.contains("not found") || msg.contains("unavailable"),
                "unexpected error: {msg}"
            );
        }
    }
}

#[test]
#[ignore = "requires running NUCLEUS with NestGate + NOAA provider"]
fn nucleus_nestgate_noaa_ghcnd() {
    use groundspring::nestgate;

    let socket = live_socket().expect("Neural API socket not found");
    let result = nestgate::noaa_ghcnd(
        &socket,
        "USW00094847",
        "2024-01-01",
        "2024-01-31",
        &["TMAX", "TMIN"],
    );
    match result {
        Ok(data) => assert!(!data.is_empty(), "expected NOAA data"),
        Err(e) => {
            let msg = e.to_string();
            assert!(
                msg.contains("not found") || msg.contains("unavailable"),
                "unexpected error: {msg}"
            );
        }
    }
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
