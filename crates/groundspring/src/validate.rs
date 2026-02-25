// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Validation harness following the hotSpring pattern.
//!
//! Provides check functions with pass/fail counters and exit code support.

#![allow(clippy::must_use_candidate)]

use std::sync::atomic::{AtomicU32, Ordering};

static PASS_COUNT: AtomicU32 = AtomicU32::new(0);
static FAIL_COUNT: AtomicU32 = AtomicU32::new(0);

/// Reset pass/fail counters.
pub fn reset() {
    PASS_COUNT.store(0, Ordering::Relaxed);
    FAIL_COUNT.store(0, Ordering::Relaxed);
}

/// Current pass count.
#[must_use]
pub fn passes() -> u32 {
    PASS_COUNT.load(Ordering::Relaxed)
}

/// Current fail count.
#[must_use]
pub fn fails() -> u32 {
    FAIL_COUNT.load(Ordering::Relaxed)
}

fn record(passed: bool) -> bool {
    if passed {
        PASS_COUNT.fetch_add(1, Ordering::Relaxed);
    } else {
        FAIL_COUNT.fetch_add(1, Ordering::Relaxed);
    }
    passed
}

/// Check that `computed` is within `tol` of `expected`.
#[must_use]
pub fn check_approx(label: &str, computed: f64, expected: f64, tol: f64) -> bool {
    let diff = (computed - expected).abs();
    let ok = diff <= tol;
    let status = if ok { "PASS" } else { "FAIL" };
    println!(
        "  [{status}] {label}: {computed:.6} (expected {expected:.6}, tol {tol:.6}, diff {diff:.6})"
    );
    record(ok)
}

/// Check that `computed` falls within [`low`, `high`].
pub fn check_range(label: &str, computed: f64, low: f64, high: f64) -> bool {
    let ok = (low..=high).contains(&computed);
    let status = if ok { "PASS" } else { "FAIL" };
    println!("  [{status}] {label}: {computed:.6} (expected [{low:.6}, {high:.6}])");
    record(ok)
}

/// Check that `computed` <= `maximum`.
pub fn check_max(label: &str, computed: f64, maximum: f64) -> bool {
    let ok = computed <= maximum;
    let status = if ok { "PASS" } else { "FAIL" };
    println!("  [{status}] {label}: {computed:.6} (max {maximum:.6})");
    record(ok)
}

/// Check that a boolean condition holds.
pub fn check_true(label: &str, condition: bool) -> bool {
    let status = if condition { "PASS" } else { "FAIL" };
    println!("  [{status}] {label}");
    record(condition)
}

/// Print summary and return process exit code (0 = all pass, 1 = any fail).
pub fn summary(experiment_name: &str) -> i32 {
    let p = passes();
    let f = fails();
    let total = p + f;
    println!("{}", "=".repeat(72));
    println!("{experiment_name}");
    println!("TOTAL: {p}/{total} PASS, {f}/{total} FAIL");
    println!("{}", "=".repeat(72));
    i32::from(f != 0)
}
