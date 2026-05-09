// SPDX-License-Identifier: AGPL-3.0-or-later

//! IPC interface for barraCuda GPU/NPU compute dispatch.
//!
//! Maps `compute.*` JSON-RPC methods to typed tarpc traits.
//! barraCuda provides GPU-accelerated math primitives that groundSpring
//! delegates to via the `barracuda` Cargo feature (library path) or
//! via IPC through ToadStool/biomeOS (ecobin path).
//!
//! # Primal-proof evolution
//!
//! The library path (`barracuda::*` direct calls) is the current default.
//! The IPC path (`compute.*` via biomeOS) is the target for sovereign
//! NUCLEUS deployment where barraCuda runs as a separate ecobin.

/// Compute dispatch capabilities via barraCuda (routed via Neural API).
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
