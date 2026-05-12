# Cross-Spring Shader Evolution

**Last updated**: May 12, 2026 (V138 — 110 active delegations (67 CPU + 43 GPU), 1,130 tests, three-tier parity proven: 29/29 at all 3 tiers, barraCuda v0.3.13, guideStone L4)

The ecoPrimals shader ecosystem evolved organically as each spring
absorbed domain-specific knowledge, then shared it through barraCuda
(the standalone compute primal). This document maps the provenance and
cross-pollination of shaders across springs.

## Overview

barraCuda v0.3.13 has absorbed 708 WGSL shaders from five springs, compiled
through a universal precision pipeline (`compile_shader_universal()` +
naga IR rewrite) that gives every spring f64-class precision on any GPU.

```
hotSpring (physics)     ──→ DF64 core, spectral theory, Lanczos
wetSpring (biology)     ──→ bio primitives, diversity, ODE, Hill/Monod
neuralSpring (ML)       ──→ matmul, pairwise, swarm, batch IPR
airSpring (hydrology)   ──→ regression, metrics, moving window
groundSpring (noise)    ──→ batched multinomial, rawr, elementwise
                 ↓
           ToadStool/BarraCUDA S68+
           (universal precision, 700 shaders, zero f32-only)
```

## Shader Provenance Table

| Shader / Function | Origin Spring | Session | Other Springs Benefiting |
|---|---|---|---|
| `df64_core.wgsl` (DF64 precision) | hotSpring (biomeGate) | S58 | **all** — f64-class on consumer GPUs |
| `Fp64Strategy` auto-select | hotSpring | S58 | **all** — Native vs DF64 routing |
| `anderson.rs`, `lanczos.rs` | hotSpring spectral | S26 | groundSpring (localization), wetSpring (validation) |
| `hofstadter.rs` | hotSpring spectral | S26 | groundSpring (Almost-Mathieu) |
| `hermite_f64.wgsl`, `laguerre_f64.wgsl` | hotSpring nuclear | S26 | — |
| `su3_gauge_force_f64.wgsl` + lattice QCD suite | hotSpring HFB | S39–S64 | — |
| `batched_multinomial_f64.wgsl` | groundSpring metalForge | S64 | wetSpring (rarefaction) |
| `wright_fisher_step_f64.wgsl` | neuralSpring metalForge | S66 | groundSpring (quasispecies), wetSpring (population genetics) |
| `diversity.rs`, `bray_curtis_f64.wgsl` | wetSpring biodiversity | S64 | airSpring (sensor similarity), groundSpring (Shannon) |
| `regression.rs` (`fit_*`) | airSpring hydrology | S66 | groundSpring (WDM extrapolation) |
| `metrics.rs` (RMSE, MAE, NSE, etc.) | airSpring + groundSpring | S64 | **all** — universal stats |
| `rawr_mean` bootstrap | groundSpring bootstrap | S66 | wetSpring (rarefaction CI) |
| `pow_f64` polyfill fix | neuralSpring (Ada fix) | S-17 | **all** — unblocked Ada Lovelace |
| `math_f64.wgsl` precision fixes | wetSpring (`ldexp` fix) | S64 | **all** — f64 correctness |
| `hill_f64.wgsl`, `monod` | wetSpring QS/c-di-GMP | S68 | groundSpring (kinetics) |
| `esn_reservoir_update_f64.wgsl` | wetSpring → hotSpring | S26 | — |
| `GemmCached` (60× taxonomy speedup) | wetSpring optimization | S64 | — |
| `batch_ipr_f64.wgsl` | neuralSpring → hotSpring | S52 | hotSpring (spectral analysis) |
| `pairwise_l2_f64.wgsl`, `pairwise_hamming_f64.wgsl` | neuralSpring metalForge | S52 | wetSpring (sequence distance) |
| `smith_waterman_banded_f64.wgsl` | wetSpring → ToadStool | S27 | — |
| `felsenstein_f64.wgsl` | wetSpring phylogenetics | S27 | — |
| `gillespie_ssa_f64.wgsl` | wetSpring stochastic | S27 | groundSpring (Gillespie SSA) |
| `compile_shader_universal()` | ToadStool sovereign | S67 | **all** — precision routing |
| `op_preamble` + naga IR rewrite | ToadStool dual-layer | S68 | **all** — zero f32-only shaders |

