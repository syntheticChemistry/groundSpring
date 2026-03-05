# SPDX-License-Identifier: AGPL-3.0-only

# groundSpring → toadStool/barraCuda: Absorption Guide + Evolution Intelligence

**Date:** 2026-03-04
**From:** groundSpring V73 (33 experiments, 376/376 checks, 81 delegations)
**To:** barraCuda team (math primitives), toadStool team (hardware dispatch)
**License:** AGPL-3.0-only

---

## Purpose

This document captures everything groundSpring has learned about barraCuda
usage patterns, gaps, precision behavior, and architectural insights that
should inform barraCuda and toadStool evolution. It is a one-time comprehensive
transfer — a "here is what we know" from the consumer side.

---

## 1. What groundSpring Is

groundSpring is the **uncertainty quantification** spring — it asks "how
confident are we in these numbers?" across 10 scientific domains. It validates
Python baselines against Rust implementations and GPU acceleration. It is
the most diverse barraCuda consumer: 33 experiments spanning agricultural
sensing, spectral theory, evolutionary biology, inverse problems, WDM physics,
NPU hardware, and cross-spring uncertainty bridges.

### Evolution path consumed

```
Python baseline → Rust (pure safe) → barraCuda CPU → barraCuda GPU → metalForge cross-substrate
```

All 81 delegations follow this path. The CPU implementation is the validation
reference; the GPU implementation is for throughput.

---

## 2. Delegation Inventory (81 primitives)

### CPU delegations (47) — `#[cfg(feature = "barracuda")]`

| Domain | Primitives |
|--------|-----------|
| Stats (agreement) | `rmse`, `mae`, `nash_sutcliffe`, `mbe`, `r_squared`, `index_of_agreement`, `hit_rate` |
| Stats (correlation) | `pearson_r`, `spearman_r`, `covariance`, `std_dev` |
| Stats (distributions) | `norm_cdf`, `norm_ppf`, `chi2_decomposed` |
| Stats (metrics) | `mean`, `percentile`, `std_dev`, `sample_std_dev` |
| Stats (regression) | `fit_linear`, `fit_quadratic`, `fit_exponential`, `fit_logarithmic` |
| Stats (moving window) | `moving_window_stats_f64` |
| Bootstrap | `bootstrap_mean`, `rawr_mean`, `bootstrap_median`, `bootstrap_std` |
| Jackknife | `jackknife_mean_variance` |
| Bio diversity | `simpson`, `shannon`, `bray_curtis`, `pielou_evenness`, `rarefaction_curve` |
| Bio evolution | `kimura_fixation_prob`, `error_threshold`, `detection_power`, `detection_threshold` |
| Kinetics | `hill`, `monod` |
| Hydrology | `fao56_et0`, `hargreaves_et0`, `crop_coefficient`, `soil_water_balance` |
| Anderson | `localization_length`, `lyapunov_exponent`, `lyapunov_averaged` |
| Linalg | `solve_f64_cpu` |

### GPU delegations (34) — `#[cfg(feature = "barracuda-gpu")]`

| Domain | Primitives |
|--------|-----------|
| Anderson spectral | `anderson_sweep_averaged`, `anderson_2d`, `anderson_3d`, `almost_mathieu_hamiltonian`, `find_all_eigenvalues`, `level_spacing_ratio`, `anderson_3d_correlated`, `anderson_4d`, `wegner_block_4d` |
| Band structure | `detect_bands`, `brent` |
| Linalg (GPU) | `eigh_f64`, `cholesky_f64` |
| Optimize | `lbfgs_numerical`, `grid_search_3d`, `batched_nelder_mead_gpu` |
| ODE batch | `BatchedOdeRK4F64`, `BistableOde::cpu_derivative`, `MultiSignalOde::cpu_derivative` |
| Bio batch | `WrightFisherGpu`, `GillespieGpu`, `BatchedMultinomialGpu` |
| Hydrology batch | `BatchedElementwiseF64`, `HargreavesBatchGpu`, `SeasonalPipelineF64`, `StatefulPipeline`, `WaterBalanceState`, `McEt0PropagateGpu` |
| Jackknife GPU | `JackknifeMeanGpu` |
| Lanczos | `SpectralCsrMatrix`, `lanczos`, `lanczos_eigenvalues` |
| ESN | `ESN`, `ESNConfig` |
| GPU reduce | `SumReduceF64`, `VarianceReduceF64`, `FusedMapReduceF64`, `CorrelationF64` |
| Device | `WgpuDevice`, `test_pool::tokio_block_on` |

