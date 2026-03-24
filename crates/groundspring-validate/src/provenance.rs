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
/// Displays source, baseline commit/date, validation script, command, and
/// (when present) the author — full chain of custody per
/// `specs/PROVENANCE_SCHEMA.md`.
///
/// # Errors
///
/// Returns [`BenchFieldError`] if `_source`, `_provenance.baseline_commit`,
/// `_provenance.baseline_date`, `validation_script`, or `command` is missing
/// or not a string. The schema requires all five for reproducibility.
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
    let script =
        get_str(prov, "validation_script").or_else(|_| get_str(bench, "validation_script"))?;
    println!("  Script: {script}");
    let cmd = get_str(prov, "command").or_else(|_| get_str(bench, "command"))?;
    println!("  Command: {cmd}");
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
