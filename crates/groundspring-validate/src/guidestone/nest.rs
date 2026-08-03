// SPDX-License-Identifier: AGPL-3.0-or-later

//! NUCLEUS Layer 3: Nest Atomic — NestGate storage + provenance trio.

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

/// NestGate store → retrieve round-trip.
pub fn nest_storage_roundtrip(ctx: &mut CompositionContext, v: &mut ValidationResult) {
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

/// Provenance trio liveness (semantic + DAG provenance).
pub fn nest_provenance_health(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    for (name, cap) in [
        ("nest:provenance_semantic_alive", "commit"),
        ("nest:provenance_dag_alive", "dag"),
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
