<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# groundSpring V95 → toadStool/barraCuda/coralReef Evolution Handoff

**Date**: March 7, 2026
**From**: groundSpring V95 (907 tests, 102 delegations, 0 failures)
**To**: barraCuda team, toadStool team, coralReef team
**Supersedes**: V94 handoff (Mar 7, 2026)
**Synced against**: barraCuda `0bd401f`, toadStool S129, coralReef Phase 11
**License**: AGPL-3.0-only

## Executive Summary

groundSpring V95 contains a **critical coralReef breakthrough**: the persistent
`[PBENTRY]` errors that killed every GPU channel on SET_OBJECT were caused by a
**push buffer header field swap** — the `count` and `method/4` fields in the
Kepler+ Type 1 (INCR) header were transposed. This single bug blocked all GPU
compute method dispatch since coralReef Phase 1.

**What changed since V94:**
- **coralReef Phase 11**: Push buffer encoding fixed — `mthd_incr` count/method
  fields corrected to match NVK's `NVC0_FIFO_PKHDR_SQ`
- **NVIF constants**: Aligned to Mesa's `nvif/ioctl.h` — `ROUTE_NVIF=0x00`,
  `ROUTE_HIDDEN=0xFF`, `OWNER_NVIF=0x00`, `OWNER_ANY=0xFF`
- **All 5 GPU method tests pass** on Titan V: NOP, SET_OBJECT, INVALIDATE_SHADER_CACHES,
  SET_SHADER_LOCAL_MEMORY_WINDOW, METHOD_0_DATA_0
- Push buffer `mthd_ninc` (non-incrementing) also fixed
- 11 test files corrected across coral-driver
- V94 handoff archived

*This handoff is unidirectional: groundSpring → ecosystem. No response expected.*

---

## 0. coralReef Breakthrough: Push Buffer Encoding

### Root Cause

The Kepler+ GPU push buffer Type 1 (INCR) header format is:

```
[31:29] = type(1)  [28:16] = count  [15:13] = subchan  [12:0] = method/4
```

Our `mthd_incr` had count and method/4 **transposed**:

```rust
// BROKEN (all prior versions)
(0x1 << 29) | ((method >> 2) << 16) | (subchan << 13) | count

// CORRECT (V95, matches NVK NVC0_FIFO_PKHDR_SQ at nv_push.h:80)
(0x1 << 29) | (count << 16) | (subchan << 13) | (method >> 2)
```

For NOP (method=0, count=0), both fields are zero — the bug was invisible.
For SET_OBJECT (method=0, count=1), the broken encoding produced `0x20000001`
instead of `0x20010000`. The PBDMA interpreted the data word `0x0000c3c0` as
the *next header* — Type 0 (illegal on Kepler+), producing the `[PBENTRY]`
fault that killed every channel.

### Discovery Method

1. Confirmed NVK Vulkan compute works on Titan V (`vk_compute_test2.c`)
2. Captured NVK ioctl trace via `LD_PRELOAD` spy
3. Cross-referenced NVK's `NVC0_FIFO_PKHDR_SQ` (Mesa `nv_push.h` line 80)
4. Found `0x20000000 | (size << 16) | (subc << 13) | (mthd >> 2)`
5. Compared to our `(0x1 << 29) | ((method >> 2) << 16) | (subchan << 13) | count`
6. Fields swapped — count in method position, method in count position

### NVIF Constant Alignment

Mesa's `nvif/ioctl.h` defines (different from kernel headers):

| Constant | Mesa value | Our old value | Fixed |
|----------|-----------|---------------|-------|
| `NVIF_IOCTL_V0_ROUTE_NVIF` | `0x00` | `0xFF` | `0x00` |
| `NVIF_IOCTL_V0_ROUTE_HIDDEN` | `0xFF` | (missing) | `0xFF` |
| `NVIF_IOCTL_V0_OWNER_NVIF` | `0x00` | (missing) | `0x00` |
| `NVIF_IOCTL_V0_OWNER_ANY` | `0xFF` | `0x00` | `0xFF` |

The outer ioctl header for subchannel alloc uses `route=ROUTE_HIDDEN (0xFF)`,
`owner=OWNER_NVIF (0x00)`. The inner `NvifNewV0` uses `route=ROUTE_NVIF (0x00)`.

### Files Changed

| File | Change |
|------|--------|
| `coral-driver/src/nv/pushbuf.rs` | `mthd_incr` and `mthd_ninc` field order fixed |
| `coral-driver/src/nv/ioctl.rs` | NVIF constants aligned, `nvif_new_class` header corrected |
| `coral-driver/tests/*.rs` (11 files) | All `mthd_incr` functions corrected |

---

## 1. coralReef Pipeline Status (Phase 11)

### Working

