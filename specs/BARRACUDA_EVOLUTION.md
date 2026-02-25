# BarraCUDA Evolution Mapping

> groundSpring Rust module → BarraCUDA primitive → WGSL shader → pipeline stage

**Last updated**: February 25, 2026

## Philosophy

groundSpring follows the **Write → Absorb → Lean** cycle established by hotSpring:

1. **Write** — Pure-Rust CPU implementations in `crates/groundspring/`.
   Production WGSL shaders in `metalForge/shaders/`.
2. **Absorb** — ToadStool/BarraCUDA absorbs shaders as upstream ops.
   Handoff via `wateringHole/handoffs/`.
3. **Lean** — groundSpring rewires to `barracuda::ops::*` behind `#[cfg(feature = "barracuda")]`.

The CPU implementations are **validation references** — they must produce
identical results within documented tolerances.  The GPU implementations
are for throughput (100k+ MC samples, batch rarefaction).

## Module Mapping

> **ToadStool catch-up (S51–S62 + DF64, Feb 24–25 2026)**: Major absorption wave.
> FAO-56 ET₀ is now a GPU op (`BatchedElementwiseF64::fao56_et0_batch`).
> Shannon entropy has a GPU convenience method. Population variance resolved.
> 5 biological ODE systems absorbed. Spearman correlation added to CPU stats.
> S59: `anderson_3d_correlated`, `anderson_sweep_averaged`, `find_w_c`,
> `ridge_regression`, `ValidationHarness` absorbed.
> S60-61: `cpu-math` feature gate (wgpu optional under `gpu` feature).
> S62: `BandwidthTier`, `PeakDetectF64`.
> Post-S62: DF64 core-streaming, `ComputeDispatch` builder.
> barracuda also has `bootstrap_mean_f64.wgsl` GPU shader (65 lines).
>
> **Complete rewiring (Feb 25 2026)**: 4 new CPU delegations added:
> `covariance`, `norm_cdf`, `norm_ppf`, `chi2_statistic`,
> `analytical_localization_length`. Total: **11 active delegations**.
> 122 unit tests + 119/119 validation checks PASS in all three modes.
> Benchmarks confirm <2% overhead for compute-heavy binaries.

### Tier A — Lean (rewire to existing barracuda ops)

| groundSpring Module | BarraCUDA Op | Status | How |
|---|---|---|---|
| `stats::pearson_r` | `stats::pearson_correlation` | **DONE** (CPU delegated) | NaN-safe wrapper |
| `stats::spearman_r` | `stats::correlation::spearman_correlation` | **DONE** (CPU delegated) | NaN-safe wrapper |
| `stats::sample_std_dev` | `stats::correlation::std_dev` | **DONE** (CPU delegated) | Bessel-corrected |
| `stats::covariance` | `stats::correlation::covariance` | **DONE** (CPU delegated) | Sample covariance |
| `stats::norm_cdf` | `stats::norm_cdf` | **DONE** (CPU delegated) | Standard normal Φ(x) |
| `stats::norm_ppf` | `stats::norm_ppf` | **DONE** (CPU delegated) | Inverse Φ⁻¹(p) (Acklam) |
| `stats::chi2_statistic` | `stats::chi2_decomposed` | **DONE** (CPU delegated) | Goodness-of-fit |
| `bootstrap::bootstrap_mean` | `stats::bootstrap_mean` | **DONE** (CPU delegated) | Result struct mapping |
| `anderson::lyapunov_exponent` | `spectral::lyapunov_exponent` | **DONE** (barracuda-gpu) | Transfer matrix method |
| `anderson::lyapunov_averaged` | `spectral::lyapunov_averaged` | **DONE** (barracuda-gpu) | Multi-realization average |
| `anderson::analytical_localization_length` | `special::anderson_transport::localization_length` | **DONE** (CPU delegated) | Perturbative ξ(W,E) |
| `stats::rmse` | `ops::NormReduceF64::l2` | Pending GPU adapter | RMSE = L2(obs−mod) / √n |
| `stats::mbe` | `ops::SumReduceF64::mean` | Pending GPU adapter | MBE = mean(mod − obs) |
| `stats::r_squared` | `ops::VarianceReduceF64` + reduce | Pending GPU adapter | R² = 1 − SS_res/SS_tot |
| `stats::index_of_agreement` | `ops::FusedMapReduceF64` | Pending GPU adapter | Map: abs diffs, Reduce: sum |
| `stats::hit_rate` | `ops::FusedMapReduceF64` | Pending GPU adapter | Map: binary agree, Reduce: mean |
| `rarefaction::shannon_diversity` | `ops::FusedMapReduceF64::shannon_entropy` | Pending GPU adapter | H' = −Σ(p·ln p) — **convenience method exists** |

