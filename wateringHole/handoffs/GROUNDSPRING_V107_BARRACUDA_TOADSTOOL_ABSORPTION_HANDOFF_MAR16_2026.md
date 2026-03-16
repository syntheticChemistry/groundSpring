# groundSpring V107 — barraCuda / toadStool Absorption Handoff

**Date**: March 16, 2026
**From**: groundSpring V107 (39 modules, 906 tests, 102 delegations)
**To**: barraCuda / toadStool / coralReef teams
**Authority**: wateringHole (ecoPrimals Core Standards)

## Pins

- **barraCuda**: v0.3.5 (path dep `../../../barraCuda/crates/barracuda`)
- **toadStool**: S155b
- **coralReef**: Iteration 49+
- **groundSpring**: V107 (`c4d5d65`)

## Executive Summary

groundSpring V107 consumes 80+ unique barraCuda operations across 8 categories
(stats, linalg, ops, spectral, numerical, optimize, special, ESN). 102 active
delegations (61 CPU + 41 GPU). 22+ modules have GPU dispatch paths.

This handoff documents:
1. Complete delegation inventory by category
2. Local math still duplicating barraCuda (absorption candidates)
3. GPU promotion opportunities
4. Cross-spring learnings for barraCuda/toadStool evolution

## Quality Gates (groundSpring V107)

| Gate | Status |
|------|--------|
| `cargo test --workspace` | **906 passed, 0 failed** |
| `cargo clippy --pedantic` | **0 warnings** |
| License | **AGPL-3.0-only** |
| Bare literals in production | **0** |
| Hardcoded primal strings | **0** |
| unsafe | **forbidden** |

## Part 1: Delegation Inventory (102 total: 61 CPU + 41 GPU)

### Stats — 34 operations

| Operation | groundSpring Module | Tier |
|-----------|---------------------|------|
| `stats::mean` | stats/metrics | CPU |
| `stats::jackknife::jackknife_mean_variance` | jackknife | CPU |
| `stats::jackknife::JackknifeMeanGpu` | jackknife | GPU |
| `stats::moving_window_stats_f64` | stats/moving_window | CPU |
| `stats::correlation::{std_dev, covariance, spearman_correlation}` | stats/correlation | CPU |
| `stats::pearson_correlation` | stats/correlation | CPU |
| `stats::percentile` | stats/metrics | CPU |
| `stats::norm_cdf` / `stats::norm_ppf` | stats/distributions | CPU |
| `stats::chi2_decomposed` / `chi2_decomposed_weighted` | stats/distributions, freeze_out/chi2 | CPU |
| `stats::rmse` / `stats::mae` / `stats::mbe` | stats/agreement | CPU |
| `stats::nash_sutcliffe` / `stats::r_squared` | stats/agreement | CPU |
| `stats::index_of_agreement` / `stats::hit_rate` | stats/agreement | CPU |
| `stats::bootstrap_mean` / `stats::bootstrap::BootstrapMeanGpu` | bootstrap | CPU+GPU |
| `stats::rawr_mean` | bootstrap | CPU |
| `stats::hill` / `stats::monod` | kinetics | CPU |
| `stats::diversity::chao1_classic` | rare_biosphere | CPU |
| `stats::evolution::{detection_power, detection_threshold, error_threshold}` | rare_biosphere, quasispecies | CPU |
| `stats::spectral_density::{marchenko_pastur_bounds, empirical_spectral_density}` | anderson/spectral | CPU |
| `stats::regression::{fit_linear, fit_quadratic, fit_exponential, fit_logarithmic, fit_all}` | stats/regression, wdm | CPU |
| `stats::hydrology::fao56_et0` | fao56 | CPU |

### Linalg — 4 operations

| Operation | groundSpring Module | Tier |
|-----------|---------------------|------|
| `linalg::solve_f64_cpu` | spectral_recon | CPU |
| `linalg::cholesky_f64` | spectral_recon | GPU |
| `linalg::eigh_f64` | linalg | GPU |
| `linalg::ridge_regression` | esn (readout) | CPU |

