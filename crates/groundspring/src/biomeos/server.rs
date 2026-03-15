// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! JSON-RPC 2.0 server for the groundSpring `UniBin` `server` subcommand.
//!
//! Binds a Unix domain socket in the biomeOS socket directory and accepts
//! newline-delimited JSON-RPC requests. Each request is dispatched to the
//! provided handler function.
//!
//! biomeOS germinates groundSpring by invoking `groundspring server`, which
//! calls [`bind_socket`] → [`serve`]. The deploy graph handles ordering,
//! Songbird handles IPC mesh, and the Neural API handles capability routing.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use serde_json::Value;

use super::{BiomeOsError, FAMILY_ID, Result};

/// Resolve the socket path for this primal.
///
/// Convention: `$XDG_RUNTIME_DIR/biomeos/groundspring-{FAMILY_ID}.sock`
///
/// Fallback chain (matches `SPRING_AS_NICHE_DEPLOYMENT_STANDARD`):
/// 1. `GROUNDSPRING_SOCKET` env var (explicit override)
/// 2. `$BIOMEOS_SOCKET_DIR/groundspring-{family}.sock`
/// 3. `$XDG_RUNTIME_DIR/biomeos/groundspring-{family}.sock`
/// 4. `/run/user/{uid}/biomeos/groundspring-{family}.sock`
/// 5. `/tmp/groundspring-{family}.sock`
#[must_use]
pub fn socket_path() -> PathBuf {
    if let Ok(explicit) = std::env::var("GROUNDSPRING_SOCKET") {
        return PathBuf::from(explicit);
    }

    let family = std::env::var("FAMILY_ID").unwrap_or_else(|_| FAMILY_ID.to_string());
    let filename = format!("groundspring-{family}.sock");

    if let Ok(dir) = std::env::var("BIOMEOS_SOCKET_DIR") {
        return PathBuf::from(dir).join(filename);
    }

    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        let dir = PathBuf::from(xdg).join("biomeos");
        if dir.is_dir() || std::fs::create_dir_all(&dir).is_ok() {
            return dir.join(filename);
        }
    }

    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::MetadataExt;
        if let Ok(meta) = std::fs::metadata("/proc/self") {
            let dir = PathBuf::from(format!("/run/user/{}/biomeos", meta.uid()));
            if dir.is_dir() || std::fs::create_dir_all(&dir).is_ok() {
                return dir.join(filename);
            }
        }
    }

    PathBuf::from("/tmp").join(filename)
}

/// Bind a Unix domain socket at the resolved path.
///
/// Removes any stale socket file first.
///
/// # Errors
///
/// Returns `Err` if the socket cannot be bound.
#[cfg(unix)]
pub fn bind_socket() -> Result<(std::os::unix::net::UnixListener, PathBuf)> {
    let path = socket_path();
    let _ = std::fs::remove_file(&path);

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| BiomeOsError(format!("create socket dir: {e}")))?;
    }

    let listener = std::os::unix::net::UnixListener::bind(&path)
        .map_err(|e| BiomeOsError(format!("bind {}: {e}", path.display())))?;

    Ok((listener, path))
}

/// Handle a single already-accepted JSON-RPC connection.
///
/// Reads one newline-delimited request, calls `handler(method, params)`,
/// and writes the response. Used by the primal binary's accept loop.
#[cfg(unix)]
pub fn serve_one<F>(stream: &std::os::unix::net::UnixStream, handler: F)
where
    F: Fn(&str, &Value) -> std::result::Result<Value, String>,
{
    if let Err(e) = handle_connection(stream, &handler) {
        log::error!("connection error: {e}");
    }
}

/// Handle a single JSON-RPC connection.
#[cfg(unix)]
fn handle_connection<F>(stream: &std::os::unix::net::UnixStream, handler: &F) -> Result<()>
where
    F: Fn(&str, &Value) -> std::result::Result<Value, String>,
{
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(30)));
    let _ = stream.set_write_timeout(Some(std::time::Duration::from_secs(10)));

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .map_err(|e| BiomeOsError(format!("read: {e}")))?;

    if line.trim().is_empty() {
        return Ok(());
    }

    let request: Value = serde_json::from_str(line.trim())
        .map_err(|e| BiomeOsError(format!("invalid JSON-RPC: {e}")))?;

    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let method = request.get("method").and_then(Value::as_str).unwrap_or("");
    let params = request
        .get("params")
        .cloned()
        .unwrap_or_else(|| Value::Object(serde_json::Map::new()));

    let response = match handler(method, &params) {
        Ok(result) => serde_json::json!({
            "jsonrpc": "2.0",
            "result": result,
            "id": id,
        }),
        Err(msg) => serde_json::json!({
            "jsonrpc": "2.0",
            "error": { "code": -32000, "message": msg },
            "id": id,
        }),
    };

    let mut response_bytes = response.to_string().into_bytes();
    response_bytes.push(b'\n');

    let writer = reader.get_mut();
    writer
        .write_all(&response_bytes)
        .map_err(|e| BiomeOsError(format!("write: {e}")))?;
    writer
        .flush()
        .map_err(|e| BiomeOsError(format!("flush: {e}")))?;

    Ok(())
}

/// Clean up the socket file on shutdown.
pub fn cleanup_socket(path: &Path) {
    let _ = std::fs::remove_file(path);
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]
mod tests {
    use super::*;

    #[test]
    fn socket_path_respects_explicit_override() {
        temp_env::with_var("GROUNDSPRING_SOCKET", Some("/tmp/test.sock"), || {
            assert_eq!(socket_path(), PathBuf::from("/tmp/test.sock"));
        });
    }

    #[test]
    fn socket_path_falls_back_to_tmp() {
        temp_env::with_vars(
            [
                ("GROUNDSPRING_SOCKET", None::<&str>),
                ("BIOMEOS_SOCKET_DIR", None::<&str>),
                ("XDG_RUNTIME_DIR", None::<&str>),
            ],
            || {
                let path = socket_path();
                assert!(
                    path.to_string_lossy().contains("groundspring-"),
                    "should contain primal name: {path:?}"
                );
            },
        );
    }

    #[test]
    fn socket_path_uses_biomeos_socket_dir() {
        let dir = tempfile::tempdir().unwrap();
        temp_env::with_vars(
            [
                ("GROUNDSPRING_SOCKET", None::<&str>),
                ("BIOMEOS_SOCKET_DIR", Some(dir.path().to_str().unwrap())),
            ],
            || {
                let path = socket_path();
                assert!(path.starts_with(dir.path()));
                assert!(path.to_string_lossy().contains("groundspring-"));
            },
        );
    }
}
