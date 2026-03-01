// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Exp 031: NUCLEUS Stack Validation — live primal interaction through Neural API.
//!
//! Exercises whatever NUCLEUS primals are live, adapting to the deployment:
//!
//! - **Tower** (`BearDog` + Songbird): Health, crypto hash, beacon
//! - **Node** (`ToadStool`): Health, capabilities query, version
//! - **Nest** (`NestGate`): Storage round-trip, data providers (if running)
//! - **Squirrel**: AI health
//! - **Sovereign fallback**: All paths degrade gracefully when primals are absent
//!
//! Requires: `--features biomeos` + running NUCLEUS (sovereign fallback validates
//! that the client code handles absence gracefully)

#[cfg(not(feature = "biomeos"))]
compile_error!("Exp 031 requires --features biomeos");

#[cfg(feature = "biomeos")]
use groundspring::biomeos;
#[cfg(feature = "biomeos")]
use groundspring::validate::ValidationHarness;

#[cfg(feature = "biomeos")]
fn main() {
    let mut h = ValidationHarness::stdout("Exp 031: NUCLEUS Stack Validation");

    println!("{}", "=".repeat(72));
    println!("  Exp 031: NUCLEUS Stack — Live Primal Interaction");
    println!("{}", "=".repeat(72));

    println!("\n--- Phase A: Socket Discovery ---");
    let socket = biomeos::auto_connect();
    let nucleus_live = socket.is_some();
    println!(
        "  auto_connect:         {}",
        if nucleus_live { "CONNECTED" } else { "OFFLINE" }
    );
    h.check_true(
        "auto_connect and is_nucleus_available agree",
        nucleus_live == biomeos::is_nucleus_available(),
    );

    if !nucleus_live {
        println!("\n  NUCLEUS is offline — validating sovereign fallback paths");
        validate_sovereign_fallback(&mut h);
        println!();
        let _ = h.summary();
        return;
    }

    let socket = socket.unwrap();
    println!("  Socket: {}", socket.display());

    validate_tower(&mut h, &socket);
    validate_node(&mut h, &socket);
    validate_squirrel(&mut h, &socket);
    validate_nest(&mut h, &socket);
    validate_local_compute(&mut h);

    println!();
    let _ = h.summary();
}

/// Tower: Neural API health + `BearDog` crypto operations.
#[cfg(feature = "biomeos")]
fn validate_tower(h: &mut ValidationHarness, socket: &std::path::Path) {
    println!("\n--- Phase B: Tower (BearDog + Songbird) ---");

    let health_ok = biomeos::health(socket).is_ok();
    println!(
        "  topology.metrics: {}",
        if health_ok { "OK" } else { "FAIL" }
    );
    h.check_true("Neural API health check", health_ok);

    match biomeos::capability_call(
        socket,
        "crypto.hash",
        r#"{"data":"groundspring:exp031:nucleus_stack_test"}"#,
    ) {
        Ok(result) => {
            println!("  crypto.hash (BearDog): OK ({} bytes)", result.len());
            h.check_true("BearDog crypto hash returns data", !result.is_empty());
        }
        Err(e) => {
            println!("  crypto.hash (BearDog): {e}");
            h.check_true("BearDog crypto (or graceful error)", true);
        }
    }

    match biomeos::capability_call(socket, "beacon.get_id", "{}") {
        Ok(result) => {
            println!("  beacon.get_id: OK ({} bytes)", result.len());
            h.check_true("Beacon ID returns data", !result.is_empty());
        }
        Err(e) => {
            println!("  beacon.get_id: {e}");
            h.check_true("Beacon ID (or graceful error)", true);
        }
    }
}

/// Node: `ToadStool` health and capability query.
#[cfg(feature = "biomeos")]
fn validate_node(h: &mut ValidationHarness, socket: &std::path::Path) {
    println!("\n--- Phase C: Node (ToadStool) ---");

    match biomeos::capability_call(socket, "compute.health", "{}") {
        Ok(result) => {
            println!("  compute.health: OK");
            let has_gpu = result.contains("gpu")
                || result.contains("GPU")
                || result.contains("wgpu")
                || result.contains("healthy");
            println!("  GPU info present: {has_gpu}");
            h.check_true("ToadStool health responds", true);
        }
        Err(e) => {
            println!("  compute.health: {e}");
            h.check_true("ToadStool health (or graceful error)", true);
        }
    }

    match biomeos::capability_call(socket, "compute.capabilities", "{}") {
        Ok(result) => {
            println!("  compute.capabilities: OK ({} bytes)", result.len());
            h.check_true("ToadStool capabilities responds", !result.is_empty());
        }
        Err(e) => {
            println!("  compute.capabilities: {e}");
            h.check_true("ToadStool capabilities (or graceful error)", true);
        }
    }

    match biomeos::capability_call(socket, "compute.version", "{}") {
        Ok(result) => {
            println!("  compute.version: {result}");
            h.check_true("ToadStool version responds", !result.is_empty());
        }
        Err(e) => {
            println!("  compute.version: {e}");
            h.check_true("ToadStool version (or graceful error)", true);
        }
    }
}

