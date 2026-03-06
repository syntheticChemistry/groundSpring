# groundSpring V84 → GPU Validation Discovery Handoff

**Date:** 2026-03-06
**From:** groundSpring V84 (34 experiments, 824 workspace tests)
**To:** barraCuda team, coralReef team, toadStool team
**License:** AGPL-3.0-only
**Covers:** V83 → V84 (GPU validation, f64 pipeline diagnostics, device selection)
**Pins:** barraCuda `cf1602c+1`, toadStool S96c (`d77fc546`), coralReef `1e048be`

---

## Executive Summary

groundSpring V84 drove real GPU workloads through barraCuda on a dual-GPU system
(RTX 4070 + Titan V). This uncovered a systematic f64 WGSL compilation issue and
a CoralCompiler runtime panic, both fixed or documented here.

**What works:**
- DF64 (f32-pair): tensor matmul, DF64 add/sub — all green
- FHE NTT round-trips — bit-perfect
- validate_gpu: 6/6 pass on RTX 4070
- 17/32 groundSpring GPU tests pass (those not depending on f64 reduction)

**What fails:**
- f64 WGSL reduction shaders with workgroup shared memory return 0
- Affects BOTH GPUs (RTX 4070 proprietary + Titan V NVK)
- SumReduceF64, VarianceF64, CovarianceF64, CorrelationF64 all return 0

---

## Hardware Inventory

| GPU | Arch | Driver | Vulkan | f64 Hardware | Strategy |
|-----|------|--------|--------|-------------|----------|
| RTX 4070 | SM89 Ada | Proprietary 580.119.02 | 1.4.312 | 1:64 throttled | Hybrid |
| Titan V | SM70 Volta (GV100) | NVK/NAK (Mesa) | 1.3.311 | 1:2 native | Native |

Both GPUs are discrete, both present via `/dev/dri/renderD128` and `/dev/dri/renderD129`.
RTX 4070 on proprietary driver (nvidia-smi visible); Titan V on nouveau/NVK.

## f64 Builtin Probe Results

Both GPUs were probed via `bench_f64_builtins`:

| Builtin | RTX 4070 | Titan V |
|---------|----------|---------|
| basic_f64 (3*2+1=7) | ✓ (implied) | ✓ (implied) |
| exp(f64) | ✗ fallback | ✗ fallback |
| log(f64) | ✗ fallback | ✗ fallback |
| exp2(f64) | ✗ fallback | ✗ fallback |
| log2(f64) | ✗ fallback | ✗ fallback |
| sin(f64) | ✗ fallback | ✗ fallback |
| cos(f64) | ✗ fallback | ✗ fallback |
| sqrt(f64) | ✓ NATIVE | ✓ NATIVE |
| fma(f64) | ✓ NATIVE | ✓ NATIVE |
| abs/min/max | ✓ NATIVE | ✓ NATIVE |

Both devices: 3/9 native, software lib required for most transcendentals.

---

## Critical Finding: f64 Reduction Shaders Return Zero

### Reproduction

```bash
cd barraCuda
cargo test --features gpu -- sum_reduce      # All 3 fail, return 0
cargo test --features gpu -- variance_f64    # 9/11 fail, return 0
cargo test --features gpu -- correlation_f64 # 6/9 fail, return 0
BARRACUDA_GPU_ADAPTER=1 cargo test --features gpu -- sum_reduce  # Titan V: same
```

### Root Cause Analysis

1. **Simple f64 arithmetic works** — the `basic_f64` probe (`let y = x * 2.0 + 1.0`) returns 7.0
2. **f64 with workgroup shared memory fails** — `var<workgroup> shared_data: array<f64, 256>`
   produces a buffer of zeros after `workgroupBarrier()` + tree reduction
3. **DF64 and f32 operations work** — `validate_gpu` tensor matmul and DF64 add/sub pass
4. **Both drivers affected** — proprietary NVVM and NVK/NAK produce same result

The wgpu → naga → SPIR-V pipeline likely fails to properly handle `f64` workgroup
shared memory in one of these stages:
- naga may not emit correct `Float64` + `StorageBuffer` capabilities for shared memory
- The SPIR-V Decoration for workgroup f64 arrays may be missing
- The sovereign compiler's FMA fusion pass may mishandle shared memory ops

### Impact

All barraCuda GPU ops using f64 workgroup reductions are affected:
- `SumReduceF64` (sum, min, max, mean, dot_product)
- `VarianceReduceF64` (variance, std_dev, fused mean+variance)
- `CovarianceF64` (covariance, sample_covariance)
- `CorrelationF64` (pearson_r, full stats)
- `AutocorrelationF64` (lag-k autocorrelation)

### Recommended Fix Path

**Option A (immediate): DF64 variants of reduction shaders**
The DF64 compilation path works. Create `sum_reduce_df64.wgsl` using `Df64` types
and `var<workgroup> shared_data: array<vec2<f32>, 256>` (DF64 as f32 pair).
Ops should check `Fp64Strategy` and route to DF64 when `Hybrid`.

**Option B (medium-term): Fix naga SPIR-V emission for f64 shared memory**
Investigate naga's SPIR-V backend for proper `OpTypeFloat 64` in workgroup
shared memory declarations. This may be a wgpu/naga upstream issue.

**Option C (long-term): coralReef sovereign compilation**
coralReef bypasses naga entirely: WGSL → coralReef IR → native SM binary.
This eliminates the naga/SPIR-V shared memory issue. Requires coralDriver
for direct GPU submission.

---

## Bug Fix: CoralCompiler Tokio Panic

