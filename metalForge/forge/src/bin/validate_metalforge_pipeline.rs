// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! metalForge Mixed-Hardware Pipeline Validation.
//!
//! Validates the full 3-stage pipeline (NPU classify → GPU refine → CPU store)
//! with live hardware probing, `PCIe` P2P transfer strategy selection, and
//! fallback policy validation.
//!
//! Usage:
//!     cargo run --release --bin validate-metalforge-pipeline

use groundspring_forge::atomic::{AtomicCapability, NodeAtomic};
use groundspring_forge::dispatch::Workload;
use groundspring_forge::harness::Harness;
use groundspring_forge::inventory::Inventory;
use groundspring_forge::pipeline::{
    FallbackPolicy, Pipeline, Stage, StageResolution, TransferStrategy,
};
use groundspring_forge::substrate::{Capability, Identity, Properties, Substrate, SubstrateKind};
use groundspring_forge::topology::Topology;
use std::time::Instant;

fn main() {
    println!("================================================================");
    println!("metalForge Mixed-Hardware Pipeline Validation");
    println!("================================================================\n");

    let mut h = Harness::new();

    validate_anderson_regime_pipeline(&mut h);
    validate_npu_gpu_p2p_transfer(&mut h);
    validate_full_three_stage(&mut h);
    validate_fallback_degrade(&mut h);
    validate_fallback_skip(&mut h);
    validate_fallback_fail(&mut h);
    validate_gpu_only_pipeline(&mut h);
    validate_cpu_only_pipeline(&mut h);
    validate_pipeline_timing(&mut h);
    validate_node_atomic_pipeline(&mut h);

    println!("\n--- Summary ---\n");
    println!("  Each test validates pipeline planning, substrate assignment,");
    println!("  transfer strategy selection, and fallback behavior.");
    println!("  P2P = PCIe peer-to-peer (NPU↔GPU, bypassing CPU roundtrip).\n");

    h.finish();
}

fn test_gpu() -> Substrate {
    Substrate {
        kind: SubstrateKind::Gpu,
        identity: Identity::named("Titan V (f64)"),
        properties: Properties::default(),
        capabilities: vec![
            Capability::F64Compute,
            Capability::F32Compute,
            Capability::ShaderDispatch,
            Capability::ScalarReduce,
            Capability::NativeF64,
        ],
    }
}

fn test_npu() -> Substrate {
    Substrate {
        kind: SubstrateKind::Npu,
        identity: Identity::named("AKD1000"),
        properties: Properties::default(),
        capabilities: vec![
            Capability::QuantizedInference { bits: 8 },
            Capability::BatchInference { max_batch: 8 },
        ],
    }
}

fn test_cpu() -> Substrate {
    Substrate {
        kind: SubstrateKind::Cpu,
        identity: Identity::named("Host CPU"),
        properties: Properties::default(),
        capabilities: vec![Capability::F64Compute, Capability::F32Compute],
    }
}

fn anderson_classify_workload() -> Workload {
    Workload::new(
        "Anderson regime classify (int8)",
        vec![Capability::QuantizedInference { bits: 8 }],
    )
}

fn anderson_refine_workload() -> Workload {
    Workload::new(
        "Lyapunov f64 refinement",
        vec![Capability::F64Compute, Capability::ShaderDispatch],
    )
}

fn provenance_store_workload() -> Workload {
    Workload::new("Provenance store", vec![Capability::F64Compute]).prefer(SubstrateKind::Cpu)
}

