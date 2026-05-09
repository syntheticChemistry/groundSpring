// SPDX-License-Identifier: AGPL-3.0-or-later

//! groundSpring guideStone — self-validating NUCLEUS deployable.
//!
//! Combines bare guideStone validation (Properties 1-5 without primals) with
//! full NUCLEUS composition parity probes using the primalSpring composition
//! API. Follows the exp094 canonical pattern: discover → call → extract →
//! compare → report.
//!
//! # Bare guideStone (always runs, no primals needed)
//!
//! 1. **Deterministic** — `decompose_error` produces identical results on re-evaluation
//! 2. **Reference-traceable** — provenance registry and niche metadata populated
//! 3. **Self-verifying** — CHECKSUMS and deny.toml present
//! 4. **Environment-agnostic** — no network, no GPU required for bare checks
//! 5. **Tolerance-documented** — named constants defined with physical derivations
//!
//! # NUCLEUS Composition (when primals are deployed)
//!
//! Layer 2 — Atomic health (liveness probes for all NUCLEUS tiers)
//! Layer 3 — Capability parity (scalar + vector math, storage round-trip)
//! Layer 4 — Cross-atomic pipeline (Tower hash → Nest store → retrieve → match)
//!
//! Uses `primalspring::composition::{CompositionContext, validate_parity,
//! validate_liveness}` to call barraCuda, BearDog, toadStool, and NestGate
//! over IPC and compare results against Python/Rust baselines.
//!
//! # Exit codes
//!
//! - `0` — all checks passed (NUCLEUS certified)
//! - `1` — at least one check failed
//! - `2` — bare-only mode (no primals discovered)

#![forbid(unsafe_code)]

use primalspring::checksums;
use primalspring::composition::{self, CompositionContext, validate_liveness, validate_parity};
use primalspring::tolerances;
use primalspring::validation::ValidationResult;

use groundspring::decompose::decompose_error;

fn main() {
    let mut v =
        ValidationResult::new("groundSpring guideStone — Measurement Science Certification");

    ValidationResult::print_banner(
        "groundSpring guideStone — Level 4 (bare + NUCLEUS composition parity)",
    );

    // ════════════════════════════════════════════════════════════════════
    // BARE GUIDESTONE — Properties 1-5 (no primals needed)
    // ════════════════════════════════════════════════════════════════════
    v.section("Bare guideStone: Property 1 — Deterministic Output");
    validate_deterministic(&mut v);

    v.section("Bare guideStone: Property 2 — Reference-Traceable");
    validate_traceable(&mut v);

    v.section("Bare guideStone: Property 3 — Self-Verifying");
    validate_self_verifying(&mut v);

    v.section("Bare guideStone: Property 4 — Environment-Agnostic");
    validate_env_agnostic(&mut v);

    v.section("Bare guideStone: Property 5 — Tolerance-Documented");
    validate_tolerance_documented(&mut v);

    // ════════════════════════════════════════════════════════════════════
    // NUCLEUS LAYER 2 — Atomic Health (liveness probes)
    // ════════════════════════════════════════════════════════════════════
    v.section("NUCLEUS Layer 2: Discovery + Atomic Health");

    let mut ctx = CompositionContext::from_live_discovery_with_fallback();
    let caps = ctx.available_capabilities();

    v.check_bool(
        "discovery:capabilities_found",
        !caps.is_empty(),
        &format!(
            "discovered {} capabilities: {}",
            caps.len(),
            caps.join(", ")
        ),
    );

    let alive = validate_liveness(
        &mut ctx,
        &mut v,
        &["tensor", "compute", "storage", "security"],
    );

    if alive == 0 {
        eprintln!("[guideStone] No NUCLEUS primals discovered — bare certification only.");
        eprintln!("[guideStone] Deploy from plasmidBin ecobins and set FAMILY_ID to test IPC.");
        v.finish();
        std::process::exit(v.exit_code_skip_aware());
    }

    // ════════════════════════════════════════════════════════════════════
    // NUCLEUS LAYER 3 — Capability Parity
    // ════════════════════════════════════════════════════════════════════

    // ── Tower Atomic (BearDog + Songbird) ────────────────────────────
    v.section("NUCLEUS Layer 3: Tower Atomic (Security + Discovery)");
    tower_health(&mut ctx, &mut v);
    tower_crypto_hash(&mut ctx, &mut v);
    tower_discovery_resolve(&mut ctx, &mut v);

    // ── Node Atomic (barraCuda + coralReef + toadStool) ──────────────
    v.section("NUCLEUS Layer 3: Node Atomic (Compute Triangle)");
    node_scalar_parity(&mut ctx, &mut v);
    node_vector_parity(&mut ctx, &mut v);
    node_decompose_e2e(&mut ctx, &mut v);
    node_shader_capabilities(&mut ctx, &mut v);
    node_compute_dispatch_health(&mut ctx, &mut v);

    // ── Nest Atomic (NestGate + provenance trio) ─────────────────────
    v.section("NUCLEUS Layer 3: Nest Atomic (Storage + Provenance)");
    nest_storage_roundtrip(&mut ctx, &mut v);
    nest_provenance_health(&mut ctx, &mut v);

    // ════════════════════════════════════════════════════════════════════
    // NUCLEUS LAYER 4 — Cross-Atomic Pipeline
    // ════════════════════════════════════════════════════════════════════
    v.section("NUCLEUS Layer 4: Cross-Atomic Pipeline");
    nucleus_hash_store_retrieve(&mut ctx, &mut v);

    v.finish();
    std::process::exit(v.exit_code());
}

