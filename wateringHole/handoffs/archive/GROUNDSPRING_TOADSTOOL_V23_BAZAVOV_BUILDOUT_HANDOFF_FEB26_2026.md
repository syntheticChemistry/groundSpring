# groundSpring → ToadStool/BarraCUDA Handoff V23

**Date**: February 26, 2026
**Scope**: Exp 019–021 Bazavov buildout — jackknife, freeze-out inverse, spectral reconstruction
**Supersedes**: V22 (Exp 016–018)

---

## What Changed

Three new experiments completing the Bazavov inverse-problem paper queue:

| Exp | Paper | Py Checks | Rust Checks | Module | Domain |
|-----|-------|-----------|-------------|--------|--------|
| 019 | Bazavov 2025 Phys Rev D 111, 094508 (muon g-2) | 9/9 | 9/9 | `jackknife` | Statistics/Error Estimation |
| 020 | Bazavov 2016 Phys Rev D 93, 014512 (freeze-out) | 8/8 | 8/8 | `freeze_out` | Inverse Problems |
| 021 | Bazavov 2025 arXiv 2501.12259 (spectral recon) | 8/8 | 8/8 | `spectral_recon` | Inverse Problems/Spectral Reconstruction |

**Cumulative totals**: 21 experiments, 236/236 validation checks, 280 Rust tests,
21/21 mathematical parity, 47 pytest, zero clippy/ruff warnings.

---

## New Modules — GPU Potential

### `jackknife` (Exp 019)

Delete-one and block jackknife resampling for variance estimation.

**Functions**: `jackknife_mean_variance`, `jackknife_bias`, `leave_one_out_biased_variance`,
`block_jackknife_variance`

**GPU opportunity**: Embarrassingly parallel — each of N leave-one-out subsets is
independent. For large N, a single GPU dispatch computes all N leave-means in parallel.
Block jackknife similarly parallelizes over blocks.

**BarraCUDA delegation pattern**: Same as `bootstrap_mean` — fused map-reduce over
resampled subsets.

### `freeze_out` (Exp 020)

Chi-squared fitting on polynomial forward models via 2D grid search.

**Functions**: `freeze_out_curve`, `chi_squared`, `chi_squared_per_dof`, `grid_fit_2d`

**GPU opportunity**: Grid search is embarrassingly parallel — each (T₀, κ₂) grid point
evaluates independently. This is the same pattern as `seismic::grid_search_inversion`
(Exp 005) but with a polynomial forward model instead of travel-time.

**BarraCUDA delegation pattern**: Batched forward model evaluation + parallel chi-squared
reduction. Natural candidate for `BatchedGridSearch` kernel.

### `spectral_recon` (Exp 021)

Tikhonov-regularized spectral function reconstruction from Euclidean correlator.

**Functions**: `build_kernel`, `forward_correlator`, `gaussian_peak`, `tikhonov_solve`,
`peak_index`, `rmse`

**GPU opportunity**: **Highest of the three.** The entire pipeline is dense linear algebra:
- Kernel matrix construction: embarrassingly parallel (each element independent)
- KᵀK matrix multiplication: GEMM — perfect for GPU
- Cholesky decomposition: batched dense Cholesky
- Forward/backward substitution: triangular solves

**BarraCUDA delegation pattern**: This maps directly to `linalg::cholesky_solve_batch`
and `linalg::gemm_f64`. Regularization parameter scan is embarrassingly parallel
(each λ produces an independent solve).

---

## BarraCUDA Delegation Inventory Update

**Existing delegations**: 27 functions (22 CPU + 5 GPU)
**New delegation candidates** (from V22 + V23):

| Function | Module | Priority | Pattern |
|----------|--------|----------|---------|
| `jackknife_mean_variance` | `jackknife` | HIGH | Parallel leave-one-out |
| `block_jackknife_variance` | `jackknife` | MEDIUM | Parallel block deletion |
| `grid_fit_2d` | `freeze_out` | HIGH | Batched grid search (same as seismic) |
| `chi_squared` | `freeze_out` | MEDIUM | Fused map-reduce |
| `build_kernel` | `spectral_recon` | HIGH | Embarrassingly parallel matrix fill |
| `tikhonov_solve` | `spectral_recon` | **CRITICAL** | Cholesky + GEMM |
| `forward_correlator` | `spectral_recon` | HIGH | Dense GEMV |
| `batched_multinomial` | `rarefaction` | HIGH | WGSL production kernel (Tier C) |
| `transfer_matrix_trace_batch` | `band_structure` | MEDIUM | Batched 2×2 products |
| `quasispecies_sweep` | `quasispecies` | MEDIUM | Embarrassingly parallel |

---

