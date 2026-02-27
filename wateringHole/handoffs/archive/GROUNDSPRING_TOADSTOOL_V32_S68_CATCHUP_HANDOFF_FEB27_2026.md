# groundSpring → ToadStool Handoff V32: S68+ Catch-Up + Forward Declaration Cleanup

**Date**: February 27, 2026
**groundSpring Version**: V32
**ToadStool Pin**: S68+ (`e96576ee`, Feb 27 2026)
**Supersedes**: V31 (GPU Dispatch Wiring)

---

## Executive Summary

1. **9 forward declarations cleaned**: V29/V31 wired barracuda calls for functions that don't exist yet in ToadStool. These broke `--features barracuda` and `--features barracuda-gpu` compilation. All 9 are now commented out with `TODO(toadstool)` markers, restoring clean compilation on all feature combinations.

2. **29 active delegations** (23 CPU + 6 GPU) — all compile and test clean.

3. **9 pending delegations** (3 CPU + 6 GPU) — documented in code and in this handoff, ready to uncomment when ToadStool absorbs them.

4. **ToadStool S68+ universal precision architecture reviewed**: dual-layer DF64, op_preamble + naga IR rewrite, zero f32-only shaders, 700 WGSL shaders, hardware-adaptive precision routing.

5. **All tests pass**: 410/410 (default), 442/442 (biomeos), 320/320 Python, `--features barracuda` clean, `--features barracuda-gpu` clean, 0 clippy warnings.

---

## Part 1: Forward Declarations Cleaned

### V29 CPU delegations (3) — behind `#[cfg(feature = "barracuda")]`

These referenced functions that don't exist in barracuda S68+:

| groundSpring function | Expected barracuda function | File | Status |
|----------------------|---------------------------|------|--------|
| `drift::kimura_fixation_prob` | `barracuda::stats::kimura_fixation` | `drift.rs` | Commented out |
| `jackknife::jackknife_mean_variance` | `barracuda::stats::jackknife_mean_variance` | `jackknife.rs` | Commented out |
| `fao56::daily_et0` | `barracuda::stats::hydrology::fao56_et0` | `fao56.rs` | Commented out |

**Note on fao56**: ToadStool has `hargreaves_et0` and `BatchedElementwiseF64::fao56_et0_batch` but no standalone scalar `fao56_et0` in `stats::hydrology`. The full Penman-Monteith chain (15 sub-functions) needs a single-call wrapper.

### V31 GPU delegations (6) — behind `#[cfg(feature = "barracuda-gpu")]`

These referenced functions that don't exist in barracuda S68+:

| groundSpring function | Expected barracuda function | File | ToadStool status |
|----------------------|---------------------------|------|-----------------|
| `freeze_out::grid_fit_2d` | `barracuda::ops::grid::grid_fit_2d_f64` | `freeze_out.rs` | Not in `ops::grid` (has FD gradients only) |
| `seismic::grid_search_inversion` | `barracuda::ops::grid::grid_search_3d_f64` | `seismic.rs` | Not in `ops::grid` |
| `band_structure::find_band_edges` | `barracuda::spectral::band_edges_parallel` | `band_structure.rs` | Not in `spectral` |
| `quasispecies::quasispecies_simulation` | `barracuda::ops::bio::wright_fisher_simulate` | `quasispecies.rs` | Partial — `WrightFisherGpu::dispatch()` is per-generation step, not multi-generation |
| `rare_biosphere::abundance_occupancy` | `barracuda::ops::bio::batched_multinomial_occupancy` | `rare_biosphere.rs` | Partial — `BatchedMultinomialGpu` gives counts, not occupancy fractions |
| `rare_biosphere::tier_detection_rate` | `barracuda::ops::bio::batched_multinomial_tier_rate` | `rare_biosphere.rs` | Not implemented |

### Pattern for re-enabling

Each commented-out delegation follows:

```rust
// TODO(toadstool): uncomment when barracuda implements <function>
// #[cfg(feature = "barracuda-gpu")]
// {
//     if let Ok(result) = barracuda::<module>::<function>(...) {
//         return result;
//     }
// }
local_cpu_fallback(...)
```

When ToadStool implements a function, grep for `TODO(toadstool)` in the corresponding file and uncomment.

---

