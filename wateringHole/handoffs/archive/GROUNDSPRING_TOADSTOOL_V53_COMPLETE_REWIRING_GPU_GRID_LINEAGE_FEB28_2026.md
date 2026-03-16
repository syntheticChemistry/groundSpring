# groundSpring → ToadStool V53 Handoff: Complete Rewiring + GPU Grid Adapters

**Date**: February 28, 2026
**ToadStool pin**: S70+++ (`1dd7e338`)
**groundSpring version**: V53
**License**: AGPL-3.0-only

---

## Summary

Complete rewiring of all viable barracuda delegations. GPU grid adapters
built for seismic and freeze-out using pre-evaluate-on-CPU + GPU-argmin
pattern. 3 additional CPU delegations wired. Comprehensive 12-workload
benchmark suite. Cross-spring shader evolution lineage documented.

**Delegation count**: 52 → **57 active** (38 CPU + 19 GPU), **1 evolution candidate**

---

## Part 1: New GPU Delegations (V52→V53)

### 1. `seismic::grid_search_inversion` → `barracuda::ops::grid::grid_search_3d`

**Pattern**: Pre-evaluate forward model on CPU, GPU-accelerated argmin.

The haversine + travel-time forward model runs on CPU to produce an RMS
residual at every (lat, lon, depth) grid point. The resulting 3D value
grid is sent to `grid_search_3d` for parallel minimum search via
`grid_search_3d_f64.wgsl`. The GPU finds the argmin of the pre-evaluated
grid in O(N/workgroup_size) parallel steps.

