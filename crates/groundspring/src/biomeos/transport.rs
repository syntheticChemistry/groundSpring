// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Platform-agnostic transport layer for JSON-RPC over sockets.
//!
//! Unix domain sockets on Unix platforms, TCP elsewhere.

use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use super::{BiomeOsError, Result};

/// Send a JSON-RPC request and read the newline-delimited response.
///
/// Uses Unix domain sockets on Unix platforms and TCP on others.
pub(super) fn rpc_call(socket: &Path, request: &str) -> Result<String> {
    #[cfg(unix)]
    {
        unix_rpc_call(socket, request)
    }
    #[cfg(not(unix))]
    {
        let _ = socket;
        tcp_rpc_call(request)
    }
}

/// Unix domain socket transport.
#[cfg(unix)]
fn unix_rpc_call(socket: &Path, request: &str) -> Result<String> {
    use std::os::unix::net::UnixStream;

    let stream = UnixStream::connect_addr(
        &std::os::unix::net::SocketAddr::from_pathname(socket)
            .map_err(|e| BiomeOsError::Transport(format!("invalid socket path: {e}")))?,
    )
    .map_err(|e| BiomeOsError::Transport(format!("biomeOS connect {}: {e}", socket.display())))?;

    stream
        .set_read_timeout(Some(super::read_timeout()))
        .map_err(|e| BiomeOsError::Transport(format!("set read timeout: {e}")))?;
    stream
        .set_write_timeout(Some(super::connect_timeout()))
        .map_err(|e| BiomeOsError::Transport(format!("set write timeout: {e}")))?;

    send_receive_stream(&stream, request)
}

/// TCP transport for non-Unix platforms.
///
/// Reads the target address from `GROUNDSPRING_BIOMEOS_TCP` (e.g. `"127.0.0.1:9100"`).
#[cfg(not(unix))]
fn tcp_rpc_call(request: &str) -> Result<String> {
    tcp_rpc_call_with_env(request, |k| std::env::var(k).ok())
}

#[cfg(not(unix))]
fn tcp_rpc_call_with_env(request: &str, env: impl Fn(&str) -> Option<String>) -> Result<String> {
    use std::net::TcpStream;

    let addr = env("GROUNDSPRING_BIOMEOS_TCP").ok_or_else(|| {
        BiomeOsError::Transport(
            "biomeOS requires GROUNDSPRING_BIOMEOS_TCP (host:port) on non-Unix platforms"
                .to_string(),
        )
    })?;

    let stream = TcpStream::connect_timeout(
        &addr
            .parse()
            .map_err(|e| BiomeOsError::Transport(format!("invalid TCP address {addr}: {e}")))?,
        super::connect_timeout(),
    )
    .map_err(|e| BiomeOsError::Transport(format!("biomeOS TCP connect {addr}: {e}")))?;

    stream
        .set_read_timeout(Some(super::read_timeout()))
        .map_err(|e| BiomeOsError::Transport(format!("set read timeout: {e}")))?;
    stream
        .set_write_timeout(Some(super::connect_timeout()))
        .map_err(|e| BiomeOsError::Transport(format!("set write timeout: {e}")))?;

    send_receive_stream(&stream, request)
}

/// Write a newline-delimited JSON-RPC request and read the response line.
///
/// Works with any stream where `&S` implements both `Read` and `Write`
/// (e.g. `UnixStream`, `TcpStream`).
fn send_receive_stream<S>(stream: &S, request: &str) -> Result<String>
where
    for<'a> &'a S: std::io::Read + std::io::Write,
{
    let mut writer = std::io::BufWriter::new(stream);
    writer
        .write_all(request.as_bytes())
        .map_err(|e| BiomeOsError::Transport(format!("write to biomeOS: {e}")))?;
    writer
        .write_all(b"\n")
        .map_err(|e| BiomeOsError::Transport(format!("write newline: {e}")))?;
    writer
        .flush()
        .map_err(|e| BiomeOsError::Transport(format!("flush: {e}")))?;
    drop(writer);

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .map_err(|e| BiomeOsError::Transport(format!("read from biomeOS: {e}")))?;

    if line.is_empty() {
        return Err(BiomeOsError::Data(
            "biomeOS returned empty response".to_string(),
        ));
    }

    Ok(line)
}
