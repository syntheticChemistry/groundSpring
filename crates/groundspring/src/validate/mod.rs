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

mod harness;
mod sink;

pub use harness::ValidationHarness;
pub use sink::{NdjsonSink, NullSink, StdoutSink, ValidationSink, WriteSink};

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
        let escaped = sink::json_escape("a\x00b\tc");
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
