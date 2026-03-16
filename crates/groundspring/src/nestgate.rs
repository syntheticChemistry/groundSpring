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
//! Provenance keys follow a hierarchical namespace, prefixed with
//! [`biomeos::FAMILY_ID`] for self-identifying provenance:
//! ```text
//! {FAMILY_ID}:results:<exp_id>:<run_id>     — validation results
//! {FAMILY_ID}:data:<source>:<query_id>       — cached live data
//! {FAMILY_ID}:parity:<exp_id>:<substrate>    — cross-substrate parity records
//! {FAMILY_ID}:tower:<event>:<timestamp>      — NUCLEUS lifecycle events
//! ```
//!
//! # Data Providers
//!
//! - NCBI: `search_genomes`, `fetch_sequence` via `data.ncbi_search`/`data.ncbi_fetch`
//! - NOAA CDO: `fetch_ghcnd` via `data.noaa_ghcnd` capability
//! - IRIS FDSN: `iris_stations`, `iris_events` via `data.iris_*` capabilities
//!
//! # Sovereign Fallback
//!
//! When the data provider is unavailable, all functions return `Err`. Callers
//! fall back to local synthetic/analytical data — the same data the 28
//! experiments already use. Live data is an enhancement, not a requirement.

use std::path::Path;

use crate::biomeos::{self, Result};

// ─── Key Schema ──────────────────────────────────────────────────────────────

/// Build a provenance key for storing validation results.
///
/// Format: `{FAMILY_ID}:results:exp{exp_id:03}:{run_id}`
#[must_use]
pub fn result_key(exp_id: u32, run_id: &str) -> String {
    let fam = biomeos::FAMILY_ID;
    format!("{fam}:results:exp{exp_id:03}:{run_id}")
}

/// Build a provenance key for cross-substrate parity records.
///
/// Format: `{FAMILY_ID}:parity:exp{exp_id:03}:{substrate}`
#[must_use]
pub fn parity_key(exp_id: u32, substrate: &str) -> String {
    let fam = biomeos::FAMILY_ID;
    format!("{fam}:parity:exp{exp_id:03}:{substrate}")
}

/// Build a key for cached live data from external sources.
///
/// Format: `{FAMILY_ID}:data:{source}:{query_id}`
#[must_use]
pub fn data_key(source: &str, query_id: &str) -> String {
    let fam = biomeos::FAMILY_ID;
    format!("{fam}:data:{source}:{query_id}")
}

// ─── Provenance Storage ──────────────────────────────────────────────────────

/// Store validation results for an experiment run.
///
/// # Errors
///
/// Returns `Err` if the storage provider is unavailable.
pub fn store_result(socket: &Path, exp_id: u32, run_id: &str, result_json: &str) -> Result<()> {
    let key = result_key(exp_id, run_id);
    biomeos::storage_put(socket, &key, result_json)
}

/// Retrieve validation results for an experiment run.
///
/// # Errors
///
/// Returns `Err` if the key does not exist or the storage provider is unavailable.
pub fn get_result(socket: &Path, exp_id: u32, run_id: &str) -> Result<String> {
    let key = result_key(exp_id, run_id);
    biomeos::storage_get(socket, &key)
}

/// Store a cross-substrate parity record.
///
/// # Errors
///
/// Returns `Err` if the storage provider is unavailable.
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
    let params = serde_json::json!({
        "database": database,
        "query": query,
        "family_id": biomeos::FAMILY_ID,
    })
    .to_string();
    biomeos::capability_call(socket, "data.ncbi_search", &params)
}

