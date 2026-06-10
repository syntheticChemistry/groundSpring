// SPDX-License-Identifier: AGPL-3.0-or-later

//! IPC interface for `ToadStool` compute orchestration.
//!
//! `ToadStool` provides compute orchestration for the `NUCLEUS`. `groundSpring`
//! uses `ToadStool` to dispatch GPU/NPU workloads when `barraCuda` is deployed
//! as a separate ecobin rather than linked as a library dependency.
//!
//! # Capability surface
//!
//! - `compute.execute` — synchronous dispatch
//! - `compute.submit` — async job submission
//! - `compute.capabilities` — hardware/capability enumeration
//! - `compute.device.enumerate` — Phase D `LocalDeviceFactory` device listing (S254)
//! - `toadstool.validate` — workload pre-flight validation (Tier 2, Pass 14)
//! - `toadstool.list_workloads` — auto-discover available workloads (Tier 2, Pass 14)

/// `ToadStool` orchestration traits for compute pipeline management.
#[tarpc::service]
pub trait OrchestrationService {
    /// Dispatch a validated compute pipeline.
    async fn dispatch_pipeline(pipeline_json: String) -> Result<String, String>;

    /// Query pipeline execution status.
    async fn pipeline_status(pipeline_id: String) -> Result<String, String>;

    /// List available compute substrates (GPU, NPU, CPU).
    async fn substrate_inventory() -> Result<String, String>;

    /// Validate a workload TOML before dispatch.
    ///
    /// Returns validation result with GPU availability, precision tier,
    /// estimated dispatch time, warnings, and required capabilities.
    /// Upstream: `toadstool.validate` (Pass 14, S250).
    async fn validate(workload_path: String, dry_run: bool) -> Result<String, String>;

    /// List workloads available for dispatch.
    ///
    /// `filter` selects which workloads to return: `"active"`, `"all"`, or
    /// `"ready"`. Upstream: `toadstool.list_workloads` (Pass 14, S245+).
    async fn list_workloads(filter: String) -> Result<String, String>;

    /// Enumerate locally available compute devices via Phase D `LocalDeviceFactory`.
    ///
    /// Returns device descriptors including vendor, driver, DRM node, and
    /// compute capabilities. Upstream: `compute.device.enumerate` (S254).
    async fn device_enumerate() -> Result<String, String>;
}

/// Validate a workload via `ToadStool` JSON-RPC before dispatch.
///
/// Sends a `toadstool.validate` call to the discovered `ToadStool` socket.
/// Returns the response JSON containing `valid`, `gpu_available`, `precision_tier`,
/// `estimated_dispatch_time_ms`, `warnings`, and `required_capabilities`.
///
/// # Errors
///
/// Returns `BiomeOsError` if `ToadStool` is not discovered or the IPC call fails.
#[cfg(feature = "biomeos")]
pub fn validate_workload(
    socket: &std::path::Path,
    workload_path: &str,
    dry_run: bool,
) -> crate::biomeos::Result<serde_json::Value> {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "toadstool.validate",
        "params": {
            "workload_path": workload_path,
            "dry_run": dry_run,
        },
        "id": 1
    })
    .to_string();
    let response = crate::biomeos::raw_rpc_call(socket, &request)?;
    crate::biomeos::protocol::extract_rpc_result(&response)
}

/// List available workloads via `ToadStool` JSON-RPC.
///
/// `filter` selects which workloads to return (`"active"`, `"all"`, `"ready"`).
///
/// # Errors
///
/// Returns `BiomeOsError` if `ToadStool` is not discovered or the IPC call fails.
#[cfg(feature = "biomeos")]
pub fn list_workloads(
    socket: &std::path::Path,
    filter: &str,
) -> crate::biomeos::Result<serde_json::Value> {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "toadstool.list_workloads",
        "params": {
            "filter": filter,
        },
        "id": 1
    })
    .to_string();
    let response = crate::biomeos::raw_rpc_call(socket, &request)?;
    crate::biomeos::protocol::extract_rpc_result(&response)
}

/// Enumerate locally available compute devices via `ToadStool` Phase D factory.
///
/// Returns device descriptors discovered by `LocalDeviceFactory` (AMD DRM,
/// NVIDIA VFIO, etc.). Upstream: `compute.device.enumerate` (S254).
///
/// # Errors
///
/// Returns `BiomeOsError` if `ToadStool` is not discovered or the IPC call fails.
#[cfg(feature = "biomeos")]
pub fn device_enumerate(socket: &std::path::Path) -> crate::biomeos::Result<serde_json::Value> {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "compute.device.enumerate",
        "params": {},
        "id": 1
    })
    .to_string();
    let response = crate::biomeos::raw_rpc_call(socket, &request)?;
    crate::biomeos::protocol::extract_rpc_result(&response)
}

/// Attempt to discover `ToadStool` and enumerate compute devices.
///
/// Returns `Ok(None)` if `ToadStool` is not available (graceful degradation).
#[cfg(feature = "biomeos")]
pub fn try_device_enumerate() -> crate::biomeos::Result<Option<serde_json::Value>> {
    crate::primal_names::discover_socket(crate::primal_names::roles::COMPUTE).map_or_else(
        || {
            tracing::debug!("ToadStool not discovered — device enumeration skipped");
            Ok(None)
        },
        |socket| device_enumerate(&socket).map(Some),
    )
}

/// Attempt to discover `ToadStool` and validate a workload.
///
/// Returns `Ok(None)` if `ToadStool` is not available (graceful degradation).
/// Returns `Ok(Some(response))` on successful validation.
///
/// # Errors
///
/// Returns `BiomeOsError` if the IPC call fails after successful discovery.
#[cfg(feature = "biomeos")]
pub fn try_validate_workload(
    workload_path: &str,
    dry_run: bool,
) -> crate::biomeos::Result<Option<serde_json::Value>> {
    crate::primal_names::discover_socket(crate::primal_names::roles::COMPUTE).map_or_else(
        || {
            tracing::debug!("ToadStool not discovered — workload validation skipped");
            Ok(None)
        },
        |socket| validate_workload(&socket, workload_path, dry_run).map(Some),
    )
}

/// Attempt to discover `ToadStool` and list available workloads.
///
/// Returns `Ok(None)` if `ToadStool` is not available (graceful degradation).
/// Defaults to `"active"` filter if no specific filter is needed.
///
/// # Errors
///
/// Returns `BiomeOsError` if the IPC call fails after successful discovery.
#[cfg(feature = "biomeos")]
pub fn try_list_workloads(filter: &str) -> crate::biomeos::Result<Option<serde_json::Value>> {
    crate::primal_names::discover_socket(crate::primal_names::roles::COMPUTE).map_or_else(
        || {
            tracing::debug!("ToadStool not discovered — workload listing skipped");
            Ok(None)
        },
        |socket| list_workloads(&socket, filter).map(Some),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tarpc_trait_compiles() {
        fn _assert_service<T: OrchestrationService>() {}
    }

    #[test]
    fn compute_role_is_toadstool() {
        assert_eq!(crate::primal_names::roles::COMPUTE, "toadstool");
    }
}
