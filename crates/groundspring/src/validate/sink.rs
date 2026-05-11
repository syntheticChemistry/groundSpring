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
/// Intentionally minimal to avoid pulling `serde_json` into the core harness.
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
