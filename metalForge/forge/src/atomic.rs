// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! NUCLEUS atomic types — Tower, Node, and Nest compositions.
//!
//! In the ecoPrimals ecosystem, primals are composed into **atomics**:
//!
//! | Atomic | Composition | Capability |
//! |--------|-------------|------------|
//! | **Tower** | `BearDog` + `Songbird` | Encrypted IPC foundation |
//! | **Node** | Tower + `ToadStool` | + GPU compute dispatch |
//! | **Nest** | Tower + `NestGate` | + Data storage & provenance |
//! | **Full NUCLEUS** | All primals + Squirrel | Complete ecosystem |
//!
//! Each atomic declares the capabilities it provides and the primals it
//! requires. groundSpring discovers atomics at runtime via biomeOS
//! `topology.metrics` and routes work to the appropriate composition.
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

use crate::inventory::Inventory;
use crate::pipeline::{Pipeline, ResolvedPipeline};
use crate::topology::Topology;

/// Primal health status within an atomic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimalHealth {
    /// Primal is responding to health checks.
    Healthy,
    /// Primal is present but degraded (slow, partial capability).
    Degraded,
    /// Primal is not responding.
    Unavailable,
    /// Primal is not required for this atomic type.
    NotRequired,
}

/// Capabilities provided by a NUCLEUS atomic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AtomicCapability {
    /// Encrypted inter-primal communication (`BearDog` + `Songbird`).
    SecureIpc,
    /// GPU/CPU compute dispatch via `ToadStool`/barracuda.
    ComputeDispatch,
    /// NPU inference (int8 quantized, via akida-driver).
    NpuInference,
    /// Data storage and provenance (`NestGate`).
    DataStorage,
    /// Live data pipelines (NCBI, NOAA, IRIS via `NestGate`).
    LiveData,
    /// AI/ML inference (Squirrel).
    AiInference,
    /// Cross-substrate pipeline orchestration (metalForge).
    PipelineOrchestration,
}

/// Tower Atomic — `BearDog` + `Songbird` for encrypted IPC.
///
/// The foundational atomic that all others build upon. Provides
/// secure inter-primal communication via `BearDog` encryption and
/// `Songbird` socket mesh.
#[derive(Debug)]
pub struct TowerAtomic {
    /// Node identifier (e.g. "eastgate", "biomegate").
    pub node_id: String,
    /// `BearDog` primal health.
    pub beardog: PrimalHealth,
    /// `Songbird` primal health.
    pub songbird: PrimalHealth,
    /// biomeOS Neural API socket path (discovered at runtime).
    pub socket_path: Option<String>,
}

/// Node Atomic — Tower + `ToadStool` for GPU compute.
///
/// Extends Tower with `ToadStool` compute capabilities. Hosts the
/// metalForge substrate inventory for hardware-aware dispatch.
#[derive(Debug)]
pub struct NodeAtomic {
    /// Tower foundation.
    pub tower: TowerAtomic,
    /// `ToadStool` primal health.
    pub toadstool: PrimalHealth,
    /// Local hardware inventory (GPUs, NPUs, CPU).
    pub inventory: Inventory,
    /// Device topology for transfer cost modeling.
    pub topology: Topology,
}

/// Nest Atomic — Tower + `NestGate` for data storage.
///
/// Extends Tower with `NestGate` data capabilities. Provides storage,
/// provenance, and live data pipeline access (NCBI, NOAA, IRIS).
#[derive(Debug)]
pub struct NestAtomic {
    /// Tower foundation.
    pub tower: TowerAtomic,
    /// `NestGate` primal health.
    pub nestgate: PrimalHealth,
    /// Available data capabilities.
    pub data_capabilities: Vec<AtomicCapability>,
}

/// Full NUCLEUS — all primals for complete ecosystem.
#[derive(Debug)]
pub struct FullNucleus {
    /// Node atomic (Tower + `ToadStool`).
    pub node: NodeAtomic,
    /// `NestGate` primal health.
    pub nestgate: PrimalHealth,
    /// Squirrel primal health.
    pub squirrel: PrimalHealth,
}

impl TowerAtomic {
    /// Create a Tower atomic for a given node.
    pub fn new(node_id: impl Into<String>) -> Self {
        Self {
            node_id: node_id.into(),
            beardog: PrimalHealth::Unavailable,
            songbird: PrimalHealth::Unavailable,
            socket_path: None,
        }
    }

