<!-- SPDX-License-Identifier: AGPL-3.0-only -->
<!-- Copyright (C) 2026 ecoPrimals / Squirrel Team -->

# groundSpring → ToadStool V66: Stats Tier A GPU + Bistable Batch ODE

**Date**: March 2, 2026
**From**: groundSpring (V66)
**To**: ToadStool / BarraCUDA team
**groundSpring Version**: V66 (stats GPU Tier A + bistable batch + metalForge expansion)
**ToadStool Pin**: S79 (`f97fc2ae`)
**Supersedes**: V65 (docs sweep + comprehensive absorption handoff)
**Tests**: 776 workspace (414 lib + 362 integration/validation) + 1 doc-test
**Clippy**: Clean (zero warnings, `clippy::pedantic` + `clippy::nursery`)
**Docs**: Clean (`cargo doc --no-deps`)
**Format**: Clean (`cargo fmt --all -- --check`)
**License**: AGPL-3.0-only (unified)

---

## Executive Summary

- **71 active delegations** (43 CPU + 28 GPU) — up from 67 (37 CPU + 26 GPU)
- Stats Tier A completed: MAE, NSE, R² now GPU-dispatched via `FusedMapReduceF64`. Papers 1-5 sensor/meteorology/ET₀/sequencing/seismic stats are fully GPU-capable.
- Bistable ODE batch integration via `BatchedOdeRK4F64` — parallel RK4 trajectories on GPU for Paper 10 (Fernandez 2020 bistable switching)
- 26 metalForge workloads (was 23), 26 tolerance specs — cross-substrate validation complete
- 776 tests (was 771), 5 new three-tier GPU parity tests
- V65 zero-debt certification preserved: zero unsafe, zero TODO, zero `.unwrap()`, zero mocks, zero bare `#[allow]`

---

## Part 1: New GPU Delegations (V66)

### 1.1 Stats Tier A Completion (Papers 1-5)

| Function | Barracuda API | GPU Dispatch Path |
|----------|--------------|-------------------|
| `stats::agreement::mae` | `FusedMapReduceF64::l1_norm` | Residuals → L1 norm → divide by N |
| `stats::agreement::nash_sutcliffe` | `FusedMapReduceF64::sum_of_squares` × 2 | SS_res and SS_tot via dual reductions |
| `stats::agreement::r_squared` | `FusedMapReduceF64::sum_of_squares` × 2 | Same as NSE (mathematically identical) |

**Why this matters**: These are the core agreement metrics for every sensor calibration experiment. MAE, NSE/R² are computed in Exp 001-005, Exp 022-024, and every validation binary. With GPU dispatch, large-N sensor arrays benefit from parallel reduction.

**Pattern**: Precompute residuals on CPU (O(N) scalar subtraction), dispatch reduction to GPU (the expensive part). This is the same pattern used for `rmse_gpu` and `mbe_gpu`.

### 1.2 Bistable Batch ODE (Paper 10)

| Function | Barracuda API | GPU Dispatch Path |
|----------|--------------|-------------------|
| `bistable::integrate_batch` | `BatchedOdeRK4F64::integrate` | Flat `[B×5]` states + `[B×17]` params |

**Configuration**: Uses `BatchedRk4Config` with `h` (step size), `n_batches`, `n_steps`, `clamp_min=0.0`, `clamp_max=1e6`. The `to_flat()` method on `BistableParams` produces a 21-element array; only the first 17 elements (matching `N_PARAMS`) are passed to the GPU kernel.

**CPU fallback**: Sequential `ode::integrate` per initial condition.

### 1.3 Multi-Signal Batch (Paper 11 — CPU path)

| Function | Status | Notes |
|----------|--------|-------|
| `multisignal::integrate_batch` | CPU only | 7-variable ODE, no GPU kernel yet |

**toadStool action**: The multi-signal cooperation ODE (`cooperation_ode_rk4_f64.wgsl`) exists in barracuda's shader library. A `BatchedOdeRK4F64` variant with `N_VARS=7` and `N_PARAMS=27` would enable GPU promotion for this workload.