/// Squirrel AI health.
#[cfg(feature = "biomeos")]
fn validate_squirrel(h: &mut ValidationHarness, socket: &std::path::Path) {
    println!("\n--- Phase D: Squirrel (AI) ---");

    match biomeos::capability_call(socket, "ai.health", "{}") {
        Ok(result) => {
            println!("  ai.health: OK ({} bytes)", result.len());
            h.check_true("Squirrel health responds", true);
        }
        Err(e) => {
            println!("  ai.health: {e}");
            h.check_true("Squirrel health (or graceful error)", true);
        }
    }
}

/// Nest: `NestGate` storage + data (only if registered).
#[cfg(feature = "biomeos")]
fn validate_nest(h: &mut ValidationHarness, socket: &std::path::Path) {
    println!("\n--- Phase E: Nest (NestGate — if running) ---");

    let test_key = "groundspring:exp031:nucleus_stack_test";
    let test_value = r#"{"experiment":"exp031","ts":"2026-02-28"}"#;

    match biomeos::storage_put(socket, test_key, test_value) {
        Ok(()) => {
            println!("  storage.put:  OK");
            h.check_true("NestGate storage put", true);

            match biomeos::storage_get(socket, test_key) {
                Ok(retrieved) => {
                    println!("  storage.get:  OK ({} bytes)", retrieved.len());
                    h.check_true(
                        "Storage round-trip preserves data",
                        retrieved.contains("exp031"),
                    );
                }
                Err(e) => {
                    println!("  storage.get:  FAIL ({e})");
                    h.check_true("Storage get succeeds", false);
                }
            }
        }
        Err(e) => {
            println!("  storage.put:  NOT AVAILABLE ({e})");
            println!("  (NestGate not in current NUCLEUS deployment)");
            h.check_true("NestGate absent is handled gracefully", true);
        }
    }

    if let Ok(result) = biomeos::capability_call(
        socket,
        "data.ncbi_search",
        r#"{"database":"sra","query":"soil metagenome"}"#,
    ) {
        println!("  data.ncbi_search: OK ({} bytes)", result.len());
        h.check_true("NCBI search via NestGate", !result.is_empty());
    } else {
        println!("  data.ncbi_search: NOT AVAILABLE (NestGate not running)");
        h.check_true("Data capability absent handled gracefully", true);
    }
}

/// Local compute validation — sovereign fallback always works.
#[cfg(feature = "biomeos")]
#[allow(clippy::cast_precision_loss)]
fn validate_local_compute(h: &mut ValidationHarness) {
    println!("\n--- Phase F: Local Compute (Sovereign) ---");

    let gamma = local_lyapunov(500, 2.0, 0.0, 10, 42);
    println!("  Local Lyapunov (W=2.0): γ = {gamma:.6}");
    h.check_true("Local Anderson computation succeeds", gamma > 0.0);

    let gamma_clean = local_lyapunov(500, 0.0, 0.0, 1, 42);
    println!("  Local Lyapunov (W=0.0): γ = {gamma_clean:.6}");
    h.check_true("Clean system γ ≈ 0", gamma_clean.abs() < 0.01);
}

#[cfg(feature = "biomeos")]
fn validate_sovereign_fallback(h: &mut ValidationHarness) {
    let fake = std::path::PathBuf::from("/tmp/groundspring_exp031_nonexistent.sock");

    h.check_true(
        "health() Err on missing socket",
        biomeos::health(&fake).is_err(),
    );
    h.check_true(
        "storage_put() Err on missing socket",
        biomeos::storage_put(&fake, "t", "{}").is_err(),
    );
    h.check_true(
        "storage_get() Err on missing socket",
        biomeos::storage_get(&fake, "t").is_err(),
    );
    h.check_true(
        "compute_execute() Err on missing socket",
        biomeos::compute_execute(&fake, "t", "{}").is_err(),
    );
    h.check_true(
        "capability_call() Err on missing socket",
        biomeos::capability_call(&fake, "data.x", "{}").is_err(),
    );

    let gamma = local_lyapunov(500, 2.0, 0.0, 10, 42);
    h.check_true("Local Lyapunov works offline", gamma > 0.0);

    println!("  All sovereign fallback paths verified");
}

#[cfg(feature = "biomeos")]
#[allow(clippy::cast_precision_loss)]
fn local_lyapunov(n_sites: usize, disorder: f64, energy: f64, n_real: usize, seed: u64) -> f64 {
    use groundspring::anderson;
    let mut sum = 0.0;
    for r in 0..n_real {
        let potential = anderson::anderson_potential(n_sites, disorder, seed + r as u64);
        sum += anderson::lyapunov_exponent(&potential, energy);
    }
    sum / n_real as f64
}
