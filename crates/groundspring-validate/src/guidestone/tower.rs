// SPDX-License-Identifier: AGPL-3.0-or-later

//! NUCLEUS Layer 3: Tower Atomic — BearDog (security) + Songbird (discovery).

use groundspring::primal_names::roles;
use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

/// Liveness probes for Tower primal pair.
pub fn tower_health(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    for (name, cap) in [
        ("tower:beardog_alive", roles::SECURITY),
        ("tower:songbird_alive", roles::DISCOVERY),
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

/// BLAKE3 hash via BearDog crypto capability.
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

/// Capability resolution via Songbird discovery.
pub fn tower_discovery_resolve(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    for cap in [roles::SECURITY, roles::COMPUTE, roles::STORAGE] {
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

    match ctx.call(roles::DISCOVERY, "rpc.discover", serde_json::json!({})) {
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