## Key Cross-Pollination Cycles

### 1. hotSpring DF64 → All Springs

hotSpring's quantum chromodynamics work required 14+ decimal digits on
consumer GPUs that lack native f64. The DF64 technique (double-float via
f32 pairs) was developed for lattice QCD gauge forces, then generalized
into `df64_core.wgsl` during the biomeGate precision sprint (S58).

**Impact**: Every spring's GPU workloads now get f64-class precision on
RTX 4070 Ada. groundSpring's Anderson localization, wetSpring's diversity
fusion, and airSpring's batched ET₀ all benefit transparently.

### 2. wetSpring Bio → neuralSpring metalForge → All Springs

wetSpring developed biodiversity primitives (Shannon, Simpson, Bray-Curtis,
rarefaction) for microbiome analysis. neuralSpring's metalForge then
generalized these into GPU batch ops (`DiversityFusionGpu`,
`BatchedMultinomialGpu`, `WrightFisherGpu`) as part of the evolutionary
computation framework.

**Impact**: groundSpring's rare biosphere experiments delegate to
`BatchedMultinomialGpu` (GPU occupancy). The shader originated in
wetSpring, was hardened by neuralSpring, and serves groundSpring.

### 3. neuralSpring pow_f64 → airSpring + wetSpring

neuralSpring's S-17 discovery that Ada Lovelace GPUs crash on `pow(f64)`
through the NVVM/NAK compiler led to the polyfill patcher that replaces
`pow` with `exp(y * ln(x))` at the naga IR level.

**Impact**: Without this fix, airSpring's hydrology (ET₀ batch) and
wetSpring's ODE solvers were completely blocked on Ada GPUs. One fix in
neuralSpring unblocked two springs.

### 4. airSpring Regression → groundSpring WDM

airSpring developed `fit_linear`, `fit_quadratic`, `fit_exponential`, and
`fit_logarithmic` for soil moisture curve fitting and sensor calibration.
groundSpring delegates `finite_size_extrapolate` (Warm Dense Matter
analysis) to `fit_linear` for L→∞ extrapolation.

**Impact**: Domain expertise in agricultural sensor calibration directly
enables materials physics research.

### 5. groundSpring RAWR → wetSpring Rarefaction

groundSpring's Randomly-Adjusted Weighted Resampling (RAWR) bootstrap
was developed for measurement noise characterization. wetSpring uses it
for rarefaction confidence intervals.

**Impact**: Error quantification methodology flows from noise analysis
to biodiversity assessment.

## Absorption Timeline

| Session | Date | Spring Absorption | groundSpring Impact |
|---|---|---|---|
| S26 | Feb 21 | hotSpring spectral + ESN | Anderson, Almost-Mathieu delegations |
| S27 | Feb 21 | wetSpring + neuralSpring shaders | Bio primitive availability |
| S39 | Feb 22 | 7 neuralSpring + 3 wetSpring + 11 hotSpring | Base delegation surface |
| S52 | Feb 24 | 18 cross-spring items | batch IPR, pairwise distance |
| S58 | Feb 24 | hotSpring DF64, wetSpring ODE/NMF | DF64 infrastructure |
| S64 | Feb 25 | diversity, stats metrics | 6 new CPU delegations |
| S66 | Feb 26 | regression, hydrology, RAWR | 5 new CPU delegations |
| S67 | Feb 27 | `compile_shader_universal()` | Transparent precision upgrade |
| S68 | Feb 27 | Dual-layer precision, zero f32-only | Universal precision complete |

## Benchmark Results (V97, March 7, 2026)

Three-mode benchmark (`benchmark_cross_spring --release`):

