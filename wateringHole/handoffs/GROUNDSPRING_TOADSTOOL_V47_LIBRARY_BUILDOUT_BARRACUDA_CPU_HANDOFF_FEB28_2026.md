# groundSpring → ToadStool V47: Library Buildout + BarraCUDA CPU Expansion

**Date**: February 28, 2026
**ToadStool pin**: S68+ (`e96576ee`)
**groundSpring**: V47 (library buildout + 7 new barracuda CPU delegations)
**Previous**: V46 (idiomatic Rust evolution), V45 (validation gap closure: 292/292)
**License**: AGPL-3.0-or-later

---

## Summary

V47 is a **library expansion** pass — 7 new pure-Rust functions with barracuda
CPU delegation built in from day one. These are ecology, statistics, and kinetics
primitives that make the library complete for the full experiment portfolio and
future cross-spring usage.

**Delegation count**: 39 → **46 active** (37 CPU + 9 GPU). 7 pending unchanged.

---

## Part 1: New BarraCUDA CPU Delegations

### 1.1 Ecology metrics (rarefaction module)

| Function | BarraCUDA Op | Notes |
|----------|-------------|-------|
| `rarefaction::simpson_diversity` | `stats::diversity::simpson` (S64) | `1 − Σpᵢ²` — takes u64 counts, converts internally |
| `rarefaction::bray_curtis` | `stats::diversity::bray_curtis` (S64) | `Σ|aᵢ−bᵢ|/Σ(aᵢ+bᵢ)` — takes f64 abundances |
| `rarefaction::analytical_rarefaction` | `stats::diversity::rarefaction_curve` (S64) | Hypergeometric expected species — no RNG needed |

**Design note**: `chao1` stays local (formula mismatch: groundSpring uses classic
Chao 1984 `f₁²/(2f₂)`, barracuda uses bias-corrected Chao & Chiu 2016
`f₁(f₁−1)/(2(f₂+1))`). Delegation would break Python baseline provenance.

### 1.2 Kinetics

| Function | BarraCUDA Op | Notes |
|----------|-------------|-------|
| `kinetics::monod` | `stats::metrics::monod` (S66) | `r·x/(K+x)` — Monod saturation kinetics |

Complements existing `hill` / `hill_repress` delegation.

### 1.3 Bootstrap extensions

| Function | BarraCUDA Op | Notes |
|----------|-------------|-------|
| `bootstrap::bootstrap_median` | `stats::bootstrap_median` (S64) | Robust CI for median (outlier-resistant) |
| `bootstrap::bootstrap_std` | `stats::bootstrap_std` (S64) | CI for standard deviation |

### 1.4 Statistics

| Function | BarraCUDA Op | Notes |
|----------|-------------|-------|
| `stats::moving_window_stats` | `stats::moving_window_stats_f64` (S66) | New `stats::moving_window` submodule — mean/var/min/max |

### 1.5 All delegations use the standard pattern

```rust
#[cfg(feature = "barracuda")]
{
    // delegate to barracuda; type conversion if needed
    barracuda::stats::simpson(&f_counts)
}
#[cfg(not(feature = "barracuda"))]
local_cpu_implementation(counts)
```

For fallible ops: `if let Ok(result) = barracuda::stats::...(...)` with CPU fallback.

---

## Part 2: What ToadStool Could Absorb

### 2.1 Still pending (unchanged from V46)

| groundSpring Function | BarraCUDA Target | Status |
|----------------------|-----------------|--------|
| `drift::kimura_fixation_prob` | `stats::kimura_fixation` | Not in barracuda S68+ |
| `jackknife::jackknife_mean_variance` | `stats::jackknife_mean_variance` | Not in barracuda S68+ |
| `fao56::daily_et0` | `stats::hydrology::fao56_et0` (scalar) | Batch exists, scalar doesn't |
| `seismic::grid_search_inversion` | `ops::grid::grid_search_3d_f64` | No grid-search op |
| `freeze_out::grid_fit_2d` | `ops::grid::grid_fit_2d_f64` | No grid-search op |
| `band_structure::find_band_edges` | `spectral::band_edges_parallel` | Per-energy parallel dispatch |

