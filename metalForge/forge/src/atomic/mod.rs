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

mod types;

mod nest;
mod node;
mod nucleus;
mod tower;

pub use types::*;

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use crate::inventory::Inventory;
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
