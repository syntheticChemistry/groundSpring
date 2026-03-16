# groundSpring → ToadStool V55 Handoff: barracuda Evolution Review + Absorption Guide

**Date**: February 28, 2026
**From**: groundSpring (V55)
**To**: ToadStool / BarraCUDA team
**ToadStool pin**: S70+++ (`1dd7e338`)
**License**: AGPL-3.0-only

---

## Executive Summary

groundSpring has reached full maturity: 28 experiments validated against
open data, 292/292 checks PASS, 57 active barracuda delegations (38 CPU +
19 GPU), 95/95 three-tier parity tests proving CPU = barracuda-CPU, and
Rust measured 11.6× faster than Python (excl. LAPACK-bound). The math
portability chain Python → Rust → barracuda CPU is **proven**.

This handoff provides:
1. Complete inventory of what groundSpring delegates to barracuda
2. What groundSpring learned that's relevant to barracuda's evolution
3. The 1 remaining evolution candidate and what it needs
4. Cross-spring shader lineage for provenance tracking
5. Performance data for barracuda's benchmark suite

---

## Part 1: Complete Delegation Inventory (57 Active)

### CPU Delegations (38) — `#[cfg(feature = "barracuda")]`

| Module | Function | barracuda Target | Absorbed |
|--------|----------|-----------------|----------|
| stats/agreement | rmse | barracuda::stats::rmse | S64 |
| stats/agreement | mae | barracuda::stats::mae | S64 |
| stats/agreement | nash_sutcliffe | barracuda::stats::nash_sutcliffe | S64 |
| stats/agreement | mbe | barracuda::stats::mbe | S64 |
| stats/agreement | r_squared | barracuda::stats::r_squared | S64 |
| stats/agreement | index_of_agreement | barracuda::stats::index_of_agreement | S64 |
| stats/agreement | hit_rate | barracuda::stats::hit_rate | S64 |
| stats/metrics | mean | barracuda::stats::mean | S64 |
| stats/metrics | sample_std_dev | barracuda::stats::sample_std_dev | S64 |
| stats/metrics | percentile | barracuda::stats::percentile | S64 |
| stats/correlation | pearson_r | barracuda::stats::pearson_r | S64 |
| stats/correlation | spearman_r | barracuda::stats::spearman_r | S64 |
| stats/correlation | covariance | barracuda::stats::covariance | S64 |
| stats/distributions | norm_cdf | barracuda::stats::norm_cdf | S64 |
| stats/distributions | norm_ppf | barracuda::stats::norm_ppf | S64 |
| stats/distributions | chi2_statistic | barracuda::stats::chi2_statistic | S64 |
| stats/regression | fit_linear | barracuda::stats::regression::fit_linear | S66 |
| stats/regression | fit_quadratic | barracuda::stats::regression::fit_quadratic | S66 |
| stats/regression | fit_exponential | barracuda::stats::regression::fit_exponential | S66 |
| stats/regression | fit_logarithmic | barracuda::stats::regression::fit_logarithmic | S66 |
| stats/moving_window | moving_window_stats | barracuda::stats::moving_window_stats_f64 | S66 |
| bootstrap | bootstrap_mean | barracuda::stats::bootstrap_mean | S64 |
| bootstrap | rawr_mean | barracuda::stats::rawr_mean | S66 |
| bootstrap | bootstrap_median | barracuda::stats::bootstrap_median | S64 |
| bootstrap | bootstrap_std | barracuda::stats::bootstrap_std | S64 |
| kinetics | hill | barracuda::stats::hill | S68 |
| kinetics | monod | barracuda::stats::monod | S68 |
| kinetics | hill_repress | via 1.0 - barracuda::stats::hill | S68 |
| rarefaction | simpson_diversity | barracuda::stats::simpson | S64 |
| rarefaction | bray_curtis | barracuda::stats::bray_curtis | S64 |
| rarefaction | analytical_rarefaction | barracuda::stats::analytical_rarefaction | S64 |
| rarefaction | shannon_diversity | barracuda::stats::shannon_diversity | S64 |
| rarefaction | evenness | barracuda::stats::evenness | S64 |
| rare_biosphere | chao1 | barracuda::stats::diversity::chao1_classic | S70 |
| rare_biosphere | detection_power | barracuda::stats::evolution::detection_power | S70 |
| rare_biosphere | detection_threshold | barracuda::stats::evolution::detection_threshold | S70 |
| drift | kimura_fixation_prob | barracuda::stats::evolution::kimura_fixation_prob | S70 |
| quasispecies | error_threshold | barracuda::stats::evolution::error_threshold | S70 |
| fao56 | daily_et0 | barracuda::stats::hydrology::fao56_et0 | S70 |
| jackknife | jackknife_mean_variance | barracuda::stats::jackknife::jackknife_mean_variance | S70 |
| anderson | analytical_localization_length | barracuda::special::anderson_transport::localization_length | S56 |
| wdm | green_kubo_integrate | barracuda::numerical::trapz | S56 |
| bistable | bistable_derivative | BistableOde::cpu_derivative | S58 |
| multisignal | multisignal_derivative | MultiSignalOde::cpu_derivative | S58 |

