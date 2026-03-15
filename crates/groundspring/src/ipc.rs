// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Type-safe IPC service traits for ecosystem communication.
//!
//! Implements the ecoPrimals `UNIVERSAL_IPC_STANDARD_V3`:
//! - **JSON-RPC 2.0**: Human-readable, debuggable (see [`crate::biomeos`])
//! - **tarpc**: High-performance, type-safe, Rust-native (this module)
//!
//! Both protocols coexist; transport negotiation selects the optimal wire
//! format at connection time. JSON-RPC is always available; tarpc activates
//! when both endpoints support it.
//!
//! # Semantic method naming
//!
//! All methods follow `domain.operation` format per
//! `SEMANTIC_METHOD_NAMING_STANDARD`:
//! - `science.*` — groundSpring experiment capabilities
//! - `compute.*` — GPU/NPU dispatch
//! - `storage.*` — Provenance and data
//! - `data.*` — Live data pipelines (NCBI, NOAA, IRIS)

/// groundSpring science capabilities exposed to the ecosystem.
///
/// These are the capabilities groundSpring registers with `biomeOS` for
/// other primals to discover and invoke at runtime.
#[tarpc::service]
pub trait GroundSpringScience {
    /// Validate an Anderson localization experiment configuration.
    async fn anderson_validation(
        disorder_strength: f64,
        lattice_size: u32,
        precision: String,
    ) -> Result<String, String>;

    /// Run noise decomposition (bias-variance) on provided measurements.
    async fn noise_decomposition(observed: Vec<f64>, predicted: Vec<f64>)
    -> Result<String, String>;

    /// Check cross-substrate parity for an experiment.
    async fn parity_check(exp_id: u32, substrate: String) -> Result<String, String>;

    /// Propagate ET₀ uncertainty through FAO-56 Penman-Monteith.
    async fn et0_propagation(params: String) -> Result<String, String>;
}

/// Compute dispatch capabilities (routed via Neural API).
#[tarpc::service]
pub trait ComputeDispatch {
    /// Submit a compute job for asynchronous execution.
    async fn submit(op: String, params_json: String) -> Result<String, String>;

    /// Query the status of a submitted job.
    async fn status(job_id: String) -> Result<String, String>;

    /// Execute a compute operation synchronously.
    async fn execute(op: String, params_json: String) -> Result<String, String>;

    /// List available compute capabilities and hardware.
    async fn capabilities() -> Result<String, String>;
}

/// Storage capabilities (routed via Neural API).
#[tarpc::service]
pub trait StorageService {
    /// Store a key-value pair with provenance.
    async fn put(key: String, value: String, family_id: String) -> Result<(), String>;

    /// Retrieve a value by key.
    async fn get(key: String, family_id: String) -> Result<String, String>;
}

/// Live data pipeline capabilities (routed via Neural API).
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