### Ops — 14 operations

| Operation | groundSpring Module | Tier |
|-----------|---------------------|------|
| `ops::peak_detect_f64::PeakDetectF64` | anderson/spectral | GPU |
| `ops::fft::Fft1DF64` | spectral_recon | GPU |
| `ops::grid::grid_search_3d` | freeze_out/grid, seismic | GPU |
| `ops::bio::{BatchedMultinomialGpu, BatchedMultinomialConfig}` | rare_biosphere, rarefaction | GPU |
| `ops::sum_reduce_f64::SumReduceF64::mean` | stats/metrics, stats/agreement | GPU |
| `ops::variance_reduce_f64::VarianceReduceF64` | stats/metrics | GPU |
| `ops::variance_f64_wgsl::VarianceF64` | stats/metrics | GPU |
| `ops::correlation_f64_wgsl::CorrelationF64` | stats/correlation | GPU |
| `ops::covariance_f64_wgsl::CovarianceF64` | stats/correlation | GPU |
| `ops::fused_map_reduce_f64::FusedMapReduceF64` | stats/agreement | GPU |
| `ops::batched_ode_rk4::{BatchedOdeRK4F64, BatchedRk4Config}` | bistable | GPU |
| `ops::batched_elementwise_f64::BatchedElementwiseF64` | fao56 | GPU |

### Spectral — 16 operations

| Operation | groundSpring Module | Tier |
|-----------|---------------------|------|
| `spectral::{spectral_bandwidth, spectral_condition_number, classify_spectral_phase}` | anderson/spectral | CPU |
| `spectral::anderson_potential` | anderson | CPU |
| `spectral::{lyapunov_exponent, lyapunov_averaged}` | anderson | GPU |
| `spectral::anderson_sweep_averaged` | anderson, tissue_anderson | GPU |
| `spectral::find_w_c` | tissue_anderson | GPU |
| `spectral::{anderson_2d, anderson_3d}` | anderson | GPU |
| `spectral::anderson_3d_correlated` | tissue_anderson | GPU |
| `spectral::anderson::{anderson_4d, wegner_block_4d}` | tissue_anderson | GPU |
| `spectral::level_spacing_ratio` | almost_mathieu | CPU |
| `spectral::{almost_mathieu_hamiltonian, find_all_eigenvalues}` | almost_mathieu | GPU |
| `spectral::detect_bands` | band_structure | GPU |

### Numerical + Optimize + Special — 7 operations

| Operation | groundSpring Module | Tier |
|-----------|---------------------|------|
| `numerical::OdeSystem` (trait) | bistable, multisignal | CPU |
| `numerical::ode_bio::{BistableOde, MultiSignalOde}` | bistable, multisignal | CPU |
| `optimize::lbfgs_numerical` | freeze_out/grid | CPU |
| `optimize::batched_nelder_mead_gpu` | freeze_out/nelder_mead | GPU |
| `optimize::brent` | band_structure | CPU |
| `special::anderson_transport::localization_length` | anderson | CPU |

### ESN — 2 operations

| Operation | groundSpring Module | Tier |
|-----------|---------------------|------|
| `esn_v2::ESN` | esn/classifier | GPU |
| `esn_v2::ESNConfig` | esn/classifier | GPU |

## Part 2: Local Math — Absorption Candidates

These are functions in groundSpring that duplicate or could delegate to barraCuda:

### P0 — Should absorb (duplicates existing barraCuda ops)

| Local Function | File | barraCuda Equivalent | Impact |
|----------------|------|----------------------|--------|
| `welford_population` | stats/metrics.rs | `stats::mean` + `VarianceReduceF64` | Used in 6+ call sites; main local CPU stats engine |
| `mat_transpose_mul` / `mat_transpose_vec` | spectral_recon.rs | `linalg::GemmF64` (batched) | Aᵀ·B and Aᵀ·v for Tikhonov inversion |
| `wdm::vendor_parity_mean_variance` | wdm.rs | `SumReduceF64::mean` + `VarianceReduceF64` | Green-Kubo analysis |

