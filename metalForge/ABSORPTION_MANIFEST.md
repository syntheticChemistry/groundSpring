# groundSpring Absorption Manifest

> Inventory of code for the Write → Absorb → Lean cycle with ToadStool/BarraCUDA.
>
> Following the hotSpring pattern: write locally, validate against CPU
> baselines, hand off via `wateringHole/handoffs/`, ToadStool absorbs as
> GPU ops, groundSpring rewires to upstream and deletes local code.

**Last updated**: March 7, 2026 (V95 — 102 active delegations (61 CPU + 41 GPU), barraCuda `0bd401f`, toadStool S129, coralReef Phase 11, 907 tests, clippy pedantic clean. V95: coralReef push buffer breakthrough — sovereign GPU dispatch on Titan V. V87: coralReef sovereign compilation. V84: dual-GPU probe. V83: pin refresh. V82: BootstrapMeanGpu dispatch)

## Absorption Status Summary

| Domain | Status | Count | Notes |
|---|---|---|---|
| Stats (CPU) | **Lean** | 30 | agreement, correlation, regression, distributions, diversity, hydrology (FAO-56, Hargreaves, Makkink, Turc, Hamon), bootstrap, jackknife, moving_window |
| Spectral (GPU) | **Lean** | 15 | Anderson 1D-4D, Lanczos, Almost-Mathieu, Wegner RG, band detect |
| Bio ops (GPU) | **Lean** | 5 | Gillespie, Wright-Fisher, BatchedMultinomial ×3 |
| Linalg (GPU) | **Lean** | 2 | eigh_f64, solve_f64_cpu + cholesky |
| Optimize | **Lean** | 2 | L-BFGS, batched Nelder-Mead GPU |
| Pipeline (GPU) | **Lean** | 5 | fao56 batch, Hargreaves GPU, McEt0, seasonal pipeline, water balance |
| ODE (CPU) | **Lean** | 2 | BistableOde, MultiSignalOde |
| Reduce ops (GPU) | **Lean** | 4 | sum_reduce, variance_reduce, fused_map_reduce, correlation_f64 |
| ESN (GPU) | **Lean** | 1 | esn_v2::ESN regime classification |
| Grid ops (GPU) | **Lean** | 1 | grid_search_3d (seismic) |
| **Total** | | **102** | 61 CPU + 41 GPU |
| PRNG | **Adapt** | 1 | xorshift64→xoshiro alignment pending |
| Scalar math | **Stays local** | 5 | decompose, haversine, travel_time |
| NPU | **Lean** | 1 | akida-driver (ToadStool hardware, not barraCuda math) |

---

## WGSL Shader Inventory

| Shader | Lines | Status | Notes |
|---|---|---|---|
| `anderson_lyapunov.wgsl` | ~80 | **Reference** — f64 Lyapunov exponent | Delegated to `barracuda::spectral::lyapunov_*`; kept as validation reference and provenance artifact |
| `anderson_lyapunov_f32.wgsl` | ~80 | **Reference** — f32 variant | Superseded by barraCuda universal precision (DF64 fallback on consumer GPUs); kept as provenance artifact |

**Absorbed into barraCuda (removed V62)**:
- `batched_multinomial.wgsl` → `barracuda::ops::bio::BatchedMultinomialGpu` (S76)
- `mc_et0_propagate.wgsl` → `barracuda::stats::hydrology::gpu::McEt0PropagateGpu` (S72)

---

## Tier A — Lean (93 active: 56 CPU + 37 GPU)

Full delegation inventory as of V83, barraCuda v0.3.3:

### CPU delegations (51)

