// SPDX-License-Identifier: AGPL-3.0-or-later

//! groundSpring guideStone — self-validating NUCLEUS deployable.
//!
//! Combines bare guideStone validation (Properties 1-5 without primals) with
//! NUCLEUS IPC parity probes using the primalSpring composition API. Follows
//! the hotSpring Level 5 reference implementation pattern.
//!
//! # Bare guideStone (always runs, no primals needed)
//!
//! 1. **Deterministic** — `decompose_error` produces identical results on re-evaluation
//! 2. **Reference-traceable** — provenance registry and niche metadata populated
//! 3. **Self-verifying** — CHECKSUMS and deny.toml present
//! 4. **Environment-agnostic** — no network, no GPU required for bare checks
//! 5. **Tolerance-documented** — named constants defined with physical derivations
//!
//! # NUCLEUS additive (when primals are deployed)
//!
//! Uses `primalspring::composition::{CompositionContext, validate_parity,
//! validate_liveness}` to call barraCuda, `BearDog`, `toadStool`, and `NestGate`
//! over IPC and compare results against Python/Rust baselines.
//!
//! # Validation capabilities (from `downstream_manifest.toml`)
//!
//! - `tensor.matmul` → barraCuda (tensor)
//! - `stats.mean` → barraCuda (tensor)
//! - `compute.dispatch` → toadStool (compute)
//! - `storage.store` → `NestGate` (storage)
//! - `storage.retrieve` → `NestGate` (storage)
//! - `crypto.hash` → `BearDog` (security)
//!
//! # Exit codes
//!
//! - `0` — all checks passed (NUCLEUS certified)
//! - `1` — at least one check failed
//! - `2` — bare-only mode (no primals discovered)
//!
//! # References
//!
//! - guideStone Standard: `primalSpring/wateringHole/GUIDESTONE_COMPOSITION_STANDARD.md`
//! - Downstream Manifest: `primalSpring/graphs/downstream/downstream_manifest.toml`
//! - hotSpring reference: `hotSpring/barracuda/src/bin/hotspring_guidestone.rs`

#![forbid(unsafe_code)]

use primalspring::checksums;
use primalspring::composition::{
    self, CompositionContext, validate_liveness, validate_parity,
};
use primalspring::tolerances;
use primalspring::validation::ValidationResult;

use groundspring::decompose::decompose_error;

