// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Validation harness following the hotSpring pattern.
//!
//! [`ValidationHarness`] tracks pass/fail counters and writes results to
//! any [`ValidationSink`] destination (defaults to stdout via [`StdoutSink`]).
//!
//! The [`ValidationSink`] trait (absorbed from ludoSpring/rhizoCrypt/primalSpring)
//! provides a higher-level abstraction over raw `Write`, supporting structured
//! section markers and silent sinks for programmatic use.
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

// ─── ValidationSink Trait ────────────────────────────────────────────────────

/// Abstraction for validation output destinations.
///
/// Absorbed from ludoSpring V22 / rhizoCrypt v0.13 / primalSpring validation
/// patterns. Allows validation harnesses to target stdout, in-memory buffers,
/// or null sinks (benchmarks / CI) without changing harness logic.
pub trait ValidationSink {
    /// Record a passing check.
    fn record_pass(&mut self, label: &str, detail: &str);
    /// Record a failing check.
    fn record_fail(&mut self, label: &str, detail: &str);
    /// Begin a named section (visual grouping in output).
    fn section(&mut self, name: &str);
    /// Write a summary line.
    fn write_summary(&mut self, text: &str);
}

/// Sink that writes to an [`io::Write`] destination — the standard path.
///
/// [`StdoutSink`] wraps `io::Stdout`; use [`WriteSink`] for custom writers.
pub struct WriteSink<W: Write> {
    writer: W,
}

impl<W: Write> WriteSink<W> {
    /// Wrap any [`Write`] as a validation sink.
    #[must_use]
    pub const fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Access the inner writer (e.g. for reading captured bytes in tests).
    #[must_use]
    pub const fn inner(&self) -> &W {
        &self.writer
    }
}

impl<W: Write> ValidationSink for WriteSink<W> {
    fn record_pass(&mut self, label: &str, detail: &str) {
        let _ = writeln!(self.writer, "  [PASS] {label}: {detail}");
    }

    fn record_fail(&mut self, label: &str, detail: &str) {
        let _ = writeln!(self.writer, "  [FAIL] {label}: {detail}");
    }

    fn section(&mut self, name: &str) {
        let _ = writeln!(self.writer, "\n--- {name} ---\n");
    }

    fn write_summary(&mut self, text: &str) {
        let _ = writeln!(self.writer, "{text}");
    }
}

/// Convenience alias for a stdout-backed sink.
pub type StdoutSink = WriteSink<io::Stdout>;

/// Sink that discards all output — for benchmarks or programmatic validation.
pub struct NullSink;

impl ValidationSink for NullSink {
    fn record_pass(&mut self, _label: &str, _detail: &str) {}
    fn record_fail(&mut self, _label: &str, _detail: &str) {}
    fn section(&mut self, _name: &str) {}
    fn write_summary(&mut self, _text: &str) {}
}

// ─── ValidationHarness ──────────────────────────────────────────────────────

/// Validation harness with independent pass/fail counters.
///
/// Output goes to the [`ValidationSink`] provided at construction.
/// Use [`stdout`](Self::stdout) for terminal output,
/// [`new`](Self::new) with a [`WriteSink`] for custom writers,
/// or [`silent`](Self::silent) for zero-output programmatic use.
pub struct ValidationHarness<S: ValidationSink = StdoutSink> {
    name: String,
    passes: u32,
    fails: u32,
    sink: S,
}

impl ValidationHarness<StdoutSink> {
    /// Create a harness that writes to stdout (the common case).
    #[must_use]
    pub fn stdout(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passes: 0,
            fails: 0,
            sink: WriteSink::new(io::stdout()),
        }
    }
}

impl ValidationHarness<NullSink> {
    /// Create a harness that discards all output.
    #[must_use]
    pub fn silent(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passes: 0,
            fails: 0,
            sink: NullSink,
        }
    }
}

impl<W: Write> ValidationHarness<WriteSink<W>> {
    /// Create a harness with a custom writer (backwards-compatible path).
    #[must_use]
    pub fn new(name: impl Into<String>, writer: W) -> Self {
        Self {
            name: name.into(),
            passes: 0,
            fails: 0,
            sink: WriteSink::new(writer),
        }
    }
}