---

## 3. Patterns That Work Well

### Graceful fallback pattern

Every delegation site follows:

```rust
#[cfg(feature = "barracuda")]
{
    if let Ok(result) = barracuda::stats::rmse(observed, predicted) {
        result
    } else {
        local_rmse(observed, predicted)
    }
}
#[cfg(not(feature = "barracuda"))]
{
    local_rmse(observed, predicted)
}
```

The CPU path is **always compiled**. The barracuda path is optional. This
means groundSpring works standalone (zero external deps) and gains
performance when barracuda is available.

### GPU exit guard

```rust
fn exit_no_gpu() {
    if std::env::var("BARRACUDA_REQUIRE_GPU").is_ok() {
        std::process::exit(1);
    }
    std::process::exit(0);
}
```

Validation binaries that require GPU exit cleanly in CI without GPU hardware.
The `BARRACUDA_REQUIRE_GPU=1` env var makes the skip a failure for GPU CI.

### Feature gating pattern

```toml
[features]
barracuda = ["dep:barracuda"]
barracuda-gpu = ["barracuda", "barracuda/gpu"]
```

Two tiers: CPU-only delegation and GPU delegation. GPU implies CPU.

---

## 4. Gaps + Evolution Requests

### P1: FFT (real, complex)

`spectral_recon.rs` (Exp 021) does Tikhonov regularization. A GPU FFT
would enable full spectral reconstruction on GPU. Currently the Cholesky
GPU path works but FFT would unlock the optimal algorithm.

### P1: Eigenvector solver (tridiag)

`transport.rs` (Exp 012) needs eigenvectors, not just eigenvalues.
barraCuda's Sturm solver returns eigenvalues only. A tridiag eigenvector
solver (inverse iteration or divide-and-conquer) would promote transport
from Tier B to Tier A.

### P2: PRNG alignment

groundSpring uses `xorshift64` (local). barraCuda uses `xoshiro128**`.
Full baseline regeneration required to switch. Low priority but needed
for bitwise GPU-CPU reproducibility.

### P2: Parallel 3D grid dispatch

`seismic.rs` (Exp 005) does 3D grid search. A GPU parallel grid dispatch
would be straightforward and high-throughput.

### P3: `unified_hardware::ComputeScheduler` public API

metalForge manually routes across CPU/GPU/NPU substrates. If toadStool
exposes a `ComputeScheduler`, springs could delegate substrate selection
entirely.

### P3: Structured benchmark output

Validation binaries print text. A `BenchmarkReport` struct that emits
JSON would enable automated regression tracking.

---

## 5. Precision Insights from 33 Experiments

### Tolerance tiers discovered

Through 33 experiments across 10 domains, groundSpring discovered that
numerical tolerances naturally cluster into 13 tiers (see V73 handoff).
The key insight for barraCuda:

- **GPU introduces ~1 extra ULP** per transcendental vs CPU. If CPU
  achieves `tol::EXACT` (1e-12), GPU typically achieves `tol::ANALYTICAL`
  (1e-10) — one tier looser.
- **Batch dispatch does not degrade precision** for independent operations
  (embarrassingly parallel). `BatchedElementwiseF64` matches scalar.
- **Reduce operations lose ~1 tier** due to non-deterministic summation
  order. `SumReduceF64` is `tol::ANALYTICAL` where sequential sum is
  `tol::EXACT`.
- **ODE integration accumulates** — RK4 with dt=0.01 over 1000 steps
  reaches `tol::INTEGRATION` (1e-8). GPU and CPU agree at this level.

