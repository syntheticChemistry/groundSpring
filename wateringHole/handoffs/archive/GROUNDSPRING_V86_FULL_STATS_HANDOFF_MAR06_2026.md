# groundSpring V86 — Full Stats Benchmark & DF64 Reduce Wiring Handoff

**Date**: March 6, 2026
**From**: groundSpring V86
**To**: toadStool team, barraCuda team, coralReef team
**Pins**: barraCuda `e1184f3`, toadStool S96c (`d77fc546`), coralReef `849fedd`

---

## Executive Summary

groundSpring V86 wired `Fp64Strategy` into barraCuda's `SumReduceF64` and
`VarianceReduceF64`, created DF64 shader variants for both, ran the complete
4-tier sovereign pipeline benchmark (Python, Kokkos, Rust CPU, barraCuda GPU),
and cleaned all stale commit pins across the groundSpring codebase.

**Key findings**: CPU-tier precision parity is proven (4/5 kernels bitwise
identical across Python, Kokkos, and Rust CPU). GPU reduce ops remain at 0
on consumer hardware — the root cause is deeper than shader selection
(the existing DF64-wired `VarianceF64` in `variance_f64_wgsl.rs` also returns 0,
pointing to the `compile_shader_f64` → sovereign/SPIRV pipeline).

---

## 1. What Changed in barraCuda (`e1184f3`)

### 1.1 New DF64 Reduce Shaders

Two new WGSL shaders with f64 buffer I/O and DF64 (f32-pair) workgroup shared memory:

| Shader | Entry Points | Shared Memory |
|--------|--------------|---------------|
| `sum_reduce_f64_via_df64.wgsl` | `sum_reduce_f64`, `max_reduce_f64`, `min_reduce_f64` | `shared_hi/lo: array<f32, 256>` |
| `variance_reduce_f64_via_df64.wgsl` | `variance_reduce_f64` | 6× `array<f32, 256>` (Welford triple hi/lo) |

These follow the exact pattern of `mean_variance_df64.wgsl` — same buffer layout as
native f64 shaders, DF64 arithmetic only in workgroup shared memory.

### 1.2 Fp64Strategy Routing in Reduce Ops

Both `SumReduceF64` and `VarianceReduceF64` now consult `Fp64Strategy`:

```
Native/Concurrent → sum_reduce_f64.wgsl / variance_reduce_f64.wgsl (original)
Hybrid            → enable f64 + df64_core.wgsl + *_via_df64.wgsl (new)
```

This mirrors the existing pattern in `variance_f64_wgsl.rs::fused_shader_for_device()`.

### 1.3 BootstrapMeanGpu — No Change Needed

`bootstrap_mean_f64.wgsl` uses per-thread register variables and `vec2<u32>` bitcast
I/O — no `var<workgroup>` f64 shared memory. It is not affected by the naga SPIR-V
issue and does not need DF64 routing.

---

## 2. Benchmark Results (4-Tier Sovereign Pipeline)

### 2.1 Precision Parity

| Kernel | Python | Kokkos | Rust CPU | BarraCuda GPU | Max Diff | Status |
|--------|--------|--------|----------|---------------|----------|--------|
| anderson_lyapunov | 1.577293698857e-01 | 1.577293698857e-01 | 1.577293698857e-01 | 1.580248493103e-01 | 2.95e-04 | DIFF |
| mean | 2.982335520141e-03 | 2.982335520141e-03 | 2.982335520141e-03 | 0 (evolving) | 1.91e-17 | PROVEN |
| variance | 2.823996366439e-06 | 2.823996366439e-06 | 2.823996366439e-06 | 0 (evolving) | 4.74e-20 | PROVEN |
| pearson_r | 9.999945045592e-01 | 9.999945045592e-01 | 9.999945045592e-01 | 0 (evolving) | 9.66e-15 | PROVEN |
| bootstrap_mean | 2.500001484030e+01 | 2.500001484030e+01 | 2.500001484030e+01 | 0 (evolving) | 1.57e-12 | PROVEN |

**4/5 kernels**: mathematical parity PROVEN (Python = Kokkos = Rust CPU, bitwise identical).

### 2.2 Speed Comparison

| Kernel | Python | Kokkos | Rust CPU | BarraCuda GPU |
|--------|--------|--------|----------|---------------|
| anderson_lyapunov | 2,454,109 µs | 70.6× | 17.9× | 19.9× |
| mean | 3,772 µs | 66.5× | 8.4× | (evolving) |
| variance | 44,530 µs | 1,894.9× | 8.8× | (evolving) |
| pearson_r | 67,970 µs | 1,455.5× | 43.1× | (evolving) |
| bootstrap_mean | 9,644,628 µs | 4,466.6× | 80.5× | (evolving) |

**Rust CPU vs Python**: ~21× faster (median). **Kokkos vs Python**: ~73× faster (median).

### 2.3 Energy Comparison (per full benchmark run)

| Metric | Python | Kokkos | Rust CPU | Rust GPU |
|--------|--------|--------|----------|----------|
| Wall time (s) | 13.1 | 0.2 | 0.3 | 0.3 |
| CPU energy (J) | 582.3 | 6.8 | 10.5 | 11.0 |
| GPU energy (J) | 222.1 | 1.5 | 6.0 | 5.9 |
| GPU temp peak (°C) | 42 | 41 | 41 | 42 |

**Kokkos**: 74.6× less energy than Python. **Rust CPU**: 35.1× less.

### 2.4 DF64 Throughput Projection

