<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# groundSpring V94 → toadStool/barraCuda Evolution Handoff

**Date**: March 7, 2026
**From**: groundSpring V94 (907 tests, 102 delegations, 0 failures)
**To**: barraCuda team, toadStool team, coralReef team (FYI)
**Supersedes**: V93 handoff (Mar 7, 2026)
**Synced against**: barraCuda `0bd401f`, toadStool S129, coralReef Phase 10
**License**: AGPL-3.0-only

## Executive Summary

groundSpring V94 syncs against the latest ecosystem state (barraCuda
`0bd401f`, toadStool S129, coralReef Phase 10) and rewires new delegations.
This handoff corrects stale information in the toadStool absorption tracker
(which shows groundSpring at V85 — we are at V94 with 907 tests and 102
delegations).

**What changed since V93:**
- Synced to barraCuda `0bd401f` (cross-spring evolution + API debt elimination)
- +1 CPU delegation: `barracuda::stats::shannon` in drift Shannon computation
- `CorrelationFull` API evolved: `.r_squared()` and `.covariance()` convenience
  methods mirror barraCuda's `CorrelationResult` (0bd401f)
- `covariance_gpu` simplified: uses `CorrelationResult::covariance()` directly
- 907 tests (+4 new for CorrelationFull methods), 0 failures
- All quality gates clean (fmt, clippy pedantic+nursery, doc, test)

*This handoff is unidirectional: groundSpring → ecosystem. No response expected.*

---

## 0. Absorption Tracker Correction

toadStool's `SPRING_ABSORPTION_TRACKER.md` (S129) shows:

```
groundSpring | V85 | 812+390 | 87 delegations
```

**Current state**: groundSpring V94, 907 Rust tests + 261 Python tests, **102
delegations** (61 CPU + 41 GPU). The tracker is 9 versions behind.

### Items tracker shows as P2 DONE that we confirm are wired

| Item | Tracker says | groundSpring status |
|------|-------------|---------------------|
| L-BFGS GPU (batched numerical gradient) | P2 DONE | **Wired** in `freeze_out.rs` → `barracuda::optimize::lbfgs_numerical` since V85 |
| Tridiag QL eigenvector solver | P2 DONE | **Wired** in `linalg.rs` → local QL. barraCuda `spectral::tridiag::tridiag_eigenvectors` (Sturm + inverse iteration) now available as alternative path. Not yet wired — QL outperforms for single decomposition |

### Items NOT in tracker that need adding (V85→V94 delta)

| New delegation | Version | barraCuda API |
|---------------|---------|---------------|
| AutocorrelationF64 GPU | V91 | `barracuda::ops::autocorrelation_f64_wgsl` |
| Empirical Spectral Density | V91 | `barracuda::stats::spectral_density` |
| PeakDetectF64 | V91 | `barracuda::ops::peak_detect_f64` |
| CovarianceF64 GPU | V91 | `barracuda::ops::covariance_f64_wgsl` |
| Marchenko-Pastur bounds | V91 | `barracuda::stats::spectral_density` |
| FFT power spectrum | V93 | `barracuda::ops::fft::Fft1DF64` |
| Shannon (drift) | V94 | `barracuda::stats::shannon` |
| 3 module smart-refactors | V93 | — |
| 4 new CorrelationFull tests | V94 | — |

---

## 1. Evolution Requests to barraCuda

### P0 — Blocks GPU Parity (All Springs) — CONFIRMED STILL OPEN

**Fp64Strategy regression in `SumReduceF64`/`VarianceReduceF64`**

Verified against `0bd401f`: `SumReduceF64` and `VarianceReduceF64` still use
a single native f64 shader with **no `Fp64Strategy` branching**. On consumer
GPUs (RTX 3090/4070/2080 — `Hybrid` Fp64Strategy), these ops always use native
f64. Other ops (`VarianceF64`, `CorrelationF64`, `WeightedDotF64`, `GemmF64`)
correctly branch on `GpuDriverProfile::fp64_strategy()` and select DF64 shaders
for `Hybrid` devices.

