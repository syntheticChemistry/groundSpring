# Cross-Spring Shader Evolution

**Last updated**: March 7, 2026 (V97 — 102 active delegations (61 CPU + 41 GPU), 936 Rust workspace tests, three-tier parity proven: 29/29 at all 3 tiers)

The ecoPrimals shader ecosystem evolved organically as each spring
absorbed domain-specific knowledge, then shared it through barraCuda
(the standalone compute primal). This document maps the provenance and
cross-pollination of shaders across springs.

## Overview

barraCuda v0.3.3 has absorbed 708 WGSL shaders from five springs, compiled
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

Three-mode benchmark (`benchmark-cross-spring --release`):

### Cross-Spring Provenance Benchmark (barraCuda `2a6c072`, toadStool S130)

| Workload | Origin Spring | Time | Notes |
|---|---|---|---|
| Stats metrics (6, n=10K) | airSpring + groundSpring → S64 | 2.9s | GPU dispatch (precision-routed) |
| Fused mean+variance (n=50K) | hotSpring DF64 → Welford | 3.7ms | hotSpring `df64_core.wgsl` lineage |
| Bootstrap RAWR (n=5K, B=1K) | groundSpring → S66 | 30.3ms | RAWR resampling |
| Regression fits (3 models, n=1K) | airSpring → S66 | 25µs | Linear/quadratic/exponential |
| Shannon diversity (S=200) | wetSpring → S64 | 1.5ms | Pielou evenness included |
| ET₀ 5 methods (Uccle) | airSpring → barraCuda v0.3.2 | <1µs each | PM/Hargreaves/Makkink/Turc/Hamon |
| Anderson Lyapunov (L=200, R=500) | hotSpring → S26 | 2.4ms | Transfer matrix, spectral theory |

### CPU vs GPU Benchmark (bench-cpu-vs-gpu)

| Workload | CPU (ms) | GPU (ms) | Notes |
|---|---|---|---|
| Gillespie SSA (100 traj) | 5.2 | 23.3 | GPU overhead dominates at small batch |
| Wright-Fisher (100 trials) | 19.5 | 232.7 | GPU overhead; wins at 10K+ trials |
| Multinomial (200 reps × 6 taxa) | 1.0 | 4.4 | GPU crossover at ~1K reps |
| Covariance (5K pairs, GPU) | 0.43 | — | CovarianceF64 single-pass |
| Autocorrelation (10K, lag 200) | 0.40 | — | AutocorrelationF64 |
| Seismic grid (31×31×7) | 7210 | — | CPU-intensive, GPU candidate |

### Precision Routing (V97)

11 GPU dispatch paths now check `PrecisionRoutingAdvice` via `get_device_f64_safe()`:
- `F64Native` → proceed (workgroup f64 reductions safe)
- `Df64Only` → proceed (barraCuda routes DF64 shaders internally)
- `F64NativeNoSharedMem` → skip GPU, fall back to CPU (naga shared-mem f64 zeros bug)
- `F32Only` → skip GPU, fall back to CPU

### Three-Tier Parity (V97)

| Tier | Tests | Status |
|---|---|---|
| Physics (Anderson, band, transport, seismic, freeze-out) | 27 | PASS |
| Stats (bootstrap, RMSE, regression, correlation) | 24 | PASS |
| Bio (drift, jackknife, rare biosphere, quasispecies, rarefaction) | — | PASS |
| GPU (workloads, CPU vs GPU dispatch) | — | PASS |

## GPU Delegation Status (V97 — 102 delegations)

| Delegation | Status | Shader Origin |
|---|---|---|
| 61 CPU delegations (stats, bio, hydrology, linalg, spectral) | **WIRED** | Cross-spring via barraCuda |
| 41 GPU delegations (reductions, multinomial, ODE, optimization) | **WIRED** | Cross-spring via barraCuda GPU ops |
| 21 GPU paths with precision routing | **WIRED** (V97) | `get_device_f64_safe()` + runtime f64 smoke test |
| `BatchedOdeRK45F64` (adaptive RK45) | Available | wetSpring V95 → barraCuda |
| `GpuView<T>` (persistent GPU buffer) | Available | barraCuda pipeline |
| `LSCFRK` (lattice integrators) | Available | hotSpring lattice QCD |
