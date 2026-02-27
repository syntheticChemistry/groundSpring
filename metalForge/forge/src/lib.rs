// SPDX-License-Identifier: AGPL-3.0-or-later

//! groundSpring `MetalForge` — hardware discovery and cross-substrate dispatch.
//!
//! Discovers GPU (via wgpu), NPU (`BrainChip` AKD1000 via device nodes), and
//! CPU (via procfs) at runtime. Routes groundSpring workloads by capability:
//!
//! - **f64 + shader** -> GPU (Anderson transfer matrix, Almost-Mathieu eigenvalues)
//! - **quant(int8)** -> NPU (regime classification, saturation prediction)
//! - **f64 scalar** -> CPU (decomposition, extrapolation, PRNG)

pub mod dispatch;
pub mod inventory;
pub mod probe;
pub mod substrate;
pub mod workloads;
