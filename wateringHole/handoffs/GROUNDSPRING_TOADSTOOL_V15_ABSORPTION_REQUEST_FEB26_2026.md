# groundSpring → ToadStool/BarraCUDA Absorption Request V15

**Date**: February 26, 2026
**From**: groundSpring (noise validation + environmental modeling)
**To**: ToadStool/BarraCUDA team
**Previous**: V14 (S65 revalidation)
**ToadStool HEAD**: `17932267` (S65)
**groundSpring HEAD**: current (V14 + docs cleanup)

---

## Purpose

This handoff requests ToadStool to absorb groundSpring's remaining local
shaders and act on lessons learned from 25 delegations across 11 experiments.
V14 confirmed everything works; this V15 is the absorption action request.

---

## Part 1: Shaders Ready for Absorption

### 1.1 `batched_multinomial.wgsl` (Priority: High)

**Location**: `metalForge/shaders/batched_multinomial.wgsl` (112 lines)

```
Purpose:    n_reps multinomial draws of depth reads from a community
Params:     { n_taxa: u32, depth: u32, n_reps: u32, _pad: u32 }
Bindings:   @group(0) @binding(0) params (uniform)
            @group(0) @binding(1) cumulative_probs (read)
            @group(0) @binding(2) seeds (read-write, 4 × u32 per rep)
            @group(0) @binding(3) counts (read-write)
Dispatch:   (ceil(n_reps/64), 1, 1) @ workgroup_size(64)
PRNG:       xoshiro128** (matches barracuda standard)
CPU ref:    groundspring::rarefaction::multinomial_sample()
Validated:  15/15 checks (validate-rarefaction), 190/190 tests × 3 modes
```

**Why absorb**: Rarefaction is used by wetSpring (metagenomics), groundSpring
(sequencing noise), and potentially neuralSpring (bootstrap resampling). The
GPU kernel enables 100k+ replicate rarefaction curves in seconds.

**Blocker for groundSpring rewiring**: Current `barracuda::ops::batched_multinomial`
uses closure-based cumulative_probs RNG, while groundSpring uses seed-based
`Xorshift64`. A seed-based `(abundances, depth, seed)` API would simplify
delegation. See Part 3 suggestion #3.

### 1.2 `mc_et0_propagate.wgsl` (Priority: Medium)

**Location**: `metalForge/shaders/mc_et0_propagate.wgsl` (149 lines)

```
Purpose:    Monte Carlo uncertainty propagation through FAO-56 ET₀
Params:     { n_samples: u32, _pad × 3 }
Bindings:   @group(0) @binding(0) params (uniform)
            @group(0) @binding(1) base_inputs (read)
            @group(0) @binding(2) uncertainties (read)
            @group(0) @binding(3) seeds (read-write)
            @group(0) @binding(4) output (read-write)
Dispatch:   (ceil(n_samples/64), 1, 1) @ workgroup_size(64)
PRNG:       xoshiro128** (matches barracuda standard)
CPU ref:    validate_fao56::monte_carlo_et0()
Note:       Equation chain now superseded by barracuda Op::Fao56Et0;
            wrapping the existing batched op with MC perturbation preferred.
```

**Why absorb**: Pairs with the already-absorbed `Op::Fao56Et0` to enable
end-to-end uncertainty quantification on GPU. Used by airSpring and groundSpring.

---

## Part 2: Semantic Fixes Requested

### 2.1 `pielou_evenness` edge case (S≤1)

**Current barracuda behavior**: Returns `0.0` for single-species communities.
**Ecology convention**: All major ecology packages (vegan R, scipy, scikit-bio)
return `1.0` for S≤1 (trivially even).

groundSpring currently has a pre-delegation adapter:
```rust
if s <= 1 { return 1.0; }
// then delegate to barracuda
```

**Request**: Update `barracuda::stats::pielou_evenness` to return `1.0` for
S≤1, matching ecology convention. This removes adapter code from all consumers.

### 2.2 Spectral CPU promotion

Five barracuda spectral functions are pure CPU implementations but are gated
behind `#[cfg(feature = "gpu")]` in barracuda's `lib.rs`:

- `lyapunov_exponent`
- `lyapunov_averaged`
- `level_spacing_ratio`
- `find_all_eigenvalues`
- `almost_mathieu_hamiltonian`

**Request**: Promote these to a CPU-only module (e.g., `spectral_cpu` or
`numerical::spectral`) so Springs can use them without pulling in the GPU
dependency chain. This would let groundSpring change from `barracuda-gpu`
to `barracuda` for 5 delegations, reducing build times.

### 2.3 Seed-based `batched_multinomial` API

**Current**: Closure-based `cumulative_probs` RNG interface.
**Request**: Add `(abundances: &[u64], depth: u64, seed: u64) -> Vec<u64>`
alongside the existing API. This matches the pattern used by all Springs
and enables straightforward delegation without adapter closures.

---

## Part 3: Cross-Spring Learnings for ToadStool Evolution

### 3.1 What Worked Exceptionally Well

