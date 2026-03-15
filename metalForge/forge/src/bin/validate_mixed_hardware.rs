// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Validation binary for mixed-hardware pipeline dispatch.
//!
//! Validates `PCIe` topology inference, pipeline planning, fallback chains,
//! NUCLEUS atomic composition, and cross-substrate pipeline orchestration.
//!
//! Provenance: all checks use synthetic substrate inventories (no live
//! hardware required). Mixed-hardware compute parity is validated in
//! `validate-metalforge-cross-substrate`.

use groundspring_forge::atomic::{
    AtomicCapability, FullNucleus, NestAtomic, NodeAtomic, PrimalHealth, TowerAtomic,
};
use groundspring_forge::dispatch::{self, Workload};
use groundspring_forge::harness::Harness;
use groundspring_forge::inventory::Inventory;
use groundspring_forge::pipeline::{FallbackPolicy, Pipeline, Stage};
use groundspring_forge::substrate::{
    Capability, GpuArch, Identity, Properties, Substrate, SubstrateKind,
};
use groundspring_forge::tolerance::ToleranceTier;
use groundspring_forge::topology::{BandwidthTier, Topology};

fn titan_v() -> Substrate {
    Substrate {
        kind: SubstrateKind::Gpu,
        identity: Identity::named("NVIDIA TITAN V"),
        properties: Properties {
            gpu_arch: Some(GpuArch::Volta),
            memory_bytes: Some(12 * 1024 * 1024 * 1024),
            has_f64: true,
            ..Properties::default()
        },
        capabilities: vec![
            Capability::F64Compute,
            Capability::F32Compute,
            Capability::ShaderDispatch,
            Capability::ScalarReduce,
            Capability::NativeF64,
        ],
    }
}

fn rtx_4070() -> Substrate {
    Substrate {
        kind: SubstrateKind::Gpu,
        identity: Identity::named("NVIDIA GeForce RTX 4070"),
        properties: Properties {
            gpu_arch: Some(GpuArch::Ada),
            memory_bytes: Some(12 * 1024 * 1024 * 1024),
            has_f64: true,
            ..Properties::default()
        },
        capabilities: vec![
            Capability::F64Compute,
            Capability::F32Compute,
            Capability::ShaderDispatch,
            Capability::ScalarReduce,
            Capability::TimestampQuery,
        ],
    }
}

fn akd1000() -> Substrate {
    Substrate {
        kind: SubstrateKind::Npu,
        identity: Identity::named("BrainChip AKD1000"),
        properties: Properties::default(),
        capabilities: vec![
            Capability::F32Compute,
            Capability::QuantizedInference { bits: 8 },
            Capability::QuantizedInference { bits: 4 },
            Capability::BatchInference { max_batch: 8 },
            Capability::WeightMutation,
        ],
    }
}

fn test_cpu() -> Substrate {
    Substrate {
        kind: SubstrateKind::Cpu,
        identity: Identity::named("AMD Ryzen 9 7950X"),
        properties: Properties {
            memory_bytes: Some(64 * 1024 * 1024 * 1024),
            core_count: Some(16),
            thread_count: Some(32),
            ..Properties::default()
        },
        capabilities: vec![
            Capability::F64Compute,
            Capability::F32Compute,
            Capability::SimdVector,
        ],
    }
}

fn biomegate_inventory() -> Vec<Substrate> {
    vec![titan_v(), rtx_4070(), akd1000(), test_cpu()]
}

