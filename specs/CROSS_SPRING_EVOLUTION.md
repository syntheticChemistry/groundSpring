# Cross-Spring Shader Evolution

> How barracuda primitives flow between springs — and why it matters.

**Last updated**: February 25, 2026

## The Story

BarraCUDA (phase1/toadstool/crates/barracuda) is the shared Rust crate that
all ecoPrimals springs depend on. Each spring contributes domain-specific
math that gets absorbed into barracuda as general-purpose primitives. Then
OTHER springs use those primitives, creating cross-pollination.

This document traces the provenance of every barracuda primitive that
groundSpring uses — or could use — and shows how the ecosystem grows.

## The Write → Absorb → Lean Cycle

```
Spring writes local Rust    barracuda absorbs upstream    consuming spring leans
  (pure CPU reference)    →   (CPU + GPU + shader)     →   (#[cfg(feature)] delegate)
```

| Phase | What Happens | Who Benefits |
|-------|-------------|--------------|
| Write | Spring implements math in pure Rust | The writing spring (immediate) |
| Absorb | ToadStool team absorbs into barracuda | All springs (shared primitives) |
| Lean | Spring rewires to barracuda delegation | The leaning spring (GPU path, DRY) |
| Cross | Another spring discovers the primitive | The whole ecosystem |

## Provenance Map

### hotSpring → barracuda → everyone

hotSpring is the precision engine. Its f64/DF64 work enables all other
springs to do real science at full precision on consumer GPUs.

| Primitive | barracuda Location | Who Uses It |
|-----------|-------------------|-------------|
| DF64 core (Knuth-Dekker) | `shaders/math/df64_core.wgsl` | All springs needing FP64 on FP32 hardware |
| Fp64Strategy (Native/Hybrid) | `device/driver_profile.rs` | Dispatch layer for all GPU ops |
| Lanczos eigensolver | `spectral/lanczos.rs` | wetSpring (PCA), neuralSpring (spectral ML) |
| Hermite/Laguerre polynomials | `special/hermite.rs`, `special/laguerre.rs` | Physics across all springs |
| CG solver | `ops/lattice/cg.rs` | Any spring with sparse linear systems |
| Anderson spectral theory | `spectral/anderson*.rs` | **groundSpring** (Exp 008) |
| Lyapunov exponent | `spectral/lyapunov*.rs` | **groundSpring** (delegated) |
| `localization_length` | `special/anderson_transport.rs` | **groundSpring** (new delegation) |

**groundSpring impact**: hotSpring's spectral theory gives us GPU-accelerated
Anderson localization. The DF64 path means our f64-critical statistics
(RMSE, R², Pearson/Spearman) can run on consumer GPUs without precision loss.

### wetSpring → barracuda → everyone

wetSpring is the biology engine. Its metagenomics and ecology math
provides the bio primitives that groundSpring uses for sequencing noise
analysis.

| Primitive | barracuda Location | Who Uses It |
|-----------|-------------------|-------------|
| Bray-Curtis dissimilarity | `ops/bray_curtis_f64.rs` | Ecology diversity metrics |
| Shannon entropy (GPU) | `ops/fused_map_reduce_f64.rs` | **groundSpring** (rarefaction) |
| Simpson index (GPU) | `ops/fused_map_reduce_f64.rs` | Diversity metrics |
| NMF factorization | `linalg/nmf.rs` | Metagenomics, topic modelling |
| Ridge regression | `linalg/ridge.rs` | ESN readout, regularization |
| 5 biological ODEs | `numerical/ode_bio/` | wetSpring Waters papers |
| BatchedOdeRK4 | `numerical/ode_generic.rs` | All springs with ODE systems |
| Smith-Waterman | `ops/bio/smith_waterman.rs` | Sequence alignment |
| Gillespie GPU | `ops/bio/gillespie_gpu.rs` | **groundSpring** (Exp 006, future) |
| chi_squared_f64 | `special/chi_squared.rs` | **groundSpring** (new: `chi2_statistic`) |

