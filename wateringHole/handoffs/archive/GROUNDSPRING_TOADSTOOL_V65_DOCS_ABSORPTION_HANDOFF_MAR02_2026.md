<!-- SPDX-License-Identifier: AGPL-3.0-only -->
<!-- Copyright (C) 2026 ecoPrimals / Squirrel Team -->

# groundSpring → ToadStool V65: Comprehensive Absorption Handoff

**Date**: March 2, 2026
**From**: groundSpring (V65)
**To**: ToadStool / BarraCUDA team
**groundSpring Version**: V65 (docs sweep + absorption handoff + paper queue review)
**ToadStool Pin**: S79 (`f97fc2ae`)
**Supersedes**: V64 (deep audit — idiomatic Rust evolution, biomeos refactoring)
**Tests**: 752 workspace (409 lib + 343 integration/validation) + 1 doc-test
**Clippy**: Clean (zero warnings, `clippy::pedantic` + `clippy::nursery`)
**Docs**: Clean (`cargo doc --no-deps`)
**Format**: Clean (`cargo fmt --all -- --check`)
**License**: AGPL-3.0-only (unified — all .rs, .py, .sh, .md)

---

## Executive Summary

- groundSpring V64 deep audit certified zero-debt: zero `unsafe`, zero TODO/FIXME, zero `.unwrap()` in production lib, zero mocks in production, zero `#[allow]` without reason, all files under 1000 lines, all epsilon guards documented
- **67 active delegations** (37 CPU + 26 GPU + 4 cross-spring) — comprehensive inventory below with absorption priorities for ToadStool
- Paper queue: **33 experiments across 28 papers**, all using open data, validated at CPU tier (376/376), GPU tier partially (73/73 `validate-gpu-tier`), metalForge partially (172 checks)
- Three-tier hardware matrix: CPU → barracuda CPU → barracuda GPU → metalForge mixed-hardware — this handoff maps every paper to its hardware evolution path
- **PRNG alignment** remains the single architectural divergence — this handoff details the migration plan and what ToadStool needs to provide
- Tissue Anderson module (Paper 12, Exp 033) demonstrates the Write → Absorb → Lean cycle working end-to-end: domain code delegates to `barracuda::spectral`, GPU delegation inherited automatically

---

## Part 1: What groundSpring Has Learned for ToadStool

### 1.1 Idiomatic Rust Patterns Worth Adopting Upstream

| Pattern | Where Applied | Recommendation for ToadStool |
|---------|--------------|------------------------------|
| `f64::total_cmp()` | Replaced all `partial_cmp().unwrap_or(Ordering::Equal)` | Audit ToadStool for remaining `partial_cmp` chains |
| `#[expect(lint, reason = "...")]` | All lint suppressions self-documenting | Replace bare `#[allow]` throughout barracuda — catches stale suppressions at compile time |
| `BenchResult<T>` result-based API | `validate/lib.rs` typed errors | ToadStool test harnesses could use similar typed benchmark field errors |
| `std::process::exit(h.summary())` | All 33+13 validation binaries | Consider `#[must_use]` lint on `ValidationHarness::summary()` |
| Smart module refactoring | `biomeos/` (834→4 files), `tissue_anderson/` (916→641+268) | Domain boundaries > arbitrary line splits |
| `s.to_owned()` over `s.clone()` for `&str → String` | `biomeos/protocol.rs` | Minor but idiomatic — `to_owned()` signals intent on borrowed→owned |

### 1.2 Tolerance Philosophy

groundSpring's tolerances are centrally defined in `crates/groundspring-validate/src/lib.rs`:

| Constant | Value | Justification |
|----------|-------|---------------|
| `TOL_EXACT` | `1e-10` | Summation-only paths (no transcendentals) |
| `TOL_LITERATURE` | `1e-6` | Published precision of reference values |
| `TOL_DECOMPOSITION` | `1e-4` | MC variance inherent in decomposition |

Test-level tolerances follow a principle: **exact for summation, approximate for transcendentals/multi-pass GPU**. The `1e-10` / `1e-6` split maps directly to CPU vs GPU parity expectations.

### 1.3 Epsilon Guards

Every production epsilon guard is now documented with rationale:

| Module | Guard | Purpose |
|--------|-------|---------|
| `drift.rs` | `1e-10` | Prevents ÷0 when population has near-zero fitness |
| `gillespie.rs` | `1e-15` | ~10× f64 epsilon — prevents ÷0 in steady-state mean |
| `validate_vendor_parity.rs` | `1e-20` | Below physically meaningful diffusion coefficients |

**ToadStool action**: Consider adding similar guard documentation requirements to barracuda's internal epsilon constants.

---

## Part 2: Complete Delegation Inventory (67 Active)

### 2.1 CPU Delegations (37 functions via `#[cfg(feature = "barracuda")]`)

| Domain | Module | Functions | Barracuda API |
|--------|--------|-----------|---------------|
| Stats | `stats::distributions` | `norm_cdf`, `norm_ppf`, `chi2_statistic` | `barracuda::stats::norm_*`, `chi2_decomposed` |
| Stats | `stats::regression` | `fit_linear`, `fit_quadratic`, `fit_exponential`, `fit_logarithmic` | `barracuda::stats::regression::*` |
| Stats | `stats::agreement` | `rmse`, `mae`, `mbe`, `nse`, `r²`, `ia`, `hit_rate` | `barracuda::stats::*` |
| Stats | `stats::correlation` | `pearson_r`, `spearman_r`, `covariance` | `barracuda::stats::*_correlation`, `covariance` |
| Stats | `stats::metrics` | `mean`, `std_dev`, `percentile` | `barracuda::stats::mean`, `std_dev`, `percentile` |
| Stats | `stats::moving_window` | `moving_window_stats` | `barracuda::stats::moving_window_stats_f64` |
| Bootstrap | `bootstrap` | `bootstrap_mean`, `rawr_mean`, `bootstrap_median`, `bootstrap_std` | `barracuda::stats::bootstrap_*` |
| Ecology | `rarefaction` | `simpson`, `bray_curtis`, `rarefaction_curve`, `shannon`, `pielou_evenness` | `barracuda::stats::diversity::*` |
| Ecology | `rare_biosphere` | `chao1_classic`, `detection_power`, `detection_threshold` | `barracuda::stats::diversity::*`, `evolution::*` |
| Evolution | `drift` | `kimura_fixation_prob` | `barracuda::stats::evolution::kimura_fixation_prob` |
| Evolution | `quasispecies` | `error_threshold` | `barracuda::stats::evolution::error_threshold` |
| Kinetics | `kinetics` | `hill`, `monod` | `barracuda::stats::hill`, `monod` |
| Hydrology | `fao56` | `daily_et0`, `hargreaves_et0`, `crop_coefficient`, `soil_water_balance` | `barracuda::stats::hydrology::*` |
| Physics | `freeze_out` | `chi2_analysis` | `barracuda::stats::chi2::chi2_decomposed_weighted` |
| Physics | `anderson` | `analytical_localization_length` | `barracuda::special::anderson_transport::localization_length` |
| ODE | `bistable`, `multisignal` | `cpu_derivative` | `barracuda::numerical::ode_bio::*` |
| Integration | `wdm` | `green_kubo_integrate` | `barracuda::numerical::trapz` |
| Jackknife | `jackknife` | `jackknife_mean_variance` | `barracuda::stats::jackknife::*` |

### 2.2 GPU Delegations (26 functions via `#[cfg(feature = "barracuda-gpu")]`)