fn validate_anderson_regime_pipeline(h: &mut Harness) {
    println!("\n--- Anderson Regime Pipeline (NPU → GPU → CPU) ---\n");

    let subs = vec![test_gpu(), test_npu(), test_cpu()];
    let topo = Topology::infer(&subs);

    let pipeline = Pipeline::new("Anderson regime classification")
        .stage(Stage::new(
            "NPU int8 classify",
            anderson_classify_workload(),
            256,
        ))
        .stage(Stage::new(
            "GPU f64 Lyapunov refinement",
            anderson_refine_workload(),
            8192,
        ))
        .stage(Stage::new(
            "CPU provenance store",
            provenance_store_workload(),
            1024,
        ));

    let resolved = pipeline.plan(&subs, &topo);
    resolved.print_summary();

    h.check("Pipeline all assigned", resolved.all_assigned());
    h.check("Pipeline 3 stages", resolved.stages.len() == 3);

    h.check(
        "Stage 0 → NPU",
        resolved.stages[0]
            .substrate
            .is_some_and(|s| s.kind == SubstrateKind::Npu),
    );
    h.check(
        "Stage 1 → GPU",
        resolved.stages[1]
            .substrate
            .is_some_and(|s| s.kind == SubstrateKind::Gpu),
    );
    h.check(
        "Stage 2 → CPU",
        resolved.stages[2]
            .substrate
            .is_some_and(|s| s.kind == SubstrateKind::Cpu),
    );
}

fn validate_npu_gpu_p2p_transfer(h: &mut Harness) {
    println!("\n--- NPU → GPU PCIe Transfer Strategy ---\n");

    let subs = vec![test_npu(), test_gpu(), test_cpu()];
    let topo = Topology::infer(&subs);

    let pipeline = Pipeline::new("P2P transfer test")
        .stage(Stage::new(
            "NPU classify",
            anderson_classify_workload(),
            256,
        ))
        .stage(Stage::new("GPU refine", anderson_refine_workload(), 8192));

    let resolved = pipeline.plan(&subs, &topo);

    let npu_gpu_link = topo.best_link(0, 1);
    let has_p2p = npu_gpu_link.is_some_and(|l| l.tier.is_peer_to_peer());

    if has_p2p {
        h.check(
            "NPU→GPU P2P detected",
            resolved.stages[1].transfer == TransferStrategy::PeerToPeer,
        );
        println!("  Transfer: PCIe P2P (bypassing CPU host memory)");
    } else {
        h.check(
            "NPU→GPU host bounce (AKD1000 PcieLow)",
            resolved.stages[1].transfer == TransferStrategy::HostBounce,
        );
        println!("  Transfer: Host bounce (AKD1000 is PCIe 2.0 x1)");
    }

    let tier_label = npu_gpu_link.map_or("(no link)", |l| l.tier.label());
    println!("  Link tier: {tier_label}");
    println!(
        "  Estimated transfer cost: {}µs",
        resolved.stages[1].transfer_cost_us
    );
    println!(
        "  Total pipeline overhead: {}µs",
        resolved.total_transfer_us
    );
}

fn validate_full_three_stage(h: &mut Harness) {
    println!("\n--- Full 3-Stage Pipeline Execution ---\n");

    let subs = vec![test_npu(), test_gpu(), test_cpu()];
    let topo = Topology::infer(&subs);

    let pipeline = Pipeline::new("Full Anderson pipeline")
        .stage(Stage::new("classify", anderson_classify_workload(), 256))
        .stage(Stage::new("refine", anderson_refine_workload(), 8192))
        .stage(Stage::new("store", provenance_store_workload(), 1024));

    let t0 = Instant::now();
    let resolved = pipeline.plan(&subs, &topo);
    let plan_us = t0.elapsed().as_micros();

    println!("  Planning time: {plan_us}µs");
    println!("  P2P transfers: {}", resolved.p2p_transfer_count());
    println!("  Degraded stages: {}", resolved.degraded_count());
    println!("  Fully optimal: {}", resolved.fully_optimal);

    h.check("Full pipeline all assigned", resolved.all_assigned());
    h.check("Full pipeline optimal", resolved.fully_optimal);
    h.check("Planning < 1ms", plan_us < 1000);
}

