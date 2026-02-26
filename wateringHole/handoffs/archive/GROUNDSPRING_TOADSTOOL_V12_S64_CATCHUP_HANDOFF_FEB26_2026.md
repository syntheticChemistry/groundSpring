# groundSpring → ToadStool/BarraCUDA Handoff V12: S64 Catch-Up + 20 Delegations

**Date**: February 26, 2026
**From**: groundSpring (validation spring — measurement noise characterization)
**To**: ToadStool / BarraCUDA team
**Supersedes**: V11 (full-suite parity + benchmarks, Feb 26)
**ToadStool baseline**: Sessions 50–65 (Feb 23–25, 2026) — S64 stats absorption
**License**: AGPL-3.0-or-later

---

## Executive Summary

ToadStool Session 64 absorbed `stats::metrics` and `stats::diversity` from
airSpring/groundSpring, giving barracuda CPU implementations of RMSE, MBE,
R², IoA, hit rate, Shannon, Simpson, Chao1, and Bray-Curtis. groundSpring
immediately wired 6 new delegations, bringing the total from 14 to **20**.

Three pre-existing barracuda-mode bugs were also fixed:
- `OdeSystem` trait not imported for `BistableOde`/`MultiSignalOde` delegation
- `barracuda::spectral::hofstadter` module path (now private, re-exported at `spectral::`)
- Dead-code warnings for local helpers when barracuda feature is enabled

### Quality Gates (all green)

| Gate | Result |
|------|--------|
| `cargo clippy --workspace -- -D warnings` × 3 modes | **0 warnings** |
| `cargo test --workspace` × 3 modes | **all PASS** |
| 11 validation binaries × 3 modes | **144/144 PASS** |
| `python3 -m pytest tests/` | **34/34 PASS** |
| Mathematical parity (Python ⇌ Rust) | **11/11 PROVEN** |
| Barracuda delegation overhead | **~0%** (release benchmarks) |
| Hardcoded primal names | **Zero** |
| Unsafe Rust | **Forbidden** (workspace lint) |

---

## Part 1: What Changed Since V11

### 6 new barracuda delegations (+6, total 20)

| # | groundSpring fn | barracuda target | Feature | Pattern |
|---|----------------|------------------|---------|---------|
| 15 | `stats::rmse` | `stats::rmse` | `barracuda` | Direct (`#[cfg]` branch) |
| 16 | `stats::mbe` | `stats::mbe` | `barracuda` | Direct (`#[cfg]` branch) |
| 17 | `stats::r_squared` | `stats::r_squared` | `barracuda` | Direct (`#[cfg]` branch) |
| 18 | `stats::index_of_agreement` | `stats::index_of_agreement` | `barracuda` | Direct (`#[cfg]` branch) |
| 19 | `stats::hit_rate` | `stats::hit_rate` | `barracuda` | Direct (`#[cfg]` branch) |
| 20 | `rarefaction::shannon_diversity` | `stats::shannon` | `barracuda` | u64→f64 conversion + direct |