fn validate_topology(h: &mut Harness) {
    println!("--- Section A: PCIe Topology ---\n");

    let subs = biomegate_inventory();
    let topo = Topology::infer(&subs);

    h.check(
        "topology infers links for all pairs",
        topo.links.len() == 12,
    );

    h.check(
        "GPU→NPU link is PCIe-low (AKD1000)",
        topo.best_link(0, 2)
            .is_some_and(|l| l.tier == BandwidthTier::PcieLow),
    );
    h.check(
        "GPU→GPU link is PCIe-peer",
        topo.best_link(0, 1)
            .is_some_and(|l| l.tier == BandwidthTier::PciePeer),
    );
    h.check(
        "CPU→GPU link is PCIe-host",
        topo.best_link(3, 0)
            .is_some_and(|l| l.tier == BandwidthTier::PcieHost),
    );

    h.check(
        "P2P pairs include GPU↔GPU",
        topo.has_p2p(0, 1) && topo.has_p2p(1, 0),
    );
    h.check("No P2P via CPU", !topo.has_p2p(3, 0) && !topo.has_p2p(0, 3));

    let small_transfer = BandwidthTier::PcieLow.transfer_time_us(1024);
    let large_transfer = BandwidthTier::PcieLow.transfer_time_us(100 * 1024 * 1024);
    h.check(
        "transfer time scales with data size",
        large_transfer > small_transfer,
    );
}

fn validate_fallback_chains(h: &mut Harness) {
    println!("\n--- Section B: Fallback Chains ---\n");

    let subs = biomegate_inventory();

    let work_f64 = Workload::new(
        "f64 compute",
        vec![Capability::F64Compute, Capability::ShaderDispatch],
    );
    let chain = dispatch::fallback_chain(&work_f64, &subs);
    h.check("f64 fallback chain non-empty", !chain.is_empty());
    h.check(
        "f64 chain starts with NativeF64 GPU (Titan V)",
        chain[0].substrate.identity.name.contains("TITAN V"),
    );
    h.check(
        "f64 chain includes RTX 4070 as fallback",
        chain
            .iter()
            .any(|d| d.substrate.identity.name.contains("RTX 4070")),
    );

    let work_npu = Workload::new(
        "int8 classify",
        vec![Capability::QuantizedInference { bits: 8 }],
    );
    let npu_chain = dispatch::fallback_chain(&work_npu, &subs);
    h.check(
        "NPU chain routes to AKD1000",
        npu_chain
            .first()
            .is_some_and(|d| d.substrate.kind == SubstrateKind::Npu),
    );

    let work_impossible = Workload::new(
        "impossible",
        vec![Capability::QuantizedInference { bits: 2 }],
    );
    h.check(
        "impossible workload has empty fallback chain",
        dispatch::fallback_chain(&work_impossible, &subs).is_empty(),
    );
}

fn validate_pipeline_planning(h: &mut Harness) {
    println!("\n--- Section C: Pipeline Planning ---\n");

    let subs = biomegate_inventory();
    let topo = Topology::infer(&subs);

    let pipeline = Pipeline::new("NPU classify → GPU refine → CPU store")
        .stage(Stage::new(
            "NPU regime classify",
            Workload::new("classify", vec![Capability::QuantizedInference { bits: 8 }]),
            256,
        ))
        .stage(Stage::new(
            "GPU Lyapunov sweep",
            Workload::new(
                "lyapunov",
                vec![Capability::F64Compute, Capability::ShaderDispatch],
            ),
            80_000,
        ))
        .stage(Stage::new(
            "CPU provenance store",
            Workload::new("store", vec![Capability::F64Compute]).prefer(SubstrateKind::Cpu),
            1024,
        ));

    let resolved = pipeline.plan(&subs, &topo);
    h.check("3-stage pipeline all assigned", resolved.all_assigned());
    h.check("pipeline has 3 stages", resolved.stages.len() == 3);
    h.check(
        "stage 0 → NPU",
        resolved.stages[0]
            .substrate
            .is_some_and(|s| s.kind == SubstrateKind::Npu),
    );
    h.check(
        "stage 1 → GPU (Titan V for f64)",
        resolved.stages[1]
            .substrate
            .is_some_and(|s| s.identity.name.contains("TITAN V")),
    );
    h.check(
        "stage 2 → CPU (provenance)",
        resolved.stages[2]
            .substrate
            .is_some_and(|s| s.kind == SubstrateKind::Cpu),
    );
    h.check("pipeline is fully optimal", resolved.fully_optimal);

    let cpu_only = vec![test_cpu()];
    let cpu_topo = Topology::infer(&cpu_only);

    let degrade_pipeline = Pipeline::new("degrade test")
        .stage(
            Stage::new(
                "NPU only",
                Workload::new("npu_work", vec![Capability::QuantizedInference { bits: 8 }]),
                256,
            )
            .with_fallback(FallbackPolicy::Degrade),
        )
        .stage(Stage::new(
            "CPU work",
            Workload::new("cpu_work", vec![Capability::F64Compute]),
            1024,
        ));
    let degraded = degrade_pipeline.plan(&cpu_only, &cpu_topo);
    h.check("degraded pipeline still assigned", degraded.all_assigned());
    h.check("degraded count is 1", degraded.degraded_count() == 1);
    h.check(
        "degraded pipeline not fully optimal",
        !degraded.fully_optimal,
    );

    let skip_pipeline = Pipeline::new("skip test").stage(
        Stage::new(
            "GPU only",
            Workload::new(
                "gpu_only",
                vec![Capability::F64Compute, Capability::ShaderDispatch],
            ),
            4096,
        )
        .with_fallback(FallbackPolicy::Skip),
    );
    let skipped = skip_pipeline.plan(&cpu_only, &cpu_topo);
    h.check("skipped stage not assigned", !skipped.all_assigned());
}

