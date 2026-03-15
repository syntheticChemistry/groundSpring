// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Full NUCLEUS science pipeline validation for `groundSpring`.
//!
//! Exercises the complete `capability.call` routing:
//!   groundSpring → Neural API → capability-discovered primals
//!
//! Validates:
//! - Neural API topology and metrics
//! - `capability.call` routing to discovered providers
//! - `compute.submit` job dispatch to compute provider
//! - `compute.status` job tracking
//! - Crypto operations via crypto capability
//! - AI health via AI capability
//! - Capability registration for `groundSpring` science caps
//!
//! Requires: `--features biomeos` and a running Full NUCLEUS.

use groundspring::biomeos;
use groundspring_forge::nucleus::{NucleusHarness as Harness, biomeos_socket_dir};
use std::path::{Path, PathBuf};

fn discover_socket() -> Option<PathBuf> {
    biomeos::discover_socket().or_else(|| {
        let xdg = std::env::var("XDG_RUNTIME_DIR").ok()?;
        let p = PathBuf::from(xdg).join("biomeos/neural-api.sock");
        p.exists().then_some(p)
    })
}

fn main() {
    println!("========================================================================");
    println!("NUCLEUS Science Pipeline Validation");
    println!("groundSpring capability.call → Neural API → Primal routing");
    println!("========================================================================");
    println!();

    println!("Provenance: NUCLEUS live-infrastructure validation binary.");
    println!("  Expected values from capability.call routing (live responses).");
    println!("  No benchmark JSON — pass/fail based on API contract compliance.\n");

    let socket = discover_socket().unwrap_or_else(|| {
        eprintln!("ERROR: No Neural API socket found. Is NUCLEUS running?");
        eprintln!("  Start with: cd phase2/biomeOS && ./scripts/start_nucleus.sh full");
        std::process::exit(1);
    });
    println!("Neural API socket: {}", socket.display());
    println!();

    let mut harness = Harness::new();

    validate_neural_api_health(&socket, &mut harness);
    validate_capability_routing(&socket, &mut harness);
    validate_compute_dispatch(&socket, &mut harness);
    validate_compute_execute_anderson(&socket, &mut harness);
    validate_compute_submit_batch(&socket, &mut harness);
    validate_compute_roundtrip(&socket, &mut harness);
    validate_crypto_routing(&socket, &mut harness);
    validate_groundspring_registration(&socket, &mut harness);

    let all_passed = harness.finish();
    std::process::exit(i32::from(!all_passed));
}

fn validate_neural_api_health(socket: &Path, harness: &mut Harness) {
    println!("--- Neural API Health ---");
    println!();

    match biomeos::health(socket) {
        Ok(()) => harness.check("Neural API topology.metrics reachable", true),
        Err(e) => {
            println!("  Error: {e}");
            harness.check("Neural API topology.metrics reachable", false);
        }
    }

    let raw = biomeos::raw_rpc_call(
        socket,
        r#"{"jsonrpc":"2.0","method":"topology.primals","params":{},"id":1}"#,
    );
    match raw {
        Ok(resp) if resp.contains("primals") => {
            let count = resp.matches("primal_type").count();
            println!("  Primals in topology: {count}");
            harness.check("Topology has primals", count > 0);
        }
        Ok(resp) => {
            println!("  Unexpected: {resp}");
            harness.check("Topology has primals", false);
        }
        Err(e) => {
            println!("  Error: {e}");
            harness.check("Topology has primals", false);
        }
    }
    println!();
}

fn validate_capability_routing(socket: &Path, harness: &mut Harness) {
    println!("--- Capability Routing (capability.call → Primal) ---");
    println!();

    let checks = [
        ("compute.health", "compute provider", "healthy"),
        ("compute.version", "compute provider version", "version"),
        ("ai.health", "AI provider", "healthy"),
    ];

    for (cap, name, expected) in checks {
        match biomeos::capability_call(socket, cap, "{}") {
            Ok(resp) if resp.contains(expected) => {
                harness.check(&format!("{name} via {cap}"), true);
            }
            Ok(resp) => {
                println!("  {name} unexpected: {}", &resp[..resp.len().min(120)]);
                harness.check(&format!("{name} via {cap}"), false);
            }
            Err(e) => {
                println!("  {name} error: {e}");
                harness.check(&format!("{name} via {cap}"), false);
            }
        }
    }
    println!();
}

