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

use groundspring_validate::guidestone::{bare, cross, nest, node, tower};
use primalspring::composition::{CompositionContext, validate_liveness};
use primalspring::validation::ValidationResult;

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
    bare::validate_deterministic(&mut v);

    v.section("Bare guideStone: Property 2 — Reference-Traceable");
    bare::validate_traceable(&mut v);

    v.section("Bare guideStone: Property 3 — Self-Verifying");
    bare::validate_self_verifying(&mut v);

    v.section("Bare guideStone: Property 4 — Environment-Agnostic");
    bare::validate_env_agnostic(&mut v);

    v.section("Bare guideStone: Property 5 — Tolerance-Documented");
    bare::validate_tolerance_documented(&mut v);

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
    tower::tower_health(&mut ctx, &mut v);
    tower::tower_crypto_hash(&mut ctx, &mut v);
    tower::tower_discovery_resolve(&mut ctx, &mut v);

    // ── Node Atomic (barraCuda + coralReef + toadStool) ──────────────
    v.section("NUCLEUS Layer 3: Node Atomic (Compute Triangle)");
    node::node_scalar_parity(&mut ctx, &mut v);
    node::node_vector_parity(&mut ctx, &mut v);
    node::node_decompose_e2e(&mut ctx, &mut v);
    node::node_shader_capabilities(&mut ctx, &mut v);
    node::node_compute_dispatch_health(&mut ctx, &mut v);

    // ── Nest Atomic (NestGate + provenance trio) ─────────────────────
    v.section("NUCLEUS Layer 3: Nest Atomic (Storage + Provenance)");
    nest::nest_storage_roundtrip(&mut ctx, &mut v);
    nest::nest_provenance_health(&mut ctx, &mut v);

    // ════════════════════════════════════════════════════════════════════
    // NUCLEUS LAYER 4 — Cross-Atomic Pipeline
    // ════════════════════════════════════════════════════════════════════
    v.section("NUCLEUS Layer 4: Cross-Atomic Pipeline");
    cross::nucleus_hash_store_retrieve(&mut ctx, &mut v);

    v.finish();
    std::process::exit(v.exit_code());
}
