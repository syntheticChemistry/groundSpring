# Sovereign Compute Pipeline — Cross-Primal Handoff

**Date**: March 5, 2026
**From**: groundSpring V80b
**To**: barraCuda, toadStool, coralNAK teams
**Purpose**: Map the sovereign pipeline, identify gaps, accelerate coralNAK

---

## Current Pipeline (What Exists Today)

```
WGSL shader source
    │
    ▼
┌──────────────────────────────────────────────────────────────────┐
│  barraCuda — Sovereign Shader Pipeline                          │
│                                                                  │
│  op_preamble (F16/F32/F64/DF64 abstraction)                    │
│  → DF64 rewrite (infix → bridge functions, when Hybrid)        │
│  → ShaderTemplate (exp/log polyfills for broken drivers)        │
│  → WgslOptimizer (@ilp_region, @unroll_hint)                   │
│  → SovereignCompiler::compile()                                 │
│    → FMA fusion, dead expr elimination                          │
│    → naga validate                                              │
│    → SPIR-V emission                                            │
│  → wgpu SPIRV_SHADER_PASSTHROUGH (Vulkan only)                 │
└────────────────────────┬─────────────────────────────────────────┘
                         │
          ┌──────────────┼──────────────┐
          ▼              ▼              ▼
    ┌──────────┐  ┌──────────┐  ┌──────────┐
    │  NVIDIA  │  │   AMD    │  │  Intel   │
    │          │  │          │  │          │
    │ Propri-  │  │  RADV    │  │  ANV     │
    │ etary:   │  │  (ACO)   │  │          │
    │  PTXAS   │  │          │  │          │
    │          │  │          │  │          │
    │ Open:    │  │          │  │          │
    │  NVK     │  │          │  │          │
    │  (NAK)   │  │          │  │          │
    └──────────┘  └──────────┘  └──────────┘
```

### What works

| Vendor | Driver | Compiler | f64 | Status |
|--------|--------|----------|-----|--------|
| NVIDIA (compute) | Proprietary | PTXAS | Native | Works (RTX 4070 confirmed) |
| NVIDIA (compute) | NVK/nouveau | NAK | **Broken** — freezes, f64 gap | **System freeze on Titan V** |
| AMD | RADV | ACO | Native (CDNA2), DF64 (RDNA) | Untested locally |
| Intel | ANV | ANV | DF64 (Arc) | Untested locally |
| Software | llvmpipe | LLVM | f64 | Works (CI fallback) |

### Local hardware

| Device | PCI | Driver | f64 Rate | Fp64Strategy | Status |
|--------|-----|--------|----------|--------------|--------|
| RTX 4070 (Ada) | 01:00.0 | nvidia proprietary | 1:64 | Hybrid (DF64) | Active |
| Titan V (GV100) | 05:00.0 | **nouveau** | 1:2 (Full) | Native | **Frozen — nouveau compute crash** |
| i9-12900K | — | — | CPU | barracuda CPU | Active (51 delegations) |

---

## The Sovereignty Gap

### Problem: The Titan V is the most valuable GPU but can't be used

The Titan V has **full-rate f64** (1:2 FP64:FP32) — ideal for scientific compute.
But it's on nouveau, and NVK's NAK compiler:
1. **Cannot emit f64 transcendentals** (sin, cos, exp, log, sqrt) — MUFU is f32-only
2. **Compute dispatch crashes** nouveau, freezing the system
3. **Known workarounds** in barraCuda (`NvkExpF64Crash`, `NvkLogF64Crash`,
   `NvkSinCosF64Imprecise`) only help when NAK doesn't crash outright

### Pragmatic fix (today)

Bind Titan V to the proprietary nvidia driver (both GPUs on same driver).
This gives full f64 + DF64 hybrid on consumer GPU — but requires proprietary software.

### Sovereign fix (coralNAK)

```
barraCuda (future):  WGSL → naga → coral-nak → native binary → coralDriver → GPU
```

This eliminates NAK, NVK, and nouveau from the pipeline entirely.

---

## coralNAK Status and Critical Path

### Current state (Phase 2 complete)

- **183 tests**, 0 errors, `cargo check` clean
- NAK sources (72 files, 51K LOC) compile against Rust stubs
- 12 Mesa stub modules evolved to real implementations
- SM20–SM120 instruction encoders compiled
- ISA tables, latency models, SPH generation in place
- **Missing**: SPIR-V frontend, f64 lowering, userspace driver

### Phase roadmap

