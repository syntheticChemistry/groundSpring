// SPDX-License-Identifier: AGPL-3.0-or-later

//! Typed IPC client for groundSpring measurement capabilities.

use std::path::Path;

use super::GroundSpringScienceClient;
use super::discovery::discover_ipc_socket;
use crate::ipc_error::{IpcError, IpcResult};

/// Typed IPC client for groundSpring measurement capabilities.
///
/// Wraps a tarpc channel connected over Unix domain socket transport.
/// The client discovers the endpoint at runtime via socket path, never
/// hardcoding primal addresses.
pub struct GroundSpringClient {
    inner: GroundSpringScienceClient,
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
            GroundSpringScienceClient::new(tarpc::client::Config::default(), transport).spawn();

        Ok(Self { inner: client })
    }

    /// Connect via runtime-discovered socket (env-based discovery).
    ///
    /// Fallback chain:
    /// 1. `GROUNDSPRING_SOCKET` env var (via [`crate::primal_names::socket_env_var`])
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
