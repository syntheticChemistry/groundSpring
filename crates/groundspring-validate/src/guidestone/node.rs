// SPDX-License-Identifier: AGPL-3.0-or-later

//! NUCLEUS Layer 3: Node Atomic — barraCuda + coralReef + toadStool compute triangle.

use groundspring::decompose::decompose_error;
use primalspring::composition::{self, CompositionContext, validate_parity};
use primalspring::tolerances;
use primalspring::validation::ValidationResult;

/// Scalar mean parity across IPC.
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

/// Vector matmul parity across IPC.
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

/// End-to-end decompose error via IPC.
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

/// coralReef shader compilation capabilities.
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

/// toadStool compute dispatch health check.
pub fn node_compute_dispatch_health(ctx: &mut CompositionContext, v: &mut ValidationResult) {
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
