# groundSpring Absorption Manifest

> Inventory of code for the Write → Absorb → Lean cycle with ToadStool/BarraCUDA.
>
> Following the hotSpring pattern: write locally, validate against CPU
> baselines, hand off via `wateringHole/handoffs/`, ToadStool absorbs as
> GPU ops, groundSpring rewires to upstream and deletes local code.

**Last updated**: February 26, 2026 (V14 — S66 revalidation, 26 delegations, 49.5× Exp 009)

## Absorption Status Summary

| Module | Status | Tier | Target |
|---|---|---|---|
| `stats::pearson_r` | **Lean** — CPU delegated | A | `barracuda::stats::pearson_correlation` |
| `stats::spearman_r` | **Lean** — CPU delegated | A | `barracuda::stats::correlation::spearman_correlation` |
| `stats::sample_std_dev` | **Lean** — CPU delegated | A | `barracuda::stats::correlation::std_dev` |
| `stats::rmse` | **Lean** — CPU delegated | A | `barracuda::stats::rmse` |
| `stats::mbe` | **Lean** — CPU delegated | A | `barracuda::stats::mbe` |
| `stats::r_squared` | **Lean** — CPU delegated | A | `barracuda::stats::r_squared` |
| `stats::index_of_agreement` | **Lean** — CPU delegated | A | `barracuda::stats::index_of_agreement` |
| `stats::hit_rate` | **Lean** — CPU delegated | A | `barracuda::stats::hit_rate` |
| `stats::mean` | **Lean** — CPU delegated | A | `barracuda::stats::mean` |
| `stats::percentile` | **Lean** — CPU delegated | A | `barracuda::stats::percentile` |
| `rarefaction::shannon_diversity` | **Lean** — CPU delegated | A | `barracuda::stats::shannon` |
| `rarefaction::evenness` | **Lean** — CPU delegated + S≤1 adapter | A | `barracuda::stats::pielou_evenness` |
| `prng::Xorshift64` | **Adapt** — needs PRNG alignment | B | `barracuda::ops::PrngXoshiro` |
| `seismic::grid_search_inversion` | **Write** — parallel grid dispatch | B | new workgroup dispatch |
| `rarefaction::multinomial_sample` | **Write** — WGSL production shader ready | C | new `ops::batched_multinomial_f64` |
| `fao56::daily_et0` | **Absorbed** — equation chain in barracuda | — | `barracuda::ops::BatchedElementwiseF64::fao56_et0_batch` |
| `fao56::daily_et0` (MC wrapper) | **Write** — WGSL production shader ready | C | new `ops::mc_et0_propagate_f64` |
| `bootstrap::bootstrap_mean` | **Lean** — CPU delegated (GPU shader exists) | A | `barracuda::stats::bootstrap_mean` + `bootstrap_mean_f64.wgsl` |
| `bootstrap::rawr_mean` | **Lean** — CPU delegated (S66) | A | `barracuda::stats::rawr_mean` |
| `anderson::lyapunov_exponent` | **Lean** — GPU delegated | A | `barracuda::spectral::lyapunov_exponent` (requires `barracuda-gpu`) |
| `anderson::lyapunov_averaged` | **Lean** — GPU delegated | A | `barracuda::spectral::lyapunov_averaged` (requires `barracuda-gpu`) |
| `anderson::anderson_potential` | **Write** — local (matches barracuda when `barracuda-gpu` enabled) | B | `barracuda::spectral::anderson_potential` |
| `gillespie::birth_death_ssa` | **Write** — local CPU impl | B | `barracuda::ops::bio::GillespieGpu` (GPU-only, no CPU fallback) |
| `decompose::*` | **Stays local** — scalar math, no GPU benefit | — | N/A |
| `validate::*` | **Stays local** — harness, not compute | — | N/A |
| `seismic::haversine_km` | **Stays local** — scalar trig | — | N/A |
| `seismic::travel_time_1d` | **Stays local** — one sqrt + division | — | N/A |

