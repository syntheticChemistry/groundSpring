// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

use crate::inventory::Inventory;
use crate::pipeline::{Pipeline, ResolvedPipeline};
use crate::topology::Topology;

use super::types::{AtomicCapability, NodeAtomic, PrimalHealth, TowerAtomic};

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
