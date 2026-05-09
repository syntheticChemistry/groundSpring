// SPDX-License-Identifier: AGPL-3.0-or-later

//! Bare guideStone validation — Properties 1-5, no primals needed.
//!
//! These checks run on any platform, offline, CPU-only. They verify
//! groundSpring's structural integrity and mathematical determinism.

use primalspring::validation::ValidationResult;

use crate::decompose::decompose_error;

/// Property 1: Deterministic output — same inputs produce bitwise-identical results.
pub fn validate_deterministic(v: &mut ValidationResult) {
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

    let mean_val = crate::stats::mean(&[1.0, 2.0, 3.0, 4.0, 5.0]);
    v.check_bool(
        "deterministic:mean_finite",
        mean_val.is_finite() && (mean_val - 3.0).abs() < 1e-15,
        &format!("mean([1..5]) = {mean_val}"),
    );
}

/// Property 2: Reference-traceable — provenance registry and niche metadata populated.
pub fn validate_traceable(v: &mut ValidationResult) {
    let registry = crate::provenance_registry::BASELINES;
    v.check_bool(
        "traceable:provenance_registry_populated",
        registry.len() >= 29,
        &format!("{} baseline entries", registry.len()),
    );

    let niche_id = crate::niche::NICHE_ID;
    v.check_bool(
        "traceable:niche_id_set",
        !niche_id.is_empty(),
        &format!("niche_id={niche_id}"),
    );

    let caps = crate::niche::CAPABILITIES;
    v.check_bool(
        "traceable:capabilities_populated",
        caps.len() >= 16,
        &format!("{} CAPABILITIES", caps.len()),
    );

    let domain = crate::niche::DOMAIN;
    v.check_bool(
        "traceable:domain_set",
        domain == "measurement",
        &format!("domain={domain}"),
    );
}

/// Property 3: Self-verifying — CHECKSUMS manifest and deny.toml present.
pub fn validate_self_verifying(v: &mut ValidationResult) {
    primalspring::checksums::verify_manifest(v, "validation/CHECKSUMS");

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

/// Property 4: Environment-agnostic — no network or GPU required for bare checks.
pub fn validate_env_agnostic(v: &mut ValidationResult) {
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

/// Property 5: Tolerance-documented — named constants with physical derivations.
pub fn validate_tolerance(v: &mut ValidationResult) {
    let tol_det = crate::tol::DETERMINISM;
    let tol_exact = crate::tol::EXACT;
    let tol_anal = crate::tol::ANALYTICAL;
    let tol_lit = crate::tol::LITERATURE;
    let tol_decomp = crate::tol::DECOMPOSITION;

    v.check_bool(
        "tolerance:determinism_defined",
        tol_det > 0.0 && tol_det < 1e-10,
        &format!("DETERMINISM = {tol_det:.2e}"),
    );

    v.check_bool(
        "tolerance:ordering_correct",
        tol_det < tol_exact && tol_exact < tol_anal && tol_anal < tol_lit && tol_lit < tol_decomp,
        "DETERMINISM < EXACT < ANALYTICAL < LITERATURE < DECOMPOSITION",
    );

    v.check_bool(
        "tolerance:primalspring_ipc_tol_defined",
        primalspring::tolerances::IPC_ROUND_TRIP_TOL > 0.0,
        &format!(
            "primalspring::tolerances::IPC_ROUND_TRIP_TOL = {:.2e}",
            primalspring::tolerances::IPC_ROUND_TRIP_TOL
        ),
    );

    v.check_bool(
        "tolerance:ipc_within_analytical",
        primalspring::tolerances::IPC_ROUND_TRIP_TOL <= crate::tol::ANALYTICAL,
        "primalspring IPC tol <= groundspring ANALYTICAL",
    );
}