---

## WGSL Shader Inventory

| Shader | Lines | Status | Bindings | Dispatch |
|---|---|---|---|---|
| `batched_multinomial.wgsl` | 112 | **Production** — xoshiro PRNG + binary search | 4 bindings (params, cumulative, seeds, counts) | `(ceil(n_reps/64), 1, 1)` |
| `mc_et0_propagate.wgsl` | 149 | **Production** — full equation chain + MC wrapper | 5 bindings (params, base, uncertainties, seeds, output) | `(ceil(n_samples/64), 1, 1)` |

Both shaders use xoshiro128** matching `barracuda::ops::prng_xoshiro_wgsl`.

---

## Tier A — Lean (26 delegated)

### All delegated

| Function | BarraCUDA target | Wiring |
|---|---|---|
| `pearson_r` | `stats::pearson_correlation` | `#[cfg(feature = "barracuda")]` NaN-safe |
| `spearman_r` | `stats::correlation::spearman_correlation` | `#[cfg(feature = "barracuda")]` NaN-safe |
| `sample_std_dev` | `stats::correlation::std_dev` | `#[cfg(feature = "barracuda")]` |
| `covariance` | `stats::correlation::covariance` | `#[cfg(feature = "barracuda")]` if-let |
| `norm_cdf` | `stats::norm_cdf` | `#[cfg(feature = "barracuda")]` direct |
| `norm_ppf` | `stats::norm_ppf` | `#[cfg(feature = "barracuda")]` direct |
| `chi2_statistic` | `stats::chi2_decomposed` | `#[cfg(feature = "barracuda")]` struct mapping |
| `bootstrap_mean` | `stats::bootstrap_mean` | `#[cfg(feature = "barracuda")]` Result mapping |
| `rawr_mean` | `stats::rawr_mean` | `#[cfg(feature = "barracuda")]` Result mapping (S66) |
| `lyapunov_exponent` | `spectral::lyapunov_exponent` | `#[cfg(feature = "barracuda-gpu")]` |
| `lyapunov_averaged` | `spectral::lyapunov_averaged` | `#[cfg(feature = "barracuda-gpu")]` |
| `analytical_localization_length` | `special::localization_length` | `#[cfg(feature = "barracuda")]` |
| `almost_mathieu_hamiltonian` | `spectral::almost_mathieu_hamiltonian` | `#[cfg(feature = "barracuda-gpu")]` |
| `bistable_derivative` | `BistableOde::cpu_derivative` | `#[cfg(feature = "barracuda")]` OdeSystem trait |
| `multisignal_derivative` | `MultiSignalOde::cpu_derivative` | `#[cfg(feature = "barracuda")]` OdeSystem trait |
| `rmse` | `stats::rmse` | `#[cfg(feature = "barracuda")]` direct (S64) |
| `mbe` | `stats::mbe` | `#[cfg(feature = "barracuda")]` direct (S64) |
| `r_squared` | `stats::r_squared` | `#[cfg(feature = "barracuda")]` direct (S64) |
| `index_of_agreement` | `stats::index_of_agreement` | `#[cfg(feature = "barracuda")]` direct (S64) |
| `hit_rate` | `stats::hit_rate` | `#[cfg(feature = "barracuda")]` direct (S64) |
| `shannon_diversity` | `stats::shannon` | `#[cfg(feature = "barracuda")]` u64→f64 (S64) |
| `mean` | `stats::mean` | `#[cfg(feature = "barracuda")]` direct (S64) |
| `percentile` | `stats::percentile` | `#[cfg(feature = "barracuda")]` direct (S64) |
| `level_spacing_ratio` | `spectral::level_spacing_ratio` | `#[cfg(feature = "barracuda-gpu")]` sort adapter |
| `almost_mathieu_eigenvalues` | `spectral::find_all_eigenvalues` | `#[cfg(feature = "barracuda-gpu")]` Sturm tridiag → **49.5× Exp 009** |
| `evenness` | `stats::pielou_evenness` | `#[cfg(feature = "barracuda")]` u64→f64 + S≤1 adapter |

