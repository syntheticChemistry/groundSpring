# groundSpring → ToadStool V52 Handoff: S70+ Catch-Up

**Date**: February 28, 2026
**ToadStool pin**: S70+++ (`1dd7e338`)
**groundSpring version**: V52
**License**: AGPL-3.0-or-later

---

## Summary

ToadStool S70+ absorbed the 4 remaining CPU delegation candidates from
groundSpring's pending list. This handoff documents the rewiring, validates
mathematical parity, and reclassifies 3 GPU grid ops as evolution candidates
(available in barracuda but with different algorithms than groundSpring's
domain-specific implementations).

**Delegation count**: 48 → **52 active** (35 CPU + 17 GPU), **0 pending**

---

## Part 1: New CPU Delegations (V51→V52)

### 1. `drift::kimura_fixation_prob`

```
groundSpring: kimura_fixation_prob(pop_size: usize, selection: f64, initial_freq: f64) -> f64
barracuda:    stats::evolution::kimura_fixation_prob(pop_size: usize, selection: f64, initial_freq: f64) -> f64
```

Exact signature match. Infallible `#[cfg]` pattern — when `barracuda` is enabled,
the barracuda implementation is used unconditionally. CPU fallback function and
constants gated behind `#[cfg(not(feature = "barracuda"))]`.

### 2. `jackknife::jackknife_mean_variance`

```
groundSpring: jackknife_mean_variance(data: &[f64]) -> Result<JackknifeResult, InputError>
barracuda:    stats::jackknife::jackknife_mean_variance(data: &[f64]) -> Option<JackknifeResult>
```

Adapted return type: `Option::Some` → `Ok(JackknifeResult{..})` mapping the
barracuda `JackknifeResult` fields. Falls back to CPU when barracuda returns
`None` (insufficient data).

### 3. `fao56::daily_et0`

```
groundSpring: daily_et0(inp: &DailyWeatherInputs) -> f64
barracuda:    stats::hydrology::fao56_et0(t_max, t_min, rh_max, rh_min, wind_2m, rs, elevation, lat_deg, doy: u32) -> Option<f64>
```

Key adaptation: barracuda's `rs` parameter expects solar radiation in MJ/m²/day,
while groundSpring's `DailyWeatherInputs` provides `sunshine_hours`. The delegation
pre-computes `Rs` from sunshine hours using Ångström (FAO-56 Eq. 35) and
converts wind speed from 10m to 2m height before passing to barracuda.
`day_of_year` widened from `u16` to `u32` via `u32::from()`.

### 4. `rare_biosphere::chao1`

```
groundSpring: chao1(counts: &[u64]) -> f64
barracuda:    stats::diversity::chao1_classic(counts: &[u64]) -> f64
```

Exact signature match. Formula parity confirmed: both use Chao 1984
`S_obs + f₁²/(2f₂)` with bias-corrected fallback `S_obs + f₁(f₁−1)/2`
when `f₂ = 0`. barracuda S70+ explicitly named it `chao1_classic` with `u64`
input (not the `f64` Chao & Chiu 2016 variant). CPU fallback extracted
to `chao1_cpu()` gated behind `#[cfg(not(feature = "barracuda"))]`.

---

## Part 2: GPU Grid Ops — Interface Mismatch Analysis

These 3 ops exist in `barracuda::ops::grid` (S70+) but implement
**different algorithms** than groundSpring's domain-specific functions:

| groundSpring function | barracuda op | Mismatch |
|-|-|-|
| `freeze_out::grid_fit_2d` | `grid::grid_fit_2d` | gS: chi-squared minimizer over polynomial forward model; b: bilinear surface fit `z = a+bx+cy+dxy` |
| `seismic::grid_search_inversion` | `grid::grid_search_3d` | gS: evaluates haversine forward model + RMS residual in one pass; b: finds minimum of pre-evaluated value grid |
| `band_structure::find_band_edges` | `grid::band_edges_parallel` | gS: transfer matrix half-trace sign-change scan; b: min/max extraction from sorted eigenvalue blocks |

**Status**: Reclassified from "pending delegation" to "evolution candidates".
Each could be delegated by uploading the forward model as a GPU compute shader
(natural next step for metalForge workload evolution).

---

## Part 3: Quality State

| Gate | Status |
|------|--------|
| `cargo fmt --check` | PASS |
| `cargo clippy pedantic+nursery` (default) | 0 warnings |
| `cargo clippy pedantic+nursery` (barracuda) | 0 warnings |
| `cargo clippy pedantic+nursery` (barracuda-gpu) | 0 warnings |
| `cargo doc --no-deps` | clean |
| `cargo test --workspace` | PASS |
| `cargo test --workspace --features barracuda` | PASS |
| Zero `#[allow]` (1 `allow(clippy::missing_const_for_fn)` with reason) | ✓ |
| Zero `unsafe` | ✓ |
| Zero `TODO(toadstool)` | ✓ |

---

## Part 4: Full Delegation Inventory

### CPU delegated (35, `#[cfg(feature = "barracuda")]`)

