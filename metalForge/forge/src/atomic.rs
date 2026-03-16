// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! NUCLEUS atomic types — Tower, Node, and Nest compositions.
//!
//! In the ecoPrimals ecosystem, primals are composed into **atomics**:
//!
//! | Atomic | Required Capabilities | Provides |
//! |--------|----------------------|----------|
//! | **Tower** | `SecureIpc` | Encrypted IPC foundation |
//! | **Node** | Tower + `ComputeDispatch` | + GPU compute dispatch |
//! | **Nest** | Tower + `DataStorage` | + Data storage & provenance |
//! | **Full NUCLEUS** | All capabilities | Complete ecosystem |
//!
//! Each atomic declares the capabilities it provides and discovers
//! providers at runtime via biomeOS `topology.metrics`. No hardcoded
//! primal names — only capability semantics.
//!
//! # Mixed hardware coordination
//!
//! Node atomics can host multiple substrates (GPU, NPU, CPU). The
//! [`NodeAtomic`] type combines metalForge substrate inventory with
//! NUCLEUS compute routing, enabling mixed-hardware pipelines:
//!
//! ```text
//! Nest (data) → Node (compute: NPU classify → GPU refine) → Nest (store)
//! ```

use std::collections::BTreeMap;

use crate::inventory::Inventory;
use crate::pipeline::{Pipeline, ResolvedPipeline};
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

impl TowerAtomic {
    /// Create a Tower atomic for a given node.
    pub fn new(node_id: impl Into<String>) -> Self {
        Self {
            node_id: node_id.into(),
            providers: ProviderHealthMap::new(),
            socket_path: None,
        }
    }

    /// Set health for a capability provider discovered at runtime.
    pub fn set_provider_health(&mut self, capability: &str, health: PrimalHealth) {
        self.providers.insert(capability.to_string(), health);
    }

    /// Check if the Tower has healthy secure IPC (all required providers up).
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        !self.providers.is_empty()
            && self
                .providers
                .values()
                .all(|h| matches!(h, PrimalHealth::Healthy))
    }

    /// List capabilities provided by this atomic.
    #[must_use]
    pub fn capabilities(&self) -> Vec<AtomicCapability> {
        if self.is_healthy() {
            vec![AtomicCapability::SecureIpc]
        } else {
            Vec::new()
        }
    }
}

impl NodeAtomic {
    /// Create a Node atomic with hardware discovery.
    pub fn new(node_id: impl Into<String>) -> Self {
        let inventory = Inventory::discover();
        let topology = Topology::infer(&inventory.substrates);
        Self {
            tower: TowerAtomic::new(node_id),
            compute: PrimalHealth::Unavailable,
            inventory,
            topology,
        }
    }

    /// Create a Node atomic with a pre-built inventory (for testing).
    #[must_use]
    pub fn with_inventory(node_id: impl Into<String>, inventory: Inventory) -> Self {
        let topology = Topology::infer(&inventory.substrates);
        Self {
            tower: TowerAtomic::new(node_id),
            compute: PrimalHealth::Unavailable,
            inventory,
            topology,
        }
    }

    /// Check if compute dispatch is available.
    #[must_use]
    pub const fn can_compute(&self) -> bool {
        matches!(self.compute, PrimalHealth::Healthy)
            || matches!(self.compute, PrimalHealth::Degraded)
    }

    /// List capabilities provided by this atomic.
    #[must_use]
    pub fn capabilities(&self) -> Vec<AtomicCapability> {
        let mut caps = self.tower.capabilities();
        if self.can_compute() {
            caps.push(AtomicCapability::ComputeDispatch);
        }
        if self
            .inventory
            .first(crate::substrate::SubstrateKind::Npu)
            .is_some()
        {
            caps.push(AtomicCapability::NpuInference);
        }
        caps.push(AtomicCapability::PipelineOrchestration);
        caps
    }

    /// Plan a pipeline on this node's hardware.
    #[must_use]
    pub fn plan_pipeline<'a>(&'a self, pipeline: &'a Pipeline) -> ResolvedPipeline<'a> {
        pipeline.plan(&self.inventory.substrates, &self.topology)
    }

    /// Check if this node has NPU↔GPU P2P capability.
    #[must_use]
    pub fn has_npu_gpu_p2p(&self) -> bool {
        let npu_idx = self
            .inventory
            .substrates
            .iter()
            .position(|s| s.kind == crate::substrate::SubstrateKind::Npu);
        let gpu_idx = self
            .inventory
            .substrates
            .iter()
            .position(|s| s.kind == crate::substrate::SubstrateKind::Gpu);
        match (npu_idx, gpu_idx) {
            (Some(n), Some(g)) => self.topology.has_p2p(n, g),
            _ => false,
        }
    }
}

