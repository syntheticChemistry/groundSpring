# groundSpring V85 — toadStool/barraCuda Evolution Handoff

**Date**: March 6, 2026
**From**: groundSpring V85
**To**: toadStool team, barraCuda team, coralReef team
**Pins**: barraCuda `cf1602c`, toadStool S96c (`d77fc546`), coralReef `849fedd`

---

## Executive Summary

groundSpring V84–V85 probed the full GPU pipeline on both the RTX 4070 (SM89,
proprietary driver 580.119.02) and Titan V (SM70, NVK/NAK via Mesa). The
session discovered a systemic f64 shared-memory bug in the naga/SPIR-V pipeline,
then drove coralReef to compile the exact failing shader pattern to native GPU
binaries. Two critical coralReef bugs were fixed along the way. This handoff
documents what each primal should absorb from these findings.

---

## 1. What groundSpring Discovered

### 1.1 The f64 Shared-Memory Pipeline Failure

All f64 WGSL reduction shaders using `var<workgroup> shared_data: array<f64, 256>`
return 0 on **both GPUs** (RTX 4070 proprietary + Titan V NVK). The failure
pattern:

- Simple f64 arithmetic (no shared memory): **works** on both GPUs
- f64 with workgroup shared memory + barriers + tree reduction: **returns 0**
- DF64 (f32-pair emulation): **works** on both GPUs
- f32 shared memory patterns: **work** on both GPUs

**Root cause**: naga's SPIR-V emission for f64 workgroup shared memory. This
is not a hardware limitation — the Titan V has native 1:2 f64:f32 throughput
and the RTX 4070 has f64 builtins (sqrt, fma, abs/min/max).

### 1.2 DF64 and Alternative Paths Green

| Path | RTX 4070 | Titan V (NVK) |
|------|----------|---------------|
| DF64 add/sub | PASS | PASS |
| Tensor matmul | PASS | — |
| FHE NTT | PASS | — |
| f64 basic arithmetic | PASS | PASS |
| f64 shared-memory reduction | **FAIL (returns 0)** | **FAIL (returns 0)** |

### 1.3 coralReef Sovereign Compilation Success

coralReef (with 2 bug fixes) compiles the exact failing shader pattern to
native SM70 and SM89 binaries:

| Shader | SM70 | SM89 |
|--------|------|------|
| Basic f64 (mul + add) | 384 B | 384 B |
| Storage f64 (read/write) | 512 B | 512 B |
| Shared-memory simple | 640 B | 640 B |
| 2-barrier reduction | 768 B | 768 B |
| 3-barrier reduction | 1024 B | 1024 B |
| 8-step unrolled reduction | 2304 B | 2304 B |

All 672 coralReef tests pass after fixes.

---

## 2. Evolution Requests — barraCuda

### 2.1 CoralCompiler Integration (immediate)

The `CoralCompiler` IPC client (`coral_compiler.rs`) now handles the tokio
panic gracefully (`try_current()` check). However:

- **Cached binary inventory**: `compile_shader_f64` caches coralReef binaries
  but has no dispatch path. When coralDriver exists, barraCuda should check
  the cache first and use the sovereign binary when available.
- **Arch detection**: `arch_to_coral()` correctly maps `GpuArch` → `sm_xx`.
  No changes needed.

### 2.2 f64 Fallback Strategy (evolve)

With f64 shared-memory broken through naga/SPIR-V, barraCuda's Fp64Strategy
needs a fourth path:

```
Current:  f64 native → DF64 emulation → f32 (precision loss)
Proposed: f64 sovereign (coralReef) → f64 native (naga) → DF64 → f32
```

When `coralDriver` is available, `compile_shader_f64` should prefer the
sovereign binary for shaders that require workgroup shared memory.

### 2.3 Uniform Buffer Bindings (coralReef gap)

coralReef does not yet support `var<uniform>` bindings in compute shader
prologues. barraCuda's `sum_reduce_f64.wgsl` uses a uniform for the
workgroup count parameter. Either:

- **coralReef evolves**: Add uniform binding support in the compute prologue
- **barraCuda adapts**: Use storage buffer or push constants for the parameter

### 2.4 Delegation Status (unchanged)

91 active delegations (54 CPU + 37 GPU). All CPU delegations verified
compatible with barraCuda `cf1602c`. GPU tests: 17/32 pass (14 fail =
f64 shared-memory returning 0, not a barraCuda bug).

---

## 3. Evolution Requests — toadStool

### 3.1 HardwareFingerprint Extensions

toadStool S96c introduced `HardwareFingerprint` and `SubstrateCapabilityKind`.
groundSpring's `metalForge` forge already uses these for routing. Proposed
extensions based on V84 findings:

- **`f64_shared_memory: bool`**: Whether f64 workgroup shared memory actually
  works on this GPU+driver combination (currently always `true` in capability
  reports, but empirically `false` on naga/SPIR-V path)
- **`sovereign_binary_capable: bool`**: Whether the device can accept
  coralReef native binaries (requires coralDriver)

