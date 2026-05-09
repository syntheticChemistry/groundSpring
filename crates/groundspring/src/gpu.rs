// SPDX-License-Identifier: AGPL-3.0-or-later
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
        .get_or_init(|| get_device_with_env(|k| std::env::var(k).ok()))
        .clone()
}

fn get_device_with_env(env: impl Fn(&str) -> Option<String>) -> Option<Arc<WgpuDevice>> {
    if env("GROUNDSPRING_GPU").as_deref() == Some("0") {
        return None;
    }
    let future = async { WgpuDevice::new().await };
    barracuda::device::test_pool::tokio_block_on(future)
        .ok()
        .map(Arc::new)
}

/// Query precision routing advice for the cached GPU device.
///
/// Returns the hardware-appropriate precision strategy based on device
/// capabilities detection (barraCuda `DeviceCapabilities` from Sprint 14):
///
/// - [`PrecisionRoutingAdvice::F64Native`] — workgroup f64 reductions safe
/// - [`PrecisionRoutingAdvice::F64NativeNoSharedMem`] — avoid `var<workgroup>` f64
/// - [`PrecisionRoutingAdvice::Df64Only`] — use DF64 (f32-pair) for f64 work
/// - [`PrecisionRoutingAdvice::F32Only`] — no f64 support
///
/// Returns `None` if no GPU is available.
#[must_use]
pub fn precision_routing() -> Option<PrecisionRoutingAdvice> {
    use barracuda::device::capabilities::DeviceCapabilities;
    let device = get_device()?;
    let caps = DeviceCapabilities::from_device(&device);
    Some(caps.precision_routing())
}

/// Returns `true` when the GPU can safely run f64 workgroup-reduction
/// shaders (sum, variance, correlation, etc.).
///
/// Checks **both** the driver profile classification (fast, no dispatch)
/// **and** a one-time runtime smoke test that actually runs a small
/// `SumReduceF64::mean` on `[1.0; 4]`.  This catches GPUs that the
/// driver profile classifies as `F64Native` but whose naga/SPIR-V
/// path silently produces zeros for workgroup shared-memory f64.
///
/// Returns `false` for `F64NativeNoSharedMem`, `F32Only`, no GPU,
/// or a failed runtime smoke test.
#[must_use]
pub fn f64_reductions_safe() -> bool {
    static SAFE: OnceLock<bool> = OnceLock::new();
    *SAFE.get_or_init(|| {
        let profile_ok = matches!(
            precision_routing(),
            Some(PrecisionRoutingAdvice::F64Native | PrecisionRoutingAdvice::Df64Only)
        );
        if !profile_ok {
            return false;
        }
        f64_reduction_smoke_test()
    })
}

/// Acceptable deviation for the f64 reduction smoke test.
///
/// The test computes `mean([1.0; 4])` on the GPU — a result of exactly 1.0
/// is expected. A 1% tolerance catches silent-zero bugs and gross precision
/// failures without flagging rounding at the ULP level.
const F64_REDUCTION_SMOKE_TOL: f64 = 0.01;

/// Run a tiny GPU reduction and verify the result is non-zero.
fn f64_reduction_smoke_test() -> bool {
    let Some(device) = get_device() else {
        return false;
    };
    let test_data = [1.0_f64; 4];
    let Ok(result) = barracuda::ops::sum_reduce_f64::SumReduceF64::mean(device, &test_data) else {
        return false;
    };
    (result - 1.0).abs() < F64_REDUCTION_SMOKE_TOL
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_device_does_not_panic() {
        let _ = get_device();
    }

    #[test]
    fn gpu_disabled_env_returns_none() {
        let result = get_device_with_env(|k| {
            if k == "GROUNDSPRING_GPU" {
                Some("0".into())
            } else {
                None
            }
        });
        assert!(result.is_none());
    }

    #[test]
    fn smoke_tol_is_reasonable() {
        assert!(F64_REDUCTION_SMOKE_TOL > 0.0 && F64_REDUCTION_SMOKE_TOL < 0.1);
    }
}
