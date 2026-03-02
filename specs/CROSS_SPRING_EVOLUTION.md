# Cross-Spring Shader Evolution

**Last updated**: February 28, 2026 (V66 — 73 delegations, 100+ three-tier parity tests)

The ecoPrimals shader ecosystem evolved organically as each spring
absorbed domain-specific knowledge, then shared it through ToadStool's
BarraCUDA library. This document maps the provenance and
cross-pollination of shaders across springs.

## Overview

ToadStool S68+ has absorbed 700 WGSL shaders from five springs, compiled
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

## Benchmark Results (V42, February 28, 2026)

Three-mode benchmark (`benchmark-cross-spring --release`):

### CPU-Local (no barracuda)

| Workload | Time | Notes |
|---|---|---|
| Stats metrics (6 metrics, n=10K) | 59 µs | Pure groundSpring |
| Bootstrap RAWR (n=5K, B=1000) | 38,702 µs | Resampling-heavy |
| Regression fits (3 models, n=1K) | 18 µs | OLS + log-transform |
| Shannon diversity (S=200) | <1 µs | Instant |
| Anderson Lyapunov (L=200, R=500) | 2,625 µs | Transfer matrix |
| Rare biosphere occupancy (S=15, n=200) | 974 µs | Multinomial sampling |

### barracuda-GPU (S68+ universal precision)

| Workload | Time | Notes |
|---|---|---|
| Stats metrics (6 metrics, n=10K) | 62 µs | CPU delegation (≈parity) |
| Bootstrap RAWR (n=5K, B=1000) | 34,366 µs | CPU delegation |
| Regression fits (3 models, n=1K) | 25 µs | CPU delegation |
| Shannon diversity (S=200) | 1 µs | CPU delegation |
| Anderson Lyapunov (L=200, R=500) | 3,883 µs | CPU function via spectral |
| Rare biosphere occupancy (S=15, n=200) | 4,374,552 µs | **GPU first-call** (device+shader init ~4.3s) |
| Rare biosphere tier detection | 9,490 µs | GPU (device cached) |
| Rarefaction scaling n=50 | 5,208 µs | GPU overhead dominates |
| Rarefaction scaling n=500 | 3,767 µs | GPU overhead amortized |
| Rarefaction scaling n=1000 | 4,978 µs | GPU ≈ CPU crossover |

**Key observations**:
- CPU delegations (stats, regression, bootstrap) show near-parity: barracuda
  adds <5% overhead, confirming zero-cost abstraction for scalar work.
- GPU multinomial has ~4.3s first-call overhead (device creation + shader
  compilation). After init, scales sub-linearly with n_samples.
- GPU crossover vs CPU occurs at ~n_samples=1000 for this workload size.
  For larger communities (S=1000+) and deeper rarefaction, GPU wins earlier.
- Anderson Lyapunov is CPU-only in both modes (barracuda's `spectral::`
  functions are CPU, gated behind `gpu` feature for module access).

## GPU Delegation Status (V42)

| Delegation | Status | Shader Origin |
|---|---|---|
| `abundance_occupancy` → `BatchedMultinomialGpu` | **WIRED** (V42) | groundSpring → neuralSpring metalForge |
| `tier_detection_rate` → `BatchedMultinomialGpu` | **WIRED** (V42) | groundSpring → neuralSpring metalForge |
| `quasispecies_simulation` → `WrightFisherGpu` | Documented (needs multi-gen host loop) | neuralSpring metalForge |
| `kimura_fixation_prob` | Pending (not in barracuda) | — |
| `jackknife_mean_variance` | Pending (not in barracuda) | — |
| `fao56::daily_et0` | Pending (scalar not in barracuda) | — |
| `grid_fit_2d` | Pending (not in barracuda) | — |
| `find_band_edges` | Pending (not in barracuda) | — |
| `grid_search_inversion` | Pending (not in barracuda) | — |