impl NestAtomic {
    /// Create a Nest atomic.
    pub fn new(node_id: impl Into<String>) -> Self {
        Self {
            tower: TowerAtomic::new(node_id),
            storage: PrimalHealth::Unavailable,
            data_capabilities: Vec::new(),
        }
    }

    /// Check if data storage is available.
    #[must_use]
    pub const fn can_store(&self) -> bool {
        matches!(self.storage, PrimalHealth::Healthy)
    }

    /// List capabilities provided by this atomic.
    #[must_use]
    pub fn capabilities(&self) -> Vec<AtomicCapability> {
        let mut caps = self.tower.capabilities();
        if self.can_store() {
            caps.push(AtomicCapability::DataStorage);
            for dc in &self.data_capabilities {
                if !caps.contains(dc) {
                    caps.push(*dc);
                }
            }
        }
        caps
    }
}

impl FullNucleus {
    /// Check if all capabilities are healthy.
    #[must_use]
    pub fn is_fully_healthy(&self) -> bool {
        self.node.tower.is_healthy()
            && self.node.can_compute()
            && matches!(self.storage, PrimalHealth::Healthy)
            && matches!(self.inference, PrimalHealth::Healthy)
    }

    /// List all capabilities of the full NUCLEUS.
    #[must_use]
    pub fn capabilities(&self) -> Vec<AtomicCapability> {
        let mut caps = self.node.capabilities();
        if matches!(self.storage, PrimalHealth::Healthy) {
            caps.push(AtomicCapability::DataStorage);
            caps.push(AtomicCapability::LiveData);
        }
        if matches!(self.inference, PrimalHealth::Healthy) {
            caps.push(AtomicCapability::AiInference);
        }
        caps
    }

