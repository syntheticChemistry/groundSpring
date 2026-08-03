// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Integration tests for validation scenario execution.
//!
//! Rust-tier scenarios run fully offline. Live gate-deployment tests require
//! proto-nucleate primals over IPC and are marked `#[ignore]`.

#![cfg(feature = "validation")]
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use std::sync::Arc;

use groundspring::validation::{Tier, Track, build_registry};
use primalspring::composition::CompositionContext;
use primalspring::validation::{CheckOutcome, NullSink, ValidationResult};

fn quiet_result(experiment: &str) -> ValidationResult {
    ValidationResult::new(experiment).with_sink(Arc::new(NullSink))
}

fn run_scenario(id: &str) -> ValidationResult {
    let registry = build_registry();
    let scenario = registry
        .find(id)
        .unwrap_or_else(|| panic!("scenario {id} not registered"));
    let mut v = quiet_result(id);
    let mut ctx = CompositionContext::from_live_discovery_with_fallback();
    (scenario.run)(&mut v, &mut ctx);
    v
}

// ── Registry ─────────────────────────────────────────────────────────

#[test]
fn registry_contains_gate_deployment_scenario() {
    let registry = build_registry();

    assert!(!registry.is_empty(), "registry should contain scenarios");

    let gate = registry
        .find("gate-deployment-validation")
        .expect("gate-deployment-validation scenario must be registered");
    assert_eq!(gate.meta.track, Track::CompositionParity);
    assert_eq!(gate.meta.tier, Tier::Live);
    assert!(!gate.meta.description.is_empty());
}

#[test]
fn registry_filter_by_tier_rust_includes_offline_scenarios() {
    let registry = build_registry();
    let rust_ids: Vec<&str> = registry
        .filter_by_tier(Tier::Rust)
        .map(|s| s.meta.id)
        .collect();

    assert!(
        rust_ids.contains(&"decompose-bias-variance"),
        "Rust tier filter should include decompose-bias-variance, got {rust_ids:?}"
    );
}

// ── Rust-tier: meaningful pass/fail ───────────────────────────────────

#[test]
fn decompose_scenario_produces_pass_results() {
    let v = run_scenario("decompose-bias-variance");

    assert_eq!(v.failed, 0, "decompose scenario should pass offline");
    assert!(v.passed >= 3, "expected at least 3 passing checks");
    assert_eq!(v.skipped, 0);

    for name in [
        "decompose:bias_fraction_valid",
        "decompose:pythagorean",
        "decompose:random_std_positive",
    ] {
        let check = v
            .checks
            .iter()
            .find(|c| c.name == name)
            .unwrap_or_else(|| panic!("missing check {name}"));
        assert!(check.passed(), "check {name} should pass: {}", check.detail);
        assert!(
            !check.detail.is_empty(),
            "check {name} should include a meaningful detail string"
        );
    }
}

#[test]
fn scenario_checks_distinguish_pass_fail_and_skip_outcomes() {
    let pass_v = run_scenario("decompose-bias-variance");
    assert!(
        pass_v
            .checks
            .iter()
            .all(|c| c.outcome == CheckOutcome::Pass),
        "Rust-tier decompose checks should all pass"
    );

    let gate_v = run_scenario("gate-deployment-validation");
    assert!(
        !gate_v.checks.is_empty(),
        "gate deployment scenario should emit checks even when primals are offline"
    );
    assert!(
        gate_v.checks.iter().any(|c| matches!(
            c.outcome,
            CheckOutcome::Pass | CheckOutcome::Skip | CheckOutcome::Fail
        )),
        "gate deployment should record explicit pass/fail/skip outcomes"
    );
}

// ── Gate deployment scenario ─────────────────────────────────────────

#[test]
fn gate_deployment_scenario_runs_and_records_checks() {
    let v = run_scenario("gate-deployment-validation");

    assert!(
        !v.checks.is_empty(),
        "gate deployment scenario should produce CheckResult items"
    );

    let gate_checks: Vec<_> = v
        .checks
        .iter()
        .filter(|c| c.name.starts_with("gate:"))
        .collect();
    assert!(
        !gate_checks.is_empty(),
        "expected gate:* checks, got: {:?}",
        v.checks.iter().map(|c| &c.name).collect::<Vec<_>>()
    );

    for check in &gate_checks {
        assert!(
            !check.detail.is_empty(),
            "gate check {} should include detail",
            check.name
        );
    }
}

#[test]
fn gate_deployment_offline_is_skip_tolerant() {
    let v = run_scenario("gate-deployment-validation");

    // Without biomeos+tarpc-ipc features, the scenario skips entirely.
    // With those features but no primals, individual gate:* checks skip.
    // Either way, offline CI must not hard-fail.
    if v.checks
        .iter()
        .any(|c| c.name == "gate:requires-biomeos-and-ipc")
    {
        assert_eq!(v.skipped, 1);
        assert_eq!(v.failed, 0);
    } else {
        assert_eq!(
            v.failed,
            0,
            "offline gate deployment should skip, not fail: {:?}",
            v.checks
                .iter()
                .filter(|c| c.outcome == CheckOutcome::Fail)
                .collect::<Vec<_>>()
        );
    }
}

// ── Live gate deployment ─────────────────────────────────────────────
// Requires proto-nucleate primals (beardog, barracuda, coralreef, toadstool,
// nestgate) reachable over tarpc IPC with biomeos feature enabled.

#[test]
#[ignore = "requires running proto-nucleate primals over biomeos/tarpc IPC"]
fn gate_deployment_all_primals_healthy() {
    temp_env::with_var("GROUNDSPRING_GATE_STRICT", Some("1"), || {
        let v = run_scenario("gate-deployment-validation");

        assert_eq!(
            v.failed,
            0,
            "gate deployment should pass with all primals healthy: {:?}",
            v.checks
                .iter()
                .filter(|c| c.outcome == CheckOutcome::Fail)
                .collect::<Vec<_>>()
        );
        assert!(
            v.passed >= 5,
            "expected gate IPC checks to pass, got passed={} skipped={}",
            v.passed,
            v.skipped
        );

        let aggregate = v
            .checks
            .iter()
            .find(|c| c.name == "gate:proto-nucleate-healthy")
            .expect("gate:proto-nucleate-healthy check should exist");
        assert!(
            aggregate.passed(),
            "aggregate health check should pass: {}",
            aggregate.detail
        );
    });
}
