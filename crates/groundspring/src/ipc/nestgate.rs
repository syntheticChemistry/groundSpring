// SPDX-License-Identifier: AGPL-3.0-or-later

//! IPC interface for NestGate storage and data pipelines.
//!
//! NestGate provides content-addressed storage and live data acquisition
//! from public repositories (NCBI, NOAA GHCND, IRIS FDSN). groundSpring
//! uses NestGate for:
//! - Storing/retrieving validation artifacts with provenance
//! - Acquiring real-world datasets for Experiments 029-032
//!
//! # Capability surface
//!
//! - `storage.put` / `storage.get` — content-addressed storage
//! - `data.ncbi_search` / `data.ncbi_fetch` — NCBI sequence data
//! - `data.noaa_ghcnd` — NOAA weather observations
//! - `data.iris_stations` / `data.iris_events` — IRIS seismic data

/// Storage capabilities (routed via Neural API to NestGate).
#[tarpc::service]
pub trait StorageService {
    /// Store a key-value pair with provenance.
    async fn put(key: String, value: String, family_id: String) -> Result<(), String>;

    /// Retrieve a value by key.
    async fn get(key: String, family_id: String) -> Result<String, String>;
}

/// Live data pipeline capabilities (routed via Neural API to NestGate).
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