---

## Part 2: New metalForge Workloads

| Workload | Capabilities | Tolerance Tier |
|----------|-------------|----------------|
| MAE (GPU fused) | `F64Compute`, `ShaderDispatch` | Analytical |
| NSE/R² (GPU fused) | `F64Compute`, `ShaderDispatch` | Analytical |
| Bistable ODE batch (GPU RK4) | `F64Compute`, `ShaderDispatch` | Statistical |

Total workloads: 26 (was 23). Total tolerances: 26 (was 23).

---

## Part 3: Barracuda API Usage Review

### 3.1 APIs Currently Consumed (V66 inventory)

| Barracuda Module | groundSpring Functions Using It | Count |
|-----------------|-------------------------------|-------|
| `barracuda::stats::*` | mean, std_dev, rmse, mae, mbe, nse, r², ia, hit_rate, pearson, spearman, covariance, norm_cdf, norm_ppf, chi2, percentile, moving_window_stats, bootstrap_mean, rawr_mean, bootstrap_median, bootstrap_std | 21 |
| `barracuda::stats::regression::*` | fit_linear, fit_quadratic, fit_exponential, fit_logarithmic | 4 |
| `barracuda::stats::hydrology::*` | fao56_et0, hargreaves_et0, crop_coefficient, soil_water_balance | 4 |
| `barracuda::stats::diversity::*` | chao1_classic | 1 |
| `barracuda::stats::evolution::*` | kimura_fixation_prob, error_threshold, detection_power, detection_threshold | 4 |
| `barracuda::stats::hill`, `monod` | hill, monod | 2 |
| `barracuda::stats::*` (other) | bray_curtis, rarefaction_curve, pielou_evenness, shannon, simpson | 5 |
| `barracuda::stats::jackknife::*` | jackknife_mean_variance, JackknifeMeanGpu | 2 |
| `barracuda::numerical::*` | BistableOde::cpu_derivative, MultiSignalOde::cpu_derivative, trapz | 3 |
| `barracuda::special::anderson_transport::*` | localization_length | 1 |
| `barracuda::spectral::*` | lyapunov_exponent, lyapunov_averaged, anderson_sweep_averaged, almost_mathieu_hamiltonian, find_all_eigenvalues, level_spacing_ratio, detect_bands, lanczos, anderson_3d_correlated, find_w_c | 10 |
| `barracuda::optimize::*` | brent | 1 |
| `barracuda::linalg::*` | eigh_f64, cholesky_f64, solve_f64_cpu | 3 |
| `barracuda::ops::*` (GPU) | FusedMapReduceF64, SumReduceF64, VarianceReduceF64, CorrelationF64, BatchedMultinomialGpu, GillespieGpu, WrightFisherGpu, BatchedElementwiseF64, HargreavesBatchGpu, BatchedOdeRK4F64, grid_search_3d | 11 |
| **Total** | | **72** |

### 3.2 APIs Not Yet Consumed (Absorption Candidates)

| Barracuda API | Potential groundSpring Use | Priority |
|---------------|--------------------------|----------|
| `BatchedOdeRK4F64` variant (7-var) | multisignal ODE batch GPU | MEDIUM |
| `batched_multinomial` (Tier C) | rarefaction GPU batch | HIGH |
| FFT (real, complex) | Spectral recon (Paper 6 Bazavov) | MEDIUM |
| `PrngXoshiro` (public CPU-side) | PRNG alignment across springs | HIGH |
| Eigenvector solver (tridiag QL) | Transport eigenvectors GPU | MEDIUM |

### 3.3 PRNG Alignment Status

groundSpring uses `Xorshift64` (CPU) and `Xoshiro128**` (GPU-aligned) via local implementations. The ecosystem goal is unified `PrngXoshiro` from barracuda.

**toadStool action**: Export `PrngXoshiro` as a public CPU-side struct so springs can replace local PRNG implementations. This is the single largest remaining architectural divergence.

---

