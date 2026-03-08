<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# groundSpring V97 → toadStool/barraCuda/coralReef GPU Smoke Test Handoff

**Date**: March 7, 2026
**From**: groundSpring V97 (936 tests, 102 delegations, 0 failures, three-tier parity proven)
**To**: barraCuda team, toadStool team, coralReef team
**Supersedes**: V96 handoff (Mar 7, 2026)
**Synced against**: barraCuda `2a6c072`, toadStool S130 (`88a545df`), coralReef Iteration 7 (`72e6d13`)
**License**: AGPL-3.0-only

## Executive Summary

groundSpring V97 is a **GPU correctness release** that adds a runtime f64
reduction smoke test and completes the three-tier parity proof: all 29
validation binaries now PASS at default CPU, barracuda-CPU, and
barracuda-GPU tiers.

**What changed since V96:**
- **Runtime f64 reduction smoke test**: `f64_reductions_safe()` now runs
  `mean([1.0; 4])` on GPU and verifies the result is 1.0. Cached in
  `OnceLock<bool>`, runs once per process. Detects GPUs where
  `GpuDriverProfile` says `F64Native` but naga/SPIR-V workgroup
  shared-memory f64 reductions silently produce zeros
- **21 GPU dispatch paths guarded**: 11 stochastic/reduction sites changed
  from `get_device()` to `get_device_f64_safe()`
- **936 tests** (was 925), all quality gates pass
- **382 Python correctness tests** pass (zero baseline drift)
- **Three-tier parity**: 29/29 at all 3 tiers (CPU, barracuda-CPU, barracuda-GPU)

*This handoff is unidirectional: groundSpring → ecosystem. No response expected.*

---

## 1. GPU Smoke Test Finding — Action Item for barraCuda

### Problem

The RTX 4070 (Ada Lovelace, proprietary NVIDIA driver 570.x) is classified
as `PrecisionRoutingAdvice::F64Native` by `GpuDriverProfile::from_device()`.
However, its naga/SPIR-V backend silently produces **zeros** for
`var<workgroup>` f64 shared-memory reductions (`SumReduceF64::mean`,
`VarianceReduceF64::population_std`, `grid_search_3d` argmin).

Per-element f64 operations (eigensolvers, Cholesky, ODE batch,
elementwise ET₀) work correctly — only workgroup shared-memory f64
is broken.

### Observed Behavior

```
# RTX 4070 (Ada, f64 1:64, proprietary driver)
SumReduceF64::mean([1.0, 1.0, 1.0, 1.0]) → 0.0  (expected: 1.0)
VarianceReduceF64::population_std(data)    → 0.0  (expected: non-zero)
grid_search_3d(rms_values)                 → wrong argmin indices

# Titan V (Volta, f64 1:2, NVK driver)
SumReduceF64::mean([1.0, 1.0, 1.0, 1.0]) → 1.0  ✓
VarianceReduceF64::population_std(data)    → correct  ✓
```

### groundSpring Workaround

Added a runtime smoke test to `gpu::f64_reductions_safe()`:

```rust
fn f64_reduction_smoke_test() -> bool {
    let Some(device) = get_device() else { return false };
    let test_data = [1.0_f64; 4];
    let Ok(result) = SumReduceF64::mean(device, &test_data) else { return false };
    (result - 1.0).abs() < 0.01
}
```

### P0 Action for barraCuda

The `GpuDriverProfile` classification for Ada Lovelace + proprietary
driver should be `F64NativeNoSharedMem`, not `F64Native`. The
shared-memory f64 path is broken on this hardware/driver combination.

**Affected users**: Any consumer Ada GPU (RTX 4060/4070/4080/4090) on
the proprietary NVIDIA driver. These are the most common discrete GPUs
in the ecosystem.

### P1 Action for toadStool

Consider adding a similar runtime verification to toadStool's
`ComputeDispatch` pipeline. When a dispatch includes workgroup f64
reductions, verify the device passes a smoke test before routing to GPU.

---

## 2. GPU Dispatch Path Summary

### Now guarded by `get_device_f64_safe()`

