# Sovereign Compute Pipeline — Cross-Primal Handoff

**Date**: March 5, 2026
**From**: groundSpring V80b
**To**: barraCuda, toadStool, coralReef teams
**Purpose**: Map the sovereign pipeline, identify gaps, accelerate coralReef

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

### Philosophy: Eliminate friction, not teams

The goal is not to replace Mesa's compiler teams — it is to give them a better
foundation. NAK, ACO, and ANV developers are ISA experts. They know scheduling,
register allocation, and instruction encoding deeply. What they spend too much
time on is C build systems, memory bugs, vendor detection boilerplate, and
precision guesswork.

coralReef provides a pure-Rust compiler stack with:
- **Working IR, optimizer, and register allocator** — the infrastructure is done
- **Validated precision** — 11,161+ checks across 70+ papers, documented ULP budgets
- **DF64 three-tier precision** — every card is useful, no hardware left idle
- **Entire classes of bugs eliminated** — Rust ownership, no C FFI, no memory corruption
- **AGPL-3.0** — stays open, nobody forks it proprietary

The pitch to ISA experts: adopt Rust and these scientific standards, and you
spend all your time on the fun math and optimizations, none on whether you
have the right hardware or the right build system. coralReef handles the
vendor-agnostic IR, precision strategy, and validation pipeline. You bring
ISA knowledge and make your backend sing.

The architecture makes this natural: vendor-agnostic optimization passes
are shared, and only the final instruction encoding is backend-specific.
An AMD developer adds RDNA/CDNA encoding tables. An Intel developer adds
Xe EU encoding. They reuse everything else.

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

### Sovereign fix (coralReef)

```
barraCuda (future):  WGSL → naga → coral-reef → native binary → coralDriver → GPU
```

This eliminates NAK, NVK, and nouveau from the pipeline entirely.

---

## coralReef Status (Phases 1–5 Complete)

### Current state (as of March 5, 2026 — commit 2e89541)

- **390 tests**, 0 failures, 2 ignored
- **Phases 1–5 all complete** — sovereign Rust compiler, no Mesa C dependencies
- **Rename complete**: `coralNAK` → `coralReef` across all crates, dirs, and docs
- `cargo check`, `cargo clippy -D warnings`, `cargo fmt --check`, `cargo doc` — all PASS
- 37.1% line / 44.9% function coverage (structural floor from encoder match arms)
- 0 files > 1000 LOC (all oversized files smart-refactored by algorithm/concern)

### Spring absorption (commit 567e3e6)

coralReef has been actively absorbing patterns from ecoPrimals springs:

| Pattern | Source | Applied in |
|---------|--------|------------|
| `HashMap` → `BTreeMap` (deterministic serialization) | groundSpring V73 tolerance arch | `health.rs` |
| `unsafe unwrap_unchecked` → safe `unwrap()` | groundSpring CONTRIBUTING | `builder/mod.rs` |
| Silent-default `unwrap_or(0)` → direct cast | groundSpring V76 "silent defaults are bugs" | `program.rs` |
| Hardcoded `/tmp/` → `std::env::temp_dir()` | ecoPrimals sovereignty standard | `opt_instr_sched_common.rs` |
| Cross-spring provenance doc-comments | CROSS_SPRING_SHADER_EVOLUTION | `lower_f64/` |

### Smart refactoring (commit 2e89541)

Three files at the 1000 LOC ceiling split by algorithm/concern (not mechanical):

| Before | After | Strategy |
|--------|-------|----------|
| `poly.rs` (998 LOC) | `poly/{mod,exp2,log2,trig}.rs` (129+182+142+349) | Split by algorithm family |
| `opt_copy_prop.rs` (998 LOC) | `opt_copy_prop/{mod,types,tests}.rs` (688+51+264) | Extracted types + tests |
| `func.rs` (986 LOC) | `func.rs` (590) + `func_builtins.rs` (163) + `func_math.rs` (247) | Extracted builtin resolution + math translation |

Panic evolution in IR types:
- `StCacheOp::select` store-to-constant `panic!` → `debug_assert` + safe fallback (`WriteThrough`)
- `AtomType::F/U/I` constructors `panic!` → `Option<AtomType>` (callers handle `None`)