| Domain | Module | Functions | GPU API |
|--------|--------|-----------|---------|
| Spectral | `anderson` | `lyapunov_exponent`, `lyapunov_averaged`, `disorder_sweep`, `anderson_2d`, `anderson_3d` | `barracuda::spectral::*` |
| Spectral | `almost_mathieu` | `level_spacing_ratio`, `hamiltonian`, `eigenvalues` | `barracuda::spectral::*` |
| Spectral | `band_structure` | `refine_band_edge`, `detect_band_ranges` | `barracuda::optimize::brent`, `spectral::detect_bands` |
| Spectral | `lanczos` | `eigenvalues`, `eigenvalues_from_csr` | `barracuda::spectral::lanczos*` |
| Linalg | `spectral_recon` | `tikhonov_solve` | `barracuda::linalg::cholesky_f64`, `solve_f64_cpu` |
| Linalg | `linalg` | `tridiag_eigh_barracuda` | `barracuda::linalg::eigh_f64` |
| Grid | `freeze_out` | `grid_fit_2d` | `barracuda::ops::grid::grid_search_3d` |
| Grid | `seismic` | `grid_search_inversion` | `barracuda::ops::grid::grid_search_3d` |
| Bio | `drift` | `wright_fisher_fixation_batch` | `barracuda::ops::bio::WrightFisherGpu` |
| Bio | `gillespie` | `birth_death_ssa_batch` | `barracuda::ops::bio::GillespieGpu` |
| Bio | `rarefaction` | `multinomial_sample_batch` | `barracuda::ops::bio::BatchedMultinomialGpu` |
| Bio | `rare_biosphere` | `abundance_occupancy`, `tier_detection_rate` | `barracuda::ops::bio::BatchedMultinomialGpu` |
| Hydrology | `fao56` | `daily_et0_batch`, `hargreaves_et0_batch` | `BatchedElementwiseF64`, `HargreavesBatchGpu` |
| Stats | `jackknife` | `jackknife_mean` | `JackknifeMeanGpu` |
| Stats | `stats::agreement` | `rmse`, `mbe` | `FusedMapReduceF64`, `SumReduceF64` |
| Stats | `stats::metrics` | `mean`, `std_dev` | `SumReduceF64`, `VarianceReduceF64` |
| Stats | `stats::correlation` | `pearson_r` | `CorrelationF64` |
| ML | `esn` | `EsnClassifier` | `barracuda::esn_v2::ESN` |

### 2.3 Cross-Spring Delegations (4)

| Delegation | Origin Spring | Barracuda Path |
|-----------|---------------|----------------|
| Anderson spectral (GPU) | hotSpring S26 | `barracuda::spectral::lyapunov_*` |
| Diversity (CPU) | wetSpring | `barracuda::stats::diversity::*` |
| Regression (CPU) | airSpring | `barracuda::stats::regression::*` |
| ESN reservoir (GPU) | wetSpring → hotSpring | `barracuda::esn_v2::ESN` |

---

## Part 3: Barracuda APIs Not Yet Consumed (Absorption Opportunities)

| Barracuda API | Location | groundSpring Use | Priority |
|---------------|----------|------------------|----------|
| `FusedMapReduceF64::shannon_entropy` | `ops::fused_map_reduce_f64` | GPU path for `rarefaction::shannon` (currently CPU-delegated only) | **HIGH** |
| `FusedMapReduceF64::simpson_index` | `ops::fused_map_reduce_f64` | GPU path for `rarefaction::simpson` | MEDIUM |
| `VarianceReduceF64::population_variance` | `ops::variance_reduce_f64` | GPU variance for uncertainty budgets | MEDIUM |
| `PeakDetectF64` | `ops` (S62) | Local maxima with prominence — concept edge detection | LOW |
| `anderson_3d_correlated` | `spectral` (S59) | Correlated disorder for tissue Anderson (Paper 12 §2.3) | **HIGH** |
| `find_w_c` | `spectral` (S59) | Critical disorder interpolation for tissue barrier analysis | **HIGH** |
| `ridge_regression` | `linalg` (S59) | Tikhonov regression for ESN readout (currently barracuda-internal) | LOW |
| `NmfResult` | `linalg::nmf` | Non-negative matrix factorization for R. Anderson metagenomics | MEDIUM |
| `CapacitorOde`, `CooperationOde` | `numerical::ode_bio` | Additional bio ODE systems for Waters paper extensions | LOW |
| `bootstrap_mean_f64.wgsl` | `stats` | GPU bootstrap mean shader (65 lines) | MEDIUM |
| `ops::PrngXoshiro` | `ops` | PRNG alignment (see Part 5) | **CRITICAL** |

### Priority Absorption Sequence

1. **PRNG alignment** — `Xorshift64` → `xoshiro128**` (Part 5)
2. **`anderson_3d_correlated`** + **`find_w_c`** — enables correlated disorder in tissue Anderson (Paper 12 §2.3) and critical barrier interpolation
3. **Shannon/Simpson GPU** — `FusedMapReduceF64` for large-sample diversity (Exp 004/016/023/030)
4. **`PeakDetectF64`** — GPU-accelerated concept edge detection for large disorder sweeps

---

## Part 4: Paper Queue × Three-Tier Hardware Matrix

### 4.1 Complete Experiment Status (33 experiments, 28 papers)

