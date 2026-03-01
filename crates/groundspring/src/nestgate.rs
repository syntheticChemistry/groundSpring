// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Data pipeline for experiment data and provenance via biomeOS capabilities.
//!
//! Routes through biomeOS capability-based discovery — groundSpring declares
//! what capability it needs (`storage.store`, `data.ncbi_search`, etc.) and
//! biomeOS discovers which primal provides it at runtime. No hardcoded primal
//! references; only self-knowledge and capability semantics.
//!
//! # Key Schema
//!
//! Provenance keys follow a hierarchical namespace:
//! ```text
//! groundspring:results:<exp_id>:<run_id>     — validation results
//! groundspring:data:<source>:<query_id>       — cached live data
//! groundspring:parity:<exp_id>:<substrate>    — cross-substrate parity records
//! groundspring:tower:<event>:<timestamp>      — NUCLEUS lifecycle events
//! ```
//!
//! # Data Providers
//!
//! - NCBI: `search_genomes`, `fetch_sequence` via `NestGate`'s `ncbi_live_provider`
//! - NOAA CDO: `fetch_ghcnd` via `NestGate`'s `noaa_cdo_live_provider`
//!
//! # Sovereign Fallback
//!
//! When `NestGate` is unavailable, all functions return `Err`. Callers fall back
//! to local synthetic/analytical data — the same data the 28 experiments already
//! use. Live data is an enhancement, not a requirement.

use std::path::Path;

use crate::biomeos::{self, Result};

// ─── Key Schema ──────────────────────────────────────────────────────────────

/// Build a provenance key for storing validation results.
///
/// Format: `groundspring:results:exp{exp_id:03}:{run_id}`
#[must_use]
pub fn result_key(exp_id: u32, run_id: &str) -> String {
    format!("groundspring:results:exp{exp_id:03}:{run_id}")
}

/// Build a provenance key for cross-substrate parity records.
///
/// Format: `groundspring:parity:exp{exp_id:03}:{substrate}`
#[must_use]
pub fn parity_key(exp_id: u32, substrate: &str) -> String {
    format!("groundspring:parity:exp{exp_id:03}:{substrate}")
}

/// Build a key for cached live data from external sources.
///
/// Format: `groundspring:data:{source}:{query_id}`
#[must_use]
pub fn data_key(source: &str, query_id: &str) -> String {
    format!("groundspring:data:{source}:{query_id}")
}

// ─── Provenance Storage ──────────────────────────────────────────────────────

/// Store validation results for an experiment run.
///
/// # Errors
///
/// Returns `Err` if `NestGate` is unavailable.
pub fn store_result(socket: &Path, exp_id: u32, run_id: &str, result_json: &str) -> Result<()> {
    let key = result_key(exp_id, run_id);
    biomeos::storage_put(socket, &key, result_json)
}

/// Retrieve validation results for an experiment run.
///
/// # Errors
///
/// Returns `Err` if the key does not exist or `NestGate` is unavailable.
pub fn get_result(socket: &Path, exp_id: u32, run_id: &str) -> Result<String> {
    let key = result_key(exp_id, run_id);
    biomeos::storage_get(socket, &key)
}

/// Store a cross-substrate parity record.
///
/// # Errors
///
/// Returns `Err` if `NestGate` is unavailable.
pub fn store_parity(socket: &Path, exp_id: u32, substrate: &str, parity_json: &str) -> Result<()> {
    let key = parity_key(exp_id, substrate);
    biomeos::storage_put(socket, &key, parity_json)
}

// ─── NCBI Data Provider ─────────────────────────────────────────────────────

/// Search NCBI genomes via biomeOS `data.ncbi_search` capability.
///
/// Routes through capability-based discovery — biomeOS determines which
/// primal handles NCBI data access at runtime.
///
/// # Errors
///
/// Returns `Err` if the data provider is unavailable or the NCBI query fails.
pub fn ncbi_search(socket: &Path, database: &str, query: &str) -> Result<String> {
    let params = format!(
        r#"{{"database":"{}","query":"{}","family_id":"groundspring"}}"#,
        biomeos::escape_json_pub(database),
        biomeos::escape_json_pub(query),
    );
    biomeos::capability_call(socket, "data.ncbi_search", &params)
}