**groundSpring impact**: wetSpring's Shannon entropy GPU path means our
rarefaction experiment can run at GPU scale. Their Gillespie GPU op is
the target for our Exp 006 (c-di-GMP signal specificity) when we complete
Phase 2 PRNG alignment. The chi-squared distribution from wetSpring V18
strengthens our goodness-of-fit testing.

### neuralSpring → barracuda → everyone

neuralSpring is the ML/optimization engine. Its validation harness
pattern and metalForge ops benefit all springs.

| Primitive | barracuda Location | Who Uses It |
|-----------|-------------------|-------------|
| ValidationHarness | `validation.rs` | All springs (S59 pattern) |
| Belief propagation | `linalg/graph.rs` | Graph-based inference |
| Boltzmann sampling | `sample/metropolis.rs` | MCMC methods |
| Numerical Hessian | `numerical/hessian.rs` | Sensitivity analysis |
| erf/erfc/gamma | `special/erf.rs`, `special/gamma.rs` | **groundSpring** (norm_cdf, chi2) |
| Bessel functions | `special/bessel.rs` | Wave propagation |

**groundSpring impact**: neuralSpring's special functions (erf, gamma)
underpin our new `norm_cdf`, `norm_ppf`, and `chi2_statistic` delegations.
The ValidationHarness from S59 offers a potential evolution path for
our validation binaries.

### groundSpring → barracuda (pending absorption)

groundSpring contributes back to the ecosystem through its unique
measurement noise primitives:

| Primitive | Current Location | Absorption Status |
|-----------|-----------------|-------------------|
| RAWR resampling | `bootstrap.rs` | Spec written, awaiting barracuda kernel |
| Multinomial sampling | `metalForge/shaders/batched_multinomial.wgsl` | WGSL ready, needs absorption |
| MC error propagation | `metalForge/shaders/mc_et0_propagate.wgsl` | WGSL ready, FAO-56 base absorbed |
| Bias-variance decomp | `decompose.rs` | Stays local (scalar ops) |
| Grid-search inversion | `seismic.rs` | Phase 2b (parallel dispatch) |

### airSpring → barracuda → groundSpring

| Primitive | barracuda Location | groundSpring Use |
|-----------|-------------------|-----------------|
| FAO-56 ET₀ batch | `ops/batched_elementwise_f64.rs` | Supersedes our Tier C shader |
| Richards PDE | `pde/richards.rs` | Future soil moisture modelling |

## The Cross-Pollination Graph

```
                    ┌─────────────┐
             ┌──────│  hotSpring   │──────┐
             │      │ (precision)  │      │
             │      └──────┬──────┘      │
             │             │DF64         │Spectral
             ▼             ▼             ▼
    ┌────────────┐  ┌────────────┐  ┌────────────┐
    │ neuralSpring│  │ barracuda  │  │groundSpring│
    │  (ML/opt)  │──│  (shared)  │──│  (noise)   │
    └────────────┘  └──────┬─────┘  └────────────┘
             ▲             │             ▲
             │             │Bio          │Shannon
             │      ┌──────┴──────┐      │
             └──────│  wetSpring   │──────┘
                    │ (biology)    │
                    └─────────────┘
```

## Delegation Inventory (groundSpring → barracuda)

### Active Delegations (11 total)

