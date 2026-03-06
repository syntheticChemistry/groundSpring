# groundSpring V83 → Dependency Catch-Up + Pin Refresh Handoff

**Date:** 2026-03-06
**From:** groundSpring V83 (34 experiments, 395/395 checks, 824 workspace tests)
**To:** barraCuda team, toadStool team, coralReef team
**License:** AGPL-3.0-only
**Covers:** V82 → V83 (dependency pin refresh, cross-spring evolution catch-up)
**Pins:** barraCuda `cf1602c`, toadStool S96c (`d77fc546`), coralReef `1e048be`

---

## Executive Summary

- groundSpring V83 is a compatibility verification and pin refresh — no new code
- All 91 delegations verified compatible with barraCuda `cf1602c` (0 API changes)
- 824 tests pass, 0 clippy warnings, all validation binaries green
- Updated pins: barraCuda `a4c20a5` → `cf1602c`, toadStool S95 → S96c, coralReef `2e89541` → `1e048be`

---

## What Changed Upstream

### barraCuda a4c20a5 → cf1602c

Deep debt resolution sprint — 954 files changed, 15,889 insertions:

| Category | Changes |
|----------|---------|
| **Quality** | JSON-RPC 2.0 notification compliance, unsafe elimination (2 remaining — wgpu API), zero-copy docs |
| **New GPU primitives** | `AutocorrelationF64` (time-series GPU op), `GpuView<T>` (Kokkos-style persistent buffers), `anderson_lyapunov_f32/f64.wgsl` |
| **Sovereign** | `CoralCompiler` for coralReef integration, `df64_rewrite/` split from god file |
| **Shaders** | `fft_radix2_f64.wgsl`, `vacf_dot_f64.wgsl`, nuclear shaders (chi2, SEMF, spin-orbit, deformed) |
| **Benchmarks** | `kokkos_parity.rs` started (mean/variance upload + GPU-resident) |
| **Metrics** | 708 WGSL shaders, 3,471+ tests, 62 integration suites |

**groundSpring impact**: None — all 91 delegation entry points are unchanged. The new GPU primitives (AutocorrelationF64, GpuView, anderson_lyapunov shaders) run underneath existing public API calls or are available for future delegation.

### toadStool S95 → S96c (d77fc546)

Sovereign pipeline infrastructure and structural cleanup:

| Category | Changes |
|----------|---------|
| **Sovereign** | `HardwareFingerprint` (estimated_tflops_f32/f64, sovereign_capable), `SubstrateCapabilityKind` (12 variants: F64Native, Df64Emulation, Spmv, Eigen, Cg, Fft, ...) |
| **Substrate** | `SubstrateType` expanded 4→8 variants (IntegratedGpu, Npu, Tpu, Fpga, Dsp, Quantum) |
| **God file splits** | 5 files >1000 LOC split: dispatch.rs→7, detection.rs→3, engine.rs→2, lib.rs→2, templates.rs→4 |
| **API orphan** | `crates/api/` fossilized, BYOB logic moved to `runtime/container/` |
| **Metrics** | 18,028 tests, 144 ComputeDispatch ops, 84% line coverage |

**groundSpring impact**: None — groundSpring's only toadStool dependency is `akida-driver` (unchanged). New `HardwareFingerprint` and `is_sovereign_capable()` APIs available for future metalForge integration.

### coralReef 2e89541 → 1e048be

Vendor-neutral naming evolution:

| Category | Changes |
|----------|---------|
| **Architecture** | `nak/` → `codegen/` (vendor-neutral module naming) |
| **Frontend** | New `Frontend` trait + `NagaFrontend` — pluggable shader language frontends |
| **Naming** | `MuFuOp` → `TranscendentalOp`, C-style fields → Rust idiomatic |
| **Phase** | Phase 5.5 naming evolution complete, Phase 6 multi-vendor in progress |
| **Tests** | 390 → 672 tests (73% increase) |

**groundSpring impact**: Sovereign pipeline handoff updated — coralReef's vendor-neutral architecture makes the AMD/Intel backend path clearer.

---

## Delegation Inventory (unchanged at 91)

### CPU Delegations (54)

All verified compatible — barraCuda's public stats/spectral/numerical/special APIs unchanged.

### GPU Delegations (37)

All verified compatible — wgpu 28 API unchanged, shader dispatches work.

---

## Future Delegation Candidates (new from this catch-up)

| barraCuda Capability | groundSpring Use Case | Priority |
|---------------------|----------------------|----------|
| `AutocorrelationF64` | WDM transport coefficients, jackknife AR(1) | P2 — WDM is synthetic VACF, not raw time-series |
| `GpuView<T>` | Chain `pearson_full` → `mean_and_std_dev` without CPU round-trip | P2 — optimization, not new functionality |
| `anderson_lyapunov_f32.wgsl` | f32 Lyapunov on consumer GPUs (RTX 4070) | P3 — already delegating to public API |
| `fft_radix2_f64.wgsl` | Spectral reconstruction FFT gap | P3 — shader exists, driver orchestration needed |
| `CoralCompiler` | Titan V sovereign binary compilation | P2 — needs coralDriver first |
| `HardwareFingerprint` | metalForge substrate characterization | P3 — future toadStool integration |

---

## Validation Certificate

```
cargo check --workspace           PASS  (clean build with updated deps)
cargo clippy -D warnings          PASS  (0 warnings)
cargo test --workspace            PASS  (824 tests, 0 failures)
coralReef cargo test --workspace  PASS  (672 tests, 0 failures)
Pin compatibility                 PASS  (91/91 delegations verified)
```

---

## Recommended Next Steps

### groundSpring
1. Wire `AutocorrelationF64` GPU dispatch for WDM module (P2)
2. Wire `GpuView` for chained GPU stats operations (P2, optimization)
3. Build out remaining paper queue experiments
4. Prepare for coralDriver integration when available

### barraCuda
1. Complete Kokkos GPU parity benchmarks (started, not validated)
2. Document `GpuView` usage patterns for springs
3. Wire `fft_radix2_f64` into driver-orchestrated FFT API

### toadStool
1. Continue ComputeDispatch migration (144/283 ops)
2. Expose `HardwareFingerprint` to springs via IPC

### coralReef
1. Phase 6: AMD backend (RDNA3 instruction encoding)
2. Naga 24 → 28 alignment with barraCuda/wgpu
3. `coralDriver` for kernel launch (Titan V proving ground)
