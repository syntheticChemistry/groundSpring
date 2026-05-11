// SPDX-License-Identifier: AGPL-3.0-or-later

//! NUCLEUS Layer 4: Cross-Atomic Pipeline — Tower hash → Nest roundtrip.

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

/// Full cross-atomic pipeline: hash via Tower → store in Nest → retrieve → verify.
pub fn nucleus_hash_store_retrieve(ctx: &mut CompositionContext, v: &mut ValidationResult) {
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