| Function | BarraCUDA target | Wiring |
|---|---|---|
| `pearson_r` | `stats::pearson_correlation` | `#[cfg(feature = "barracuda")]` NaN-safe |
| `spearman_r` | `stats::correlation::spearman_correlation` | `#[cfg(feature = "barracuda")]` NaN-safe |
| `sample_std_dev` | `stats::correlation::std_dev` | `#[cfg(feature = "barracuda")]` |
| `covariance` | `stats::correlation::covariance` | `#[cfg(feature = "barracuda")]` if-let |
| `norm_cdf` | `stats::norm_cdf` | `#[cfg(feature = "barracuda")]` direct |
| `norm_ppf` | `stats::norm_ppf` | `#[cfg(feature = "barracuda")]` direct |
| `chi2_statistic` | `stats::chi2_decomposed` | `#[cfg(feature = "barracuda")]` struct mapping |
| `chi2_analysis` | `stats::chi2::chi2_decomposed_weighted` | `#[cfg(feature = "barracuda")]` per-datum residuals |
| `bootstrap_mean` | `stats::bootstrap_mean` | `#[cfg(feature = "barracuda")]` Result mapping |
| `bootstrap_median` | `stats::bootstrap_median` | `#[cfg(feature = "barracuda")]` S64 |
| `bootstrap_std` | `stats::bootstrap_std` | `#[cfg(feature = "barracuda")]` S64 |
| `rawr_mean` | `stats::rawr_mean` | `#[cfg(feature = "barracuda")]` S66 Dirichlet-weighted |
| `analytical_localization_length` | `special::anderson_transport::localization_length` | `#[cfg(feature = "barracuda")]` |
| `bistable_derivative` | `numerical::ode_bio::BistableOde::cpu_derivative` | `#[cfg(feature = "barracuda")]` OdeSystem trait |
| `multisignal_derivative` | `numerical::ode_bio::MultiSignalOde::cpu_derivative` | `#[cfg(feature = "barracuda")]` OdeSystem trait |
| `rmse` | `stats::rmse` | `#[cfg(feature = "barracuda")]` direct S64 |
| `mae` | `stats::mae` | `#[cfg(feature = "barracuda")]` direct S66 |
| `mbe` | `stats::mbe` | `#[cfg(feature = "barracuda")]` direct S64 |
| `nash_sutcliffe` | `stats::nash_sutcliffe` | `#[cfg(feature = "barracuda")]` S64 |
| `r_squared` | `stats::r_squared` | `#[cfg(feature = "barracuda")]` direct S64 |
| `index_of_agreement` | `stats::index_of_agreement` | `#[cfg(feature = "barracuda")]` direct S64 |
| `hit_rate` | `stats::hit_rate` | `#[cfg(feature = "barracuda")]` direct S64 |
| `shannon_diversity` | `stats::diversity::shannon` | `#[cfg(feature = "barracuda")]` u64→f64 S64 |
| `simpson_diversity` | `stats::diversity::simpson` | `#[cfg(feature = "barracuda")]` S64 |
| `bray_curtis` | `stats::diversity::bray_curtis` | `#[cfg(feature = "barracuda")]` S64 |
| `analytical_rarefaction` | `stats::diversity::rarefaction_curve` | `#[cfg(feature = "barracuda")]` hypergeometric S64 |
| `evenness` | `stats::pielou_evenness` | `#[cfg(feature = "barracuda")]` u64→f64 S≤1 adapter |
| `chao1_classic` | `stats::diversity::chao1_classic` | `#[cfg(feature = "barracuda")]` |
| `detection_power` | `stats::evolution::detection_power` | `#[cfg(feature = "barracuda")]` |
| `detection_threshold` | `stats::evolution::detection_threshold` | `#[cfg(feature = "barracuda")]` |
| `kimura_fixation_prob` | `stats::evolution::kimura_fixation_prob` | `#[cfg(feature = "barracuda")]` S70+ |
| `error_threshold` | `stats::evolution::error_threshold` | `#[cfg(feature = "barracuda")]` quasispecies |
| `mean` | `stats::mean` | `#[cfg(feature = "barracuda")]` direct S64 |
| `percentile` | `stats::percentile` | `#[cfg(feature = "barracuda")]` direct S64 |
| `hill` | `stats::hill` | `#[cfg(feature = "barracuda")]` domain guard S68 |
| `monod` | `stats::monod` | `#[cfg(feature = "barracuda")]` S66 |
| `fit_linear` | `stats::regression::fit_linear` | `#[cfg(feature = "barracuda")]` S66 OLS |
| `fit_quadratic` | `stats::regression::fit_quadratic` | `#[cfg(feature = "barracuda")]` S66 Cramer |
| `fit_exponential` | `stats::regression::fit_exponential` | `#[cfg(feature = "barracuda")]` S66 log-linearized |
| `fit_logarithmic` | `stats::regression::fit_logarithmic` | `#[cfg(feature = "barracuda")]` S66 ln-linearized |
| `fao56_et0` | `stats::hydrology::fao56_et0` | `#[cfg(feature = "barracuda")]` Penman-Monteith |
| `hargreaves_et0` | `stats::hydrology::hargreaves_et0` | `#[cfg(feature = "barracuda")]` |
| `crop_coefficient` | `stats::hydrology::crop_coefficient` | `#[cfg(feature = "barracuda")]` |
| `soil_water_balance` | `stats::hydrology::soil_water_balance` | `#[cfg(feature = "barracuda")]` |
| `jackknife_mean_variance` | `stats::jackknife::jackknife_mean_variance` | `#[cfg(feature = "barracuda")]` S70+ |
| `moving_window_stats` | `stats::moving_window_stats_f64` | `#[cfg(feature = "barracuda")]` S66 |
| `finite_size_extrapolate` | `stats::regression::fit_linear` | `#[cfg(feature = "barracuda")]` 1/N^(1/d) |

