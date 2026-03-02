// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Shared GPU device singleton for barracuda-gpu delegations.
//!
//! `WgpuDevice` creation is async and expensive (~50ms). This module
//! provides a process-wide cached device via [`get_device`], used by
//! GPU-accelerated paths in [`rare_biosphere`](crate::rare_biosphere)
//! and [`quasispecies`](crate::quasispecies).
//!
//! # Precision strategy (`ToadStool` S68+)
//!
//! Uses `new_f64_capable()` which consults barracuda's device registry
//! and runtime f64 probe cache. On NVK/NAK where `SHADER_F64` is
//! advertised but f64 compilation actually fails (groundSpring V35/V37
//! discovery), the probe returns `false` and the device falls back to
//! DF64 emulation (double-float f32-pair, ~48-bit mantissa).

use std::sync::{Arc, OnceLock};

use barracuda::device::WgpuDevice;

static DEVICE: OnceLock<Option<Arc<WgpuDevice>>> = OnceLock::new();

/// Get the cached GPU device, creating it on first call.
///
/// Prefers f64-capable GPUs via `WgpuDevice::new_f64_capable()`,
/// falling back to any available GPU if none support native f64.
/// Returns `None` if no GPU is available (CI, headless, etc.).
pub fn get_device() -> Option<Arc<WgpuDevice>> {
    DEVICE
        .get_or_init(|| {
            let future = async {
                match WgpuDevice::new_f64_capable().await {
                    Ok(dev) => Ok(dev),
                    Err(_) => WgpuDevice::new().await,
                }
            };
            barracuda::device::test_pool::tokio_block_on(future)
                .ok()
                .map(Arc::new)
        })
        .clone()
}
