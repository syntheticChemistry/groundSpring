// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

use super::types::{
    FallbackPolicy, Pipeline, ResolvedPipeline, ResolvedStage, Stage, StageResolution,
    TransferStrategy,
};
use crate::dispatch::{self, Workload};
use crate::substrate::{Substrate, SubstrateKind};
use crate::topology::Topology;

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

/// Find a CPU substrate as fallback.
fn find_cpu_fallback(substrates: &[Substrate]) -> Option<&Substrate> {
    substrates.iter().find(|s| s.kind == SubstrateKind::Cpu)
}