| # | Experiment | Paper | CPU (Rust) | Barracuda CPU | Barracuda GPU | metalForge | Open Data |
|---|-----------|-------|:----------:|:-------------:|:-------------:|:----------:|:---------:|
| 001 | Sensor Noise | Dong 2020 | **36/36** | 3 stats | Tier A pending | After GPU | PASS |
| 002 | Observation Gap | ERA5 | **13/13** | 3 stats | Tier A pending | After GPU | PASS |
| 003 | Error Propagation | FAO-56 | **15/15** | fao56 absorbed | Tier C (`BatchedElementwiseF64`) | After GPU | PASS |
| 004 | Sequencing Noise | synthetic | **15/15** | — | Tier C (`BatchedMultinomialGpu`) | After GPU | PASS |
| 005 | Seismic Inversion | synthetic | **9/9** | — | Tier B (grid dispatch) | After GPU | PASS |
| 006 | Signal Specificity | Massie 2012 | **12/12** | — | `GillespieGpu` (ready) | After GPU | PASS |
| 007 | RAWR Resampling | Wang 2021 | **11/11** | `bootstrap_mean` | Embarrassingly parallel | After GPU | PASS |
| 008 | Anderson Localization | Bourgain-K 2018 | **8/8** | 2 CPU | `spectral::*` GPU (ready) | After GPU | PASS |
| 009 | Quasiperiodic | Jitomirskaya-K 2018 | **8/8** | eigenvalues | `spectral::*` GPU (ready) | After GPU | PASS |
| 010 | Bistable Switching | Fernandez 2020 | **10/10** | `BistableOde` | `BatchedEighGpu` (ready) | After GPU | PASS |
| 011 | Multi-Signal QS | Srivastava 2011 | **9/9** | `MultiSignalOde` | `CooperationOde` (ready) | After GPU | PASS |
| 012 | Spin Chain Transport | Kachkovskiy 2016 | **18/18** | — | tridiag_eigh candidate | After GPU | PASS |
| 013 | Resampling Convergence | Lee & Liu 2024 | **8/8** | `bootstrap` | Embarrassingly parallel | After GPU | PASS |
| 014 | Drift vs Selection | R. Anderson 2022 | **7/7** | kimura_fixation | WrightFisher GPU | After GPU | PASS |
| 015 | Uncertainty Bridge | cross-domain | **8/8** | analytical ξ | spectral GPU | After GPU | PASS |
| 016 | Rare Biosphere | R. Anderson 2015 | **12/12** | chao1, diversity | `BatchedMultinomialGpu` | After GPU | PASS |
| 017 | Quasispecies | Dolson 2023 | **6/6** | error_threshold | WF sim GPU | After GPU | PASS |
| 018 | Band Edge Structure | Filonov-K 2018 | **10/10** | detect_bands | Brent GPU | After GPU | PASS |
| 019 | Jackknife | Bazavov 2025 | **9/9** | jackknife | `JackknifeMeanGpu` | After GPU | PASS |
| 020 | Freeze-Out Inverse | Bazavov 2016 | **8/8** | chi2_analysis | grid dispatch | After GPU | PASS |
| 021 | Spectral Recon | Bazavov 2025 | **8/8** | tikhonov_solve | Cholesky GPU | After GPU | PASS |
| 022 | ET₀ → Anderson | cross-spring | **7/7** | fao56 + anderson | spectral GPU | After GPU | PASS |
| 023 | No-Till Sampling | cross-spring | **7/7** | diversity | multinomial GPU | After GPU | PASS |
| 024 | Aggregate Stability | cross-spring | **8/8** | stats | spectral GPU | After GPU | PASS |
| 025 | f32/f64 Drift | WDM | **7/7** | trapz | — | After GPU | PASS |
| 026 | Size Convergence | WDM | **7/7** | fit_linear | — | After GPU | PASS |
| 027 | Vendor Parity | WDM | **7/7** | — | — | After GPU | PASS |
| 028 | NPU Anderson | hardware | **9/9** | — | — | **Live** (AKD1000 DMA) | PASS |
| 029 | Real GHCND ET₀ | NOAA | **6/6** | biomeos | — | — | PASS |
| 030 | Real NCBI 16S | NCBI | **9/9** | biomeos | — | — | PASS |
| 031 | NUCLEUS Stack | infra | **28/28** | biomeos | — | — | PASS |
| 032 | IRIS Seismic | IRIS FDSN | **12/12** | biomeos | — | — | PASS |
| 033 | Tissue Anderson | Paper 12 | **29/29** | anderson spectral | lyapunov GPU | After GPU | PASS |

