# groundSpring V78 — Modern Rewiring + Cross-Spring Benchmark Evolution

**Date:** 2026-03-05
**From:** groundSpring V78
**To:** barraCuda team, toadStool team, ecoPrimals ecosystem
**Supersedes:** V77 (wgpu 28 migration)
**barraCuda pin:** v0.3.3 (`4629bdd`)
**toadStool pin:** S94b (`9d359814`)
**groundSpring tests:** 806 passed, 0 failed
**Delegations:** 84 (50 CPU + 34 GPU)
**License:** AGPL-3.0-only

---

## Executive Summary

V78 completes the modern rewiring cycle. Three new ET₀ delegations absorb
airSpring lineage ops from barraCuda v0.3.2 (Makkink, Turc, Hamon). The
fused `mean_and_std_dev` API wires Welford single-pass GPU dispatch
(hotSpring DF64 lineage) into rarefaction and Gillespie, replacing the
2-dispatch mean + std_dev pattern. The cross-spring benchmark now tracks
shader provenance from all five springs through barraCuda v0.3.3.

## What Changed

### 1. Fused Mean+Variance (hotSpring DF64 → Welford)

New `stats::mean_and_std_dev()` uses `VarianceF64::mean_variance(data, 0)` —
single GPU dispatch returning `[mean, variance]`. Wired into:

- `rarefaction::rarefaction_at_depth`: 4 dispatches → 2 (genera + Shannon)
- `gillespie::birth_death_ssa_batch_cpu`: 2 dispatches → 1
- `gillespie::birth_death_ssa_batch_gpu`: 2 dispatches → 1

**Cross-spring provenance**: Welford algorithm shader from hotSpring precision
lineage. DF64 variant (`mean_variance_df64.wgsl`) auto-selected on consumer
GPUs via `Fp64Strategy`, giving ~10× throughput.

### 2. Three New ET₀ Methods (airSpring → barraCuda v0.3.2)

| Method | Inputs | barraCuda function |
|--------|--------|--------------------|
| `fao56::makkink_et0` | T_mean, Rs | `barracuda::stats::hydrology::makkink_et0` |
| `fao56::turc_et0` | T_mean, Rs, RH | `barracuda::stats::hydrology::turc_et0` |
| `fao56::hamon_et0` | T_mean, N | `barracuda::stats::hydrology::hamon_et0` |

Each has a sovereign CPU fallback using standard equations. All three
originated in airSpring V068/V069 and were absorbed into barraCuda v0.3.2.

### 3. Cross-Spring Benchmark Evolution

`benchmark_cross_spring.rs` now includes:

- **Fused mean+variance benchmark**: Compares fused vs separate dispatch
- **ET₀ method comparison**: 5-method table (PM, Hargreaves, Makkink, Turc, Hamon)
- **Phase 5 timeline**: barraCuda v0.3.3 / toadStool S94b evolution
- **8 new provenance entries**: wgpu 28, DF64 tiers, fused shaders,
  Makkink/Turc/Hamon, TensorContext, S94b decoupling

## Cross-Spring Shader Provenance (How Things Evolved to Be Helpful)

### hotSpring → All Springs
- **DF64 core** (S58): f64-class precision on consumer GPUs. Every spring
  that delegates to barracuda `*_f64` ops gets this free.
- **Welford mean+variance** (v0.3.3): Single-pass fused reduction. groundSpring
  V78 wires this into rarefaction (wetSpring bio lineage) and Gillespie
  (wetSpring SSA lineage).
- **Spectral theory** (S26): Anderson, Lanczos, Hofstadter → groundSpring
  Anderson localization experiments.
- **Sturm tridiag** (S26): 49.5× speedup for Almost-Mathieu eigenvalues.

### wetSpring → neuralSpring → All Springs
- **Bio primitives** (S64): Diversity (Shannon, Simpson, Bray-Curtis),
  Gillespie SSA, Hill/Monod kinetics → used by groundSpring for rare
  biosphere, signal specificity, drift experiments.