| Phase | Status | What it enables | Effort |
|-------|--------|-----------------|--------|
| 2 — Wire NAK | **Complete** | NAK compiles in pure Rust | Done |
| 3 — SPIR-V frontend | Not started | End-to-end shader compilation | Medium |
| 4 — f64 lowering | Not started | **Sovereign f64 transcendentals** | Medium |
| 5 — Standalone | In progress | Remove all Mesa deps | Low |
| 6 — coralDriver | Not started | Userspace GPU submission | High |

### Phase 3: SPIR-V frontend (critical path)

- Add `from_spirv.rs` — translate naga SPIR-V → coral-nak IR
- Wire into `compile()` (currently returns `NotImplemented`)
- End-to-end test: SPIR-V compute shader → native binary
- **Dependency**: naga 24 (already a dev-dependency)

### Phase 4: f64 lowering (the sovereignty-enabling phase)

Per `F64_LOWERING_THEORY.md`, lowering strategies using DFMA hardware:

| Function | Strategy | ULP budget |
|----------|----------|------------|
| `sqrt(f64)` | `MUFU.RSQ64H` seed + 2 Newton iterations via DFMA | ≤ 1 |
| `rcp(f64)` | `MUFU.RCP64H` seed + 2 Newton iterations via DFMA | ≤ 1 |
| `exp2(f64)` | Integer/fraction split + degree-6 minimax polynomial | ≤ 2 |
| `log2(f64)` | Exponent extraction + `MUFU.LOG2(f32)` + Newton refinement | ≤ 2 |
| `sin/cos(f64)` | Cody-Waite range reduction + degree-7 minimax polynomial | ≤ 4 |

### Phase 6: coralDriver (full sovereignty)

- Userspace GPU driver (no kernel-mode driver needed beyond basic DRM)
- Memory management (coralMem)
- Command buffer builder and submission (coralQueue)
- **groundSpring is assigned Level 4 work** per `SOVEREIGN_COMPUTE_EVOLUTION.md`

---

## What Each Primal Contributes

### barraCuda → coralNAK

| Asset | Location | How it helps |
|-------|----------|--------------|
| DF64 math implementations | `shaders/math/math_f64.wgsl` | Cody-Waite + minimax polynomials — direct source for Phase 4 coefficients |
| DF64 transcendentals | `shaders/math/df64_transcendentals.wgsl` | 15 f64 functions with Lanczos/Horner patterns |
| NAK workaround catalog | `device/driver_profile/workarounds.rs` | 5 documented NAK deficiencies to test against |
| NAK stress-test shaders | `batched_eigh_nak_optimized_f64.wgsl` | Encode loop unrolling, spills, scheduling, FMA fusion issues |
| f64 precision benchmarks | `bench_f64_builtins` | Validation targets for coralNAK f64 output |
| DF64 naga rewriter | `df64_rewrite.rs` | Patterns for NAK compound-assignment bugs |
| Driver profile system | `device/driver_profile/` | `GpuArch`, `Fp64Rate`, `Fp64Strategy` — coralNAK can reuse detection |

### toadStool → coralNAK

| Asset | Location | How it helps |
|-------|----------|--------------|
| GPU adapter discovery | `capabilities.rs` | Enumerate GPUs, detect f64 support, workgroup limits |
| `TOADSTOOL_GPU_ADAPTER` | `capabilities.rs` | Multi-GPU selection (coralNAK needs to target specific adapters) |
| NPU dispatch traits | `npu_dispatch.rs` | Vendor-agnostic neuromorphic interface for future NPU compilation |
| Latency models | `SOVEREIGN_COMPUTE.md` | SM70–SM89, RDNA2/3, Apple M, Intel Xe models |
| Backend strategy | `strategy.rs` | `SovereignOnly` mode that refuses proprietary paths |

### groundSpring → coralNAK

| Asset | Location | How it helps |
|-------|----------|--------------|
| Validation pipeline | 34 binaries, 395 checks | End-to-end validation targets for coralNAK-compiled shaders |
| f64 precision baselines | Python controls + Rust match | Ground truth for f64 transcendental accuracy |
| Mixed-hardware experience | metalForge | Real workloads spanning GPU + NPU + CPU |
| Level 4 assignment | SOVEREIGN_COMPUTE_EVOLUTION | coralDriver, coralMem, coralQueue implementation |
| Makkink bug discovery | V80b | Example of cross-validation catching constant typo (−0.012 → −0.12) |

---

## Multi-Vendor Expansion Roadmap

### Already in barraCuda

