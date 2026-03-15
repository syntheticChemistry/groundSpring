// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Multi-stage pipeline dispatch — chain workloads across substrates.
//!
//! A pipeline is an ordered sequence of stages, each targeting a specific
//! substrate. Data flows between stages via typed intermediate buffers.
//! The pipeline optimizer uses [`Topology`] to minimize transfer overhead,
//! preferring `PCIe` P2P over CPU bounce when possible.
//!
//! # Example: NPU classification → GPU refinement
//!
//! ```text
//! Stage 0: NPU (int8 classify)  →  regime labels [0,1,2]
//! Stage 1: GPU (f64 Lyapunov)   ←  regime labels → full spectrum
//! Stage 2: CPU (provenance)     ←  spectrum → stored results
//! ```
//!
//! The NPU→GPU transfer uses `PCIe` P2P when devices are on the same bus,
//! bypassing the CPU host memory round-trip entirely.

use crate::dispatch::{self, Workload};
use crate::substrate::{Substrate, SubstrateKind};
use crate::topology::Topology;

/// Transfer strategy between pipeline stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferStrategy {
    /// Direct peer-to-peer DMA (NPU↔GPU via `PCIe`, GPU↔GPU via `NvLink`).
    PeerToPeer,
    /// Bounce through CPU host memory (always available).
    HostBounce,
    /// No transfer needed (same device or CPU-only stage).
    None,
}

/// A single stage in a multi-substrate pipeline.
#[derive(Debug)]
pub struct Stage {
    /// Human-readable stage name.
    pub name: String,
    /// Workload to execute at this stage.
    pub workload: Workload,
    /// Estimated output size in bytes (for transfer cost modeling).
    pub output_bytes: u64,
    /// Fallback behavior if the preferred substrate is unavailable.
    pub fallback: FallbackPolicy,
}

/// What to do when a stage's preferred substrate is unavailable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallbackPolicy {
    /// Try the next-best substrate (GPU→CPU, NPU→CPU).
    Degrade,
    /// Skip this stage entirely (output is empty).
    Skip,
    /// Fail the pipeline immediately.
    Fail,
}

/// A resolved pipeline stage with substrate assignment.
#[derive(Debug)]
pub struct ResolvedStage<'a> {
    /// Original stage definition.
    pub stage: &'a Stage,
    /// Assigned substrate (or `None` if skipped).
    pub substrate: Option<&'a Substrate>,
    /// How data arrives from the previous stage.
    pub transfer: TransferStrategy,
    /// Estimated transfer cost in microseconds.
    pub transfer_cost_us: u64,
    /// Why this substrate was chosen.
    pub reason: StageResolution,
}

/// How a stage was resolved during planning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageResolution {
    /// Workload dispatched to optimal substrate.
    Optimal,
    /// Workload degraded to fallback substrate.
    Degraded,
    /// Stage skipped per fallback policy.
    Skipped,
}

/// A complete pipeline definition.
#[derive(Debug)]
pub struct Pipeline {
    /// Pipeline name for logging and provenance.
    pub name: String,
    /// Ordered stages.
    pub stages: Vec<Stage>,
}

/// A resolved pipeline ready for execution.
#[derive(Debug)]
pub struct ResolvedPipeline<'a> {
    /// Pipeline name.
    pub name: &'a str,
    /// Resolved stages with substrate assignments.
    pub stages: Vec<ResolvedStage<'a>>,
    /// Total estimated transfer overhead in microseconds.
    pub total_transfer_us: u64,
    /// Whether any stage was degraded or skipped.
    pub fully_optimal: bool,
}