fn validate_atomics(h: &mut Harness) {
    println!("\n--- Section D: NUCLEUS Atomics ---\n");

    let tower = TowerAtomic::new("eastgate");
    h.check("tower unhealthy by default", !tower.is_healthy());
    h.check(
        "tower has no capabilities when unhealthy",
        tower.capabilities().is_empty(),
    );

    let mut healthy_tower = TowerAtomic::new("eastgate");
    healthy_tower.set_provider_health("crypto", PrimalHealth::Healthy);
    healthy_tower.set_provider_health("discovery", PrimalHealth::Healthy);
    h.check(
        "tower healthy with both primals",
        healthy_tower.is_healthy(),
    );
    h.check(
        "tower provides SecureIpc",
        healthy_tower
            .capabilities()
            .contains(&AtomicCapability::SecureIpc),
    );

    let inv = Inventory {
        substrates: biomegate_inventory(),
    };
    let mut node = NodeAtomic::with_inventory("biomegate", inv);
    node.tower
        .set_provider_health("crypto", PrimalHealth::Healthy);
    node.tower
        .set_provider_health("discovery", PrimalHealth::Healthy);
    node.compute = PrimalHealth::Healthy;

    h.check("node can compute", node.can_compute());
    h.check(
        "node has ComputeDispatch",
        node.capabilities()
            .contains(&AtomicCapability::ComputeDispatch),
    );
    h.check(
        "node has NpuInference (AKD1000 discovered)",
        node.capabilities()
            .contains(&AtomicCapability::NpuInference),
    );
    h.check(
        "node has PipelineOrchestration",
        node.capabilities()
            .contains(&AtomicCapability::PipelineOrchestration),
    );

    let mut nest = NestAtomic::new("westgate");
    nest.storage = PrimalHealth::Healthy;
    nest.data_capabilities.push(AtomicCapability::LiveData);
    h.check("nest can store", nest.can_store());
    h.check(
        "nest has DataStorage",
        nest.capabilities().contains(&AtomicCapability::DataStorage),
    );
}

