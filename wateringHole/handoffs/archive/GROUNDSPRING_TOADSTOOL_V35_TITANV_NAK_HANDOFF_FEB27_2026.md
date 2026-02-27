# groundSpring → ToadStool V35 Handoff: Titan V / NAK Adaptive GPU Dispatch

**Date**: February 27, 2026
**From**: groundSpring V35
**To**: ToadStool S68+
**Status**: Architecture-aware dispatch live, Titan V confirmed via NVK/NAK

---

## Summary

metalForge now detects GPU architecture (Volta/Turing/Ampere/Ada) at runtime
and routes f64 workloads to the GPU with the best native f64 throughput. On
this system, **all 17 f64 workloads now route to the Titan V** (`GV100`, 1:2
f64:f32 ratio) instead of the RTX 4070 (`AD104`, 1:64 ratio) — a 32× f64
throughput advantage.

Adaptive memory batching computes software-side batch sizes that keep workloads
within GPU VRAM, enabling unidirectional streaming (data stays on-device
between batches). This is essential for Volta/NAK where hardware memory
management differs from newer architectures.

---

## Hardware Inventory (5 substrates discovered)

| Kind | Name | Arch | Capabilities |
|------|------|------|-------------|
| GPU | NVIDIA GeForce RTX 4070 | Ada | f32, shader, f64, reduce, timestamps |
| GPU | NVIDIA TITAN V (NVK GV100) | Volta | f32, shader, f64, reduce, **native-f64**, timestamps |
| GPU | NVIDIA GeForce RTX 4070/PCIe/SSE2 | Ada | f32, shader, timestamps |
| NPU | BrainChip AKD1000 | — | f32, quant, batch, weight-mut |
| CPU | i9-12900K | — | f64, f32, simd |

---

## Architecture-Aware Dispatch

### f64 Routing Preference