### Phase completion summary

| Phase | Status | What was delivered |
|-------|--------|-------------------|
| 1 — Scaffold | **Complete** | NAK extracted (72 files), stub crate, ISA crate |
| 1.5 — Foundation | **Complete** | UniBin, JSON-RPC 2.0 IPC, AGPL-3.0, capability discovery |
| 2 — Wire NAK | **Complete** | Full pipeline compiles in pure Rust |
| 2.5–2.9 — Refactor | **Complete** | All files split < 1000 LOC, 8.7K dead code removed |
| 3 — naga Frontend | **Complete** | `from_spirv/` (2,530 LOC): WGSL/SPIR-V → NAK SSA IR |
| 3.5 — Deep Debt | **Complete** | 4 legacy FFI stubs deleted, Result propagation, zero-copy IPC |
| 4 — f64 Lowering | **Complete** | `lower_f64/` (1,557 LOC): all 6 transcendentals via DFMA |
| 4.5 — Error Safety | **Complete** | Pipeline fully fallible, 36 panics evolved to Result |
| 5 — Standalone | **Complete** | All 7 stub modules evolved to pure Rust (including QMD for Kepler–Blackwell) |

### f64 transcendental precision (delivered)

| Function | Strategy | Precision |
|----------|----------|-----------|
| `sqrt(f64)` | `MUFU.RSQ64H` seed + 2 Newton-Raphson via DFMA | Full f64 |
| `rcp(f64)` | `MUFU.RCP64H` seed + 2 Newton-Raphson via DFMA | Full f64 |
| `exp2(f64)` | Range reduction + degree-6 Horner polynomial + ldexp | Full f64 |
| `log2(f64)` | `MUFU.LOG2` seed + Newton refinement (EX2/RCP correction) | ~46-bit |
| `sin(f64)` | Cody-Waite reduction + minimax polynomial + quadrant correction | Full domain |
| `cos(f64)` | Cody-Waite reduction + minimax polynomial + quadrant correction | Full domain |

### End-to-end pipeline (now working)

```
WGSL source → parse_wgsl() → naga Module
    → from_spirv::translate() → NAK SSA IR (Shader)
    → opt_copy_prop → opt_dce → opt_crs → opt_lop → opt_prmt
    → opt_out → opt_jump_thread → opt_bar_prop → opt_uniform_instrs
    → opt_instr_sched_prepass
    → lower_f64_transcendentals (F64Sqrt → MUFU+Newton, etc.)
    → legalize() → assign_regs() → lower_par_copies → lower_copy_swap
    → assign_deps_serial → opt_instr_sched_postpass
    → gather_info() → encode_shader()
    → CompiledShader { header, code }
```

### What remains

| Phase | Status | Description |
|-------|--------|-------------|
| 6 — coralDriver | Not started | Userspace GPU submission (DRM ioctl + memory mapping) |
| 7 — coralGpu | Not started | Unified Rust GPU abstraction replacing wgpu |
| Precision refinement | Future | log2 second Newton iteration (→ full f64), exp2 subnormal handling |
| Vendor-agnostic IR | Future | Extract `coral-ir` for AMD/Intel/NPU backends (see below) |
| naga 24 → 28 upgrade | Future | Evaluate for breaking changes |
| barraCuda integration | Future | Replace `wgpu SPIR-V passthrough` with `coral-nak → coralDriver` |

**groundSpring is assigned Level 4 work** (coralDriver, coralMem, coralQueue)
per `SOVEREIGN_COMPUTE_EVOLUTION.md`.

---

## What Each Primal Contributes

### barraCuda → coralReef

coralReef has **already absorbed** the f64 polynomial strategies from Phase 4
guidance. Remaining assets to share:

| Asset | Location | Status | How it helps |
|-------|----------|--------|--------------|
| DF64 math implementations | `shaders/math/math_f64.wgsl` | **Absorbed** — polynomial coefficients now in `lower_f64/poly.rs` | Cody-Waite + minimax already ported |
| DF64 transcendentals | `shaders/math/df64_transcendentals.wgsl` | Partially absorbed | 15 f64 functions — coralReef covers 6, remaining useful for DF64-as-utilization-tier |
| NAK workaround catalog | `device/driver_profile/workarounds.rs` | Still relevant | 5 documented NAK deficiencies — coralReef should verify it doesn't reproduce them |
| NAK stress-test shaders | `batched_eigh_nak_optimized_f64.wgsl` | Still relevant | Test encoding: loop unrolling, spills, scheduling, FMA fusion |
| f64 precision benchmarks | `bench_f64_builtins` | Still relevant | Validation targets for coralReef compiled f64 output vs PTXAS |
| DF64 naga rewriter | `df64_rewrite.rs` | **Absorbed for NVIDIA path** | Pattern knowledge reusable for AMD/Intel DF64 backends |
| Driver profile system | `device/driver_profile/` | Still relevant | `Fp64Rate`, `Fp64Strategy` → coralReef should adopt three-tier precision model |
| DF64 utilization strategy | `Fp64Strategy::Concurrent` | **Key for all backends** | Three-tier precision (f32/DF64/f64) as hardware utilization strategy |

### toadStool → coralReef

| Asset | Location | How it helps |
|-------|----------|--------------|
| GPU adapter discovery | `capabilities.rs` | Enumerate GPUs, detect f64 support, workgroup limits |
| `TOADSTOOL_GPU_ADAPTER` | `capabilities.rs` | Multi-GPU selection (coralReef needs to target specific adapters) |
| NPU dispatch traits | `npu_dispatch.rs` | Vendor-agnostic neuromorphic interface for future NPU compilation |
| Latency models | `SOVEREIGN_COMPUTE.md` | SM70–SM89, RDNA2/3, Apple M, Intel Xe models |
| Backend strategy | `strategy.rs` | `SovereignOnly` mode that refuses proprietary paths |

### groundSpring → coralReef

| Asset | Location | How it helps |
|-------|----------|--------------|
| Validation pipeline | 34 binaries, 395 checks | End-to-end validation targets for coralReef-compiled shaders |
| f64 precision baselines | Python controls + Rust match | Ground truth for f64 transcendental accuracy |
| Mixed-hardware experience | metalForge | Real workloads spanning GPU + NPU + CPU |
| Level 4 assignment | SOVEREIGN_COMPUTE_EVOLUTION | coralDriver, coralMem, coralQueue implementation |
| Makkink bug discovery | V80b | Example of cross-validation catching constant typo (−0.012 → −0.12) |

---

## coralReef Architecture Evolution — Eliminating the Vendor Concept

### Current state: NVIDIA-only IR

Today coralReef's IR is NVIDIA-specific:
- `GpuArch` = SM70/75/80/86/89 (NVIDIA shader models only)
- `MuFuOp` = NVIDIA-specific transcendental unit (Sin, Cos, Rcp64H...)
- `RegFile` = NVIDIA register files (GPR, UGPR, Pred, Carry, Bar)
- Encoders: `sm20/`, `sm32/`, `sm50/`, `sm70_encode/` — all NVIDIA instruction formats

### Target state: Vendor-agnostic foundation for ISA experts

The goal is **one IR, shared optimizations, pluggable backends** — so that ISA
experts from any vendor can contribute encoding tables and scheduling heuristics
without rebuilding the entire compiler stack. The architecture mirrors LLVM's
model (shared IR + pluggable backends) but for GPU compute:

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

#### Phase B: AMD backend — contribution surface for ACO developers

AMD GPUs use a different ISA. An ACO developer already knows this table — what
coralReef gives them is a working IR/optimizer/RA to plug their knowledge into:

| NVIDIA (done) | AMD (contribution target) | Notes |
|--------|-----|-------|
| SASS (SM70-120) | GCN / RDNA / CDNA ISA | Different instruction encoding |
| MUFU (f32 transcendentals) | `v_rcp_f32`, `v_sqrt_f32` | Similar SFU but different encoding |
| DFMA (f64 FMA) | `v_fma_f64` | Native on CDNA2; 1:16 on RDNA3 |
| Warp size = 32 | Wave size = 32 or 64 | AMD supports both (wave32, wave64) |
| GPR + UGPR | VGPR + SGPR | AMD uses scalar + vector split |