fn validate_fallback_degrade(h: &mut Harness) {
    println!("\n--- Fallback: Degrade (NPU absent → CPU fallback) ---\n");

    let subs = vec![test_gpu(), test_cpu()];
    let topo = Topology::infer(&subs);

    let pipeline = Pipeline::new("Degrade test")
        .stage(
            Stage::new("NPU classify", anderson_classify_workload(), 256)
                .with_fallback(FallbackPolicy::Degrade),
        )
        .stage(Stage::new("GPU refine", anderson_refine_workload(), 8192));

    let resolved = pipeline.plan(&subs, &topo);
    h.check("Degrade pipeline all assigned", resolved.all_assigned());
    h.check(
        "Stage 0 degraded",
        resolved.stages[0].reason == StageResolution::Degraded,
    );
    h.check(
        "Stage 0 fell to CPU",
        resolved.stages[0]
            .substrate
            .is_some_and(|s| s.kind == SubstrateKind::Cpu),
    );
    h.check("Not fully optimal", !resolved.fully_optimal);
    println!("  NPU absent → degraded to CPU. Pipeline continues.");
}

fn validate_fallback_skip(h: &mut Harness) {
    println!("\n--- Fallback: Skip (NPU absent → stage skipped) ---\n");

    let subs = vec![test_gpu(), test_cpu()];
    let topo = Topology::infer(&subs);

    let pipeline = Pipeline::new("Skip test")
        .stage(
            Stage::new("NPU classify", anderson_classify_workload(), 256)
                .with_fallback(FallbackPolicy::Skip),
        )
        .stage(Stage::new("GPU refine", anderson_refine_workload(), 8192));

    let resolved = pipeline.plan(&subs, &topo);
    h.check(
        "Skip: stage 0 skipped",
        resolved.stages[0].substrate.is_none(),
    );
    h.check(
        "Skip: stage 0 reason",
        resolved.stages[0].reason == StageResolution::Skipped,
    );
    h.check(
        "Skip: stage 1 still assigned",
        resolved.stages[1].substrate.is_some(),
    );
    println!("  NPU absent → stage skipped. Remaining stages proceed.");
}

fn validate_fallback_fail(h: &mut Harness) {
    println!("\n--- Fallback: Fail (GPU absent → pipeline fails) ---\n");

    let subs = vec![test_cpu()];
    let topo = Topology::infer(&subs);

    let pipeline = Pipeline::new("Fail test").stage(
        Stage::new("GPU-only work", anderson_refine_workload(), 4096)
            .with_fallback(FallbackPolicy::Fail),
    );

    let resolved = pipeline.plan(&subs, &topo);
    h.check(
        "Fail: stage not assigned",
        resolved.stages[0].substrate.is_none(),
    );
    h.check("Fail: not all assigned", !resolved.all_assigned());
    println!("  GPU absent + Fail policy → pipeline cannot execute.");
}

fn validate_gpu_only_pipeline(h: &mut Harness) {
    println!("\n--- GPU-Only Pipeline (no cross-device transfer) ---\n");

    let subs = vec![test_gpu(), test_cpu()];
    let topo = Topology::infer(&subs);

    let pipeline = Pipeline::new("GPU chain")
        .stage(Stage::new(
            "eigendecompose",
            Workload::new(
                "eigh",
                vec![Capability::F64Compute, Capability::ShaderDispatch],
            ),
            16384,
        ))
        .stage(Stage::new(
            "spectral recon",
            Workload::new(
                "tikhonov",
                vec![Capability::F64Compute, Capability::ShaderDispatch],
            ),
            8192,
        ));

    let resolved = pipeline.plan(&subs, &topo);
    h.check("GPU chain all assigned", resolved.all_assigned());
    h.check("GPU chain fully optimal", resolved.fully_optimal);
    h.check(
        "GPU→GPU no transfer",
        resolved.stages[1].transfer == TransferStrategy::None,
    );
    h.check("Zero transfer overhead", resolved.total_transfer_us == 0);
    println!("  Same-device pipeline: zero transfer overhead.");
}