### 2.2 Absorption priorities for ToadStool

**High value** (would unlock GPU tier for multiple experiments):
1. `fao56_et0` scalar — trivial: expose existing `fao56_et0_cpu` as public (currently `#[cfg(test)]` only)
2. `jackknife_mean_variance` — embarrassingly parallel, useful for Exp 019

**Medium value** (GPU acceleration candidates):
3. `grid_search_3d_f64` — embarrassingly parallel 3D scan (Exp 005, 020)
4. `grid_fit_2d_f64` — embarrassingly parallel 2D chi-squared (Exp 020)
5. `kimura_fixation_prob` — pure scalar, trivial kernel (Exp 014)

**Low priority** (needs design):
6. `band_edges_parallel` — per-energy transfer matrix scan (Exp 018)

### 2.3 Learnings relevant to ToadStool evolution

1. **Formula divergence matters**: `chao1` cannot be delegated because barracuda
   uses a different bias-correction formula than the Python baseline. When
   absorbing ecology metrics, ToadStool should consider offering both classic
   (Chao 1984) and bias-corrected (Chao & Chiu 2016) variants.

2. **u64 → f64 conversion pattern**: All ecology functions in groundSpring take
   `&[u64]` (integer count data), but barracuda takes `&[f64]`. This conversion
   allocates. Consider adding `&[u64]` overloads to `stats::diversity` to avoid
   allocation for integer-count callers (common in amplicon sequencing).

3. **PRNG alignment**: All 7 stochastic experiments use `Xorshift64` locally but
   barracuda uses `Xoshiro128**`. Phase 2b PRNG alignment needs baseline
   regeneration. This is the single largest blocker for GPU tier reproducibility.

4. **Moving window**: The sliding window algorithm in both groundSpring and
   barracuda is two-pass (exact mean, then variance). For GPU promotion, a
   single-pass Welford online algorithm would reduce memory bandwidth.

---

## Part 3: Quality State

| Gate | Result |
|------|--------|
| `cargo fmt --check` | PASS |
| `cargo clippy -D warnings` | 0 warnings |
| `cargo clippy -W pedantic` | 0 warnings |
| `cargo doc --no-deps` | clean |
| `cargo test -p groundspring --lib` | 322/322 PASS |
| `cargo test --workspace` | all PASS |
| `cargo test --features barracuda` | 322/322 PASS |
| Validation binaries | 292/292 PASS (28 experiments) |
| `#[allow]` annotations | 0 |
| `unsafe` blocks | 0 |
| Production mocks | 0 |

---

## Part 4: Full Delegation Inventory (46 active + 7 pending)

### CPU delegations (37 active)

