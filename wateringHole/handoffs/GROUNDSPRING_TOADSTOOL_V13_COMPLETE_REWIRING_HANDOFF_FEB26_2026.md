# groundSpring → ToadStool/BarraCUDA Handoff V13

**Date**: February 26, 2026
**From**: groundSpring (noise validation + environmental modeling)
**To**: ToadStool/BarraCUDA team
**Previous**: V12 (S64 Catch-Up, archived)

---

## Executive Summary

groundSpring has completed its full rewiring against the modern ToadStool S65
barracuda API. **24 functions** now delegate to barracuda with graceful CPU
fallback, up from 20 in V12.

The headline win: the Sturm tridiag eigenvalue solver (from hotSpring's S26
spectral module, now `barracuda::spectral::find_all_eigenvalues`) enables a
**50× speedup** for Experiment 009 (Quasiperiodic Localization), closing the
LAPACK performance gap that was our only remaining Tier A bottleneck.

**Three-mode validation**: 0 clippy warnings × 3 modes, 144/144 checks × 3
modes. Three-mode benchmark: 14.5s (local) → 3.3s (barracuda-gpu).

---

## Part 1: What Changed (V12 → V13)

### 4 New Delegations

| # | groundSpring fn | barracuda fn | Gate | Impact |
|---|----------------|--------------|------|--------|
| 21 | `stats::mean` | `stats::mean` | `barracuda` | Used across all experiments |
| 22 | `stats::percentile` | `stats::percentile` | `barracuda` | Bootstrap CI computation |
| 23 | `anderson::level_spacing_ratio` | `spectral::level_spacing_ratio` | `barracuda-gpu` | Exp 009 spectral stats |
| 24 | `anderson::almost_mathieu_eigenvalues` | `spectral::find_all_eigenvalues` | `barracuda-gpu` | **50× Exp 009 speedup** |

### Code Changes

1. **`stats/metrics.rs`**: `mean` and `percentile` delegate via `#[cfg(feature = "barracuda")]`.
   The `f64_usize` import is now conditionally compiled (`#[cfg(not(feature = "barracuda"))]`).

2. **`anderson.rs`**:
   - `level_spacing_ratio`: sorts then delegates to `barracuda::spectral::level_spacing_ratio`
     (barracuda assumes sorted input; groundSpring API sorts in place).
   - New `almost_mathieu_eigenvalues(n, coupling, alpha, theta) -> Vec<f64>`:
     - barracuda-gpu: gets tridiag (diag, off) → `find_all_eigenvalues` (O(n²) Sturm)
     - local: builds dense Hamiltonian → Givens QR (O(n³) dense, 100 iterations)
   - Dense Givens QR code moved from validation binary to library, gated behind
     `#[cfg(not(feature = "barracuda-gpu"))]`.

3. **`validate_quasiperiodic.rs`**: Replaced inline `eigenvalues_qr` + 80 lines of QR
   code with single call to `almost_mathieu_eigenvalues`. ~240 lines → ~170 lines.

### Performance Impact

| Experiment | Local (ms) | Barracuda CPU (ms) | Barracuda-GPU (ms) | Speedup |
|-----------|-----------|-------------------|-------------------|---------|
| Exp 009 (quasiperiodic) | 11,717 | 11,355 | **234** | **50×** |
| Total suite | 14,530 | 14,282 | **3,274** | **4.4×** |

The Sturm bisection solver exploits the tridiagonal structure of the
Almost-Mathieu Hamiltonian. This is the exact structure that makes the
solver O(n²) vs O(n³) for dense QR — a fundamental algorithmic improvement,
not just an implementation optimization.

---

## Part 2: Complete Delegation Inventory (24 active)

### CPU delegations (`#[cfg(feature = "barracuda")]`) — 17

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

### GPU delegations (`#[cfg(feature = "barracuda-gpu")]`) — 5

| # | groundSpring | barracuda | Pattern |
|---|-------------|-----------|---------|
| 20 | `lyapunov_exponent` | `spectral::lyapunov_exponent` | `#[cfg]` direct |
| 21 | `lyapunov_averaged` | `spectral::lyapunov_averaged` | `#[cfg]` direct |
| 22 | `almost_mathieu_hamiltonian` | `spectral::almost_mathieu_hamiltonian` | `#[cfg]` λ/2 convention |
| 23 | `level_spacing_ratio` | `spectral::level_spacing_ratio` | `#[cfg]` sort adapter |
| 24 | `almost_mathieu_eigenvalues` | `spectral::find_all_eigenvalues` | `#[cfg]` Sturm tridiag |

---

## Part 3: Cross-Spring Evolution (S58–S65)

The 24 delegations trace their lineage through 5 ecoPrimals Springs:

### hotSpring → Precision + Spectral (Delegations 9, 20-24)

| Session | Contribution | groundSpring Impact |
|---------|-------------|-------------------|
| S26 | spectral/anderson.rs, tridiag.rs | lyapunov_*, level_spacing, **find_all_eigenvalues** |
| S52 | special/anderson_transport.rs | analytical_localization_length |
| S58 | df64_core.wgsl, Fp64Strategy | Consumer GPU f64 precision |
| S60 | DF64 FMA + transcendentals | FP32-core speed for all f64 ops |
| S64 | 8 lattice SU(3) shaders | Nuclear physics in barracuda core |

