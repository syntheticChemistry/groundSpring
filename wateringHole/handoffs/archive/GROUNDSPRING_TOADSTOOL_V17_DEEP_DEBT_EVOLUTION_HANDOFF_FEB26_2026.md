# groundSpring → ToadStool/BarraCUDA V17 — Deep Debt Evolution + Absorption Guidance

**Date**: February 26, 2026
**From**: groundSpring (noise validation + environmental modeling)
**To**: ToadStool/BarraCUDA team
**Previous**: V16 (S66 catch-up + rewiring)
**ToadStool HEAD**: `045103a7` (S66 Wave 5)
**groundSpring HEAD**: `de0822a` (deep debt evolution)
**License**: AGPL-3.0-or-later

---

## Executive Summary

- **26 active delegations** (21 CPU + 5 GPU) fully validated across 3 modes
- **Bug fixed**: `covariance`, `pearson_r`, `spearman_r` were silently returning
  0.0 on barracuda error instead of falling through to CPU — now fixed
- **20 `#[allow(unreachable_code)]` eliminated** via proper `#[cfg]` patterns
- **14 experiments, 177/177 checks, 205/205 tests** across all three modes
- **4 open V15 requests remain** — pielou S≤1, spectral CPU promotion,
  batched_multinomial seed API, mc_et0 shader absorption
- **3 new experiments (012-014)** provide barracuda absorption candidates:
  `tridiag_eigh` → `eigh_f64`, `wright_fisher_fixation` → `WrightFisherGpu`,
  `transport_exponent` → `fit_linear`

---

## Part 1: Bug Report — Barracuda Error Handling in Springs

### 1.1 The covariance/correlation bug (FIXED in groundSpring)

When `barracuda::stats::correlation::covariance()` returned `Err`, groundSpring
was returning `0.0` instead of falling through to the CPU implementation. Same
bug existed in `pearson_r` and `spearman_r`.

**Root cause**: The delegation pattern used an exhaustive block:

```rust
#[cfg(feature = "barracuda")]
{
    if let Ok(c) = barracuda::stats::correlation::covariance(x, y) {
        return c;
    }
    return 0.0;  // BUG: should fall through to CPU
}
```

**Fix**: Remove the `return 0.0;` so the function falls through to CPU:

```rust
#[cfg(feature = "barracuda")]
if let Ok(c) = barracuda::stats::correlation::covariance(x, y) {
    return c;
}
covariance_cpu(x, y)
```

**toadStool action**: Check if other Springs have the same pattern. The
`return 0.0` after `if let Ok` is a common copy-paste error when adapting
the infallible delegation pattern (`return barracuda::fn()`) to fallible
calls (`if let Ok`). Consider adding a clippy lint or doc warning about this.

### 1.2 Recommended delegation patterns for Springs

groundSpring now uses two clean patterns with zero lint suppressions:

**Pattern A — Infallible barracuda calls (return value directly):**

```rust
pub fn rmse(observed: &[f64], modeled: &[f64]) -> f64 {
    #[cfg(feature = "barracuda")]
    return barracuda::stats::rmse(observed, modeled);
    #[cfg(not(feature = "barracuda"))]
    rmse_cpu(observed, modeled)
}
```

**Pattern B — Fallible barracuda calls (return Result):**

```rust
pub fn pearson_r(x: &[f64], y: &[f64]) -> f64 {
    #[cfg(feature = "barracuda")]
    if let Ok(r) = barracuda::stats::pearson_correlation(x, y) {
        return if r.is_nan() { 0.0 } else { r };
    }
    pearson_r_cpu(x, y)
}
```

Key principles:
1. CPU `_cpu` functions are always compiled (never behind `#[cfg(not)]`)
2. The *call site* in the public function uses `#[cfg]` gating
3. No `#[allow(unreachable_code)]` needed with either pattern
4. Fallible calls ALWAYS fall through to CPU on error

**toadStool action**: Consider documenting these patterns in a shared guide
for all Springs, or adding them to the wateringHole delegation standard.