**Cross-spring lineage**: `grid_search_3d_f64.wgsl` shader was absorbed
from groundSpring's metalForge workload → ToadStool S70+. The
`ComputeDispatch` builder pattern comes from ToadStool S69++ architecture
evolution (originally from hotSpring's lattice QCD dispatch).

### 2. `freeze_out::grid_fit_2d` → `barracuda::ops::grid::grid_search_3d`

**Pattern**: Same pre-evaluate + GPU-argmin, with degenerate z-dimension = 1.

The Bazavov freeze-out polynomial forward model evaluates chi-squared at
each (T₀, κ₂) point on CPU. The 2D grid is passed to `grid_search_3d`
with z_grid = [0.0] for the parallel minimum search.

---

## Part 2: New CPU Delegations (V52→V53)

### 3. `quasispecies::error_threshold` → `barracuda::stats::evolution::error_threshold`

barracuda returns `Option<f64>` (None if σ ≤ 1 or L = 0); groundSpring
falls back to local computation when None.

### 4. `rare_biosphere::detection_power` → `barracuda::stats::evolution::detection_power`

Exact signature match. Infallible `#[cfg]` pattern.

### 5. `rare_biosphere::detection_threshold` → `barracuda::stats::evolution::detection_threshold`

Exact signature match. Infallible `#[cfg]` pattern.

---

## Part 3: Cross-Spring Shader Evolution Lineage

### hotSpring → barracuda (25+ shaders)

**Domain**: Nuclear physics, lattice QCD, MD, spectral theory.

| Contribution | Sessions | Impact on groundSpring |
|---|---|---|
| DF64 core-streaming (`df64_core.wgsl`) | S58 | Foundation for all f64 GPU computation |
| Spectral theory (Lanczos, Anderson, Hofstadter) | v0.6.0 | `anderson::lyapunov_exponent`, `almost_mathieu::eigenvalues` |
| Sturm tridiagonal eigensolve | S60+ | `linalg::tridiag_eigh` (47.4× peak speedup) |
| Lattice QCD (SU(3), CG, Wilson, HMC) | v0.6.1–v0.6.4 | DF64 precision → all GPU stats dispatch |
| ESN multi-head transport | v0.6.15 | Pattern for batch GPU dispatch |
| `FusedMapReduceF64` shader | hotSpring MD | `rmse_gpu`, `mbe_gpu` |

### wetSpring → barracuda (25+ shaders)

**Domain**: Metagenomics, life science, diversity.

| Contribution | Sessions | Impact on groundSpring |
|---|---|---|
| Bio primitives (Shannon, Simpson, Bray-Curtis) | S64 handoff v4–v8 | `rarefaction::simpson_diversity`, `bray_curtis` |
| Gillespie SSA GPU shader | S64 | `gillespie::birth_death_ssa_batch` |
| `BatchedMultinomialGpu` | S64 | `rare_biosphere::abundance_occupancy` |
| ODE generic solver | S65 | Foundation for `bistable`, `multisignal` |
| RTX 4070 f64 precision discovery | S64 | pow/exp/log behavior → all GPU stats |
| `log_f64` fix | S58 | Numerical stability in all GPU reductions |

### neuralSpring → barracuda (25+ shaders)

**Domain**: ML, evolutionary algorithms, spectral diagnostics.

| Contribution | Sessions | Impact on groundSpring |
|---|---|---|
| `WrightFisherGpu` | S66 metalForge | `drift::wright_fisher_fixation_batch` |
| Batch fitness evaluation | S69 metalForge | Pattern for stochastic batch dispatch |
| Xoshiro128** PRNG | S58 | GPU PRNG for all stochastic batch ops |
| `SimpleMLP` (JSON weights) | S70 | Available for future ML dispatch |
| RK45 adaptive | S69 | Available for ODE integration dispatch |

### airSpring → barracuda (12+ shaders)

**Domain**: Precision agriculture, hydrology.

| Contribution | Sessions | Impact on groundSpring |
|---|---|---|
| `BatchedElementwiseF64` (FAO-56 ET₀) | S66 metalForge | `fao56::daily_et0_batch` |
| Hargreaves ET₀, Van Genuchten, dual Kc | S70 | Extended hydrology pipeline |
| Moving window stats | S66 | `stats::moving_window::moving_window_stats` |
| Anderson coupling shader | S70 | `anderson::anderson_potential` GPU path |
| Regression suite (linear, quadratic, etc.) | S66 V009 | All `regression::fit_*` delegations |

### groundSpring → barracuda (5+ shaders)

**Domain**: Noise validation, grid search, uncertainty quantification.

| Contribution | Sessions | Impact on ToadStool |
|---|---|---|
| Jackknife mean/variance | S70 | `barracuda::stats::jackknife` |
| Kimura fixation, quasispecies | S70 | `barracuda::stats::evolution` |
| Chao1 classic (u64) | S70 | `barracuda::stats::diversity::chao1_classic` |
| FAO-56 scalar ET₀ | S70 | `barracuda::stats::hydrology::fao56_et0` |
| Grid search/fit shaders | S70 | `barracuda::ops::grid::grid_search_3d`, `grid_fit_2d` |
| `mc_et0_propagate_f64.wgsl` | S69 | Monte Carlo ET₀ propagation |
| `batched_multinomial_f64.wgsl` | S69 metalForge | Rarefaction batched sampling |

---

## Part 4: Benchmark Results (barracuda CPU mode)

```
Workload                                        CPU (ms) Batch/GPU (ms)    Speedup
----------------------------------------------------------------------------------
Gillespie SSA (100 trajectories)                    4.91           4.82       1.0×
Wright-Fisher fixation (100 trials)                19.05          19.53       1.0×
FAO-56 ET₀ (500 station-days)                       0.08           0.07       1.0×
FAO-56 scalar ET₀ (1 station-day)                   0.00              -          -
Kimura fixation (15 configs)                        0.00              -          -
Jackknife mean/var (500 points)                     0.00              -          -
Chao1 richness (200 taxa)                           0.00              -          -
Seismic inversion (31×31×7 grid)                    1.10              -          -
Freeze-out grid fit (61×41 grid)                    0.12              -          -
Rare biosphere (200sp × 100 samples)               13.33              -          -
Anderson Lyapunov (1000 sites)                      0.02              -          -
Neutral diversity (20sp × 500 gens)                 2.71              -          -
```

Notes: GPU batch speedups require GPU hardware (Titan V / RTX 4070).
The barracuda CPU path provides sovereignty (same math, no GPU required).

---

## Part 5: Quality State

| Gate | Status |
|------|--------|
| `cargo fmt --check` | PASS |
| `cargo clippy pedantic+nursery` ×3 modes | 0 warnings |
| `cargo doc --no-deps` | clean |
| `cargo test --workspace --features barracuda` | PASS |
| `bench-cpu-vs-gpu` (12 workloads) | measured |
| Zero `TODO(toadstool)` | ✓ |
| Zero `unsafe` | ✓ |

---

## Part 6: Evolution Candidate (Not Wired)

| groundSpring function | barracuda op | Reason |
|---|---|---|
| `band_structure::find_band_edges` | `ops::grid::band_edges_parallel` | Different algorithm: transfer matrix half-trace sign-change scan vs eigenvalue min/max extraction |

Requires a custom WGSL shader implementing the transfer matrix scan.
Natural candidate for ToadStool absorption once the transfer matrix
kernel is built.

---

## Handoff Checklist

- [x] ToadStool S70+++ commit reviewed and all APIs verified
- [x] 2 GPU grid adapters built (seismic + freeze-out via grid_search_3d)
- [x] 3 new CPU delegations wired (error_threshold, detection_power, detection_threshold)
- [x] 12-workload benchmark suite expanded and measured
- [x] Cross-spring shader evolution lineage documented (hotSpring, wetSpring, neuralSpring, airSpring, groundSpring)
- [x] All quality gates passing
- [x] V52 handoff archived
