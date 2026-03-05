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

### Philosophy: Eliminate the concept of vendors

The long-term vision is not "support multiple vendors" — it is to make the
concept of GPU vendors irrelevant to the compilation pipeline. Today, every GPU
goes through a vendor-specific compiler (PTXAS for NVIDIA, ACO for AMD, ANV for
Intel). coralNAK aims to replace **all** of them with a single sovereign Rust
compiler that has pluggable ISA backends. The IR, optimizations, f64 lowering
strategy selection, and scheduling are vendor-agnostic. Only the final
instruction encoding step knows which hardware it targets — and even that is a
table-driven process, not a separate codebase.

This also means coralNAK replaces the Mesa C build system and its C dependencies
entirely — not just for NVIDIA, but for AMD and Intel as well. Mesa's ACO, ANV,
and NAK are reference implementations to learn from, but the goal is a single
pure-Rust compiler that makes all three obsolete for compute workloads.

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

## coralNAK Architecture Evolution — Eliminating the Vendor Concept

### Current state: NVIDIA-only IR

Today coralNAK's IR is NVIDIA-specific:
- `GpuArch` = SM70/75/80/86/89 (NVIDIA shader models only)
- `MuFuOp` = NVIDIA-specific transcendental unit (Sin, Cos, Rcp64H...)
- `RegFile` = NVIDIA register files (GPR, UGPR, Pred, Carry, Bar)
- Encoders: `sm20/`, `sm32/`, `sm50/`, `sm70_encode/` — all NVIDIA instruction formats

### Target state: Vendor-agnostic compilation

The goal is **one compiler, one IR, any GPU** — eliminating the concept of
vendors from the compilation pipeline entirely. The architecture should mirror
LLVM's approach (one IR, pluggable backends) but for GPU compute:

```
WGSL / SPIR-V
     │
     ▼
┌────────────────────────────────────────────────┐
│  coral-ir  (vendor-agnostic intermediate)      │
│                                                │
│  Ops: FAdd, FMul, FMA, Load, Store, Barrier,  │
│       Reduce, Broadcast, Branch, Call, ...     │
│  Types: f16, f32, f64, i32, i64, vec2-4, ...  │
│  Annotations: workgroup_size, shared_mem, ...  │
│                                                │
│  Passes (vendor-agnostic):                     │
│    opt_copy_prop, opt_dce, opt_lop,            │
│    opt_jump_thread, constant_folding,          │
│    opt_instr_sched_prepass                     │
└──────────────────────┬─────────────────────────┘
                       │
     ┌─────────────────┼─────────────────┐
     ▼                 ▼                 ▼
┌──────────┐    ┌──────────┐    ┌──────────┐
│ nvidia/  │    │  amd/    │    │ intel/   │
│          │    │          │    │          │
│ legalize │    │ legalize │    │ legalize │
│ f64_lower│    │ f64_lower│    │ f64_lower│
│ alloc_reg│    │ alloc_reg│    │ alloc_reg│
│ encode   │    │ encode   │    │ encode   │
│          │    │          │    │          │
│ SM70-120 │    │ RDNA2/3  │    │ Xe       │
│ SASS     │    │ CDNA2/3  │    │ EU ISA   │
└──────────┘    └──────────┘    └──────────┘
```

### What needs to happen

#### Phase A: Extract vendor-agnostic IR from NAK IR

The current NAK IR (`nak/ir/`) already has vendor-agnostic concepts buried
inside vendor-specific types. The refactoring path:

1. **Split `GpuArch`** into a trait:
   ```rust
   pub trait GpuTarget {
       fn has_native_f64(&self) -> bool;
       fn max_regs(&self) -> u32;
       fn max_shared_mem(&self) -> u32;
       fn warp_size(&self) -> u32;     // 32 for NVIDIA, 32/64 for AMD, varies for Intel
       fn f64_lowering(&self) -> F64Strategy;
   }
   ```
   Current `GpuArch` (SM70-89) becomes one implementation. AMD/Intel add others.

2. **Generalize `MuFuOp`** — NVIDIA's MUFU is vendor-specific. The IR should
   express intent (`Transcendental::Sin(f64)`) and the backend lowers to:
   - NVIDIA: MUFU.RSQ64H + Newton iterations
   - AMD: `v_sqrt_f64` (native on CDNA2) or polynomial on RDNA
   - Intel: implementation-specific

3. **Generalize `RegFile`** — NVIDIA's register model (GPR/UGPR/Pred) differs
   from AMD's (VGPR/SGPR/VCC) and Intel's (GRF). The vendor-agnostic IR uses
   virtual registers; the backend assigns to physical register files.

#### Phase B: AMD backend

AMD GPUs use a different ISA than NVIDIA:

| NVIDIA | AMD | Notes |
|--------|-----|-------|
| SASS (SM70-120) | GCN / RDNA / CDNA ISA | Different instruction encoding |
| MUFU (f32 transcendentals) | `v_rcp_f32`, `v_sqrt_f32` | Similar SFU but different encoding |
| DFMA (f64 FMA) | `v_fma_f64` | Native on CDNA2; 1:16 on RDNA3 |
| Warp size = 32 | Wave size = 32 or 64 | AMD supports both (wave32, wave64) |
| GPR + UGPR | VGPR + SGPR | AMD uses scalar + vector split |

Resources for AMD backend:
- Mesa ACO compiler (MIT/open-source) — the AMD equivalent of NAK
- LLVM AMDGPU backend — reference for instruction selection
- AMD ISA documentation (publicly available for GCN, RDNA, CDNA)

