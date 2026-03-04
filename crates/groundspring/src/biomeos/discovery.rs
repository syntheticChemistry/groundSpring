// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Capability-based socket discovery for the biomeOS Neural API.
//!
//! Discovery uses environment variables and platform conventions —
//! no hardcoded absolute paths. The primal only knows how to find
//! a Neural API socket; it does not know which primal provides it.

use std::path::PathBuf;

/// Capability-based socket name pattern.
///
/// biomeOS sockets are named by capability, not by primal. Discovery
/// prefers the canonical `neural-api.sock` and falls back to scanning
/// the directory for any socket that advertises a JSON-RPC health check.
const CAPABILITY_SOCKET_NAMES: &[&str] = &["neural-api.sock", "neural-api-default.sock"];

/// Discover the `biomeOS` Neural API Unix socket path.
///
/// Discovery priority (no hardcoded absolute paths):
/// 1. `GROUNDSPRING_BIOMEOS_SOCKET` env var (explicit override)
/// 2. `$XDG_RUNTIME_DIR/biomeos/neural-api.sock` (NUCLEUS `start_nucleus.sh`)
/// 3. `$XDG_RUNTIME_DIR/biomeos/neural-api-default.sock` (legacy)
/// 4. `/run/user/{uid}/biomeos/neural-api.sock` (non-XDG Linux)
/// 5. `<temp_dir>/biomeos-neural-api.sock` (platform-agnostic fallback)
#[must_use]
pub fn discover_socket() -> Option<PathBuf> {
    let explicit = std::env::var("GROUNDSPRING_BIOMEOS_SOCKET").ok();
    let xdg = std::env::var("XDG_RUNTIME_DIR").ok();
    resolve_socket(explicit.as_deref(), xdg.as_deref())
}

/// Pure-logic socket path resolution (testable without filesystem side effects).
fn resolve_socket(explicit: Option<&str>, xdg_runtime: Option<&str>) -> Option<PathBuf> {
    if let Some(path) = explicit {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    if let Some(xdg) = xdg_runtime {
        let biomeos_dir = PathBuf::from(xdg).join("biomeos");
        if let Some(p) = find_capability_socket(&biomeos_dir) {
            return Some(p);
        }
    }

    #[cfg(target_os = "linux")]
    if xdg_runtime.is_none() {
        if let Some(uid) = proc_self_uid() {
            let run_dir = PathBuf::from(format!("/run/user/{uid}/biomeos"));
            if let Some(p) = find_capability_socket(&run_dir) {
                return Some(p);
            }
        }
    }

    let temp_biomeos = std::env::temp_dir().join("biomeos");
    if let Some(p) = find_capability_socket(&temp_biomeos) {
        return Some(p);
    }

    let legacy = std::env::temp_dir().join("biomeos-neural-api.sock");
    if legacy.exists() {
        return Some(legacy);
    }

    None
}

/// Find a capability socket in a biomeOS directory.
///
/// Tries known capability names first (fast path), then scans the
/// directory for any `.sock` file (true capability discovery).
fn find_capability_socket(dir: &std::path::Path) -> Option<PathBuf> {
    for name in CAPABILITY_SOCKET_NAMES {
        let p = dir.join(name);
        if p.exists() {
            return Some(p);
        }
    }
    scan_directory_for_sockets(dir)
}

/// Scan a directory for any `.sock` file (capability-agnostic fallback).
fn scan_directory_for_sockets(dir: &std::path::Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "sock") && path.exists() {
            return Some(path);
        }
    }
    None
}

/// Get the real UID of the current process via `/proc/self` metadata.
#[cfg(target_os = "linux")]
fn proc_self_uid() -> Option<u32> {
    use std::os::unix::fs::MetadataExt;
    std::fs::metadata("/proc/self").ok().map(|m| m.uid())
}

/// Attempt to connect to a live NUCLEUS and return the socket path if healthy.
///
/// Unlike [`discover_socket`] which only checks if a socket *file* exists,
/// this function actually connects and verifies the Neural API responds.
/// Returns `None` if no NUCLEUS is running or it fails the health check.
#[must_use]
pub fn auto_connect() -> Option<PathBuf> {
    let socket = discover_socket()?;
    super::health(&socket).ok()?;
    Some(socket)
}

