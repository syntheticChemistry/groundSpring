// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Shared GPU device singleton for barracuda-gpu delegations.
//!
//! `WgpuDevice` creation is async and expensive (~50ms). This module
//! provides a process-wide cached device via [`get_device`], used by
//! GPU-accelerated paths in [`rare_biosphere`](crate::rare_biosphere)
//! and [`quasispecies`](crate::quasispecies).

use std::sync::{Arc, OnceLock};

use barracuda::device::WgpuDevice;

static DEVICE: OnceLock<Option<Arc<WgpuDevice>>> = OnceLock::new();

/// Get the cached GPU device, creating it on first call.
///
/// Returns `None` if no GPU is available (CI, headless, etc.).
pub fn get_device() -> Option<Arc<WgpuDevice>> {
    DEVICE
        .get_or_init(|| {
            // Auto::new() returns pooled Arc<WgpuDevice> from barracuda's LazyLock pool.
            // WgpuDevice::new() returns a fresh device; we wrap in Arc for sharing.
            pollster::block_on(WgpuDevice::new()).ok().map(Arc::new)
        })
        .clone()
}
