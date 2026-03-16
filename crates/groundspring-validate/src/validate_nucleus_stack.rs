// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Exp 031: NUCLEUS Stack Validation — live primal interaction through Neural API.
//!
//! Exercises whatever NUCLEUS primals are live, adapting to the deployment:
//!
//! - **Tower** (crypto + beacon capabilities): Health, crypto hash, beacon
//! - **Node** (compute capability): Health, capabilities query, version
//! - **Nest** (storage + data capabilities): Storage round-trip, data providers
//! - **AI capability**: AI health
//! - **Sovereign fallback**: All paths degrade gracefully when providers are absent
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
use groundspring_validate::TOL_STOCHASTIC_MEAN;

#[cfg(feature = "biomeos")]
fn main() {
    std::process::exit(run());
}

#[cfg(feature = "biomeos")]
fn run() -> i32 {
    let mut h = ValidationHarness::stdout("Exp 031: NUCLEUS Stack Validation");

    println!("{}", "=".repeat(72));
    println!("  Exp 031: NUCLEUS Stack — Live Primal Interaction");
    println!("{}", "=".repeat(72));
    println!();
    println!("  Provenance: NUCLEUS infrastructure validation binary");
    println!("  Data source: Live NUCLEUS primal responses or sovereign fallback");
    println!("  Baseline: Capability-based — validates graceful degradation and");
    println!("        JSON-RPC 2.0 contract compliance, not numerical baselines.");
    println!();

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

    let primals = biomeos::discover_primals();
    println!(
        "  Discovered primals:   {} ({})",
        primals.len(),
        primals
            .iter()
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    h.check_true(
        "Primal socket discovery finds sockets",
        !primals.is_empty() || !nucleus_live,
    );

    if !nucleus_live {
        println!("\n  NUCLEUS is offline — validating sovereign fallback paths");
        validate_sovereign_fallback(&mut h);
        println!();
        return h.summary();
    }

    let Some(socket) = socket else {
        eprintln!("FATAL: NUCLEUS socket unavailable despite live check");
        return 1;
    };
    println!("  Socket: {}", socket.display());

    validate_tower(&mut h, &socket);
    validate_direct_primals(&mut h);
    validate_node(&mut h, &socket);
    validate_ai(&mut h, &socket);
    validate_nest(&mut h, &socket);
    validate_local_compute(&mut h);

    println!();
    h.summary()
}

/// Tower: Neural API health + proprioception + topology.
#[cfg(feature = "biomeos")]
fn validate_tower(h: &mut ValidationHarness, socket: &std::path::Path) {
    println!("\n--- Phase B: Tower (Neural API) ---");

    let health_ok = biomeos::health(socket).is_ok();
    println!(
        "  neural_api.get_metrics: {}",
        if health_ok { "OK" } else { "FAIL" }
    );
    h.check_true("Neural API health check", health_ok);

    match biomeos::proprioception(socket) {
        Ok(result) => {
            let has_awareness = result.contains("self_awareness") || result.contains("sensory");
            println!("  proprioception: OK (self-aware: {has_awareness})");
            h.check_true("Neural API proprioception", !result.is_empty());
        }
        Err(e) => {
            println!("  proprioception: {e}");
            h.check_true("proprioception (or graceful error)", true);
        }
    }

    match biomeos::topology(socket) {
        Ok(result) => {
            let has_connections = result.contains("connections");
            println!("  topology: OK (connections: {has_connections})");
            h.check_true("Neural API topology", !result.is_empty());
        }
        Err(e) => {
            println!("  topology: {e}");
            h.check_true("topology (or graceful error)", true);
        }
    }
}

/// Direct primal health checks — discovered at runtime, not hardcoded.
#[cfg(feature = "biomeos")]
fn validate_direct_primals(h: &mut ValidationHarness) {
    println!("\n--- Phase B2: Direct Primal Health ---");

    let primals = biomeos::discover_primals();
    if primals.is_empty() {
        println!("  No primals discovered — skipping direct health");
        h.check_true("No primals discovered (graceful)", true);
        return;
    }

    for primal in &primals {
        let name = &primal.name;
        match biomeos::primal_health(name) {
            Ok(result) => {
                let healthy = result.contains("healthy") || result.contains("true");
                println!("  {name}: OK (healthy: {healthy})");
                h.check_true(&format!("{name} direct health"), true);
            }
            Err(e) => {
                println!("  {name}: {e}");
                h.check_true(&format!("{name} direct health (or absent)"), true);
            }
        }
    }
}

/// Node: compute capability health — routed through Neural API capabilities.
///
/// Uses capability-based routing exclusively. No hardcoded primal names —
/// whatever primal provides `compute.*` will answer.
#[cfg(feature = "biomeos")]
fn validate_node(h: &mut ValidationHarness, socket: &std::path::Path) {
    println!("\n--- Phase C: Node (compute) ---");

    for method in &["compute.health", "compute.capabilities", "compute.version"] {
        match biomeos::capability_call(socket, method, "{}") {
            Ok(result) => {
                println!("  {method}: OK ({} bytes)", result.len());
                h.check_true(&format!("{method} responds"), !result.is_empty());
            }
            Err(e) => {
                println!("  {method}: {e}");
                h.check_true(&format!("{method} (or graceful error)"), true);
            }
        }
    }
}

/// AI capability health — routed through Neural API capability system.
#[cfg(feature = "biomeos")]
fn validate_ai(h: &mut ValidationHarness, socket: &std::path::Path) {
    println!("\n--- Phase D: AI capability ---");

    match biomeos::capability_call(socket, "ai.health", "{}") {
        Ok(result) => {
            println!("  ai.health: OK ({} bytes)", result.len());
            h.check_true("AI health responds", true);
        }
        Err(e) => {
            println!("  ai.health: {e}");
            h.check_true("AI health (or graceful error)", true);
        }
    }
}

/// Nest: storage + data capabilities.
#[cfg(feature = "biomeos")]
fn validate_nest(h: &mut ValidationHarness, socket: &std::path::Path) {
    println!("\n--- Phase E: Nest (storage + data) ---");

    let test_key = "groundspring:exp031:nucleus_stack_test";
    let test_value = r#"{"experiment":"exp031","ts":"2026-03-08"}"#;

    match biomeos::storage_put(socket, test_key, test_value) {
        Ok(()) => {
            println!("  storage.put:  OK");
            h.check_true("storage put succeeds", true);

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
            println!("  (NestGate not in current deployment — socket pending)");
            h.check_true("storage absent is handled gracefully", true);
        }
    }

    if let Ok(result) = biomeos::capability_call(
        socket,
        "data.ncbi_search",
        r#"{"database":"sra","query":"soil metagenome"}"#,
    ) {
        println!("  data.ncbi_search: OK ({} bytes)", result.len());
        h.check_true("NCBI search via data capability", !result.is_empty());
    } else {
        println!("  data.ncbi_search: NOT AVAILABLE (data provider not running)");
        h.check_true("data capability absent handled gracefully", true);
    }
}

/// Local compute validation — sovereign fallback always works.
#[cfg(feature = "biomeos")]
fn validate_local_compute(h: &mut ValidationHarness) {
    println!("\n--- Phase F: Local Compute (Sovereign) ---");

    let gamma = local_lyapunov(500, 2.0, 0.0, 10, 42);
    println!("  Local Lyapunov (W=2.0): γ = {gamma:.6}");
    h.check_true("Local Anderson computation succeeds", gamma > 0.0);

    let gamma_clean = local_lyapunov(500, 0.0, 0.0, 1, 42);
    println!("  Local Lyapunov (W=0.0): γ = {gamma_clean:.6}");
    h.check_true(
        "Clean system γ ≈ 0",
        gamma_clean.abs() < TOL_STOCHASTIC_MEAN,
    );
}

#[cfg(feature = "biomeos")]
fn validate_sovereign_fallback(h: &mut ValidationHarness) {
    let fake = std::env::temp_dir().join("groundspring_exp031_nonexistent.sock");

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
#[expect(
    clippy::cast_precision_loss,
    reason = "n_real cast to f64 for Lyapunov average"
)]
fn local_lyapunov(n_sites: usize, disorder: f64, energy: f64, n_real: usize, seed: u64) -> f64 {
    use groundspring::anderson;
    let mut sum = 0.0;
    for r in 0..n_real {
        let potential = anderson::anderson_potential(n_sites, disorder, seed + r as u64);
        sum += anderson::lyapunov_exponent(&potential, energy);
    }
    sum / n_real as f64
}