fn validate_cpu_only_pipeline(h: &mut Harness) {
    println!("\n--- CPU-Only Pipeline (sovereign degradation) ---\n");

    let subs = vec![test_cpu()];
    let topo = Topology::infer(&subs);

    let pipeline = Pipeline::new("CPU sovereign")
        .stage(
            Stage::new("classify", anderson_classify_workload(), 256)
                .with_fallback(FallbackPolicy::Degrade),
        )
        .stage(
            Stage::new("refine", anderson_refine_workload(), 8192)
                .with_fallback(FallbackPolicy::Degrade),
        )
        .stage(Stage::new("store", provenance_store_workload(), 1024));

    let resolved = pipeline.plan(&subs, &topo);
    h.check("CPU-only all assigned", resolved.all_assigned());
    h.check(
        "CPU-only: all stages CPU",
        resolved.stages.iter().all(|s| {
            s.substrate
                .is_some_and(|sub| sub.kind == SubstrateKind::Cpu)
        }),
    );
    h.check(
        "CPU-only: degraded count >= 2",
        resolved.degraded_count() >= 2,
    );
    println!("  Full sovereign degradation: all stages on CPU.");
}

fn validate_pipeline_timing(h: &mut Harness) {
    println!("\n--- Pipeline Timing Instrumentation ---\n");

    let subs = vec![test_npu(), test_gpu(), test_cpu()];
    let topo = Topology::infer(&subs);

    let pipeline = Pipeline::new("Timed pipeline")
        .stage(Stage::new(
            "NPU classify",
            anderson_classify_workload(),
            256,
        ))
        .stage(Stage::new("GPU refine", anderson_refine_workload(), 8192))
        .stage(Stage::new("CPU store", provenance_store_workload(), 1024));

    let t0 = Instant::now();
    let resolved = pipeline.plan(&subs, &topo);
    let plan_us = t0.elapsed().as_micros();

    for (i, stage) in resolved.stages.iter().enumerate() {
        let sub = stage
            .substrate
            .map_or("(skipped)", |s| s.identity.name.as_str());
        let xfer = match stage.transfer {
            TransferStrategy::PeerToPeer => "P2P",
            TransferStrategy::HostBounce => "HOST",
            TransferStrategy::None => "-",
        };
        println!(
            "  Stage {i}: {:<30} → {:<20} xfer={xfer} cost={}µs",
            stage.stage.name, sub, stage.transfer_cost_us,
        );
    }
    println!(
        "  Total transfer: {}µs, plan time: {plan_us}µs",
        resolved.total_transfer_us
    );

    h.check("Timing: plan < 1ms", plan_us < 1000);
    h.check("Timing: 3 stages resolved", resolved.stages.len() == 3);
}

fn validate_node_atomic_pipeline(h: &mut Harness) {
    println!("\n--- Node Atomic Pipeline Planning ---\n");

    let inventory = Inventory {
        substrates: vec![test_gpu(), test_npu(), test_cpu()],
    };
    let node_name = std::env::var("GROUNDSPRING_TEST_NODE").unwrap_or_else(|_| "eastgate".into());
    let node = NodeAtomic::with_inventory(&node_name, inventory);

    let pipeline = Pipeline::new("Anderson via NodeAtomic")
        .stage(Stage::new("classify", anderson_classify_workload(), 256))
        .stage(Stage::new("refine", anderson_refine_workload(), 8192))
        .stage(Stage::new("store", provenance_store_workload(), 1024));

    let resolved = node.plan_pipeline(&pipeline);
    resolved.print_summary();

    h.check("NodeAtomic pipeline all assigned", resolved.all_assigned());
    h.check(
        "NodeAtomic has NPU inference",
        node.capabilities()
            .iter()
            .any(|c| matches!(c, AtomicCapability::NpuInference)),
    );
    h.check(
        "NodeAtomic has pipeline orchestration",
        node.capabilities()
            .iter()
            .any(|c| matches!(c, AtomicCapability::PipelineOrchestration)),
    );
    println!("  NPU↔GPU P2P capability: {}", node.has_npu_gpu_p2p());
}
