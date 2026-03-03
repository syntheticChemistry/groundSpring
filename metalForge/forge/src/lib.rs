// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! groundSpring `MetalForge` — hardware discovery and cross-substrate dispatch.
//!
//! Discovers GPU (via wgpu), NPU (`BrainChip` AKD1000 via device nodes), and
//! CPU (via procfs) at runtime. Routes groundSpring workloads by capability:
//!
//! - **f64 + shader** -> GPU (Anderson transfer matrix, Almost-Mathieu eigenvalues)
//! - **quant(int8)** -> NPU (regime classification, saturation prediction)
//! - **f64 scalar** -> CPU (decomposition, extrapolation, PRNG)
//!
//! # barraCuda S80+ primitives
//!
//! Multi-op GPU pipelines can use `barracuda::device::batched_encoder::BatchedEncoder`
//! (barraCuda S80) to fuse multiple compute dispatches into a single command
//! encoder submission, reducing per-op `queue.submit()` overhead.
//!
//! Device-lost resilience is available via `barracuda::error::BarracudaError::is_device_lost()`
//! and `barracuda::error::BarracudaError::is_retriable()` for long-running
//! GPU validation pipelines (barraCuda S87).

pub mod atomic;
pub mod dispatch;
pub mod harness;
pub mod inventory;
pub mod pipeline;
pub mod probe;
pub mod remote;
pub mod substrate;
pub mod tolerance;
pub mod topology;
pub mod workloads;