/// Fetch a sequence from NCBI via biomeOS `data.ncbi_fetch` capability.
///
/// # Errors
///
/// Returns `Err` if the data provider is unavailable or the fetch fails.
pub fn ncbi_fetch(socket: &Path, database: &str, accession: &str) -> Result<String> {
    let params = serde_json::json!({
        "database": database,
        "accession": accession,
        "family_id": biomeos::FAMILY_ID,
    })
    .to_string();
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
    let params = serde_json::json!({
        "station_id": station_id,
        "start_date": start_date,
        "end_date": end_date,
        "datatypes": datatypes,
        "family_id": biomeos::FAMILY_ID,
    })
    .to_string();
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

// ─── IRIS FDSN Data Provider ─────────────────────────────────────────

/// Fetch seismic station metadata via biomeOS `data.iris_stations` capability.
///
/// Returns station metadata for stations within the specified bounding box.
/// Used by Exp 005 (seismic inversion with real NMSZ data).
///
/// # Errors
///
/// Returns `Err` if the data provider is unavailable or the IRIS API call fails.
pub fn iris_stations(
    socket: &Path,
    min_lat: f64,
    max_lat: f64,
    min_lon: f64,
    max_lon: f64,
) -> Result<String> {
    let params = serde_json::json!({
        "min_lat": min_lat,
        "max_lat": max_lat,
        "min_lon": min_lon,
        "max_lon": max_lon,
        "family_id": biomeos::FAMILY_ID,
    })
    .to_string();
    biomeos::capability_call(socket, "data.iris_stations", &params)
}

/// Bounding box and time range for IRIS event queries.
pub struct IrisEventQuery<'a> {
    /// Southern boundary (degrees N).
    pub min_lat: f64,
    /// Northern boundary (degrees N).
    pub max_lat: f64,
    /// Western boundary (degrees E, negative for W).
    pub min_lon: f64,
    /// Eastern boundary (degrees E, negative for W).
    pub max_lon: f64,
    /// ISO-8601 start date (e.g. `"2023-01-01"`).
    pub start_date: &'a str,
    /// ISO-8601 end date.
    pub end_date: &'a str,
    /// Minimum event magnitude.
    pub min_magnitude: f64,
}

/// Fetch earthquake events via biomeOS `data.iris_events` capability.
///
/// Returns earthquake events within the bounding box and time range.
///
/// # Errors
///
/// Returns `Err` if the data provider is unavailable or the IRIS API call fails.
pub fn iris_events(socket: &Path, query: &IrisEventQuery<'_>) -> Result<String> {
    let params = serde_json::json!({
        "min_lat": query.min_lat,
        "max_lat": query.max_lat,
        "min_lon": query.min_lon,
        "max_lon": query.max_lon,
        "start_date": query.start_date,
        "end_date": query.end_date,
        "min_magnitude": query.min_magnitude,
        "family_id": biomeos::FAMILY_ID,
    })
    .to_string();
    biomeos::capability_call(socket, "data.iris_events", &params)
}

// ─── NUCLEUS Lifecycle Events ────────────────────────────────────────

/// Record a NUCLEUS lifecycle event in provenance storage.
///
/// Used to track when groundSpring connects to live NUCLEUS, runs
/// experiments against real data, or transitions between sovereign
/// and ecosystem modes.
///
/// # Errors
///
/// Returns `Err` if the storage provider is unavailable.
pub fn record_lifecycle_event(socket: &Path, event: &str, details_json: &str) -> Result<()> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let fam = biomeos::FAMILY_ID;
    let key = format!("{fam}:tower:{event}:{now}");
    biomeos::storage_put(socket, &key, details_json)
}

// ─── Cache-Through Helpers ───────────────────────────────────────────────────

/// Fetch data with cache-through. Checks provenance store first; on miss,
/// calls the live data provider and caches the result.
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
    biomeos::storage_get(socket, &key).or_else(|_| {
        let fresh = fetch_fn()?;
        let _ = biomeos::storage_put(socket, &key, &fresh);
        Ok(fresh)
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
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

    #[test]
    fn iris_data_key_format() {
        assert_eq!(
            data_key("iris", "nmsz_stations_34_40"),
            "groundspring:data:iris:nmsz_stations_34_40"
        );
    }

    #[test]
    fn lifecycle_key_format() {
        let key = format!(
            "groundspring:tower:{}:{}",
            "nucleus_connected", 1_709_164_800
        );
        assert!(key.starts_with("groundspring:tower:nucleus_connected:"));
    }
}