**CPU tier**: 376/376 PASS across 33 validation binaries.
**Barracuda CPU tier**: 37 delegations consumed, 95 three-tier parity tests.
**Barracuda GPU tier**: 26 delegations, 73/73 `validate-gpu-tier` checks.
**metalForge tier**: 172 checks (130 forge + 42 mixed-hardware).
**Open data**: 33/33 experiments use open data or open systems.

### 4.2 Papers Needing GPU Tier Completion

| Priority | Papers | Blocker | ToadStool Action |
|----------|--------|---------|-----------------|
| **HIGH** | 1-5 (stats Tier A) | GPU adapter for `FusedMapReduceF64` | Wire GPU feature path for reduce ops |
| **HIGH** | 4, 20-21 | `BatchedMultinomialGpu` signature alignment | Align cumulative_probs + RNG interface |
| **MEDIUM** | 5, 8 | Grid search GPU dispatch | `grid_search_3d_f64` op |
| **MEDIUM** | 12, 18 | Tridiag eigenvector solver | `tridiag_eigh_eigenvectors` (currently eigenvalues only) |
| **LOW** | 25-27 | WDM-specific integration | Minimal — uses delegated primitives |

### 4.3 Papers Ready for metalForge

Once GPU tier completes for a paper, metalForge dispatch is straightforward
via `groundspring-forge` (existing crate with PCIe topology, fallback chains,
NUCLEUS atomics). Currently:

- **Live**: Exp 028 (NPU Anderson on AKD1000)
- **Ready**: Any paper with both CPU and GPU paths validated
- **Validated infrastructure**: 19 workloads across 5 substrates (17 GPU + 2 NPU)

---

## Part 5: PRNG Alignment — The Last Architectural Divergence

### Current State

| Property | groundSpring `Xorshift64` | BarraCUDA `PrngXoshiro` |
|----------|--------------------------|------------------------|
| State size | 64 bits | 256 bits |
| Period | 2⁶⁴ − 1 | 2²⁵⁶ − 1 |
| Quality | Fails BigCrush | Passes BigCrush |
| GPU kernel | None | `prng_xoshiro_wgsl` |

### What groundSpring Has Done

- `Xoshiro128StarStar` wrapper implemented with full API parity (V28)
- `GpuAlignedRng` type alias defined (V63)
- `DefaultRng` alias ready to switch when barracuda feature activates
- 10 PRNG tests validate xoshiro compatibility

### What ToadStool Needs to Provide

1. **Scalar `PrngXoshiro` API** — groundSpring needs `fn next_u64(&mut self) -> u64` and `fn next_f64(&mut self) -> f64` on a CPU-side xoshiro128** that produces the same stream as the GPU kernel
2. **Seed initialization** — SplitMix64 from u64 seed, matching GPU kernel behavior

### Migration Steps (Multi-Session)

1. Feature-gate the PRNG — `#[cfg(feature = "barracuda")]` delegates to barracuda xoshiro
2. Regenerate all 28 Python baselines with Python xoshiro128** port
3. Update all benchmark JSONs (new expected values, new `baseline_commit`, `prng_algorithm: "xoshiro128**"`)
4. Verify 376/376 checks pass
5. Archive xorshift64 baselines to `control/archive/xorshift64/`

**ToadStool action**: Export `PrngXoshiro` as a public CPU-side struct with the scalar API above. This unblocks groundSpring's Tier B GPU promotion for all stochastic experiments.

---

## Part 6: Tissue Anderson — Write → Absorb → Lean in Action

Exp 033 (Paper 12) demonstrates the full cycle:

```
Write: tissue_anderson module with domain types (SkinLayer, CellType, etc.)
       → generates Anderson potentials from cell-type composition
       → calls anderson::lyapunov_exponent for spectral analysis

Lean:  anderson::lyapunov_exponent already delegates to barracuda::spectral
       → tissue Anderson gets GPU acceleration for free
       → drug scoring remains CPU-only (combinatorial, not compute-bound)
```

### Absorption Candidates from Paper 12