### Tier B — Adapt (needs alignment or wrapper)

| groundSpring Module | BarraCUDA Target | Blocker | Action |
|---|---|---|---|
| `prng::Xorshift64` | `ops::PrngXoshiro` (f64) | Different PRNG algorithm | Align to xoshiro; retain xorshift as CPU reference |
| `seismic::grid_search_inversion` | Parallel grid dispatch | No existing grid-search op | Dispatch as 3D workgroup; reduce min RMS |
| `rarefaction::multinomial_sample` | `ops::PrngXoshiro` + binary search | No batched multinomial | Production WGSL in metalForge |
| `gillespie::birth_death_ssa` | `ops::bio::GillespieGpu` | GPU-only (no CPU fallback) | Write CPU → GPU dispatch when adapter ready |
| `bootstrap::rawr_mean` | New: `ops::rawr_weighted_mean_f64` | No RAWR kernel in barracuda | Embarrassingly parallel — write metalForge shader |
| `anderson::anderson_potential` | `spectral::anderson_potential` | Requires `barracuda-gpu` feature | Align PRNG seeds |

### Tier C — ~~New Kernel Required~~ → Partially Absorbed

| Proposed Kernel | Status | Notes |
|---|---|---|
| `ops::mc_et0_propagate_f64` | **SUPERSEDED** — `BatchedElementwiseF64::fao56_et0_batch()` already in barracuda | ToadStool absorbed FAO-56 as `Op::Fao56Et0` with 9-input batch (tmax, tmin, rh_max, rh_min, wind, Rs, elev, lat, doy). GPU + CPU reference. groundSpring's `mc_et0_propagate.wgsl` MC wrapper remains valuable for uncertainty propagation. |
| `ops::batched_multinomial_f64` | **Still needed** — not in barracuda | Production WGSL in `metalForge/shaders/batched_multinomial.wgsl` |

### Stays Local (no GPU benefit)

| Module | Reason |
|---|---|
| `decompose::decompose_error` | Two scalar ops: bias² = MBE², var = RMSE² - MBE² |
| `decompose::noise_floor_reduction` | Three scalar ops |
| `validate::ValidationHarness` | Harness, not compute. Equivalent to `barracuda::validation::ValidationHarness` but with groundSpring-specific method names |
| `seismic::haversine_km` | Single scalar trig |
| `seismic::travel_time_1d` | One sqrt + division |

### New barracuda ops relevant to groundSpring (discovered S51–S62+)