### P1 — Could promote to GPU

| Local Function | File | Promotion Path |
|----------------|------|----------------|
| `bootstrap_median` / `bootstrap_std` | bootstrap.rs | Extend `BootstrapMeanGpu` pattern |
| `moving_window_stats_cpu` | stats/moving_window.rs | GPU sliding-window reduce |
| `regression::fit_*` (all 5) | stats/regression.rs | Batched GPU regression |
| `freeze_out::chi2_decomposed_weighted` | freeze_out/chi2.rs | Per-datum parallel chi² |

### Stays Local (by design)

| Function | Reason |
|----------|--------|
| `decompose::bias_variance_decompose` | 2 scalar ops, no GPU benefit |
| `validate::*` | Test harness, not compute |
| `quasispecies::quasispecies_simulation` | Per-generation thinning, round-trip dominates |
| `band_structure` coarse scan | Data-dependent 2×2 chains, L=2-10, below GPU threshold |
| `fao56::equations::*` | Domain-specific formulas (FAO-56 Eq. 6–17), no barraCuda equivalent needed |
| `seismic::haversine_km` / `travel_time_1d` | Geometric primitives, 3 lines each |

## Part 3: GPU Tier Status

### 22 modules with GPU dispatch paths

anderson, anderson/spectral, almost_mathieu, band_structure, bistable,
bootstrap, drift, esn, fao56, freeze_out/grid, freeze_out/nelder_mead,
gillespie, jackknife, linalg, rare_biosphere, rarefaction/sampling,
seismic, spectral_recon, stats/agreement, stats/correlation, stats/metrics,
tissue_anderson, transport, wdm

### CPU-only modules (4 remaining, GPU promotion candidates)

| Module | Current | Promotion Path |
|--------|---------|----------------|
| stats/moving_window | CPU `moving_window_stats_cpu` | GPU sliding-window reduce op |
| stats/regression | CPU `fit_*` | Batched GPU regression (one dispatch for N fits) |
| freeze_out/chi2 | CPU `chi2_decomposed_weighted` | Parallel per-datum chi² |
| bootstrap (rawr/median/std) | CPU variants | Extend `BootstrapMeanGpu` pattern |

### Intentionally CPU-only (3 modules)

| Module | Reason |
|--------|--------|
| decompose | 2 scalar ops |
| quasispecies | Per-generation thinning |
| validate | Test harness |

## Part 4: Cross-Spring Learnings for barraCuda/toadStool

### 1. Tolerance provenance as a pattern

groundSpring V107 added mathematical derivation, source citations, and
validation binary references to all 13 tolerance constants. Pattern:

```rust
/// CDF/erf approximation (A&S 7.1.26, two-layer composition).
///
/// Provenance: Abramowitz & Stegun formula 7.1.26 has max error 1.5e-7;
/// our chi² CDF compounds erf twice, giving ~1e-6.
/// Source: Abramowitz & Stegun (1964), §7.1.26.
/// Validated: `validate_decompose`, `validate_freeze_out`.
pub const CDF_APPROX: f64 = 1e-6;
```

**Recommendation for barraCuda**: Adopt tolerance provenance in `barracuda::tol`
so springs can cite upstream precision bounds.

### 2. Enriched niche metadata

groundSpring V107 `niche.rs` now provides `const fn operation_dependencies()`
and `const fn cost_estimates()` with structured types:

```rust
pub struct CostEstimate {
    pub capability: &'static str,
    pub estimated_ms: u32,
    pub gpu_beneficial: bool,
    pub peak_memory_bytes: u64,
    pub deterministic: bool,
}
```

**Recommendation for toadStool**: If toadStool absorbs niche metadata for
scheduling, define a standard `NicheMetadata` struct that all springs provide.

### 3. Typed BiomeOsError with #[non_exhaustive]

groundSpring V106 evolved `BiomeOsError(String)` to a 7-variant enum with
`#[non_exhaustive]`. This enables match-based recovery (retry on Transport,
fail on Discovery).