impl<S: ValidationSink> ValidationHarness<S> {
    /// Create a harness with an arbitrary [`ValidationSink`].
    #[must_use]
    pub fn with_sink(name: impl Into<String>, sink: S) -> Self {
        Self {
            name: name.into(),
            passes: 0,
            fails: 0,
            sink,
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

    /// Access the underlying sink.
    #[must_use]
    pub const fn sink(&self) -> &S {
        &self.sink
    }

    /// Begin a named section in the output.
    pub fn section(&mut self, name: &str) {
        self.sink.section(name);
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
        let detail =
            format!("{computed:.6} (expected {expected:.6}, tol {tol:.6}, diff {diff:.6})");
        if ok {
            self.sink.record_pass(label, &detail);
        } else {
            self.sink.record_fail(label, &detail);
        }
        self.record(ok)
    }

    /// Check that `computed` falls within [`low`, `high`].
    pub fn check_range(&mut self, label: &str, computed: f64, low: f64, high: f64) -> bool {
        let ok = (low..=high).contains(&computed);
        let detail = format!("{computed:.6} (expected [{low:.6}, {high:.6}])");
        if ok {
            self.sink.record_pass(label, &detail);
        } else {
            self.sink.record_fail(label, &detail);
        }
        self.record(ok)
    }

    /// Check that `computed` <= `maximum`.
    pub fn check_max(&mut self, label: &str, computed: f64, maximum: f64) -> bool {
        let ok = computed <= maximum;
        let detail = format!("{computed:.6} (max {maximum:.6})");
        if ok {
            self.sink.record_pass(label, &detail);
        } else {
            self.sink.record_fail(label, &detail);
        }
        self.record(ok)
    }

    /// Check that `computed` >= `minimum`.
    pub fn check_min(&mut self, label: &str, computed: f64, minimum: f64) -> bool {
        let ok = computed >= minimum;
        let detail = format!("{computed:.6} (min {minimum:.6})");
        if ok {
            self.sink.record_pass(label, &detail);
        } else {
            self.sink.record_fail(label, &detail);
        }
        self.record(ok)
    }

    /// Check that a boolean condition holds.
    pub fn check_true(&mut self, label: &str, condition: bool) -> bool {
        if condition {
            self.sink.record_pass(label, "");
        } else {
            self.sink.record_fail(label, "");
        }
        self.record(condition)
    }

    /// Write summary and return process exit code (0 = all pass, 1 = any fail).
    #[must_use]
    pub fn summary(&mut self) -> i32 {
        let total = self.passes + self.fails;
        let sep = "=".repeat(72);
        self.sink.write_summary(&sep);
        self.sink.write_summary(&self.name);
        self.sink.write_summary(&format!(
            "TOTAL: {}/{total} PASS, {}/{total} FAIL",
            self.passes, self.fails
        ));
        self.sink.write_summary(&sep);
        i32::from(self.fails != 0)
    }
}

impl<S: ValidationSink> std::fmt::Debug for ValidationHarness<S> {
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

    fn harness(name: &str) -> ValidationHarness<WriteSink<Vec<u8>>> {
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
        let output = String::from_utf8_lossy(h.sink().inner());
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
        let output = String::from_utf8_lossy(h.sink().inner());
        assert!(output.contains("1/2 PASS"));
        assert!(output.contains("1/2 FAIL"));
    }

    #[test]
    fn debug_impl_shows_fields() {
        let h = harness("debug_test");
        let dbg = format!("{h:?}");
        assert!(dbg.contains("debug_test"));
        assert!(dbg.contains("passes"));
        assert!(dbg.contains("fails"));
    }

    #[test]
    fn check_range_fail_output() {
        let mut h = harness("rng");
        h.check_range("too_low", 0.5, 1.0, 2.0);
        h.check_range("too_high", 3.0, 1.0, 2.0);
        let output = String::from_utf8_lossy(h.sink().inner());
        assert!(output.contains("[FAIL] too_low"));
        assert!(output.contains("[FAIL] too_high"));
    }

    #[test]
    fn check_min_fail_output() {
        let mut h = harness("min_fail");
        assert!(!h.check_min("below", 3.0, 5.0));
        let output = String::from_utf8_lossy(h.sink().inner());
        assert!(output.contains("[FAIL] below"));
    }

    #[test]
    fn null_sink_tracks_counts_silently() {
        let mut h = ValidationHarness::silent("null_test");
        h.check_true("a", true);
        h.check_true("b", false);
        h.section("ignored");
        assert_eq!(h.passes(), 1);
        assert_eq!(h.fails(), 1);
        assert_eq!(h.summary(), 1);
    }

    #[test]
    fn section_appears_in_output() {
        let mut h = harness("section_test");
        h.section("Part 1");
        h.check_true("x", true);
        let output = String::from_utf8_lossy(h.sink().inner());
        assert!(output.contains("Part 1"));
    }

    #[test]
    fn with_sink_custom() {
        let mut h = ValidationHarness::with_sink("custom", NullSink);
        h.check_true("silent", true);
        assert_eq!(h.passes(), 1);
    }
}
