// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Validation output sinks: trait + stdout, null, and NDJSON implementations.
//!
//! Absorbed from ludoSpring V22 / rhizoCrypt v0.13 / primalSpring patterns.

use std::io::{self, Write};

/// Abstraction for validation output destinations.
///
/// Allows validation harnesses to target stdout, in-memory buffers,
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
/// Retained for unit tests; [`NdjsonSink`] uses `serde_json` for serialization.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "exercised by validate::tests::json_escape_handles_control_chars"
    )
)]
pub(super) fn json_escape(s: &str) -> String {
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
        let obj = serde_json::json!({
            "type": "check",
            "status": "pass",
            "label": label,
            "detail": detail,
        });
        self.write_json(&obj.to_string());
    }

    fn record_fail(&mut self, label: &str, detail: &str) {
        let obj = serde_json::json!({
            "type": "check",
            "status": "fail",
            "label": label,
            "detail": detail,
        });
        self.write_json(&obj.to_string());
    }

    fn section(&mut self, name: &str) {
        let obj = serde_json::json!({
            "type": "section",
            "name": name,
        });
        self.write_json(&obj.to_string());
    }

    fn write_summary(&mut self, text: &str) {
        let obj = serde_json::json!({
            "type": "summary",
            "text": text,
        });
        self.write_json(&obj.to_string());
    }
}

/// Runtime-dispatch sink that selects text or NDJSON output based on
/// `--format json` CLI flag. Used by [`super::ValidationHarness::from_args`].
///
/// Avoids trait objects by using an enum; projectNUCLEUS Tier 2 ingestion
/// requires structured JSON output while CLI users keep the human-readable
/// default.
pub enum AutoSink {
    /// Human-readable text output (default).
    Text(WriteSink<io::Stdout>),
    /// Machine-readable NDJSON output (`--format json`).
    Json(NdjsonSink<io::Stdout>),
}

impl AutoSink {
    /// Select sink based on process arguments.
    ///
    /// Recognises `--format json` (or `--format=json`). Anything else
    /// (including no args) produces the text sink.
    #[must_use]
    pub fn from_args() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let json_requested = args
            .windows(2)
            .any(|w| w[0] == "--format" && w[1] == "json")
            || args.iter().any(|a| a == "--format=json");
        if json_requested {
            Self::Json(NdjsonSink::new(io::stdout()))
        } else {
            Self::Text(WriteSink::new(io::stdout()))
        }
    }
}

impl ValidationSink for AutoSink {
    fn record_pass(&mut self, label: &str, detail: &str) {
        match self {
            Self::Text(s) => s.record_pass(label, detail),
            Self::Json(s) => s.record_pass(label, detail),
        }
    }

    fn record_fail(&mut self, label: &str, detail: &str) {
        match self {
            Self::Text(s) => s.record_fail(label, detail),
            Self::Json(s) => s.record_fail(label, detail),
        }
    }

    fn section(&mut self, name: &str) {
        match self {
            Self::Text(s) => s.section(name),
            Self::Json(s) => s.section(name),
        }
    }

    fn write_summary(&mut self, text: &str) {
        match self {
            Self::Text(s) => s.write_summary(text),
            Self::Json(s) => s.write_summary(text),
        }
    }
}
