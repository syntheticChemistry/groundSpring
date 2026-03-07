<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# groundSpring V93 → toadStool/barraCuda Evolution Handoff

**Date**: March 7, 2026
**From**: groundSpring V93 (903 tests, 101 delegations, 0 failures)
**To**: barraCuda team, toadStool team, coralReef team (FYI)
**Supersedes**: V91 handoff (Mar 7, 2026)
**License**: AGPL-3.0-only

## Executive Summary

groundSpring V92–V93 completed smart module refactoring, FFT wiring, and
coverage expansion on top of the V91 ecosystem rewire. This handoff updates
the evolution requests and provides new absorption opportunities.

**What changed since V91:**
- 903 tests (+96), 101 delegations (+1 FFT), 0 failures
- 3 large modules smart-refactored: rarefaction → diversity/sampling/orchestration,
  drift → monitor/simulation, tissue_anderson → geometry/simulation
- FFT wired into `spectral_recon::fft_power_spectrum()` via `Fft1DF64`
- 16 new equation tests (fao56), 10 compartment tests, 12 sweep tests, 3 FFT tests
- All files < 530 lines (down from 893 max at V91)
- `cargo fmt` + `clippy` (pedantic+nursery) + `doc` + `test` all clean

*This handoff is unidirectional: groundSpring → ecosystem. No response expected.*

---

## 1. Evolution Requests to barraCuda

### P0 — Blocks GPU Parity (All Springs) — STILL OPEN

**Fp64Strategy regression in `SumReduceF64`/`VarianceReduceF64`**

Status unchanged from V91. On RTX 4070 (Hybrid Fp64Strategy),
`SumReduceF64::mean` and `VarianceReduceF64::population_std` return incorrect
values. All 6 GPU-dispatched tests that hit these ops fail. CPU paths work.

The bug returns `Ok(wrong_value)` rather than `Err`, so groundSpring's
fallback-to-CPU pattern never triggers.

**Impact**: Blocks GPU parity for all springs using reduce ops.

### P1 — Missing/Gated APIs — STILL OPEN

| Request | Detail | Status |
|---------|--------|--------|
| `multinomial_sample_cpu` outside `cfg(gpu)` | CPU function behind GPU feature gate | **Open** |
| `tridiag_eigh` eigenvectors on GPU | Sturm eigenvalues exist, not eigenvectors | **Open** |

### P2 — Alignment

| Request | Detail | Status |
|---------|--------|--------|
| PRNG alignment | Xorshift64 (Python) vs Xoshiro128** (GPU) | **Open** |
| GPU test patterns | Mock/stub `WgpuDevice` for CI | **Open** |

### NEW — FFT Wiring Feedback

groundSpring V93 wired `Fft1DF64` into `spectral_recon::fft_power_spectrum()`.
The API works well but required `tokio_block_on` to bridge async → sync.
Consider providing a synchronous `Fft1DF64::execute_blocking()` wrapper or
documenting the recommended async bridging pattern for science primals.

---

## 2. Evolution Requests to toadStool

### P2 — Integration — UPDATED

| Request | Detail | Status |
|---------|--------|--------|
| Log subscriber initialization | toadStool doesn't init subscriber for science primals | **Open** |
| `SubstrateCapabilityKind::Fft` routing | FFT now wired in V93; toadStool should route | **Actionable** |

### P3 — Sovereign Pipeline

| Request | Detail | Status |
|---------|--------|--------|
| Wire `shader.compile.*` to coralReef IPC | Returns "not yet available" | **Open** |

---

## 3. Cross-Spring Insights (unchanged from V91)

See `specs/CROSS_SPRING_SHADER_EVOLUTION.md`. Key points:
- DF64 is the most cross-spring shader (all 5 springs)
- FFT radix-2 serves 3+ springs (PPPM, spectral analysis, signal processing)
- 6 shaders have 3+ spring blast radius — need dedicated regression tests
- metalForge pattern (30 workloads, 187 checks) available for toadStool absorption

---

## 4. Sovereign Pipeline Gaps (unchanged)

```
barraCuda WGSL → coralReef compile → native binary → coralDriver → DRM → GPU
                  ✓ COMPLETE         ✓ CACHED       ✗ SCAFFOLD    ✗ N/A
```

Critical path: coralDriver AMD `DRM_AMDGPU_CS` submission.

---

## 5. New groundSpring Capabilities for Absorption

### From V93 (new)

| Capability | Module | Note |
|------------|--------|------|
| `DriftMonitor` (extracted) | `drift/monitor.rs` | Standalone advisory for N_e·s tracking. Reusable by any evolutionary optimizer, not drift-specific. |
| Diversity indices (extracted) | `rarefaction/diversity.rs` | Simpson, Shannon, Bray-Curtis, Pielou in one module. Clean API for any community ecology use. |
| `fft_power_spectrum` CPU fallback | `spectral_recon.rs` | O(N²) DFT for correlators. Cross-validation reference for GPU FFT. |
| Tissue geometry types (extracted) | `tissue_anderson/geometry.rs` | Cell types, disorder functions, potentials. Reusable building blocks. |

### From V91 (still available)

| Capability | Module | Note |
|------------|--------|------|
| `optimal_block_size()` | `wdm.rs` | ACF → autocorrelation time → block size. Universal for jackknife. |
| `detect_transition()` | `anderson/spectral.rs` | Peak detection on sweep derivatives. General pattern. |
| `empirical_spectral_density` CPU fallback | `anderson/spectral.rs` | Histogram ESD, cross-validation reference. |

---

## 6. Quality State

| Gate | V91 | V93 |
|------|-----|-----|
| cargo test --workspace | 807 | **903** |
| Delegations | 100 (59+41) | **101** (60+41) |
| TODO/FIXME in source | 0 | 0 |
| unsafe code | 0 | 0 |
| Max file size (lines) | 893 | **529** |
| cargo clippy (pedantic+nursery) | PASS | PASS |
| cargo fmt --check | PASS | PASS |
| cargo doc --workspace | PASS | PASS |
| Benchmark workloads | 21 | 21 |

---

## 7. Superseded Handoffs

This handoff supersedes:
- `GROUNDSPRING_V91_TOADSTOOL_BARRACUDA_EVOLUTION_HANDOFF_MAR07_2026.md`

All prior handoffs archived for provenance.
