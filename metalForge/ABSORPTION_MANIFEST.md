# groundSpring Absorption Manifest

> Inventory of code for the Write → Absorb → Lean cycle with ToadStool/BarraCUDA.
>
> Following the hotSpring pattern: write locally, validate against CPU
> baselines, hand off via `wateringHole/handoffs/`, ToadStool absorbs as
> GPU ops, groundSpring rewires to upstream and deletes local code.

**Last updated**: February 25, 2026 (Phase 1c — paper queue buildout)

## Absorption Status Summary

| Module | Status | Tier | Target |
|---|---|---|---|
| `stats::pearson_r` | **Lean** — CPU delegated | A | `barracuda::stats::pearson_correlation` |
| `stats::spearman_r` | **Lean** — CPU delegated | A | `barracuda::stats::correlation::spearman_correlation` |
| `stats::sample_std_dev` | **Lean** — CPU delegated | A | `barracuda::stats::correlation::std_dev` |
| `stats::rmse` | **Ready** — GPU op exists, needs adapter | A | `barracuda::ops::NormReduceF64::l2` |
| `stats::mbe` | **Ready** — GPU op exists, needs adapter | A | `barracuda::ops::SumReduceF64::mean` |
| `stats::r_squared` | **Ready** — GPU op exists, needs adapter | A | `barracuda::ops::VarianceReduceF64` + reduce |
| `stats::index_of_agreement` | **Ready** — GPU op exists, needs adapter | A | `barracuda::ops::FusedMapReduceF64` |
| `stats::hit_rate` | **Ready** — GPU op exists, needs adapter | A | `barracuda::ops::FusedMapReduceF64` |
| `rarefaction::shannon_diversity` | **Ready** — GPU convenience method exists | A | `barracuda::ops::FusedMapReduceF64::shannon_entropy` |
| `prng::Xorshift64` | **Adapt** — needs PRNG alignment | B | `barracuda::ops::PrngXoshiro` |
| `seismic::grid_search_inversion` | **Write** — parallel grid dispatch | B | new workgroup dispatch |
| `rarefaction::multinomial_sample` | **Write** — WGSL production shader ready | C | new `ops::batched_multinomial_f64` |
| `fao56::daily_et0` | **Absorbed** — equation chain in barracuda | — | `barracuda::ops::BatchedElementwiseF64::fao56_et0_batch` |
| `fao56::daily_et0` (MC wrapper) | **Write** — WGSL production shader ready | C | new `ops::mc_et0_propagate_f64` |
| `bootstrap::bootstrap_mean` | **Lean** — CPU delegated | A | `barracuda::stats::bootstrap_mean` |
| `bootstrap::rawr_mean` | **Write** — no barracuda RAWR yet | C | new `ops::rawr_weighted_mean_f64` |
| `anderson::lyapunov_exponent` | **Lean** — GPU delegated | A | `barracuda::spectral::anderson::lyapunov_exponent` (requires `barracuda-gpu`) |
| `anderson::lyapunov_averaged` | **Lean** — GPU delegated | A | `barracuda::spectral::anderson::lyapunov_averaged` (requires `barracuda-gpu`) |
| `anderson::anderson_potential` | **Write** — local (matches barracuda when `barracuda-gpu` enabled) | B | `barracuda::spectral::anderson::anderson_potential` |
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

## Tier A — Lean (6 delegated, 6 GPU pending adapter)

### Already delegated

| Function | BarraCUDA target | Wiring |
|---|---|---|
| `pearson_r` | `stats::pearson_correlation` | `#[cfg(feature = "barracuda")]` NaN-safe |
| `spearman_r` | `stats::correlation::spearman_correlation` | `#[cfg(feature = "barracuda")]` NaN-safe |
| `sample_std_dev` | `stats::correlation::std_dev` | `#[cfg(feature = "barracuda")]` |
| `bootstrap_mean` | `stats::bootstrap_mean` | `#[cfg(feature = "barracuda")]` Result mapping |
| `lyapunov_exponent` | `spectral::anderson::lyapunov_exponent` | `#[cfg(feature = "barracuda-gpu")]` |
| `lyapunov_averaged` | `spectral::anderson::lyapunov_averaged` | `#[cfg(feature = "barracuda-gpu")]` |

### Pending GPU adapter

These have existing barracuda GPU ops but need `#[cfg(feature = "barracuda")]`
wiring with a `WgpuDevice`:

| Function | GPU op | How |
|---|---|---|
| `rmse` | `NormReduceF64::l2` | L2(obs − mod) / √n |
| `mbe` | `SumReduceF64::mean` | mean(mod − obs) |
| `r_squared` | `VarianceReduceF64` + `SumReduceF64` | 1 − SS_res/SS_tot |
| `index_of_agreement` | `FusedMapReduceF64` | custom map + sum |
| `hit_rate` | `FusedMapReduceF64` | binary agree map + mean |
| `shannon_diversity` | `FusedMapReduceF64::shannon_entropy` | convenience method exists |

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
- [x] CPU reference passes all validation checks (119/119)
- [x] Binding layout documented in this manifest
- [x] Dispatch geometry documented (workgroup size, grid dims)
- [x] f64 precision throughout (no f32 truncation)
- [x] PRNG matches barracuda (xoshiro128**)
- [x] Handoff V3 posted in `wateringHole/handoffs/`
- [ ] Tolerance comparison: GPU output vs CPU reference
- [ ] ToadStool absorption confirmed