| Component | Status |
|-----------|--------|
| WGSL → SASS compilation (SM70/SM86) | Working |
| QMD v2.1 (Volta) / v3.0 (Ampere) | Working |
| DRM VM_INIT + VM_BIND + EXEC | Working |
| NVIF class object creation | Working |
| Push buffer: SET_OBJECT | Working |
| Push buffer: INVALIDATE_SHADER_CACHES | Working |
| Push buffer: SET_SHADER_LOCAL_MEMORY_WINDOW | Working |
| Channel survival after all methods | Working |

### Remaining Blockers (P0)

| Blocker | Detail | Owner |
|---------|--------|-------|
| QMD constant buffer binding | `buffer_vas` passed but ignored — shaders cannot access storage buffers | coralReef |
| Binding layout mapping | No WGSL `@binding(N)` → QMD CBUF index mapping | coralReef |
| GPR count from compiler | QMD hardcodes 32 GPRs; compiler knows actual count | coralReef |

### Hardening (P1)

| Item | Detail | Owner |
|------|--------|-------|
| Fence synchronization | EXEC is fire-and-forget; no fence wait | coralReef |
| NvDevice alignment | `NvDevice::open()` uses wrong VM_INIT params; needs `0x80_0000_0000` | coralReef |
| Shared memory sizing | QMD shared memory not from compiler analysis | coralReef |
| AMD submission | PM4 built but `DRM_AMDGPU_CS` not complete | coralReef |

---

## 2. barraCuda Evolution Requests

### P0 — Blocks GPU Parity

| Request | Detail |
|---------|--------|
| Fix Fp64Strategy in `SumReduceF64` | Missing Hybrid/Native branching; consumer GPUs produce wrong values |
| Fix Fp64Strategy in `VarianceReduceF64` | Same issue; both are the only f64 ops without the strategy |

### P1 — Functional Gaps

| Request | Detail |
|---------|--------|
| `multinomial_sample_cpu` outside cfg(gpu) | CPU fallback gated behind GPU feature; needed for CPU-only builds |
| Tridiagonal eigenvectors GPU | Eigenvalues run on GPU (Sturm); eigenvectors CPU-only (inverse iteration) |
| `CoralReefDevice` backend | barraCuda dispatch via coralReef sovereign SASS; new backend alongside `WgpuDevice` |

### P2 — Quality

| Request | Detail |
|---------|--------|
| PRNG alignment | Xorshift64 (Python/CPU) vs Xoshiro128** (GPU) |
| Mock WgpuDevice | CI without real GPU hardware |

---

## 3. toadStool Evolution Requests

### P1

| Request | Detail |
|---------|--------|
| Update absorption tracker | Shows V85/87 delegations; actual V95/102 delegations |
| coralReef IPC integration | Discover and route to coralReef for sovereign compilation |
| FFT capability routing | `SubstrateCapabilityKind::Fft` wired in V93 |

### P2

| Request | Detail |
|---------|--------|
| Unidirectional streaming | Reduce dispatch round-trips for coralReef DRM EXEC path |

---

## 4. Absorption Tracker Delta (V94→V95)

No new delegations in V95. The 102 delegation count (61 CPU + 41 GPU) is unchanged.
V95 is a coralReef-focused release.

---

## 5. Sovereign Pipeline — End-to-End Status

```
WGSL shader (groundSpring/barraCuda)
    │
    ▼
coral-reef compiler (WGSL → SASS)           ✅ WORKING
    │
    ▼
QMD construction + buffer binding            ❌ P0: CBUFs not populated
    │
    ▼
Push buffer builder (SET_OBJECT + QMD)       ✅ WORKING (fixed V95)
    │
    ▼
DRM EXEC submission                          ✅ WORKING (fixed V95)
    │
    ▼
GPU executes shader                          ⚠️ UNTESTED (no buffer binding = zeros)
    │
    ▼
Fence wait + readback                        ❌ P1: no sync
    │
    ▼
barraCuda CoralReefDevice backend            ❌ P1: wgpu only
    │
    ▼
toadStool routes to coralReef               ❌ P1: no IPC discovery
```

The single biggest remaining blocker is **QMD constant buffer binding**.
Once `buffer_vas` are written into QMD CBUF slots, the E2E sovereign
compute test should produce real results.

---

## 6. NVK ioctl Trace Reference

The `/tmp/nvk_compute_trace.log` captured from a working NVK Vulkan compute
dispatch on the Titan V provides the golden reference for:

- VM_INIT parameters: `kernel_managed_addr=0x80_0000_0000`, `size=0x80_0000_0000`
- NVIF class creation sequence (CE → 3D → COMPUTE)
- NVIF inner/outer header byte layout
- EXEC submission format
- GEM_NEW + VM_BIND patterns

This trace was instrumental in identifying the push buffer field swap and
should be preserved as a reference for coralReef development.

---

*This handoff is unidirectional: groundSpring → ecosystem. No response expected.*