| # | groundSpring | barracuda | Session |
|---|-------------|-----------|---------|
| 1 | `stats::pearson_r` | `stats::pearson_correlation` | S50 |
| 2 | `stats::spearman_r` | `stats::correlation::spearman_correlation` | S50 |
| 3 | `stats::sample_std_dev` | `stats::correlation::std_dev` | S50 |
| 4 | `stats::covariance` | `stats::correlation::covariance` | S50 |
| 5 | `stats::norm_cdf` | `stats::norm_cdf` | S50 |
| 6 | `stats::norm_ppf` | `stats::norm_ppf` | S50 |
| 7 | `stats::chi2_statistic` | `stats::chi2_decomposed` | S50 |
| 8 | `bootstrap::bootstrap_mean` | `stats::bootstrap_mean` | S50 |
| 9 | `anderson::analytical_localization_length` | `special::anderson_transport::localization_length` | S59 |
| 10 | `bistable::bistable_derivative` | `numerical::ode_bio::BistableOde::cpu_derivative` | S58 |
| 11 | `multisignal::multisignal_derivative` | `numerical::ode_bio::MultiSignalOde::cpu_derivative` | S58 |
| 12 | `stats::rmse` | `stats::metrics::rmse` | S64 |
| 13 | `stats::mbe` | `stats::metrics::mbe` | S64 |
| 14 | `stats::r_squared` | `stats::metrics::r_squared` | S64 |
| 15 | `stats::index_of_agreement` | `stats::metrics::index_of_agreement` | S64 |
| 16 | `stats::hit_rate` | `stats::metrics::hit_rate` | S64 |
| 17 | `rarefaction::shannon_diversity` | `stats::diversity::shannon` | S64 |
| 18 | `stats::mean` | `stats::metrics::mean` | S64 |
| 19 | `stats::percentile` | `stats::metrics::percentile` | S64 |
| 20 | `rarefaction::evenness` | `stats::pielou_evenness` | S64 |
| 21 | `bootstrap::rawr_mean` | `stats::rawr_mean` | S66 |
| 22 | `kinetics::hill` | `stats::hill` | S68 |
| 23 | `kinetics::hill_repress` | `stats::hill` (1 − hill) | S68 |
| 24 | `wdm::finite_size_extrapolate` | `stats::regression::fit_linear` | S66 |
| 25 | `stats::mae` | `stats::metrics::mae` | S66 |
| 26 | `stats::nash_sutcliffe` | `stats::nash_sutcliffe` | S64 |
| 27 | `stats::regression::fit_linear` | `stats::regression::fit_linear` | S66 |
| 28 | `stats::regression::fit_quadratic` | `stats::regression::fit_quadratic` | S66 |
| 29 | `stats::regression::fit_exponential` | `stats::regression::fit_exponential` | S66 |
| 30 | `stats::regression::fit_logarithmic` | `stats::regression::fit_logarithmic` | S66 |
| 31 | `kinetics::monod` | `stats::metrics::monod` | S66 |
| 32 | `rarefaction::simpson_diversity` | `stats::diversity::simpson` | S64 |
| 33 | `rarefaction::bray_curtis` | `stats::diversity::bray_curtis` | S64 |
| 34 | `rarefaction::analytical_rarefaction` | `stats::diversity::rarefaction_curve` | S64 |
| 35 | `bootstrap::bootstrap_median` | `stats::bootstrap_median` | S64 |
| 36 | `bootstrap::bootstrap_std` | `stats::bootstrap_std` | S64 |
| 37 | `stats::moving_window_stats` | `stats::moving_window_stats_f64` | S66 |

### GPU delegations (9 active)

| # | groundSpring | barracuda | Notes |
|---|-------------|-----------|-------|
| 1 | `anderson::lyapunov_exponent` | `spectral::lyapunov_exponent` | Transfer matrix |
| 2 | `anderson::lyapunov_averaged` | `spectral::lyapunov_averaged` | Multi-realization |
| 3 | `almost_mathieu::hamiltonian` | `spectral::almost_mathieu_hamiltonian` | λ/2 coupling |
| 4 | `almost_mathieu::eigenvalues` | `spectral::find_all_eigenvalues` | Sturm tridiag — 49.5× |
| 5 | `almost_mathieu::level_spacing_ratio` | `spectral::level_spacing_ratio` | Sort adapter |
| 6 | `band_structure::detect_band_ranges` | `spectral::detect_bands` | Gap detection |
| 7 | `spectral_recon::tikhonov_solve` | `linalg::solve_f64_cpu` | Gauss–Jordan |
| 8 | `rare_biosphere::abundance_occupancy` | `ops::bio::BatchedMultinomialGpu` | Batched sampling |
| 9 | `rare_biosphere::tier_detection_rate` | `ops::bio::BatchedMultinomialGpu` | Tier-sliced |

---

## Handoff Checklist

- [x] All 28 validation binaries PASS (292/292)
- [x] barracuda feature compiles cleanly (zero warnings)
- [x] 7 new delegations tested with and without barracuda feature
- [x] All `TODO(toadstool)` comments reference current S68+ state
- [x] Delegation inventory matches code (46 active, verified by grep)
- [x] BARRACUDA_EVOLUTION.md updated with new Tier A entries
- [x] PAPER_REVIEW_QUEUE.md delegation count updated
- [x] README.md delegation count and test count updated