fn validate_degradation(h: &mut Harness) {
    println!("\n--- Section E: NUCLEUS Degradation ---\n");

    let inv = Inventory {
        substrates: biomegate_inventory(),
    };
    let mut nucleus = FullNucleus {
        node: NodeAtomic::with_inventory("strandgate", inv),
        storage: PrimalHealth::Unavailable,
        inference: PrimalHealth::Unavailable,
    };

    h.check(
        "degradation: sovereign when nothing healthy",
        nucleus.degradation_level() == "Sovereign (local only)",
    );

    nucleus
        .node
        .tower
        .set_provider_health("crypto", PrimalHealth::Healthy);
    nucleus
        .node
        .tower
        .set_provider_health("discovery", PrimalHealth::Healthy);
    h.check(
        "degradation: tower when only IPC",
        nucleus.degradation_level() == "Tower only (no compute)",
    );

    nucleus.node.compute = PrimalHealth::Healthy;
    h.check(
        "degradation: node when compute available",
        nucleus.degradation_level() == "Node only (no storage)",
    );

    nucleus.storage = PrimalHealth::Healthy;
    h.check(
        "degradation: node+nest without AI",
        nucleus.degradation_level() == "Node + Nest (no AI)",
    );

    nucleus.inference = PrimalHealth::Healthy;
    h.check(
        "degradation: full NUCLEUS",
        nucleus.degradation_level() == "Full NUCLEUS",
    );
    h.check("full nucleus is_fully_healthy", nucleus.is_fully_healthy());

    let caps = nucleus.capabilities();
    h.check(
        "full nucleus has all key capabilities",
        caps.contains(&AtomicCapability::SecureIpc)
            && caps.contains(&AtomicCapability::ComputeDispatch)
            && caps.contains(&AtomicCapability::DataStorage)
            && caps.contains(&AtomicCapability::AiInference),
    );
}

fn validate_tolerances(h: &mut Harness) {
    println!("\n--- Section F: Tolerance Tiers ---\n");

    h.check(
        "exact tolerance < analytical",
        ToleranceTier::Exact.relative_tolerance() < ToleranceTier::Analytical.relative_tolerance(),
    );
    h.check(
        "statistical < quantized",
        ToleranceTier::Statistical.relative_tolerance()
            < ToleranceTier::Quantized.relative_tolerance(),
    );
    h.check(
        "30 workload tolerances defined",
        groundspring_forge::tolerance::all_tolerances().len() == 30,
    );
}

fn validate_gpu_npu_bypass(h: &mut Harness) {
    println!("\n--- Section G: GPU→NPU PCIe Bypass ---\n");

    let subs = biomegate_inventory();
    let topo = Topology::infer(&subs);

    let pipeline = Pipeline::new("GPU→NPU→CPU (PCIe bypass)")
        .stage(Stage::new(
            "GPU Anderson 4D lattice",
            Workload::new(
                "anderson_4d",
                vec![Capability::F64Compute, Capability::ShaderDispatch],
            ),
            500_000,
        ))
        .stage(Stage::new(
            "NPU regime classify (int8 DMA)",
            Workload::new(
                "regime_classify",
                vec![Capability::QuantizedInference { bits: 8 }],
            ),
            256,
        ))
        .stage(Stage::new(
            "CPU provenance store",
            Workload::new("store", vec![Capability::F64Compute]).prefer(SubstrateKind::Cpu),
            1024,
        ));

    let resolved = pipeline.plan(&subs, &topo);
    h.check("GPU→NPU→CPU pipeline all assigned", resolved.all_assigned());
    h.check(
        "stage 0 → GPU (Titan V for f64 Anderson)",
        resolved.stages[0]
            .substrate
            .is_some_and(|s| s.identity.name.contains("TITAN V")),
    );
    h.check(
        "stage 1 → NPU (AKD1000 for int8)",
        resolved.stages[1]
            .substrate
            .is_some_and(|s| s.kind == SubstrateKind::Npu),
    );
    h.check(
        "stage 2 → CPU (provenance)",
        resolved.stages[2]
            .substrate
            .is_some_and(|s| s.kind == SubstrateKind::Cpu),
    );

    let gpu_idx = 0_usize;
    let npu_idx = 2_usize;
    let gpu_npu_link = topo.best_link(gpu_idx, npu_idx);
    h.check(
        "GPU→NPU PCIe link exists (bypass CPU)",
        gpu_npu_link.is_some(),
    );
    h.check(
        "GPU→NPU is PCIe-low (direct DMA)",
        gpu_npu_link.is_some_and(|l| l.tier == BandwidthTier::PcieLow),
    );

    let gpu_npu_direct = topo.transfer_time_us(gpu_idx, npu_idx, 65536);
    let gpu_cpu = topo.transfer_time_us(gpu_idx, 3, 65536);
    let cpu_npu = topo.transfer_time_us(3, npu_idx, 65536);
    let gpu_cpu_npu_roundtrip = gpu_cpu + cpu_npu;
    h.check(
        "GPU→NPU direct ≤ GPU→CPU→NPU round-trip (bypass avoids CPU hop)",
        gpu_npu_direct <= gpu_cpu_npu_roundtrip,
    );

    let reverse_pipeline = Pipeline::new("NPU→GPU→CPU (reverse)")
        .stage(Stage::new(
            "NPU pre-classify",
            Workload::new(
                "pre_classify",
                vec![Capability::QuantizedInference { bits: 8 }],
            ),
            256,
        ))
        .stage(Stage::new(
            "GPU spectral refine",
            Workload::new(
                "spectral",
                vec![Capability::F64Compute, Capability::ShaderDispatch],
            ),
            200_000,
        ))
        .stage(Stage::new(
            "CPU aggregate",
            Workload::new("aggregate", vec![Capability::F64Compute]).prefer(SubstrateKind::Cpu),
            4096,
        ));

    let rev_resolved = reverse_pipeline.plan(&subs, &topo);
    h.check(
        "NPU→GPU→CPU pipeline all assigned",
        rev_resolved.all_assigned(),
    );
    h.check("reverse pipeline fully optimal", rev_resolved.fully_optimal);
}