### GPU delegations (37)

| Function | BarraCUDA target | Wiring |
|---|---|---|
| `lyapunov_exponent` | `spectral::lyapunov_exponent` | `#[cfg(feature = "barracuda-gpu")]` transfer matrix |
| `lyapunov_averaged` | `spectral::lyapunov_averaged` | `#[cfg(feature = "barracuda-gpu")]` multi-realization |
| `almost_mathieu_hamiltonian` | `spectral::almost_mathieu_hamiltonian` | `#[cfg(feature = "barracuda-gpu")]` λ/2 coupling |
| `almost_mathieu_eigenvalues` | `spectral::find_all_eigenvalues` | `#[cfg(feature = "barracuda-gpu")]` Sturm tridiag **49.5×** |
| `level_spacing_ratio` | `spectral::level_spacing_ratio` | `#[cfg(feature = "barracuda-gpu")]` sort adapter |
| `disorder_sweep` | `spectral::anderson_sweep_averaged` | `#[cfg(feature = "barracuda-gpu")]` + CPU fallback |
| `anderson_2d_eigenvalues` | `spectral::anderson_2d` + `spectral::lanczos` | `#[cfg(feature = "barracuda-gpu")]` S59 |
| `anderson_3d_eigenvalues` | `spectral::anderson_3d` + `spectral::lanczos` | `#[cfg(feature = "barracuda-gpu")]` S59 W_c≈16.5 |
| `anderson_4d` | `spectral::anderson::anderson_4d` | `#[cfg(feature = "barracuda-gpu")]` S88 |
| `wegner_block_4d` | `spectral::anderson::wegner_block_4d` | `#[cfg(feature = "barracuda-gpu")]` S88 RG |
| `spectral_bandwidth` | `spectral::spectral_bandwidth` | `#[cfg(feature = "barracuda-gpu")]` |
| `spectral_condition_number` | `spectral::spectral_condition_number` | `#[cfg(feature = "barracuda-gpu")]` |
| `classify_spectral_phase` | `spectral::classify_spectral_phase` | `#[cfg(feature = "barracuda-gpu")]` |
| `detect_band_ranges` | `spectral::detect_bands` | `#[cfg(feature = "barracuda-gpu")]` hotSpring v0.6 |
| `lanczos_eigenvalues` | `spectral::lanczos` + `spectral::lanczos_eigenvalues` | `#[cfg(feature = "barracuda-gpu")]` S59 Lanczos |
| `tikhonov_solve` | `linalg::solve_f64_cpu` + `linalg::cholesky_f64` | `#[cfg(feature = "barracuda-gpu")]` Gauss-Jordan fallback |
| `tridiag_eigh` | `linalg::eigh_f64` | `#[cfg(feature = "barracuda-gpu")]` Jacobi validation |
| `abundance_occupancy` | `ops::bio::BatchedMultinomialGpu` | `#[cfg(feature = "barracuda-gpu")]` S64 |
| `tier_detection_rate` | `ops::bio::BatchedMultinomialGpu` | `#[cfg(feature = "barracuda-gpu")]` S64 |
| `multinomial_sample_batch` | `ops::bio::BatchedMultinomialGpu` | `#[cfg(feature = "barracuda-gpu")]` cumulative prob adapter |
| `birth_death_ssa_batch` | `ops::bio::GillespieGpu` | `#[cfg(feature = "barracuda-gpu")]` batch dispatch |
| `wright_fisher_fixation_batch` | `ops::bio::WrightFisherGpu` | `#[cfg(feature = "barracuda-gpu")]` device acquisition |
| `fao56_et0_batch` | `ops::batched_elementwise_f64::BatchedElementwiseF64` | `#[cfg(feature = "barracuda-gpu")]` |
| `hargreaves_et0_batch` | `stats::hydrology::HargreavesBatchGpu` | `#[cfg(feature = "barracuda-gpu")]` |
| `mc_et0_propagate` | `stats::hydrology::gpu::McEt0PropagateGpu` | `#[cfg(feature = "barracuda-gpu")]` seasonal |
| `seasonal_pipeline` | `stats::hydrology::gpu::SeasonalPipelineF64` | `#[cfg(feature = "barracuda-gpu")]` S88 |
| `jackknife_mean_gpu` | `ops::bio::JackknifeMeanGpu` | `#[cfg(feature = "barracuda-gpu")]` S71 |
| `correlation_f64` | `ops::correlation_f64_wgsl::CorrelationF64` | `#[cfg(feature = "barracuda-gpu")]` |
| `sum_reduce_mean` | `ops::sum_reduce_f64::SumReduceF64::mean` | `#[cfg(feature = "barracuda-gpu")]` |
| `variance_reduce_std` | `ops::variance_reduce_f64::VarianceReduceF64::population_std` | `#[cfg(feature = "barracuda-gpu")]` |
| `fused_map_reduce` | `ops::fused_map_reduce_f64::FusedMapReduceF64` | `#[cfg(feature = "barracuda-gpu")]` Shannon/Simpson |
| `esn_classifier` | `esn_v2::ESN` | `#[cfg(feature = "barracuda-gpu")]` S59 regime classification |
| `grid_search_3d` | `ops::grid::grid_search_3d` | `#[cfg(feature = "barracuda-gpu")]` seismic |
| `lbfgs_numerical` | `optimize::lbfgs_numerical` + `LbfgsConfig` | `#[cfg(feature = "barracuda")]` freeze-out refinement |

