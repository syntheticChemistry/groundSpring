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

use crate::dispatch::Workload;
use crate::substrate::Substrate;

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