- **GemmCached** (S64): 60× taxonomy speedup → feeds neuralSpring ML.
- **Smith-Waterman, Felsenstein** (S27): Sequence alignment/phylogenetics.

### neuralSpring → hotSpring + wetSpring
- **pow_f64 polyfill** (S-17): Unblocked Ada Lovelace GPUs for all springs.
- **batch_ipr_f64** (S52): Spectral analysis → feeds hotSpring/groundSpring.
- **AlphaFold2 Evoformer** (S69): 17 sovereign protein folding shaders.

### airSpring → groundSpring
- **Regression** (S66): `fit_linear`, `fit_quadratic` → WDM extrapolation.
- **Hydrology** (S66/S70): Hargreaves, FAO-56, crop coefficient, soil water
  balance → groundSpring ET₀ experiments.
- **Makkink/Turc/Hamon** (v0.3.2): 3 new ET₀ methods → groundSpring V78.
- **Richards PDE** (V045): Unsaturated flow → available for groundSpring.

### groundSpring → wetSpring + All Springs
- **RAWR bootstrap** (S66): Feeds wetSpring rarefaction confidence intervals.
- **Batched multinomial** (S64): GPU rarefaction → used by wetSpring.
- **13-tier tolerance architecture** (V73): Named tolerance pattern adopted
  by wetSpring (164 tiers), available to all springs.
- **NucleusHarness pattern** (V76): Reusable validation harness for NUCLEUS.

### Bidirectional Flow
```
hotSpring precision → all springs get f64-class on consumer GPUs
wetSpring bio       → neuralSpring ML pipelines
neuralSpring fixes  → unblocked Ada for airSpring + wetSpring
airSpring hydrology → groundSpring ET₀ + seasonal pipeline
groundSpring noise  → wetSpring rarefaction confidence intervals
All springs         → ToadStool absorbs → barraCuda → all springs consume
```

## Delegation Inventory (V78: 84 total)

| Category | Count | Examples |
|----------|-------|---------|
| Stats CPU | 22 | mean, std_dev, pearson_r, rmse, mbe, norm_cdf, fit_linear |
| Bootstrap/Jackknife CPU | 5 | rawr_mean, bootstrap_mean, jackknife_mean_variance |
| Diversity CPU | 5 | bray_curtis, shannon, chao1, evenness, rarefaction_curve |
| Evolution CPU | 4 | kimura_fixation_prob, error_threshold, detection_power/threshold |
| Hydrology CPU | 7 | fao56_et0, hargreaves, makkink, turc, hamon, crop_coeff, soil_water |
| Anderson/Spectral CPU | 3 | localization_length, spectral_diagnostics, hill/monod |
| ODE/Kinetics CPU | 2 | bistable_derivative, multisignal_derivative |
| Optimization CPU | 2 | lbfgs_refine, chi2_analysis |
| **CPU subtotal** | **50** | |
| Stats GPU | 8 | mean, std_dev, rmse, mae, mbe, nse, r², pearson_r |
| Diversity GPU | 3 | shannon, simpson, multinomial_batch |
| Bio GPU | 4 | Gillespie, Wright-Fisher, rare biosphere occupancy/tier |
| Hydrology GPU | 5 | fao56_batch, hargreaves_batch, mc_et0, seasonal, multi_day |
| Spectral GPU | 8 | lyapunov, anderson_sweep, 2D/3D/4D, Almost-Mathieu, band_structure |
| Linalg GPU | 3 | tridiag_eigh, tikhonov, grid_search |
| ESN/Lanczos GPU | 2 | EsnClassifier, lanczos_eigenvalues |
| Bistable GPU | 1 | integrate_batch (RK4) |
| **GPU subtotal** | **34** | |
| **Total** | **84** | |

## Quality Gates

- `cargo fmt --check`: PASS
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS
- `cargo doc --workspace --no-deps`: PASS
- `cargo test --workspace`: 806 passed, 0 failed
- Deep debt zero maintained