---

## Tier B — Adapt

| Module | Blocker | Action |
|---|---|---|
| `prng::Xorshift64` | Different PRNG algorithm | Align to xoshiro; retain xorshift as CPU reference |
| `seismic::grid_search_inversion` | No grid-search GPU op | Dispatch as 3D workgroup; reduce min RMS |

---

## Tier C — Write (production WGSL ready for absorption)

### `batched_multinomial.wgsl`

Runs `n_reps` multinomial draws of `depth` reads from a community.

```
Params:     { n_taxa: u32, depth: u32, n_reps: u32, _pad: u32 }
Bindings:   params (uniform), cumulative (read), seeds (rw), counts (rw)
Dispatch:   (ceil(n_reps / 64), 1, 1) @ workgroup_size(64)
PRNG:       xoshiro128** (4 × u32 state per replicate)
Algorithm:  depth draws with binary-search assignment over cumulative probs
CPU ref:    groundspring::rarefaction::multinomial_sample()
```

### `mc_et0_propagate.wgsl`

Perturbs weather inputs with normal noise and computes ET₀ through FAO-56.

```
Params:     { n_samples: u32, _pad × 3 }
Bindings:   params (uniform), base_inputs (read), uncertainties (read), seeds (rw), output (rw)
Dispatch:   (ceil(n_samples / 64), 1, 1) @ workgroup_size(64)
PRNG:       xoshiro128** (4 × u32 state per sample)
Algorithm:  Box-Muller perturbation → full Penman-Monteith chain
CPU ref:    validate_fao56::monte_carlo_et0()
NOTE:       Equation chain is superseded by barracuda Op::Fao56Et0 — when
            absorbed, replace compute_et0() with the existing batched op.
```

---

## Stays Local

| Module | Reason |
|---|---|
| `decompose::decompose_error` | Two scalar ops (bias² = MBE², variance = RMSE² − MBE²) |
| `decompose::noise_floor_reduction` | Three scalar ops |
| `validate::ValidationHarness` | Harness, not compute |
| `seismic::haversine_km` | Single scalar trig |
| `seismic::travel_time_1d` | One sqrt + division |

---

## Handoff Checklist (per shader)

- [x] Production WGSL file with documented bindings
- [x] CPU reference passes all validation checks (185/185 across 15 binaries)
- [x] Binding layout documented in this manifest
- [x] Dispatch geometry documented (workgroup size, grid dims)
- [x] f64 precision throughout (no f32 truncation)
- [x] PRNG matches barracuda (xoshiro128**)
- [x] Handoff V14 posted in `wateringHole/handoffs/` (V13, V12, V11, V10, V9, V8 archived)
- [x] All 26 barracuda delegations use `#[cfg]` or `if let Ok` with CPU fallback always compiled
- [x] Mathematical parity: 15/15 PROVEN (Python ⇌ Rust, `data/parity_report.json`)
- [x] PRNG alignment investigated: requires full rebaseline (documented in V8 handoff)
- [x] ToadStool S64 catch-up: 6 new CPU delegations (metrics + shannon), 3 bug fixes
- [x] Complete rewiring: 4 more delegations (mean, percentile, level_spacing_ratio, eigenvalues)
- [x] Exp 009: 49.5× speedup from Sturm tridiag solver (hotSpring S26 spectral)
- [x] Three-mode revalidation (local / barracuda / barracuda-gpu): all PASS × 3, 0 warnings × 3
- [x] Fixed OdeSystem trait import, hofstadter module path, dead-code gates
- [ ] Tolerance comparison: GPU output vs CPU reference
- [ ] ToadStool absorption of groundSpring shaders confirmed
