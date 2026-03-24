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

/// Machine-readable NDJSON sink for CI and pipeline consumption.
///
/// Emits one JSON object per line (Newline-Delimited JSON). Each check
/// produces a `{"type":"check","status":"pass"|"fail","label":"...","detail":"..."}`
/// line. Sections emit `{"type":"section","name":"..."}`. Summaries emit
/// `{"type":"summary","text":"..."}`.
///
/// Absorbed from wetSpring V132 `StreamItem` NDJSON pattern.
pub struct NdjsonSink<W: Write> {
    writer: W,
}

impl<W: Write> NdjsonSink<W> {
    /// Wrap any [`Write`] as an NDJSON validation sink.
    #[must_use]
    pub const fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Access the inner writer (e.g. for reading captured bytes in tests).
    #[must_use]
    pub const fn inner(&self) -> &W {
        &self.writer
    }

    fn write_json(&mut self, json: &str) {
        let _ = writeln!(self.writer, "{json}");
    }
}

/// Escape a string for safe embedding inside a JSON string value.
///
/// Handles `"`, `\`, and control characters (including newlines) per RFC 8259.
/// This is intentionally minimal — we avoid pulling `serde_json` into the
/// core validation harness (which has zero optional deps).
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_ascii_control() => {
                use std::fmt::Write as _;
                let _ = write!(out, "\\u{:04x}", u32::from(c));
            }
            c => out.push(c),
        }
    }
    out
}

impl<W: Write> ValidationSink for NdjsonSink<W> {
    fn record_pass(&mut self, label: &str, detail: &str) {
        let l = json_escape(label);
        let d = json_escape(detail);
        self.write_json(&format!(
            r#"{{"type":"check","status":"pass","label":"{l}","detail":"{d}"}}"#
        ));
    }

    fn record_fail(&mut self, label: &str, detail: &str) {
        let l = json_escape(label);
        let d = json_escape(detail);
        self.write_json(&format!(
            r#"{{"type":"check","status":"fail","label":"{l}","detail":"{d}"}}"#
        ));
    }