    /// Check if the Tower is healthy (both primals responding).
    #[must_use]
    pub const fn is_healthy(&self) -> bool {
        matches!(self.beardog, PrimalHealth::Healthy)
            && matches!(self.songbird, PrimalHealth::Healthy)
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
            toadstool: PrimalHealth::Unavailable,
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
            toadstool: PrimalHealth::Unavailable,
            inventory,
            topology,
        }
    }

    /// Check if compute dispatch is available.
    #[must_use]
    pub const fn can_compute(&self) -> bool {
        matches!(self.toadstool, PrimalHealth::Healthy)
            || matches!(self.toadstool, PrimalHealth::Degraded)
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
            nestgate: PrimalHealth::Unavailable,
            data_capabilities: Vec::new(),
        }
    }

    /// Check if data storage is available.
    #[must_use]
    pub const fn can_store(&self) -> bool {
        matches!(self.nestgate, PrimalHealth::Healthy)
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
    /// Check if all primals are healthy.
    #[must_use]
    pub const fn is_fully_healthy(&self) -> bool {
        self.node.tower.is_healthy()
            && self.node.can_compute()
            && matches!(self.nestgate, PrimalHealth::Healthy)
            && matches!(self.squirrel, PrimalHealth::Healthy)
    }

    /// List all capabilities of the full NUCLEUS.
    #[must_use]
    pub fn capabilities(&self) -> Vec<AtomicCapability> {
        let mut caps = self.node.capabilities();
        if matches!(self.nestgate, PrimalHealth::Healthy) {
            caps.push(AtomicCapability::DataStorage);
            caps.push(AtomicCapability::LiveData);
        }
        if matches!(self.squirrel, PrimalHealth::Healthy) {
            caps.push(AtomicCapability::AiInference);
        }
        caps
    }

    /// The sovereign degradation level — what's available when parts fail.
    #[must_use]
    pub const fn degradation_level(&self) -> &'static str {
        if self.is_fully_healthy() {
            "Full NUCLEUS"
        } else if self.node.can_compute() && matches!(self.nestgate, PrimalHealth::Healthy) {
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
mod tests {
    use super::*;
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

    #[test]
    fn tower_unhealthy_by_default() {
        let tower = TowerAtomic::new("eastgate");
        assert!(!tower.is_healthy());
        assert!(tower.capabilities().is_empty());
    }

    #[test]
    fn tower_healthy_when_primals_respond() {
        let mut tower = TowerAtomic::new("eastgate");
        tower.beardog = PrimalHealth::Healthy;
        tower.songbird = PrimalHealth::Healthy;
        assert!(tower.is_healthy());
        assert!(tower.capabilities().contains(&AtomicCapability::SecureIpc));
    }

    #[test]
    fn node_has_compute_when_toadstool_healthy() {
        let mut node = NodeAtomic::with_inventory("eastgate", test_inventory());
        node.toadstool = PrimalHealth::Healthy;
        assert!(node.can_compute());
        assert!(node
            .capabilities()
            .contains(&AtomicCapability::ComputeDispatch));
    }

    #[test]
    fn node_has_npu_inference_when_discovered() {
        let node = NodeAtomic::with_inventory("biomegate", test_inventory());
        assert!(node
            .capabilities()
            .contains(&AtomicCapability::NpuInference));
    }

    #[test]
    fn node_always_has_pipeline_orchestration() {
        let node = NodeAtomic::with_inventory("eastgate", test_inventory());
        assert!(node
            .capabilities()
            .contains(&AtomicCapability::PipelineOrchestration));
    }

    #[test]
    fn nest_can_store_when_nestgate_healthy() {
        let mut nest = NestAtomic::new("westgate");
        nest.nestgate = PrimalHealth::Healthy;
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
            nestgate: PrimalHealth::Unavailable,
            squirrel: PrimalHealth::Unavailable,
        };

        assert_eq!(nucleus.degradation_level(), "Sovereign (local only)");

        nucleus.node.tower.beardog = PrimalHealth::Healthy;
        nucleus.node.tower.songbird = PrimalHealth::Healthy;
        assert_eq!(nucleus.degradation_level(), "Tower only (no compute)");

        nucleus.node.toadstool = PrimalHealth::Healthy;
        assert_eq!(nucleus.degradation_level(), "Node only (no storage)");

        nucleus.nestgate = PrimalHealth::Healthy;
        assert_eq!(nucleus.degradation_level(), "Node + Nest (no AI)");

        nucleus.squirrel = PrimalHealth::Healthy;
        assert_eq!(nucleus.degradation_level(), "Full NUCLEUS");
        assert!(nucleus.is_fully_healthy());
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
            nestgate: PrimalHealth::Healthy,
            squirrel: PrimalHealth::Healthy,
        };
        nucleus.node.tower.beardog = PrimalHealth::Healthy;
        nucleus.node.tower.songbird = PrimalHealth::Healthy;
        nucleus.node.toadstool = PrimalHealth::Healthy;

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
}
