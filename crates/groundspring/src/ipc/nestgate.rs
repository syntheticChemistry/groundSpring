// SPDX-License-Identifier: AGPL-3.0-or-later

//! IPC interface for `NestGate` storage and data pipelines.
//!
//! `NestGate` provides content-addressed storage and live data acquisition
//! from public repositories (NCBI, NOAA GHCND, IRIS FDSN). `groundSpring`
//! uses `NestGate` for:
//! - Storing/retrieving validation artifacts with provenance
//! - Acquiring real-world datasets for Experiments 029-032
//! - NOAA GHCND daily weather CSV ingestion (NestGate pipeline exercise)
//!
//! # Wire names
//!
//! - `content.put` / `content.get` — content-addressed storage (CAS)
//! - `data.noaa_ghcnd` — NOAA weather observations
//! - `data.ncbi_search` / `data.ncbi_fetch` — NCBI sequence data
//! - `data.iris_stations` / `data.iris_events` — IRIS seismic data
//!
//! # NestGate pipeline exercise (NOAA GHCND)
//!
//! The upstream audit identifies NOAA GHCND as the easiest real dataset
//! to ingest via `NestGate`. The pipeline:
//! 1. Call `data.noaa_ghcnd` with station/date/element params
//! 2. Receive CSV daily observations
//! 3. Validate against expected statistical properties
//! 4. Store validated results via `content.put` with BLAKE3 provenance

/// Storage capabilities (routed via Neural API to `NestGate`).
#[tarpc::service]
pub trait StorageService {
    /// Store a key-value pair with provenance (CAS `content.put`).
    async fn put(key: String, value: String, family_id: String) -> Result<(), String>;

    /// Retrieve a value by key (CAS `content.get`).
    async fn get(key: String, family_id: String) -> Result<String, String>;
}

/// Live data pipeline capabilities (routed via Neural API to `NestGate`).
#[tarpc::service]
pub trait DataPipeline {
    /// Search NCBI databases.
    async fn ncbi_search(database: String, query: String) -> Result<String, String>;

    /// Fetch a sequence from NCBI by accession.
    async fn ncbi_fetch(database: String, accession: String) -> Result<String, String>;

    /// Fetch GHCND daily weather observations.
    async fn noaa_ghcnd(params_json: String) -> Result<String, String>;

    /// Fetch IRIS seismic station metadata.
    async fn iris_stations(params_json: String) -> Result<String, String>;
}

/// Store content via `NestGate` JSON-RPC (`content.put`).
///
/// # Errors
///
/// Returns `BiomeOsError` if `NestGate` is not discovered or the IPC call fails.
#[cfg(feature = "biomeos")]
pub fn content_put(
    socket: &std::path::Path,
    key: &str,
    value: &str,
    family_id: &str,
) -> crate::biomeos::Result<serde_json::Value> {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "content.put",
        "params": {
            "key": key,
            "value": value,
            "family_id": family_id,
        },
        "id": 1
    })
    .to_string();
    let response = crate::biomeos::raw_rpc_call(socket, &request)?;
    parse_jsonrpc_response(&response)
}

/// Retrieve content via `NestGate` JSON-RPC (`content.get`).
///
/// # Errors
///
/// Returns `BiomeOsError` if `NestGate` is not discovered or the IPC call fails.
#[cfg(feature = "biomeos")]
pub fn content_get(
    socket: &std::path::Path,
    key: &str,
    family_id: &str,
) -> crate::biomeos::Result<serde_json::Value> {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "content.get",
        "params": {
            "key": key,
            "family_id": family_id,
        },
        "id": 1
    })
    .to_string();
    let response = crate::biomeos::raw_rpc_call(socket, &request)?;
    parse_jsonrpc_response(&response)
}

/// Fetch NOAA GHCND daily observations via `NestGate` JSON-RPC.
///
/// `station_id`: e.g., `"USW00094728"` (Central Park, NY)
/// `start_date` / `end_date`: `"YYYY-MM-DD"` format
/// `elements`: Comma-separated list (e.g., `"TMAX,TMIN,PRCP"`)
///
/// # Errors
///
/// Returns `BiomeOsError` if `NestGate` is not discovered or the IPC call fails.
#[cfg(feature = "biomeos")]
pub fn noaa_ghcnd_fetch(
    socket: &std::path::Path,
    station_id: &str,
    start_date: &str,
    end_date: &str,
    elements: &str,
) -> crate::biomeos::Result<serde_json::Value> {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "data.noaa_ghcnd",
        "params": {
            "station_id": station_id,
            "start_date": start_date,
            "end_date": end_date,
            "elements": elements,
        },
        "id": 1
    })
    .to_string();
    let response = crate::biomeos::raw_rpc_call(socket, &request)?;
    parse_jsonrpc_response(&response)
}

/// Attempt to discover `NestGate` and fetch NOAA GHCND data.
///
/// Returns `Ok(None)` if `NestGate` is not available (graceful degradation).
#[cfg(feature = "biomeos")]
pub fn try_noaa_ghcnd_fetch(
    station_id: &str,
    start_date: &str,
    end_date: &str,
    elements: &str,
) -> crate::biomeos::Result<Option<serde_json::Value>> {
    crate::primal_names::discover_socket(crate::primal_names::roles::STORAGE).map_or_else(
        || {
            tracing::debug!("NestGate not discovered — NOAA GHCND fetch skipped");
            Ok(None)
        },
        |socket| noaa_ghcnd_fetch(&socket, station_id, start_date, end_date, elements).map(Some),
    )
}

/// Attempt to discover `NestGate` and store content.
///
/// Returns `Ok(None)` if `NestGate` is not available (graceful degradation).
#[cfg(feature = "biomeos")]
pub fn try_content_put(
    key: &str,
    value: &str,
    family_id: &str,
) -> crate::biomeos::Result<Option<serde_json::Value>> {
    crate::primal_names::discover_socket(crate::primal_names::roles::STORAGE).map_or_else(
        || {
            tracing::debug!("NestGate not discovered — content put skipped");
            Ok(None)
        },
        |socket| content_put(&socket, key, value, family_id).map(Some),
    )
}

/// Attempt to discover `NestGate` and retrieve content.
///
/// Returns `Ok(None)` if `NestGate` is not available (graceful degradation).
#[cfg(feature = "biomeos")]
pub fn try_content_get(
    key: &str,
    family_id: &str,
) -> crate::biomeos::Result<Option<serde_json::Value>> {
    crate::primal_names::discover_socket(crate::primal_names::roles::STORAGE).map_or_else(
        || {
            tracing::debug!("NestGate not discovered — content get skipped");
            Ok(None)
        },
        |socket| content_get(&socket, key, family_id).map(Some),
    )
}

/// Extract `result` or `error` from a JSON-RPC 2.0 response.
#[cfg(feature = "biomeos")]
fn parse_jsonrpc_response(response: &str) -> crate::biomeos::Result<serde_json::Value> {
    let parsed: serde_json::Value = serde_json::from_str(response)
        .map_err(|e| crate::biomeos::BiomeOsError::Protocol(format!("invalid JSON: {e}")))?;
    if let Some(err) = parsed.get("error") {
        return Err(crate::biomeos::BiomeOsError::Protocol(err.to_string()));
    }
    Ok(parsed.get("result").cloned().unwrap_or(parsed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tarpc_traits_compile() {
        fn _assert_storage<T: StorageService>() {}
        fn _assert_pipeline<T: DataPipeline>() {}
    }

    #[test]
    fn storage_role_is_nestgate() {
        assert_eq!(crate::primal_names::roles::STORAGE, "nestgate");
    }
}