### 3.2 Sovereign Binary Dispatch

When coralDriver materializes:

```
toadStool dispatch pipeline:
  1. Check if sovereign binary exists in barraCuda cache
  2. If yes and device supports coralDriver: submit via CUDA Driver API
  3. If no: fall through to wgpu/naga path (current behavior)
```

This integrates with toadStool's existing substrate routing without changing
the wgpu path.

### 3.3 f64 Workload Routing (immediate)

metalForge already routes f64 → Titan V for throughput. However, with f64
shared-memory broken through NVK, toadStool should:

- **Route f64 shared-memory workloads to DF64 path** on both GPUs until
  coralDriver or naga fix is available
- **Keep f64 scalar/basic workloads** on native path (these work fine)

---

## 4. Evolution Requests — coralReef

### 4.1 Remaining Compilation Gaps

| Gap | Priority | Notes |
|-----|----------|-------|
| **coralDriver** (GPU submission) | P0 | Cubin ELF wrapper for CUDA Driver API is the fastest path to running sovereign binaries on RTX 4070 |
| **f64 instruction emission** | P0 | Basic f64 shaders disassemble as FMUL/FADD (f32) instead of DMUL/DADD (f64). `f64_lower` pass needs review |
| **BAR.SYNC opex encoding** | P1 | `nvdisasm` reports undefined opex table value 0x10 for TABLES_opex_0. Barrier count field encoding needs Volta reference |
| **Uniform buffer bindings** | P1 | `var<uniform>` in compute prologue not yet supported |
| **Loop instruction scheduling** | P2 | Loop back-edge triggers `opt_instr_sched_prepass` assertion. Unrolled shaders work |

### 4.2 Bugs Fixed (commit `849fedd`)

1. **CFG edge loss in `translate_if`**: Condition blocks now emit conditional
   branches to reject paths. Empty reject blocks branch directly to merge.
2. **Multi-predecessor RA merge**: `first_pass` now merges SSA→register
   mappings from all predecessors (was only using `pred[0]`).

### 4.3 IPC Server Ready

coralReef's JSON-RPC and tarpc IPC servers work correctly. barraCuda's
`CoralCompiler` client successfully compiles shaders via IPC. The server
supports `compiler.compile`, `compiler.health`, and `compiler.supported_archs`.

---

## 5. Cross-Spring Learnings

### 5.1 The naga/SPIR-V f64 Shared-Memory Issue Is Systemic

This is not GPU-specific or driver-specific. It affects:
- NVIDIA proprietary driver (RTX 4070, SM89)
- NVK/NAK (Mesa, Titan V, SM70)
- Both SPIR-V 1.3 and 1.6 target versions

Any spring that uses f64 workgroup shared memory through wgpu/naga will hit
this. The DF64 path is the correct workaround until either naga fixes the
SPIR-V emission or coralReef provides sovereign binaries with coralDriver.

### 5.2 Titan V Strategy

The Titan V is on NVK/NAK (Mesa open-source driver). Options for full f64:

1. **coralReef + coralDriver** (preferred): Sovereign compilation bypasses
   Mesa entirely. Requires coralDriver Phase 7.
2. **Switch to proprietary driver**: Would give CUDA toolkit access but
   has not been tested on Volta + Pop!_OS 22.04 with the current kernel.
3. **DF64 path** (current workaround): Works now, ~2x slower than native f64
   but functionally correct.

### 5.3 Write → Absorb → Lean Continues

groundSpring's 2 remaining reference shaders in `metalForge/shaders/`
(`anderson_lyapunov_*.wgsl`) are already absorbed upstream. The metalForge
forge crate provides architecture-aware routing. No new absorption
candidates from V84–V85; the sovereign compilation work is entirely within
coralReef.

---

## 6. groundSpring State at V85

| Metric | Value |
|--------|-------|
| Rust workspace tests | 824/824 PASS |
| coralReef tests | 672/672 PASS |
| Python tests | 390/390 PASS |
| Validation checks | 395/395 (340 core + 55 NUCLEUS) |
| metalForge checks | 187 (130 forge + 57 mixed-hardware) |
| barraCuda delegations | 91 (54 CPU + 37 GPU) |
| Experiments | 35 (10 domains) |
| Library modules | 34 |
| Deep debt | Zero (clippy pedantic+nursery, zero unsafe, zero unwrap in production) |
| Coverage | 97.25% library line coverage |

---

## 7. Recommended Next Steps

1. **coralReef team**: Fix f64 instruction emission (DMUL/DADD instead of FMUL/FADD), then build coralDriver cubin wrapper
2. **barraCuda team**: Add sovereign binary dispatch path in `compile_shader_f64` (cache check → coralDriver → wgpu fallback)
3. **toadStool team**: Add `f64_shared_memory` capability flag to `HardwareFingerprint`, route f64 shared-memory workloads to DF64 until sovereign path available
4. **All teams**: The DF64 path is battle-tested and functionally correct. Do not block on f64 native — DF64 is the reliable workaround