## Part 2: Active Delegations (29)

### CPU delegations (23) — `#[cfg(feature = "barracuda")]`

| # | groundSpring | barracuda | Module |
|---|-------------|-----------|--------|
| 1 | `pearson_r` | `stats::pearson_correlation` | correlation |
| 2 | `spearman_r` | `stats::correlation::spearman_correlation` | correlation |
| 3 | `covariance` | `stats::correlation::covariance` | correlation |
| 4 | `rmse` | `stats::rmse` | metrics |
| 5 | `mbe` | `stats::mbe` | metrics |
| 6 | `r_squared` | `stats::r_squared` | metrics |
| 7 | `index_of_agreement` | `stats::index_of_agreement` | metrics |
| 8 | `hit_rate` | `stats::hit_rate` | metrics |
| 9 | `mean` | `stats::mean` | metrics |
| 10 | `std_dev` | `stats::correlation::std_dev` | metrics |
| 11 | `percentile` | `stats::percentile` | metrics |
| 12 | `norm_cdf` | `stats::norm_cdf` | distributions |
| 13 | `norm_ppf` | `stats::norm_ppf` | distributions |
| 14 | `chi_squared` | `stats::chi2_decomposed` | distributions |
| 15 | `hill` | `stats::hill` | kinetics |
| 16 | `shannon` | `stats::shannon` | rarefaction |
| 17 | `pielou_evenness` | `stats::pielou_evenness` | rarefaction |
| 18 | `bootstrap_mean` | `stats::bootstrap_mean` | bootstrap |
| 19 | `rawr_mean` | `stats::rawr_mean` | bootstrap |
| 20 | `fit_linear` | `stats::regression::fit_linear` | regression |
| 21 | `localization_length` | `special::anderson_transport::localization_length` | anderson |
| 22 | `multisignal_derivative` | `numerical::ode_bio::MultiSignalOde::cpu_derivative` | multisignal |
| 23 | `bistable_derivative` | `numerical::ode_bio::BistableOde::cpu_derivative` | bistable |

### GPU delegations (6) — `#[cfg(feature = "barracuda-gpu")]`

| # | groundSpring | barracuda | Module |
|---|-------------|-----------|--------|
| 24 | `lyapunov_exponent` | `spectral::lyapunov_exponent` | anderson |
| 25 | `lyapunov_averaged` | `spectral::lyapunov_averaged` | anderson |
| 26 | `level_spacing_ratio` | `spectral::level_spacing_ratio` | almost_mathieu |
| 27 | `hamiltonian` | `spectral::almost_mathieu_hamiltonian` | almost_mathieu |
| 28 | `eigenvalues` | `spectral::find_all_eigenvalues` | almost_mathieu |
| 29 | `tikhonov_solve` | `linalg::solve_f64_cpu` | spectral_recon |

---

## Part 3: ToadStool S68+ Evolution Summary

### Key architectural changes since groundSpring's last ToadStool review

**Universal precision architecture (S67-S68)**:
- **Dual-layer DF64**: Layer 1 (`op_preamble`) provides abstract operations (`op_add`/`op_mul`/`op_pack`/`op_unpack`) for F16/F32/F64/DF64. Layer 2 (`df64_rewrite.rs`) uses naga-guided f64 infix rewrite with bridge functions.
- **Zero f32-only shaders**: 296 f32 WGSL files deleted, all f64 canonical with `LazyLock` downcast.
- **Hardware-adaptive precision**: `GpuDriverProfile::fp64_strategy()` routes compute GPUs (1:2 FP64:FP32) to native f64, consumer GPUs (1:64) to DF64 on FP32 cores.
- **DF64 performance**: ~9.9× throughput vs native f64 on consumer GPUs (RTX 3090, 4070).
- **F16 hardened**: `downcast_f64_to_f16()` with sentinel protection + literal clamping (±65504.0).

**Cross-spring absorption (S66)**:
- `stats::regression`, `stats::hydrology`, `stats::moving_window_f64`, `bootstrap::rawr_mean` absorbed from airSpring/groundSpring.
- `stats::diversity` (Shannon, Bray-Curtis, rarefaction) — all from groundSpring's rare biosphere work.