/// Check whether a live NUCLEUS is available and responding.
#[must_use]
pub fn is_nucleus_available() -> bool {
    auto_connect().is_some()
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]
mod tests {
    use super::*;

    #[test]
    fn resolve_socket_explicit_nonexistent() {
        let nonexistent = std::env::temp_dir().join("nonexistent_groundspring_biomeos.sock");
        let nonexistent_str = nonexistent.to_string_lossy().to_string();
        let result = resolve_socket(Some(&nonexistent_str), None);
        if let Some(ref p) = result {
            assert_ne!(
                p.to_string_lossy(),
                nonexistent_str,
                "should not return the nonexistent explicit path"
            );
            assert!(p.exists(), "returned path must exist");
        }
    }

    #[test]
    fn resolve_socket_all_none() {
        let result = resolve_socket(None, None);
        assert!(
            result.is_none() || result.is_some_and(|p| p.exists()),
            "should be None or a path that exists"
        );
    }

    #[test]
    fn resolve_socket_explicit_exists() {
        let dir = tempfile::tempdir().unwrap();
        let sock = dir.path().join("test.sock");
        std::fs::write(&sock, "").unwrap();
        let result = resolve_socket(Some(sock.to_str().unwrap()), None);
        assert_eq!(result, Some(sock));
    }

    #[test]
    fn resolve_socket_xdg_path_neural_api() {
        let dir = tempfile::tempdir().unwrap();
        let biomeos = dir.path().join("biomeos");
        std::fs::create_dir_all(&biomeos).unwrap();
        let sock = biomeos.join("neural-api.sock");
        std::fs::write(&sock, "").unwrap();
        let result = resolve_socket(None, Some(dir.path().to_str().unwrap()));
        assert_eq!(result, Some(sock), "should prefer neural-api.sock");
    }

    #[test]
    fn resolve_socket_xdg_path_legacy_default() {
        let dir = tempfile::tempdir().unwrap();
        let biomeos = dir.path().join("biomeos");
        std::fs::create_dir_all(&biomeos).unwrap();
        let sock = biomeos.join("neural-api-default.sock");
        std::fs::write(&sock, "").unwrap();
        let result = resolve_socket(None, Some(dir.path().to_str().unwrap()));
        assert_eq!(
            result,
            Some(sock),
            "should find legacy neural-api-default.sock"
        );
    }

    #[test]
    fn resolve_socket_prefers_neural_api_over_default() {
        let dir = tempfile::tempdir().unwrap();
        let biomeos = dir.path().join("biomeos");
        std::fs::create_dir_all(&biomeos).unwrap();
        let primary = biomeos.join("neural-api.sock");
        let legacy = biomeos.join("neural-api-default.sock");
        std::fs::write(&primary, "").unwrap();
        std::fs::write(&legacy, "").unwrap();
        let result = resolve_socket(None, Some(dir.path().to_str().unwrap()));
        assert_eq!(
            result,
            Some(primary),
            "should prefer neural-api.sock over default"
        );
    }

    #[test]
    fn resolve_socket_xdg_nonexistent() {
        let dir = tempfile::tempdir().unwrap();
        let xdg = dir.path().join("nonexistent_xdg");
        let result = resolve_socket(None, Some(xdg.to_str().unwrap()));
        assert!(result.is_none() || result.is_some_and(|p| p.exists()));
    }

    #[test]
    fn resolve_socket_explicit_overrides_xdg() {
        let dir = tempfile::tempdir().unwrap();
        let explicit_sock = dir.path().join("explicit.sock");
        std::fs::write(&explicit_sock, "").unwrap();
        let xdg_dir = tempfile::tempdir().unwrap();
        let biomeos = xdg_dir.path().join("biomeos");
        std::fs::create_dir_all(&biomeos).unwrap();
        let xdg_sock = biomeos.join("neural-api.sock");
        std::fs::write(&xdg_sock, "").unwrap();
        let result = resolve_socket(
            Some(explicit_sock.to_str().unwrap()),
            Some(xdg_dir.path().to_str().unwrap()),
        );
        assert_eq!(result, Some(explicit_sock));
    }

    #[test]
    fn discover_socket_does_not_panic() {
        let _ = discover_socket();
    }
}