---

## Tier B — Adapt (1 remaining)

| Module | Blocker | Action |
|---|---|---|
| `prng::Xorshift64` | Different PRNG algorithm | Align to xoshiro128**; retain xorshift as CPU reference |

All other Tier B items have been resolved and moved to Tier A:
- `kimura_fixation_prob` — resolved S70+, now in `barracuda::stats::evolution`
- `jackknife_mean_variance` — resolved S70+, now in `barracuda::stats::jackknife`
- `fao56::daily_et0` — resolved S70+, now in `barracuda::stats::hydrology`
- `seismic::grid_search_inversion` — resolved via `barracuda::ops::grid::grid_search_3d`
- `gillespie::birth_death_ssa` — resolved via `barracuda::ops::bio::GillespieGpu` batch dispatch
- `rarefaction::multinomial_sample` — resolved via `barracuda::ops::bio::BatchedMultinomialGpu`
- `anderson::anderson_potential` — resolved via `barracuda::spectral::anderson_potential`

---

## Tier C — Fully Absorbed

Both Tier C shaders have been absorbed upstream and local copies removed (V62):
- `batched_multinomial.wgsl` → `barracuda::ops::bio::BatchedMultinomialGpu` (S76)
- `mc_et0_propagate.wgsl` → `barracuda::stats::hydrology::gpu::McEt0PropagateGpu` (S72)

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

- [x] 102 active delegations (61 CPU + 41 GPU) verified, barraCuda `0bd401f`, toadStool S129
- [x] CPU reference passes all validation checks (34 binaries, 907 tests)
- [x] All delegations use `#[cfg]` or `if let Ok` with CPU fallback always compiled
- [x] Mathematical parity: 28/28 PROVEN (Python ⇌ Rust, `data/parity_report.json`)
- [x] Three-mode revalidation (local / barracuda / barracuda-gpu): all PASS, 0 warnings
- [x] `cargo clippy -- -D warnings -W clippy::pedantic` PASS (default + barracuda modes)
- [x] 97.25% line coverage (`cargo llvm-cov`, target 90%)
- [x] 13-tier named tolerance architecture (`tol::`, `eps::`)
- [x] ~170 bare float literals → named constants
- [x] All Tier C shaders absorbed into barraCuda (batched_multinomial S76, mc_et0_propagate S72)
- [x] ToadStool absorption of groundSpring V68 confirmed (anderson_4d, wegner_block_4d, LbfgsGpu, tridiag_eigenvectors)
- [x] barraCuda budding complete: standalone primal at `ecoPrimals/barraCuda/`
- [ ] PRNG alignment (xorshift64 → xoshiro128**) — requires full rebaseline