/// Fetch a sequence from NCBI via biomeOS `data.ncbi_fetch` capability.
///
/// # Errors
///
/// Returns `Err` if the data provider is unavailable or the fetch fails.
pub fn ncbi_fetch(socket: &Path, database: &str, accession: &str) -> Result<String> {
    let params = format!(
        r#"{{"database":"{}","accession":"{}","family_id":"groundspring"}}"#,
        biomeos::escape_json_pub(database),
        biomeos::escape_json_pub(accession),
    );
    biomeos::capability_call(socket, "data.ncbi_fetch", &params)
}

// ─── NOAA CDO Data Provider ──────────────────────────────────────────────────

/// Fetch GHCND daily observations via biomeOS `data.noaa_ghcnd` capability.
///
/// Returns daily weather data for the specified station, date range, and
/// variable set. Used by Exp 002 (ET₀ validation with live weather data).
///
/// # Errors
///
/// Returns `Err` if the data provider is unavailable or the NOAA API call fails.
pub fn noaa_ghcnd(
    socket: &Path,
    station_id: &str,
    start_date: &str,
    end_date: &str,
    datatypes: &[&str],
) -> Result<String> {
    let dt_json: Vec<String> = datatypes
        .iter()
        .map(|d| format!("\"{}\"", biomeos::escape_json_pub(d)))
        .collect();
    let params = format!(
        r#"{{"station_id":"{}","start_date":"{}","end_date":"{}","datatypes":[{}],"family_id":"groundspring"}}"#,
        biomeos::escape_json_pub(station_id),
        biomeos::escape_json_pub(start_date),
        biomeos::escape_json_pub(end_date),
        dt_json.join(","),
    );
    biomeos::capability_call(socket, "data.noaa_ghcnd", &params)
}

/// Fetch FAO-56 weather variables via biomeOS `data.noaa_ghcnd` capability.
///
/// Convenience wrapper that requests the specific GHCND variables needed
/// for Penman-Monteith ET₀ calculation: TMAX, TMIN, AWND, RHAV/RHMN/RHMX.
///
/// # Errors
///
/// Returns `Err` if the data provider is unavailable or the NOAA API call fails.
pub fn noaa_fao56_variables(
    socket: &Path,
    station_id: &str,
    start_date: &str,
    end_date: &str,
) -> Result<String> {
    noaa_ghcnd(
        socket,
        station_id,
        start_date,
        end_date,
        &["TMAX", "TMIN", "AWND", "RHAV"],
    )
}

// ─── Cache-Through Helpers ───────────────────────────────────────────────────

/// Fetch data with `NestGate` cache. Checks provenance store first; on miss,
/// calls the live provider and caches the result.
///
/// # Errors
///
/// Returns `Err` if both cache lookup and live fetch fail.
pub fn fetch_cached(
    socket: &Path,
    source: &str,
    query_id: &str,
    fetch_fn: impl FnOnce() -> Result<String>,
) -> Result<String> {
    let key = data_key(source, query_id);

    if let Ok(cached) = biomeos::storage_get(socket, &key) {
        return Ok(cached);
    }

    let fresh = fetch_fn()?;

    // Best-effort cache: don't fail if storage is unavailable
    let _ = biomeos::storage_put(socket, &key, &fresh);

    Ok(fresh)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn result_key_format() {
        assert_eq!(
            result_key(8, "run_001"),
            "groundspring:results:exp008:run_001"
        );
        assert_eq!(
            result_key(28, "latest"),
            "groundspring:results:exp028:latest"
        );
    }

    #[test]
    fn parity_key_format() {
        assert_eq!(parity_key(8, "gpu"), "groundspring:parity:exp008:gpu");
        assert_eq!(parity_key(28, "npu"), "groundspring:parity:exp028:npu");
    }

    #[test]
    fn data_key_format() {
        assert_eq!(
            data_key("ncbi", "16s_amplicon_srr123456"),
            "groundspring:data:ncbi:16s_amplicon_srr123456"
        );
        assert_eq!(
            data_key("noaa_cdo", "USW00094847_2025"),
            "groundspring:data:noaa_cdo:USW00094847_2025"
        );
    }

    #[test]
    fn key_zero_padding() {
        assert_eq!(result_key(1, "x"), "groundspring:results:exp001:x");
        assert_eq!(result_key(100, "x"), "groundspring:results:exp100:x");
    }
}
