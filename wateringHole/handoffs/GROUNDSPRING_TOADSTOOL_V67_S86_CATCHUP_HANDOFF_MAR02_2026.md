# groundSpring → ToadStool V67 Handoff: S86 Catch-Up

**Date**: March 2, 2026
**groundSpring version**: V67
**ToadStool pin**: S86 (`7e01ac7e`)
**Previous pin**: S79 (`f97fc2ae`)

## Executive Summary

groundSpring caught up to ToadStool S86, rewiring 5 API touchpoints and
adding 2 new GPU delegations. Canonical metrics:

- **73 active delegations**: 43 CPU + 30 GPU (+2 GPU from V66)
- **776 Rust workspace tests** (unchanged — new functions exercised by existing coverage)
- **28 metalForge workloads** / **28 tolerance specs** (+2 each)
- **Zero debt**: 0 unsafe / 0 TODO / 0 `.unwrap()` / 0 `#[allow]`

## What Changed (S79 → S86)

ToadStool evolved through 8 commits since our S79 pin:

| Commit | Summary |
|--------|---------|
| `7e01ac7e` | Fix: ungate CPU modules incorrectly behind `#[cfg(feature = "gpu")]` |
| `2fee1969` | S86: update stale session references across root docs |
| `3eb7622d` | S84–S86: ComputeDispatch +33 ops (111→144), deep debt evolution |
| `94b5acc0` | S81+S82: +16 ComputeDispatch ops, OS memory detection, creation.rs DRY |
| `6661a9d1` | S80: Root docs updated, stale scripts cleaned |
| `1e36abb9` | S80: Batch Nelder-Mead GPU, fused_mlp, StatefulPipeline, ComputeDispatch |
| `1326a669` | S80: Driver workarounds, BatchedEncoder, P1 completions |
| `3388fee2` | S80: Nautilus absorption, ComputeDispatch batch, socket consolidation |

### Key barracuda API additions relevant to groundSpring

| Module | New API | groundSpring Action |
|--------|---------|---------------------|
| `stats::hydrology::gpu` | `McEt0PropagateGpu` | **Wired** → `fao56::monte_carlo_et0` |
| `stats::hydrology::gpu` | `SeasonalPipelineF64` | **Wired** → `fao56::seasonal_step` |
| `ops::bio` | `BatchedMultinomialConfig` (breaking) | **Fixed** → 3 call sites updated |
| `optimize` | `lbfgs`, `lbfgs_numerical` | Available — candidate for freeze_out refinement |
| `optimize` | `BrentGpu` (Van Genuchten, Green-Ampt) | Available — candidate for soil hydrology |
| `pde` | `RichardsGpu` | Available — candidate for infiltration modeling |
| `pipeline` | `BatchedStatefulF64` | Available — candidate for multi-day water balance |
| `multi_gpu` | `SubstratePipeline`, `InterconnectTopology` | Available — candidate for metalForge evolution |
| `spectral` | `anderson_4d`, `wegner_block_4d` | Available — candidate for tissue_anderson 4D |

## New groundSpring Delegations (V67)

### 1. `fao56::monte_carlo_et0` → `McEt0PropagateGpu`

GPU Monte Carlo uncertainty propagation through the full FAO-56
Penman-Monteith equation chain. `n_samples` perturbed ET₀ values
computed in a single GPU dispatch.

```rust
use groundspring::fao56::{monte_carlo_et0, Et0Uncertainties, example_18_inputs};

let base = example_18_inputs();
let unc = Et0Uncertainties {
    sigma_tmax: 0.5, sigma_tmin: 0.5,
    sigma_rhmax: 5.0, sigma_rhmin: 5.0,
    sigma_wind_frac: 0.10, sigma_sun_frac: 0.05,
};
let mc = monte_carlo_et0(&base, &unc, 10_000, 42);
// mc.mean ≈ 3.88, mc.std ≈ 0.14, mc.pct_05 < 3.88 < mc.pct_95
```

### 2. `fao56::seasonal_step` → `SeasonalPipelineF64`

Fused seasonal pipeline: ET₀ → Kc interpolation → water balance
→ yield stress computation. Single GPU dispatch per spatial cell array.

```rust
use groundspring::fao56::{seasonal_step, SeasonalCellInputs, SeasonalParams};

let cells = vec![SeasonalCellInputs { /* weather + theta */ }];
let params = SeasonalParams { /* growth stage + soil */ };
let output = seasonal_step(&cells, &params);
// output[i].et0, .kc, .etc, .theta_new, .stress
```

### 3. `BatchedMultinomialGpu::sample` — API break fix

The `sample` method now takes 5 parameters (was 4). Three call sites
updated in `rarefaction.rs` and `rare_biosphere.rs`:

```rust
// OLD: gpu.sample(&cumulative, &mut seeds, depth, n_reps)
// NEW:
let config = BatchedMultinomialConfig {
    cumulative_probs: true,
    seed: None,
};
gpu.sample(&cumulative, Some(&mut seeds), depth, n_reps, config)
```

## New metalForge Workloads

| Workload | Capability | Tolerance Tier |
|----------|------------|----------------|
| MC ET₀ propagation (GPU) | F64Compute, ShaderDispatch | Statistical |
| Seasonal pipeline (GPU fused) | F64Compute, ShaderDispatch | Analytical |

## Action Items for ToadStool

### P1: `SeasonalGpuParams` private padding fields

`SeasonalGpuParams._pad0` and `_pad1` are private, preventing struct literal
construction. groundSpring uses `bytemuck::Zeroable::zeroed()` + field assignment
as a workaround. Either make padding `pub` or add a constructor:

```rust
impl SeasonalGpuParams {
    pub fn new(cell_count: u32, day_of_year: u32, ...) -> Self { ... }
}
```

### P2: L-BFGS integration for grid search refinement

`barracuda::optimize::lbfgs` is available but not yet wired into groundSpring.
Candidate integration points:
- `freeze_out::grid_fit_2d` — L-BFGS post-grid-search refinement
- `seismic::grid_search_inversion` — gradient-based location refinement
- `spectral_recon::tikhonov_solve` — regularization parameter fitting

### P3: Future absorption candidates

| groundSpring feature | barracuda API | Notes |
|---------------------|---------------|-------|
| Richards infiltration | `RichardsGpu` | New soil hydrology capability |
| Multi-day water balance | `BatchedStatefulF64` | GPU-resident state for sequential days |
| 4D tissue Anderson | `anderson_4d`, `wegner_block_4d` | Extend Paper 12 to 4D |
| metalForge pipeline | `SubstratePipeline` | Replace local pipeline dispatch |

## Barracuda API Usage Review (V67)

groundSpring now touches 73 barracuda API surfaces (43 CPU + 30 GPU)
across 17 source modules. The heaviest consumers:

| Module | CPU | GPU | Total |
|--------|-----|-----|-------|
| fao56 | 5 | 4 | 9 |
| stats/agreement | 7 | 5 | 12 |
| rarefaction | 5 | 3 | 8 |
| anderson | 1 | 5 | 6 |
| bootstrap | 4 | 0 | 4 |
| rare_biosphere | 3 | 2 | 5 |

## Quality Gates (V67)

```
cargo fmt --check          → clean
cargo clippy (pedantic)    → 0 warnings
cargo test --workspace     → 776 passed, 0 failed
cargo doc --no-deps        → clean
```