    fn section(&mut self, name: &str) {
        let n = json_escape(name);
        self.write_json(&format!(r#"{{"type":"section","name":"{n}"}}"#));
    }

    fn write_summary(&mut self, text: &str) {
        let t = json_escape(text);
        self.write_json(&format!(r#"{{"type":"summary","text":"{t}"}}"#));
    }
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

    /// Near-zero denominator guard for relative error computation.
    ///
    /// When `|expected|` is below this threshold, `check_relative` and
    /// `check_abs_or_rel` fall back to absolute comparison to avoid
    /// division by near-zero. Matches `tol::DETERMINISM` (~5× f64 ε).
    const RELATIVE_DENOM_GUARD: f64 = crate::tol::DETERMINISM;

    /// Check that `computed` is within `rel_tol` *relative* to `expected`.
    ///
    /// Uses `|computed − expected| / |expected|` when `|expected|` exceeds
    /// `RELATIVE_DENOM_GUARD` (= [`tol::DETERMINISM`](crate::tol::DETERMINISM)),
    /// falling back to absolute comparison otherwise (avoids division by
    /// near-zero). Matches the metalForge `tolerance::compare` semantics
    /// and the hotSpring/neuralSpring `check_rel` pattern.
    pub fn check_relative(
        &mut self,
        label: &str,
        computed: f64,
        expected: f64,
        rel_tol: f64,
    ) -> bool {
        let abs_diff = (computed - expected).abs();
        let rel_err = if expected.abs() > Self::RELATIVE_DENOM_GUARD {
            abs_diff / expected.abs()
        } else {
            abs_diff
        };
        let ok = rel_err <= rel_tol;
        let detail = format!(
            "{computed:.6} (expected {expected:.6}, rel_tol {rel_tol:.6}, rel_err {rel_err:.6})"
        );
        if ok {
            self.sink.record_pass(label, &detail);
        } else {
            self.sink.record_fail(label, &detail);
        }
        self.record(ok)
    }

    /// Check that `computed` matches `expected` within *either* an absolute
    /// or relative tolerance — whichever is more lenient.
    ///
    /// Combines [`check_approx`](Self::check_approx) (absolute) and
    /// [`check_relative`](Self::check_relative) semantics: a check passes
    /// if `|computed − expected| ≤ abs_tol` **or** the relative error
    /// `|computed − expected| / |expected| ≤ rel_tol`. This handles both
    /// near-zero values (where absolute tolerance dominates) and large-magnitude
    /// values (where relative tolerance is more meaningful).
    pub fn check_abs_or_rel(
        &mut self,
        label: &str,
        computed: f64,
        expected: f64,
        abs_tol: f64,
        rel_tol: f64,
    ) -> bool {
        let abs_diff = (computed - expected).abs();
        let abs_ok = abs_diff <= abs_tol;
        let rel_err = if expected.abs() > Self::RELATIVE_DENOM_GUARD {
            abs_diff / expected.abs()
        } else {
            abs_diff
        };
        let rel_ok = rel_err <= rel_tol;
        let ok = abs_ok || rel_ok;
        let detail = format!(
            "{computed:.6} (expected {expected:.6}, \
             abs_tol {abs_tol:.6}, rel_tol {rel_tol:.6}, \
             diff {abs_diff:.6}, rel_err {rel_err:.6})"
        );
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
    fn check_relative_pass() {
        let mut h = harness("test");
        assert!(h.check_relative("close", 1.005, 1.0, 0.01));
        assert_eq!(h.passes(), 1);
        assert_eq!(h.fails(), 0);
    }

    #[test]
    fn check_relative_fail() {
        let mut h = harness("test");
        assert!(!h.check_relative("far", 1.5, 1.0, 0.01));
        assert_eq!(h.fails(), 1);
    }

    #[test]
    fn check_relative_near_zero_falls_back_to_absolute() {
        let mut h = harness("test");
        assert!(h.check_relative("near_zero", 1e-16, 0.0, 1e-10));
        assert_eq!(h.passes(), 1);
    }

    #[test]
    fn check_abs_or_rel_pass_via_absolute() {
        let mut h = harness("test");
        assert!(h.check_abs_or_rel("abs_wins", 100.001, 100.0, 0.01, 1e-10));
        assert_eq!(h.passes(), 1);
    }

    #[test]
    fn check_abs_or_rel_pass_via_relative() {
        let mut h = harness("test");
        assert!(h.check_abs_or_rel("rel_wins", 1000.5, 1000.0, 0.001, 0.001));
        assert_eq!(h.passes(), 1);
    }

    #[test]
    fn check_abs_or_rel_fail_both() {
        let mut h = harness("test");
        assert!(!h.check_abs_or_rel("both_fail", 2.0, 1.0, 0.01, 0.01));
        assert_eq!(h.fails(), 1);
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

    // ── NDJSON sink tests ────────────────────────────────────────────────

    fn ndjson_harness(name: &str) -> ValidationHarness<NdjsonSink<Vec<u8>>> {
        ValidationHarness::with_sink(name, NdjsonSink::new(Vec::new()))
    }

    #[test]
    fn ndjson_pass_emits_check_line() {
        let mut h = ndjson_harness("ndjson");
        h.check_true("ok", true);
        let output = String::from_utf8_lossy(h.sink().inner());
        assert!(output.contains(r#""type":"check""#));
        assert!(output.contains(r#""status":"pass""#));
        assert!(output.contains(r#""label":"ok""#));
    }

    #[test]
    fn ndjson_fail_emits_check_line() {
        let mut h = ndjson_harness("ndjson");
        h.check_true("bad", false);
        let output = String::from_utf8_lossy(h.sink().inner());
        assert!(output.contains(r#""status":"fail""#));
    }

    #[test]
    fn ndjson_section_emits_section_line() {
        let mut h = ndjson_harness("ndjson");
        h.section("Part A");
        let output = String::from_utf8_lossy(h.sink().inner());
        assert!(output.contains(r#""type":"section""#));
        assert!(output.contains(r#""name":"Part A""#));
    }

    #[test]
    fn ndjson_summary_emits_summary_line() {
        let mut h = ndjson_harness("ndjson");
        h.check_true("x", true);
        let _ = h.summary();
        let output = String::from_utf8_lossy(h.sink().inner());
        assert!(output.contains(r#""type":"summary""#));
    }

    #[test]
    fn ndjson_lines_are_newline_delimited() {
        let mut h = ndjson_harness("ndjson");
        h.check_true("a", true);
        h.check_true("b", false);
        let output = String::from_utf8_lossy(h.sink().inner());
        assert_eq!(output.lines().count(), 2);
    }

    #[test]
    fn ndjson_escapes_quotes_in_label() {
        let mut h = ndjson_harness("escape");
        h.check_true("val\"ue", true);
        let output = String::from_utf8_lossy(h.sink().inner());
        assert!(output.contains(r#""label":"val\"ue""#));
        assert_eq!(output.lines().count(), 1);
    }

    #[test]
    fn ndjson_escapes_backslash_and_newline() {
        let mut h = ndjson_harness("escape");
        h.check_true("a\\b\nc", false);
        let output = String::from_utf8_lossy(h.sink().inner());
        assert!(output.contains(r"a\\b\nc"));
        assert_eq!(output.lines().count(), 1);
    }

    #[test]
    fn ndjson_injection_produces_single_line() {
        let mut h = ndjson_harness("inject");
        h.check_true(r#"x","detail":"injected"},{"type":"check"#, true);
        let output = String::from_utf8_lossy(h.sink().inner());
        assert_eq!(
            output.lines().count(),
            1,
            "injection must not create extra lines"
        );
    }

    #[test]
    fn json_escape_handles_control_chars() {
        let escaped = super::json_escape("a\x00b\tc");
        assert!(!escaped.contains('\x00'));
        assert!(escaped.contains("\\t"));
    }

    // ── relative-denom-guard constant ────────────────────────────────

    #[test]
    fn relative_denom_guard_matches_tol_determinism() {
        use crate::tol;
        assert!(
            (ValidationHarness::<NullSink>::RELATIVE_DENOM_GUARD - tol::DETERMINISM).abs()
                < f64::EPSILON
        );
    }
}
