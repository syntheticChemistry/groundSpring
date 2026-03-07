<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# groundSpring V96 → toadStool/barraCuda/coralReef Upstream Rewire Handoff

**Date**: March 7, 2026
**From**: groundSpring V96 (925 tests, 102 delegations, 0 failures)
**To**: barraCuda team, toadStool team, coralReef team
**Supersedes**: V95 handoff (Mar 7, 2026)
**Synced against**: barraCuda `2a6c072`, toadStool S130 (`88a545df`), coralReef Iteration 7 (`72e6d13`)
**License**: AGPL-3.0-only

## Executive Summary

groundSpring V96 is an **upstream rewire release** that catches up to the
latest barraCuda (module decomposition, shader.compile.* IPC alignment, lint
hardening), toadStool S130 (cross-spring shader rewiring, coralReef proxy,
provenance tracking), and coralReef Iteration 7 (safety boundary, ioctl
layout tests, CFG domain-split).

**What changed since V95:**
- **PrecisionRoutingAdvice wired**: `gpu::precision_routing()` queries
  barraCuda `GpuDriverProfile` and returns hardware-appropriate f64 routing
  advice (F64Native / F64NativeNoSharedMem / Df64Only / F32Only)
- **Public API**: `groundspring::gpu_precision_routing()` re-exported with
  `#[cfg(feature = "barracuda-gpu")]` gate
- **Pin updates**: barraCuda `0bd401f` → `2a6c072`, toadStool S129 → S130,
  coralReef Phase 11 → Iteration 7
- **925 tests** (was 907) — 18 new tests from upstream evolution
- **Stale clippy fix**: Removed unfulfilled `#[expect(clippy::cast_precision_loss)]`
  from `validate_real_ghcnd_et0.rs`; collapsed nested `if let` in
  `stats/correlation.rs` per `collapsible_if`
- **Doc sync**: README, CONTROL_EXPERIMENT_STATUS, BARRACUDA_EVOLUTION updated
  with new pins and test counts
- All quality gates pass: fmt, clippy (pedantic+nursery, -D warnings), doc, test

*This handoff is unidirectional: groundSpring → ecosystem. No response expected.*

---

## 1. Precision Routing Integration

### New API Surface

```rust
// crates/groundspring/src/gpu.rs
pub use barracuda::device::driver_profile::PrecisionRoutingAdvice;

pub fn precision_routing() -> Option<PrecisionRoutingAdvice> {
    let device = get_device()?;
    let profile = GpuDriverProfile::from_device(&device);
    Some(profile.precision_routing())
}

// crates/groundspring/src/lib.rs (public re-export)
#[cfg(feature = "barracuda-gpu")]
pub fn gpu_precision_routing() -> Option<gpu::PrecisionRoutingAdvice>;
```

### Routing Advice Tiers

| Advice | Meaning | groundSpring Action |
|--------|---------|---------------------|
| `F64Native` | Full f64, workgroup reductions safe | Use native f64 shaders |
| `F64NativeNoSharedMem` | f64 compute OK, `var<workgroup>` f64 returns zeros | Route reductions through DF64 or scalar |
| `Df64Only` | No reliable native f64 | Use DF64 (f32-pair) for all f64 work |
| `F32Only` | No f64 at all | Fall back to f32 |

### Provenance

- `PrecisionRoutingAdvice` originated in groundSpring V84-V85 (f64 shared-memory bug discovery)
- Absorbed into toadStool S128, re-exported from barraCuda `device::driver_profile`
- Now round-tripped back into groundSpring as a first-class API

---

## 2. Upstream Pin Updates

| Dependency | Old Pin | New Pin | Key Changes |
|------------|---------|---------|-------------|
| barraCuda | `0bd401f` | `2a6c072` | Module decomposition, shader.compile.* IPC alignment, lint hardening, LSCFRK integrators, DF64 bug fix |
| toadStool | S129 (`e1cdaa3e`) | S130 (`88a545df`) | Cross-spring shader rewiring, coralReef proxy, provenance tracking, doc cleanup |
| coralReef | Phase 11 | Iteration 7 (`72e6d13`) | Safety boundary, ioctl layout tests, CFG domain-split |

### barraCuda evolution since `0bd401f`

