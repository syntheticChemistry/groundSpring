<!-- SPDX-License-Identifier: AGPL-3.0-only -->
<!-- Copyright (C) 2026 ecoPrimals / Squirrel Team -->

# groundSpring → ToadStool V64: Deep Audit + BarraCuda Evolution

**Date**: March 2, 2026
**groundSpring Version**: V64
**ToadStool Pin**: S79 (`f97fc2ae`)
**Supersedes**: V63 (brain architecture + capability-based discovery)
**Tests**: 752 workspace (409 lib + 343 integration/validation) + 1 doc-test
**Clippy**: Clean (zero warnings, `clippy::pedantic`)
**Docs**: Clean (`cargo doc --no-deps`)
**Format**: Clean (`cargo fmt --all -- --check`)
**License**: AGPL-3.0-only (unified — all .rs, .py, .sh, .md)

---

## Executive Summary

- Deep codebase audit: zero `unsafe`, zero TODO/FIXME, zero `.unwrap()` in production library code, zero mocks in production, all files under 1000 lines
- Evolved `validate/lib.rs` to `Result`-based API (`BenchResult<T>`) while maintaining backward-compatible panicking wrappers
- Refactored `tissue_anderson` into directory module with `drug_scoring` submodule (916 → 641 + 268 lines)
- Fixed 4 validation binary exit codes (now all `std::process::exit(h.summary())`)
- Unified SPDX license identifiers to `AGPL-3.0-only` across all 74+ non-Rust files
- Evolved all `partial_cmp().unwrap_or()` to `f64::total_cmp()`, all `#[allow]` to `#[expect]` with reasons
- Absorbed shader references updated everywhere — `batched_multinomial.wgsl` (S76) and `mc_et0_propagate.wgsl` (S72) confirmed gone
- New: Exp 033 (Tissue Anderson) — 29/29 validation checks, cytokine Anderson lattice + geometry-aware drug scoring (Paper 12)
- Total: 376/376 validation checks across 33 experiments, 752 workspace tests

---

## Part 1: Code Quality Audit Results

### Zero Debt

| Category | Count | Notes |
|----------|-------|-------|
| `unsafe` blocks | **0** | Workspace-level `forbid(unsafe_code)` |
| TODO/FIXME/HACK markers | **0** | Clean codebase |
| `.unwrap()` in production lib | **0** | All in `#[cfg(test)]` only |
| Mocks in production | **0** | Only doc reference: "Zero Mocks" in `npu.rs` |
| Files > 1000 lines | **0** | Largest: `biomeos.rs` at 834 |
| `#[allow]` without reason | **0** | All converted to `#[expect(lint, reason = "...")]` |
| `partial_cmp().unwrap_or()` | **0** | All evolved to `f64::total_cmp()` |
| Panicking validation exits | **0** | All 33 binaries use `std::process::exit(h.summary())` |

### Idiomatic Rust Evolutions Applied

1. **`f64::total_cmp()`** — replaced all `partial_cmp().unwrap_or(Ordering::Equal)` patterns in `esn.rs`, `spectral_recon.rs`, `validate_quasiperiodic.rs`
2. **`#[expect]` with reasons** — all lint suppressions now use `#[expect(lint, reason = "...")]` instead of bare `#[allow]`, providing self-documenting intent
3. **`Result`-based API** — `validate/lib.rs` gains `get_f64`, `get_usize`, `get_str`, `get_f64_vec`, `get_f64_range` returning `BenchResult<T>`, with legacy `*_field` wrappers using `.expect()` for backward compatibility
4. **Smart module refactoring** — `tissue_anderson.rs` split into `tissue_anderson/mod.rs` (core physics, 641 lines) + `tissue_anderson/drug_scoring.rs` (drug repurposing, 268 lines) with `xi_from_gamma()` deduplication
5. **IPv6-safe defaults** — `127.0.0.1` → `localhost` in `validate_nestgate_ncbi.rs`

---