What an ACO contributor brings: encoding tables, wave32/64 scheduling, VGPR/SGPR
allocation strategy. What coralReef already provides: IR, copy propagation, DCE,
f64 lowering framework, DF64 three-tier precision, validation pipeline.

#### Phase C: Intel backend — contribution surface for ANV developers

Same pattern. Intel Xe EU ISA is the encoding target:

| NVIDIA (done) | Intel (contribution target) | Notes |
|--------|-------|-------|
| SASS | EU ISA | Register-based, different encoding |
| Warp = 32 | SIMD8/16/32 | Variable SIMD width |
| GPR (255) | GRF (128 × 256-bit) | Larger register file, different layout |
| f64 1:2 (Volta) | f64 minimal | Consumer Intel has very limited f64 — DF64 is the primary precision path |

What an ANV contributor brings: EU encoding, SIMD width selection, GRF layout.
What they get for free: everything above the backend.

### DF64: Utilization strategy, not workaround

DF64 (double-float f32-pair arithmetic, ~48-bit mantissa) is **not** a
workaround for hardware that lacks f64. It is a **core utilization strategy**:

- **Every GPU has massive f32 core counts.** Most sit idle during f64 workloads
  because native f64 units are scarce (1:2 at best on Volta/A100, 1:64 on
  consumer NVIDIA, 1:16 on RDNA3).
- **Most scientific compute needs more than f32 (24-bit) but not full f64
  (53-bit).** DF64's ~48-bit mantissa covers the vast majority of real-world
  precision requirements: climate models, correlation, hydrology, population
  genetics.
- **DF64 turns idle f32 cores into a precision pool.** On an RTX 4070 with
  1:64 f64 rate, DF64 gets ~32x more throughput than native f64 while
  delivering ~48-bit precision. On Volta with 1:2 f64, DF64 still provides a
  second parallel pool of f32 cores doing useful precision work.
- **Idle cores on a card is waste card.** DF64 is how you stop wasting them.

This creates three precision tiers available on **all** hardware:

| Tier | Precision | Source | Use case |
|------|-----------|--------|----------|
| f32 | ~24-bit mantissa | Native f32 cores | Visualization, inference, throughput workloads |
| DF64 ("fp48") | ~48-bit mantissa | f32 core pairs (idle capacity) | Most scientific compute, statistics, correlation |
| f64 | ~53-bit mantissa | Native f64 units (scarce) | Reference validation, accumulation, edge cases requiring full IEEE 754 |

### f64 lowering per vendor

The choice between f64 and DF64 is **precision vs utilization**, not
"has f64 vs doesn't":

| Vendor | Native f64 | DF64 opportunity | Recommended strategy |
|--------|-----------|-----------------|---------------------|
| NVIDIA (Volta/A100) | DFMA at 1:2 rate | Massive idle f32 pool | **Concurrent**: f64 for accumulators, DF64 for bulk math |
| NVIDIA (consumer) | DFMA at 1:64 rate | ~32x throughput gain | **Hybrid**: DF64 for everything except final accumulation |
| AMD (CDNA2/3) | `v_fma_f64` full-rate | f32 cores still available | **Concurrent**: both pools active |
| AMD (RDNA2/3) | `v_fma_f64` at 1:16 | ~8x throughput gain | **Hybrid**: DF64 for bulk, f64 for precision-critical |
| Intel (Xe) | Minimal f64 | Only precision path | **DF64-primary**: f64 reserved for validation |

barraCuda's `Fp64Strategy` (Native/Hybrid/Concurrent) maps directly to these
categories. coralReef should adopt the same three-tier model for its backends,
and every backend should support DF64 regardless of native f64 capability.

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

### Short-term (coralReef evolution)

3. **~~Phase 3: SPIR-V frontend~~** — **COMPLETE**. `from_spirv/` now split into
   `func.rs` (590 LOC) + `func_builtins.rs` (163) + `func_math.rs` (247).
   Handles: GlobalInvocationId, LocalInvocationId, WorkGroupId, NumWorkGroups,
   SubgroupInvocationId, LocalInvocationIndex. Math: abs, min, max, clamp,
   floor, ceil, round, sqrt, inverseSqrt, sin, cos, exp2, log2, fma.