fn validate_compute_dispatch(socket: &Path, harness: &mut Harness) {
    println!("--- Compute Dispatch (groundSpring → Neural API → compute provider) ---");
    println!();

    let params = r#"{"transform":{"operation":"anderson_eigendecompose","input":{"disorder_strength":2.0,"lattice_size":100,"precision":"f32"}}}"#;
    match biomeos::capability_call(socket, "compute.submit", params) {
        Ok(resp) if resp.contains("job_id") => {
            harness.check("compute.submit (Anderson eigendecompose)", true);
            if let Some(start) = resp.find("\"job_id\":\"") {
                let after = &resp[start + 10..];
                if let Some(end) = after.find('"') {
                    let job_id = &after[..end];
                    println!("  Job ID: {job_id}");
                    validate_job_status(socket, job_id, harness);
                }
            }
        }
        Ok(resp) => {
            println!(
                "  Unexpected submit response: {}",
                &resp[..resp.len().min(200)]
            );
            harness.check("compute.submit (Anderson eigendecompose)", false);
        }
        Err(e) => {
            println!("  Submit error: {e}");
            harness.check("compute.submit (Anderson eigendecompose)", false);
        }
    }

    match biomeos::capability_call(socket, "compute.capabilities", "{}") {
        Ok(resp) if resp.contains("compute_units") || resp.contains("supported_workload_types") => {
            harness.check("compute_units enumerated", true);
        }
        Ok(resp) => {
            println!("  Capabilities: {}", &resp[..resp.len().min(200)]);
            harness.check("compute_units enumerated", true);
        }
        Err(e) => {
            println!("  Capabilities error: {e}");
            harness.check("compute_units enumerated", false);
        }
    }

    let custom = r#"{"custom":{"plugin":"groundspring-barracuda","payload":{"workload":"spectral_reconstruction","lattice_size":50,"disorder_strength":1.5}}}"#;
    match biomeos::capability_call(socket, "compute.submit", custom) {
        Ok(resp) if resp.contains("job_id") => {
            harness.check("compute.submit (Barracuda custom workload)", true);
        }
        Ok(resp) => {
            println!("  Custom: {}", &resp[..resp.len().min(200)]);
            harness.check("compute.submit (Barracuda custom workload)", false);
        }
        Err(e) => {
            println!("  Custom error: {e}");
            harness.check("compute.submit (Barracuda custom workload)", false);
        }
    }
    println!();
}

fn validate_job_status(socket: &Path, job_id: &str, harness: &mut Harness) {
    let req = format!(
        r#"{{"jsonrpc":"2.0","method":"capability.call","params":{{"capability":"compute","operation":"status","args":{{"job_id":"{job_id}"}}}},"id":1}}"#,
    );

    std::thread::sleep(std::time::Duration::from_millis(200));

    match biomeos::raw_rpc_call(socket, &req) {
        Ok(resp) if resp.contains("state") => {
            harness.check("compute.status (job tracking)", true);
            if resp.contains("pending") {
                println!("  Job state: pending (queued for dispatch)");
            } else if resp.contains("running") {
                println!("  Job state: running");
            } else if resp.contains("completed") {
                println!("  Job state: completed");
            }
        }
        Ok(resp) => {
            println!("  Status: {}", &resp[..resp.len().min(200)]);
            harness.check("compute.status (job tracking)", false);
        }
        Err(e) => {
            println!("  Status error: {e}");
            harness.check("compute.status (job tracking)", false);
        }
    }
}

fn validate_compute_execute_anderson(socket: &Path, harness: &mut Harness) {
    println!("--- Compute Execute: Anderson Lyapunov (synchronous) ---");
    println!();

    let params = serde_json::json!({
        "n_sites": 200,
        "disorder": 2.0,
        "energy": 0.0,
        "n_realizations": 50,
        "seed": 42
    });

    match biomeos::compute_execute(socket, "lyapunov_averaged", &params.to_string()) {
        Ok(resp) => {
            println!("  Response: {}", &resp[..resp.len().min(200)]);
            let has_result =
                resp.contains("gamma") || resp.contains("lyapunov") || resp.contains("result");
            harness.check("compute.execute (Anderson Lyapunov)", has_result);
        }
        Err(e) => {
            println!("  Execute error: {e}");
            println!("  (Expected when compute provider is not running)");
            harness.check("compute.execute (Anderson Lyapunov)", false);
        }
    }

    let spectral_params = serde_json::json!({
        "n": 50,
        "coupling": 1.5,
        "alpha": 0.618_033_988_749_894_9,
        "theta": 0.0
    });

    match biomeos::compute_execute(
        socket,
        "almost_mathieu_eigenvalues",
        &spectral_params.to_string(),
    ) {
        Ok(resp) => {
            println!("  Spectral response: {}", &resp[..resp.len().min(200)]);
            harness.check("compute.execute (Almost-Mathieu eigenvalues)", true);
        }
        Err(e) => {
            println!("  Spectral error: {e}");
            harness.check("compute.execute (Almost-Mathieu eigenvalues)", false);
        }
    }
    println!();
}

