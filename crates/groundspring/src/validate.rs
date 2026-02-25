// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Validation harness following the hotSpring pattern.
//!
//! [`ValidationHarness`] tracks pass/fail counters and writes results to
//! any [`std::io::Write`] destination (defaults to stdout).
//!
//! # Example
//!
//! ```
//! use groundspring::validate::ValidationHarness;
//!
//! let mut h = ValidationHarness::stdout("demo");
//! h.check_approx("pi", 3.14159, std::f64::consts::PI, 1e-4);
//! assert_eq!(h.passes(), 1);
//! ```

use std::io::{self, Write};

/// Validation harness with independent pass/fail counters.
///
/// Output goes to the `Write` destination provided at construction.
/// Use [`stdout`](Self::stdout) for terminal output or
/// [`new`](Self::new) to supply a custom writer (e.g. `Vec<u8>`
/// for in-memory capture during tests).
pub struct ValidationHarness<W: Write = io::Stdout> {
    name: String,
    passes: u32,
    fails: u32,
    writer: W,
}

impl ValidationHarness<io::Stdout> {
    /// Create a harness that writes to stdout (the common case).
    #[must_use]
    pub fn stdout(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passes: 0,
            fails: 0,
            writer: io::stdout(),
        }
    }
}

impl<W: Write> ValidationHarness<W> {
    /// Create a harness with a custom writer.
    #[must_use]
    pub fn new(name: impl Into<String>, writer: W) -> Self {
        Self {
            name: name.into(),
            passes: 0,
            fails: 0,
            writer,
        }
    }

    /// Number of checks that passed.
    #[must_use]
    pub const fn passes(&self) -> u32 {
        self.passes
    }

    /// Number of checks that failed.
    #[must_use]
    pub const fn fails(&self) -> u32 {
        self.fails
    }

    const fn record(&mut self, passed: bool) -> bool {
        if passed {
            self.passes += 1;
        } else {
            self.fails += 1;
        }
        passed
    }

    /// Check that `computed` is within `tol` of `expected`.
    pub fn check_approx(&mut self, label: &str, computed: f64, expected: f64, tol: f64) -> bool {
        let diff = (computed - expected).abs();
        let ok = diff <= tol;
        let status = if ok { "PASS" } else { "FAIL" };
        let _ = writeln!(
            self.writer,
            "  [{status}] {label}: {computed:.6} \
             (expected {expected:.6}, tol {tol:.6}, diff {diff:.6})"
        );
        self.record(ok)
    }

    /// Check that `computed` falls within [`low`, `high`].
    pub fn check_range(&mut self, label: &str, computed: f64, low: f64, high: f64) -> bool {
        let ok = (low..=high).contains(&computed);
        let status = if ok { "PASS" } else { "FAIL" };
        let _ = writeln!(
            self.writer,
            "  [{status}] {label}: {computed:.6} (expected [{low:.6}, {high:.6}])"
        );
        self.record(ok)
    }

    /// Check that `computed` <= `maximum`.
    pub fn check_max(&mut self, label: &str, computed: f64, maximum: f64) -> bool {
        let ok = computed <= maximum;
        let status = if ok { "PASS" } else { "FAIL" };
        let _ = writeln!(
            self.writer,
            "  [{status}] {label}: {computed:.6} (max {maximum:.6})"
        );
        self.record(ok)
    }

    /// Check that `computed` >= `minimum`.
    pub fn check_min(&mut self, label: &str, computed: f64, minimum: f64) -> bool {
        let ok = computed >= minimum;
        let status = if ok { "PASS" } else { "FAIL" };
        let _ = writeln!(
            self.writer,
            "  [{status}] {label}: {computed:.6} (min {minimum:.6})"
        );
        self.record(ok)
    }

    /// Check that a boolean condition holds.
    pub fn check_true(&mut self, label: &str, condition: bool) -> bool {
        let status = if condition { "PASS" } else { "FAIL" };
        let _ = writeln!(self.writer, "  [{status}] {label}");
        self.record(condition)
    }