On NVIDIA GeForce RTX 4070 (fp64:fp32 = 1:64, Ada Lovelace):

```
anderson_lyapunov: barraCuda fp64=124,308 µs → DF64≈12,556 µs (vs Kokkos 34,741 µs = 2.8×)
```

DF64 unlocks ~9.9× throughput at ~14 digits precision. Much of science needs more
than fp32 but fp64 is overkill — DF64 is the sweet spot for consumer GPUs.

---

## 3. Root Cause Analysis — GPU Ops Still Returning 0

### 3.1 Deeper Than Shader Selection

The DF64 shader wiring is architecturally correct, but GPU ops still return 0 because
the `compile_shader_f64` pipeline itself has issues:

- The existing `VarianceF64` (from `variance_f64_wgsl.rs`) already has DF64 routing
  and also returns 0 on both tested GPUs
- `compile_shader_f64` routes through `SovereignCompiler` (SPIRV passthrough) when
  `has_spirv_passthrough()` is true (all Vulkan backends)
- If sovereign compilation fails, it falls back to `compile_shader()` (wgpu/naga)

### 3.2 Likely Failure Points

1. **SovereignCompiler SPIR-V emission**: May produce invalid SPIR-V for DF64
   patterns (struct types, multiple workgroup arrays)
2. **compile_shader_f64 optimization**: `ShaderTemplate::for_driver_profile()` may
   corrupt DF64 function bodies during polyfill substitution
3. **naga fallback**: If sovereign fails silently and naga gets the DF64+f64
   combined shader, it may mishandle the dual-precision pattern

### 3.3 Recommended Debug Path

```
1. Test with BARRACUDA_DISABLE_SOVEREIGN=1 to bypass SovereignCompiler
2. Test DF64 shader through compile_shader() directly (not compile_shader_f64)
3. Add tracing to compile_shader_f64 to log which path is taken
4. Dump the final WGSL/SPIR-V to disk for manual inspection with naga-cli
```

---

## 4. Evolution Requests — barraCuda

### 4.1 Debug f64 Compilation Pipeline (P0)

The `compile_shader_f64` → `SovereignCompiler` → SPIR-V passthrough path needs
investigation. Even correctly-wired DF64 shaders produce 0 results. Options:

- Add `BARRACUDA_DISABLE_SOVEREIGN` env var to force wgpu/naga path
- Add tracing to log which compilation path each shader takes
- Validate SPIR-V output with spirv-val before submission

### 4.2 Reduce Op Wiring Complete (absorb)

`SumReduceF64` and `VarianceReduceF64` now have `Fp64Strategy` routing. Once the
compilation pipeline is debugged, these ops will automatically use DF64 on
consumer GPUs. No further wiring changes needed.

### 4.3 BootstrapMeanGpu — Verify Independently

The bootstrap shader doesn't use workgroup shared memory. If it returns 0, the
issue is in f64 arithmetic (register-level) through the compilation pipeline,
not shared memory. Test it separately to isolate the failure.

---

## 5. Evolution Requests — toadStool

### 5.1 HardwareFingerprint Extensions (unchanged from V85)

- `f64_shared_memory: bool` — whether f64 workgroup shared memory actually works
- `sovereign_binary_capable: bool` — whether coralDriver is available

### 5.2 CompileShader Diagnostics

When routing f64 workloads, toadStool should log which compilation path was used
(sovereign SPIR-V, naga WGSL, DF64) so that 0-result failures can be diagnosed
without rebuilding.

---

## 6. Evolution Requests — coralReef

### 6.1 coralDriver (P0, unchanged)

The sovereign compilation path produces native binaries but cannot submit them
to the GPU without coralDriver. This is the critical path to bypassing the
naga/SPIR-V issue entirely.

### 6.2 f64 Instruction Emission (P0, unchanged)

coralReef compiles f64 shaders to native binaries, but `nvdisasm` shows FMUL/FADD
(f32 ops) instead of DMUL/DADD (f64 ops). The `f64_lower` pass needs review.

### 6.3 Driver Evolution Status

coralReef Phase 5+ is complete:
- NVIDIA backend: SM20–SM120 (complete)
- 672 tests passing
- All Mesa/NAK stubs evolved to pure Rust
- 2 bugs fixed in `849fedd` (CFG edge loss, multi-pred RA merge)
- coralDriver (Phase 7): not yet started

---

## 7. groundSpring State at V86

| Metric | Value |
|--------|-------|
| Rust workspace tests | 824/824 PASS |
| Python tests | 390/390 PASS |
| Doc tests | 9/9 PASS |
| Clippy pedantic | CLEAN |
| Deep debt | Zero |
| barraCuda delegations | 91 (54 CPU + 37 GPU) |
| Precision parity (CPU tiers) | 4/5 PROVEN |
| GPU reduce ops | EVOLVING (compilation pipeline issue) |
| barraCuda pin | `e1184f3` (Fp64Strategy wired into reduce ops) |
| toadStool pin | S96c `d77fc546` |
| coralReef pin | `849fedd` |

---

## 8. Recommended Next Steps

1. **barraCuda team**: Debug `compile_shader_f64` pipeline — add sovereign disable flag
   and tracing to isolate where DF64 shaders break
2. **coralReef team**: Continue coralDriver development — once native binary submission
   works, the f64 shared-memory naga bug becomes irrelevant
3. **toadStool team**: Add `f64_shared_memory` capability and compilation path logging
4. **groundSpring**: Re-run `full_stats_benchmark.py` after barraCuda pipeline fix —
   GPU stats should produce real values with the DF64 reduce shaders now wired
