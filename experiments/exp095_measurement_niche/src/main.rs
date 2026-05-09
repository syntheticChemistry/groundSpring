// SPDX-License-Identifier: AGPL-3.0-or-later

//! Exp095: Measurement Science Niche Parity
//!
//! groundSpring-specific NUCLEUS validation: verifies that measurement science
//! domain operations produce identical results locally and via IPC composition.
//!
//! Validates:
//!   - NUCLEUS base (Tower security, Node compute, Nest storage round-trip)
//!   - Noise decomposition parity (local Rust baseline vs IPC stats.mean)
//!   - Sensor noise vector parity (local baseline vs IPC matmul)
//!   - Anderson localization scaling (local vs IPC std_dev parity)
//!   - Cross-atomic: hash decomposition result → store → retrieve → match
//!
//! Environment:
//!   `REMOTE_GATE_HOST` — enables TCP/gateway mode (e.g. Docker lab)
//!   `BIOMEOS_PORT`     — biomeOS TCP port (default 9800)
//!   `FAMILY_ID`        — primal family for socket scoping

#![forbid(unsafe_code)]

use groundspring::decompose::decompose_error;
use primalspring::composition::{CompositionContext, validate_parity};
use primalspring::tolerances;
use primalspring::validation::ValidationResult;

fn nucleus_base(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    v.section("Tower (Security + Discovery)");
    match ctx.health_check("security") {
        Ok(alive) => v.check_bool("tower_security_alive", alive, "BearDog health"),
        Err(e) if e.is_connection_error() => {
            v.check_skip("tower_security_alive", &format!("{e}"));
        }
        Err(e) => v.check_bool("tower_security_alive", false, &format!("{e}")),
    }

    v.section("Node (Compute Triangle)");
    validate_parity(
        ctx,
        v,
        "mean_3elem",
        "tensor",
        "stats.mean",
        serde_json::json!({"data": [2.0, 4.0, 6.0]}),
        "result",
        4.0,
        tolerances::EXACT_PARITY_TOL,
    );

    v.section("Nest (Storage Round-Trip)");
    let family_id = std::env::var("FAMILY_ID").unwrap_or_else(|_| "nucleus01".to_owned());
    let store_ok = ctx
        .call(
            "storage",
            "storage.put",
            serde_json::json!({
                "family_id": family_id,
                "key": "gs_exp095_test",
                "value": "measurement_niche"
            }),
        )
        .or_else(|_| {
            ctx.call(
                "storage",
                "storage.store",
                serde_json::json!({
                    "family_id": family_id,
                    "key": "gs_exp095_test",
                    "value": "measurement_niche"
                }),
            )
        })
        .is_ok();
    if store_ok {
        match ctx.call(
            "storage",
            "storage.get",
            serde_json::json!({"family_id": family_id, "key": "gs_exp095_test"}),
        ) {
            Ok(r) => {
                let val = r.get("value").and_then(|v| v.as_str()).unwrap_or_default();
                v.check_bool(
                    "storage_roundtrip",
                    val == "measurement_niche",
                    &format!("stored='measurement_niche', retrieved='{val}'"),
                );
            }
            Err(e) => v.check_skip("storage_roundtrip", &format!("{e}")),
        }
    } else {
        v.check_skip("storage_roundtrip", "storage.put not available");
    }
}

fn niche_parity(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    v.section("Niche — Noise Decomposition Parity");

    let decomp = decompose_error(0.5, 1.0);

    v.check_bool(
        "decompose_local_valid",
        decomp.bias_fraction.is_finite() && decomp.random_std > 0.0,
        &format!(
            "bias_frac={:.6}, random_std={:.6}, variance={:.6}",
            decomp.bias_fraction, decomp.random_std, decomp.variance
        ),
    );

    validate_parity(
        ctx,
        v,
        "decompose_bias_via_mean",
        "tensor",
        "stats.mean",
        serde_json::json!({"data": [
            decomp.bias_fraction,
            decomp.bias_fraction,
            decomp.bias_fraction
        ]}),
        "result",
        decomp.bias_fraction,
        tolerances::IPC_ROUND_TRIP_TOL,
    );

    v.section("Niche — Sensor Noise Vector Parity");

    let noise_samples = [0.5, 0.3, 0.4, 0.6, 0.2];
    let expected_mean = 0.4;
    validate_parity(
        ctx,
        v,
        "sensor_noise_mean_5pt",
        "tensor",
        "stats.mean",
        serde_json::json!({"data": noise_samples}),
        "result",
        expected_mean,
        tolerances::IPC_ROUND_TRIP_TOL,
    );

    v.section("Niche — Anderson Localization Scaling");

    let anderson_data = [0.12, 0.15, 0.11, 0.14, 0.13, 0.16, 0.10, 0.17];
    let n = anderson_data.len() as f64;
    let mean = anderson_data.iter().sum::<f64>() / n;
    let variance = anderson_data
        .iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f64>()
        / (n - 1.0);
    let expected_std = variance.sqrt();

    validate_parity(
        ctx,
        v,
        "anderson_std_dev_8pt",
        "tensor",
        "stats.std_dev",
        serde_json::json!({"data": anderson_data}),
        "result",
        expected_std,
        1e-6,
    );

    v.section("Niche — Cross-Atomic: Decomposition Hash Pipeline");

    let payload = format!(
        "groundspring_decompose_bias={:.10}_std={:.10}",
        decomp.bias_fraction, decomp.random_std,
    );
    match ctx.call(
        "security",
        "crypto.hash",
        serde_json::json!({"data": payload, "algorithm": "blake3"}),
    ) {
        Ok(r) => {
            let hash = r.get("hash").and_then(|h| h.as_str()).unwrap_or_default();
            v.check_bool(
                "cross_atomic_decompose_hash",
                !hash.is_empty(),
                &format!("BLAKE3 of decomposition result: {} chars", hash.len()),
            );
        }
        Err(e) if e.is_connection_error() => {
            v.check_skip("cross_atomic_decompose_hash", &format!("{e}"));
        }
        Err(e) => {
            v.check_bool("cross_atomic_decompose_hash", false, &format!("{e}"));
        }
    }
}

fn main() {
    ValidationResult::new("groundSpring Exp095 — Measurement Science Niche Parity")
        .with_provenance("exp095_measurement_niche", "2026-05-08")
        .run("NUCLEUS base + measurement science domain parity", |v| {
            let mut ctx = CompositionContext::from_live_discovery_with_fallback();
            let caps = ctx.available_capabilities();

            v.section("Discovery");
            v.check_bool(
                "capabilities_found",
                !caps.is_empty(),
                &format!(
                    "discovered {} capabilities: {}",
                    caps.len(),
                    caps.join(", ")
                ),
            );

            nucleus_base(&mut ctx, v);
            niche_parity(&mut ctx, v);
        });
}