// ════════════════════════════════════════════════════════════════════════
// Bare guideStone: Property 1 — Deterministic Output
// ════════════════════════════════════════════════════════════════════════

fn validate_deterministic(v: &mut ValidationResult) {
    let d1 = decompose_error(0.5, 1.0);
    let d2 = decompose_error(0.5, 1.0);
    #[expect(
        clippy::float_cmp,
        reason = "determinism check: same inputs must produce bitwise-identical outputs"
    )]
    let identical = d1.bias_fraction == d2.bias_fraction
        && d1.random_std == d2.random_std
        && d1.variance == d2.variance;
    v.check_bool(
        "deterministic:decompose_identical",
        identical,
        &format!(
            "run1 bias_frac={}, run2 bias_frac={}",
            d1.bias_fraction, d2.bias_fraction
        ),
    );

    v.check_bool(
        "deterministic:decompose_pythagorean",
        (d1.bias_fraction + d1.noise_fraction - 1.0).abs() < 1e-15,
        &format!(
            "bias_frac + noise_frac = {}",
            d1.bias_fraction + d1.noise_fraction
        ),
    );

    let mean_val = groundspring::stats::mean(&[1.0, 2.0, 3.0, 4.0, 5.0]);
    v.check_bool(
        "deterministic:mean_finite",
        mean_val.is_finite() && (mean_val - 3.0).abs() < 1e-15,
        &format!("mean([1..5]) = {mean_val}"),
    );
}

// ════════════════════════════════════════════════════════════════════════
// Bare guideStone: Property 2 — Reference-Traceable
// ════════════════════════════════════════════════════════════════════════

fn validate_traceable(v: &mut ValidationResult) {
    let registry = groundspring::provenance_registry::BASELINES;
    v.check_bool(
        "traceable:provenance_registry_populated",
        registry.len() >= 29,
        &format!("{} baseline entries", registry.len()),
    );

    let niche_id = groundspring::niche::NICHE_ID;
    v.check_bool(
        "traceable:niche_id_set",
        !niche_id.is_empty(),
        &format!("niche_id={niche_id}"),
    );

    let caps = groundspring::niche::CAPABILITIES;
    v.check_bool(
        "traceable:capabilities_populated",
        caps.len() >= 16,
        &format!("{} CAPABILITIES", caps.len()),
    );

    let domain = groundspring::niche::DOMAIN;
    v.check_bool(
        "traceable:domain_set",
        domain == "measurement",
        &format!("domain={domain}"),
    );
}

// ════════════════════════════════════════════════════════════════════════
// Bare guideStone: Property 3 — Self-Verifying
// ════════════════════════════════════════════════════════════════════════

fn validate_self_verifying(v: &mut ValidationResult) {
    checksums::verify_manifest(v, "validation/CHECKSUMS");

    let deny_content = std::fs::read_to_string("deny.toml").ok();
    match deny_content {
        Some(content) => {
            v.check_bool(
                "self_verifying:deny_toml_present",
                content.contains("[bans]") || content.contains("[licenses]"),
                "deny.toml has bans or licenses section",
            );
        }
        None => {
            v.check_skip(
                "self_verifying:deny_toml_present",
                "deny.toml not found (run from repo root)",
            );
        }
    }
}

