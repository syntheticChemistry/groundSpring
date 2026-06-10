// SPDX-License-Identifier: AGPL-3.0-or-later

//! IPC interface for `barraCuda` GPU/NPU compute dispatch.
//!
//! Maps `compute.*` JSON-RPC methods to typed tarpc traits.
//! `barraCuda` provides GPU-accelerated math primitives that `groundSpring`
//! delegates to via the `barracuda` Cargo feature (library path) or
//! via IPC through `ToadStool`/`biomeOS` (ecobin path).
//!
//! # Capability surface
//!
//! - `compute.execute` — synchronous compute dispatch
//! - `compute.submit` — async job submission
//! - `compute.capabilities` — hardware/capability enumeration
//! - `barracuda.precision.route` — precision tier advisory (Tier 2, Pass 14)
//! - `health.version` — trio-consistent version probe (Sprint 69)

/// Compute dispatch capabilities via `barraCuda` (routed via Neural API).
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

    /// Query precision routing advice for a domain and hardware hint.
    ///
    /// Returns recommended precision tier (`DF64`, `F32`, etc.), FMA safety,
    /// and compiler requirements. Upstream: `barracuda.precision.route` (Pass 14).
    async fn precision_route(domain: String, hardware_hint: String) -> Result<String, String>;

    /// Trio-consistent version probe for automated tooling.
    ///
    /// Returns `{ primal, version, rust_version }`. Matches `toadStool` and
    /// `coralReef` `health.version` surface (Sprint 69).
    async fn health_version() -> Result<String, String>;
}

/// Query `barraCuda` precision routing advice via JSON-RPC.
///
/// Sends a `barracuda.precision.route` call to the discovered `barraCuda` socket.
/// Returns the response JSON containing `recommended_tier`, `fma_safe`,
/// `requires_compiler`, and `hardware_hint`.
///
/// # Errors
///
/// Returns `BiomeOsError` if `barraCuda` is not discovered or the IPC call fails.
#[cfg(feature = "biomeos")]
pub fn precision_route(
    socket: &std::path::Path,
    domain: &str,
    hardware_hint: &str,
) -> crate::biomeos::Result<serde_json::Value> {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "barracuda.precision.route",
        "params": {
            "domain": domain,
            "hardware_hint": hardware_hint,
        },
        "id": 1
    })
    .to_string();
    let response = crate::biomeos::raw_rpc_call(socket, &request)?;
    crate::biomeos::protocol::extract_rpc_result(&response)
}

/// Query `barraCuda` build identity via JSON-RPC.
///
/// Trio-consistent with `toadStool` and `coralReef` `health.version` (Sprint 69).
/// Returns `{ primal, version, rust_version }` for plasmidBin doctor and
/// upgrade verification.
///
/// # Errors
///
/// Returns `BiomeOsError` if `barraCuda` is not discovered or the IPC call fails.
#[cfg(feature = "biomeos")]
pub fn health_version(socket: &std::path::Path) -> crate::biomeos::Result<serde_json::Value> {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.version",
        "params": {},
        "id": 1
    })
    .to_string();
    let response = crate::biomeos::raw_rpc_call(socket, &request)?;
    crate::biomeos::protocol::extract_rpc_result(&response)
}

/// Attempt to discover `barraCuda` and query build identity.
///
/// Returns `Ok(None)` if `barraCuda` is not available (graceful degradation).
#[cfg(feature = "biomeos")]
pub fn try_health_version() -> crate::biomeos::Result<Option<serde_json::Value>> {
    crate::primal_names::discover_socket(crate::primal_names::roles::GPU_MATH).map_or_else(
        || {
            tracing::debug!("barraCuda not discovered — health.version skipped");
            Ok(None)
        },
        |socket| health_version(&socket).map(Some),
    )
}

/// Attempt to discover `barraCuda` and query precision routing advice.
///
/// Returns `Ok(None)` if `barraCuda` is not available (graceful degradation).
/// Returns `Ok(Some(response))` on successful query.
///
/// # Errors
///
/// Returns `BiomeOsError` if the IPC call fails after successful discovery.
#[cfg(feature = "biomeos")]
pub fn try_precision_route(
    domain: &str,
    hardware_hint: &str,
) -> crate::biomeos::Result<Option<serde_json::Value>> {
    crate::primal_names::discover_socket(crate::primal_names::roles::GPU_MATH).map_or_else(
        || {
            tracing::debug!("barraCuda not discovered — precision route skipped");
            Ok(None)
        },
        |socket| precision_route(&socket, domain, hardware_hint).map(Some),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tarpc_trait_compiles() {
        fn _assert_service<T: ComputeDispatch>() {}
    }

    #[test]
    fn gpu_math_role_is_barracuda() {
        assert_eq!(crate::primal_names::roles::GPU_MATH, "barracuda");
    }

    #[test]
    fn compute_role_is_toadstool() {
        assert_eq!(crate::primal_names::roles::COMPUTE, "toadstool");
    }
}
