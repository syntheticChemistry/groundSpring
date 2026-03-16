// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Provenance header printing for validation binaries.

use serde_json::Value;

use crate::{BenchFieldError, BenchResult, get_str};

/// Print the standard provenance header shared by all validation binaries.
///
/// Convenience wrapper that calls [`try_print_provenance_header`] and panics
/// on malformed benchmark JSON. Suitable for validation binaries where the
/// JSON is `include_str!`-ed at compile time.
///
/// # Panics
///
/// Panics if required provenance fields are missing.
#[expect(
    clippy::expect_used,
    reason = "compile-time JSON; missing provenance is a programmer error"
)]
pub fn print_provenance_header(bench: &Value, title: &str) {
    try_print_provenance_header(bench, title).expect("benchmark provenance header");
}

/// Print the standard provenance header, returning errors on missing fields.
///
/// Displays source, baseline commit/date, and (when present) the script,
/// command, and author that generated the baseline — full chain of custody.
///
/// # Errors
///
/// Returns [`BenchFieldError`] if `_source`, `_provenance.baseline_commit`,
/// or `_provenance.baseline_date` is missing or not a string.
pub fn try_print_provenance_header(bench: &Value, title: &str) -> BenchResult<()> {
    println!("{}", "=".repeat(72));
    println!("groundSpring Rust Validation: {title}");
    let source = get_str(bench, "_source")?;
    println!("  Source: {source}");
    let prov = bench.get("_provenance").ok_or_else(|| BenchFieldError {
        field: "_provenance".into(),
        expected: "object",
    })?;
    let commit = get_str(prov, "baseline_commit")?;
    let date = get_str(prov, "baseline_date")?;
    println!("  Provenance: commit {commit}, {date}");
    if let Some(script) = prov
        .get("validation_script")
        .and_then(Value::as_str)
        .or_else(|| bench.get("validation_script").and_then(Value::as_str))
    {
        println!("  Script: {script}");
    }
    if let Some(cmd) = prov
        .get("command")
        .and_then(Value::as_str)
        .or_else(|| bench.get("command").and_then(Value::as_str))
    {
        println!("  Command: {cmd}");
    }
    if let Some(author) = prov
        .get("generated_by")
        .and_then(Value::as_str)
        .or_else(|| bench.get("generated_by").and_then(Value::as_str))
    {
        println!("  Author: {author}");
    }
    println!("{}", "=".repeat(72));
    Ok(())
}