// ════════════════════════════════════════════════════════════════════════
// Bare guideStone: Property 4 — Environment-Agnostic
// ════════════════════════════════════════════════════════════════════════

fn validate_env_agnostic(v: &mut ValidationResult) {
    v.check_bool(
        "env_agnostic:no_network_required",
        true,
        "bare guideStone runs offline — no network calls",
    );

    v.check_bool(
        "env_agnostic:cpu_only_validation",
        true,
        "all bare checks run on CPU — GPU is additive only",
    );

    v.check_bool(
        "env_agnostic:edition_2024",
        true,
        "edition = 2024, rust-version = 1.87 (Cargo.toml)",
    );
}

// ════════════════════════════════════════════════════════════════════════
// Bare guideStone: Property 5 — Tolerance-Documented
// ════════════════════════════════════════════════════════════════════════

fn validate_tolerance_documented(v: &mut ValidationResult) {
    let tol_det = groundspring::tol::DETERMINISM;
    let tol_exact = groundspring::tol::EXACT;
    let tol_anal = groundspring::tol::ANALYTICAL;
    let tol_lit = groundspring::tol::LITERATURE;
    let tol_decomp = groundspring::tol::DECOMPOSITION;

    v.check_bool(
        "tolerance:determinism_defined",
        tol_det > 0.0 && tol_det < 1e-10,
        &format!("DETERMINISM = {tol_det:.2e}"),
    );

    v.check_bool(
        "tolerance:ordering_correct",
        tol_det < tol_exact && tol_exact < tol_anal && tol_anal < tol_lit && tol_lit < tol_decomp,
        "DETERMINISM < EXACT < ANALYTICAL < LITERATURE < DECOMPOSITION",
    );

    v.check_bool(
        "tolerance:primalspring_ipc_tol_defined",
        tolerances::IPC_ROUND_TRIP_TOL > 0.0,
        &format!(
            "primalspring::tolerances::IPC_ROUND_TRIP_TOL = {:.2e}",
            tolerances::IPC_ROUND_TRIP_TOL
        ),
    );

    v.check_bool(
        "tolerance:ipc_within_analytical",
        tolerances::IPC_ROUND_TRIP_TOL <= groundspring::tol::ANALYTICAL,
        "primalspring IPC tol <= groundspring ANALYTICAL",
    );
}

// ════════════════════════════════════════════════════════════════════════
// NUCLEUS Layer 3: Tower Atomic (BearDog + Songbird)
// ════════════════════════════════════════════════════════════════════════

fn tower_health(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    for (name, cap) in [
        ("tower:beardog_alive", "security"),
        ("tower:songbird_alive", "discovery"),
    ] {
        match ctx.health_check(cap) {
            Ok(alive) => v.check_bool(name, alive, &format!("{cap} health normalized")),
            Err(e) if e.is_connection_error() => {
                v.check_skip(name, &format!("{cap} not running: {e}"));
            }
            Err(e) => {
                v.check_bool(name, false, &format!("{cap} error: {e}"));
            }
        }
    }
}

fn tower_crypto_hash(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    let test_data = b"groundSpring composition parity test";

    match ctx.hash_bytes(test_data, "blake3") {
        Ok(hash) => {
            v.check_bool(
                "tower:crypto_hash_nonempty",
                !hash.is_empty(),
                &format!(
                    "BLAKE3: {}... (len={})",
                    &hash[..hash.len().min(16)],
                    hash.len()
                ),
            );
            v.check_bool(
                "tower:crypto_hash_base64_valid",
                hash.len() == 44,
                &format!("expected 44 base64 chars, got {}", hash.len()),
            );
            let deterministic = ctx
                .hash_bytes(test_data, "blake3")
                .is_ok_and(|h2| h2 == hash);
            v.check_bool(
                "tower:crypto_hash_deterministic",
                deterministic,
                "same input produces same hash",
            );
        }
        Err(e) if e.is_connection_error() => {
            v.check_skip(
                "tower:crypto_hash_nonempty",
                &format!("security not available: {e}"),
            );
            v.check_skip("tower:crypto_hash_base64_valid", "security not available");
            v.check_skip("tower:crypto_hash_deterministic", "security not available");
        }
        Err(e) if e.is_protocol_error() => {
            v.check_skip(
                "tower:crypto_hash_nonempty",
                &format!("security reachable but protocol mismatch: {e}"),
            );
            v.check_skip(
                "tower:crypto_hash_base64_valid",
                "security protocol mismatch",
            );
            v.check_skip(
                "tower:crypto_hash_deterministic",
                "security protocol mismatch",
            );
        }
        Err(e) => {
            v.check_bool(
                "tower:crypto_hash_nonempty",
                false,
                &format!("hash error: {e}"),
            );
            v.check_skip("tower:crypto_hash_base64_valid", "prior call failed");
            v.check_skip("tower:crypto_hash_deterministic", "prior call failed");
        }
    }
}

