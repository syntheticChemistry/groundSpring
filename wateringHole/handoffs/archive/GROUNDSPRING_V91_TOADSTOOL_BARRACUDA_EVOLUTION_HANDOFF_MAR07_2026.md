# groundSpring V91 → toadStool/barraCuda Evolution Handoff

**Date**: March 7, 2026
**From**: groundSpring V91
**To**: barraCuda team, toadStool team, coralReef team (FYI)
**License**: AGPL-3.0-only

## Executive Summary

groundSpring has completed a full ecosystem rewire (V90–V91) and deep-debt
execution, validating 100 delegations against barraCuda v0.3.3, toadStool S128,
and coralReef Phase 9. This handoff documents:

1. **Evolution requests** (bugs found, missing APIs, alignment gaps)
2. **Insights** from cross-spring shader provenance analysis
3. **Sovereign compute pipeline gaps** discovered during review
4. **New groundSpring capabilities** available for absorption

*This handoff is unidirectional: groundSpring → ecosystem. No response expected.*

---

## 1. Evolution Requests to barraCuda

### P0 — Blocks GPU Parity (All Springs)

**Fp64Strategy regression in `SumReduceF64`/`VarianceReduceF64`**

Commit `ed82625` wired `Fp64Strategy` into reduce ops. On RTX 4070 (Hybrid
Fp64Strategy), `SumReduceF64::mean` and `VarianceReduceF64::population_std`
return incorrect values (zeros or wrong magnitudes). All 6 GPU-dispatched tests
that hit these ops fail. The CPU paths work perfectly.

The groundSpring delegation pattern is `if let Ok(v) = gpu_fn() { return v; }` —
the bug is that the ops return `Ok(wrong_value)` rather than `Err`, so the
fallback never triggers.

**Suggestion**: Add a sanity check or NaN guard in the reduce shaders for Hybrid
devices. Alternatively, fall back to DF64 shader path rather than Native when the
driver profile is Hybrid.

**Impact**: Blocks GPU parity tests for groundSpring, wetSpring, hotSpring, and
all other springs that use these fundamental reduce ops.

### P1 — Missing/Gated APIs

| Request | Detail |
|---------|--------|
| `multinomial_sample_cpu` outside `cfg(gpu)` | This is a pure CPU function but lives behind `#[cfg(feature = "gpu")]`. groundSpring needs it for rarefaction without requiring GPU feature gate. |
| `tridiag_eigh` eigenvectors on GPU | barraCuda has Sturm eigenvalues but not eigenvectors. groundSpring's `transport::tridiag_eigh` can't delegate to GPU. |

### P2 — Alignment

| Request | Detail |
|---------|--------|
| PRNG alignment | groundSpring uses Xorshift64 (Python baseline); barraCuda GPU uses Xoshiro128**. Need migration path or seed-translation layer for Tier B validation parity. |
| GPU test coverage patterns | Mock/stub `WgpuDevice` for CI environments without GPU. Currently groundSpring GPU tests can only run on real hardware. |

---

## 2. Evolution Requests to toadStool

### P2 — Integration

| Request | Detail |
|---------|--------|
| Log subscriber initialization | groundSpring uses `log` crate but toadStool doesn't initialize a subscriber when launching groundSpring as a science primal. Warnings are silently lost. |
| `SubstrateCapabilityKind::Fft` routing | When groundSpring wires FFT ops (P1 future), toadStool should route FFT workloads to FFT-capable substrates. |

### P3 — Sovereign Pipeline

| Request | Detail |
|---------|--------|
| Wire `shader.compile.*` to real coralReef IPC | Currently returns "coralReef native path not yet available". When coralDriver is ready, this will enable sovereign dispatch for groundSpring's 41 GPU workloads. |

---

## 3. Insights from Cross-Spring Shader Provenance Analysis

groundSpring V91 documented the full provenance tree for 708+ WGSL shaders
flowing between springs via barraCuda (see `specs/CROSS_SPRING_SHADER_EVOLUTION.md`).

### Key findings for barraCuda team

1. **DF64 is the most cross-spring shader**: hotSpring's `df64_core.wgsl` powers
   earth science MC, marine bio diversity, ML validation, and nuclear structure
   across all five springs. Any DF64 regression or precision improvement affects
   the entire ecosystem.

2. **FFT is the second most cross-spring op**: groundSpring's `fft_radix2_f64.wgsl`
   (Cooley-Tukey) is used for PPPM electrostatics (hotSpring), spectral analysis
   (neuralSpring), and signal processing (wetSpring). Consider promoting it to a
   core op with dedicated regression tests across multiple use cases.

