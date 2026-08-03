// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Multi-stage pipeline dispatch — chain workloads across substrates.

mod planner;
mod summary;
mod types;

pub use types::*;

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use crate::dispatch::Workload;
    use crate::substrate::{Capability, Identity, Properties, Substrate, SubstrateKind};
    use crate::topology::Topology;

    fn gpu_sub() -> Substrate {
        Substrate {
            kind: SubstrateKind::Gpu,
            identity: Identity::named("Test GPU"),
            properties: Properties::default(),
            capabilities: vec![
                Capability::F64Compute,
                Capability::ShaderDispatch,
                Capability::ScalarReduce,
            ],
        }
    }

    fn npu_sub() -> Substrate {
        Substrate {
            kind: SubstrateKind::Npu,
            identity: Identity::named("BrainChip AKD1000"),
            properties: Properties::default(),
            capabilities: vec![
                Capability::QuantizedInference { bits: 8 },
                Capability::BatchInference { max_batch: 8 },
            ],
        }
    }

    fn cpu_sub() -> Substrate {
        Substrate {
            kind: SubstrateKind::Cpu,
            identity: Identity::named("Test CPU"),
            properties: Properties::default(),
            capabilities: vec![Capability::F64Compute, Capability::F32Compute],
        }
    }

    #[test]
    fn pipeline_plans_npu_then_gpu() {
        let subs = vec![gpu_sub(), npu_sub(), cpu_sub()];
        let topo = Topology::infer(&subs);

        let pipeline = Pipeline::new("NPU classify → GPU refine")
            .stage(Stage::new(
                "NPU classify",
                Workload::new("classify", vec![Capability::QuantizedInference { bits: 8 }]),
                1024,
            ))
            .stage(Stage::new(
                "GPU Lyapunov",
                Workload::new(
                    "lyapunov",
                    vec![Capability::F64Compute, Capability::ShaderDispatch],
                ),
                8192,
            ));

        let resolved = pipeline.plan(&subs, &topo);
        assert!(resolved.all_assigned());
        assert_eq!(resolved.stages.len(), 2);
        assert_eq!(
            resolved.stages[0].substrate.unwrap().kind,
            SubstrateKind::Npu
        );
        assert_eq!(
            resolved.stages[1].substrate.unwrap().kind,
            SubstrateKind::Gpu
        );
    }

    #[test]
    fn pipeline_degrades_when_npu_missing() {
        let subs = vec![gpu_sub(), cpu_sub()];
        let topo = Topology::infer(&subs);

        let pipeline = Pipeline::new("degrade test").stage(
            Stage::new(
                "NPU classify",
                Workload::new("classify", vec![Capability::QuantizedInference { bits: 8 }]),
                1024,
            )
            .with_fallback(FallbackPolicy::Degrade),
        );

        let resolved = pipeline.plan(&subs, &topo);
        assert!(resolved.all_assigned());
        assert_eq!(resolved.degraded_count(), 1);
        assert_eq!(
            resolved.stages[0].substrate.unwrap().kind,
            SubstrateKind::Cpu
        );
    }

    #[test]
    fn pipeline_skips_when_policy_is_skip() {
        let subs = vec![cpu_sub()];
        let topo = Topology::infer(&subs);

        let pipeline = Pipeline::new("skip test").stage(
            Stage::new(
                "GPU work",
                Workload::new(
                    "gpu_only",
                    vec![Capability::F64Compute, Capability::ShaderDispatch],
                ),
                4096,
            )
            .with_fallback(FallbackPolicy::Skip),
        );

        let resolved = pipeline.plan(&subs, &topo);
        assert!(!resolved.all_assigned());
        assert!(resolved.stages[0].substrate.is_none());
        assert_eq!(resolved.stages[0].reason, StageResolution::Skipped);
    }

    #[test]
    fn pipeline_fully_optimal_when_all_match() {
        let subs = vec![gpu_sub(), cpu_sub()];
        let topo = Topology::infer(&subs);

        let pipeline = Pipeline::new("optimal").stage(Stage::new(
            "GPU work",
            Workload::new(
                "compute",
                vec![Capability::F64Compute, Capability::ShaderDispatch],
            ),
            4096,
        ));

        let resolved = pipeline.plan(&subs, &topo);
        assert!(resolved.fully_optimal);
    }

    #[test]
    fn three_stage_pipeline_with_transfers() {
        let subs = vec![npu_sub(), gpu_sub(), cpu_sub()];
        let topo = Topology::infer(&subs);

        let pipeline = Pipeline::new("NPU → GPU → CPU")
            .stage(Stage::new(
                "classify",
                Workload::new("classify", vec![Capability::QuantizedInference { bits: 8 }]),
                256,
            ))
            .stage(Stage::new(
                "compute",
                Workload::new(
                    "lyapunov",
                    vec![Capability::F64Compute, Capability::ShaderDispatch],
                ),
                8192,
            ))
            .stage(Stage::new(
                "store",
                Workload::new("provenance", vec![Capability::F64Compute]),
                1024,
            ));

        let resolved = pipeline.plan(&subs, &topo);
        assert!(resolved.all_assigned());
        assert_eq!(resolved.stages.len(), 3);
        resolved.print_summary();
    }

    #[test]
    fn empty_pipeline_plans_ok() {
        let subs = vec![cpu_sub()];
        let topo = Topology::infer(&subs);
        let pipeline = Pipeline::new("empty");
        let resolved = pipeline.plan(&subs, &topo);
        assert!(resolved.all_assigned());
        assert!(resolved.fully_optimal);
        assert_eq!(resolved.total_transfer_us, 0);
    }

    #[test]
    fn same_substrate_no_transfer() {
        let subs = vec![gpu_sub(), cpu_sub()];
        let topo = Topology::infer(&subs);

        let pipeline = Pipeline::new("GPU→GPU")
            .stage(Stage::new(
                "first",
                Workload::new(
                    "a",
                    vec![Capability::F64Compute, Capability::ShaderDispatch],
                ),
                4096,
            ))
            .stage(Stage::new(
                "second",
                Workload::new(
                    "b",
                    vec![Capability::F64Compute, Capability::ShaderDispatch],
                ),
                4096,
            ));

        let resolved = pipeline.plan(&subs, &topo);
        assert_eq!(resolved.stages[1].transfer, TransferStrategy::None);
        assert_eq!(resolved.stages[1].transfer_cost_us, 0);
    }

    #[test]
    fn npu_to_gpu_p2p_bypasses_cpu() {
        let subs = vec![npu_sub(), gpu_sub(), cpu_sub()];
        let topo = Topology::infer(&subs);

        let pipeline = Pipeline::new("NPU→GPU direct")
            .stage(Stage::new(
                "npu_classify",
                Workload::new("classify", vec![Capability::QuantizedInference { bits: 8 }]),
                256,
            ))
            .stage(Stage::new(
                "gpu_compute",
                Workload::new(
                    "anderson",
                    vec![Capability::F64Compute, Capability::ShaderDispatch],
                ),
                65536,
            ));

        let resolved = pipeline.plan(&subs, &topo);
        assert!(resolved.all_assigned());
        assert_eq!(
            resolved.stages[0].substrate.unwrap().kind,
            SubstrateKind::Npu,
            "stage 0 → NPU"
        );
        assert_eq!(
            resolved.stages[1].substrate.unwrap().kind,
            SubstrateKind::Gpu,
            "stage 1 → GPU"
        );
        assert!(
            resolved.stages[1].transfer != TransferStrategy::None,
            "NPU→GPU transfer should be non-trivial"
        );
    }

    #[test]
    fn mixed_pipeline_fallback_preserves_stages() {
        let subs = vec![cpu_sub()];
        let topo = Topology::infer(&subs);

        let pipeline = Pipeline::new("degrade all")
            .stage(
                Stage::new(
                    "npu_classify",
                    Workload::new("classify", vec![Capability::QuantizedInference { bits: 8 }]),
                    256,
                )
                .with_fallback(FallbackPolicy::Degrade),
            )
            .stage(
                Stage::new(
                    "gpu_compute",
                    Workload::new(
                        "anderson",
                        vec![Capability::F64Compute, Capability::ShaderDispatch],
                    ),
                    65536,
                )
                .with_fallback(FallbackPolicy::Degrade),
            )
            .stage(Stage::new(
                "cpu_store",
                Workload::new("provenance", vec![Capability::F64Compute]),
                1024,
            ));

        let resolved = pipeline.plan(&subs, &topo);
        assert_eq!(resolved.stages.len(), 3);
        assert_eq!(
            resolved.degraded_count(),
            2,
            "NPU+GPU should degrade to CPU"
        );
        for stage in &resolved.stages {
            if let Some(sub) = stage.substrate {
                assert_eq!(sub.kind, SubstrateKind::Cpu, "all should degrade to CPU");
            }
        }
    }

    #[test]
    fn mixed_pipeline_fail_on_missing_gpu() {
        let subs = vec![cpu_sub()];
        let topo = Topology::infer(&subs);

        let pipeline = Pipeline::new("fail without GPU").stage(
            Stage::new(
                "gpu_only",
                Workload::new(
                    "anderson",
                    vec![Capability::F64Compute, Capability::ShaderDispatch],
                ),
                65536,
            )
            .with_fallback(FallbackPolicy::Fail),
        );

        let resolved = pipeline.plan(&subs, &topo);
        assert!(!resolved.all_assigned(), "pipeline should fail without GPU");
        assert!(
            resolved.stages[0].substrate.is_none(),
            "Fail policy: no substrate assigned"
        );
    }

    #[test]
    fn five_stage_heterogeneous_pipeline() {
        let subs = vec![npu_sub(), gpu_sub(), cpu_sub()];
        let topo = Topology::infer(&subs);

        let pipeline = Pipeline::new("full science pipeline")
            .stage(Stage::new(
                "ingest",
                Workload::new("provenance", vec![Capability::F64Compute]),
                2048,
            ))
            .stage(Stage::new(
                "classify",
                Workload::new("classify", vec![Capability::QuantizedInference { bits: 8 }]),
                512,
            ))
            .stage(Stage::new(
                "gpu_sweep",
                Workload::new(
                    "anderson_sweep",
                    vec![Capability::F64Compute, Capability::ShaderDispatch],
                ),
                131_072,
            ))
            .stage(Stage::new(
                "cpu_analyze",
                Workload::new("statistics", vec![Capability::F64Compute]),
                4096,
            ))
            .stage(Stage::new(
                "store",
                Workload::new("provenance", vec![Capability::F64Compute]),
                1024,
            ));

        let resolved = pipeline.plan(&subs, &topo);
        assert!(resolved.all_assigned(), "all 5 stages should be assigned");
        assert_eq!(resolved.stages.len(), 5);
        assert!(
            resolved.total_transfer_us > 0,
            "heterogeneous pipeline should have transfer costs"
        );
    }
}