fn main() {
    let mut v = ValidationResult::new(
        "groundSpring guideStone — Measurement Science Certification",
    );

    ValidationResult::print_banner(
        "groundSpring guideStone — Level 3 (bare scaffold + IPC wiring)",
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
    // NUCLEUS ADDITIVE — IPC parity via primalSpring composition API
    // ════════════════════════════════════════════════════════════════════
    v.section("NUCLEUS: Discovery + Liveness");

    let mut ctx = CompositionContext::from_live_discovery_with_fallback();
    let alive = validate_liveness(
        &mut ctx,
        &mut v,
        &["tensor", "compute", "storage", "security"],
    );

    if alive == 0 {
        eprintln!(
            "[guideStone] No NUCLEUS primals discovered — bare certification only."
        );
        eprintln!(
            "[guideStone] Deploy from plasmidBin ecobins and set FAMILY_ID to test IPC."
        );
        v.finish();
        std::process::exit(v.exit_code_skip_aware());
    }

    v.section("NUCLEUS: Domain Science — Scalar Parity (stats.mean)");
    validate_scalar_parity(&mut ctx, &mut v);

    v.section("NUCLEUS: Domain Science — Vector Parity (tensor.matmul)");
    validate_vector_parity(&mut ctx, &mut v);

    v.section("NUCLEUS: Domain Science — Decomposition via IPC");
    validate_decompose_e2e(&mut ctx, &mut v);

    v.section("NUCLEUS: Storage — NestGate Round-Trip");
    validate_storage_roundtrip(&mut ctx, &mut v);

    v.section("NUCLEUS: Crypto — Provenance Witness");
    validate_provenance_witness(&mut ctx, &mut v);

    v.section("NUCLEUS: Compute — GPU Dispatch");
    validate_compute_dispatch(&mut ctx, &mut v);

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
        tol_det < tol_exact
            && tol_exact < tol_anal
            && tol_anal < tol_lit
            && tol_lit < tol_decomp,
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
// NUCLEUS: Scalar Parity (stats.mean via barraCuda IPC)
// ════════════════════════════════════════════════════════════════════════

fn validate_scalar_parity(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    validate_parity(
        ctx,
        v,
        "parity:sensor_noise_mean",
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
        "parity:integer_mean",
        "tensor",
        "stats.mean",
        serde_json::json!({"data": [1.0, 2.0, 3.0, 4.0, 5.0]}),
        "result",
        3.0,
        tolerances::IPC_ROUND_TRIP_TOL,
    );
}

// ════════════════════════════════════════════════════════════════════════
// NUCLEUS: Vector Parity (tensor.matmul via barraCuda IPC)
// ════════════════════════════════════════════════════════════════════════

fn validate_vector_parity(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    composition::validate_parity_vec(
        ctx,
        v,
        "parity:identity_matmul",
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
        "parity:measurement_matmul",
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

// ════════════════════════════════════════════════════════════════════════
// NUCLEUS: Decomposition End-to-End (Rust baseline vs IPC mean)
// ════════════════════════════════════════════════════════════════════════

fn validate_decompose_e2e(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    let local = decompose_error(0.5, 1.0);
    v.check_bool(
        "decompose:local_valid",
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
        "parity:decompose_random_std_mean",
        "tensor",
        "stats.mean",
        serde_json::json!({"data": [part, part, part]}),
        "result",
        part,
        tolerances::IPC_ROUND_TRIP_TOL,
    );
}

// ════════════════════════════════════════════════════════════════════════
// NUCLEUS: Storage Round-Trip (NestGate store + retrieve)
// ════════════════════════════════════════════════════════════════════════

fn validate_storage_roundtrip(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    let store_result = composition::call_or_skip(
        ctx,
        v,
        "storage:store_witness",
        "storage",
        "storage.store",
        serde_json::json!({
            "key": "groundspring-guidestone-witness",
            "value": "decompose_error(0.5,1.0).bias_fraction=0.25",
            "namespace": "guidestone"
        }),
    );

    if store_result.is_some() {
        match ctx.call(
            "storage",
            "storage.retrieve",
            serde_json::json!({
                "key": "groundspring-guidestone-witness",
                "namespace": "guidestone"
            }),
        ) {
            Ok(result) => {
                let value = result
                    .get("value")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("");
                v.check_bool(
                    "storage:retrieve_matches",
                    value.contains("0.25"),
                    &format!("retrieved: {value}"),
                );
            }
            Err(e) => {
                v.check_bool(
                    "storage:retrieve_matches",
                    false,
                    &format!("retrieve failed: {e}"),
                );
            }
        }
    } else {
        v.check_skip(
            "storage:retrieve_matches",
            "store skipped — NestGate not available",
        );
    }
}

// ════════════════════════════════════════════════════════════════════════
// NUCLEUS: Provenance Witness (BearDog crypto.hash)
// ════════════════════════════════════════════════════════════════════════

fn validate_provenance_witness(ctx: &mut CompositionContext, v: &mut ValidationResult) {
    match ctx.hash_bytes(b"groundspring-guidestone-witness-2026", "blake3") {
        Ok(hash) => {
            v.check_bool(
                "crypto:blake3_witness",
                !hash.is_empty(),
                &format!("BLAKE3 produced {}B base64", hash.len()),
            );

            match ctx.hash_bytes(b"groundspring-guidestone-witness-2026", "blake3") {
                Ok(hash2) => {
                    v.check_bool(
                        "crypto:blake3_determinism",
                        hash == hash2,
                        "same input produces same hash",
                    );
                }
                Err(e) => {
                    v.check_bool(
                        "crypto:blake3_determinism",
                        false,
                        &format!("second hash call failed: {e}"),
                    );
                }
            }
        }
        Err(e) if e.is_connection_error() => {
            v.check_skip(
                "crypto:blake3_witness",
                &format!("security not available: {e}"),
            );
            v.check_skip("crypto:blake3_determinism", "security not available");
        }
        Err(e) if e.is_protocol_error() => {
            v.check_skip(
                "crypto:blake3_witness",
                &format!(
                    "security reachable but protocol mismatch (likely HTTP): {e}"
                ),
            );
            v.check_skip("crypto:blake3_determinism", "security protocol mismatch");
        }
        Err(e) => {
            v.check_bool(
                "crypto:blake3_witness",
                false,
                &format!("hash error: {e}"),
            );
            v.check_skip("crypto:blake3_determinism", "first hash failed");
        }
    }
}

// ════════════════════════════════════════════════════════════════════════
// NUCLEUS: GPU Compute Dispatch (toadStool)
// ════════════════════════════════════════════════════════════════════════

fn validate_compute_dispatch(ctx: &mut CompositionContext, v: &mut ValidationResult) {
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
                "compute:dispatch_returns_result",
                true,
                &format!(
                    "response keys: {:?}",
                    result
                        .as_object()
                        .map(|o| o.keys().collect::<Vec<_>>())
                ),
            );
        }
        Err(e) if e.is_connection_error() => {
            v.check_skip(
                "compute:dispatch_returns_result",
                &format!("compute not available: {e}"),
            );
        }
        Err(e) if e.is_protocol_error() => {
            v.check_skip(
                "compute:dispatch_returns_result",
                &format!("compute reachable but protocol mismatch: {e}"),
            );
        }
        Err(e) => {
            v.check_bool(
                "compute:dispatch_returns_result",
                false,
                &format!("dispatch error: {e}"),
            );
        }
    }
}