1. `e30b3ae` — PrecisionRoutingAdvice, BatchedOdeRK45F64 (new adaptive RK45)
2. `cb9325c` — doc refresh, test count updates, false positive fixes
3. `659ade9` — complete shader.compile.* IPC contract alignment
4. `2a6c072` — module decomposition, Phase 10 IPC alignment, lint hardening

### New barraCuda primitives available (not yet consumed)

| Primitive | Purpose | groundSpring Status |
|-----------|---------|---------------------|
| `BatchedOdeRK45F64` | Adaptive Dormand-Prince 5(4) GPU ODE | Available; current experiments use fixed-step RK4 correctly |
| `GpuView<T>` | Persistent GPU buffer (zero-readback chains) | Available; pending barraCuda ops that accept `&GpuView<T>` |
| `LSCFRK` integrators | Lattice QCD gradient flow | Not applicable to groundSpring experiments |

---

## 3. Quality Gate Status

| Gate | Status | Detail |
|------|--------|--------|
| `cargo fmt --check` | PASS | All groundSpring code formatted |
| `cargo clippy --all-features -D warnings` | PASS | Pedantic + nursery, 0 warnings |
| `cargo doc --no-deps` | PASS | 0 warnings, 44 pages |
| `cargo test --workspace` | PASS | **925/925**, 0 ignored, 0 failed |
| `unsafe` | PASS | `#![forbid(unsafe_code)]` in all 3 crates |
| Max file size | PASS | 724 lines max (`freeze_out.rs`) |
| SPDX headers | PASS | All files `AGPL-3.0-only` |

---

## 4. Evolution Requests

### barraCuda P0 (from V95, still open)

- Fix `Fp64Strategy` in `SumReduceF64` and `VarianceReduceF64` — Hybrid/Native
  branching should select DF64 shader variant on Hybrid devices

### toadStool P1 (from V95, updated)

- Update `SPRING_ABSORPTION_TRACKER.md`: groundSpring pin V85 → **V96**,
  delegations 87 → **102** (61 CPU + 41 GPU), tests 812 → **925**
- Wire `shader.compile.*` IPC to live coralReef when running
  (currently returns "not yet available")
- Add FFT capability routing to `SubstrateCapabilityKind`

### groundSpring completed (V96b)

- **Precision routing wired into 11 GPU dispatch paths** — all f64 workgroup
  reduction paths now call `get_device_f64_safe()` which checks
  `PrecisionRoutingAdvice` before dispatch:
  - `pearson_full_gpu`, `pearson_r_gpu`, `covariance_gpu` (correlation.rs)
  - `mean_and_std_dev_gpu` (metrics.rs)
  - `coefficient_of_efficiency_gpu`, `rmse_gpu`, `mae_gpu` (agreement.rs)
  - `jackknife_mean_gpu` (jackknife.rs)
  - `simpson_diversity_gpu`, `shannon_diversity_gpu` (rarefaction/diversity.rs)
  - `autocorrelation_gpu` (wdm.rs)
- Three-tier parity tests: 51/51 PASS (physics 27, stats 24)
- Cross-spring benchmark validated with provenance annotations

### groundSpring next steps

- Wire `BatchedOdeRK45F64` when an experiment needs adaptive stepping
- Add `cargo llvm-cov` to CI (target 90% library coverage)
- Consume `GpuView<T>` when barraCuda ops accept `&GpuView<T>` directly

---

## 5. Delegation Summary