impl Pipeline {
    /// Create a new pipeline with a name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            stages: Vec::new(),
        }
    }

    /// Add a stage to the pipeline.
    #[must_use]
    pub fn stage(mut self, stage: Stage) -> Self {
        self.stages.push(stage);
        self
    }

    /// Plan the pipeline: assign substrates and compute transfer costs.
    ///
    /// Uses the topology to determine transfer strategies between stages
    /// and the dispatch router to select substrates.
    #[must_use]
    pub fn plan<'a>(
        &'a self,
        substrates: &'a [Substrate],
        topology: &Topology,
    ) -> ResolvedPipeline<'a> {
        let mut resolved_stages = Vec::with_capacity(self.stages.len());
        let mut total_transfer_us = 0u64;
        let mut fully_optimal = true;
        let mut prev_substrate_idx: Option<usize> = None;

        for stage in &self.stages {
            let decision = dispatch::route(&stage.workload, substrates);
            let (substrate, reason) = match decision {
                Some(d) => (Some(d.substrate), StageResolution::Optimal),
                None => match stage.fallback {
                    FallbackPolicy::Degrade => {
                        let degraded = find_cpu_fallback(substrates);
                        if degraded.is_some() {
                            fully_optimal = false;
                        }
                        (degraded, StageResolution::Degraded)
                    }
                    FallbackPolicy::Skip => {
                        fully_optimal = false;
                        (None, StageResolution::Skipped)
                    }
                    FallbackPolicy::Fail => (None, StageResolution::Optimal),
                },
            };

            let (transfer, transfer_cost) =
                if let (Some(prev_idx), Some(sub)) = (prev_substrate_idx, substrate) {
                    let curr_idx = substrates
                        .iter()
                        .position(|s| std::ptr::eq(s, sub))
                        .unwrap_or(0);
                    if prev_idx == curr_idx {
                        (TransferStrategy::None, 0)
                    } else {
                        let prev_output = resolved_stages
                            .last()
                            .map_or(0, |rs: &ResolvedStage<'_>| rs.stage.output_bytes);
                        let link = topology.best_link(prev_idx, curr_idx);
                        let cost = link.map_or(0, |l| l.tier.transfer_time_us(prev_output));
                        let strategy = link.map_or(TransferStrategy::HostBounce, |l| {
                            if l.tier.is_peer_to_peer() {
                                TransferStrategy::PeerToPeer
                            } else {
                                TransferStrategy::HostBounce
                            }
                        });
                        (strategy, cost)
                    }
                } else {
                    (TransferStrategy::None, 0)
                };

            total_transfer_us += transfer_cost;

            if let Some(sub) = substrate {
                prev_substrate_idx = substrates.iter().position(|s| std::ptr::eq(s, sub));
            }

            resolved_stages.push(ResolvedStage {
                stage,
                substrate,
                transfer,
                transfer_cost_us: transfer_cost,
                reason,
            });
        }

        ResolvedPipeline {
            name: &self.name,
            stages: resolved_stages,
            total_transfer_us,
            fully_optimal,
        }
    }
}

impl Stage {
    /// Create a new pipeline stage.
    pub fn new(name: impl Into<String>, workload: Workload, output_bytes: u64) -> Self {
        Self {
            name: name.into(),
            workload,
            output_bytes,
            fallback: FallbackPolicy::Degrade,
        }
    }

    /// Set the fallback policy.
    #[must_use]
    pub const fn with_fallback(mut self, policy: FallbackPolicy) -> Self {
        self.fallback = policy;
        self
    }
}

impl ResolvedPipeline<'_> {
    /// Check if all stages have assigned substrates (no skips, no failures).
    #[must_use]
    pub fn all_assigned(&self) -> bool {
        self.stages.iter().all(|s| s.substrate.is_some())
    }

    /// Count how many stages use P2P transfers.
    #[must_use]
    pub fn p2p_transfer_count(&self) -> usize {
        self.stages
            .iter()
            .filter(|s| s.transfer == TransferStrategy::PeerToPeer)
            .count()
    }

    /// Count how many stages had to degrade to a fallback substrate.
    #[must_use]
    pub fn degraded_count(&self) -> usize {
        self.stages
            .iter()
            .filter(|s| s.reason == StageResolution::Degraded)
            .count()
    }

    /// Print a human-readable pipeline summary.
    pub fn print_summary(&self) {
        println!("Pipeline: {}", self.name);
        println!(
            "  Stages: {} | Transfer overhead: {}µs | Optimal: {}",
            self.stages.len(),
            self.total_transfer_us,
            self.fully_optimal,
        );
        for (i, rs) in self.stages.iter().enumerate() {
            let sub_name = rs
                .substrate
                .map_or("(skipped)", |s| s.identity.name.as_str());
            let transfer_str = match rs.transfer {
                TransferStrategy::PeerToPeer => "←P2P←",
                TransferStrategy::HostBounce => "←HOST←",
                TransferStrategy::None => "",
            };
            println!(
                "  [{i}] {:<30} → {:<20} {transfer_str} ({:?})",
                rs.stage.name, sub_name, rs.reason,
            );
        }
    }
}

/// Find a CPU substrate as fallback.
fn find_cpu_fallback(substrates: &[Substrate]) -> Option<&Substrate> {
    substrates.iter().find(|s| s.kind == SubstrateKind::Cpu)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::substrate::{Capability, Identity, Properties};

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
