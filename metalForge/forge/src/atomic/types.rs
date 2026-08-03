// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

use std::collections::BTreeMap;

use crate::inventory::Inventory;
use crate::topology::Topology;

/// Primal health status within an atomic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimalHealth {
    /// Provider is responding to health checks.
    Healthy,
    /// Provider is present but degraded (slow, partial capability).
    Degraded,
    /// Provider is not responding.
    Unavailable,
    /// Capability is not required for this atomic type.
    NotRequired,
}

/// Capabilities provided by a NUCLEUS atomic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AtomicCapability {
    /// Encrypted inter-primal communication (IPC foundation).
    SecureIpc,
    /// GPU/CPU compute dispatch via barracuda.
    ComputeDispatch,
    /// NPU inference (int8 quantized, via akida-driver).
    NpuInference,
    /// Data storage and provenance.
    DataStorage,
    /// Live data pipelines (NCBI, NOAA, IRIS).
    LiveData,
    /// AI/ML inference.
    AiInference,
    /// Cross-substrate pipeline orchestration (metalForge).
    PipelineOrchestration,
}

/// Runtime-discovered capability provider health map.
///
/// Keys are capability identifiers (e.g. `"crypto"`, `"discovery"`),
/// values are the health status of the provider for that capability.
/// Populated at runtime via `topology.metrics`, never hardcoded.
pub type ProviderHealthMap = BTreeMap<String, PrimalHealth>;

/// Tower Atomic — secure IPC foundation.
///
/// The foundational atomic that all others build upon. Provides
/// secure inter-primal communication. Discovered at runtime via
/// capability probing, not by naming specific primals.
#[derive(Debug)]
pub struct TowerAtomic {
    /// Node identifier (e.g. "eastgate", "biomegate").
    pub node_id: String,
    /// Runtime-discovered capability providers and their health.
    pub providers: ProviderHealthMap,
    /// biomeOS Neural API socket path (discovered at runtime).
    pub socket_path: Option<String>,
}

/// Node Atomic — Tower + compute dispatch.
///
/// Extends Tower with compute capabilities. Hosts the metalForge
/// substrate inventory for hardware-aware dispatch.
#[derive(Debug)]
pub struct NodeAtomic {
    /// Tower foundation.
    pub tower: TowerAtomic,
    /// Compute dispatch provider health.
    pub compute: PrimalHealth,
    /// Local hardware inventory (GPUs, NPUs, CPU).
    pub inventory: Inventory,
    /// Device topology for transfer cost modeling.
    pub topology: Topology,
}

/// Nest Atomic — Tower + data storage.
///
/// Extends Tower with data capabilities. Provides storage,
/// provenance, and live data pipeline access (NCBI, NOAA, IRIS).
#[derive(Debug)]
pub struct NestAtomic {
    /// Tower foundation.
    pub tower: TowerAtomic,
    /// Data storage provider health.
    pub storage: PrimalHealth,
    /// Available data capabilities.
    pub data_capabilities: Vec<AtomicCapability>,
}

/// Full NUCLEUS — all capabilities for complete ecosystem.
#[derive(Debug)]
pub struct FullNucleus {
    /// Node atomic (Tower + compute).
    pub node: NodeAtomic,
    /// Data storage provider health.
    pub storage: PrimalHealth,
    /// AI/ML inference provider health.
    pub inference: PrimalHealth,
}