3. **Cross-spring validation**: 6 shaders are used by 3+ springs. Regression in
   any of these has outsized blast radius:
   - `df64_core.wgsl` (all springs)
   - `fft_radix2_f64.wgsl` (hotSpring, groundSpring, wetSpring)
   - `fused_map_reduce_f64.wgsl` (wetSpring, airSpring, hotSpring)
   - `mean_variance_f64.wgsl` (all springs)
   - `correlation_full_f64.wgsl` (groundSpring, hotSpring, wetSpring)
   - `batched_elementwise_f64.wgsl` (airSpring, wetSpring, hotSpring)

4. **Absorption velocity**: The Mar 3-5 2026 wave absorbed shaders from all 5
   springs simultaneously. This shows the model works — springs write shaders,
   barraCuda absorbs, all springs benefit. The provenance documentation makes
   this traceable.

### Key findings for toadStool team

1. **Precision routing matters**: groundSpring V84-V85 discovered that naga/SPIR-V
   f64 shared-memory reductions return zeros on certain GPUs. toadStool's
   `f64_shared_memory_reliable` flag (always false) is correct. When coralReef
   sovereign path replaces naga, this should be re-evaluated.

2. **SubstrateCapabilityKind expansion**: toadStool already has `Fft`. Consider
   adding `Autocorrelation`, `PeakDetect`, `SpectralDensity` as capabilities
   that springs can discover and route to.

3. **metalForge as bridge**: groundSpring's metalForge (30 workloads, 187 checks)
   validates cross-substrate dispatch. The pattern could be absorbed by toadStool
   as a universal cross-substrate validation framework.

---

## 4. Sovereign Compute Pipeline Gaps (for all teams)

During the V91 review, groundSpring audited the full sovereign pipeline:

```
barraCuda WGSL → coralReef compile → native binary → coralDriver → DRM → GPU
                  ✓ COMPLETE         ✓ CACHED       ✗ SCAFFOLD    ✗ N/A
```

| Gap | Owner | Status |
|-----|-------|--------|
| coralDriver AMD: real `DRM_AMDGPU_CS` submission | coralReef | `submit_command` returns `Ok(())` (scaffold) |
| coralDriver NVIDIA: nouveau pushbuf | coralReef | `dispatch` returns `Err(Unsupported)` |
| coral-gpu: wire compile → dispatch | coralReef | `GpuContext` compiles only |
| barraCuda: replace wgpu with coral-gpu | barraCuda | WHATS_NEXT, not started |
| toadStool: wire shader.compile.* | toadStool | Stubs, `coral_reef_available: false` |
| Bulk 708-shader validation through coralReef | cross-primal | Not done |

**Recommendation**: The coralDriver dispatch gap is the critical path. Once AMD
submission works, the entire pipeline can be validated end-to-end. groundSpring
will wire sovereign dispatch as soon as it's available.

---

## 5. New groundSpring Capabilities for Absorption

### Ready for barraCuda absorption

| Capability | Module | Note |
|------------|--------|------|
| `optimal_block_size()` | wdm.rs | Uses ACF → integrated autocorrelation time → block size recommendation. Pure statistics, no domain dependency. Useful for any jackknife analysis. |
| `detect_transition()` | anderson.rs | Peak detection on sweep derivatives for transition detection. General pattern applicable beyond Anderson. |
| `empirical_spectral_density` CPU fallback | anderson.rs | Simple histogram-based ESD. Could complement the existing barraCuda version as a cross-validation reference. |

### Benchmark data available

groundSpring V91 benchmarks (21 workloads) provide baseline timings for all
delegated operations on i9-12900K CPU. When GPU dispatch is fixed, this will
give CPU vs GPU speedup data for all groundSpring workloads.

---

## 6. Quality State

| Gate | Status |
|------|--------|
| cargo fmt --check | PASS |
| cargo clippy --workspace --all-targets -D warnings -W pedantic -W nursery | PASS |
| cargo check --workspace | PASS |
| cargo doc --workspace --no-deps | PASS |
| cargo test --workspace | 807 tests, 0 failures |
| Line coverage (llvm-cov) | 91.55% |
| TODO/FIXME in source | 0 |
| unsafe code | 0 (temp-env RAII for tests) |
| Files > 1000 lines | 0 |
| Delegations | 100 (59 CPU + 41 GPU) |
| Benchmark workloads | 21 |

---

## 7. Superseded Handoffs

This handoff supersedes:
- `GROUNDSPRING_V91_COMPLETE_REWIRE_HANDOFF_MAR07_2026.md` (internal rewire status)
- `GROUNDSPRING_V90_DEEP_DEBT_REWIRE_HANDOFF_MAR07_2026.md` (deep debt status)
- `GROUNDSPRING_V89_REWIRE_EVOLUTION_HANDOFF_MAR06_2026.md` (V89 rewire)

These are archived for provenance. This document is the canonical V91 handoff.