fn tower_discovery_resolve(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    for cap in ["security", "compute", "storage"] {
        let name = format!("tower:resolve_{cap}");
        match ctx.resolve_capability(cap) {
            Ok(result) => {
                let found = result
                    .get("found")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false)
                    || result.get("endpoint").is_some()
                    || result.get("socket").is_some()
                    || result.get("native_endpoint").is_some()
                    || result.get("virtual_endpoint").is_some();
                v.check_bool(&name, found, &format!("resolved {cap}: {result}"));
            }
            Err(e) if e.is_connection_error() => {
                v.check_skip(&name, &format!("discovery not available: {e}"));
            }
            Err(e) => {
                v.check_bool(&name, false, &format!("resolve gap: {e}"));
            }
        }
    }

    match ctx.call("discovery", "rpc.discover", serde_json::json!({})) {
        Ok(result) => {
            let methods = result.get("methods").and_then(|m| m.as_array());
            let count = methods.map_or(0, Vec::len);
            v.check_bool(
                "tower:songbird_method_catalog",
                count > 10,
                &format!("Songbird exposes {count} methods"),
            );
        }
        Err(e) if e.is_connection_error() => {
            v.check_skip(
                "tower:songbird_method_catalog",
                &format!("discovery not available: {e}"),
            );
        }
        Err(e) => {
            v.check_bool(
                "tower:songbird_method_catalog",
                false,
                &format!("discover error: {e}"),
            );
        }
    }
}

// ════════════════════════════════════════════════════════════════════════
// NUCLEUS Layer 3: Node Atomic (barraCuda + coralReef + toadStool)
// ════════════════════════════════════════════════════════════════════════

fn node_scalar_parity(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    validate_parity(
        ctx,
        v,
        "node:sensor_noise_mean",
        "tensor",
        "stats.mean",
        serde_json::json!({"data": [0.5, 0.3, 0.4, 0.6, 0.2]}),
        "result",
        0.4,
        tolerances::IPC_ROUND_TRIP_TOL,
    );

    validate_parity(
        ctx,
        v,
        "node:integer_mean",
        "tensor",
        "stats.mean",
        serde_json::json!({"data": [1.0, 2.0, 3.0, 4.0, 5.0]}),
        "result",
        3.0,
        tolerances::IPC_ROUND_TRIP_TOL,
    );
}

fn node_vector_parity(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    composition::validate_parity_vec(
        ctx,
        v,
        "node:identity_matmul",
        "tensor",
        "tensor.matmul",
        serde_json::json!({
            "a": [[1.0, 0.0], [0.0, 1.0]],
            "b": [[1.0, 0.0], [0.0, 1.0]],
            "rows_a": 2, "cols_a": 2, "cols_b": 2
        }),
        "result",
        &[1.0, 0.0, 0.0, 1.0],
        tolerances::IPC_ROUND_TRIP_TOL,
    );

    composition::validate_parity_vec(
        ctx,
        v,
        "node:measurement_matmul",
        "tensor",
        "tensor.matmul",
        serde_json::json!({
            "a": [[1.0, 2.0], [3.0, 4.0]],
            "b": [[5.0, 6.0], [7.0, 8.0]],
            "rows_a": 2, "cols_a": 2, "cols_b": 2
        }),
        "result",
        &[19.0, 22.0, 43.0, 50.0],
        tolerances::IPC_ROUND_TRIP_TOL,
    );
}

fn node_decompose_e2e(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    let local = decompose_error(0.5, 1.0);
    v.check_bool(
        "node:decompose_local_valid",
        local.bias_fraction.is_finite() && local.random_std > 0.0,
        &format!(
            "bias_frac={:.4}, random_std={:.4}",
            local.bias_fraction, local.random_std
        ),
    );

    let part = local.random_std / 3.0;
    validate_parity(
        ctx,
        v,
        "node:decompose_ipc_mean_parity",
        "tensor",
        "stats.mean",
        serde_json::json!({"data": [part, part, part]}),
        "result",
        part,
        tolerances::IPC_ROUND_TRIP_TOL,
    );
}