### Cross-Spring Provenance Benchmark (barraCuda v0.3.13, toadStool S158+)

| Workload | Origin Spring | Time | Notes |
|---|---|---|---|
| Stats metrics (6, n=10K) | airSpring + groundSpring → S64 | 2.9s | GPU dispatch (precision-routed) |
| Fused mean+variance (n=50K) | hotSpring DF64 → Welford | 3.7ms | hotSpring `df64_core.wgsl` lineage |
| Bootstrap RAWR (n=5K, B=1K) | groundSpring → S66 | 30.3ms | RAWR resampling |
| Regression fits (3 models, n=1K) | airSpring → S66 | 25µs | Linear/quadratic/exponential |
| Shannon diversity (S=200) | wetSpring → S64 | 1.5ms | Pielou evenness included |
| ET₀ 5 methods (Uccle) | airSpring → barraCuda v0.3.2 | <1µs each | PM/Hargreaves/Makkink/Turc/Hamon |
| Anderson Lyapunov (L=200, R=500) | hotSpring → S26 | 2.4ms | Transfer matrix, spectral theory |

### CPU vs GPU Benchmark (bench_cpu_vs_gpu)

| Workload | CPU (ms) | GPU (ms) | Notes |
|---|---|---|---|
| Gillespie SSA (100 traj) | 5.2 | 23.3 | GPU overhead dominates at small batch |
| Wright-Fisher (100 trials) | 19.5 | 232.7 | GPU overhead; wins at 10K+ trials |
| Multinomial (200 reps × 6 taxa) | 1.0 | 4.4 | GPU crossover at ~1K reps |
| Covariance (5K pairs, GPU) | 0.43 | — | CovarianceF64 single-pass |
| Autocorrelation (10K, lag 200) | 0.40 | — | AutocorrelationF64 |
| Seismic grid (31×31×7) | 7210 | — | CPU-intensive, GPU candidate |

### Precision Routing (V98)

21 GPU dispatch paths check `PrecisionRoutingAdvice` via `get_device_f64_safe()` +
runtime f64 reduction smoke test:
- `F64Native` + smoke test PASS → proceed (workgroup f64 reductions verified)
- `Df64Only` → proceed (barraCuda routes DF64 shaders internally)
- `F64NativeNoSharedMem` → skip GPU, fall back to CPU (naga shared-mem f64 zeros bug)
- `F32Only` → skip GPU, fall back to CPU
- Smoke test FAIL → skip GPU regardless of driver profile (Ada Lovelace zeros bug)

### Three-Tier Parity (V98, March 8 2026)

| Tier | Count | Status |
|---|---|---|
| Default CPU (29 validation binaries) | 29/29 | **PASS** |
| BarraCUDA CPU (29 validation binaries) | 29/29 | **PASS** |
| BarraCUDA GPU (29 validation binaries) | 29/29 | **PASS** |
| Python correctness (396 tests) | 396/396 | **PASS** |
| Rust workspace (936 tests) | 936/936 | **PASS** |
| metalForge (140 tests) | 140/140 | **PASS** |

### Validation Binary Benchmark (29 binaries, release mode)

| Mode | Wall time (s) | Speedup |
|---|---|---|
| Local (no features) | 12.5 | baseline |
| BarraCUDA CPU | 18.4 | 0.68× (dispatch overhead on small workloads) |
| **BarraCUDA GPU** | **9.9** | **1.27×** |

## GPU Delegation Status (V98 — 110 delegations)

| Delegation | Status | Shader Origin |
|---|---|---|
| 61 CPU delegations (stats, bio, hydrology, linalg, spectral) | **WIRED** | Cross-spring via barraCuda |
| 41 GPU delegations (reductions, multinomial, ODE, optimization) | **WIRED** | Cross-spring via barraCuda GPU ops |
| 21 GPU paths with precision routing + smoke test | **WIRED** (V98) | `get_device_f64_safe()` + `f64_reduction_smoke_test()` |
| `BatchedOdeRK45F64` (adaptive RK45) | Available | wetSpring V95 → barraCuda |
| `GpuView<T>` (persistent GPU buffer) | Available | barraCuda pipeline |
| `LSCFRK` (lattice integrators) | Available | hotSpring lattice QCD |