| # | groundSpring function | barracuda target |
|---|---|---|
| 1 | `stats::agreement::concordance_correlation` | `stats::correlation::concordance_correlation` |
| 2 | `stats::agreement::cohens_kappa` | `stats::metrics::cohens_kappa` |
| 3 | `stats::agreement::fleiss_kappa` | `stats::metrics::fleiss_kappa` |
| 4 | `stats::metrics::rmse` | `stats::metrics::rmse` |
| 5 | `stats::metrics::mbe` | `stats::metrics::mbe` |
| 6 | `stats::metrics::d_index` | `stats::metrics::d_index` |
| 7 | `stats::metrics::nse` | `stats::metrics::nse` |
| 8 | `stats::metrics::kge` | `stats::metrics::kge` |
| 9 | `stats::metrics::pbias` | `stats::metrics::pbias` |
| 10 | `stats::metrics::rsr` | `stats::metrics::rsr` |
| 11 | `stats::metrics::monod` | `stats::metrics::monod` |
| 12 | `stats::metrics::hill` | `stats::metrics::hill` |
| 13 | `stats::correlation::pearson_r` | `stats::correlation::pearson_r` |
| 14 | `stats::correlation::spearman_rho` | `stats::correlation::spearman_rho` |
| 15 | `stats::correlation::kendall_tau` | `stats::correlation::kendall_tau` |
| 16 | `bootstrap::rawr_mean` | `stats::bootstrap::rawr_mean` |
| 17 | `bootstrap::rawr_mean_variance` | `stats::bootstrap::rawr_mean_variance` |
| 18 | `bootstrap::bootstrap_ci` | `stats::bootstrap::bootstrap_ci` |
| 19 | `regression::fit_linear` | `stats::regression::fit_linear` |
| 20 | `regression::fit_quadratic` | `stats::regression::fit_quadratic` |
| 21 | `regression::fit_exponential` | `stats::regression::fit_exponential` |
| 22 | `regression::fit_power` | `stats::regression::fit_power` |
| 23 | `regression::fit_logarithmic` | `stats::regression::fit_logarithmic` |
| 24 | `regression::predict_linear` | `stats::regression::predict_linear` |
| 25 | `rare_biosphere::detection_probability` | `stats::diversity::detection_probability` |
| 26 | `rare_biosphere::required_depth` | `stats::diversity::required_depth` |
| 27 | `rare_biosphere::detection_power_frequency_based` | `stats::diversity::detection_power_frequency_based` |
| 28 | `gillespie::birth_death_ssa` | `stats::evolution::birth_death_ssa` |
| 29 | `drift::wright_fisher_fixation` | `stats::evolution::wright_fisher_fixation` |
| 30 | `fao56::hargreaves_et0` | `stats::hydrology::hargreaves_et0` |
| 31 | `fao56::crop_coefficient` | `stats::hydrology::crop_coefficient` |
| 32 | `drift::kimura_fixation_prob` | `stats::evolution::kimura_fixation_prob` **(V52)** |
| 33 | `jackknife::jackknife_mean_variance` | `stats::jackknife::jackknife_mean_variance` **(V52)** |
| 34 | `fao56::daily_et0` | `stats::hydrology::fao56_et0` **(V52)** |
| 35 | `rare_biosphere::chao1` | `stats::diversity::chao1_classic` **(V52)** |

### GPU delegated (17, `#[cfg(feature = "barracuda-gpu")]`)

| # | groundSpring function | barracuda GPU op |
|---|---|---|
| 1 | `stats::agreement::mean` | `SumReduceF64` → divide |
| 2 | `stats::agreement::std_dev` | `VarianceReduceF64` → sqrt |
| 3 | `stats::agreement::rmse` | `FusedMapReduceF64` |
| 4 | `stats::agreement::mbe` | `FusedMapReduceF64` |
| 5 | `stats::correlation::pearson_r` | `CorrelationF64` |
| 6 | `gillespie::birth_death_ssa_batch` | `GillespieGpu` |
| 7 | `drift::wright_fisher_fixation_batch` | `WrightFisherGpu` |
| 8 | `fao56::daily_et0_batch` | `BatchedElementwiseF64` |
| 9-17 | (9 additional GPU dispatches from V43-V51) | Various reduce/elementwise ops |

---

## Part 5: Evolution Candidates (Not Wired)

| groundSpring function | barracuda op | Reason |
|---|---|---|
| `freeze_out::grid_fit_2d` | `ops::grid::grid_fit_2d` | Different algorithm (surface fit vs chi-squared) |
| `seismic::grid_search_inversion` | `ops::grid::grid_search_3d` | Needs forward model evaluation on GPU |
| `band_structure::find_band_edges` | `ops::grid::band_edges_parallel` | Different algorithm (eigenvalue extraction vs transfer matrix scan) |

These are candidates for metalForge shader evolution — the forward models
(freeze-out polynomial, haversine travel-time, transfer matrix half-trace)
would be uploaded as compute shaders, then barracuda's grid search/minimize
infrastructure could be used for the parallel scan.

---

## Handoff Checklist

- [x] ToadStool S70+ commit reviewed (`1dd7e338`)
- [x] 4 new CPU delegations wired with correct API adaptation
- [x] fao56_et0 Rs conversion validated (FAO-56 Example 18 test passes)
- [x] chao1 formula parity confirmed (Chao 1984 `u64`)
- [x] 3 GPU grid ops analyzed — interface mismatch documented
- [x] Zero `TODO(toadstool)` remaining in codebase
- [x] All quality gates passing (fmt, clippy×3, doc, test×2)
- [x] V51 handoff archived
- [x] Docs updated (README, CHANGELOG, CONTROL_EXPERIMENT_STATUS)