fn validate_nucleus_coordination(h: &mut Harness) {
    println!("\n--- Section H: NUCLEUS Atomic Coordination ---\n");

    let inv = Inventory {
        substrates: biomegate_inventory(),
    };
    let mut node = NodeAtomic::with_inventory("biomegate", inv);
    node.tower
        .set_provider_health("crypto", PrimalHealth::Healthy);
    node.tower
        .set_provider_health("discovery", PrimalHealth::Healthy);
    node.compute = PrimalHealth::Healthy;

    let mut nest = NestAtomic::new("datagate");
    nest.storage = PrimalHealth::Healthy;
    nest.data_capabilities.push(AtomicCapability::LiveData);

    let mut nucleus = FullNucleus {
        node,
        storage: PrimalHealth::Healthy,
        inference: PrimalHealth::Healthy,
    };

    h.check(
        "NUCLEUS fully healthy for mixed dispatch",
        nucleus.is_fully_healthy(),
    );

    let caps = nucleus.capabilities();
    h.check(
        "NUCLEUS has compute + storage + inference",
        caps.contains(&AtomicCapability::ComputeDispatch)
            && caps.contains(&AtomicCapability::DataStorage)
            && caps.contains(&AtomicCapability::AiInference),
    );

    h.check(
        "NUCLEUS has NPU inference via node inventory",
        caps.contains(&AtomicCapability::NpuInference),
    );

    h.check(
        "NUCLEUS has pipeline orchestration",
        caps.contains(&AtomicCapability::PipelineOrchestration),
    );

    nucleus.inference = PrimalHealth::Unavailable;
    h.check(
        "degraded NUCLEUS still dispatches compute",
        nucleus.node.can_compute(),
    );
    h.check(
        "degraded NUCLEUS level is Node + Nest (no AI)",
        nucleus.degradation_level() == "Node + Nest (no AI)",
    );
}

fn main() {
    let mut h = Harness::new();

    println!("========================================================================");
    println!("Mixed Hardware Pipeline Validation");
    println!("========================================================================\n");
    println!("Provenance: synthetic substrate inventories — no live hardware required.");
    println!("  Topology, fallback chains, and pipeline planning from constructed test data.");
    println!("  Cross-substrate parity validated in validate-metalforge-cross-substrate.\n");

    validate_topology(&mut h);
    validate_fallback_chains(&mut h);
    validate_pipeline_planning(&mut h);
    validate_atomics(&mut h);
    validate_degradation(&mut h);
    validate_tolerances(&mut h);
    validate_gpu_npu_bypass(&mut h);
    validate_nucleus_coordination(&mut h);

    println!("\n========================================================================");
    h.finish();
}