**Impact**: `SumReduceF64::mean` and `VarianceReduceF64::population_std` return
incorrect values on consumer GPUs. The bug returns `Ok(wrong_value)` rather than
`Err`, so spring fallback-to-CPU never triggers.

**Fix**: Wire `Fp64Strategy` into `SumReduceF64` and `VarianceReduceF64`
matching the pattern used in `variance_f64_wgsl.rs` and `correlation_f64_wgsl.rs`.

### P1 — Missing/Gated APIs — UPDATED

| Request | Detail | Status |
|---------|--------|--------|
| `multinomial_sample_cpu` outside `cfg(gpu)` | CPU function behind GPU feature gate | **Open** — still in `ops::bio` which requires `gpu` feature |
| `tridiag_eigenvectors` GPU | Sturm + inverse iteration now on CPU (0bd401f). GPU batched path still missing | **Partially addressed** — CPU available, GPU open |

### P2 — Alignment

| Request | Detail | Status |
|---------|--------|--------|
| PRNG alignment | Xorshift64 (Python) vs Xoshiro128** (GPU) | **Open** |
| GPU test patterns | Mock/stub `WgpuDevice` for CI | **Open** |

### Feedback — New APIs (V94)

1. **`CorrelationResult::r_squared()` / `covariance()`** — Excellent additions
   (0bd401f). groundSpring V94 mirrors these on `CorrelationFull` and uses
   `CorrelationResult::covariance()` directly in the GPU covariance path.

2. **`Fft1DF64` async pattern** — Still requires `tokio_block_on` bridge.
   A synchronous `execute_blocking()` wrapper would reduce boilerplate in
   science primals. Low priority since the current pattern works.

3. **Shader provenance registry** — `shaders::provenance` tracks 2 groundSpring
   origin shaders (`anderson_lyapunov_f64.wgsl`, `chi_squared_f64.wgsl`) and
   7 shaders consumed by groundSpring. Provenance is accurate.

---

## 2. Evolution Requests to toadStool

### Absorption Tracker Update Request

Please update `SPRING_ABSORPTION_TRACKER.md`:

```
groundSpring | V94 | 907+261 | 102 delegations (61 CPU + 41 GPU)
```

### S128→S129 Items Verified

| toadStool feature | groundSpring awareness |
|-------------------|----------------------|
| `PrecisionRoutingAdvice` (S128) | Noted — groundSpring currently routes via barraCuda's `Fp64Strategy`. When toadStool provides runtime `precision_routing()` via JSON-RPC, groundSpring can consume it |
| `SubstrateCapabilityKind::Fft` (S96+) | FFT wired in V93; toadStool routing can direct to barraCuda `Fft1DF64` |
| `f64_shared_memory_reliable` (S128) | Noted — confirms our P0 Fp64Strategy report (shared-mem reductions fail) |
| `sovereign_binary_capable` (S128) | Noted for future coralDriver dispatch |
| 19,109 tests / 83% coverage | Excellent ecosystem health signal |

### P2 — Integration — UPDATED

| Request | Detail | Status |
|---------|--------|--------|
| Log subscriber initialization | `init_enhanced_logging` in `setup.rs` — confirmed functional for CLI. Science primals using toadStool as library need guidance | **Low priority** |
| `SubstrateCapabilityKind::Fft` routing | FFT wired in V93+; toadStool should advertise FFT capability for science primals | **Actionable** |

### P3 — Sovereign Pipeline

| Request | Detail | Status |
|---------|--------|--------|
| Wire `shader.compile.*` to coralReef IPC | 4 JSON-RPC methods exist (S128) — need coralReef to be running | **Open** |

---

## 3. coralReef Phase 10 — Status Update

### groundSpring shader compilation verified