### GPU Delegations (19) — `#[cfg(feature = "barracuda-gpu")]`

| Module | Function | barracuda GPU Op | Shader |
|--------|----------|-----------------|--------|
| stats/agreement | rmse_gpu | FusedMapReduceF64 | fused_map_reduce_f64.wgsl |
| stats/agreement | mbe_gpu | SumReduceF64 | fused_map_reduce_f64.wgsl |
| stats/metrics | mean_gpu | SumReduceF64 | fused_map_reduce_f64.wgsl |
| stats/metrics | std_dev_gpu | VarianceReduceF64 | fused_map_reduce_f64.wgsl |
| stats/correlation | pearson_r_gpu | CorrelationF64 | correlation_f64.wgsl |
| fao56 | daily_et0_batch | BatchedElementwiseF64 | batched_elementwise_f64.wgsl |
| rare_biosphere | abundance_occupancy | BatchedMultinomialGpu | batched_multinomial_f64.wgsl |
| rare_biosphere | tier_detection_rate | BatchedMultinomialGpu | batched_multinomial_f64.wgsl |
| drift | wright_fisher_fixation_batch | WrightFisherGpu | wright_fisher_f64.wgsl |
| gillespie | birth_death_ssa_batch | GillespieGpu | gillespie_ssa_f64.wgsl |
| anderson | lyapunov_exponent | barracuda::spectral | anderson_coupling_f64.wgsl |
| anderson | lyapunov_averaged | barracuda::spectral | anderson_coupling_f64.wgsl |
| almost_mathieu | level_spacing_ratio | barracuda::spectral | batch_ipr_f64.wgsl |
| almost_mathieu | hamiltonian | barracuda::spectral | - |
| almost_mathieu | eigenvalues | barracuda::spectral | batched_eigh_*.wgsl |
| band_structure | detect_band_ranges | barracuda::spectral::detect_bands | - |
| spectral_recon | tikhonov_solve | barracuda::linalg::solve_f64_cpu | - |
| seismic | grid_search_inversion | barracuda::ops::grid::grid_search_3d | grid_search_3d_f64.wgsl |
| freeze_out | grid_fit_2d | barracuda::ops::grid::grid_search_3d | grid_search_3d_f64.wgsl |

### Evolution Candidate (1)

| Module | Function | barracuda Op | Issue |
|--------|----------|-------------|-------|
| band_structure | find_band_edges | ops::grid::band_edges_parallel | Algorithm mismatch: transfer matrix half-trace sign-change scan vs eigenvalue min/max extraction |

**What's needed**: A custom WGSL shader implementing the transfer matrix
product and half-trace computation per energy point, then sign-change
detection for band edge identification. The existing `band_edges_parallel`
uses a different algorithm (eigenvalue extraction) that doesn't match
groundSpring's physics.

---

## Part 2: What groundSpring Learned (Relevant to barracuda)

### 2.1 API Adaptation Patterns

When barracuda's API doesn't exactly match groundSpring's domain needs,
we use these patterns:

1. **Option adapter**: barracuda returns `Option<T>`, groundSpring needs
   `Result<T, InputError>`. Wrap with `if let Some(r) = barracuda_fn() { Ok(r) } else { fallback() }`.

2. **Unit conversion**: barracuda's `fao56_et0` expects `Rs` (solar radiation)
   and `u2` (wind at 2m), but groundSpring provides `sunshine_hours` and
   `wind_10m_km_h`. Pre-compute the conversions before calling barracuda.

3. **Pre-evaluate + GPU argmin**: For grid search problems where the forward
   model is domain-specific (haversine, freeze-out polynomial), pre-evaluate
   the objective function on CPU into a value grid, then use `grid_search_3d`
   for the GPU-accelerated minimum search.

4. **Degenerate dimension**: `grid_search_3d` can handle 2D problems by
   passing `z_grid = vec![0.0]` with the values repeated for z=0 only.

### 2.2 Clippy Pedantic Compatibility

groundSpring compiles with `clippy::pedantic` and `clippy::nursery`. Some
barracuda patterns trigger these lints:

- `needless_return` in `#[cfg]` blocks: use expression-position pattern
  instead of `return` statements.
- `missing_const_for_fn`: functions that are `const` without features but
  runtime with features need `#[allow(clippy::missing_const_for_fn)]`.
- `similar_names`: domain variables like `t0`/`k2` need `#[expect]` or
  inline the usage.

### 2.3 Performance Findings

