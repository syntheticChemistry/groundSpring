// SPDX-License-Identifier: AGPL-3.0-or-later

//! groundSpring certification engine — absorbed guideStone organelle.
//!
//! Validates that groundSpring is correctly deployed and functioning,
//! from bare structural checks (no primals needed) through full
//! NUCLEUS composition parity.
//!
//! # Layers
//!
//! - **L0 (Bare)**: Deterministic output, reference-traceable, self-verifying,
//!   environment-agnostic, tolerance-documented. No IPC needed.
//! - **L2 (Atomic Health)**: Liveness probes for all NUCLEUS tiers.
//! - **L3 (Capability Parity)**: Scalar + vector math, storage round-trip.
//! - **L4 (Cross-Atomic Pipeline)**: Tower hash → Nest store → retrieve → match.
//! - **L5 (Bonding Model)**: Crypto sign/verify, capability reflection,
//!   method introspection, mesh topology awareness.
//!
//! # Exit codes
//!
//! - `0` — all checks passed (NUCLEUS certified)
//! - `1` — at least one check failed
//! - `2` — bare-only mode (no primals discovered)

mod bare;
mod bonding;
mod composition;

pub use bare::{
    validate_deterministic, validate_env_agnostic, validate_self_verifying, validate_tolerance,
    validate_traceable,
};
pub use bonding::certify_bonding;
pub use composition::{
    certify_composition, nest_provenance_health, nest_storage_roundtrip,
    node_compute_dispatch_health, node_decompose_e2e, node_scalar_parity, node_shader_capabilities,
    node_vector_parity, nucleus_hash_store_retrieve, tower_crypto_hash, tower_discovery_resolve,
    tower_health,
};

/// Maximum certification layer supported by groundSpring.
pub const MAX_LAYER: u8 = 5;

/// Run the full certification engine up to the specified layer.
///
/// - Layer 0: Bare guideStone (Properties 1-5, no primals needed)
/// - Layer 2: Atomic health (liveness probes)
/// - Layer 3: Capability parity (math, storage)
/// - Layer 4: Cross-atomic pipeline
/// - Layer 5: Bonding model (crypto, reflection, introspection, mesh)
///
/// Returns the `ValidationResult` after all layers complete.
#[must_use]
pub fn certify(max_layer: u8) -> primalspring::validation::ValidationResult {
    let mut v = primalspring::validation::ValidationResult::new(
        "groundSpring guideStone — Measurement Science Certification",
    );

    primalspring::validation::ValidationResult::print_banner(&format!(
        "groundSpring guideStone — Layer {max_layer} certification"
    ));

    v.section("Bare guideStone: Property 1 — Deterministic Output");
    validate_deterministic(&mut v);

    v.section("Bare guideStone: Property 2 — Reference-Traceable");
    validate_traceable(&mut v);

    v.section("Bare guideStone: Property 3 — Self-Verifying");
    validate_self_verifying(&mut v);

    v.section("Bare guideStone: Property 4 — Environment-Agnostic");
    validate_env_agnostic(&mut v);

    v.section("Bare guideStone: Property 5 — Tolerance-Documented");
    validate_tolerance(&mut v);

    if max_layer >= 2 {
        certify_composition(&mut v, max_layer);
    }

    if max_layer >= 5 {
        let mut ctx =
            primalspring::composition::CompositionContext::from_live_discovery_with_fallback();
        certify_bonding(&mut ctx, &mut v);
    }

    v.finish();
    v
}