| Op | Module | groundSpring Use |
|---|---|---|
| `BatchedElementwiseF64::fao56_et0_batch` | `ops::batched_elementwise_f64` | FAO-56 ET₀ batch compute — **supersedes our Tier C shader** |
| `FusedMapReduceF64::shannon_entropy` | `ops::fused_map_reduce_f64` | Shannon diversity — **convenience method, GPU-ready** |
| `FusedMapReduceF64::simpson_index` | `ops::fused_map_reduce_f64` | Simpson diversity — bonus |
| `VarianceReduceF64::population_variance` | `ops::variance_reduce_f64` | Population variance — **resolves semantics mismatch** |
| `VarianceReduceF64::population_std` | `ops::variance_reduce_f64` | Population std dev |
| `spearman_correlation` | `stats::correlation` | **DONE** — groundSpring delegates |
| `CapacitorOde`, `CooperationOde`, etc. | `numerical::ode_bio` | Waters paper-ready ODE systems |
| `NmfResult` | `linalg::nmf` | NMF for R. Anderson metagenomics |
| `anderson_3d_correlated` | `spectral` | S59 — correlated disorder (Méndez-Bermúdez) |
| `anderson_sweep_averaged` | `spectral` | S59 — disorder sweep ⟨r⟩(W) with stderr |
| `find_w_c` | `spectral` | S59 — critical disorder interpolation |
| `ridge_regression` | `linalg` | S59 — Tikhonov regression from ESN readout |
| `PeakDetectF64` | `ops` | S62 — GPU local-maxima with prominence |
| `BandwidthTier` | `dispatch` | S62 — PCIe/NvLink bandwidth-aware routing |

## Feature Gate

```toml
# Cargo.toml
[features]
default = []
barracuda = ["dep:barracuda"]
barracuda-gpu = ["barracuda", "barracuda/gpu"]

[dependencies]
barracuda = { path = "../../../phase1/toadstool/crates/barracuda", optional = true, default-features = false }
```

Two feature gates:
- `barracuda` — enables CPU delegation (`stats::bootstrap_mean`, `pearson_r`, etc.)
- `barracuda-gpu` — enables GPU + spectral delegation (`anderson::lyapunov_*`, `GillespieGpu`)

The CPU implementation remains the default and the validation reference.

## What NOT to Duplicate

BarraCUDA primitives that already exist and MUST NOT be reimplemented:

- Variance computation (`ops::variance_f64_wgsl`)
- Covariance (`ops::covariance_f64_wgsl`)
- Correlation (`ops::correlation_f64_wgsl`)
- Fused map-reduce (`ops::fused_map_reduce_f64`)
- Nelder-Mead optimization (`optimize::nelder_mead_gpu`)
- PRNG (`ops::prng_xoshiro_wgsl`)
- Shannon/Bray-Curtis diversity (`ops::bray_curtis_f64`)
- Norm reduce (`ops::norm_reduce_f64`)
- Sum reduce (`ops::sum_reduce_f64`)
- Bootstrap mean/std (`stats::bootstrap_mean`, `stats::bootstrap_std`)

groundSpring's CPU implementations serve as **validation references** for
these GPU kernels.

## GPU Promotion Blockers