All 6 use the clean `#[cfg]` / `#[cfg(not)]` pattern (no `if let Ok` needed
since barracuda's metrics functions are infallible).

### 3 bug fixes (pre-existing)

1. **OdeSystem trait import**: `BistableOde::cpu_derivative` and
   `MultiSignalOde::cpu_derivative` are trait methods. Added
   `use barracuda::numerical::OdeSystem as _` (feature-gated) to both modules.
2. **hofstadter module path**: `barracuda::spectral::hofstadter` became private
   in a recent ToadStool refactor. Updated to use the re-exported
   `barracuda::spectral::almost_mathieu_hamiltonian`.
3. **Dead-code gates**: Local helper functions (`hill`, `hill_repress`,
   `bistable_derivative_local`, `multisignal_derivative_local`) now gated with
   `#[cfg(not(feature = "barracuda"))]` to avoid warnings when delegating.

### batched_multinomial absorbed (not yet rewired)

ToadStool S64 absorbed `batched_multinomial` as `BatchedMultinomialGpu` +
`multinomial_sample_cpu`. groundSpring rewiring is **deferred** because of a
signature mismatch: barracuda takes `(cumulative_probs, depth, rng_closure)`
while groundSpring takes `(abundances, depth, seed)`. A thin adapter would
bridge these, but isn't urgent since groundSpring's CPU implementation
is validated and performant.

---

## Part 2: Complete Delegation Inventory (20 active)

| # | groundSpring | barracuda | Feature | Notes |
|---|-------------|-----------|---------|-------|
| 1 | `pearson_r` | `stats::pearson_correlation` | `barracuda` | NaN guard |
| 2 | `spearman_r` | `stats::correlation::spearman_correlation` | `barracuda` | NaN guard |
| 3 | `sample_std_dev` | `stats::correlation::std_dev` | `barracuda` | Bessel-corrected |
| 4 | `covariance` | `stats::correlation::covariance` | `barracuda` | Sample covariance |
| 5 | `norm_cdf` | `stats::norm_cdf` | `barracuda` | Infallible |
| 6 | `norm_ppf` | `stats::norm_ppf` | `barracuda` | Acklam rational |
| 7 | `chi2_statistic` | `stats::chi2_decomposed` | `barracuda` | Struct mapping |
| 8 | `bootstrap_mean` | `stats::bootstrap_mean` | `barracuda` | Result struct |
| 9 | `lyapunov_exponent` | `spectral::lyapunov_exponent` | `barracuda-gpu` | Transfer matrix |
| 10 | `lyapunov_averaged` | `spectral::lyapunov_averaged` | `barracuda-gpu` | Multi-realization |
| 11 | `analytical_localization_length` | `special::localization_length` | `barracuda` | Perturbative ξ(W,E) |
| 12 | `almost_mathieu_hamiltonian` | `spectral::almost_mathieu_hamiltonian` | `barracuda-gpu` | λ/2 coupling convention |
| 13 | `bistable_derivative` | `BistableOde::cpu_derivative` | `barracuda` | OdeSystem trait |
| 14 | `multisignal_derivative` | `MultiSignalOde::cpu_derivative` | `barracuda` | OdeSystem trait |
| 15 | `rmse` | `stats::rmse` | `barracuda` | **NEW** S64 absorption |
| 16 | `mbe` | `stats::mbe` | `barracuda` | **NEW** S64 absorption |
| 17 | `r_squared` | `stats::r_squared` | `barracuda` | **NEW** S64 absorption |
| 18 | `index_of_agreement` | `stats::index_of_agreement` | `barracuda` | **NEW** S64 absorption |
| 19 | `hit_rate` | `stats::hit_rate` | `barracuda` | **NEW** S64 absorption |
| 20 | `shannon_diversity` | `stats::shannon` | `barracuda` | **NEW** S64 absorption, u64→f64 |

### Remaining gaps (not in barracuda or not wired)

| groundSpring fn | Barracuda status | Action |
|----------------|------------------|--------|
| `rarefaction::multinomial_sample` | `multinomial_sample_cpu` absorbed (S64) | Wire thin adapter (cumulative_probs + closure) |
| `gillespie::birth_death_ssa` | `GillespieGpu` (GPU-only, no CPU fallback) | Add `simulate_cpu()` to barracuda |
| `bootstrap::rawr_mean` | Not in barracuda | Write RAWR kernel |
| `prng::Xorshift64` | Xoshiro128** in barracuda | PRNG alignment (Phase 2b) |
| `seismic::grid_search_inversion` | No grid-search op | Dispatch as 3D workgroup |

---

## Part 3: Updated Absorption Priorities for ToadStool

### Priority 1: RAWR Weighted Resampling Kernel (unchanged)

CPU reference: `groundspring::bootstrap::rawr_mean()`.
Embarrassingly parallel. No WGSL yet.
**Suggested barracuda target**: `ops::rawr_weighted_mean_f64`

### Priority 2: Gillespie CPU Fallback (unchanged)

`GillespieGpu` is GPU-only. Both wetSpring and groundSpring need CPU fallback.
**Action**: Add `simulate_cpu()` method to barracuda.

### Priority 3: Dense Eigenvalue Solver (unchanged)

Exp 009 custom QR is 19× slower than LAPACK. GPU eigenvalue kernel would close gap.

### Priority 4: multinomial_sample Adapter (NEW)

`multinomial_sample_cpu` is in barracuda but has different signature.
Could be wired with a thin adapter that:
1. Converts abundances to cumulative probabilities
2. Wraps Xorshift64 as a `FnMut() -> f64` closure
3. Converts `Vec<u32>` output to `Vec<u64>`

Low priority since groundSpring's native implementation is validated and fast.

### ~~Priority 5: CPU Error Metrics~~ — DONE

ToadStool S64 absorbed all 6 error metrics. groundSpring has wired them.

---

## Part 4: Delegation Patterns Reference

Two patterns are used depending on barracuda API:

**Pattern A — Infallible functions (direct branch)**:
Used for metrics, distributions, shannon. The barracuda function returns a
plain `f64`, so we branch cleanly:
```rust
#[cfg(feature = "barracuda")]
{
    barracuda::stats::rmse(observed, modeled)
}
#[cfg(not(feature = "barracuda"))]
{
    // local implementation
}
```

**Pattern B — Fallible functions (if-let with CPU fallback)**:
Used for correlation, bootstrap, ODE. The barracuda function returns `Result`,
so we try it and fall back:
```rust
#[cfg(feature = "barracuda")]
{
    if let Ok(r) = barracuda::stats::pearson_correlation(x, y) {
        return if r.is_nan() { 0.0 } else { r };
    }
    0.0
}
#[cfg(not(feature = "barracuda"))]
{
    // local implementation
}
```

### ODE trait pattern
ODE delegations require the `OdeSystem` trait in scope:
```rust
#[cfg(feature = "barracuda")]
use barracuda::numerical::OdeSystem as _;
```

---

## Part 5: Three-Tier Validation Status

### Tier 1: BarraCUDA CPU — COMPLETE

| Status | Count |
|--------|-------|
| Experiments | 11/11 |
| Validation checks | 144/144 PASS |
| Delegations | **20 active** |
| Parity (Python ⇌ Rust) | 11/11 PROVEN |
| Speedup | 23.4× (compute-bound) |

### Tier 2: BarraCUDA GPU — Next

Unchanged from V11. Key blockers:
- `WgpuDevice` lifecycle for GPU adapter promotion
- RAWR kernel, Gillespie CPU fallback
- FFT kernel (not in barracuda)
- Dense eigenvalue solver

### Tier 3: metalForge Cross-Substrate — After GPU

Unchanged from V11.

---

## Handoff Checklist

- [x] 20 delegations verified against S64+ barracuda API
- [x] 6 new delegations wired and validated in three modes
- [x] 3 pre-existing barracuda-mode bugs fixed
- [x] Three-mode clippy: 0 warnings × 3 modes
- [x] Three-mode tests: all PASS × 3 modes
- [x] Three-mode validation: 144/144 checks × 3 modes
- [x] 34 Python tests passing
- [x] Mathematical parity: 11/11 PROVEN
- [x] batched_multinomial absorption noted (rewiring deferred)
- [x] Updated absorption priorities
- [x] V11 archived

---

*groundSpring: 11 experiments, 144 checks, 20 delegations (6 new from S64),
23.4× faster than Python, 11/11 mathematical parity proven, zero barracuda
overhead. ToadStool Priorities 1–3 remain for RAWR, Gillespie CPU, and
GPU eigenvalue kernel.*