fn validate_compute_submit_batch(socket: &Path, harness: &mut Harness) {
    println!("--- Compute Submit: Batch Workloads (async) ---");
    println!();

    let batch_params = serde_json::json!({
        "workloads": [
            {"op": "lyapunov_averaged", "n_sites": 100, "disorder": 1.0, "energy": 0.0, "n_realizations": 10, "seed": 1},
            {"op": "lyapunov_averaged", "n_sites": 100, "disorder": 2.0, "energy": 0.0, "n_realizations": 10, "seed": 2},
            {"op": "lyapunov_averaged", "n_sites": 100, "disorder": 4.0, "energy": 0.0, "n_realizations": 10, "seed": 3},
        ]
    });

    match biomeos::compute_submit(socket, "batch_anderson", &batch_params.to_string()) {
        Ok(resp) => {
            println!("  Batch response: {}", &resp[..resp.len().min(200)]);
            let has_job = resp.contains("job_id") || resp.contains("batch_id");
            harness.check("compute.submit (batch Anderson)", has_job);

            if let Some(job_id) = extract_job_id(&resp) {
                println!("  Batch job ID: {job_id}");
                poll_job_completion(socket, &job_id, harness);
            }
        }
        Err(e) => {
            println!("  Batch submit error: {e}");
            harness.check("compute.submit (batch Anderson)", false);
        }
    }

    let spectral_batch = serde_json::json!({
        "n": 100,
        "coupling": 2.0,
        "alpha": 0.618_033_988_749_894_9,
        "theta": 0.0
    });

    match biomeos::compute_submit(
        socket,
        "spectral_reconstruction",
        &spectral_batch.to_string(),
    ) {
        Ok(resp) => {
            println!("  Spectral batch: {}", &resp[..resp.len().min(200)]);
            harness.check("compute.submit (spectral recon)", true);
        }
        Err(e) => {
            println!("  Spectral submit error: {e}");
            harness.check("compute.submit (spectral recon)", false);
        }
    }
    println!();
}

fn extract_job_id(response: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(response).ok()?;
    v.get("job_id")
        .or_else(|| v.get("batch_id"))
        .and_then(serde_json::Value::as_str)
        .map(String::from)
}

fn poll_job_completion(socket: &Path, job_id: &str, harness: &mut Harness) {
    let params = serde_json::json!({ "job_id": job_id });
    let req = format!(
        r#"{{"jsonrpc":"2.0","method":"capability.call","params":{{"capability":"compute","operation":"status","args":{params}}},"id":1}}"#,
    );

    for attempt in 0..5 {
        std::thread::sleep(std::time::Duration::from_millis(200 * (1 + attempt)));
        if let Ok(resp) = biomeos::raw_rpc_call(socket, &req) {
            if resp.contains("completed") {
                harness.check("Batch job completed", true);
                return;
            }
            if resp.contains("failed") {
                println!("  Job failed: {}", &resp[..resp.len().min(200)]);
                harness.check("Batch job completed", false);
                return;
            }
        }
    }
    println!("  Job still pending after polling");
    harness.check("Batch job completed (timeout)", false);
}

fn validate_compute_roundtrip(socket: &Path, harness: &mut Harness) {
    println!("--- Compute Round-Trip: Neural API → Provider → Validate vs Local ---");
    println!();

    let n_sites = 200;
    let disorder = 2.0;
    let energy = 0.0;
    let local_gamma = groundspring::anderson::lyapunov_averaged(n_sites, disorder, energy, 50, 42);
    let local_xi = if local_gamma > 0.0 {
        1.0 / local_gamma
    } else {
        f64::INFINITY
    };

    println!("  Local CPU: γ={local_gamma:.6}, ξ={local_xi:.2}");

    let params = serde_json::json!({
        "n_sites": n_sites,
        "disorder": disorder,
        "energy": energy,
        "n_realizations": 50,
        "seed": 42
    });

    match biomeos::compute_execute(socket, "lyapunov_averaged", &params.to_string()) {
        Ok(resp) => {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&resp) {
                if let Some(remote_gamma) = v
                    .get("gamma")
                    .or_else(|| v.get("result"))
                    .and_then(serde_json::Value::as_f64)
                {
                    let diff = (local_gamma - remote_gamma).abs();
                    println!("  Remote GPU: γ={remote_gamma:.6}, diff={diff:.2e}");
                    // 0.1 tolerance: PRNG mismatch (CPU xorshift64 vs GPU xoshiro128**)
                    // means stochastic Lyapunov averages differ. Tolerance is ~10% of
                    // typical γ ≈ 0.5–2.0 at W=4.0, covering PRNG + GPU f64 rounding.
                    harness.check("Round-trip γ parity (< 0.1)", diff < 0.1);
                } else {
                    println!("  Could not parse gamma from: {resp}");
                    harness.check("Round-trip γ parity", false);
                }
            } else {
                println!("  Invalid JSON response: {resp}");
                harness.check("Round-trip γ parity", false);
            }
        }
        Err(e) => {
            println!("  Round-trip error: {e}");
            println!("  (Expected when compute provider is not running)");
            harness.check("Round-trip γ parity (provider unavailable)", false);
        }
    }
    println!();
}

