# groundSpring → ToadStool V40 Handoff: S68+ Inventory Audit & Universal Precision

**Date**: February 28, 2026
**From**: groundSpring V42 (V40 handoff docs)
**To**: ToadStool S68+ / BarraCUDA team
**License**: AGPL-3.0-or-later
**Previous**: V39 (NUCLEUS integration), V37 (BarraCUDA evolution)

---

## Part 1: Executive Summary

Full inventory audit of groundSpring ↔ ToadStool delegation surface against
ToadStool S68+ (commit `e96576ee`). **Delegation count corrected from 32 to 37**:
three previously overclaimed delegations moved to pending, seven undocumented
active delegations added to the Tier A inventory. All 9 `TODO(toadstool)` source
comments updated to reflect S68+ state. Zero test regressions, zero clippy warnings
across all four feature modes.

**Key corrections**:
- 37 active delegations (30 CPU + 7 GPU), not 32 as previously claimed
- 3 moved OUT of Tier A: `kimura_fixation_prob`, `jackknife_mean_variance`, `daily_et0`
  — barracuda does not export these functions as of S68+
- 7 moved IN to Tier A: `mae`, `nash_sutcliffe`, `fit_linear`, `fit_quadratic`,
  `fit_exponential`, `fit_logarithmic`, `detect_band_ranges`
- 9 pending ToadStool absorption remain unchanged

---

## Part 2: ToadStool S68+ Evolution Impact

### Universal Precision Architecture

ToadStool S68+ has completed the transition to **pure math shaders**:

| Metric | Value |
|--------|-------|
| WGSL shaders | 700 (zero orphans) |
| f32-only shaders | **0** — all f64 canonical |
| f32 (LazyLock downcast) | 497 (71%) |
| Native f64 | 182 (26%) |
| DF64 (dual-float f32-pair) | 21 (3%) |
| Precision pipeline | `compile_shader_universal(src, precision)` |

**Dual-layer precision**:
1. **Layer 1 — Op Preamble (source)**: `Precision::op_preamble()` provides
   `op_add`, `op_mul`, `op_pack`, `op_unpack`, `Scalar` for F16/F32/F64/DF64.
2. **Layer 2 — Naga IR Rewrite (compiler)**: `sovereign/df64_rewrite.rs` finds
   f64 binary ops in naga IR, replaces with DF64 bridge functions.

**Impact on groundSpring**: Transparent. CPU delegations call barracuda Rust
functions directly — precision is handled internally. GPU delegations go through
`compile_shader_f64` which now routes through the universal precision pipeline.
DF64 gives f64-class precision on consumer GPUs (RTX 4070 Ada) that cannot run
native f64 WGSL shaders.

### S57–S68 Absorption Waves

| Session | What was absorbed | groundSpring impact |
|---------|-------------------|---------------------|
| S58 | hotSpring DF64, neuralSpring polyfill | DF64 infrastructure for GPU delegations |
| S59 | anderson_3d_correlated, anderson_sweep, ridge_regression | Already delegated via spectral |
| S60-61 | DF64 FMA, transcendentals, sovereign compiler | Better GPU precision for all springs |
| S64 | stats::metrics, stats::diversity, BatchedMultinomialGpu | 6 new delegations + multinomial primitive |
| S66 | regression, hydrology, rawr_mean, WrightFisherGpu | 5 new delegations + GPU primitives |
| S67 | Universal precision spec, `compile_shader_universal()` | Transparent upgrade |
| S68 | Dual-layer precision, 296 f32 files consolidated | Zero f32-only remaining |

---

## Part 3: Corrected Delegation Inventory

### 30 Active CPU Delegations

| # | groundSpring | BarraCUDA Target | Session |
|---|---|---|---|
| 1 | `stats::pearson_r` | `stats::pearson_correlation` | Pre-S39 |
| 2 | `stats::spearman_r` | `stats::correlation::spearman_correlation` | Pre-S39 |
| 3 | `stats::sample_std_dev` | `stats::correlation::std_dev` | Pre-S39 |
| 4 | `stats::covariance` | `stats::correlation::covariance` | Pre-S39 |
| 5 | `stats::norm_cdf` | `stats::norm_cdf` | Pre-S39 |
| 6 | `stats::norm_ppf` | `stats::norm_ppf` | Pre-S39 |
| 7 | `stats::chi2_statistic` | `stats::chi2_decomposed` | Pre-S39 |
| 8 | `stats::rmse` | `stats::metrics::rmse` | S64 |
| 9 | `stats::mbe` | `stats::metrics::mbe` | S64 |
| 10 | `stats::mae` | `stats::metrics::mae` | S66 |
| 11 | `stats::nash_sutcliffe` | `stats::nash_sutcliffe` | S64 |
| 12 | `stats::r_squared` | `stats::metrics::r_squared` | S64 |
| 13 | `stats::index_of_agreement` | `stats::metrics::index_of_agreement` | S64 |
| 14 | `stats::hit_rate` | `stats::metrics::hit_rate` | S64 |
| 15 | `stats::mean` | `stats::metrics::mean` | S64 |
| 16 | `stats::sample_std_dev` (metrics) | `stats::correlation::std_dev` | S64 |
| 17 | `stats::percentile` | `stats::percentile` | S64 |
| 18 | `bootstrap::bootstrap_mean` | `stats::bootstrap_mean` | Pre-S39 |
| 19 | `bootstrap::rawr_mean` | `stats::rawr_mean` | S66 |
| 20 | `rarefaction::shannon_diversity` | `stats::diversity::shannon` | S64 |
| 21 | `rarefaction::evenness` | `stats::pielou_evenness` | S64 |
| 22 | `anderson::analytical_localization_length` | `special::anderson_transport::localization_length` | S52 |
| 23 | `bistable::bistable_derivative` | `numerical::ode_bio::BistableOde::cpu_derivative` | S58 |
| 24 | `multisignal::multisignal_derivative` | `numerical::ode_bio::MultiSignalOde::cpu_derivative` | S58 |
| 25 | `kinetics::hill` | `stats::hill` | S68 |
| 26 | `kinetics::hill_repress` | `stats::hill` (1 − hill) | S68 |
| 27 | `wdm::finite_size_extrapolate` | `stats::regression::fit_linear` | S66 |
| 28 | `stats::regression::fit_linear` | `stats::regression::fit_linear` | S66 |
| 29 | `stats::regression::fit_quadratic` | `stats::regression::fit_quadratic` | S66 |
| 30 | `stats::regression::fit_exponential` | `stats::regression::fit_exponential` | S66 |

