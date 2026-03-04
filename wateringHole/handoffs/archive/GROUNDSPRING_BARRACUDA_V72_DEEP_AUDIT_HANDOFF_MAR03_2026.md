# SPDX-License-Identifier: AGPL-3.0-only

# groundSpring → barraCuda/toadStool: V72 Deep Audit + Evolution Feedback

**Date:** 2026-03-03
**From:** groundSpring V72
**To:** barraCuda team, toadStool team, ecoPrimals ecosystem
**barraCuda version:** v0.3.1 (standalone)
**groundSpring tests:** 786+ passed, 0 failed
**License:** AGPL-3.0-only

---

## Executive Summary

groundSpring completed a comprehensive codebase audit and deep debt evolution.
All quality gates now pass clean: clippy pedantic (zero warnings), fmt, doc
(-D warnings), full workspace tests, 28/28 validation binaries, 27/27 Python
experiments. This handoff captures what we learned and what the barraCuda and
toadStool teams should absorb.

---

## Part 1: What We Fixed (Lessons for All Springs)

### Silent data defaults are bugs

We found `unwrap_or(0.0)` and `unwrap_or("")` in JSON parsing paths that
silently injected zero-valued records when upstream data was missing or
malformed. This is a validation fidelity risk — zeroed records pass range
checks and contaminate statistical results without warning.

**Pattern to adopt**: Use `let Some(...) = ... else { continue; }` (skip
invalid records) or `expect("descriptive message")` (fail loudly on
mandatory fields). Never default scientific data to zero.

### HashMap breaks determinism

`HashMap` iteration order is non-deterministic. In validation binaries that
enumerate results by iteration order (e.g., assigning day-of-year indices),
this produces different outputs across runs. Use `BTreeMap` for any map
whose iteration order affects output.

### Provenance must be enforced, not defaulted

Our `print_provenance_header` was defaulting missing `_source` and
`baseline_commit` to "unknown". These are mandatory fields (enforced by
`test_baseline_integrity.py`). If a benchmark JSON is missing provenance,
the binary should panic — not silently proceed with "unknown" provenance.

### const fn validation for conditional compilation

When a function has `#[cfg]` branches where one path is trivial (e.g.,
returns `Ok(None)` when a feature is disabled), clippy's
`missing_const_for_fn` fires. The solution is to extract shared validation
into a `const fn` that both paths can call.

---

## Part 2: barraCuda Primitives Consumed (81)

groundSpring consumes 81 barraCuda primitives (47 CPU + 34 GPU):

| Domain | Count | Key Primitives |
|--------|:-----:|----------------|
| Stats (agreement) | 7 | rmse, mae, nash_sutcliffe, mbe, r_squared, index_of_agreement, hit_rate |
| Stats (correlation) | 4 | pearson_r, spearman_r, covariance, std_dev |
| Stats (regression) | 4 | fit_linear, fit_quadratic, fit_exponential, fit_logarithmic |
| Stats (distributions) | 3 | norm_cdf, norm_ppf, chi2_decomposed |
| Stats (metrics) | 4 | mean, percentile, std_dev, sample_std_dev |
| Stats (moving window) | 1 | moving_window_stats_f64 |
| Bootstrap | 4 | bootstrap_mean, rawr_mean, bootstrap_median, bootstrap_std |
| Jackknife | 2 | jackknife_mean_variance, JackknifeMeanGpu |
| Bio diversity | 5 | simpson, shannon, bray_curtis, pielou_evenness, rarefaction_curve |
| Bio evolution | 4 | kimura_fixation_prob, error_threshold, detection_power, detection_threshold |
| Kinetics | 2 | hill, monod |
| Hydrology (FAO-56) | 6 | fao56_et0, hargreaves_et0, crop_coefficient, soil_water_balance, BatchedElementwiseF64, HargreavesBatchGpu |
| FAO-56 pipeline | 3 | SeasonalPipelineF64, StatefulPipeline, WaterBalanceState |
| Anderson spectral | 8 | lyapunov_exponent, lyapunov_averaged, anderson_sweep_averaged, find_w_c, localization_length, anderson_3d_correlated, anderson_4d, wegner_block_4d |
| Almost-Mathieu | 3 | almost_mathieu_hamiltonian, find_all_eigenvalues, level_spacing_ratio |
| Band structure | 2 | brent, detect_bands |
| Linalg | 3 | eigh_f64, cholesky_f64, solve_f64_cpu |
| Optimize | 3 | lbfgs_numerical, grid_search_3d, batched_nelder_mead_gpu |
| ODE | 2 | BistableOde::cpu_derivative, BatchedOdeRK4F64 |
| Drift | 2 | WrightFisherGpu, GillespieGpu |
| Lanczos | 3 | SpectralCsrMatrix, lanczos, lanczos_eigenvalues |
| ESN | 2 | ESN, ESNConfig |
| Rare biosphere | 1 | BatchedMultinomialGpu |
| GPU reduce | 4 | SumReduceF64, VarianceReduceF64, FusedMapReduceF64, CorrelationF64 |
| Device | 2 | WgpuDevice, test_pool::tokio_block_on |

### Chao1 divergence (intentional)

groundSpring's `chao1` uses classic Chao 1984: `f₁²/(2f₂)` with integer
counting. barraCuda uses bias-corrected Chao & Chiu 2016:
`f₁(f₁−1)/(2(f₂+1))` with float equality. Delegation would break Python
baseline provenance. This stays local permanently.

---

## Part 3: What barraCuda Should Absorb