| Pattern | Origin | Impact |
|---------|--------|--------|
| Sturm tridiag eigensolve | hotSpring S26 | **49.5× speedup** for Exp 009 (quasiperiodic localization) |
| `OdeSystem` trait dispatch | wetSpring bio-ODE | Clean delegation for bistable + multisignal experiments |
| `stats::metrics` bulk absorption | airSpring/groundSpring S64 | 6 metrics delegated in one session |
| `if let Ok` + CPU fallback | hotSpring pattern | Zero-cost when barracuda unavailable; graceful degradation |
| `#[cfg(feature)]` with always-compiled CPU | groundSpring V14 | CPU code never goes stale; tests run in all modes |

### 3.2 What Springs Will Need Next

| Need | Which Springs | barracuda Module |
|------|---------------|------------------|
| Batch resampling (RAWR weighted bootstrap) | groundSpring, wetSpring | New: `stats::rawr_weighted_mean` |
| GPU Gillespie SSA CPU fallback | groundSpring, wetSpring | `numerical::ode_bio::GillespieGpu` |
| Grid-search parallel dispatch | groundSpring (seismic) | New: `ops::grid_search_dispatch` |
| PRNG xoshiro128** CPU alignment | All Springs | `prng::Xoshiro128StarStar` CPU-side |
| Batch rarefaction curves | groundSpring, wetSpring | Use absorbed `batched_multinomial` + seed API |

### 3.3 Delegation Patterns That Other Springs Should Adopt

groundSpring's delegation pattern is now mature and documented. Key principles:

1. **Always compile CPU code** — no `#[cfg(not(feature))]` around CPU paths
2. **Extract `_cpu` suffixed private functions** — keeps public API clean
3. **Pre-validate edge cases before delegating** — catches semantic mismatches
4. **Document λ/2 style convention differences** — prevents silent numerical errors
5. **Three-mode CI** — test with default, barracuda, barracuda-gpu in CI

---

## Part 4: Complete Delegation Inventory (25 active)

### CPU delegations (`barracuda` feature) — 20

| # | groundSpring | barracuda | Semantic notes |
|---|-------------|-----------|----------------|
| 1 | `pearson_r` | `stats::pearson_correlation` | NaN-safe |
| 2 | `spearman_r` | `stats::correlation::spearman_correlation` | NaN-safe |
| 3 | `sample_std_dev` | `stats::correlation::std_dev` | Bessel-corrected |
| 4 | `covariance` | `stats::correlation::covariance` | Sample covariance |
| 5 | `norm_cdf` | `stats::norm_cdf` | Standard normal Φ(x) |
| 6 | `norm_ppf` | `stats::norm_ppf` | Inverse Φ⁻¹(p) |
| 7 | `chi2_statistic` | `stats::chi2_decomposed` | Goodness-of-fit |
| 8 | `bootstrap_mean` | `stats::bootstrap_mean` | Result struct mapping |
| 9 | `analytical_localization_length` | `special::localization_length` | Perturbative ξ(W,E) |
| 10 | `bistable_derivative` | `BistableOde::cpu_derivative` | OdeSystem trait |
| 11 | `multisignal_derivative` | `MultiSignalOde::cpu_derivative` | OdeSystem trait |
| 12 | `rmse` | `stats::rmse` | S64 absorption |
| 13 | `mbe` | `stats::mbe` | S64 absorption |
| 14 | `r_squared` | `stats::r_squared` | S64 absorption |
| 15 | `index_of_agreement` | `stats::index_of_agreement` | S64 absorption |
| 16 | `hit_rate` | `stats::hit_rate` | S64 absorption |
| 17 | `shannon_diversity` | `stats::shannon` | u64→f64 conversion |
| 18 | `mean` | `stats::mean` | Direct |
| 19 | `percentile` | `stats::percentile` | Direct |
| 20 | `evenness` | `stats::pielou_evenness` | u64→f64 + S≤1 adapter |

### GPU delegations (`barracuda-gpu` feature) — 5

| # | groundSpring | barracuda | Semantic notes |
|---|-------------|-----------|----------------|
| 21 | `lyapunov_exponent` | `spectral::lyapunov_exponent` | Transfer matrix |
| 22 | `lyapunov_averaged` | `spectral::lyapunov_averaged` | seed convention differs |
| 23 | `almost_mathieu_hamiltonian` | `spectral::almost_mathieu_hamiltonian` | λ/2 coupling convention |
| 24 | `level_spacing_ratio` | `spectral::level_spacing_ratio` | Sort adapter |
| 25 | `almost_mathieu_eigenvalues` | `spectral::find_all_eigenvalues` | Sturm tridiag (49.5×) |

---

## Part 5: Validation State

| Metric | Value |
|--------|-------|
| Experiments | 11 |
| Validation checks | 144/144 PASS |
| Rust tests | 190/190 PASS × 3 modes |
| Clippy warnings | 0 × 3 modes |
| Python tests | 37/37 PASS |
| Coverage | 99.11% |
| Delegations | 25 (20 CPU + 5 GPU) |
| Total benchmark (local) | 14,893 ms |
| Total benchmark (barracuda-gpu) | 3,926 ms (3.8× faster) |
| Best single-experiment speedup | Exp 009: 49.5× (Sturm tridiag) |

---

*groundSpring → ToadStool V15 absorption request. 2 shaders ready for
absorption. 3 semantic improvements requested. 25 delegations validated.
Cross-spring evolution from hotSpring, wetSpring, airSpring, neuralSpring.*