| Module | Function | barraCuda Op |
|--------|----------|-------------|
| `stats/metrics.rs` | `mean_gpu` | `SumReduceF64::mean` |
| `stats/metrics.rs` | `std_dev_gpu` | `VarianceReduceF64::population_std` |
| `stats/agreement.rs` | `mbe_gpu` | `SumReduceF64::mean` |
| `fao56/pipeline.rs` | `monte_carlo_et0_gpu` | `McEt0PropagateGpu` |
| `fao56/pipeline.rs` | `seasonal_step_gpu` | `SeasonalPipelineF64` |
| `fao56/pipeline.rs` | `mc_mean_variance_gpu` | `VarianceF64` |
| `gillespie.rs` | `birth_death_ssa_batch_gpu` | `GillespieGpu` |
| `bootstrap.rs` | `bootstrap_mean_gpu` | `BootstrapMeanGpu` |
| `rarefaction/sampling.rs` | `multinomial_sample_batch_gpu` | `BatchedMultinomialGpu` |
| `rare_biosphere.rs` | `abundance_occupancy_gpu` | `BatchedMultinomialGpu` |
| `rare_biosphere.rs` | `tier_detection_rate_gpu` | `BatchedMultinomialGpu` |
| `drift/mod.rs` | `wf_batch_gpu` | `WrightFisherGpu` |
| `seismic.rs` | `grid_search_inversion_gpu` | `grid_search_3d` |

### Still using `get_device()` (per-element, no shared memory — work correctly)

| Module | Function | barraCuda Op |
|--------|----------|-------------|
| `anderson/spectral.rs` | Eigenvalue sweep | `spectral::*` |
| `spectral_recon.rs` | Cholesky solve | `linalg::cholesky_f64` |
| `spectral_recon.rs` | Tikhonov solve | `linalg::tikhonov_solve_gpu` |
| `freeze_out.rs` | Grid search + L-BFGS | `ops::grid`, `optimize::lbfgs_numerical` |
| `bistable.rs` | ODE batch | `BatchedOdeRK4F64` |
| `fao56/mod.rs` | ET₀ batch (×2) | `BatchedElementwiseF64`, `HargreavesBatchGpu` |

---

## 3. Three-Tier Validation Results

| Tier | Validation | Result |
|------|-----------|--------|
| Python baselines | 382 correctness tests | **382/382 PASS** |
| Rust default (CPU) | 936 workspace tests | **936/936 PASS** |
| Rust default (CPU) | 29 validation binaries | **29/29 PASS** |
| BarraCUDA CPU | 29 validation binaries | **29/29 PASS** |
| BarraCUDA GPU | 29 validation binaries | **29/29 PASS** |
| metalForge | 140 tests + 8 binaries | **148/148 PASS** |
| Clippy pedantic+nursery | workspace + barracuda-gpu | **0 warnings** |

### Titan V Verification

With `WGPU_ADAPTER_NAME="NVIDIA TITAN V"`, the smoke test passes and
all GPU ops run on native f64 hardware. Spot-checked 7 previously-failing
experiments (signal-specificity, fao56, et0-anderson, weather, seismic,
decompose, anderson) — all PASS.

---

## 4. Kokkos Parity Baseline

```
groundSpring Rust Tier 2 Baseline (CPU, default features)

Anderson Localization: gamma_avg = 0.1579244366  (xi = 6.332)
  Derrida-Gardner:     xi ~ 96/W^2 = 6.0000
Statistical Reductions: mean, variance, pearson_r — verified
Bootstrap:             estimate = 25.000015, CI covers true mean
```

JSON output in `data/` (gitignored) for comparison with Kokkos reference.

---

## 5. Evolution Requests

### P0 — barraCuda GpuDriverProfile Fix

Reclassify Ada Lovelace + proprietary NVIDIA driver as
`F64NativeNoSharedMem`. The current `F64Native` classification causes
silent GPU zeros for shared-memory f64 reductions.

### P1 — toadStool Runtime Verification

Add a one-time GPU reduction smoke test to the `ComputeDispatch`
pipeline. This protects all springs, not just groundSpring.

### P2 — Eigenvector Solver (Paper 17)

Paper 17 (Kachkovskiy transport) has eigenvalues on GPU via Sturm but
eigenvectors still CPU-only (`tridiag_eigh` QL algorithm). A GPU
tridiagonal eigenvector solver would complete the spectral GPU tier.

---

## 6. Quality Gates

```
cargo fmt --check                 → PASS
cargo clippy --workspace          → 0 warnings (pedantic + nursery)
cargo clippy --features gpu       → 0 warnings
cargo doc --workspace --no-deps   → PASS
cargo test --workspace            → 936 passed, 0 failed
validation binaries (×3 tiers)    → 87/87 PASS (29 × 3 tiers)
Python correctness tests          → 382/382 PASS
```

---

## 7. Delegation Inventory (unchanged from V96)

102 active delegations (61 CPU + 41 GPU). No new delegations in V97 —
this release focused on correctness of existing GPU paths.

Full inventory in `specs/BARRACUDA_EVOLUTION.md`.