| Workload | Python | Rust CPU | Speedup | GPU Potential |
|----------|--------|---------|---------|--------------|
| Seismic grid (31×31×7) | 7.4s | 0.15s | 51.2× | High (parallel grid eval) |
| Signal specificity (Gillespie) | 26.7s | 0.85s | 31.3× | Very high (batch SSA) |
| Multi-signal QS (ODE) | 4.3s | 0.14s | 30.0× | High (batch ODE) |
| Anderson (1000 sites) | 21.9s | 0.74s | 29.7× | Already GPU-wired |
| Precision drift (WDM) | 26.9s | 3.12s | 8.6× | High (batch Green-Kubo) |

The biggest GPU opportunities are the stochastic batch workloads
(Gillespie, Wright-Fisher) and the embarrassingly parallel grid searches.

### 2.4 f64 GPU Compatibility

On GPUs without native f64 support (RTX 4070), the `enable f64` WGSL
directive causes shader compilation failure in wgpu/naga. This triggers
wgpu's uncaptured error handler which panics in test mode.

**Recommendation**: barracuda should catch shader compilation failures
gracefully and return `Err(...)` instead of relying on the wgpu uncaptured
error handler. This would allow clean fallback without test panics.

### 2.5 Cross-Spring Shader Evolution

groundSpring's 57 delegations span functions that originated in 5 different
Springs. Tracking this provenance (already done via `provenance.rs` tags)
is valuable for debugging and evolution planning.

| Origin Spring | groundSpring Functions Using | Key Shaders |
|--------------|---------------------------|-------------|
| hotSpring | anderson, almost_mathieu, spectral_recon, GPU stats | df64_core, fused_map_reduce, batch_ipr |
| wetSpring | rarefaction, rare_biosphere, gillespie | batched_multinomial, bray_curtis, gillespie_ssa |
| neuralSpring | drift (WrightFisherGpu), band_structure | wright_fisher, batch_fitness |
| airSpring | fao56, regression, moving_window | batched_elementwise, linear_regression |
| groundSpring | kimura, jackknife, chao1, detection, grid search | grid_search_3d, mc_et0_propagate |

---

## Part 3: Benchmark Data for barracuda

### 3.1 CPU vs GPU Benchmark (12 workloads)

```
Workload                                        CPU (ms) Batch/GPU (ms)
Gillespie SSA (100 trajectories)                    4.91           4.82
Wright-Fisher fixation (100 trials)                19.05          19.53
FAO-56 ET₀ (500 station-days)                       0.08           0.07
FAO-56 scalar ET₀ (1 station-day)                   0.00              -
Kimura fixation (15 configs)                        0.00              -
Jackknife mean/var (500 points)                     0.00              -
Chao1 richness (200 taxa)                           0.00              -
Seismic inversion (31×31×7 grid)                    1.10              -
Freeze-out grid fit (61×41 grid)                    0.12              -
Rare biosphere (200sp × 100 samples)               13.33              -
Anderson Lyapunov (1000 sites)                      0.02              -
Neutral diversity (20sp × 500 gens)                 2.71              -
```

Note: This was measured with barracuda CPU only (no GPU hardware available
for f64 dispatch). GPU numbers pending Titan V/A100 validation.

### 3.2 Three-Tier Parity Test Coverage

95 tests covering all 57 delegations. Each test verifies:
- Default (no barracuda) produces result R₁
- barracuda CPU produces result R₂
- |R₁ - R₂| < ε (typically 1e-12 for deterministic, seed-matched for stochastic)

---

## Part 4: Recommended barracuda Evolutions

### 4.1 High Priority (groundSpring blocking)

1. **Graceful shader compilation failure**: Return `Err(...)` instead of
   triggering wgpu uncaptured error when `enable f64` is not supported.
   This enables clean CPU fallback on consumer GPUs.

2. **Transfer matrix shader**: Custom WGSL for transfer matrix half-trace
   computation per energy point. Enables `band_edges` delegation (currently
   the only evolution candidate).

### 4.2 Medium Priority (performance)

3. **Jackknife leave-one-out GPU kernel**: N delete-one-out subsets are
   embarrassingly parallel. For N > 10K, GPU dispatch would be significant.

4. **Rarefaction batch GPU**: `rarefaction_at_depth` runs multinomial
   replicates in a loop. `BatchedMultinomialGpu` could handle the batch.

5. **Neutral diversity trajectory GPU**: Multi-species Wright-Fisher is
   embarrassingly parallel across species and generations.

### 4.3 Low Priority (future)

6. **FFT (real, complex)**: Not in barracuda. Would enable spectral
   reconstruction via frequency domain.

7. **Eigenvector solver (tridiag)**: Eigenvalues use Sturm bisection;
   eigenvectors still CPU-only (inverse iteration).

---

## Handoff Checklist

- [x] Complete delegation inventory (57 active + 1 candidate)
- [x] API adaptation patterns documented
- [x] Performance data provided (Rust vs Python + CPU vs GPU)
- [x] f64 GPU compatibility issue documented
- [x] Cross-spring shader lineage mapped
- [x] Recommended barracuda evolutions prioritized
- [x] All quality gates green (fmt/clippy/doc/test)
