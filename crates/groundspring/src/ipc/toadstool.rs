// SPDX-License-Identifier: AGPL-3.0-or-later

//! IPC interface for ToadStool compute orchestration.
//!
//! ToadStool provides compute orchestration for the NUCLEUS. groundSpring
//! uses ToadStool to dispatch GPU/NPU workloads when barraCuda is deployed
//! as a separate ecobin rather than linked as a library dependency.
//!
//! # Capability surface
//!
//! - `compute.execute` — synchronous dispatch
//! - `compute.submit` — async job submission
//! - `compute.capabilities` — hardware/capability enumeration

/// ToadStool orchestration traits for compute pipeline management.
#[tarpc::service]
pub trait OrchestrationService {
    /// Dispatch a validated compute pipeline.
    async fn dispatch_pipeline(pipeline_json: String) -> Result<String, String>;

    /// Query pipeline execution status.
    async fn pipeline_status(pipeline_id: String) -> Result<String, String>;

    /// List available compute substrates (GPU, NPU, CPU).
    async fn substrate_inventory() -> Result<String, String>;
}
