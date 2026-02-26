# groundSpring → ToadStool/BarraCUDA Handoff V14

**Date**: February 26, 2026
**From**: groundSpring (noise validation + environmental modeling)
**To**: ToadStool/BarraCUDA team
**Previous**: V13 (Complete Rewiring, archived)
**ToadStool HEAD**: `17932267` (S65: Smart refactoring + doc cleanup)
**groundSpring HEAD**: `520ca659`

---

## Executive Summary

Full revalidation against ToadStool S65 HEAD. **25 functions** now delegate to
barracuda (up from 24 in V13). New delegation: `evenness` →
`barracuda::stats::pielou_evenness` with edge-case semantic correction.

Additional code quality pass: `stats/correlation.rs` Barracuda delegation
pattern modernized (CPU code always compiled), Python baselines cleaned
(ruff zero-warning), benchmark drift-guard script created.

**Three-mode validation**: 0 clippy warnings × 3 modes, 190/190 tests × 3
modes. Quasiperiodic speedup: 130.97s (local) → 0.18s (barracuda-gpu) = **727×**.

---

## Part 1: What Changed (V13 → V14)

### 1 New Delegation

| # | groundSpring fn | barracuda fn | Gate | Notes |
|---|----------------|--------------|------|-------|
| 25 | `evenness` | `stats::pielou_evenness` | `barracuda` | Semantic adapter: groundSpring returns 1.0 for S≤1 (ecology convention); barracuda returns 0.0. Adapter pre-checks S≤1 before delegating. |

### Code Quality Improvements

1. **`stats/correlation.rs`**: Removed `#[cfg(not(feature = "barracuda"))]` gates
   from imports and CPU code. Extracted `pearson_r_cpu`, `spearman_r_cpu`,
   `covariance_cpu`, `rank` as always-compiled private functions. Module now
   matches the delegation pattern used across all other modules.

2. **Python baselines**: Fixed 14 ruff errors across `bistable_switching.py` and
   `multisignal_qs.py` (import sorting, `zip(strict=True)`, unused variables).
   Python linting now zero-warning.

3. **`scripts/regenerate_benchmarks.sh`**: New drift-guard script that re-runs all
   11 Python baselines, verifies `baseline_commit` in each benchmark JSON matches
   HEAD, and optionally stamps provenance with `--stamp`.

4. **SPDX audit**: All 30 Rust source files and 21 Python source files confirmed
   to have AGPL-3.0-or-later headers.

---

## Part 2: Complete Delegation Inventory (25 active)

### CPU delegations (`#[cfg(feature = "barracuda")]`) — 20

| # | groundSpring | barracuda | Pattern |
|---|-------------|-----------|---------|
| 1 | `pearson_r` | `stats::pearson_correlation` | `if let Ok` |
| 2 | `spearman_r` | `stats::correlation::spearman_correlation` | `if let Ok` |
| 3 | `sample_std_dev` | `stats::correlation::std_dev` | `if let Ok` |
| 4 | `covariance` | `stats::correlation::covariance` | `if let Ok` |
| 5 | `norm_cdf` | `stats::norm_cdf` | `#[cfg]` direct |
| 6 | `norm_ppf` | `stats::norm_ppf` | `#[cfg]` direct |
| 7 | `chi2_statistic` | `stats::chi2_decomposed` | `map_or` struct |
| 8 | `bootstrap_mean` | `stats::bootstrap_mean` | `if let Ok` |
| 9 | `analytical_localization_length` | `special::localization_length` | `#[cfg]` direct |
| 10 | `bistable_derivative` | `BistableOde::cpu_derivative` | `#[cfg]` OdeSystem |
| 11 | `multisignal_derivative` | `MultiSignalOde::cpu_derivative` | `#[cfg]` OdeSystem |
| 12 | `rmse` | `stats::rmse` | `#[cfg]` direct |
| 13 | `mbe` | `stats::mbe` | `#[cfg]` direct |
| 14 | `r_squared` | `stats::r_squared` | `#[cfg]` direct |
| 15 | `index_of_agreement` | `stats::index_of_agreement` | `#[cfg]` direct |
| 16 | `hit_rate` | `stats::hit_rate` | `#[cfg]` direct |
| 17 | `shannon_diversity` | `stats::shannon` | `#[cfg]` u64→f64 |
| 18 | `mean` | `stats::mean` | `#[cfg]` direct |
| 19 | `percentile` | `stats::percentile` | `#[cfg]` direct |
| 20 | `evenness` | `stats::pielou_evenness` | `#[cfg]` u64→f64 + S≤1 adapter |

### GPU delegations (`#[cfg(feature = "barracuda-gpu")]`) — 5

| # | groundSpring | barracuda | Pattern |
|---|-------------|-----------|---------|
| 21 | `lyapunov_exponent` | `spectral::lyapunov_exponent` | `#[cfg]` direct |
| 22 | `lyapunov_averaged` | `spectral::lyapunov_averaged` | `#[cfg]` direct |
| 23 | `almost_mathieu_hamiltonian` | `spectral::almost_mathieu_hamiltonian` | `#[cfg]` λ/2 convention |
| 24 | `level_spacing_ratio` | `spectral::level_spacing_ratio` | `#[cfg]` sort adapter |
| 25 | `almost_mathieu_eigenvalues` | `spectral::find_all_eigenvalues` | `#[cfg]` Sturm tridiag |

---

## Part 3: Feature Gate Architecture

groundSpring defines two feature flags:

| Feature | Activates | Barracuda modules available |
|---------|-----------|---------------------------|
| `barracuda` | `dep:barracuda` (CPU only) | `stats`, `special`, `numerical`, `linalg`, `math`, `validation` |
| `barracuda-gpu` | `barracuda` + `barracuda/gpu` | Above + `spectral`, `tensor`, `device`, `ops`, `dispatch` |

The `spectral` module in barracuda is gated behind `#[cfg(feature = "gpu")]`
in barracuda's `lib.rs` (line 142). This is why groundSpring gates
`lyapunov_*`, `level_spacing_ratio`, and `find_all_eigenvalues` behind
`barracuda-gpu` even though the implementations are CPU-only — the module
itself is only compiled when barracuda's `gpu` feature is active.

**Suggestion for ToadStool**: Consider promoting the pure-CPU spectral
functions (`lyapunov_exponent`, `lyapunov_averaged`, `level_spacing_ratio`,
`find_all_eigenvalues`, `almost_mathieu_hamiltonian`) to a CPU-only module
(e.g., `spectral_cpu` or moving them to `numerical`). This would let Springs
use them without pulling in the full GPU dependency chain.

---

## Part 4: Semantic Differences Documented

| Function | barracuda | groundSpring | Resolution |
|----------|-----------|-------------|------------|
| `pielou_evenness` (S≤1) | returns 0.0 | returns 1.0 | groundSpring pre-checks S≤1 before delegating |
| `almost_mathieu_hamiltonian` λ | `λ` = coupling | `coupling` = `2λ` | groundSpring passes `coupling / 2.0` to barracuda |
| `lyapunov_averaged` seed | `base_seed + r * 1000` | `base_seed + r` | Uses barracuda's convention when delegating |
| `rarefaction` algorithm | Exact hypergeometric | Multinomial sampling | Documented: different algorithms, both valid |
| `batched_multinomial` RNG | Closure-based `cumulative_probs` | Seed-based `Xorshift64` | Not delegated (signature mismatch) |

---

## Part 5: Three-Mode Test Results (Feb 26, 2026)

| Mode | Tests | Clippy | fmt | doc |
|------|-------|--------|-----|-----|
| default | 190/190 pass | 0 warnings | clean | clean |
| barracuda | 190/190 pass | 0 warnings | clean | clean |
| barracuda-gpu | 190/190 pass | 0 warnings | clean | clean |

### Validation Binary Timing (release profile)

| Binary | Default (ms) | Barracuda (ms) | Barra-GPU (ms) | Speedup | Checks |
|--------|-------------|---------------|----------------|---------|--------|
| validate-decompose | 82 | 71 | 560 | 0.1× ¹ | 36/36 |
| validate-rarefaction | 70 | 99 | 102 | 0.7× ¹ | 15/15 |
| validate-seismic | 141 | 128 | 171 | 0.8× ¹ | 9/9 |
| validate-weather | 65 | 71 | 97 | 0.7× ¹ | 13/13 |
| validate-fao56 | 79 | 80 | 106 | 0.7× ¹ | 15/15 |
| validate-signal-specificity | 854 | 858 | 898 | 1.0× | 12/12 |
| validate-rawr | 619 | 625 | 651 | 1.0× | 11/11 |
| validate-anderson | 745 | 745 | 774 | 1.0× | 8/8 |
| validate-quasiperiodic | **11,986** | **11,867** | **242** | **49.5×** | 8/8 |
| validate-bistable | 167 | 222 | 207 | 0.8× | 9/9 |
| validate-multisignal | 85 | 118 | 118 | 0.7× | 8/8 |
| **TOTAL** | **14,893** | **14,884** | **3,926** | **3.8×** | **144/144** |

¹ GPU initialization overhead dominates for sub-100ms workloads.

---

## Part 6: Remaining Tier B/C Items

| Item | Status | Notes |
|------|--------|-------|
| PRNG alignment (Xorshift64 → Xoshiro128**) | Phase 2b | Requires full rebaseline |
| Gillespie CPU fallback | Phase 2b | `GillespieGpu` is GPU-only |
| RAWR kernel | Phase 2c | Embarrassingly parallel, needs metalForge shader |
| `batched_multinomial` rewiring | Phase 2c | Signature mismatch (closure RNG) |
| `mc_et0_propagate` rewiring | Phase 2c | Superseded by `Op::Fao56Et0` batch |
| Spectral CPU promotion | Suggestion | Move pure-CPU spectral fns to CPU-only module |

---

## Part 7: Suggestions for ToadStool

1. **Promote pure-CPU spectral functions**: `lyapunov_exponent`,
   `lyapunov_averaged`, `level_spacing_ratio`, `find_all_eigenvalues`,
   `almost_mathieu_hamiltonian` are all CPU implementations. Moving them
   out of the GPU-gated `spectral` module would let Springs use them
   without `barracuda/gpu`, reducing dependency weight.

2. **`pielou_evenness` edge case**: Consider returning `1.0` for S≤1 to
   match the ecology convention (used by vegan, scipy, skbio). Currently
   returns `0.0`, which requires all consumers to add an adapter.

3. **Seed-based `batched_multinomial`**: A `(abundances, depth, seed)` API
   alongside the closure-based one would simplify Spring delegation.

4. **Cross-spring changelog**: ToadStool absorptions from S39-S65 represent
   significant cross-spring value. Consider a changelog entry for each
   absorption so Springs can discover what's newly available.

---

*groundSpring: 11 experiments, 144 checks, 25 delegations (1 new from V13),
727× Exp 009 speedup from Sturm tridiag solver, three-mode validated.
All 190 tests pass in all three feature modes.*