| Vendor | Detection | Precision | Workarounds | Tested |
|--------|-----------|-----------|-------------|--------|
| NVIDIA (proprietary) | `NvidiaProprietary` | Native (Volta+), DF64 (consumer) | Ada f64 transcendentals | Yes |
| NVIDIA (NVK) | `Nvk` | Native/DF64 | exp/log/sincos crash workarounds | Partially (freezes) |
| AMD (RADV) | `Radv` | Native (CDNA2), DF64 (RDNA2/3) | None documented | Untested |
| Intel (ANV) | `Intel` | DF64 (Arc) | None documented | Untested |
| Apple (Metal) | `AppleM` | Software f64 | N/A | Untested |
| Software | `Software` | f64 via LLVM | N/A | CI verified |

### coralNAK vendor expansion (future)

coralNAK currently targets **NVIDIA only** (SM20–SM120 encoders). To achieve
multi-vendor sovereignty:

1. **coral-aco** — AMD GPU compiler (port Mesa ACO to Rust)
2. **coral-anv** — Intel GPU compiler (port Mesa ANV to Rust)
3. **coral-metal** — Apple GPU compiler (would need Metal shader binary format)

This is a long-term roadmap. In the near term, AMD and Intel sovereign compute
works through wgpu → RADV/ANV (open-source Mesa drivers) — which is already
sovereign on those platforms since the drivers are open source.

### NPU support (toadStool + barraCuda)

| NPU | Driver | Interface | Status |
|-----|--------|-----------|--------|
| BrainChip Akida (AKD1000) | akida-driver (VFIO/kernel/userspace) | `NpuDispatch` trait | Implemented, no hardware present |
| Intel Loihi | — | `NpuDispatch` trait (planned) | Design target |
| SpiNNaker | — | `NpuDispatch` trait (planned) | Design target |

---

## Recommended Next Steps

### Immediate (this week)

1. **Bind Titan V to nvidia proprietary driver** — enables native f64 GPU compute
   on the most capable hardware. `barracuda-gpu` feature becomes safe to use.
   ```bash
   # Option A: Blacklist nouveau for GV100 via PCI ID
   # Option B: Use NVIDIA persistence daemon for both GPUs
   ```

2. **Run `cargo test --features barracuda-gpu`** — validate all 812 tests on
   real GPU hardware (RTX 4070 DF64 path).

### Short-term (coralNAK acceleration)

3. **Phase 3: SPIR-V frontend** — implement `from_spirv.rs` using naga's SPIR-V
   module. This is the gate for everything else. barraCuda's `spv_emit` tests
   provide SPIR-V payloads to test against.

4. **Phase 4: f64 lowering** — port barraCuda's `math_f64.wgsl` polynomial
   coefficients into coralNAK's instruction emitter. Start with `sqrt` and `rcp`
   (simplest — just MUFU seed + Newton iterations).

### Medium-term (sovereign pipeline)

5. **Integration test**: `WGSL → naga → SPIR-V → coral-nak → binary`, compare
   output against `WGSL → naga → SPIR-V → NVK/NAK → binary` on the same shader.

6. **Benchmark**: Compare coralNAK-compiled f64 transcendentals against
   NVIDIA proprietary PTXAS output for ULP accuracy and throughput.

7. **coralDriver prototype**: Minimal userspace GPU submission for Volta
   (GV100). groundSpring's Level 4 assignment.

### Long-term (full sovereignty)

8. **Multi-GPU dispatch via coralNAK**: Titan V (native f64 via coralNAK) +
   RTX 4070 (DF64 via coralNAK) — no proprietary drivers needed.

9. **AMD/Intel**: Open-source drivers (RADV/ANV) already provide sovereignty.
   coral-aco and coral-anv are future expansion points.

10. **NPU integration**: When Akida hardware is available, wire
    `NpuDispatch` through metalForge workloads for GPU→NPU pipeline testing.

---

## Quality Gates

All changes must pass:
- `cargo fmt --check`: PASS
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS
- `cargo test --workspace`: PASS
- Zero TODO/FIXME/unsafe/unwrap in production
- All files < 1000 lines
- Cross-spring provenance documented

---

## Dependencies

```
groundSpring ──→ barraCuda (path dep, CPU + GPU math)
               └→ toadStool/akida-driver (NPU, optional)

barraCuda ──→ wgpu 28 (GPU runtime)
           └→ naga 28 (shader parsing/emission)
           └→ sourDough (primal lifecycle)

toadStool ──→ wgpu 28 (GPU discovery)
           └→ akida-driver (NPU)
           └→ sourDough (primal lifecycle)

coralNAK ──→ naga 24 (dev-dep, SPIR-V frontend)
           └→ sourDough (primal lifecycle)
           └→ (no wgpu — generates native binaries directly)

sourDough ──→ (scaffold/lifecycle — symlinked from phase2/)
```