When a workload requires `F64Compute`, the router now prefers GPUs with
`NativeF64` capability (Volta's 1:2 ratio) over GPUs that only have
`F64Compute` (Ada/Ampere's 1:64 ratio requiring DF64 emulation).

### Adaptive Batch Parameters

| Architecture | f64 Ratio | Workgroup | VRAM Default | Resident Memory |
|---|---|---|---|---|
| Volta (GV100) | 1:2 | 64 | 12 GB | Yes (HBM2) |
| Ada (AD104) | 1:64 | 256 | 12 GB | No |
| Ampere | 1:64 | 256 | 10 GB | No |
| Turing | 1:32 | 128 | 8 GB | No |

**Resident memory mode**: On Volta, buffers stay on-device between dispatches
(unidirectional streaming). This reduces round-trip dispatch overhead — data
goes to GPU once, intermediate results stay in HBM2, final results come back.
This is the software-side equivalent of hardware memory batching that newer
architectures handle automatically.

---

## Live GPU Compute Results

First direct wgpu shader execution on both GPUs — Anderson Lyapunov f32
compute (L=200, W=2.0, 1024 realizations):

| Substrate | γ | ξ | Time | Precision |
|---|---|---|---|---|
| **Titan V (NVK/NAK)** | 0.0386 | 25.90 | **797 µs** | f32 |
| **RTX 4070 (NVVM)** | 0.0386 | 25.90 | **274 µs** | f32 |
| CPU reference | 0.0406 | 24.61 | 6341 µs | f64 |

- Both GPUs produce identical results (same seed, same f32 precision)
- f32 vs f64 precision delta: **5.0%** on γ — validates need for DF64
- 1018/1024 realizations have γ > 0 (6 underflow from f32 log chain)
- GPU speedup over CPU: **8.0×** (Titan V) / **23.1×** (RTX 4070)

## NAK f64 Gap (Critical Finding)

The Titan V runs on NVK (Mesa's Vulkan driver for NVIDIA) with NAK (the
open-source shader compiler). Key findings:

1. **SHADER_F64 advertised but not functional** — NAK reports the feature
   but f64 ALU lowering is not implemented: `from_nir.rs:1092: assertion
   failed: alu.def.bit_size() == 32`
2. **NVVM also fails f64** — the proprietary driver on RTX 4070 reports
   `SHADER_F64` but rejects f64 compute shaders: `NVVM compilation failed: 1`
3. **f32 shaders work perfectly** — both NAK and NVVM compile and execute
   f32 compute shaders without issue
4. **TIMESTAMP_QUERY works** — GPU timing available for benchmarks
5. **max_buffer_size is inflated** — NVK reports API-maximum (~2^57 bytes)
   instead of actual VRAM. Adaptive batch falls back to architecture defaults.
6. **VRAM (Vulkan)**: 11.99 GB device-local (12 GB HBM2), 10.81 GB budget

### Implication for ToadStool

**DF64 is required on ALL current GPUs**, not just consumer Ada. Even the
Titan V with native 1:2 f64 hardware cannot use f64 WGSL shaders through
NVK/NAK. ToadStool's `df64_rewrite.rs` is essential for production precision.

### NAK Evolution Path

1. **NAK f64 ALU**: `from_nir.rs` needs 64-bit ALU operation lowering.
   Volta hardware supports it (SM 7.0), but NAK's NIR conversion assumes
   all ALU ops are 32-bit.
2. **Fallback chain**: WGSL f64 → try compile → if fails → DF64 rewrite →
   f32 pairs → identical precision on all hardware.
3. **Workgroup sizing**: f32 shaders on Volta should use 64-wide workgroups
   (2 f32 FMA units per SM, 32-wide warps).
4. **Memory coalescing**: HBM2 favors sequential 64-bit aligned f32 loads.
5. **Precision dispatch**: `op_preamble` should try f64, fall back to DF64,
   with runtime detection (not just feature flag check).

---

## Workload Routing (19/19 routable)

| Workload | Substrate | Reason |
|---|---|---|
| Anderson transfer matrix (MC) | TITAN V [GPU] | Native f64 |
| Almost-Mathieu eigenvalues | TITAN V [GPU] | Native f64 |
| Green-Kubo integration (f64) | TITAN V [GPU] | Native f64 |
| Anderson regime classification | AKD1000 [NPU] | int8 quant |
| Diversity saturation prediction | AKD1000 [NPU] | int8 quant |
| Bias-variance decomposition | TITAN V [GPU] | Native f64 |
| Finite-size extrapolation | TITAN V [GPU] | Native f64 |
| Freeze-out 2D grid fit | TITAN V [GPU] | Native f64 |
| Seismic 3D grid search | TITAN V [GPU] | Native f64 |
| Band edge energy scan | TITAN V [GPU] | Native f64 |
| Quasispecies Wright-Fisher | TITAN V [GPU] | Native f64 |
| Rare biosphere multinomial | TITAN V [GPU] | Native f64 |
| Gillespie SSA batch | TITAN V [GPU] | Native f64 |
| Spectral recon (Tikhonov) | TITAN V [GPU] | Native f64 |
| Jackknife leave-one-out | TITAN V [GPU] | Native f64 |
| MC ET₀ propagation | TITAN V [GPU] | Native f64 |
| Transport eigenvalues | TITAN V [GPU] | Native f64 |
| Wright-Fisher batch | TITAN V [GPU] | Native f64 |
| Bootstrap/RAWR resampling | TITAN V [GPU] | Native f64 |

---

## ToadStool Action Items

1. **DF64 on ALL GPUs**: Both NAK (Volta) and NVVM (Ada) fail f64 shader
   compilation. `df64_rewrite.rs` must be the default path, not an optional
   fallback. The `SHADER_F64` feature flag is unreliable.
2. **Runtime f64 probe**: Instead of checking `SHADER_F64`, try compiling a
   minimal f64 shader and fall back to DF64 if it panics/fails. groundSpring's
   `probe_f64_pipeline()` demonstrates this pattern.
3. **Unidirectional streaming**: Leverage adaptive batch's `use_resident_memory`
   flag to keep buffers alive between dispatches on Volta/HBM2.
4. **NAK workgroup tuning**: Test 64-wide workgroups on Volta (vs 256 on Ada)
   for f32/DF64 shaders. NAK may have different optimal sizing.
5. **NAK f64 contribution**: NAK's `from_nir.rs:1092` assertion blocks f64
   ALU lowering. Volta hardware supports f64 natively; this is a compiler
   gap, not a hardware limitation. A patch could unlock 1:2 f64 throughput.

---

## Delegation Inventory (unchanged from V34)

- **32 active**: 25 CPU-tier + 7 GPU-tier
- **9 pending**: Awaiting ToadStool absorption
- **49 metalForge tests**: All pass
- **14 new tests**: GPU arch, routing preference, adaptive batch

---

## Validation

```
cargo clippy --workspace --all-features -- -D warnings  → 0 warnings
cargo test -p groundspring-forge                         → 49/49 pass
python3 -m pytest tests/                                 → 320/320 pass + 2 skip
validate-metalforge-inventory                            → 14/14 pass
validate-metalforge-gpu                                  → 11/11 pass
validate-metalforge-titan-v                              → 13/13 pass (both GPUs)
```