The Sturm bisection eigenvalue solver (S26) is the direct cause of the
50× Exp 009 speedup. It exploits the tridiagonal structure of the
Almost-Mathieu Hamiltonian — a cross-spring win from nuclear physics.

### wetSpring → Bio-Statistical + Diversity (Delegations 10-11, 17)

| Session | Contribution | groundSpring Impact |
|---------|-------------|-------------------|
| S15 | log_f64 fix, ridge_regression | Shannon entropy accuracy |
| S58 | 5 ODE biosystems | BistableOde, MultiSignalOde derivatives |
| S59 | anderson_3d_correlated, find_w_c | Future Anderson extensions |
| S64 | stats::diversity (Shannon, Simpson, Chao1) | shannon_diversity delegation |

### airSpring → Error Metrics (Delegations 12-16, 18-19)

| Session | Contribution | groundSpring Impact |
|---------|-------------|-------------------|
| S64 | stats::metrics (RMSE, MBE, NSE, R², IoA, hit_rate, mean, percentile) | 7 metric delegations |
| S49 | FAO-56 ET₀ validation | Independent validation of shared metrics |

### neuralSpring → Dispatch + Infrastructure

| Session | Contribution | groundSpring Impact |
|---------|-------------|-------------------|
| S52 | domain_ops.rs dispatch pattern | GPU dispatch blueprint |
| S54 | spectral diagnostics | Level spacing analysis patterns |
| S58 | pow_f64 polyfill (NAK/Ada Lovelace) | f64 transcendentals work on NVVM |
| S59 | ValidationHarness, require! macro | Validation infrastructure |

### groundSpring → Patterns Back to Ecosystem

| Contribution | Benefit |
|-------------|---------|
| `if let Ok` + CPU fallback | Adopted as wateringHole delegation standard |
| `ValidationHarness` pattern | Absorbed as `barracuda::validation::ValidationHarness` |
| Three-mode validation | Proves correctness across feature configurations |
| Dense QR → Sturm demonstration | Quantified 50× win of algorithmic choice |

---

## Part 4: Three-Mode Benchmark (Feb 26, 2026)

| Binary | Local (ms) | Barracuda (ms) | Barra-GPU (ms) | Checks |
|--------|-----------|---------------|----------------|--------|
| validate-decompose | 62 | 71 | 86 | 36/36 |
| validate-rarefaction | 66 | 79 | 92 | 15/15 |
| validate-seismic | 121 | 125 | 140 | 9/9 |
| validate-weather | 59 | 70 | 86 | 13/13 |
| validate-fao56 | 71 | 82 | 97 | 15/15 |
| validate-signal-specificity | 839 | 856 | 872 | 12/12 |
| validate-rawr | 611 | 620 | 635 | 11/11 |
| validate-anderson | 728 | 744 | 722 | 8/8 |
| validate-quasiperiodic | **11,717** | **11,355** | **234** | 8/8 |
| validate-bistable | 169 | 185 | 202 | 9/9 |
| validate-multisignal | 87 | 95 | 108 | 8/8 |
| **TOTAL** | **14,530** | **14,282** | **3,274** | **144/144** |

All 144/144 validation checks PASS in all three modes.

---

## Part 5: Remaining Tier B/C Items

| Item | Status | Notes |
|------|--------|-------|
| PRNG alignment (Xorshift64 → Xoshiro128**) | Phase 2b | Requires full rebaseline |
| Gillespie CPU fallback | Phase 2b | `GillespieGpu` is GPU-only |
| RAWR kernel | Phase 2c | Embarrassingly parallel, needs metalForge shader |
| `batched_multinomial` rewiring | Phase 2c | Signature mismatch (cumulative_probs + closure RNG) |
| `mc_et0_propagate` rewiring | Phase 2c | Superseded by `Op::Fao56Et0` batch |

---

## Part 6: Suggestions for ToadStool

1. **Consider re-exporting tridiag solver at `spectral` level**: `find_all_eigenvalues`
   is powerful and widely useful — making it easier to discover would benefit all Springs.

2. **CPU `rarefaction_curve`**: barracuda's uses exact hypergeometric; groundSpring uses
   multinomial sampling. Both are valid but not interchangeable. Consider documenting
   the algorithmic difference.

3. **`batched_multinomial` API**: The closure-based RNG makes delegation harder. A
   seed-based API would let Springs delegate without carrying the PRNG state.

4. **The cross-spring story matters**: groundSpring's 50× Exp 009 speedup came from
   a nuclear physics eigenvalue solver (hotSpring S26). Document these wins in
   ToadStool's STATUS.md — it validates the multi-spring architecture.

---

*groundSpring: 11 experiments, 144 checks, 24 delegations (4 new from complete rewiring),
50× Exp 009 speedup from Sturm tridiag solver, three-mode validated.*
