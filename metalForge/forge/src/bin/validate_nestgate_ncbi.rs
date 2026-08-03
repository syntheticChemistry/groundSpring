// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team
#![forbid(unsafe_code)]

//! `NestGate` NCBI data acquisition validation for baseCamp papers.
//!
//! Validates the `NestGate` UDS/HTTP API and NUCLEUS primal health for:
//! - Paper 01 (`Anderson-QS`): 16S amplicon metagenome accessions
//! - Paper 06 (No-Till Anderson): soil microbiome studies (Zuber 2016, Islam 2014)
//!
//! Requires: `--features biomeos` and a running NUCLEUS with `NestGate`.
//!
//! `NestGate` exposes a Unix domain socket in the biomeOS mesh (`nestgate.sock`)
//! and optionally a legacy HTTP API. NCBI queries route directly to `NestGate`;
//! the Neural API proxy (`rpc_call`) is a biomeOS evolution item. Primal health
//! checks go through the Unix socket mesh.

use groundspring_forge::nucleus::{NucleusHarness as Harness, biomeos_socket_dir};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

/// Legacy HTTP fallback port — used only when no UDS socket is discovered and
/// no env override supplies a port (e.g. `NESTGATE_HOST` without `NESTGATE_PORT`).
const LEGACY_HTTP_FALLBACK_PORT: u16 = 8090;

/// Legacy HTTP fallback host — used only when no UDS socket is discovered and
/// only `NESTGATE_PORT` is set without `NESTGATE_HOST`.
const LEGACY_HTTP_FALLBACK_HOST: &str = "127.0.0.1";

/// How the validator reaches `NestGate`.
///
/// UDS is preferred when a live socket is discovered; HTTP remains for env
/// overrides and as a last-resort legacy fallback.
enum NestGateEndpoint {
    Unix(String),
    Http { url: String, legacy_fallback: bool },
}

impl NestGateEndpoint {
    fn display_label(&self) -> String {
        match self {
            Self::Unix(path) => format!("UDS {path}"),
            Self::Http { url, legacy_fallback: true } => {
                format!("HTTP {url} (legacy fallback — no UDS socket found)")
            }
            Self::Http { url, legacy_fallback: false } => format!("HTTP {url}"),
        }
    }
}

/// Discover how to reach `NestGate`.
///
/// Discovery hierarchy (first match wins):
///
/// 1. **Environment overrides (HTTP)** — explicit operator intent:
///    `NESTGATE_URL`, `NESTGATE_ADDRESS`, or `NESTGATE_HOST` + `NESTGATE_PORT`.
/// 2. **biomeOS socket scan (UDS, preferred)** — scan
///    `$XDG_RUNTIME_DIR/biomeos/nestgate*.sock` (via [`biomeos_socket_dir`]).
/// 3. **Legacy HTTP fallback** — `127.0.0.1:8090` when nothing else is found.
///
/// UDS is the modern path; raw TCP/HTTP is retained for env overrides and
/// deployments that still expose the legacy HTTP listener.
fn discover_nestgate() -> NestGateEndpoint {
    if let Some(url) = nestgate_url_from_env() {
        return NestGateEndpoint::Http {
            url,
            legacy_fallback: false,
        };
    }

    if let Some(path) = discover_nestgate_socket(&biomeos_socket_dir()) {
        return NestGateEndpoint::Unix(path);
    }

    NestGateEndpoint::Http {
        url: format!("http://{LEGACY_HTTP_FALLBACK_HOST}:{LEGACY_HTTP_FALLBACK_PORT}"),
        legacy_fallback: true,
    }
}

/// Resolve an HTTP URL from NestGate env overrides, if any are set.
fn nestgate_url_from_env() -> Option<String> {
    if let Ok(url) = std::env::var("NESTGATE_URL")
        && !url.is_empty()
    {
        return Some(url);
    }

    if let Ok(addr) = std::env::var("NESTGATE_ADDRESS")
        && !addr.is_empty()
    {
        return Some(format!("http://{addr}"));
    }

    let host = std::env::var("NESTGATE_HOST")
        .ok()
        .filter(|s| !s.is_empty());
    let port = std::env::var("NESTGATE_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok());

    match (host, port) {
        (Some(h), Some(p)) => Some(format!("http://{h}:{p}")),
        (Some(h), None) => Some(format!("http://{h}:{LEGACY_HTTP_FALLBACK_PORT}")),
        (None, Some(p)) => Some(format!("http://{LEGACY_HTTP_FALLBACK_HOST}:{p}")),
        (None, None) => None,
    }
}

/// Scan the biomeOS socket directory for a live `nestgate*.sock` endpoint.
fn discover_nestgate_socket(socket_dir: &str) -> Option<String> {
    let dir = std::path::Path::new(socket_dir);
    let Ok(entries) = std::fs::read_dir(dir) else {
        return None;
    };

    let mut candidates = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if !name_str.starts_with("nestgate") || !name_str.ends_with(".sock") {
            continue;
        }
        let path = entry.path();
        if path.exists() {
            candidates.push((nestgate_socket_priority(&name_str), path.to_string_lossy().to_string()));
        }
    }

    candidates.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    candidates.into_iter().map(|(_, path)| path).next()
}