    /// Write summary and return process exit code (0 = all pass, 1 = any fail).
    #[must_use]
    pub fn summary(&mut self) -> i32 {
        let total = self.passes + self.fails;
        let sep = "=".repeat(72);
        let _ = writeln!(self.writer, "{sep}");
        let _ = writeln!(self.writer, "{}", self.name);
        let _ = writeln!(
            self.writer,
            "TOTAL: {}/{total} PASS, {}/{total} FAIL",
            self.passes, self.fails
        );
        let _ = writeln!(self.writer, "{sep}");
        i32::from(self.fails != 0)
    }
}

impl<W: Write> std::fmt::Debug for ValidationHarness<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ValidationHarness")
            .field("name", &self.name)
            .field("passes", &self.passes)
            .field("fails", &self.fails)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn harness(name: &str) -> ValidationHarness<Vec<u8>> {
        ValidationHarness::new(name, Vec::new())
    }

    #[test]
    fn new_harness_has_zero_counts() {
        let h = harness("test");
        assert_eq!(h.passes(), 0);
        assert_eq!(h.fails(), 0);
    }

    #[test]
    fn check_approx_pass() {
        let mut h = harness("test");
        assert!(h.check_approx("val", 1.005, 1.0, 0.01));
        assert_eq!(h.passes(), 1);
        assert_eq!(h.fails(), 0);
    }

    #[test]
    fn check_approx_fail() {
        let mut h = harness("test");
        assert!(!h.check_approx("val", 1.0, 1.5, 0.01));
        assert_eq!(h.passes(), 0);
        assert_eq!(h.fails(), 1);
    }

    #[test]
    fn check_range_boundaries() {
        let mut h = harness("test");
        assert!(h.check_range("at_low", 1.0, 1.0, 2.0));
        assert!(h.check_range("at_high", 2.0, 1.0, 2.0));
        assert!(!h.check_range("below", 0.5, 1.0, 2.0));
        assert!(!h.check_range("above", 2.5, 1.0, 2.0));
        assert_eq!(h.passes(), 2);
        assert_eq!(h.fails(), 2);
    }

    #[test]
    fn check_max_boundary() {
        let mut h = harness("test");
        assert!(h.check_max("equal", 5.0, 5.0));
        assert!(h.check_max("below", 4.0, 5.0));
        assert!(!h.check_max("above", 6.0, 5.0));
    }

    #[test]
    fn check_min_boundary() {
        let mut h = harness("test");
        assert!(h.check_min("equal", 5.0, 5.0));
        assert!(h.check_min("above", 6.0, 5.0));
        assert!(!h.check_min("below", 4.0, 5.0));
    }

    #[test]
    fn check_true_pass_and_fail() {
        let mut h = harness("test");
        assert!(h.check_true("truthy", true));
        assert!(!h.check_true("falsy", false));
        assert_eq!(h.passes(), 1);
        assert_eq!(h.fails(), 1);
    }

    #[test]
    fn summary_returns_zero_on_all_pass() {
        let mut h = harness("test");
        h.check_true("ok", true);
        assert_eq!(h.summary(), 0);
    }

    #[test]
    fn summary_returns_one_on_any_fail() {
        let mut h = harness("test");
        h.check_true("ok", true);
        h.check_true("bad", false);
        assert_eq!(h.summary(), 1);
    }

    #[test]
    fn independent_harnesses_do_not_interfere() {
        let mut h1 = harness("h1");
        let mut h2 = harness("h2");
        h1.check_true("a", true);
        h2.check_true("b", false);
        assert_eq!(h1.passes(), 1);
        assert_eq!(h1.fails(), 0);
        assert_eq!(h2.passes(), 0);
        assert_eq!(h2.fails(), 1);
    }

    #[test]
    fn output_captures_pass_fail_labels() {
        let mut h = harness("capture");
        h.check_approx("temp", 20.5, 20.5, 0.01);
        h.check_approx("bad", 1.0, 9.0, 0.01);
        let output = String::from_utf8_lossy(&h.writer);
        assert!(output.contains("[PASS] temp"));
        assert!(output.contains("[FAIL] bad"));
    }

    #[test]
    fn summary_output_contains_totals() {
        let mut h = harness("totals_test");
        h.check_true("a", true);
        h.check_true("b", false);
        let code = h.summary();
        assert_eq!(code, 1);
        let output = String::from_utf8_lossy(&h.writer);
        assert!(output.contains("1/2 PASS"));
        assert!(output.contains("1/2 FAIL"));
    }
}