## Cross-Spring Shader Provenance (V98)

### What Each Spring Contributed to barraCuda (784 WGSL shaders)

| Spring | Domain | Key Shaders | Consumed By |
|--------|--------|-------------|-------------|
| **hotSpring** | Precision, MD, QCD | `df64_core.wgsl`, `df64_transcendentals.wgsl`, `stress_virial_f64.wgsl`, `cg_kernels_f64.wgsl`, `esn_readout_f64.wgsl` | ALL springs (DF64 core is universal) |
| **wetSpring** | Bio, Diversity | `smith_waterman_banded_f64.wgsl`, `gillespie_ssa_f64.wgsl`, `fused_map_reduce_f64.wgsl`, `hmm_forward_f64.wgsl`, `bray_curtis_f64.wgsl` | neuralSpring, airSpring, hotSpring |
| **neuralSpring** | ML, Stats | `fused_chi_squared_f64.wgsl`, `fused_kl_divergence_f64.wgsl`, `matrix_correlation_f64.wgsl`, `linear_regression_f64.wgsl`, `batch_ipr_f64.wgsl` | ALL springs |
| **airSpring** | Hydrology | `hargreaves_et0_f64.wgsl`, `seasonal_pipeline.wgsl`, `moving_window_f64.wgsl`, `brent_f64.wgsl` | wetSpring, neuralSpring |
| **groundSpring** | Spectral, Uncertainty | `anderson_lyapunov_f64.wgsl`, `chi_squared_f64.wgsl`, `welford_mean_variance_f64.wgsl`, `mc_et0_propagate_f64.wgsl` | ALL springs (chi-squared universal) |

### Evolution Timeline

| Date | Event | From | To | Impact |
|------|-------|------|-----|--------|
| Feb 2026 | f32→f64 evolution | hotSpring S49 | ALL springs | All shaders become f64-canonical |
| Feb 2026 | 15 DF64 transcendentals | hotSpring S71 | barraCuda | sin/cos/exp/log/sqrt in double-float precision |
| Feb 2026 | Sturm tridiag eigensolver | hotSpring S26 | groundSpring, wetSpring | **47.7× speedup** for quasiperiodic eigenvalues |
| Mar 3 | Cross-spring absorption S83 | ALL springs | barraCuda/toadStool | stress_virial, ESN readout, CG kernels unified |
| Mar 5 | barraCuda budded from toadStool | toadStool S93 | barraCuda v0.3.0 | Math primal (WHAT) separated from hardware primal (WHERE) |
| Mar 5 | wgpu 28 migration | ALL springs | barraCuda v0.3.3 | Arc removal, workgroup constants, modern API |
| Mar 6 | f64 shared-memory bug | groundSpring V84-V85 | ALL springs | `PrecisionRoutingAdvice` — RTX 4070 naga zeros detected |
| Mar 7 | `shader.compile.*` IPC | toadStool S130 | coralReef proxy | Sovereign shader compilation via coralReef |
| Mar 7 | coralReef Iteration 52+ | coralReef | ALL springs | AMD E2E dispatch, f64 reduction shader fix for SM70/SM89 |
| Mar 8 | **V98 upstream rewire** | barraCuda `a898dee` | groundSpring | Typed errors, named constants, three-tier parity proven |

### Cross-Spring Flow Matrix

```
           Contributes To →
           hot  wet  neural  air  ground
hot        —    ✓    ✓       ✓    ✓       (DF64, MD, spectral)
wet        ✓    —    ✓       ✓    ✓       (bio, diversity, alignment)
neural     ✓    ✓    —       ✓    ✓       (chi², KL, correlation, ESN)
air        ✓    ✓    —       —    ✓       (ET₀, hydrology, Brent)
ground     ✓    ✓    ✓       ✓    —       (Anderson, chi², uncertainty, f64 bug)
```