## Part 2: BarraCuda Integration Inventory (67 Delegations)

### CPU Delegations (37 functions via `#[cfg(feature = "barracuda")]`)

| Domain | Module | Functions | Barracuda API |
|--------|--------|-----------|---------------|
| Stats | `stats::distributions` | norm_cdf, norm_ppf, chi2_statistic | `barracuda::stats::norm_*`, `chi2_decomposed` |
| Stats | `stats::regression` | fit_linear, fit_quadratic, fit_exponential, fit_logarithmic | `barracuda::stats::regression::*` |
| Stats | `stats::agreement` | rmse, mae, mbe, nse, r², ia, hit_rate | `barracuda::stats::*` |
| Stats | `stats::correlation` | pearson_r, spearman_r, covariance | `barracuda::stats::*_correlation`, `covariance` |
| Stats | `stats::metrics` | mean, std_dev, percentile | `barracuda::stats::mean`, `std_dev`, `percentile` |
| Stats | `stats::moving_window` | moving_window_stats | `barracuda::stats::moving_window_stats_f64` |
| Bootstrap | `bootstrap` | bootstrap_mean, rawr_mean, bootstrap_median, bootstrap_std | `barracuda::stats::bootstrap_*` |
| Ecology | `rarefaction` | simpson, bray_curtis, rarefaction_curve, shannon, pielou_evenness | `barracuda::stats::*` |
| Ecology | `rare_biosphere` | chao1_classic, detection_power, detection_threshold | `barracuda::stats::diversity::*`, `evolution::*` |
| Evolution | `drift` | kimura_fixation_prob | `barracuda::stats::evolution::kimura_fixation_prob` |
| Evolution | `quasispecies` | error_threshold | `barracuda::stats::evolution::error_threshold` |
| Kinetics | `kinetics` | hill, monod | `barracuda::stats::hill`, `monod` |
| Hydrology | `fao56` | daily_et0, hargreaves_et0, crop_coefficient, soil_water_balance | `barracuda::stats::hydrology::*` |
| Physics | `freeze_out` | chi2_analysis | `barracuda::stats::chi2::chi2_decomposed_weighted` |
| Physics | `anderson` | analytical_localization_length | `barracuda::special::anderson_transport::localization_length` |
| ODE | `bistable`, `multisignal` | cpu_derivative | `barracuda::numerical::ode_bio::*` |
| Integration | `wdm` | green_kubo_integrate | `barracuda::numerical::trapz` |
| Jackknife | `jackknife` | jackknife_mean_variance | `barracuda::stats::jackknife::*` |

### GPU Delegations (26 functions via `#[cfg(feature = "barracuda-gpu")]`)

| Domain | Module | Functions | GPU API |
|--------|--------|-----------|---------|
| Spectral | `anderson` | lyapunov_exponent, lyapunov_averaged, disorder_sweep, anderson_2d, anderson_3d | `barracuda::spectral::*` |
| Spectral | `almost_mathieu` | level_spacing_ratio, hamiltonian, eigenvalues | `barracuda::spectral::*` |
| Spectral | `band_structure` | refine_band_edge, detect_band_ranges | `barracuda::optimize::brent`, `spectral::detect_bands` |
| Spectral | `lanczos` | eigenvalues, eigenvalues_from_csr | `barracuda::spectral::lanczos*` |
| Linalg | `spectral_recon` | tikhonov_solve | `barracuda::linalg::cholesky_f64`, `solve_f64_cpu` |
| Linalg | `linalg` | tridiag_eigh_barracuda | `barracuda::linalg::eigh_f64` |
| Grid | `freeze_out` | grid_fit_2d | `barracuda::ops::grid::grid_search_3d` |
| Grid | `seismic` | grid_search_inversion | `barracuda::ops::grid::grid_search_3d` |
| Bio | `drift` | wright_fisher_fixation_batch | `barracuda::ops::bio::WrightFisherGpu` |
| Bio | `gillespie` | birth_death_ssa_batch | `barracuda::ops::bio::GillespieGpu` |
| Bio | `rarefaction` | multinomial_sample_batch | `barracuda::ops::bio::BatchedMultinomialGpu` |
| Bio | `rare_biosphere` | abundance_occupancy, tier_detection_rate | `barracuda::ops::bio::BatchedMultinomialGpu` |
| Hydrology | `fao56` | daily_et0_batch, hargreaves_et0_batch | `BatchedElementwiseF64`, `HargreavesBatchGpu` |
| Stats | `jackknife` | jackknife_mean | `JackknifeMeanGpu` |
| Stats | `stats::agreement` | rmse, mbe | `FusedMapReduceF64`, `SumReduceF64` |
| Stats | `stats::metrics` | mean, std_dev | `SumReduceF64`, `VarianceReduceF64` |
| Stats | `stats::correlation` | pearson_r | `CorrelationF64` |
| ML | `esn` | EsnClassifier | `barracuda::esn_v2::ESN` |