#### Phase C: Intel backend

Intel Xe GPUs use EU (Execution Unit) ISA:

| NVIDIA | Intel | Notes |
|--------|-------|-------|
| SASS | EU ISA | Register-based, different encoding |
| Warp = 32 | SIMD8/16/32 | Variable SIMD width |
| GPR (255) | GRF (128 × 256-bit) | Larger register file, different layout |
| f64 1:2 (Volta) | f64 minimal | Consumer Intel has very limited f64 |

Resources:
- Mesa ANV/iris compiler (open-source)
- Intel GPU ISA documentation (publicly available for Xe)

### f64 lowering per vendor

The f64 software lowering strategy differs by hardware:

| Vendor | Hardware f64 | Software lowering needed |
|--------|-------------|------------------------|
| NVIDIA (Volta/A100) | Native DFMA, MUFU 64H seeds | Only transcendentals (sin, cos, exp, log) |
| NVIDIA (consumer) | DFMA at 1:64 rate | Everything benefits from DF64 |
| AMD (CDNA2/3) | Native `v_fma_f64` full-rate | Only transcendentals |
| AMD (RDNA2/3) | `v_fma_f64` at 1:16 | DF64 beneficial for throughput |
| Intel (Xe) | Minimal f64 | Full DF64 or software lowering |

barraCuda's `Fp64Strategy` (Native/Hybrid/Concurrent) maps directly to these
categories. coralNAK should adopt the same classification for its backends.

### NPU as another "backend" (long-term)

The vendor-agnostic IR can eventually target NPU hardware:

| NPU | Interface | Compiler path |
|-----|-----------|---------------|
| BrainChip Akida | `NpuDispatch` (toadStool) | IR → sparse inference graph → Akida bitstream |
| Intel Loihi | `NpuDispatch` (planned) | IR → spiking network graph → Loihi config |

This is a longer-term evolution — NPU compilation is fundamentally different
from GPU (event-driven vs SIMD) — but the vendor-agnostic IR makes it possible.

### Practical guidance: How to build vendor-agnostic without blocking NVIDIA

The refactoring to vendor-agnostic IR does **not** need to block NVIDIA progress.
The recommended approach:

1. **Finish NVIDIA end-to-end first** (Phase 3 + 4) — get SPIR-V → SASS binary
   working with f64 transcendentals. This proves the pipeline works.

2. **Retrospective extraction**: Once NVIDIA works, identify which IR types
   and optimization passes are truly NVIDIA-specific vs vendor-agnostic. In
   practice, most of the NAK IR (op types, control flow, SSA form) is already
   generic — the NVIDIA-specific parts are:
   - `MuFuOp` (NVIDIA SFU opcodes)
   - `sm*_encode/` (instruction binary format)
   - `sm*_instr_latencies` (scheduling tables)
   - `sph.rs` (Shader Program Header — NVIDIA format)
   - Register file names (GPR/UGPR vs VGPR/SGPR)

3. **Introduce `coral-ir` crate**: Move generic types to a new crate. The
   NVIDIA backend re-exports or wraps them. New backends (AMD, Intel) import
   `coral-ir` directly.

4. **AMD backend first** (after NVIDIA): ACO is well-documented and AMD ISA
   docs are public. GCN/RDNA instruction encoding is simpler than SASS in some
   ways (fixed-width, less scheduling complexity). Mesa's ACO is MIT-licensed
   and can be studied freely.

5. **Key abstraction boundaries** to plan for now (even during NVIDIA work):
   - `fn compile(ir: &CoralIr, target: &dyn GpuTarget) -> Vec<u8>` — the
     top-level API should accept a target trait, not a concrete arch enum
   - `fn lower_f64(op: &Transcendental, target: &dyn GpuTarget) -> Vec<Instr>`
     — f64 lowering is parameterized by hardware capabilities
   - `fn encode(instrs: &[Instr], target: &dyn GpuTarget) -> Vec<u8>` — final
     encoding is the only truly vendor-specific step

This way the NVIDIA path continues at full speed while the architecture
naturally evolves toward multi-vendor support.

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

### Long-term (full sovereignty — vendor-free)

8. **Multi-GPU dispatch via coralNAK**: Titan V (native f64 via coralNAK) +
   RTX 4070 (DF64 via coralNAK) — no proprietary drivers needed.

9. **Vendor-agnostic IR** (`coral-ir`): Extract vendor-neutral IR from NAK IR,
   making optimization passes (copy prop, DCE, scheduling) work for any GPU.
   See "Eliminating the Vendor Concept" section above.

10. **AMD backend**: Add RDNA/CDNA instruction encoding and register allocation
    to coralNAK (not a separate project — a backend module within coralNAK).
    Mesa ACO (MIT-licensed) is the reference implementation.

11. **Intel backend**: Add Xe EU ISA encoding. Mesa ANV/iris is the reference.
    Both AMD and Intel backends share the same `coral-ir` and optimization
    passes — only legalization, register allocation, and encoding differ.

12. **NPU backend**: When Akida hardware is available, wire
    `NpuDispatch` through metalForge workloads for GPU→NPU pipeline testing.
    NPU is a different compute paradigm (event-driven vs SIMD) but the
    vendor-agnostic IR makes it a natural extension.

13. **coralDriver per architecture**: Userspace GPU drivers for each ISA family,
    eliminating Mesa/kernel driver dependencies entirely. The goal is a single
    Rust binary that compiles and dispatches to any GPU without C dependencies.

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