**Recommendation for toadStool**: Define a standard `IpcError` enum that all
springs can reuse (Transport, Protocol, Serialization, etc.).

### 4. primal_names.rs pattern

Both wetSpring V119 and groundSpring V106 use centralized `primal_names.rs`
for all IPC identifiers. Misspelling a constant is a compile error; misspelling
a string literal is silent.

**Recommendation for barraCuda/toadStool**: If cross-primal name constants
are needed, consider publishing a shared `ecoprimals-names` crate.

### 5. Feature-gated dead code

groundSpring V107 fixed dead-code warnings by gating constants behind the
same `#[cfg(feature = "...")]` as their consumers. Pattern:

```rust
#[cfg(not(feature = "barracuda"))]
const BULK_PHASE_OUTLIER_THRESHOLD: f64 = 0.05;
```

**Recommendation**: Any const used only behind a feature gate should itself
be gated to avoid dead-code warnings in other configurations.

### 6. erfc stability (from hotSpring)

hotSpring Exp 046 documented erfc cancellation risk at large x.
groundSpring's chi² CDF uses `erfc(x) = 1 - erf(x)` which cancels at large x.

**Recommendation for barraCuda**: Implement direct asymptotic expansion for
`erfc` at large x (barraCuda ISSUE-006) so all springs benefit.

## Part 5: metalForge Status

- **30 workloads** (24 GPU + 2 NPU + 2 CPU-only + 2 mixed)
- **140 metalForge checks** (all PASS)
- **2 local WGSL shaders** (Anderson Lyapunov f64 + f32 in `metalForge/shaders/`)
- **Architecture**: PCIe topology, NUCLEUS atomics (Tower/Node/Nest/Full), GPU→NPU P2P bypass

### Shader absorption candidates

| Shader | Status | Notes |
|--------|--------|-------|
| `anderson_lyapunov.wgsl` (f64) | Active | Unique to groundSpring; Lyapunov exponent via transfer matrix |
| `anderson_lyapunov_f32.wgsl` | Active | f32 variant for precision comparison |

These are highly domain-specific (Anderson transfer matrix). Consider absorbing
into `barracuda::spectral::wgsl/` if other springs need Lyapunov computation.

## Delegation Map: groundSpring → barraCuda

```
groundSpring (39 modules, V107)
    │
    ├── stats (34 ops) ─── barracuda::stats::*
    │   ├── metrics ──── mean, percentile, variance
    │   ├── correlation ─ pearson, spearman, covariance
    │   ├── agreement ── rmse, mae, mbe, nse, r², ioa
    │   ├── regression ─ linear, quadratic, exp, log
    │   └── distributions ── chi², norm_cdf, norm_ppf
    │
    ├── linalg (4 ops) ─── barracuda::linalg::*
    │   ├── eigh_f64 (GPU Jacobi)
    │   ├── cholesky_f64 (GPU)
    │   ├── solve_f64_cpu
    │   └── ridge_regression
    │
    ├── ops (14 ops) ─── barracuda::ops::*
    │   ├── GPU reduce (sum, variance, correlation, covariance)
    │   ├── peak_detect_f64, fft, grid_search_3d
    │   ├── batched_multinomial, batched_ode_rk4
    │   └── batched_elementwise_f64 (FAO-56)
    │
    ├── spectral (16 ops) ─── barracuda::spectral::*
    │   ├── anderson_{2d, 3d, 4d, correlated, sweep}
    │   ├── lyapunov_{exponent, averaged}
    │   ├── almost_mathieu, detect_bands
    │   └── level_spacing_ratio, find_w_c
    │
    ├── numerical (3 ops) ─── barracuda::numerical::*
    │   └── OdeSystem trait, BistableOde, MultiSignalOde
    │
    ├── optimize (3 ops) ─── barracuda::optimize::*
    │   └── lbfgs, batched_nelder_mead_gpu, brent
    │
    ├── special (1 op) ─── barracuda::special::*
    │   └── localization_length
    │
    └── esn (2 ops) ─── barracuda::esn_v2::*
        └── ESN, ESNConfig (GPU reservoir)
```