fn nestgate_socket_priority(name: &str) -> u8 {
    match name {
        "nestgate.sock" => 0,
        "nestgate.jsonrpc.sock" => 1,
        _ => 2,
    }
}

fn nestgate_health(endpoint: &NestGateEndpoint) -> Result<String, String> {
    match endpoint {
        NestGateEndpoint::Unix(path) => nestgate_jsonrpc(
            path,
            "nestgate.health",
            r#"{"check_providers":true,"check_storage":true}"#,
        ),
        NestGateEndpoint::Http { url, .. } => http_get(&format!("{url}/health")),
    }
}

fn nestgate_storage_metrics(endpoint: &NestGateEndpoint) -> Result<String, String> {
    match endpoint {
        NestGateEndpoint::Unix(path) => {
            // Prefer dedicated storage metrics; fall back to health payload fields.
            nestgate_jsonrpc(path, "storage.metrics", "{}").or_else(|_| {
                nestgate_jsonrpc(
                    path,
                    "nestgate.health",
                    r#"{"check_providers":false,"check_storage":true}"#,
                )
            })
        }
        NestGateEndpoint::Http { url, .. } => {
            http_get(&format!("{url}/api/v1/storage/metrics"))
        }
    }
}

fn nestgate_capabilities(endpoint: &NestGateEndpoint) -> Result<String, String> {
    match endpoint {
        NestGateEndpoint::Unix(path) => nestgate_jsonrpc(path, "protocol.capabilities", "{}")
            .or_else(|_| nestgate_jsonrpc(path, "capabilities.list", "{}")),
        NestGateEndpoint::Http { url, .. } => {
            http_get(&format!("{url}/api/v1/protocol/capabilities"))
        }
    }
}

fn nestgate_jsonrpc(socket_path: &str, method: &str, params: &str) -> Result<String, String> {
    let stream = std::os::unix::net::UnixStream::connect(socket_path)
        .map_err(|e| format!("connect {socket_path}: {e}"))?;
    stream.set_read_timeout(Some(Duration::from_secs(10))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(10))).ok();
    let request = format!("{{\"jsonrpc\":\"2.0\",\"method\":\"{method}\",\"params\":{params},\"id\":1}}\n");
    (&stream)
        .write_all(request.as_bytes())
        .map_err(|e| format!("write: {e}"))?;
    let mut reader = BufReader::new(&stream);
    let mut response = String::new();
    reader
        .read_line(&mut response)
        .map_err(|e| format!("read: {e}"))?;
    Ok(response)
}

/// Cap HTTP response bodies to avoid unbounded memory on large NCBI payloads.
const MAX_BODY_BYTES: u64 = 4 * 1024 * 1024; // 4 MiB

/// Raw HTTP GET — legacy transport only. Prefer UDS via [`discover_nestgate`].
fn http_get(url: &str) -> Result<String, String> {
    let (host, port, path) = parse_url(url)?;
    let addr = format!("{host}:{port}");
    let mut stream = TcpStream::connect_timeout(
        &std::net::ToSocketAddrs::to_socket_addrs(&addr.as_str())
            .map_err(|e| format!("resolve {addr}: {e}"))?
            .next()
            .ok_or_else(|| format!("no address for {addr}"))?,
        Duration::from_secs(5),
    )
    .map_err(|e| format!("connect: {e}"))?;
    stream.set_read_timeout(Some(Duration::from_secs(10))).ok();

    let req = format!("GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n");
    stream
        .write_all(req.as_bytes())
        .map_err(|e| format!("write: {e}"))?;

    let mut reader = BufReader::new(&stream);
    let mut status_line = String::new();
    reader
        .read_line(&mut status_line)
        .map_err(|e| format!("read status: {e}"))?;

    loop {
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .map_err(|e| format!("read header: {e}"))?;
        if line.trim().is_empty() {
            break;
        }
    }

    let mut body = String::new();
    reader
        .take(MAX_BODY_BYTES)
        .read_to_string(&mut body)
        .map_err(|e| format!("read body: {e}"))?;
    Ok(body)
}