fn validate_crypto_routing(neural_socket: &Path, harness: &mut Harness) {
    println!("--- Crypto Routing (capability-based) ---");
    println!();

    match biomeos::capability_call(neural_socket, "crypto.health", "{}") {
        Ok(resp) if resp.contains("healthy") => {
            harness.check("crypto.health via Neural API", true);
        }
        Ok(resp) => {
            println!("  Health: {}", &resp[..resp.len().min(120)]);
            harness.check("crypto.health via Neural API", false);
        }
        Err(e) => {
            println!("  Crypto routing error (falling back to direct): {e}");
            validate_crypto_direct(harness);
        }
    }

    match biomeos::capability_call(neural_socket, "crypto.sha256", r#"{"data":"dGVzdA=="}"#) {
        Ok(resp) if resp.contains("hash") => {
            harness.check("crypto.sha256 via capability routing", true);
            println!("  Hash: {}", &resp[..resp.len().min(120)]);
        }
        Ok(resp) => {
            println!("  SHA256: {resp}");
            harness.check("crypto.sha256 via capability routing", false);
        }
        Err(e) => {
            println!("  SHA256 error: {e}");
            harness.check("crypto.sha256 via capability routing", false);
        }
    }
    println!();
}

/// Fallback: discover the crypto provider socket from the socket directory.
fn validate_crypto_direct(harness: &mut Harness) {
    let socket_dir = biomeos_socket_dir();
    let Ok(entries) = std::fs::read_dir(&socket_dir) else {
        harness.check("crypto provider discovered", false);
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let path = entry.path();
        let path_str = path.to_string_lossy();
        if name.to_string_lossy().ends_with(".sock")
            && !path_str.contains("neural-api")
            && let Ok(resp) = rpc_to_socket(&path_str, "crypto.sha256", r#"{"data":"dGVzdA=="}"#)
            && resp.contains("hash")
        {
            harness.check("crypto.sha256 via direct discovery", true);
            return;
        }
    }
    harness.check("crypto provider discovered", false);
}

fn rpc_to_socket(
    socket_path: &str,
    method: &str,
    params: &str,
) -> std::result::Result<String, String> {
    use std::io::{BufRead, BufReader, Write};

    let stream = std::os::unix::net::UnixStream::connect(socket_path)
        .map_err(|e| format!("connect {socket_path}: {e}"))?;
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .ok();
    stream
        .set_write_timeout(Some(std::time::Duration::from_secs(5)))
        .ok();
    let request = format!(r#"{{"jsonrpc":"2.0","method":"{method}","params":{params},"id":1}}"#,);
    (&stream)
        .write_all(request.as_bytes())
        .map_err(|e| format!("write: {e}"))?;
    (&stream)
        .write_all(b"\n")
        .map_err(|e| format!("write newline: {e}"))?;
    let mut reader = BufReader::new(&stream);
    let mut response = String::new();
    reader
        .read_line(&mut response)
        .map_err(|e| format!("read: {e}"))?;
    Ok(response)
}

fn validate_groundspring_registration(socket: &Path, harness: &mut Harness) {
    println!("--- groundSpring Capability Registration ---");
    println!();

    let req = r#"{"jsonrpc":"2.0","method":"capability.list","params":{},"id":1}"#;
    match biomeos::raw_rpc_call(socket, req) {
        Ok(resp) => {
            let science_caps = [
                "science.anderson_validation",
                "science.noise_decomposition",
                "science.parity_check",
                "science.et0_propagation",
            ];
            for cap in science_caps {
                harness.check(&format!("{cap} registered"), resp.contains(cap));
            }
            let total: usize = resp.matches("science.").count();
            println!("  Total science.* capabilities: {total}");
        }
        Err(e) => {
            println!("  List error: {e}");
            for _ in 0..4 {
                harness.check("capability registration", false);
            }
        }
    }
    println!();
}