**Sovereign compiler (S61-63)**:
- `SovereignCompiler`: naga-IR optimizer with FMA fusion, dead expression elimination, SPIR-V passthrough.

**Current barracuda metrics**: 700 WGSL shaders, 2,546+ barracuda tests, 0 clippy warnings.

### What this means for groundSpring

1. **Precision is no longer a barrier**: ToadStool's universal precision architecture means any GPU delegation from groundSpring automatically gets hardware-appropriate precision (native f64 on compute GPUs, DF64 on consumer GPUs).

2. **All f64 math is canonical**: groundSpring doesn't need to worry about f32 precision loss in barracuda GPU shaders — everything is f64 canonical with automatic downcast.

3. **DF64 gives ~48-bit mantissa on consumer GPUs**: For groundSpring's scientific workloads (freeze-out fits, seismic inversion, band structure), DF64 on an RTX 4070 provides sufficient precision at 9.9× throughput.

---

## Part 4: ToadStool Action Items

### Priority 1: Implement pending CPU delegations (3)

These are pure math with no GPU dependency:

1. **`stats::kimura_fixation(pop_size, selection, initial_freq) -> Result<f64>`**
   - Kimura (1968) analytical fixation probability: `P = (1 - exp(-4Ns p₀)) / (1 - exp(-4Ns))`
   - CPU reference: `groundspring::drift::kimura_fixation_prob_cpu`

2. **`stats::jackknife_mean_variance(data: &[f64]) -> Result<(f64, f64)>`**
   - Delete-one jackknife: `var_JK = (N-1)/N * Σ(θ̂_i - θ̄_JK)²`
   - CPU reference: `groundspring::jackknife::jackknife_mean_variance_cpu`

3. **`stats::hydrology::fao56_et0(tmax, tmin, rhmax, rhmin, wind, sunshine, altitude, latitude, doy) -> Result<f64>`**
   - Full FAO-56 Penman-Monteith chain (Allen et al. 1998, Eq. 6)
   - CPU reference: `groundspring::fao56::daily_et0_cpu`

### Priority 2: Implement GPU wrappers (6)

These need higher-level wrappers around existing ToadStool primitives:

4. **`ops::bio::wright_fisher_simulate(pop_size, genome_length, sigma, mu, n_gens, seed) -> Result<Vec<f64>>`**
   - Wraps `WrightFisherGpu::dispatch()` in a multi-generation loop
   - Returns per-generation master sequence frequencies

5. **`ops::bio::batched_multinomial_occupancy(community, depth, n_samples, seed) -> Result<Vec<f64>>`**
   - Wraps `BatchedMultinomialGpu` to compute detection frequencies (presence/absence fractions)

6. **`ops::bio::batched_multinomial_tier_rate(community, tier_lo, tier_hi, depth, n_reps, seed) -> Result<f64>`**
   - Tier detection rate from batched multinomial samples

7. **`ops::grid::grid_fit_2d_f64(observed, mu_b, sigma, t0_lo, t0_hi, t0_step, k2_lo, k2_hi, k2_step) -> Result<(f64, f64, f64)>`**
   - 2D chi-squared grid search (embarrassingly parallel)

8. **`ops::grid::grid_search_3d_f64(sta_lats, sta_lons, obs_times, vp, lat_range, lon_range, depth_range, grid_deg, depth_km) -> Result<(f64, f64, f64, f64, f64)>`**
   - 3D seismic grid search (embarrassingly parallel)

9. **`spectral::band_edges_parallel(potential, hopping, e_lo, e_hi, n_points) -> Result<Vec<f64>>`**
   - Transfer matrix half-trace scan (embarrassingly parallel across energy points)

### Priority 3: Semantic alignment

- **chao1**: groundSpring uses integer equality (`count == 1`) for singleton classification; barracuda's `stats::chao1` uses float equality. Needs Tier B alignment before delegation.

---

## Part 5: Test Verification

```
cargo test --workspace                    → 410/410 PASS
cargo test --workspace --features biomeos → 442/442 PASS
cargo test --workspace --features barracuda → PASS (clean compilation)
cargo check --features barracuda-gpu      → PASS (clean compilation)
cargo clippy --workspace --all-features   → 0 warnings (1 unfulfilled lint expectation, pre-existing)
Python: 320/320 PASS + 2 skipped
```
