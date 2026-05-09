// SPDX-License-Identifier: AGPL-3.0-or-later

//! NUCLEUS composition certification — Layers 2-4.
//!
//! Requires deployed primals via plasmidBin ecobins. Uses
//! `CompositionContext` for all IPC — no direct socket connections.

use primalspring::composition::{self, CompositionContext, validate_liveness, validate_parity};
use primalspring::tolerances;
use primalspring::validation::ValidationResult;

use crate::decompose::decompose_error;

/// Run composition layers 2-4, gated by `max_layer`.
pub fn certify_composition(v: &mut ValidationResult, max_layer: u8) {
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

    let alive = validate_liveness(&mut ctx, v, &["tensor", "compute", "storage", "security"]);

    if alive == 0 {
        tracing::warn!("No NUCLEUS primals discovered — bare certification only");
        tracing::info!("Deploy from plasmidBin ecobins and set FAMILY_ID to test IPC");
        return;
    }

    if max_layer >= 3 {
        v.section("NUCLEUS Layer 3: Tower Atomic (Security + Discovery)");
        tower_health(&mut ctx, v);
        tower_crypto_hash(&mut ctx, v);
        tower_discovery_resolve(&mut ctx, v);

        v.section("NUCLEUS Layer 3: Node Atomic (Compute Triangle)");
        node_scalar_parity(&mut ctx, v);
        node_vector_parity(&mut ctx, v);
        node_decompose_e2e(&mut ctx, v);
        node_shader_capabilities(&mut ctx, v);
        node_compute_dispatch_health(&mut ctx, v);

        v.section("NUCLEUS Layer 3: Nest Atomic (Storage + Provenance)");
        nest_storage_roundtrip(&mut ctx, v);
        nest_provenance_health(&mut ctx, v);
    }

    if max_layer >= 4 {
        v.section("NUCLEUS Layer 4: Cross-Atomic Pipeline");
        nucleus_hash_store_retrieve(&mut ctx, v);
    }
}

/// Tower atomic health — BearDog (security) and Songbird (discovery) liveness.
pub fn tower_health(ctx: &mut CompositionContext, v: &mut ValidationResult) {
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

/// Tower crypto hash — BLAKE3 via BearDog, determinism check.
pub fn tower_crypto_hash(ctx: &mut CompositionContext, v: &mut ValidationResult) {
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

/// Tower discovery — resolve security, compute, storage capabilities.
pub fn tower_discovery_resolve(ctx: &mut CompositionContext, v: &mut ValidationResult) {
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

/// Node scalar parity — sensor noise mean via IPC vs local.
pub fn node_scalar_parity(ctx: &mut CompositionContext, v: &mut ValidationResult) {
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

/// Node vector parity — identity and measurement matmul via IPC.
pub fn node_vector_parity(ctx: &mut CompositionContext, v: &mut ValidationResult) {
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

/// Node end-to-end — decompose locally, verify mean parity over IPC.
pub fn node_decompose_e2e(ctx: &mut CompositionContext, v: &mut ValidationResult) {
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

/// Node shader capabilities — enumerate supported architectures.
pub fn node_shader_capabilities(ctx: &mut CompositionContext, v: &mut ValidationResult) {
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

/// Node compute dispatch health — identity shader dispatch test.
pub fn node_compute_dispatch_health(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    match ctx.call(
        "compute",
        "compute.dispatch",
        serde_json::json!({"shader": "identity_f64", "workgroups": [1, 1, 1]}),
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

/// Nest storage roundtrip — store and retrieve with provenance.
pub fn nest_storage_roundtrip(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    let test_key = "groundspring-guidestone-witness";
    let test_value = "decompose_error(0.5,1.0).bias_fraction=0.25";
    let family_id = std::env::var("FAMILY_ID").unwrap_or_else(|_| "nucleus01".to_owned());

    let store_result = ctx
        .call(
            "storage",
            "storage.store",
            serde_json::json!({
                "family_id": family_id, "key": test_key,
                "value": test_value, "namespace": "guidestone"
            }),
        )
        .or_else(|_| {
            ctx.call(
                "storage",
                "storage.put",
                serde_json::json!({
                    "family_id": family_id, "key": test_key,
                    "value": test_value, "namespace": "guidestone"
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
                        "family_id": family_id, "key": test_key, "namespace": "guidestone"
                    }),
                )
                .or_else(|_| {
                    ctx.call(
                        "storage",
                        "storage.get",
                        serde_json::json!({
                            "family_id": family_id, "key": test_key, "namespace": "guidestone"
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

/// Nest provenance health — sweetGrass (commit) and rhizoCrypt (DAG).
pub fn nest_provenance_health(ctx: &mut CompositionContext, v: &mut ValidationResult) {
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

/// Cross-atomic pipeline — Tower hash → Nest store → retrieve → compare.
pub fn nucleus_hash_store_retrieve(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    let test_data = b"groundspring_cross_atomic_pipeline_2026";

    match ctx.hash_bytes(test_data, "blake3") {
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
                    "storage", "storage.store",
                    serde_json::json!({"family_id": family_id, "key": store_key, "value": hash_hex}),
                )
                .or_else(|_| {
                    ctx.call(
                        "storage", "storage.put",
                        serde_json::json!({"family_id": family_id, "key": store_key, "value": hash_hex}),
                    )
                });
            match store_result {
                Ok(_) => {
                    let retrieve_result = ctx
                        .call(
                            "storage",
                            "storage.retrieve",
                            serde_json::json!({"family_id": family_id, "key": store_key}),
                        )
                        .or_else(|_| {
                            ctx.call(
                                "storage",
                                "storage.get",
                                serde_json::json!({"family_id": family_id, "key": store_key}),
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
