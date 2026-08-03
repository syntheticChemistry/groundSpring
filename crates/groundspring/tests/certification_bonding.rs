// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Integration tests for the certification bonding check harness (Layer 5).
//!
//! Offline tests exercise skip-tolerant behavior when NUCLEUS primals are
//! unavailable. Live tests require deployed security, discovery, and tensor
//! roles and are marked `#[ignore]`.

#![cfg(feature = "certification")]
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use std::sync::Arc;

use groundspring::certification::{self, MAX_LAYER};
use primalspring::composition::CompositionContext;
use primalspring::validation::{CheckOutcome, NullSink, ValidationResult};

const BONDING_CHECK_PREFIX: &str = "bonding:";

fn quiet_result(experiment: &str) -> ValidationResult {
    ValidationResult::new(experiment).with_sink(Arc::new(NullSink))
}

fn run_bonding_harness() -> ValidationResult {
    let mut v = quiet_result("bonding harness integration");
    let mut ctx = CompositionContext::from_live_discovery_with_fallback();
    certification::certify_bonding(&mut ctx, &mut v);
    v
}

// ── Harness structure ────────────────────────────────────────────────

#[test]
fn bonding_checks_run_and_return_check_results() {
    let v = run_bonding_harness();

    assert!(
        !v.checks.is_empty(),
        "bonding harness should emit at least one CheckResult"
    );

    let bonding_checks: Vec<_> = v
        .checks
        .iter()
        .filter(|c| c.name.starts_with(BONDING_CHECK_PREFIX))
        .collect();
    assert!(
        !bonding_checks.is_empty(),
        "expected bonding:* checks, got: {:?}",
        v.checks.iter().map(|c| &c.name).collect::<Vec<_>>()
    );

    for check in &bonding_checks {
        assert!(
            !check.detail.is_empty(),
            "check {} should carry a non-empty detail message",
            check.name
        );
    }

    assert_eq!(
        v.evaluated() + v.skipped,
        u32::try_from(v.checks.len()).expect("check count fits u32"),
        "passed + failed + skipped should equal total checks recorded"
    );
}

#[test]
fn bonding_harness_covers_expected_check_names() {
    let v = run_bonding_harness();
    let names: Vec<&str> = v.checks.iter().map(|c| c.name.as_str()).collect();

    for expected in [
        "bonding:crypto_sign",
        "bonding:capability_list_populated",
        "bonding:method_describe",
        "bonding:mesh_resolve",
    ] {
        assert!(
            names.iter().any(|n| *n == expected),
            "missing expected bonding check {expected}; got {names:?}"
        );
    }
}

// ── Degraded: missing services (default CI) ──────────────────────────

#[test]
fn bonding_degraded_missing_services_skips_or_fails_without_panic() {
    let v = run_bonding_harness();

    // Without live primals, bonding checks should skip (connection errors)
    // rather than crash. Failures are acceptable when a reachable primal
    // returns an error, but we must never leave the harness in an empty state.
    assert!(
        v.skipped > 0 || v.failed > 0 || v.passed > 0,
        "bonding harness should record at least one outcome"
    );

    if v.skipped > 0 {
        assert!(
            v.checks.iter().any(|c| c.outcome == CheckOutcome::Skip),
            "degraded run should include skipped checks when primals are offline"
        );
    }
}

#[test]
fn certify_layer0_bare_passes_without_primals() {
    let result = certification::certify(0);

    assert!(
        result.all_passed(),
        "Layer 0 bare certification should pass offline (passed={}, failed={}, skipped={})",
        result.passed,
        result.failed,
        result.skipped
    );
    assert_eq!(result.failed, 0);
    assert!(result.passed > 0);
    assert!(
        result
            .checks
            .iter()
            .any(|c| c.name.starts_with("deterministic:")),
        "bare layer should include deterministic checks"
    );
}

#[test]
fn certify_layer5_includes_bonding_checks() {
    let result = certification::certify(MAX_LAYER);

    let bonding: Vec<_> = result
        .checks
        .iter()
        .filter(|c| c.name.starts_with(BONDING_CHECK_PREFIX))
        .collect();
    assert!(
        !bonding.is_empty(),
        "full certification at layer {MAX_LAYER} should run bonding checks"
    );
}

// ── Known-good: all services healthy ─────────────────────────────────
// Requires live NUCLEUS with security, discovery, and tensor roles.

#[test]
#[ignore = "requires running NUCLEUS with security, discovery, and tensor primals"]
fn bonding_all_services_healthy() {
    let v = run_bonding_harness();

    assert_eq!(
        v.failed,
        0,
        "bonding checks should not fail when all services are healthy: {:?}",
        v.checks
            .iter()
            .filter(|c| c.outcome == CheckOutcome::Fail)
            .collect::<Vec<_>>()
    );
    assert!(
        v.passed >= 4,
        "expected multiple bonding passes, got passed={} skipped={}",
        v.passed,
        v.skipped
    );

    for name in [
        "bonding:crypto_sign",
        "bonding:crypto_verify",
        "bonding:capability_list_populated",
        "bonding:mesh_resolve",
    ] {
        let check = v
            .checks
            .iter()
            .find(|c| c.name == name)
            .unwrap_or_else(|| panic!("missing check {name}"));
        assert!(
            check.passed(),
            "check {name} should pass with healthy services: {}",
            check.detail
        );
    }
}
