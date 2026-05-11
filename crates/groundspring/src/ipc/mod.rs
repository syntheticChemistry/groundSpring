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
//! # Per-primal modules
//!
//! Each primal has a dedicated submodule defining its IPC surface:
//! - [`barracuda`] — GPU/NPU compute dispatch via barraCuda
//! - [`toadstool`] — Compute orchestration via ToadStool
//! - [`nestgate`] — Storage and live data pipelines via NestGate
//! - [`beardog`] — Cryptographic operations via BearDog
//! - [`songbird`] — Network discovery and mesh via Songbird
//! - [`skunkbat`] — Audit logging via skunkBat (JH-5)
//! - [`coralreef`] — Sovereign shader compilation via coralReef (stub — awaiting SM rebuild)
//!
//! # Semantic method naming
//!
//! All methods follow `domain.operation` format per
//! `SEMANTIC_METHOD_NAMING_STANDARD`:
//! - `measurement.*` — groundSpring experiment capabilities
//! - `compute.*` — GPU/NPU dispatch
//! - `storage.*` — Provenance and data
//! - `data.*` — Live data pipelines (NCBI, NOAA, IRIS)
//! - `crypto.*` — Cryptographic operations
//! - `discovery.*` — Network and capability discovery
//! - `security.*` — Audit logging and threat detection

pub mod barracuda;
pub mod beardog;
pub mod coralreef;
pub mod nestgate;
pub mod skunkbat;
pub mod songbird;
pub mod toadstool;

mod client;
mod discovery;

pub use crate::ipc_error::{IpcError, IpcResult};
pub use client::GroundSpringClient;
pub use discovery::{discover_ipc_socket, tarpc_sock_name};

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

    /// Bootstrap confidence interval estimation.
    async fn bootstrap(data: Vec<f64>, n_resamples: u32) -> Result<String, String>;

    /// Rarefaction analysis for sequencing depth.
    async fn rarefaction(counts: Vec<u64>, depth: u32) -> Result<String, String>;

    /// Freeze-out curve chi-squared fitting.
    async fn freeze_out(mu_b_values: Vec<f64>, t_values: Vec<f64>) -> Result<String, String>;

    /// ESN regime classification.
    async fn regime_classification(eigenvalues: Vec<f64>) -> Result<String, String>;

    /// Uncertainty budget decomposition.
    async fn uncertainty_budget(data: Vec<f64>, params: String) -> Result<String, String>;

    /// Spectral function feature extraction.
    async fn spectral_features(correlator: Vec<f64>, params: String) -> Result<String, String>;

    /// Wright-Fisher drift simulation.
    async fn drift(params: String) -> Result<String, String>;

    /// Transfer-matrix band edge structure.
    async fn band_edge(potential: Vec<f64>, params: String) -> Result<String, String>;

    /// Rare biosphere signal detection.
    async fn rare_biosphere(counts: Vec<u64>, params: String) -> Result<String, String>;

    /// Gillespie SSA trajectory simulation.
    async fn gillespie(
        synthesis_rates: Vec<f64>,
        degradation_rate: f64,
        params: String,
    ) -> Result<String, String>;

    /// Bistable phenotypic switching simulation.
    async fn bistable(params: String) -> Result<String, String>;

    /// Quasispecies error threshold analysis.
    async fn quasispecies(sigma: f64, params: String) -> Result<String, String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tarpc_sock_name_contains_suffix() {
        let name = tarpc_sock_name();
        assert!(name.ends_with("ipc.sock"));
    }

    #[test]
    fn discover_socket_does_not_panic() {
        let _ = discover_ipc_socket();
    }
}