4. **~~Phase 4: f64 lowering~~** — **COMPLETE**. `lower_f64/` split into
   `poly/{exp2,log2,trig}.rs` + `newton.rs`. All 6 transcendentals implemented.
   Coefficients from Cephes/FDLIBM, Cody-Waite for sin/cos.

5. **log2 precision refinement** — currently ~46-bit. Second Newton-Raphson
   iteration would bring it to full f64 (~52-bit). Straightforward: add one
   more EX2/RCP correction pass in `poly/log2.rs`.

6. **Naga 24 → 28 upgrade** — coralReef is on naga 24, barraCuda/wgpu on 28.
   Alignment needed to prevent IR mismatch when sharing SPIR-V payloads.

### Medium-term (sovereign pipeline)

7. **Titan V as coralReef proving ground**: The Titan V (SM70, Volta) is the
   ideal first target for coralReef's sovereign pipeline. It has native f64
   (1:2 ratio), is coralReef's reference platform (default `GpuArch::Sm70`),
   and currently causes system freezes under NVK/nouveau when doing GPU compute.
   coralReef + coralDriver would bypass NVK entirely, solving the driver issue
   while proving sovereignty. groundSpring's Anderson spectral analysis (Exp 008,
   009) and WDM transport (Exp 025-027) are ideal validation workloads — they
   demand f64 precision and are already validated on CPU.

8. **RTX 4070 as DF64 utilization target**: The RTX 4070 (SM89, Ada) has
   consumer f64 (1:32 ratio) but abundant idle f32 cores. coralReef's DF64
   lowering (exp2, log2, sin, cos via DFMA) would give ~48-bit precision
   at near-f32 throughput. groundSpring's 34 experiments span a precision
   spectrum — some need full f64 (Anderson eigensolvers), many thrive at
   DF64/fp48 (bootstrap, correlation, regression). The hardware pair proves
   the utilization strategy: Titan V for f64-native, RTX 4070 for DF64,
   same WGSL source for both.

9. **Integration test**: `WGSL → naga → coral-reef → binary`, compare
   output against `WGSL → naga → SPIR-V → NVK/NAK → binary` on the same shader.
   coralReef's end-to-end pipeline is wired — needs real shader validation.

10. **Benchmark**: Compare coralReef-compiled f64 transcendentals against
    NVIDIA proprietary PTXAS output for ULP accuracy and throughput.

11. **coralDriver prototype**: Minimal userspace GPU submission for Volta
    (GV100). Once coralDriver exists, groundSpring can validate the full
    sovereign path: WGSL → barraCuda → coralReef → coralDriver → Titan V
    — zero C, zero vendor, zero Mesa.

### Long-term (full sovereignty — vendor-free)

12. **Multi-GPU dispatch via coralReef**: Titan V (native f64 via coralReef) +
    RTX 4070 (DF64 via coralReef) — no proprietary drivers needed.

13. **Vendor-agnostic IR** (`coral-ir`): Extract vendor-neutral IR from NAK IR,
    making optimization passes (copy prop, DCE, scheduling) work for any GPU.
    See "Eliminating the Vendor Concept" section above.

14. **AMD backend**: Add RDNA/CDNA instruction encoding and register allocation
    to coralReef (not a separate project — a backend module within coralReef).
    Mesa ACO (MIT-licensed) is the reference implementation.

15. **Intel backend**: Add Xe EU ISA encoding. Mesa ANV/iris is the reference.
    Both AMD and Intel backends share the same `coral-ir` and optimization
    passes — only legalization, register allocation, and encoding differ.

16. **NPU backend**: When Akida hardware is available, wire
    `NpuDispatch` through metalForge workloads for GPU→NPU pipeline testing.
    NPU is a different compute paradigm (event-driven vs SIMD) but the
    vendor-agnostic IR makes it a natural extension.

17. **coralDriver per architecture**: Userspace GPU drivers for each ISA family,
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

coralReef ──→ naga 24 (dev-dep, SPIR-V frontend — needs upgrade to 28)
           └→ sourDough (primal lifecycle)
           └→ (no wgpu — generates native binaries directly)

sourDough ──→ (scaffold/lifecycle — symlinked from phase2/)
```
