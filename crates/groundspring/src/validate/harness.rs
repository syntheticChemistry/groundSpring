// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Validation harness with independent pass/fail counters.

use std::io::{self, Write};

use super::sink::{AutoSink, NullSink, StdoutSink, ValidationSink, WriteSink};

/// Validation harness with independent pass/fail counters.
///
/// Output goes to the [`ValidationSink`] provided at construction.
/// Use [`stdout`](Self::stdout) for terminal output,
/// [`new`](Self::new) with a [`WriteSink`] for custom writers,
/// or [`silent`](Self::silent) for zero-output programmatic use.
pub struct ValidationHarness<S: ValidationSink = AutoSink> {
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

impl ValidationHarness<AutoSink> {
    /// Create a harness that auto-selects text or JSON output.
    ///
    /// Inspects `std::env::args()` for `--format json` (or `--format=json`).
    /// Default is human-readable text; JSON mode emits NDJSON for
    /// projectNUCLEUS Tier 2 pipeline ingestion.
    #[must_use]
    pub fn from_args(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passes: 0,
            fails: 0,
            sink: AutoSink::from_args(),
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
    pub(super) const RELATIVE_DENOM_GUARD: f64 = crate::tol::DETERMINISM;

    /// Check that `computed` is within `rel_tol` *relative* to `expected`.
    ///
    /// Falls back to absolute comparison when `|expected|` is near zero.
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

    /// Check absolute *or* relative tolerance — whichever is more lenient.
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