## Absorption Priorities for ToadStool

### Tier A — Immediate (existing barracuda primitives can handle)

1. **`jackknife_mean_variance`**: Same fused_map_reduce pattern as `bootstrap_mean`.
   Leave-one-out mean = `(full_sum - data[i]) / (N-1)` — trivially parallel.

2. **`grid_fit_2d`**: Same batched grid dispatch as `seismic::grid_search_inversion`.
   Forward model is a simple polynomial, not travel-time.

3. **`forward_correlator`**: Dense GEMV (`kernel @ rho`). Already exists in barracuda linalg.

### Tier B — New kernel needed

4. **`tikhonov_solve`**: Requires batched Cholesky factorization for the
   `(KᵀK + λI)` system. This is the most valuable new kernel — it unlocks all
   regularized inverse problems. Consider absorbing from LAPACK-style GPU libraries.

5. **`build_kernel`**: Elementwise `exp(-τ·ω)` kernel construction. Trivial WGSL
   compute shader, but needs to be registered as a barracuda op.

### Tier C — Deferred

6. **`block_jackknife_variance`**: Less common than delete-one. Can wait for demand.

---

## Learnings for ToadStool Evolution

### Numerical Patterns

1. **Cholesky decomposition**: The `tikhonov_solve` function implements a
   full Cholesky solve (LLᵀ factorization + forward/backward substitution).
   This is the first groundSpring module requiring a dense linear system solver.
   The CPU implementation is O(n³) — GPU Cholesky would be transformative for
   spectral reconstruction at scale.

2. **Ill-conditioned kernels**: The Laplace kernel `exp(-τω)` is notoriously
   ill-conditioned. At 30×60 discretization, the noiseless roundtrip RMSE is
   ~3×10⁻⁸ (not machine epsilon). The Tikhonov regularization parameter λ
   controls the bias-variance trade-off. GPU implementations must preserve
   f64 precision throughout.

3. **Grid search scaling**: `grid_fit_2d` evaluates O(n_t0 × n_k2) forward
   models, each O(n_data). At current resolution (41 × 21 × 9 = ~7700
   evaluations), CPU is fast. At lattice QCD precision (1000 × 1000 × 100),
   GPU is essential.

### Code Quality Patterns

4. **`#[allow(clippy::many_single_char_names)]`**: Standard linear algebra
   (a, b, l, x, y, n, m, k) legitimately uses single-character names.
   ToadStool should adopt this pattern for linalg modules.

5. **`#[allow(clippy::similar_names)]`**: Grid search parameters
   (`best_t0`/`best_k2`, `n_t0`/`n_k2`) are naturally similar. Allow
   rather than rename to `best_temperature`/`best_kappa`.

---

## Paper Queue Status

All Bazavov papers (6, 7, 8) are now **complete** at CPU tier:

| Paper | Status | GPU Ready? |
|-------|--------|-----------|
| 6 (Spectral recon) | **8/8 PASS** | YES — dense linalg + Cholesky |
| 7 (Muon g-2 / Jackknife) | **9/9 PASS** | YES — parallel leave-one-out |
| 8 (Freeze-out) | **8/8 PASS** | YES — parallel grid search |

**Remaining queued papers**: 22–24 (sub-thesis 06, blocked by GPU tier for Exp 001-004).

---

## Three-Tier Plan (updated)

| Tier | Experiments | Status |
|------|-------------|--------|
| **CPU** (Python + Rust) | 1–21 | **236/236 PASS** |
| **BarraCUDA CPU** | 1–21 | 27 delegations active; 10 new candidates from V22+V23 |
| **BarraCUDA GPU** | 9–12, 14–18, 19–21 | Ready (primitives exist or new kernels identified) |
| **metalForge** | After GPU tier | Blocked on GPU parity |

---

## Action Items for ToadStool

1. **Absorb `jackknife_mean_variance`** — reuse `fused_map_reduce_f64` pattern
2. **Absorb `grid_fit_2d`** — reuse batched grid dispatch from seismic
3. **Add `forward_correlator` dispatch** — dense GEMV already in barracuda
4. **Design `cholesky_solve_batch` kernel** — unlocks Tikhonov and all regularized
   inverse problems
5. **Add `build_kernel` shader** — elementwise exp(-τω) compute kernel
6. **Update delegation count** — 27 → 32+ after absorption

## Action Items for groundSpring

1. Add `#[cfg(feature = "barracuda")]` gates to new modules once absorption complete
2. Create Exp 019-021 performance benchmarks (Python vs Rust timing)
3. Begin BarraCUDA CPU validation for Exp 019-021
4. Plan sub-thesis 22-24 buildout once GPU tier established for Exp 001-004