| Shader | Target | Status |
|--------|--------|--------|
| `anderson_lyapunov_f32` | SM70 (Volta) | **PASS** — 2,272 B, 279 ms |
| `anderson_lyapunov_f64` | SM70 (Volta) | **PASS** — 4,896 B, 271 ms |
| `anderson_lyapunov_*` | AMD RDNA2 | **IGNORED** — encoder incomplete |

### IPC evolution

coralReef Phase 10 split `ipc.rs` into JSON-RPC + tarpc transports:
- `compiler.compile_wgsl` — WGSL → native binary (direct, no SPIR-V)
- `compiler.supported_archs` — GPU architecture query
- Unix domain socket + TCP transports

### Sovereign pipeline progress

```
barraCuda WGSL → coralReef compile → native binary → coralDriver → DRM → GPU
                  ✓ COMPLETE         ✓ CACHED       ✗ SCAFFOLD    ✗ N/A
```

AMD: GEM + PM4 done, CS submit + fence wait = P2
NVIDIA: QMD v3.0 done, pushbuf submit = not started

---

## 4. Cross-Spring Insights

### Shader provenance (barraCuda 0bd401f)

barraCuda's new `shaders::provenance` registry tracks cross-spring shader flow:
- 2 shaders **originated** from groundSpring
- 7 shaders **consumed** by groundSpring (from hotSpring, neuralSpring)
- `chi_squared_f64.wgsl` is the most cross-spring (consumed by all 5 springs)
- 13-tier tolerance model in coralReef aligned with groundSpring architecture

### Pattern: Write → Absorb → Lean

groundSpring V94 demonstrates the next step: consuming absorbed APIs
(`barracuda::stats::shannon`) and removing local implementations. wetSpring
(V97d) is fully lean (0 local WGSL). groundSpring has 2 local WGSL shaders
remaining in `metalForge/shaders/`.

---

## 5. Capabilities for Absorption

### From V94 (new)

| Capability | Module | Note |
|------------|--------|------|
| `CorrelationFull::r_squared()` / `covariance()` | `stats/correlation.rs` | Mirrors barraCuda's `CorrelationResult` API. Any spring consuming `pearson_full()` gets convenience methods |
| `shannon_from_abundances` delegation | `drift/mod.rs` | Clean pattern for delegating simple CPU stats to barraCuda |

### From V93 (still available)

| Capability | Module | Note |
|------------|--------|------|
| `DriftMonitor` (extracted) | `drift/monitor.rs` | Standalone advisory for N_e·s tracking |
| Diversity indices (extracted) | `rarefaction/diversity.rs` | Simpson, Shannon, Bray-Curtis, Pielou |
| `fft_power_spectrum` CPU fallback | `spectral_recon.rs` | O(N²) DFT cross-validation reference |
| Tissue geometry types (extracted) | `tissue_anderson/geometry.rs` | Reusable building blocks |
| `optimal_block_size()` | `wdm.rs` | ACF → block size. Universal for jackknife |
| `detect_transition()` | `anderson/spectral.rs` | Peak detection on derivatives |

---

## 6. Quality State

| Gate | V93 | V94 |
|------|-----|-----|
| cargo test --workspace | 903 | **907** |
| Delegations | 101 (60+41) | **102** (61+41) |
| TODO/FIXME in source | 0 | 0 |
| unsafe code | 0 | 0 |
| Max file size (lines) | 529 | 529 |
| cargo clippy (pedantic+nursery) | PASS | PASS |
| cargo fmt --check | PASS | PASS |
| cargo doc --workspace | PASS | PASS |
| Benchmark workloads | 21 | 21 |

---

## 7. Superseded Handoffs

This handoff supersedes:
- `GROUNDSPRING_V93_TOADSTOOL_BARRACUDA_EVOLUTION_HANDOFF_MAR07_2026.md`

All prior handoffs archived for provenance.