| Category | Count | Examples |
|----------|-------|---------|
| CPU stats | 21 | mean, std_dev, rmse, mae, mbe, r², NSE, hit_rate, norm_cdf, chi² |
| CPU bio | 8 | shannon, simpson, chao1, bray_curtis, kimura, detection_power |
| CPU hydrology | 8 | fao56_et0, hargreaves, makkink, turc, hamon, thornthwaite, crop_kc |
| CPU linalg | 4 | eigh_f64, solve_f64_cpu, cholesky, tridiag |
| CPU regression | 5 | fit_linear, fit_quadratic, fit_exponential, fit_logarithmic, fit_all |
| CPU numerical | 3 | trapz, hill, monod |
| CPU spectral | 5 | lyapunov, localization_length, anderson_potential, detect_bands, classify_phase |
| CPU other | 7 | bootstrap, jackknife, rawr, esn, ode_bio |
| **CPU total** | **61** | |
| GPU stats | 10 | VarianceF64, CovarianceF64, CorrelationF64, FusedMapReduceF64, SumReduceF64 |
| GPU bio | 3 | WrightFisherGpu, BatchedMultinomialGpu, GillespieGpu |
| GPU spectral | 6 | anderson_2d/3d/4d, wegner_block_4d, sweep_averaged, lanczos |
| GPU optimize | 3 | batched_nelder_mead_gpu, lbfgs_numerical, grid_search_3d |
| GPU hydrology | 3 | SeasonalPipelineF64, HargreavesBatchGpu, McEt0PropagateGpu |
| GPU linalg | 2 | eigh_f64 (dense), solve_f64 (Tikhonov) |
| GPU other | 14 | BootstrapMeanGpu, JackknifeMeanGpu, RawrWeightedMeanGpu, BatchedOdeRK4F64, ESN, PeakDetectF64, AutocorrelationF64, etc. |
| **GPU total** | **41** | |
| **Grand total** | **102** | |

---

## 6. Cross-Spring Shader Evolution (Verified V96)

The benchmark-cross-spring binary now annotates each workload with its
evolutionary origin. The ecoPrimals shader ecosystem has 708 WGSL shaders
in barraCuda, evolved organically from five springs:

### Origin → Consumer Matrix

| Origin Spring | Key Contributions | Consumer Springs |
|---------------|-------------------|------------------|
| **hotSpring** (physics) | `df64_core.wgsl`, DF64 transcendentals, SU(3), Lanczos, Sturm tridiag | **all** — f64-class on consumer GPUs |
| **wetSpring** (biology) | Smith-Waterman, Felsenstein, Gillespie SSA, HMM, `fused_map_reduce_f64.wgsl` | neuralSpring, groundSpring |
| **neuralSpring** (ML) | matrix correlation, linear regression, KL divergence, chi², batch IPR, `pow_f64` polyfill | hotSpring, groundSpring, airSpring |
| **airSpring** (hydrology) | Hargreaves, seasonal pipeline, moving window, Brent root-finding, L-BFGS | wetSpring, groundSpring |
| **groundSpring** (noise) | `anderson_lyapunov_f64.wgsl`, `chi_squared_f64.wgsl`, `welford_mean_variance_f64.wgsl`, 13-tier tolerances | **all** — validation backbone |

### Evolution Path Examples

1. **hotSpring DF64 → all springs**: hotSpring S58 created `df64_core.wgsl` (f32-pair arithmetic) for lattice QCD. Every spring's GPU ops now run on consumer GPUs via DF64 fallback.

2. **wetSpring bio → groundSpring ecology**: Shannon/Simpson/Bray-Curtis diversity indices (wetSpring S15/S64) are consumed by groundSpring's rarefaction experiments (Exp 004, 016).

3. **neuralSpring ML → groundSpring spectral**: `pow_f64` polyfill (neuralSpring S-17) unblocked Ada Lovelace for all springs. Domain dispatch pattern became the blueprint for groundSpring GPU wiring.

4. **airSpring hydrology → groundSpring physics**: RMSE/MBE/NSE/R² error metrics (airSpring S64) are the shared validation vocabulary. Brent root-finding (airSpring Richards PDE) powers groundSpring band-edge refinement. L-BFGS flows into freeze-out QCD curve fitting.

5. **groundSpring validation → all springs**: 13-tier tolerance architecture (V73) adopted by wetSpring (164 tiers for metagenomics). `if let Ok` + CPU fallback became the wateringHole standard. Three-mode validation (local/barracuda/barracuda-gpu) adopted by all springs.

6. **hotSpring 4D spectral → groundSpring tissue**: `anderson_3d_correlated` → `anderson_4d` + `wegner_block_4d` (hotSpring S84) powers groundSpring tissue immunology (Paper 12) with cytokine concentration as 4th dimension.

### Precision Evolution

