// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Shared GPU device singleton for barracuda-gpu delegations.
//!
//! `WgpuDevice` creation is async and expensive (~50ms). This module
//! provides a process-wide cached device via [`get_device`], used by
//! GPU-accelerated paths in [`rare_biosphere`](crate::rare_biosphere)
//! and [`quasispecies`](crate::quasispecies).
//!
//! # Device selection strategy
//!
//! 1. `WgpuDevice::new()` — high-performance discrete GPU (proprietary driver
//!    preferred over NVK for compute reliability).
//! 2. `WgpuDevice::new_f64_capable()` — f64-capable GPU (may select NVK/Titan V
//!    which has compute issues; reserved for when coralReef sovereign path is ready).
//!
//! Override with `WGPU_ADAPTER_NAME` environment variable for explicit selection.

use std::sync::{Arc, OnceLock};

use barracuda::device::WgpuDevice;

static DEVICE: OnceLock<Option<Arc<WgpuDevice>>> = OnceLock::new();

/// Get the cached GPU device, creating it on first call.
///
/// Uses `WgpuDevice::new()` which selects the high-performance discrete GPU.
/// On multi-GPU systems this picks the proprietary-driver GPU (RTX 4070)
/// over NVK (Titan V) which has known GPU compute reliability issues.
///
/// Override: set `WGPU_ADAPTER_NAME=NVIDIA TITAN V` to force Titan V
/// (useful once coralReef sovereign path is available).
///
/// Returns `None` if no GPU is available (CI, headless, etc.).
pub fn get_device() -> Option<Arc<WgpuDevice>> {
    DEVICE
        .get_or_init(|| {
            let future = async { WgpuDevice::new().await };
            barracuda::device::test_pool::tokio_block_on(future)
                .ok()
                .map(Arc::new)
        })
        .clone()
}