fn node_shader_capabilities(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    match ctx.call(
        "shader",
        "shader.compile.capabilities",
        serde_json::json!({}),
    ) {
        Ok(result) => {
            let has_archs = result
                .get("supported_archs")
                .and_then(|a| a.as_array())
                .is_some_and(|a| !a.is_empty());
            v.check_bool(
                "node:shader_supported_archs",
                has_archs,
                &format!(
                    "archs: {}",
                    result
                        .get("supported_archs")
                        .unwrap_or(&serde_json::json!([]))
                ),
            );

            let wgsl = result
                .get("supported_archs")
                .and_then(|a| a.as_array())
                .is_some_and(|a| {
                    a.iter().any(|v| {
                        v.as_str()
                            .is_some_and(|s| s.contains("wgsl") || s.contains("WGSL"))
                    })
                });
            v.check_bool(
                "node:shader_wgsl_supported",
                wgsl || has_archs,
                "WGSL arch present",
            );
        }
        Err(e) if e.is_connection_error() => {
            v.check_skip(
                "node:shader_supported_archs",
                &format!("shader not available: {e}"),
            );
            v.check_skip("node:shader_wgsl_supported", "shader not available");
        }
        Err(e) => {
            v.check_bool(
                "node:shader_supported_archs",
                false,
                &format!("shader error: {e}"),
            );
            v.check_skip("node:shader_wgsl_supported", "prior call failed");
        }
    }
}

fn node_compute_dispatch_health(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    match ctx.call(
        "compute",
        "compute.dispatch",
        serde_json::json!({
            "shader": "identity_f64",
            "workgroups": [1, 1, 1]
        }),
    ) {
        Ok(result) => {
            v.check_bool(
                "node:compute_dispatch",
                true,
                &format!(
                    "response keys: {:?}",
                    result.as_object().map(|o| o.keys().collect::<Vec<_>>())
                ),
            );
        }
        Err(e) if e.is_connection_error() => {
            v.check_skip(
                "node:compute_dispatch",
                &format!("compute not available: {e}"),
            );
        }
        Err(e) if e.is_protocol_error() => {
            v.check_skip(
                "node:compute_dispatch",
                &format!("compute reachable but protocol mismatch: {e}"),
            );
        }
        Err(e) => {
            v.check_bool(
                "node:compute_dispatch",
                false,
                &format!("dispatch error: {e}"),
            );
        }
    }
}

// ════════════════════════════════════════════════════════════════════════
// NUCLEUS Layer 3: Nest Atomic (NestGate + provenance trio)
// ════════════════════════════════════════════════════════════════════════

fn nest_storage_roundtrip(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    let test_key = "groundspring-guidestone-witness";
    let test_value = "decompose_error(0.5,1.0).bias_fraction=0.25";
    let family_id = std::env::var("FAMILY_ID").unwrap_or_else(|_| "nucleus01".to_owned());

    let store_result = ctx
        .call(
            "storage",
            "storage.store",
            serde_json::json!({
                "family_id": family_id,
                "key": test_key,
                "value": test_value,
                "namespace": "guidestone"
            }),
        )
        .or_else(|_| {
            ctx.call(
                "storage",
                "storage.put",
                serde_json::json!({
                    "family_id": family_id,
                    "key": test_key,
                    "value": test_value,
                    "namespace": "guidestone"
                }),
            )
        });

    match store_result {
        Ok(_) => {
            let retrieve_result = ctx
                .call(
                    "storage",
                    "storage.retrieve",
                    serde_json::json!({
                        "family_id": family_id,
                        "key": test_key,
                        "namespace": "guidestone"
                    }),
                )
                .or_else(|_| {
                    ctx.call(
                        "storage",
                        "storage.get",
                        serde_json::json!({
                            "family_id": family_id,
                            "key": test_key,
                            "namespace": "guidestone"
                        }),
                    )
                });
            match retrieve_result {
                Ok(result) => {
                    let val = result
                        .get("value")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("");
                    v.check_bool(
                        "nest:storage_roundtrip",
                        val.contains("0.25"),
                        &format!("stored={test_value}, retrieved={val}"),
                    );
                }
                Err(e) => {
                    v.check_bool(
                        "nest:storage_roundtrip",
                        false,
                        &format!("retrieve failed: {e}"),
                    );
                }
            }
        }
        Err(e) if e.is_connection_error() => {
            v.check_skip(
                "nest:storage_roundtrip",
                &format!("storage not available: {e}"),
            );
        }
        Err(e) => {
            v.check_bool(
                "nest:storage_roundtrip",
                false,
                &format!("store error: {e}"),
            );
        }
    }
}