### Tier B gaps remaining

| Gap | Priority | Notes |
|-----|----------|-------|
| FFT (real, complex) | MEDIUM | Blocks spectral_recon full GPU path |
| Eigenvector solver (tridiag) | MEDIUM | CPU-only for transport; barracuda has eigenvalues-only (Sturm) |
| PRNG alignment (xorshift64 → xoshiro128**) | LOW | Full baseline regeneration required; current seeds are stable |
| Parallel 3D grid dispatch | LOW | seismic inversion GPU optimization |

### Patterns we'd like to see evolve

| Request | Priority | Notes |
|---------|----------|-------|
| `unified_hardware::ComputeScheduler` public API | P2 | Could replace metalForge manual substrate routing |
| `device::vendor::VENDOR_*` constants | P3 | Single source of truth for GPU vendor IDs |
| `BenchmarkReport` structured output | P3 | Validation binaries could emit machine-readable reports |

---

## Part 4: Tolerance Architecture (for all Springs)

groundSpring maintains 9 named tolerance constants, each with mathematical
justification:

| Constant | Value | Justification |
|----------|-------|---------------|
| `TOL_DETERMINISM` | 1e-15 | Same seed, same path, IEEE 754 rounding only |
| `TOL_EXACT` | 1e-12 | Deterministic f64 path, identical inputs |
| `TOL_ANALYTICAL` | 1e-10 | One transcendental (sqrt, ln) introducing ~1 ULP |
| `TOL_LITERATURE` | 0.001 | Published values at 3–4 significant decimals |
| `TOL_DECOMPOSITION` | 0.005 | Pythagorean identity RMSE² = MBE² + σ² rounding |
| `TOL_STOCHASTIC_MEAN` | 0.01 | Finite-sample means, O(1/√N) sampling noise |
| `TOL_EQUILIBRIUM` | 0.1 | ODE steady-state approach tolerance |
| `TOL_RAREFACTION_PROP` | 0.05 | Multinomial at N≈50k |
| `TOL_REGIME` | 0.5 | Regime classification (localized/extended/critical) |

Each validation binary documents which tolerance applies and why. This
pattern is recommended for any Spring doing numerical validation.

---

## Part 5: Evolution Readiness (GPU Promotion Map)

### Tier A — Ready for GPU shader promotion

| Module | barracuda Primitive | Status |
|--------|---------------------|--------|
| anderson | lyapunov_*, anderson_sweep, find_w_c | GPU dispatching |
| almost_mathieu | find_all_eigenvalues, level_spacing | GPU dispatching |
| band_structure | brent, detect_bands | GPU dispatching |
| drift | WrightFisherGpu | GPU dispatching |
| fao56 | BatchedElementwiseF64, HargreavesBatchGpu | GPU dispatching |
| freeze_out | grid_search_3d, batched_nelder_mead_gpu | GPU dispatching |
| gillespie | GillespieGpu | GPU dispatching |
| jackknife | JackknifeMeanGpu | GPU dispatching |
| bistable | BatchedOdeRK4F64 | GPU dispatching |
| rarefaction | FusedMapReduceF64, BatchedMultinomialGpu | GPU dispatching |
| tissue_anderson | anderson_3d_correlated, anderson_4d, wegner_block_4d | GPU dispatching |
| esn | ESN | GPU dispatching |
| lanczos | lanczos, lanczos_eigenvalues | GPU dispatching |
| stats | SumReduceF64, VarianceReduceF64, CorrelationF64 | GPU dispatching |

### Tier B — Blocked on barracuda primitives

| Module | Blocker |
|--------|---------|
| spectral_recon | No FFT in barracuda (Cholesky GPU path works) |
| transport | Eigenvector solver not in barracuda (eigenvalues via Sturm work) |

### CPU-only by design

| Module | Reason |
|--------|--------|
| decompose | Bias-variance decomposition (scalar arithmetic) |
| prng | CPU PRNG (GPU uses shader-embedded xoshiro128**) |
| ode | Generic RK4 (GPU goes through BatchedOdeRK4F64) |

---

## Part 6: Quality Summary

| Gate | Status |
|------|--------|
| `cargo fmt --check` | PASS |
| `cargo clippy --all-targets -D warnings` | PASS |
| `cargo doc --no-deps -D warnings` | PASS |
| `cargo test --workspace` | PASS (786+ tests) |
| 28/28 validation binaries | PASS (exit 0) |
| 27/27 Python experiments | PASS (1 skip: NPU) |
| 252/252 baseline integrity | PASS |
| Zero unsafe | PASS |
| Zero todo!() / unimplemented!() | PASS |
| Zero production mocks | PASS |
| All files < 1000 lines | PASS (max 873) |
| AGPL-3.0-only SPDX | PASS (all files) |

---

## Part 7: Provenance

| Metric | V71 | V72 | Change |
|--------|-----|-----|--------|
| barraCuda pin | v0.3.1 | v0.3.1 | Unchanged |
| Active delegations | 81 | 81 | Unchanged |
| Tests | 786 | 786+ | +4 ODE tests |
| Python parity | 28/28 | 28/28 | Unchanged |
| Clippy | FAIL (1 lint) | PASS | Fixed |
| Benchmark provenance | 25/28 data_origin | 28/28 data_origin | Complete |
| Python CI coverage | Not enforced | 80% enforced | New |

---

*groundSpring V72 — 786+ tests, 28 validation binaries, 81 barracuda delegations.
The gap between models and measurements, quantified with provenance.*