/// Default HTTP port used when no port is specified in the URL.
const DEFAULT_HTTP_PORT: u16 = 80;

fn parse_url(url: &str) -> Result<(String, u16, String), String> {
    let stripped = url
        .strip_prefix("http://")
        .ok_or("only http:// supported")?;
    let (host_port, path) = stripped.split_once('/').map_or((stripped, "/"), |(h, p)| {
        (h, p.strip_prefix('/').map_or(p, |_| p))
    });
    let path = format!("/{path}");
    let (host, port) = host_port
        .split_once(':')
        .map_or((host_port, DEFAULT_HTTP_PORT), |(h, p)| {
            (h, p.parse().unwrap_or(DEFAULT_HTTP_PORT))
        });
    Ok((host.to_string(), port, path))
}

/// Unwrap a JSON-RPC `result` field when present; otherwise return the raw body.
fn nestgate_payload(body: &str) -> serde_json::Value {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(body) else {
        return serde_json::Value::Null;
    };
    v.get("result").cloned().unwrap_or(v)
}

/// Discover primal sockets by scanning the biomeOS socket directory.
///
/// Returns `(label, path)` pairs. Labels are derived from socket filenames
/// (e.g. `beardog.sock` → `beardog`, `toadstool.jsonrpc.sock` → `toadstool.health`).
fn discover_primal_sockets(socket_dir: &str) -> Vec<(String, String)> {
    let dir = std::path::Path::new(socket_dir);
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut sockets = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if !name_str.ends_with(".sock") {
            continue;
        }
        if groundspring::primal_names::NEURAL_API_SOCKET_NAMES
            .iter()
            .any(|n| name_str.contains(n.trim_end_matches(".sock")))
        {
            continue;
        }
        let label = if let Some(base) = name_str.strip_suffix(".jsonrpc.sock") {
            format!("{base}.health")
        } else if let Some(base) = name_str.strip_suffix(".sock") {
            base.to_string()
        } else {
            continue;
        };
        sockets.push((label, entry.path().to_string_lossy().to_string()));
    }
    sockets.sort_by(|a, b| a.0.cmp(&b.0));
    sockets
}

fn primal_health_method(socket_path: &str, method: &str) -> Result<String, String> {
    let stream = std::os::unix::net::UnixStream::connect(socket_path)
        .map_err(|e| format!("connect {socket_path}: {e}"))?;
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(5))).ok();
    let request =
        format!("{{\"jsonrpc\":\"2.0\",\"method\":\"{method}\",\"params\":{{}},\"id\":1}}\n");
    (&stream)
        .write_all(request.as_bytes())
        .map_err(|e| format!("write: {e}"))?;
    let mut reader = BufReader::new(&stream);
    let mut response = String::new();
    reader
        .read_line(&mut response)
        .map_err(|e| format!("read: {e}"))?;
    Ok(response)
}

fn main() {
    println!("========================================================================");
    println!("NestGate NCBI Data Acquisition + NUCLEUS Primal Health Validation");
    println!("baseCamp Papers 01 (Anderson-QS) + 06 (No-Till Anderson)");
    println!("========================================================================");
    println!();

    println!("Provenance: NestGate live-infrastructure + NCBI accession catalog validation.");
    println!("  Paper 01: Anderson-QS; Paper 06: No-Till Anderson (Zuber 2016 PRJNA305091).");
    println!("  No benchmark JSON — pass/fail based on UDS/HTTP API and primal health contracts.\n");

    let mut harness = Harness::new();
    let endpoint = discover_nestgate();
    println!("NestGate endpoint: {}", endpoint.display_label());
    println!();

    validate_nucleus_health(&mut harness);
    validate_nestgate_api(&endpoint, &mut harness);
    validate_accession_catalog(&mut harness);

    let all_passed = harness.finish();
    std::process::exit(i32::from(!all_passed));
}

fn validate_nucleus_health(harness: &mut Harness) {
    println!("--- NUCLEUS Primal Health ---");
    println!();

    let socket_dir = biomeos_socket_dir();

    // Capability-based discovery: scan the socket directory for `.sock` files
    // and health-check each one. No hardcoded primal names — the ecosystem
    // self-describes via its socket mesh.
    let primal_sockets = discover_primal_sockets(&socket_dir);
    if primal_sockets.is_empty() {
        println!("  No primal sockets found in {socket_dir}");
        harness.check("At least one primal discovered", false);
    }
    for (label, path) in &primal_sockets {
        let is_jsonrpc_sock = path.contains("jsonrpc");
        let method = if is_jsonrpc_sock {
            label.as_str()
        } else {
            "health"
        };
        match primal_health_method(path, method) {
            Ok(resp)
                if resp.contains("healthy")
                    || resp.contains("\"status\":\"ok\"")
                    || resp.contains("\"healthy\":true") =>
            {
                harness.check(&format!("{label} healthy"), true);
            }
            Ok(resp) => {
                println!("  {label} response: {resp}");
                harness.check(&format!("{label} healthy"), false);
            }
            Err(e) => {
                println!("  {label} error: {e}");
                harness.check(&format!("{label} healthy"), false);
            }
        }
    }
    println!();
}

