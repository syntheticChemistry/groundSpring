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
use tracing::{error, info, warn};

static SHUTDOWN: AtomicBool = AtomicBool::new(false);

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    let args: Vec<String> = std::env::args().collect();
    let subcommand = args.get(1).map(String::as_str);

    match subcommand {
        Some("server") => cmd_server(),
        Some("status") => cmd_status(),
        Some("version" | "--version" | "-V") => cmd_version(),
        Some("help" | "--help" | "-h") | None => cmd_help(),
        Some(other) => {
            error!(subcommand = other, "unknown subcommand");
            cmd_help()
        }
    }
}

// ─── help ────────────────────────────────────────────────────────────────────

fn cmd_help() -> ExitCode {
    println!(
        "groundspring {} — measurement noise characterization primal",
        env!("CARGO_PKG_VERSION")
    );
    println!();
    println!("USAGE:");
    println!("    groundspring <COMMAND>");
    println!();
    println!("COMMANDS:");
    println!("    server     Start JSON-RPC 2.0 server (germination mode for biomeOS)");
    println!("    status     Health check against running instance");
    println!("    version    Print version, capabilities, and build info");
    println!("    help       Print this help message");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help       Print help");
    println!("    -V, --version    Print version");
    println!();
    println!("LICENSE: AGPL-3.0-or-later");
    ExitCode::SUCCESS
}

// ─── server ──────────────────────────────────────────────────────────────────

fn cmd_server() -> ExitCode {
    dispatch::init_start_time();
    info!(version = env!("CARGO_PKG_VERSION"), "starting server");

    let (listener, socket_path) = match biomeos::server::bind_socket() {
        Ok(pair) => pair,
        Err(e) => {
            error!(error = %e, "failed to bind socket");
            return ExitCode::FAILURE;
        }
    };
    info!(path = %socket_path.display(), "listening");

    if let Some(neural_socket) = biomeos::auto_connect() {
        info!("Neural API found, registering via announce");
        match biomeos::announce_or_register(&neural_socket) {
            Ok(n) => info!(count = n, "registered capabilities"),
            Err(e) => warn!(error = %e, "registration failed (non-fatal)"),
        }

        match groundspring::provenance::start_session(&neural_socket, "server_lifecycle") {
            Ok(sid) => info!(session_id = %sid, "provenance session started"),
            Err(e) => warn!(error = %e, "provenance session skipped"),
        }
    } else {
        info!("no Neural API — sovereign mode");
    }

    install_signal_handler();

    let _ = listener.set_nonblocking(true);

    info!("ready — accepting JSON-RPC connections");

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
                error!(error = %e, "accept error");
            }
        }
    }

    info!("shutting down");
    if let Some(neural_socket) = biomeos::auto_connect() {
        let _ = biomeos::deregister_capabilities(&neural_socket);
        info!("deregistered capabilities");
    }
    biomeos::server::cleanup_socket(&socket_path);
    info!("socket cleaned up");

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
        error!(path = %socket.display(), "server not running (no socket)");
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
                    match serde_json::to_string_pretty(&v["result"]) {
                        Ok(pretty) => println!("{pretty}"),
                        Err(e) => error!("failed to format result: {e}"),
                    }
                    return ExitCode::SUCCESS;
                }
                error!("invalid response from server");
                ExitCode::FAILURE
            }
            Err(e) => {
                error!(path = %socket.display(), error = %e, "cannot connect");
                ExitCode::FAILURE
            }
        }
    }

    #[cfg(not(unix))]
    {
        error!("status subcommand requires Unix");
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
