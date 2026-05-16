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
//! # Signal dispatch (Wave 17 + Wave 20)
//!
//! `nest.store` collapses `content.put` → `dag.event.append` → `spine.seal`
//! → `braid.create` into a single graph-managed dispatch via
//! [`nest_store_dispatch`]. Falls back to raw `content.put` if biomeOS
//! signal dispatch is unavailable.
//!
//! `nest.commit` (Wave 20) collapses `event.append` → `crypto.sign` →
//! `content.put` → `session.commit` → `braid.create` via
//! [`nest_commit_dispatch`]. Used for LTEE session finalization.
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

/// Dispatch a `nest.store` signal via `CompositionContext`.
///
/// Collapses `content.put` → `dag.event.append` → `spine.seal` → `braid.create`
/// into a single signal dispatch. biomeOS manages the graph execution.
/// Falls back to raw `content.put` if `CompositionContext` is unavailable.
///
/// Wave 17 signal adoption: provenance-heavy sequences collapse to one call.
#[cfg(feature = "biomeos")]
pub fn nest_store_dispatch(
    content: &[u8],
    author: Option<&str>,
    metadata: Option<&serde_json::Value>,
) -> crate::biomeos::Result<Option<serde_json::Value>> {
    let mut params = serde_json::json!({
        "content": base64_encode(content),
    });
    if let Some(a) = author {
        params["author"] = serde_json::Value::String(a.to_string());
    }
    if let Some(m) = metadata {
        params["metadata"] = m.clone();
    }

    let ctx_result = std::panic::catch_unwind(|| {
        let mut ctx =
            primalspring::composition::CompositionContext::from_live_discovery_with_fallback();
        ctx.dispatch("nest.store", params.clone())
    });

    match ctx_result {
        Ok(Ok(result)) => {
            tracing::info!("nest.store signal dispatched successfully");
            Ok(Some(result))
        }
        Ok(Err(e)) => {
            tracing::debug!("nest.store signal failed ({e}), falling back to content.put");
            let encoded = base64_encode(content);
            let family_id = crate::biomeos::FAMILY_ID;
            try_content_put(&encoded, &encoded, family_id)
        }
        Err(_) => {
            tracing::debug!("CompositionContext unavailable, falling back to content.put");
            let encoded = base64_encode(content);
            let family_id = crate::biomeos::FAMILY_ID;
            try_content_put(&encoded, &encoded, family_id)
        }
    }
}

/// Dispatch a `nest.commit` signal via `CompositionContext`.
///
/// Collapses `event.append` → `crypto.sign` → `content.put` →
/// `session.commit` → `braid.create` into a single signal dispatch.
/// biomeOS manages the graph execution (sequential, 5 primals).
///
/// Wave 20 signal adoption: session finalization via signal dispatch.
/// Falls back to legacy `provenance.session_dehydrate` if unavailable.
#[cfg(feature = "biomeos")]
pub fn nest_commit_dispatch(session_id: &str) -> crate::biomeos::Result<Option<serde_json::Value>> {
    let params = serde_json::json!({
        "session_id": session_id,
    });

    let ctx_result = std::panic::catch_unwind(|| {
        let mut ctx =
            primalspring::composition::CompositionContext::from_live_discovery_with_fallback();
        ctx.dispatch("nest.commit", params.clone())
    });

    match ctx_result {
        Ok(Ok(result)) => {
            tracing::info!(session_id = %session_id, "nest.commit signal dispatched");
            Ok(Some(result))
        }
        Ok(Err(e)) => {
            tracing::debug!("nest.commit signal failed ({e}), falling back to legacy dehydrate");
            Ok(None)
        }
        Err(_) => {
            tracing::debug!("CompositionContext unavailable for nest.commit");
            Ok(None)
        }
    }
}

#[cfg(feature = "biomeos")]
fn base64_encode(data: &[u8]) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(data.len() * 4 / 3 + 4);
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;
        let _ = out.write_char(CHARS[(b0 >> 2) & 0x3F] as char);
        let _ = out.write_char(CHARS[((b0 << 4) | (b1 >> 4)) & 0x3F] as char);
        if chunk.len() > 1 {
            let _ = out.write_char(CHARS[((b1 << 2) | (b2 >> 6)) & 0x3F] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            let _ = out.write_char(CHARS[b2 & 0x3F] as char);
        } else {
            out.push('=');
        }
    }
    out
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