### Cross-Spring Delegations (4)

Anderson spectral (hotSpring), diversity (wetSpring), regression (airSpring), ESN reservoir (wetSpring → hotSpring).

---

## Part 3: BarraCuda APIs Not Yet Consumed

These exist in ToadStool/BarraCuda but groundSpring does not yet use them:

| Barracuda API | Location | Potential groundSpring Use |
|---------------|----------|---------------------------|
| `FusedMapReduceF64::shannon_entropy` | `ops::fused_map_reduce_f64` | GPU path for `rarefaction::shannon` — currently CPU-delegated only |
| `FusedMapReduceF64::simpson_index` | `ops::fused_map_reduce_f64` | GPU path for `rarefaction::simpson` — currently CPU-delegated only |
| `VarianceReduceF64::population_variance` | `ops::variance_reduce_f64` | GPU variance computation for uncertainty budgets |
| `PeakDetectF64` | `ops` (S62) | Local maxima with prominence — concept edge detection (currently CPU LOO) |
| `BandwidthTier` | `dispatch` (S62) | PCIe/NvLink bandwidth routing for metalForge pipeline |
| `anderson_3d_correlated` | `spectral` (S59) | Correlated disorder for tissue Anderson models |
| `find_w_c` | `spectral` (S59) | Critical disorder interpolation — tissue barrier analysis |
| `ridge_regression` | `linalg` (S59) | Tikhonov regression for ESN readout (currently barracuda-internal) |
| `NmfResult` | `linalg::nmf` | Non-negative matrix factorization for metagenomics |
| `CapacitorOde`, `CooperationOde` | `numerical::ode_bio` | Additional bio ODE systems |
| `bootstrap_mean_f64.wgsl` | `stats` | GPU bootstrap mean shader (65 lines) |
| `ops::PrngXoshiro` | `ops` | PRNG alignment: groundSpring still uses `Xorshift64` |

### Priority Absorption Recommendations

1. **PRNG alignment** (Tier B) — `Xorshift64` → `xoshiro128**`. Requires baseline regeneration across all 33 experiments. Multi-session effort. This is the only remaining architectural divergence.
2. **Shannon/Simpson GPU** — `FusedMapReduceF64` for large-sample diversity. Low effort, direct speedup for Exp 004/016/023/030.
3. **`anderson_3d_correlated`** — enables correlated disorder in tissue Anderson (Paper 12 §2.3).
4. **`find_w_c`** — critical disorder interpolation would improve tissue barrier analysis.
5. **`PeakDetectF64`** — GPU-accelerated concept edge detection for large disorder sweeps.

---

## Part 4: Evolution Recommendations for ToadStool

### What groundSpring Learned That Benefits ToadStool

1. **`Result`-based validation API** — The `BenchResult<T>` pattern provides clean error propagation for benchmark field access. ToadStool's test harnesses could benefit from similar typed errors.

2. **Exit code discipline** — Every validation binary must call `std::process::exit(h.summary())`. CI depends on this. Consider a `#[must_use]` lint on harness summary functions.