### Division-by-zero is the #1 GPU NaN source

Every GPU kernel that divides should guard with `eps::SAFE_DIV` (1e-10)
or equivalent. groundSpring found 3 production sites where inline guards
were needed: Wright-Fisher mean fitness, Gillespie SSA rate, Anderson
condition number.

### f32 vs f64 matters

Exp 025 proves f32 accumulation in Green-Kubo transport coefficients
introduces ~28% systematic bias vs f64. This is not random noise — it is
a direction-dependent drift. All scientific compute must use f64.

---

## 6. What groundSpring Does NOT Need from barraCuda

### Chao1 estimator

groundSpring uses classic Chao 1984 (`f₁²/(2f₂)` with integer counting).
barraCuda uses bias-corrected Chao & Chiu 2016 (`f₁(f₁−1)/(2(f₂+1))`).
Delegation would break Python baseline provenance. This stays local.

### PRNG core

groundSpring's `xorshift64` is a deliberate choice for reproducibility
against Python baselines using the same algorithm. GPU PRNG uses
`xoshiro128**` from barraCuda shaders. The two do not need to match.

### akida-driver

NPU hardware access stays in toadStool permanently. groundSpring's `npu`
module calls toadStool's akida-driver, not barraCuda.

---

## 7. Cross-Spring Intelligence

### What groundSpring learned from hotSpring

- The Write → Absorb → Lean cycle works. 81 delegations prove it scales.
- metalForge cross-substrate validation (CPU = GPU = NPU within tolerance)
  is the ultimate proof of mathematical portability.
- The `exit_no_gpu()` pattern prevents CI breakage without GPU hardware.

### What groundSpring learned from wetSpring

- Named tolerances at scale (wetSpring has 164) make codebase audits
  tractable. groundSpring adopted the pattern and now has 13 tiers.
- Streaming validation (wetSpring's temporal streaming at 12.9K Hz on NPU)
  informed groundSpring's NestGate pipeline design.

### What groundSpring learned from airSpring

- Local WGSL shaders (`local_elementwise.wgsl`) demonstrate the pre-absorption
  pattern. groundSpring's 2 remaining Anderson Lyapunov shaders follow this.
- `SeasonalPipelineF64` (fused multi-stage GPU) is the evolution target for
  groundSpring's FAO-56 pipeline — absorb don't reinvent.

### What other springs should learn from groundSpring

- **Tolerance tiers**: The 13-tier `tol::` module is immediately adoptable.
  Every spring should name its tolerances.
- **`eps::` guards**: Production division guards should be named constants,
  not inline `1e-10`.
- **Capability-based discovery**: Socket discovery should scan for
  capabilities, not hardcode primal names.
- **BTreeMap for deterministic iteration**: Any map whose iteration order
  affects output must be `BTreeMap`.
- **Silent defaults are bugs**: `unwrap_or(0.0)` on scientific data is data
  corruption. Use `let Some(...) else { continue }` or `expect()`.

---

## 8. Absorption Readiness

### Shaders ready for toadStool absorption

| Shader | Location | Status |
|--------|----------|--------|
| `anderson_lyapunov.wgsl` | `metalForge/shaders/` | f64, production quality, unique to groundSpring |
| `anderson_lyapunov_f32.wgsl` | `metalForge/shaders/` | f32 fallback for NAK/NVVM |

These are reference shaders for Anderson Lyapunov exponent computation.
They could be absorbed into barraCuda's spectral module as
`ops::anderson_lyapunov_f64` and `ops::anderson_lyapunov_f32`.

### Already absorbed (historical)

| Shader | Absorbed Into | Version |
|--------|--------------|---------|
| `mc_et0_propagate.wgsl` | `McEt0PropagateGpu` | S72 |
| `batched_multinomial.wgsl` | `BatchedMultinomialGpu` | S76 |

---

*groundSpring V73 → barraCuda/toadStool absorption guide.
81 delegations, 13 tolerance tiers, 33 experiments, 10 domains.
The uncertainty budget for the entire ecoPrimals ecosystem.*