## Part 4: What groundSpring Learned for ToadStool

### 4.1 Dual-Reduction Pattern for R²/NSE

The `coefficient_of_efficiency_gpu` function demonstrates a pattern for complex metrics needing multiple reductions:

1. Compute mean via `SumReduceF64::mean` (one GPU dispatch)
2. Compute SS_res via `FusedMapReduceF64::sum_of_squares` on residuals (one dispatch)
3. Compute SS_tot via `FusedMapReduceF64::sum_of_squares` on deviations (one dispatch)

Three dispatches total. For very large N, a fused kernel computing both SS_res and SS_tot in a single pass would halve the dispatches.

**toadStool action**: Consider a `DualReduceF64` op that computes two sum-of-squares simultaneously from two input vectors. This would optimize R², NSE, index_of_agreement, and any coefficient-of-efficiency variant.

### 4.2 ODE Batch Parameter Truncation

`BistableParams::to_flat()` returns 21 elements, but `BatchedOdeRK4F64::N_PARAMS` is 17. groundSpring truncates to `[..17]`. This works because the GPU shader only reads the first 17 parameters per trajectory.

**toadStool action**: Document which of the 21 flat params map to which shader uniform. The gap (params 17-20) represents feedback parameters that the GPU shader doesn't yet implement.

### 4.3 CPU-Side Residual Precomputation

For MAE and RMSE GPU paths, groundSpring precomputes the residual vector on CPU before dispatching the reduction to GPU. This is optimal for N < ~100k where the transfer cost dominates. For N > 1M, a fused GPU kernel taking two input vectors would eliminate the CPU→GPU transfer of residuals.

**toadStool action**: Consider `PairedReduceF64` taking `(observed, modeled)` directly and computing the residual map + reduction in a single pass.

---

## Part 5: Paper Queue × GPU Tier Status

| Paper | Experiment | CPU | GPU | Blocker |
|-------|-----------|:---:|:---:|---------|
| 1-5 | Sensor, gap, ET₀, seq, seismic | **PASS** | **Tier A complete** (V66) | None |
| 6 | Bazavov spectral recon | **PASS** | Cholesky wired | Dense linear algebra (highest GPU potential) |
| 7 | Bazavov jackknife | **PASS** | **Wired** (JackknifeMeanGpu) | None |
| 8 | Bazavov freeze-out | **PASS** | **Wired** (grid_search_3d) | None |
| 9-11 | Waters bio (c-di-GMP, bistable, QS) | **PASS** | **Batch wired** (V66) | Multi-signal GPU needs 7-var ODE |
| 12-13 | Liu resampling | **PASS** | CPU delegation | Embarrassingly parallel |
| 14 | Dolson eco-evo | **PASS** | WrightFisherGpu | None |
| 15-18 | Kachkovskiy spectral | **PASS** | **Wired** (spectral::*) | None |
| 20-21 | R. Anderson evolution | **PASS** | Partial (SmithWaterman, BrayCurtis) | batched_multinomial |
| 22-24 | Sub-thesis 06 cross-spring | **PASS** | Uses composed GPU ops | None |
| 25-27 | Sub-thesis 07 WDM | **PASS** | Analytical | None |
| 28 | NPU Anderson | **PASS** | N/A (NPU) | AKD1000 hardware |

---

## Part 6: Action Items for ToadStool

1. **Export `PrngXoshiro` as public CPU-side struct** — eliminates PRNG divergence across all springs (HIGH)
2. **7-variable `BatchedOdeRK4F64` variant** — enables multi-signal ODE GPU batch (MEDIUM)
3. **`DualReduceF64` fused op** — two sum-of-squares in one pass for R²/NSE (LOW)
4. **`PairedReduceF64` op** — (observed, modeled) → reduction without CPU residual precompute (LOW)
5. **Document `BatchedOdeRK4F64` param layout** — which flat indices map to which ODE params (LOW)
6. **`batched_multinomial` Tier C absorption** — rarefaction GPU batch, rare biosphere (HIGH)