    /// The sovereign degradation level — what's available when parts fail.
    #[must_use]
    pub fn degradation_level(&self) -> &'static str {
        if self.is_fully_healthy() {
            "Full NUCLEUS"
        } else if self.node.can_compute() && matches!(self.storage, PrimalHealth::Healthy) {
            "Node + Nest (no AI)"
        } else if self.node.can_compute() {
            "Node only (no storage)"
        } else if self.node.tower.is_healthy() {
            "Tower only (no compute)"
        } else {
            "Sovereign (local only)"
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use crate::pipeline::{Pipeline, Stage};
    use crate::substrate::{Capability, Identity, Properties, Substrate, SubstrateKind};

    fn test_inventory() -> Inventory {
        Inventory {
            substrates: vec![
                Substrate {
                    kind: SubstrateKind::Gpu,
                    identity: Identity::named("NVIDIA TITAN V"),
                    properties: Properties {
                        gpu_arch: Some(crate::substrate::GpuArch::Volta),
                        ..Properties::default()
                    },
                    capabilities: vec![
                        Capability::F64Compute,
                        Capability::ShaderDispatch,
                        Capability::NativeF64,
                    ],
                },
                Substrate {
                    kind: SubstrateKind::Npu,
                    identity: Identity::named("BrainChip AKD1000"),
                    properties: Properties::default(),
                    capabilities: vec![
                        Capability::QuantizedInference { bits: 8 },
                        Capability::BatchInference { max_batch: 8 },
                    ],
                },
                Substrate {
                    kind: SubstrateKind::Cpu,
                    identity: Identity::named("Test CPU"),
                    properties: Properties::default(),
                    capabilities: vec![Capability::F64Compute, Capability::F32Compute],
                },
            ],
        }
    }

    fn healthy_tower(node_id: &str) -> TowerAtomic {
        let mut tower = TowerAtomic::new(node_id);
        tower.set_provider_health("crypto", PrimalHealth::Healthy);
        tower.set_provider_health("discovery", PrimalHealth::Healthy);
        tower
    }

    #[test]
    fn tower_unhealthy_by_default() {
        let tower = TowerAtomic::new("eastgate");
        assert!(!tower.is_healthy());
        assert!(tower.capabilities().is_empty());
    }

    #[test]
    fn tower_healthy_when_providers_respond() {
        let tower = healthy_tower("eastgate");
        assert!(tower.is_healthy());
        assert!(tower.capabilities().contains(&AtomicCapability::SecureIpc));
    }

    #[test]
    fn tower_unhealthy_if_any_provider_down() {
        let mut tower = TowerAtomic::new("eastgate");
        tower.set_provider_health("crypto", PrimalHealth::Healthy);
        tower.set_provider_health("discovery", PrimalHealth::Unavailable);
        assert!(!tower.is_healthy());
    }

    #[test]
    fn node_has_compute_when_provider_healthy() {
        let mut node = NodeAtomic::with_inventory("eastgate", test_inventory());
        node.compute = PrimalHealth::Healthy;
        assert!(node.can_compute());
        assert!(
            node.capabilities()
                .contains(&AtomicCapability::ComputeDispatch)
        );
    }

    #[test]
    fn node_has_npu_inference_when_discovered() {
        let node = NodeAtomic::with_inventory("biomegate", test_inventory());
        assert!(
            node.capabilities()
                .contains(&AtomicCapability::NpuInference)
        );
    }

    #[test]
    fn node_always_has_pipeline_orchestration() {
        let node = NodeAtomic::with_inventory("eastgate", test_inventory());
        assert!(
            node.capabilities()
                .contains(&AtomicCapability::PipelineOrchestration)
        );
    }

    #[test]
    fn nest_can_store_when_provider_healthy() {
        let mut nest = NestAtomic::new("westgate");
        nest.storage = PrimalHealth::Healthy;
        assert!(nest.can_store());
        assert!(nest.capabilities().contains(&AtomicCapability::DataStorage));
    }

    #[test]
    fn nest_cannot_store_when_unavailable() {
        let nest = NestAtomic::new("westgate");
        assert!(!nest.can_store());
        assert!(!nest.capabilities().contains(&AtomicCapability::DataStorage));
    }

    #[test]
    fn full_nucleus_degradation_levels() {
        let mut nucleus = FullNucleus {
            node: NodeAtomic::with_inventory("strandgate", test_inventory()),
            storage: PrimalHealth::Unavailable,
            inference: PrimalHealth::Unavailable,
        };

        assert_eq!(nucleus.degradation_level(), "Sovereign (local only)");

        nucleus.node.tower = healthy_tower("strandgate");
        assert_eq!(nucleus.degradation_level(), "Tower only (no compute)");

        nucleus.node.compute = PrimalHealth::Healthy;
        assert_eq!(nucleus.degradation_level(), "Node only (no storage)");

        nucleus.storage = PrimalHealth::Healthy;
        assert_eq!(nucleus.degradation_level(), "Node + Nest (no AI)");

        nucleus.inference = PrimalHealth::Healthy;
        assert_eq!(nucleus.degradation_level(), "Full NUCLEUS");
    }

    #[test]
    fn node_plans_pipeline() {
        let node = NodeAtomic::with_inventory("biomegate", test_inventory());

        let pipeline = Pipeline::new("test pipeline").stage(crate::pipeline::Stage::new(
            "GPU compute",
            crate::dispatch::Workload::new(
                "compute",
                vec![Capability::F64Compute, Capability::ShaderDispatch],
            ),
            4096,
        ));

        let resolved = node.plan_pipeline(&pipeline);
        assert!(resolved.all_assigned());
        assert_eq!(
            resolved.stages[0].substrate.unwrap().kind,
            SubstrateKind::Gpu
        );
    }

    #[test]
    fn full_nucleus_capabilities() {
        let mut nucleus = FullNucleus {
            node: NodeAtomic::with_inventory("strandgate", test_inventory()),
            storage: PrimalHealth::Healthy,
            inference: PrimalHealth::Healthy,
        };
        nucleus.node.tower = healthy_tower("strandgate");
        nucleus.node.compute = PrimalHealth::Healthy;

        let caps = nucleus.capabilities();
        assert!(caps.contains(&AtomicCapability::SecureIpc));
        assert!(caps.contains(&AtomicCapability::ComputeDispatch));
        assert!(caps.contains(&AtomicCapability::DataStorage));
        assert!(caps.contains(&AtomicCapability::AiInference));
    }

    #[test]
    fn primal_health_equality() {
        assert_eq!(PrimalHealth::Healthy, PrimalHealth::Healthy);
        assert_ne!(PrimalHealth::Healthy, PrimalHealth::Degraded);
        assert_ne!(PrimalHealth::Unavailable, PrimalHealth::NotRequired);
    }

    #[test]
    fn atomic_capability_equality() {
        assert_eq!(AtomicCapability::SecureIpc, AtomicCapability::SecureIpc);
        assert_ne!(
            AtomicCapability::ComputeDispatch,
            AtomicCapability::DataStorage
        );
    }

    #[test]
    fn provider_health_map_is_dynamic() {
        let mut tower = TowerAtomic::new("testgate");
        assert!(tower.providers.is_empty());
        tower.set_provider_health("crypto", PrimalHealth::Healthy);
        tower.set_provider_health("mesh", PrimalHealth::Degraded);
        assert_eq!(tower.providers.len(), 2);
        assert_eq!(tower.providers["crypto"], PrimalHealth::Healthy);
        assert_eq!(tower.providers["mesh"], PrimalHealth::Degraded);
    }

    #[test]
    fn node_npu_gpu_link_exists() {
        let inv = test_inventory();
        let node = NodeAtomic::with_inventory("eastgate", inv);
        assert!(
            !node.topology.links_from(1).is_empty(),
            "inventory with NPU+GPU should have topology links"
        );
        let npu_idx = node
            .inventory
            .substrates
            .iter()
            .position(|s| s.kind == SubstrateKind::Npu)
            .unwrap();
        let gpu_idx = node
            .inventory
            .substrates
            .iter()
            .position(|s| s.kind == SubstrateKind::Gpu)
            .unwrap();
        assert!(
            node.topology.best_link(npu_idx, gpu_idx).is_some(),
            "NPU→GPU link should exist in topology"
        );
    }

    #[test]
    fn node_pipeline_npu_to_gpu_to_cpu() {
        let inv = test_inventory();
        let mut node = NodeAtomic::with_inventory("eastgate", inv);
        node.tower = healthy_tower("eastgate");
        node.compute = PrimalHealth::Healthy;

        let pipeline = Pipeline::new("npu_gpu_cpu_pipeline")
            .stage(Stage::new(
                "npu_classify",
                crate::dispatch::Workload::new(
                    "int8 classify",
                    vec![Capability::QuantizedInference { bits: 8 }],
                ),
                256,
            ))
            .stage(Stage::new(
                "gpu_refine",
                crate::dispatch::Workload::new(
                    "f64 spectral",
                    vec![Capability::F64Compute, Capability::ShaderDispatch],
                ),
                65536,
            ))
            .stage(Stage::new(
                "cpu_store",
                crate::dispatch::Workload::new("provenance", vec![Capability::F64Compute]),
                1024,
            ));

        let resolved = node.plan_pipeline(&pipeline);
        assert!(resolved.all_assigned(), "all 3 stages should be assigned");

        if let Some(npu_sub) = resolved.stages[0].substrate {
            assert_eq!(npu_sub.kind, SubstrateKind::Npu, "stage 0 → NPU");
        }
        if let Some(gpu_sub) = resolved.stages[1].substrate {
            assert_eq!(gpu_sub.kind, SubstrateKind::Gpu, "stage 1 → GPU");
        }
    }

    #[test]
    fn nucleus_sovereign_degradation_chain() {
        let mut nucleus = FullNucleus {
            node: NodeAtomic::with_inventory("eastgate", test_inventory()),
            storage: PrimalHealth::Unavailable,
            inference: PrimalHealth::Unavailable,
        };
        nucleus.node.tower = TowerAtomic::new("eastgate");
        nucleus.node.compute = PrimalHealth::Unavailable;

        assert_eq!(nucleus.degradation_level(), "Sovereign (local only)");

        nucleus
            .node
            .tower
            .set_provider_health("ipc", PrimalHealth::Healthy);
        nucleus
            .node
            .tower
            .set_provider_health("crypto", PrimalHealth::Healthy);
        nucleus
            .node
            .tower
            .set_provider_health("discovery", PrimalHealth::Healthy);
        assert_eq!(nucleus.degradation_level(), "Tower only (no compute)");

        nucleus.node.compute = PrimalHealth::Healthy;
        assert_eq!(nucleus.degradation_level(), "Node only (no storage)");

        nucleus.storage = PrimalHealth::Healthy;
        assert_eq!(nucleus.degradation_level(), "Node + Nest (no AI)");

        nucleus.inference = PrimalHealth::Healthy;
        assert_eq!(nucleus.degradation_level(), "Full NUCLEUS");
        assert!(nucleus.is_fully_healthy());
    }

    #[test]
    fn full_nucleus_pipeline_orchestration() {
        let mut nucleus = FullNucleus {
            node: NodeAtomic::with_inventory("eastgate", test_inventory()),
            storage: PrimalHealth::Healthy,
            inference: PrimalHealth::Healthy,
        };
        nucleus.node.tower = healthy_tower("eastgate");
        nucleus.node.compute = PrimalHealth::Healthy;

        let caps = nucleus.capabilities();
        assert!(caps.contains(&AtomicCapability::PipelineOrchestration));
        assert!(caps.contains(&AtomicCapability::NpuInference));
        assert!(caps.contains(&AtomicCapability::ComputeDispatch));
        assert!(caps.contains(&AtomicCapability::DataStorage));
    }
}
