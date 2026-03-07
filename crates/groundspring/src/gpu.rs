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
pub use barracuda::device::driver_profile::PrecisionRoutingAdvice;

static DEVICE: OnceLock<Option<Arc<WgpuDevice>>> = OnceLock::new();

/// Get the cached GPU device, creating it on first call.
///
/// Uses `WgpuDevice::new()` which selects the high-performance discrete GPU.
/// On multi-GPU systems this picks the proprietary-driver GPU (RTX 4070)
/// over NVK (Titan V) which has known GPU compute reliability issues
/// (NAK driver can cause hard system freezes on compute dispatch).
///
/// Set `GROUNDSPRING_GPU=0` to disable GPU dispatch entirely (safe mode).
/// Set `WGPU_ADAPTER_NAME=NVIDIA TITAN V` to force Titan V (only when
/// coralReef sovereign compilation path is available).
///
/// Returns `None` if no GPU is available, disabled, or init fails.
pub fn get_device() -> Option<Arc<WgpuDevice>> {
    DEVICE
        .get_or_init(|| {
            if std::env::var("GROUNDSPRING_GPU").as_deref() == Ok("0") {
                return None;
            }
            let future = async { WgpuDevice::new().await };
            barracuda::device::test_pool::tokio_block_on(future)
                .ok()
                .map(Arc::new)
        })
        .clone()
}

/// Query precision routing advice for the cached GPU device.
///
/// Returns the hardware-appropriate precision strategy based on driver
/// profile detection (barraCuda `GpuDriverProfile` from toadStool S128):
///
/// - [`PrecisionRoutingAdvice::F64Native`] — workgroup f64 reductions safe
/// - [`PrecisionRoutingAdvice::F64NativeNoSharedMem`] — avoid `var<workgroup>` f64
/// - [`PrecisionRoutingAdvice::Df64Only`] — use DF64 (f32-pair) for f64 work
/// - [`PrecisionRoutingAdvice::F32Only`] — no f64 support
///
/// Returns `None` if no GPU is available.
#[must_use]
pub fn precision_routing() -> Option<PrecisionRoutingAdvice> {
    use barracuda::device::driver_profile::GpuDriverProfile;
    let device = get_device()?;
    let profile = GpuDriverProfile::from_device(&device);
    Some(profile.precision_routing())
}

/// Returns `true` when the GPU can safely run f64 workgroup-reduction
/// shaders (sum, variance, correlation, etc.).
///
/// Returns `false` for `F64NativeNoSharedMem` (naga/SPIR-V zeros bug),
/// `F32Only` (no f64 at all), or when no GPU is available.
/// Returns `true` for `F64Native` and `Df64Only` (barraCuda routes DF64
/// shaders internally via `Fp64Strategy`).
#[must_use]
pub fn f64_reductions_safe() -> bool {
    matches!(
        precision_routing(),
        Some(PrecisionRoutingAdvice::F64Native | PrecisionRoutingAdvice::Df64Only)
    )
}

/// Get the device only when f64 reductions are safe. Convenience for GPU
/// dispatch paths that depend on workgroup f64 shared-memory reductions.
#[must_use]
pub fn get_device_f64_safe() -> Option<Arc<WgpuDevice>> {
    if f64_reductions_safe() {
        get_device()
    } else {
        None
    }
}