1. **f64 precision** — groundSpring requires f64 for statistical validity.
   All WGSL shaders must use f64 or df64 (double-float f32-pair per
   hotSpring's `df64_core.wgsl` pattern).
2. **Multinomial sampling kernel** — does not exist in BarraCUDA.
   Production WGSL in `metalForge/shaders/batched_multinomial.wgsl`.
3. **FAO-56 MC wrapper kernel** — equation chain absorbed upstream as
   `Op::Fao56Et0`; MC noise wrapper (Box-Muller perturbation + dispatch) in
   `metalForge/shaders/mc_et0_propagate.wgsl`.
4. **PRNG alignment** — xorshift64 ↔ xoshiro128** produces different
   streams. Need to regenerate baselines after alignment.

## PRNG Alignment Roadmap

groundSpring currently uses `prng::Xorshift64` (Marsaglia 2003) — simple
and deterministic, but not the same generator as BarraCUDA's `PrngXoshiro`
(xoshiro128**).  Alignment is required before Tier B GPU promotion to ensure
GPU and CPU paths produce bitwise-identical streams.

### Semantic Mismatch

| Property | groundSpring `Xorshift64` | BarraCUDA `PrngXoshiro` |
|----------|--------------------------|------------------------|
| State size | 64 bits | 256 bits |
| Period | 2⁶⁴ − 1 | 2²⁵⁶ − 1 |
| Output bits | 64 | 64 |
| Quality | Fails BigCrush | Passes BigCrush |
| GPU kernel | None | `prng_xoshiro_wgsl` |

### Migration Steps (Phase 2b)

1. **Feature-gate the PRNG** — add `#[cfg(feature = "barracuda")]` path
   that delegates to `barracuda::ops::PrngXoshiro` for stream generation.
   Keep `Xorshift64` as the default (CPU reference).
2. **Create `prng::Xoshiro128` wrapper** — thin CPU-side wrapper around
   barracuda's generator with `next_u64()` and `next_normal()` matching
   the existing API surface.
3. **Regenerate all baselines** — rerun Python baselines with a compatible
   xoshiro128** implementation (e.g., a pure-Python xoshiro128** port).
4. **Update benchmark JSONs** — new expected values, new `baseline_commit`,
   new `baseline_date`, xoshiro128** noted in `prng_algorithm` field.
5. **Verify 119/119 checks** — run full validation suite.
6. **Remove old baselines** — archive xorshift64 baselines in
   `control/archive/xorshift64/` for fossil record.

### Variance Semantics

groundSpring `std_dev` uses **population variance** (÷ N); BarraCUDA
`stats::std_dev` uses **sample variance** (÷ N−1).  groundSpring now
provides both:

- `stats::std_dev` — population (canonical for RMSE decomposition)
- `stats::sample_std_dev` — Bessel-corrected (compatible with BarraCUDA)

The `stats::pearson_r` function is already wired to
`barracuda::stats::pearson_correlation` under `#[cfg(feature = "barracuda")]`.

## Local vs BarraCUDA CPU Delegation Performance

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

Overhead in short binaries (<200ms) is barracuda link/init cost.
For compute-heavy binaries (>500ms), delegation adds <2% overhead.

## Rust vs Python Performance (Phase 1c)

Pure Rust CPU math vs interpreted Python (NumPy/SciPy), median of 3 trials:

| Experiment | Python (s) | Rust (s) | Speedup |
|---|---|---|---|
| Exp 006: Signal Specificity (Gillespie SSA) | 26.2 | 0.85 | **30.9×** |
| Exp 007: RAWR Resampling (bootstrap) | 4.4 | 0.60 | **7.3×** |
| Exp 008: Anderson Localization (transfer matrix) | 21.4 | 0.72 | **29.8×** |
| **Total** | **52.0** | **2.17** | **24.0×** |

The 7.3× speedup for RAWR (vs 30× for others) reflects NumPy's vectorized
array operations — RAWR's inner loop is a weighted dot product that NumPy
handles efficiently.  Gillespie and Anderson involve per-step branching
that Python cannot vectorize.

## Timeline

| Phase | Milestone | Status |
|---|---|---|
| Phase 0 | Python baselines | **Done** (102/102 PASS across 8 experiments) |
| Phase 1a | Rust CPU validation | **Done** (119/119 PASS across 8 binaries) |
| Phase 1b | metalForge production WGSL | **Done** (2 production shaders, 261 combined lines) |
| Phase 1c | Paper queue buildout (Exp 006-008) | **Done** (31 new checks, 18 unit tests, 24× faster than Python) |
| Phase 2a | Tier A rewire (stats + bootstrap + anderson → barracuda) | **11 delegated** (7 stats + bootstrap_mean + 2 anderson + analytical ξ); 6 GPU pending adapter |
| Phase 2b | Tier B adapt (PRNG alignment, grid dispatch, gillespie GPU) | After 2a |
| Phase 2c | Tier C absorption (multinomial, RAWR kernels) | After 2b |
| Phase 3 | Full GPU pipeline, metalForge cross-substrate | After Phase 2 |

## Cross-Reference

- `metalForge/ABSORPTION_MANIFEST.md` — Detailed absorption inventory
- `metalForge/shaders/` — Production WGSL shaders
- `specs/BARRACUDA_REQUIREMENTS.md` — GPU kernel gap analysis
