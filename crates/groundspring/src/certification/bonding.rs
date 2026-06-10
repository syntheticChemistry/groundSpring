// SPDX-License-Identifier: AGPL-3.0-or-later

//! NUCLEUS bonding certification — Layer 5.
//!
//! Validates that groundSpring can establish bonds with the NUCLEUS:
//! - Ionic bond attempt via `crypto.sign` + `crypto.verify` roundtrip
//! - Capability announcement verified via `capability.list` reflection
//! - `method.describe` introspection confirms measurement methods visible
//! - Mesh topology awareness via `ipc.resolve` with relay endpoints

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

/// Run Layer 5 bonding certification.
///
/// Requires live NUCLEUS with at minimum: BearDog (crypto), Songbird
/// (discovery), and biomeOS orchestrator.
pub fn certify_bonding(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    v.section("NUCLEUS Layer 5: Bonding Model");

    bonding_crypto_sign_verify(ctx, v);
    bonding_capability_reflection(ctx, v);
    bonding_method_introspection(ctx, v);
    bonding_mesh_topology(ctx, v);
}

/// Ionic bond: sign a challenge via BearDog, verify the signature.
fn bonding_crypto_sign_verify(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    let challenge = format!(
        "groundspring-bonding-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs())
    );

    match ctx.call(
        crate::primal_names::roles::SECURITY,
        "crypto.sign",
        serde_json::json!({
            "message": challenge,
            "algorithm": "ed25519",
        }),
    ) {
        Ok(sign_result) => {
            let signature = sign_result
                .get("signature")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            v.check_bool(
                "bonding:crypto_sign",
                !signature.is_empty(),
                &format!("Ed25519 signature: {}...", &signature[..signature.len().min(16)]),
            );

            if !signature.is_empty() {
                match ctx.call(
                    crate::primal_names::roles::SECURITY,
                    "crypto.verify",
                    serde_json::json!({
                        "message": challenge,
                        "signature": signature,
                        "algorithm": "ed25519",
                    }),
                ) {
                    Ok(verify_result) => {
                        let valid = verify_result
                            .get("valid")
                            .and_then(serde_json::Value::as_bool)
                            .unwrap_or(false);
                        v.check_bool(
                            "bonding:crypto_verify",
                            valid,
                            "signature verified against challenge",
                        );
                    }
                    Err(e) => {
                        v.check_bool(
                            "bonding:crypto_verify",
                            false,
                            &format!("verify error: {e}"),
                        );
                    }
                }
            } else {
                v.check_skip("bonding:crypto_verify", "no signature to verify");
            }
        }
        Err(e) if e.is_connection_error() => {
            v.check_skip("bonding:crypto_sign", &format!("security not available: {e}"));
            v.check_skip("bonding:crypto_verify", "security not available");
        }
        Err(e) => {
            v.check_bool("bonding:crypto_sign", false, &format!("sign error: {e}"));
            v.check_skip("bonding:crypto_verify", "sign failed");
        }
    }
}

/// Capability reflection: verify groundSpring's measurement capabilities
/// are visible in the NUCLEUS `capability.list` response.
fn bonding_capability_reflection(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    match ctx.call(
        crate::primal_names::roles::DISCOVERY,
        "capability.list",
        serde_json::json!({}),
    ) {
        Ok(result) => {
            let capabilities = result
                .get("capabilities")
                .and_then(serde_json::Value::as_array);
            let count = result
                .get("count")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0);

            v.check_bool(
                "bonding:capability_list_populated",
                count > 0 || capabilities.is_some_and(|c| !c.is_empty()),
                &format!("NUCLEUS reports {count} capabilities"),
            );

            if let Some(caps) = capabilities {
                let cap_strings: Vec<&str> = caps
                    .iter()
                    .filter_map(|c| c.as_str().or_else(|| c.get("name").and_then(|n| n.as_str())))
                    .collect();
                let has_measurement = cap_strings
                    .iter()
                    .any(|c| c.starts_with("measurement."));
                v.check_bool(
                    "bonding:measurement_capabilities_visible",
                    has_measurement,
                    &format!(
                        "measurement.* present in {} total capabilities",
                        cap_strings.len()
                    ),
                );
            } else {
                v.check_skip(
                    "bonding:measurement_capabilities_visible",
                    "capability.list returned no array",
                );
            }
        }
        Err(e) if e.is_connection_error() => {
            v.check_skip(
                "bonding:capability_list_populated",
                &format!("discovery not available: {e}"),
            );
            v.check_skip("bonding:measurement_capabilities_visible", "discovery not available");
        }
        Err(e) => {
            v.check_bool(
                "bonding:capability_list_populated",
                false,
                &format!("capability.list error: {e}"),
            );
            v.check_skip("bonding:measurement_capabilities_visible", "prior call failed");
        }
    }
}

/// Method introspection: call `method.describe` (barraCuda v0.4.0+)
/// to verify that measurement methods have runtime descriptions.
fn bonding_method_introspection(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    match ctx.call(
        "tensor",
        "method.describe",
        serde_json::json!({"method": "stats.mean"}),
    ) {
        Ok(result) => {
            let has_description = result.get("description").is_some()
                || result.get("name").is_some();
            v.check_bool(
                "bonding:method_describe",
                has_description,
                &format!(
                    "stats.mean introspection: {}",
                    result.get("description")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("(present)")
                ),
            );
        }
        Err(e) if e.is_connection_error() || e.is_protocol_error() => {
            v.check_skip(
                "bonding:method_describe",
                &format!("method.describe not available: {e}"),
            );
        }
        Err(e) => {
            v.check_bool(
                "bonding:method_describe",
                false,
                &format!("method.describe error: {e}"),
            );
        }
    }
}

/// Mesh topology: verify `ipc.resolve` returns topology-aware endpoints
/// (songBird Wave 107: MeshRelay endpoints).
fn bonding_mesh_topology(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    match ctx.call(
        crate::primal_names::roles::DISCOVERY,
        "ipc.resolve",
        serde_json::json!({"capability": "measurement"}),
    ) {
        Ok(result) => {
            let has_endpoint = result.get("endpoint").is_some()
                || result.get("socket").is_some()
                || result.get("relay").is_some();
            v.check_bool(
                "bonding:mesh_resolve",
                has_endpoint,
                &format!("measurement resolved: {result}"),
            );
        }
        Err(e) if e.is_connection_error() => {
            v.check_skip(
                "bonding:mesh_resolve",
                &format!("discovery not available: {e}"),
            );
        }
        Err(e) => {
            v.check_bool(
                "bonding:mesh_resolve",
                false,
                &format!("ipc.resolve error: {e}"),
            );
        }
    }
}