3. **`f64::total_cmp()` everywhere** — Eliminates the entire class of `partial_cmp().unwrap_or()` anti-patterns. Audit ToadStool for remaining instances.

4. **`#[expect]` over `#[allow]`** — Self-documenting lint suppression catches stale suppressions at compile time. Audit for bare `#[allow]` in ToadStool.

5. **Smart module refactoring** — The `tissue_anderson` split (physics + drug_scoring) demonstrates that file splits should follow domain boundaries, not arbitrary line counts. Shared helper extraction (`xi_from_gamma`) consolidates patterns across the split.

6. **wgpu version gap** — groundSpring pins wgpu 22; latest is 28.x. Cross-project upgrade tracked by ToadStool.

### Tissue Anderson Module for Absorption

The `tissue_anderson` module (Exp 033, 29/29 checks) introduces:

- `SkinLayer` enum (Epidermis/Dermis) with layer-specific parameters
- `CellType` enum with weighted energy and abundance
- `TissueCompartment` — generates Anderson potentials from cell-type composition
- Barrier disruption sweep (d_eff transition tracking)
- `DrugCandidate` scoring with penetration × Anderson × pathway factors
- `DeliveryRoute` (Topical/Oral/Injectable/Subcutaneous) with route-dependent kinetics

**Absorption candidate**: `tissue_anderson::compute_anderson_factor` delegates to `anderson::lyapunov_exponent` which already delegates to `barracuda::spectral`. The drug scoring logic is combinatorial (CPU-only), but disorder sweep and spectral analysis benefit from existing GPU delegation.

---

## Part 5: Dependency Health

| Dependency | Version | Status |
|------------|---------|--------|
| barracuda | path (S79) | Current — path dep via `../../../phase1/toadstool/crates/barracuda` |
| wgpu | 22 | **Outdated** (28.x latest) — cross-project upgrade |
| serde_json | 1.0.149 | Current |
| tokio | 1 | Current |
| tarpc | 0.35 | Current |
| proptest | 1 (dev) | Current |
| tempfile | 3.26 (dev) | Current |

No `-sys` crates. No C/C++ linkage. All pure Rust.

---

## Part 6: Shader Status

| Shader | Location | Status |
|--------|----------|--------|
| `anderson_lyapunov.wgsl` | `metalForge/shaders/` | Reference (f64) — kept for Titan V validation |
| `anderson_lyapunov_f32.wgsl` | `metalForge/shaders/` | Reference (f32 fallback) — kept for cross-precision testing |
| `batched_multinomial.wgsl` | *(removed V62)* | **Absorbed S76** into ToadStool |
| `mc_et0_propagate.wgsl` | *(removed V62)* | **Absorbed S72** into ToadStool |

---

## Validation

```
cargo fmt --all -- --check         ✅ Clean
cargo clippy --all-targets         ✅ Clean (pedantic, zero warnings)
  --all-features -- -W clippy::pedantic
cargo doc --no-deps                ✅ Clean
cargo test --workspace             ✅ 752 passed, 0 failed
```

---

## Cross-Spring Lineage

```
groundSpring V64 (deep audit)
  ├── V63 brain architecture (DriftAction, ConceptEdge, MultiHeadUncertainty)
  ├── V62 S79 catch-up (pollster→test_pool, shader cleanup)
  ├── V61 mixed-hardware pipeline (PCIe topology, NUCLEUS atomics)
  ├── V60 hotSpring absorption (Nautilus, 15-head ESN)
  ├── ToadStool S79 (f97fc2ae) — 844 WGSL shaders, 14,200+ tests
  └── Exp 033 tissue Anderson (Paper 12 — Gonzales immunopharmacology)
      ├── anderson::lyapunov_exponent → barracuda::spectral (GPU)
      ├── SkinLayer/CellType/TissueCompartment (new domain types)
      └── DrugCandidate/DeliveryRoute/DrugScore (geometry-aware scoring)
```