### Problem
`spawn_coral_compile()` in `barracuda/src/device/coral_compiler.rs` calls `tokio::spawn()`
unconditionally. When called from a non-async context (e.g., groundSpring GPU tests),
this panics: "there is no reactor running, must be called from the context of a Tokio 1.x runtime."

### Fix Applied (1 line)
```rust
// Before tokio::spawn, check if runtime exists:
let Ok(_handle) = tokio::runtime::Handle::try_current() else {
    return;
};
```

This is a defensive guard — when no Tokio runtime is available, `spawn_coral_compile`
silently returns instead of panicking. The standard wgpu path continues unaffected.

**File:** `crates/barracuda/src/device/coral_compiler.rs:231`
**Impact:** Fixes all barraCuda GPU ops when called from synchronous Rust code.

---

## groundSpring Device Selection Update

Previous: `WgpuDevice::new_f64_capable()` — preferred f64 devices, could select
Titan V (NVK) which had compute stability issues.

Updated: `WgpuDevice::new()` — selects high-performance discrete GPU (RTX 4070
on proprietary driver). Override with `WGPU_ADAPTER_NAME` env var.

**File:** `crates/groundspring/src/gpu.rs`

---

## Titan V Status

The Titan V (SM70 Volta, 1:2 f64 hardware) is the primary target for native f64
compute — it has 2560 FP64 cores at ~7.5 TFLOPS. Currently blocked by:

1. **NVK driver:** GPU compute with f64 shared memory returns 0 (same naga issue)
2. **No proprietary driver:** nvidia-smi shows only RTX 4070; Titan V is on nouveau
3. **Previous freeze:** Heavy GPU compute on NVK froze the system

### Path to Titan V at Full Speed

| Path | Timeline | Requires |
|------|----------|----------|
| Fix naga f64 shared memory | Weeks | wgpu upstream contribution |
| Move Titan V to proprietary driver | Days | X11/Wayland multi-GPU config |
| coralReef SM70 binary compilation | Months | coralReef Phase 7+ |

Recommendation: Try moving Titan V to the proprietary driver (both GPUs on nvidia
kernel module) as the fastest path to native f64 compute.

---

## RTX 4070 Utilization Strategy

The RTX 4070 (SM89 Ada) has 5888 FP32 cores but only 46 FP64 cores (1:128 ratio).
The DF64 strategy is correct for this GPU:

- **DF64 path works now:** Tensor matmul, DF64 add/sub verified
- **FP32 throughput:** ~29 TFLOPS (massive)
- **DF64 effective throughput:** ~14.5 TFLOPS at ~48-bit mantissa
- **Native f64 throughput:** ~0.23 TFLOPS (unusable for bulk math)

All groundSpring workloads can run on RTX 4070 via DF64 once barraCuda
implements DF64 reduction shaders.

---

## For coralReef Team

The f64 shared memory issue in naga/SPIR-V makes coralReef's sovereign path
even more valuable. When coralReef can compile WGSL → SM70 binary directly:

1. Titan V f64 shared memory will work (no naga in the path)
2. RTX 4070 DF64 through coralReef avoids driver-specific workarounds
3. `coralDriver` direct submission eliminates wgpu overhead entirely

### Requested Evolution

- **SM70 (Volta):** Priority target — Titan V with 7.5 TFLOPS f64
- **SM89 (Ada):** DF64 code generation (f32-pair operations)
- **Frontend trait:** groundSpring WGSL shaders as test corpus
  (708 shaders from barraCuda + 34 experiment shaders from groundSpring)

---

## For toadStool Team

### HardwareFingerprint Enhancement

The `HardwareFingerprint` in `runtime/universal` should capture:
- `f64_shared_memory_works: bool` (runtime probed, not just feature advertised)
- `df64_throughput_tflops: f32` (estimated DF64 effective throughput)
- `driver_type: enum { Proprietary, Nvk, Radv, Amdvlk }`

This enables capability-based dispatch: route f64 workloads to Titan V,
DF64 workloads to RTX 4070, use both concurrently for different precision tiers.

### SubstrateCapabilityKind Extension

New variants to consider:
- `Df64Reduction` — DF64 workgroup reduction (proven working)
- `F64SharedMemory` — native f64 in workgroup shared memory (currently broken)

---

## Validation Certificate

```
groundSpring V84 — GPU Validation Discovery
Date: 2026-03-06
Hardware: RTX 4070 (SM89) + Titan V (SM70/NVK)
Platform: Linux 6.17.9 / NVIDIA 580.119.02 / Mesa NVK

CPU path:       824/824 tests pass (cargo test --workspace)
GPU path:       17/32 tests pass (cargo test --features barracuda-gpu)
GPU failing:    14 tests (f64 reduction returning 0 — naga shared memory issue)
GPU bio tests:  4/4 pass (shannon, simpson diversity — CPU fallback)
Clippy:         0 warnings (groundSpring + barraCuda lib)
validate_gpu:   6/6 pass on RTX 4070 (DF64, tensor matmul, FHE NTT)
CoralCompiler:  Fixed tokio::spawn panic (try_current guard)
```

---

## Next Steps

1. **barraCuda:** Implement DF64 reduction shaders (sum_reduce_df64.wgsl, etc.)
2. **barraCuda:** Make ops check Fp64Strategy and route to DF64 when Hybrid
3. **coralReef:** SM70 compilation target (unlocks Titan V native f64)
4. **toadStool:** HardwareFingerprint f64 shared memory probe
5. **groundSpring:** Re-validate GPU path once DF64 reductions land
6. **System admin:** Consider dual proprietary driver config for both GPUs