| # | Function | barracuda Target | Feature Gate | Origin Spring |
|---|----------|-----------------|--------------|---------------|
| 1 | `stats::pearson_r` | `stats::pearson_correlation` | `barracuda` | hotSpring stats |
| 2 | `stats::spearman_r` | `stats::correlation::spearman_correlation` | `barracuda` | wetSpring S57 |
| 3 | `stats::sample_std_dev` | `stats::correlation::std_dev` | `barracuda` | hotSpring stats |
| 4 | `stats::covariance` | `stats::correlation::covariance` | `barracuda` | hotSpring stats |
| 5 | `stats::norm_cdf` | `stats::norm_cdf` | `barracuda` | neuralSpring erf |
| 6 | `stats::norm_ppf` | `stats::norm_ppf` | `barracuda` | neuralSpring erf |
| 7 | `stats::chi2_statistic` | `stats::chi2_decomposed` | `barracuda` | wetSpring V18 |
| 8 | `bootstrap::bootstrap_mean` | `stats::bootstrap_mean` | `barracuda` | hotSpring stats |
| 9 | `anderson::lyapunov_exponent` | `spectral::lyapunov_exponent` | `barracuda-gpu` | hotSpring spectral |
| 10 | `anderson::lyapunov_averaged` | `spectral::lyapunov_averaged` | `barracuda-gpu` | hotSpring spectral |
| 11 | `anderson::analytical_localization_length` | `special::anderson_transport::localization_length` | `barracuda` | hotSpring transport |

### GPU-Pending Delegations (5)

| Function | barracuda GPU Op | Blocker |
|----------|-----------------|---------|
| `stats::rmse` | `ops::NormReduceF64::l2` | Need GPU adapter |
| `stats::mbe` | `ops::SumReduceF64::mean` | Need GPU adapter |
| `stats::r_squared` | `ops::VarianceReduceF64` + reduce | Need GPU adapter |
| `stats::index_of_agreement` | `ops::FusedMapReduceF64` | Need GPU adapter |
| `rarefaction::shannon_diversity` | `FusedMapReduceF64::shannon_entropy` | Need GPU adapter |

### Stays Local (5)

| Function | Reason |
|----------|--------|
| `decompose::*` | Scalar ops, no GPU benefit |
| `seismic::haversine_km` | Single scalar trig |
| `seismic::travel_time_1d` | One sqrt + division |
| `validate::ValidationHarness` | Harness, not compute |
| `prng::Xorshift64` | Reference PRNG (Phase 2b aligns to xoshiro) |

## Performance: Local vs BarraCUDA CPU Delegation

Three-trial best-of benchmark (release mode, Feb 25 2026):

| Binary | Local (ms) | Barracuda-GPU (ms) | Overhead |
|--------|-----------|-------------------|----------|
| validate-decompose | 60 | 82 | +37% (startup) |
| validate-rarefaction | 80 | 101 | +26% (startup) |
| validate-seismic | 111 | 136 | +23% (startup) |
| validate-weather | 56 | 82 | +46% (startup) |
| validate-fao56 | 72 | 96 | +33% (startup) |
| validate-signal-specificity | 861 | 870 | **+1%** |
| validate-rawr | 613 | 626 | **+2%** |
| validate-anderson | 720 | 728 | **+1%** |
| **TOTAL** | **2573** | **2721** | **+6%** |

The overhead for short binaries is barracuda's initial linking cost.
For compute-heavy binaries (>500 ms), the overhead is negligible (1-2%).
This confirms that CPU delegation adds no meaningful performance penalty.

## What ToadStool Can Do Next

1. **Absorb `batched_multinomial.wgsl`** — Production-ready, 112 lines,
   enables GPU-scale rarefaction for Exp 004.

2. **Add RAWR kernel** — `rawr_weighted_mean_f64.wgsl`, embarrassingly
   parallel (Dirichlet weights + weighted dot product). Enables GPU-scale
   bootstrap for Exp 007.

3. **Align PRNG** — Switch groundSpring from Xorshift64 to xoshiro128**,
   enabling bitwise-identical CPU/GPU streams for stochastic experiments.

4. **GPU adapters for reduce ops** — Wire `rmse`, `mbe`, `r_squared`,
   `index_of_agreement`, `hit_rate` to existing `FusedMapReduceF64` ops.
   These already exist in barracuda as GPU kernels.

## Cross-Reference

- `specs/BARRACUDA_EVOLUTION.md` — Detailed module→GPU promotion mapping
- `metalForge/ABSORPTION_MANIFEST.md` — Shader absorption inventory
- `wateringHole/handoffs/GROUNDSPRING_TOADSTOOL_V6_EVOLUTION_FEB25_2026.md` — Active handoff
