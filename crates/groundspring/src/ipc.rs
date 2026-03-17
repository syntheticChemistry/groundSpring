// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Type-safe IPC service traits and client for ecosystem communication.
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
//! - `measurement.*` — groundSpring experiment capabilities
//! - `compute.*` — GPU/NPU dispatch
//! - `storage.*` — Provenance and data
//! - `data.*` — Live data pipelines (NCBI, NOAA, IRIS)
//!
//! # Client usage
//!
//! ```rust,ignore
//! use groundspring::ipc::GroundSpringClient;
//!
//! let client = GroundSpringClient::connect_unix("/run/user/1000/biomeos/groundspring.sock").await?;
//! let result = client.anderson_validation(3.5, 1000, "f64".into()).await?;
//! ```

use std::path::Path;

/// groundSpring measurement capabilities exposed to the ecosystem.
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

// ─── Client ──────────────────────────────────────────────────────────────

/// Error type for typed IPC client operations.
///
/// Structured variants for the IPC lifecycle: connect, transport, and
/// remote (application-level) errors. Pattern source: rhizoCrypt v0.13.0
/// `IpcErrorPhase` / healthSpring V30.
#[derive(Debug, thiserror::Error)]
pub enum IpcError {
    /// Failed to connect to the IPC socket.
    #[error("ipc connect: {0}")]
    Connect(String),
    /// Transport-level error during an RPC call.
    #[error("ipc transport: {0}")]
    Transport(String),
    /// Remote endpoint returned an application error.
    #[error("ipc remote: {0}")]
    Remote(String),
    /// No IPC socket discovered via environment.
    #[error("ipc discovery: {0}")]
    Discovery(String),
}

/// Result alias for IPC operations.
pub type IpcResult<T> = Result<T, IpcError>;

/// Typed IPC client for groundSpring measurement capabilities.
///
/// Wraps a tarpc channel connected over Unix domain socket transport.
/// The client discovers the endpoint at runtime via socket path, never
/// hardcoding primal addresses.
pub struct GroundSpringClient {
    inner: GroundSpriScienceClient,
}

impl GroundSpringClient {
    /// Connect to a groundSpring IPC endpoint over Unix domain socket.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the socket cannot be connected.
    pub async fn connect_unix(path: &Path) -> IpcResult<Self> {
        let transport =
            tarpc::serde_transport::unix::connect(path, tarpc::tokio_serde::formats::Json::default)
                .await
                .map_err(|e| IpcError::Connect(format!("{}: {e}", path.display())))?;

        let client =
            GroundSpriScienceClient::new(tarpc::client::Config::default(), transport).spawn();

        Ok(Self { inner: client })
    }

    /// Connect via runtime-discovered socket (env-based discovery).
    ///
    /// Fallback chain:
    /// 1. `GROUNDSPRING_IPC_SOCKET` env var
    /// 2. `$XDG_RUNTIME_DIR/biomeos/groundspring-ipc.sock`
    /// 3. `<temp_dir>/groundspring-ipc.sock`
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if no socket is found or connection fails.
    pub async fn connect_discovered() -> IpcResult<Self> {
        let path = discover_ipc_socket()
            .ok_or_else(|| IpcError::Discovery("no groundspring IPC socket found".into()))?;
        Self::connect_unix(&path).await
    }

    /// Validate an Anderson localization experiment.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] on transport or remote error.
    pub async fn anderson_validation(
        &self,
        disorder_strength: f64,
        lattice_size: u32,
        precision: String,
    ) -> IpcResult<String> {
        self.inner
            .anderson_validation(
                tarpc::context::current(),
                disorder_strength,
                lattice_size,
                precision,
            )
            .await
            .map_err(|e| IpcError::Transport(format!("{e}")))?
            .map_err(IpcError::Remote)
    }

    /// Run noise decomposition (bias-variance).
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] on transport or remote error.
    pub async fn noise_decomposition(
        &self,
        observed: Vec<f64>,
        predicted: Vec<f64>,
    ) -> IpcResult<String> {
        self.inner
            .noise_decomposition(tarpc::context::current(), observed, predicted)
            .await
            .map_err(|e| IpcError::Transport(format!("{e}")))?
            .map_err(IpcError::Remote)
    }

    /// Check cross-substrate parity.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] on transport or remote error.
    pub async fn parity_check(&self, exp_id: u32, substrate: String) -> IpcResult<String> {
        self.inner
            .parity_check(tarpc::context::current(), exp_id, substrate)
            .await
            .map_err(|e| IpcError::Transport(format!("{e}")))?
            .map_err(IpcError::Remote)
    }

    /// Propagate ET₀ uncertainty through FAO-56.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] on transport or remote error.
    pub async fn et0_propagation(&self, params: String) -> IpcResult<String> {
        self.inner
            .et0_propagation(tarpc::context::current(), params)
            .await
            .map_err(|e| IpcError::Transport(format!("{e}")))?
            .map_err(IpcError::Remote)
    }
}

/// Discover the groundSpring IPC socket path via environment.
fn discover_ipc_socket() -> Option<std::path::PathBuf> {
    if let Ok(explicit) = std::env::var("GROUNDSPRING_IPC_SOCKET") {
        let path = std::path::PathBuf::from(explicit);
        if path.exists() {
            return Some(path);
        }
    }

    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        let path = std::path::PathBuf::from(xdg)
            .join(crate::primal_names::BIOMEOS_SOCKET_DIR)
            .join("groundspring-ipc.sock");
        if path.exists() {
            return Some(path);
        }
    }

    let path = std::env::temp_dir().join("groundspring-ipc.sock");
    if path.exists() {
        return Some(path);
    }

    None
}
