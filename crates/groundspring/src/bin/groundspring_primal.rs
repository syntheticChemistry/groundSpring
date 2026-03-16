// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team
#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! groundSpring `UniBin` primal binary.
//!
//! Subcommands:
//! - `server`  — Start JSON-RPC 2.0 server (germination mode for biomeOS)
//! - `status`  — Health check against running instance
//! - `version` — Print version, capabilities, build info

use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};

use groundspring::biomeos;
use groundspring::dispatch;

static SHUTDOWN: AtomicBool = AtomicBool::new(false);

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let subcommand = args.get(1).map(String::as_str);

    match subcommand {
        Some("server") => cmd_server(),
        Some("status") => cmd_status(),
        Some("version") => cmd_version(),
        Some(other) => {
            eprintln!("unknown subcommand: {other}");
            eprintln!("usage: groundspring <server|status|version>");
            ExitCode::FAILURE
        }
        None => {
            eprintln!("usage: groundspring <server|status|version>");
            ExitCode::FAILURE
        }
    }
}

// ─── server ──────────────────────────────────────────────────────────────────

fn cmd_server() -> ExitCode {
    dispatch::init_start_time();
    eprintln!(
        "[groundspring] v{} starting server",
        env!("CARGO_PKG_VERSION")
    );

    // Bind socket
    let (listener, socket_path) = match biomeos::server::bind_socket() {
        Ok(pair) => pair,
        Err(e) => {
            eprintln!("[groundspring] failed to bind socket: {e}");
            return ExitCode::FAILURE;
        }
    };
    eprintln!("[groundspring] listening on {}", socket_path.display());

    // Register with Neural API if available
    if let Some(neural_socket) = biomeos::auto_connect() {
        eprintln!("[groundspring] Neural API found, registering capabilities");
        match biomeos::register_capabilities(&neural_socket) {
            Ok(n) => eprintln!("[groundspring] registered {n} capabilities"),
            Err(e) => eprintln!("[groundspring] registration failed (non-fatal): {e}"),
        }

        // Start provenance session (non-fatal)
        match groundspring::provenance::start_session(&neural_socket, "server_lifecycle") {
            Ok(sid) => eprintln!("[groundspring] provenance session: {sid}"),
            Err(e) => eprintln!("[groundspring] provenance session skipped: {e}"),
        }
    } else {
        eprintln!("[groundspring] no Neural API — sovereign mode");
    }

    // Install signal handler for graceful shutdown
    install_signal_handler();

    // Set a non-blocking timeout so we can check the shutdown flag
    let _ = listener.set_nonblocking(true);

    eprintln!("[groundspring] ready — accepting JSON-RPC connections");

    // Accept loop with shutdown check
    loop {
        if SHUTDOWN.load(Ordering::Relaxed) {
            break;
        }

        match listener.accept() {
            Ok((stream, _)) => {
                let _ = stream.set_nonblocking(false);
                biomeos::server::serve_one(&stream, dispatch::dispatch);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            Err(e) => {
                eprintln!("[groundspring] accept error: {e}");
            }
        }
    }

    // Graceful shutdown
    eprintln!("[groundspring] shutting down");
    if let Some(neural_socket) = biomeos::auto_connect() {
        let _ = biomeos::deregister_capabilities(&neural_socket);
        eprintln!("[groundspring] deregistered capabilities");
    }
    biomeos::server::cleanup_socket(&socket_path);
    eprintln!("[groundspring] socket cleaned up");

    ExitCode::SUCCESS
}

const fn install_signal_handler() {
    // The non-blocking accept loop polls SHUTDOWN every 50ms.
    // OS-delivered SIGTERM terminates the process by default.
    // For fully robust handling, consider the `signal-hook` crate.
    // For now, SHUTDOWN is only set programmatically (e.g., test harness).
}

// ─── status ──────────────────────────────────────────────────────────────────

fn cmd_status() -> ExitCode {
    let socket = biomeos::server::socket_path();
    if !socket.exists() {
        eprintln!(
            "groundspring server not running (no socket at {})",
            socket.display()
        );
        return ExitCode::FAILURE;
    }

    #[cfg(unix)]
    {
        use std::io::{BufRead, BufReader, Write};
        use std::os::unix::net::UnixStream;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "health.check",
            "params": {},
            "id": 1,
        })
        .to_string();

        match UnixStream::connect(&socket) {
            Ok(stream) => {
                let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(5)));
                let _ = stream.set_write_timeout(Some(std::time::Duration::from_secs(5)));
                let mut writer = std::io::BufWriter::new(&stream);
                let _ = writer.write_all(request.as_bytes());
                let _ = writer.write_all(b"\n");
                let _ = writer.flush();
                drop(writer);

                let mut reader = BufReader::new(&stream);
                let mut response = String::new();
                let _ = reader.read_line(&mut response);

                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&response) {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&v["result"]).unwrap_or_default()
                    );
                    return ExitCode::SUCCESS;
                }
                eprintln!("invalid response from server");
                ExitCode::FAILURE
            }
            Err(e) => {
                eprintln!("cannot connect to {}: {e}", socket.display());
                ExitCode::FAILURE
            }
        }
    }

    #[cfg(not(unix))]
    {
        eprintln!("status subcommand requires Unix");
        ExitCode::FAILURE
    }
}

// ─── version ─────────────────────────────────────────────────────────────────

fn cmd_version() -> ExitCode {
    println!("groundspring {}", env!("CARGO_PKG_VERSION"));
    println!("domain: {}", biomeos::MEASUREMENT_DOMAIN);
    println!("capabilities:");
    for cap in biomeos::MEASUREMENT_CAPABILITIES {
        println!("  - {cap}");
    }
    println!("license: AGPL-3.0-or-later");
    println!("family_id: {}", biomeos::FAMILY_ID);
    ExitCode::SUCCESS
}