---

## Part 2: Convention Differences (Active Adapters)

These semantic mismatches require pre-delegation adapters in groundSpring.
Resolving them upstream would simplify all consuming Springs.

### 2.1 `pielou_evenness` S≤1 behavior (V15 request, still open)

| | barracuda | Ecology convention | groundSpring adapter |
|---|-----------|-------------------|---------------------|
| S=0 | returns 0.0 | returns 1.0 (trivially even) | `if s <= 1 { return 1.0; }` |
| S=1 | returns 0.0 | returns 1.0 | same adapter |

Major ecology packages (vegan R, scipy, scikit-bio) all return 1.0 for S≤1.

**toadStool action**: Update `barracuda::stats::pielou_evenness` to return
1.0 for S≤1. This removes adapter code from groundSpring, wetSpring, and
any future ecological consumer.

### 2.2 `almost_mathieu_hamiltonian` coupling convention

| | barracuda | groundSpring |
|---|-----------|-------------|
| Potential | V_i = 2λ cos(2παi + θ) | V_i = λ cos(2παi + θ) |
| Transition | λ = 1 | λ = 2 |
| Adapter | — | Pass coupling/2 to barracuda |

This is a known convention difference (some papers include the factor of 2,
others don't). Both are valid; the adapter is documented and tested.

**No action needed** — just awareness for consumers.

### 2.3 `bootstrap ≠ RAWR` comparison semantics

When both `bootstrap_mean` and `rawr_mean` delegate to barracuda, they
both return the sample mean as the point estimate for small symmetric data
(e.g., `[1, 2, 3, 4, 5]`). groundSpring's test now compares CI widths
instead of point estimates.

**toadStool action**: Note in barracuda's bootstrap docs that `estimate`
is the sample mean (not the median of replicates), so two methods may
produce identical point estimates even though their CI widths differ.

---

## Part 3: Spectral CPU Promotion (V15 request, still open)

Five barracuda spectral functions are pure CPU implementations but are
gated behind `#[cfg(feature = "gpu")]`:

| Function | Pure CPU? | Reason for GPU gate |
|----------|----------|---------------------|
| `lyapunov_exponent` | Yes — transfer matrix product | Shares module with GPU SpMV |
| `lyapunov_averaged` | Yes — calls `lyapunov_exponent` in loop | Same |
| `level_spacing_ratio` | Yes — sort + ratio computation | Shares module with GPU eigensolvers |
| `find_all_eigenvalues` | Yes — Sturm bisection | Same |
| `almost_mathieu_hamiltonian` | Yes — vector generation | Shares module with Hofstadter GPU |

groundSpring must use `barracuda-gpu` feature (which pulls in wgpu) just
to access these 5 pure-CPU functions. This increases build times and
adds unnecessary GPU dependencies.

**toadStool action**: Promote these 5 functions to a CPU-accessible module
(e.g., `spectral_cpu` or `numerical::spectral`) so Springs can use
`barracuda` (no GPU) for these delegations. This would let groundSpring
change 5 delegations from `barracuda-gpu` to `barracuda`, significantly
reducing build times for CI.

---

## Part 4: New Experiments — Future Barracuda Absorption Candidates

### 4.1 Exp 012: Spin Chain Transport (Kachkovskiy 2016)

**New primitive**: `tridiag_eigh` — Symmetric tridiagonal eigendecomposition
with eigenvectors via implicit QL algorithm (Wilkinson shifts).

| Property | Value |
|----------|-------|
| Location | `groundspring::transport::tridiag_eigh` |
| Input | `(diag: &[f64], offdiag: &[f64])` |
| Output | `(eigenvalues: Vec<f64>, eigenvectors: Vec<Vec<f64>>)` |
| Algorithm | Implicit QL with Wilkinson shifts, O(n²) for tridiagonal |
| Validated | 18/18 checks, 3 coupling strengths × MSD + transport exponent |
| Lines | ~120 (core algorithm) |

**barracuda absorption path**: `linalg::eigh_f64` is dense O(n³). A
tridiagonal-specific path would be O(n²) and more memory efficient.
Alternatively, groundSpring could convert to dense and use `eigh_f64`.

**toadStool action**: Consider adding `linalg::tridiag_eigh` as a
specialized path. The QL algorithm is standard (LAPACK's `dsteqr`
equivalent). groundSpring's implementation is 120 lines, fully tested,
with named constants for iteration limits.

### 4.2 Exp 014: Drift vs Selection (R. Anderson 2022)

**New primitives**:

| Function | Location | Description |
|----------|----------|-------------|
| `wright_fisher_fixation` | `drift.rs` | Run Wright-Fisher to fixation/loss |
| `kimura_fixation_prob` | `drift.rs` | Analytical P_fix = (1-e^(-4Nsp₀))/(1-e^(-4Ns)) |
| `neutral_diversity_trajectory` | `drift.rs` | Shannon diversity under neutral drift |
| `prng::binomial` | `prng.rs` | Binomial sampling for Wright-Fisher |

**barracuda absorption path**: `WrightFisherGpu` exists for single-step
evolution. For fixation, the CPU loops to completion. A GPU kernel that
runs N parallel trajectories to fixation would be the ideal absorption.

**toadStool action**: Consider `ops::bio::wright_fisher_fixation_batch`
that runs K trajectories to fixation in parallel on GPU. This is
embarrassingly parallel and would give massive speedup for
population genetics experiments.

### 4.3 Exp 013: Resampling Convergence (Lee & Liu 2024)

Uses existing `bootstrap_mean` and `rawr_mean` — no new primitives.
Validates that CI width decreases monotonically with replicate count and
that coverage ≥ 0.90 at 10k replicates. Confirms barracuda's `rawr_mean`
produces correct convergence behavior.

---

## Part 5: Deep Debt Patterns Relevant to ToadStool Evolution

### 5.1 `#[allow]` debt elimination

groundSpring went from 21 `#[allow]` annotations to 1 (only
`clippy::many_single_char_names` in the QL algorithm, which is appropriate
for standard mathematical notation like LAPACK variable names).

The key insight: `#[allow(unreachable_code)]` was masking a real bug (the
covariance 0.0 return). Lint suppressions should be treated as debt and
audited regularly.

**toadStool action**: Audit barracuda's `#[allow]` and `#[expect]`
annotations. Each one may be hiding a real issue. Consider running
`grep -c '#\[allow\|#\[expect' crates/barracuda/src/**/*.rs` periodically.

### 5.2 Copy vs Clone for small parameter structs

groundSpring's `BistableParams` and `MultiSignalParams` (5-7 `f64` fields)
now derive `Copy` instead of using `.clone()`. This eliminates heap
allocation for parameter passing.

**Pattern**: If all fields are `Copy` types (`f64`, `usize`, `bool`, `u64`),
derive `Copy` on the struct. Reserve `Clone` for types with heap allocations.

### 5.3 Named constants for algorithm parameters

Magic numbers like `30` (QL iterations), `100` (QR iterations), `96.0`
(Derrida-Gardner constant) are now named constants with documented
provenance. This makes the code self-documenting and prevents silent
divergence when algorithms are tuned.

### 5.4 Iterator patterns for numerical code

`for i in 0..times.len() - 1` replaced with `.windows(2)` — more
idiomatic, bounds-checked, and clearly expresses intent. Works well for
time-series pair iteration (Gillespie SSA, transport exponent fitting).

---

## Part 6: Complete Delegation Inventory (26 active)

### CPU delegations (`barracuda` feature) — 21

| # | groundSpring | barracuda | Pattern |
|---|-------------|-----------|---------|
| 1 | `pearson_r` | `stats::pearson_correlation` | B (fallible) |
| 2 | `spearman_r` | `stats::correlation::spearman_correlation` | B (fallible) |
| 3 | `sample_std_dev` | `stats::correlation::std_dev` | B (fallible) |
| 4 | `covariance` | `stats::correlation::covariance` | B (fallible, **bug fixed**) |
| 5 | `norm_cdf` | `stats::norm_cdf` | A (infallible) |
| 6 | `norm_ppf` | `stats::norm_ppf` | A (infallible) |
| 7 | `chi2_statistic` | `stats::chi2_decomposed` | B (fallible) |
| 8 | `bootstrap_mean` | `stats::bootstrap_mean` | B (fallible) |
| 9 | `rawr_mean` | `stats::rawr_mean` | B (fallible, **S66 new**) |
| 10 | `analytical_localization_length` | `special::localization_length` | A (infallible) |
| 11 | `bistable_derivative` | `BistableOde::cpu_derivative` | A (infallible) |
| 12 | `multisignal_derivative` | `MultiSignalOde::cpu_derivative` | A (infallible) |
| 13 | `rmse` | `stats::rmse` | A (infallible) |
| 14 | `mbe` | `stats::mbe` | A (infallible) |
| 15 | `r_squared` | `stats::r_squared` | A (infallible) |
| 16 | `index_of_agreement` | `stats::index_of_agreement` | A (infallible) |
| 17 | `hit_rate` | `stats::hit_rate` | A (infallible) |
| 18 | `shannon_diversity` | `stats::shannon` | A (infallible) |
| 19 | `mean` | `stats::mean` | A (infallible) |
| 20 | `percentile` | `stats::percentile` | A (infallible) |
| 21 | `evenness` | `stats::pielou_evenness` | A (infallible) + S≤1 adapter |

### GPU delegations (`barracuda-gpu` feature) — 5

| # | groundSpring | barracuda | Pattern |
|---|-------------|-----------|---------|
| 22 | `lyapunov_exponent` | `spectral::lyapunov_exponent` | A (infallible) |
| 23 | `lyapunov_averaged` | `spectral::lyapunov_averaged` | A (infallible) |
| 24 | `almost_mathieu_hamiltonian` | `spectral::almost_mathieu_hamiltonian` | A (infallible) + λ/2 adapter |
| 25 | `level_spacing_ratio` | `spectral::level_spacing_ratio` | A (infallible) |
| 26 | `almost_mathieu_eigenvalues` | `spectral::find_all_eigenvalues` | A (infallible) |

---

## Part 7: Validation State

| Metric | Value |
|--------|-------|
| Experiments | 14 |
| Validation checks | 177/177 PASS × 3 modes |
| Rust tests | 205/205 PASS × 3 modes |
| Clippy warnings | 0 × 3 modes |
| `#[allow]` annotations | 1 (clippy::many_single_char_names in QL) |
| Python ruff | 0 errors |
| Delegations | 26 (21 CPU + 5 GPU) |
| Mathematical parity | 14/14 PROVEN |

---

## Part 8: Action Items Summary

| # | Action | Priority | Who |
|---|--------|----------|-----|
| 1 | Fix `pielou_evenness` S≤1 → return 1.0 | High | toadStool |
| 2 | Promote 5 spectral functions to CPU module | High | toadStool |
| 3 | Add seed-based `batched_multinomial` API | Medium | toadStool |
| 4 | Absorb `mc_et0_propagate.wgsl` shader | Medium | toadStool |
| 5 | Document delegation patterns (A/B) for Springs | Medium | toadStool/wateringHole |
| 6 | Consider `linalg::tridiag_eigh` specialized path | Low | toadStool |
| 7 | Consider `wright_fisher_fixation_batch` GPU kernel | Low | toadStool |
| 8 | Audit `#[allow]`/`#[expect]` across barracuda | Low | toadStool |
| 9 | Document bootstrap estimate semantics (sample mean) | Low | toadStool |

---

*groundSpring → ToadStool V17. Deep debt evolution complete: 0 bugs,
1 lint suppression, 26 validated delegations. 9 action items for upstream
evolution. 3 new absorption candidates from Exp 012-014. Delegation
patterns documented for cross-spring adoption.*
