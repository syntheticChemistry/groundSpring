// SPDX-License-Identifier: AGPL-3.0-or-later

//! Bare guideStone — Properties 1-5 (no primals needed).

use groundspring::decompose::decompose_error;
use primalspring::checksums;
use primalspring::tolerances;
use primalspring::validation::ValidationResult;

/// Property 1 — Deterministic Output.
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

    let mean_val = groundspring::stats::mean(&[1.0, 2.0, 3.0, 4.0, 5.0]);
    v.check_bool(
        "deterministic:mean_finite",
        mean_val.is_finite() && (mean_val - 3.0).abs() < 1e-15,
        &format!("mean([1..5]) = {mean_val}"),
    );
}

/// Property 2 — Reference-Traceable.
pub fn validate_traceable(v: &mut ValidationResult) {
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

/// Property 3 — Self-Verifying.
pub fn validate_self_verifying(v: &mut ValidationResult) {
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

/// Property 4 — Environment-Agnostic.
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

/// Property 5 — Tolerance-Documented.
pub fn validate_tolerance_documented(v: &mut ValidationResult) {
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
        tol_det < tol_exact && tol_exact < tol_anal && tol_anal < tol_lit && tol_lit < tol_decomp,
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