| Module | Function | Barracuda Potential |
|--------|----------|---------------------|
| `tissue_anderson` | `compute_anderson_factor` | Inherits `barracuda::spectral` delegation — no new kernel needed |
| `tissue_anderson` | `barrier_disruption_sweep` | 11-point sweep — embarrassingly parallel, but small N |
| `tissue_anderson::drug_scoring` | `score_drug` | Combinatorial — CPU-only, no GPU benefit |

**ToadStool action**: No new kernel needed for Paper 12. The existing `barracuda::spectral::lyapunov_*` provides the GPU path. Future expansion to `anderson_3d_correlated` (S59) would enable correlated tissue disorder models.

---

## Part 7: Dependency Health

| Dependency | Version | Status | ToadStool Impact |
|------------|---------|--------|-----------------|
| barracuda | path (S79) | Current | `../../../phase1/toadstool/crates/barracuda` |
| wgpu | 22 | **Outdated** (28.x latest) | Cross-project upgrade tracked by ToadStool |
| serde_json | 1.0.149 | Current | — |
| tokio | 1 | Current | — |
| tarpc | 0.35 | Current | — |
| proptest | 1 (dev) | Current | — |
| tempfile | 3.26 (dev) | Current | — |

No `-sys` crates. No C/C++ linkage. All pure Rust. `unsafe_code = "forbid"` workspace-wide.

---

## Part 8: Shader Status

| Shader | Location | Status |
|--------|----------|--------|
| `anderson_lyapunov.wgsl` | `metalForge/shaders/` | Reference (f64) — kept for Titan V validation |
| `anderson_lyapunov_f32.wgsl` | `metalForge/shaders/` | Reference (f32 fallback) — kept for cross-precision testing |
| `batched_multinomial.wgsl` | *(removed V62)* | **Absorbed S76** into ToadStool |
| `mc_et0_propagate.wgsl` | *(removed V62)* | **Absorbed S72** into ToadStool |

---

## Part 9: Summary of ToadStool Action Items

| # | Action | Priority | Effort |
|---|--------|----------|--------|
| 1 | Export `PrngXoshiro` as public CPU-side struct with `next_u64`/`next_f64` | **CRITICAL** | Small — expose existing internal |
| 2 | Wire GPU adapter for `FusedMapReduceF64` reduce ops (stats Tier A) | **HIGH** | Medium — adapter layer |
| 3 | Align `BatchedMultinomialGpu` signature for groundSpring consumption | **HIGH** | Medium — cumulative_probs interface |
| 4 | Add `grid_search_3d_f64` op for seismic/freeze-out GPU dispatch | **MEDIUM** | Medium — new embarrassingly parallel op |
| 5 | Add tridiag eigenvector extraction (currently eigenvalues only) | **MEDIUM** | Medium — extends existing Sturm |
| 6 | Expose `anderson_3d_correlated` for tissue Anderson correlated disorder | **MEDIUM** | Small — already in S59, needs public API |
| 7 | Expose `find_w_c` for critical disorder interpolation | **MEDIUM** | Small — already in S59, needs public API |
| 8 | Audit ToadStool for `partial_cmp().unwrap_or()` → `f64::total_cmp()` | **LOW** | Small — search and replace |
| 9 | Audit ToadStool for bare `#[allow]` → `#[expect(lint, reason)]` | **LOW** | Medium — systematic |
| 10 | Cross-project wgpu 22 → 28.x upgrade coordination | **LOW** | Large — breaking changes |

---

## Validation

```
cargo fmt --all -- --check         PASS
cargo clippy --all-targets         PASS (pedantic + nursery, zero warnings)
  --all-features -- -W clippy::pedantic
  -W clippy::nursery
cargo doc --no-deps                PASS
cargo test --workspace             PASS (752 passed, 0 failed)
```

---

## Cross-Spring Lineage

```
groundSpring V65 (docs sweep + absorption handoff)
  ├── V64 deep audit (biomeos refactor, #[expect], epsilon guards, tolerance docs)
  ├── V63 brain architecture (tissue Anderson, 6 GPU delegations, 67 total)
  ├── V62 S79 catch-up (pollster→test_pool, shader cleanup, Result API)
  ├── V61 mixed-hardware pipeline (PCIe topology, NUCLEUS atomics)
  ├── V60 hotSpring absorption (Nautilus, 15-head ESN)
  ├── ToadStool S79 (f97fc2ae) — 844 WGSL shaders, 14,200+ tests
  └── 33 experiments across 28 papers, all open data, 376/376 PASS
```