| Phase | When | What | Impact |
|-------|------|------|--------|
| f32 default | Pre-S46 | Native f32 WGSL | Limited precision |
| f64 canonical | S46-S48 | hotSpring f64 WGSL | First f64 shader |
| DF64 core | S58 | hotSpring `df64_core.wgsl` | f64-class on consumer GPUs |
| DF64 transcendentals | S60 | sqrt, exp, log, sin, cos | Full DF64 math |
| `Fp64Strategy` | S58 | Native vs DF64 routing | Per-hardware selection |
| `PrecisionRoutingAdvice` | V84-V85 | F64Native / NoSharedMem / Df64 / F32 | groundSpring → toadStool → all |
| Precision-routed dispatch | V96 | `get_device_f64_safe()` guards | 11 GPU paths hardware-aware |

---

## 7. Absorption and Evolution Guidance

### What groundSpring Has Contributed (for SPRING_ABSORPTION_TRACKER)

| Contribution | Origin | Absorbed Into | Current Status |
|--------------|--------|---------------|----------------|
| `PrecisionRoutingAdvice` enum | V84-V85 | barraCuda `device::driver_profile` | Round-tripped back; V96 wires into 11 paths |
| `f64_shared_memory_reliable` flag | V84-V85 | `GpuAdapterInfo` | Prevents naga `var<workgroup>` f64 zero bug |
| `sovereign_binary_capable` | V85 | `HardwareFingerprint` | coralDriver readiness flag |
| 13-tier tolerance architecture | V73 | wateringHole standard | wetSpring adopted 164 tiers; pattern proven |
| `if let Ok` + CPU fallback | V20+ | wateringHole standard | All springs use this pattern |
| Three-mode validation | V29+ | wateringHole standard | local/barracuda/barracuda-gpu adopted by all |
| Shaders (anderson_lyapunov, chi_squared, welford) | V68-V80 | barraCuda ops | Absorbed; metalForge retains 2 reference shaders |

### SPRING_ABSORPTION_TRACKER Update Request

| Field | Current (stale) | Correct (V96) |
|-------|-----------------|---------------|
| groundSpring version | V85 | **V96** |
| Tests | 812 | **925** |
| Delegations | 87 (51 CPU + 36 GPU) | **102 (61 CPU + 41 GPU)** |
| barraCuda pin | — | `2a6c072` |
| New since V85 | — | PrecisionRoutingAdvice wired, Shannon delegation, CorrelationFull API, FFT (Fft1DF64), smart refactoring, Exp 023/024 GPU wired, AutocorrelationF64 consumed |

### Available but Not Yet Consumed

| barraCuda Primitive | groundSpring Status | When to Wire |
|---------------------|---------------------|--------------|
| `BatchedOdeRK45F64` | Available | When an experiment needs adaptive ODE stepping |
| `GpuView<T>` | Available | When ops accept `&GpuView<T>` for zero-readback chains |
| `mean_variance_to_buffer()` | Available | GPU-resident pipelines (no host readback) |
| `CorrelationF64::correlation_full_buffer()` | Available | Zero-copy chained dispatch |
| `LSCFRK` integrators | Not applicable | Lattice QCD gradient flow only |

### Open Evolution Requests (Priority Order)

| Priority | Request | Owner | Detail |
|----------|---------|-------|--------|
| **P0** | Fix `Fp64Strategy` in `SumReduceF64`/`VarianceReduceF64` | barraCuda | Hybrid/Native branching should select DF64 shader on Hybrid devices |
| **P1** | Update SPRING_ABSORPTION_TRACKER | toadStool | groundSpring V85→V96, 87→102 delegations, 812→925 tests |
| **P1** | Wire `shader.compile.*` IPC to live coralReef | toadStool | Currently returns "not yet available" |
| **P1** | Add FFT to `SubstrateCapabilityKind` | toadStool | groundSpring uses `Fft1DF64` for spectral reconstruction |
| **P2** | Ops accepting `&GpuView<T>` | barraCuda | Enables zero-readback chains for pipeline dispatch |

### groundSpring Validation Surface

groundSpring's 35 experiments across 10 scientific domains provide a unique
validation surface for barraCuda: every experiment has a Python baseline, a
Rust CPU path, and (for 27/35) a GPU dispatch path — meaning every consumed
barraCuda primitive is validated against an independent mathematical reference.

Exp 025-027 (WDM) are especially valuable: they prove barraCuda's cross-vendor
GPU results agree at 1e-12 relative level, and that f32→f64 precision drift
is quantified at ~28% bias fraction for Green-Kubo integration.