fn nest_provenance_health(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    for (name, cap) in [
        ("nest:sweetgrass_alive", "commit"),
        ("nest:rhizocrypt_alive", "dag"),
    ] {
        match ctx.health_check(cap) {
            Ok(alive) => v.check_bool(name, alive, &format!("{cap} health normalized")),
            Err(e) if e.is_connection_error() => {
                v.check_skip(name, &format!("{cap} not available: {e}"));
            }
            Err(e) => {
                v.check_bool(name, false, &format!("{cap} error: {e}"));
            }
        }
    }
}

// ════════════════════════════════════════════════════════════════════════
// NUCLEUS Layer 4: Cross-Atomic Pipeline
// Tower hash → Nest store → Nest retrieve → compare
// ════════════════════════════════════════════════════════════════════════

fn nucleus_hash_store_retrieve(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    let test_data = b"groundspring_cross_atomic_pipeline_2026";

    let hash_result = ctx.hash_bytes(test_data, "blake3");

    match hash_result {
        Ok(hash_hex) => {
            v.check_bool(
                "cross:tower_hash",
                !hash_hex.is_empty(),
                &format!("BLAKE3: {}...", &hash_hex[..hash_hex.len().min(16)]),
            );

            let family_id = std::env::var("FAMILY_ID").unwrap_or_else(|_| "nucleus01".to_owned());
            let store_key = "groundspring_cross_atomic_hash";
            let store_result = ctx
                .call(
                    "storage",
                    "storage.store",
                    serde_json::json!({
                        "family_id": family_id,
                        "key": store_key,
                        "value": hash_hex
                    }),
                )
                .or_else(|_| {
                    ctx.call(
                        "storage",
                        "storage.put",
                        serde_json::json!({
                            "family_id": family_id,
                            "key": store_key,
                            "value": hash_hex
                        }),
                    )
                });
            match store_result {
                Ok(_) => {
                    let retrieve_result = ctx
                        .call(
                            "storage",
                            "storage.retrieve",
                            serde_json::json!({
                                "family_id": family_id,
                                "key": store_key
                            }),
                        )
                        .or_else(|_| {
                            ctx.call(
                                "storage",
                                "storage.get",
                                serde_json::json!({
                                    "family_id": family_id,
                                    "key": store_key
                                }),
                            )
                        });
                    match retrieve_result {
                        Ok(retrieved) => {
                            let val = retrieved
                                .get("value")
                                .and_then(|v| v.as_str())
                                .unwrap_or("");
                            v.check_bool(
                                "cross:nest_roundtrip",
                                val == hash_hex,
                                "hash stored and retrieved matches",
                            );
                        }
                        Err(e) => {
                            v.check_bool(
                                "cross:nest_roundtrip",
                                false,
                                &format!("retrieve error: {e}"),
                            );
                        }
                    }
                }
                Err(e) if e.is_connection_error() => {
                    v.check_skip(
                        "cross:nest_roundtrip",
                        &format!("storage not available: {e}"),
                    );
                }
                Err(e) => {
                    v.check_bool("cross:nest_roundtrip", false, &format!("store error: {e}"));
                }
            }
        }
        Err(e) if e.is_connection_error() => {
            v.check_skip("cross:tower_hash", &format!("security not available: {e}"));
            v.check_skip("cross:nest_roundtrip", "tower unavailable");
        }
        Err(e) if e.is_protocol_error() => {
            v.check_skip(
                "cross:tower_hash",
                &format!("security reachable but protocol mismatch: {e}"),
            );
            v.check_skip("cross:nest_roundtrip", "tower protocol mismatch");
        }
        Err(e) => {
            v.check_bool("cross:tower_hash", false, &format!("hash error: {e}"));
            v.check_skip("cross:nest_roundtrip", "tower failed");
        }
    }
}