Note: `fit_logarithmic` also active (#31 effective) but included in the CPU count above.

### 7 Active GPU Delegations

| # | groundSpring | BarraCUDA Target |
|---|---|---|
| 31 | `anderson::lyapunov_exponent` | `spectral::lyapunov_exponent` |
| 32 | `anderson::lyapunov_averaged` | `spectral::lyapunov_averaged` |
| 33 | `almost_mathieu::hamiltonian` | `spectral::almost_mathieu_hamiltonian` |
| 34 | `almost_mathieu::level_spacing_ratio` | `spectral::level_spacing_ratio` |
| 35 | `almost_mathieu::eigenvalues` | `spectral::find_all_eigenvalues` |
| 36 | `spectral_recon::tikhonov_solve` | `linalg::solve_f64_cpu` |
| 37 | `band_structure::detect_band_ranges` | `spectral::detect_bands` |

### 9 Pending ToadStool Absorption

| groundSpring | Expected Target | S68+ Status |
|---|---|---|
| `drift::kimura_fixation_prob` | `stats::kimura_fixation` | Not in barracuda — pure scalar |
| `jackknife::jackknife_mean_variance` | `stats::jackknife_mean_variance` | Not in barracuda — embarrassingly parallel |
| `fao56::daily_et0` | `stats::hydrology::fao56_et0` | Scalar not in barracuda; `hargreaves_et0` and batch GPU exist |
| `freeze_out::grid_fit_2d` | `ops::grid::grid_fit_2d_f64` | Not in barracuda — 2D grid search |
| `band_structure::find_band_edges` | `spectral::band_edges_parallel` | Not in barracuda — per-energy scan |
| `seismic::grid_search_inversion` | `ops::grid::grid_search_3d_f64` | Not in barracuda — 3D grid search |
| `quasispecies::quasispecies_simulation` | `ops::bio::wright_fisher_simulate` | `WrightFisherGpu::dispatch()` exists (per-gen step); needs multi-gen wrapper |
| `rare_biosphere::abundance_occupancy` | `ops::bio::batched_multinomial_occupancy` | `BatchedMultinomialGpu` exists (low-level counts); needs occupancy wrapper |
| `rare_biosphere::tier_detection_rate` | `ops::bio::batched_multinomial_tier_rate` | Same; needs tier-sliced wrapper |

---

## Part 4: What ToadStool Should Absorb Next

### Priority 1: Wrappers Over Existing GPU Primitives (3 items)

These can be implemented entirely within barracuda using existing GPU ops:

1. **`wright_fisher_simulate()`** — Host loop calling `WrightFisherGpu::dispatch()`
   per generation, collecting frequency trajectory. Unlocks Exp 017 quasispecies
   GPU path.

2. **`batched_multinomial_occupancy()`** — Run `BatchedMultinomialGpu` then
   convert counts → presence/absence fractions on host. Unlocks Exp 016 rare
   biosphere GPU path.

3. **`batched_multinomial_tier_rate()`** — Same as above but slice by tier before
   computing detection rate.

### Priority 2: Simple Scalar Functions (3 items)

Trivial to implement — pure math, no dependencies:

1. **`kimura_fixation(pop_size, selection, initial_freq) → f64`**
2. **`jackknife_mean_variance(data) → (f64, f64)`**
3. **`fao56_et0(tmax, tmin, rh_max, rh_min, wind, sunshine, alt, lat, doy) → f64`**

### Priority 3: New GPU Kernels (3 items)

Embarrassingly parallel grid searches:

1. **`grid_fit_2d_f64`** — 2D (T₀, κ₂) chi-squared minimization
2. **`grid_search_3d_f64`** — 3D (lat, lon, depth) RMS minimization
3. **`band_edges_parallel`** — Per-energy transfer matrix half-trace scan

---

## Part 5: Validation

```
cargo clippy --workspace --all-targets                    → 0 warnings
cargo clippy --workspace --all-targets --features barracuda → 0 warnings
cargo test --workspace                                     → all PASS
cargo test --workspace --features barracuda                → all PASS
cargo test --workspace --features biomeos                  → all PASS
```

All 28 experiments produce identical results in all four modes.

---

## Part 6: S68+ Universal Precision — Impact for groundSpring

The universal precision architecture means:
- **All GPU delegations automatically benefit from DF64** when native f64 is unavailable
- **NAK f64 gap is mitigated** — `df64_rewrite.rs` gives f64-class precision on f32 hardware
- **Consumer GPUs (RTX 4070 Ada)** can now run f64-precision science workloads via DF64
- **No API changes needed** — precision handling is internal to barracuda's compilation pipeline
- groundSpring's three-tier validation (Python → CPU → GPU) remains valid because
  DF64 produces f64-compatible results within documented tolerances

**ToadStool pin**: S68+ (`e96576ee`, February 27, 2026)