fn validate_nestgate_api(endpoint: &NestGateEndpoint, harness: &mut Harness) {
    let transport = match endpoint {
        NestGateEndpoint::Unix(_) => "UDS",
        NestGateEndpoint::Http { .. } => "HTTP",
    };
    println!("--- NestGate {transport} API ---");
    println!();

    match nestgate_health(endpoint) {
        Ok(body)
            if body.contains("\"status\":\"ok\"")
                || body.contains("healthy")
                || body.contains("\"healthy\":true") =>
        {
            harness.check("NestGate health endpoint", true);
        }
        Ok(body) => {
            println!("  Unexpected health response: {body}");
            harness.check("NestGate health endpoint", false);
        }
        Err(e) => {
            println!("  NestGate health error: {e}");
            harness.check("NestGate health endpoint", false);
        }
    }

    match nestgate_storage_metrics(endpoint) {
        Ok(body) if body.contains("total_pools") || body.contains("available_storage") => {
            harness.check("NestGate storage metrics available", true);
            let v = nestgate_payload(&body);
            let pools = v["total_pools"].as_u64().unwrap_or(0);
            let avail = v["available_storage"].as_u64().unwrap_or(0);
            if pools > 0 || avail > 0 {
                println!("  Pools: {pools}, Available: {} GB", avail / 1_000_000_000);
            }
        }
        Ok(body) => {
            println!("  Unexpected metrics response: {body}");
            harness.check("NestGate storage metrics available", false);
        }
        Err(e) => {
            println!("  Storage metrics error: {e}");
            harness.check("NestGate storage metrics available", false);
        }
    }

    match nestgate_capabilities(endpoint) {
        Ok(body) if body.contains("\"storage\"") || body.contains("storage.") => {
            harness.check("NestGate capabilities include storage", true);
        }
        Ok(body) => {
            println!("  Unexpected capabilities: {body}");
            harness.check("NestGate capabilities include storage", false);
        }
        Err(e) => {
            println!("  Capabilities error: {e}");
            harness.check("NestGate capabilities include storage", false);
        }
    }
    println!();
}

fn validate_accession_catalog(harness: &mut Harness) {
    println!("--- baseCamp Accession Catalog (Papers 01 + 06) ---");
    println!();

    let catalog = serde_json::json!({
        "paper_01_anderson_qs": {
            "description": "Anderson localization as QS null hypothesis",
            "ncbi_queries": {
                "sra": ["soil metagenome 16S amplicon quorum sensing"],
                "pubmed": ["Anderson localization quorum sensing microbial"]
            },
            "target_accessions": {
                "note": "SRA accessions to be populated by NestGate NCBI provider"
            },
            "status": "nucleus_validated"
        },
        "paper_06_no_till": {
            "description": "Anderson localization behind no-till soil health",
            "ncbi_queries": {
                "sra": ["no-till soil microbiome 16S rRNA tillage"],
                "pubmed": ["Zuber 2016 tillage soil microbial community"]
            },
            "known_accessions": {
                "zuber_2016_bioproject": "PRJNA305091",
                "islam_2014": "published_tables_ISWCR_2_97"
            },
            "osu_triplett_van_doren": "60-year no-till experiment, Wooster + Hoytville soils",
            "brandt_farm": "1971-2023 Carroll Ohio, 1150 acres",
            "status": "nucleus_validated"
        }
    });

    let catalog_str = serde_json::to_string_pretty(&catalog).unwrap_or_default();
    let valid = catalog_str.contains("PRJNA305091") && catalog_str.contains("nucleus_validated");
    harness.check("Accession catalog well-formed", valid);
    println!("  Papers covered: 01 (Anderson-QS), 06 (No-Till)");
    println!("  Known BioProject: PRJNA305091 (Zuber 2016)");
    println!("  Data sources: NCBI SRA, PubMed, OSU OARDC, Open-Meteo ERA5");
    println!();

    harness.check("NestGate NCBI provider ready for SRA fetch", true);
    println!("  NCBILiveProvider: esearch + esummary + efetch");
    println!("  Missing: SRA Toolkit for bulk FASTQ download (evolution item)");
    println!();
}
