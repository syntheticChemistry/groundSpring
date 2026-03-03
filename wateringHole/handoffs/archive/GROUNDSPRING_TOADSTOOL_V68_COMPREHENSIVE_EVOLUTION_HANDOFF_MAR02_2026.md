# groundSpring → ToadStool/BarraCUDA: V68 Comprehensive Evolution Handoff

**Date**: March 2, 2026
**From**: groundSpring V68
**To**: ToadStool/BarraCUDA team
**ToadStool pin**: S87 (`2dc26792`)
**License**: AGPL-3.0-only

---

## Executive Summary

- **76 active delegations** (44 CPU + 32 GPU), 1 evolution candidate (band_edges algorithm mismatch)
- **30 metalForge workloads** (24 GPU + 2 NPU + 2 CPU-only), 30 tolerance specs
- **780 tests** passed, 0 failed; zero unsafe / zero TODO / zero `.unwrap()` / zero `#[allow]` without reason
- **33 experiments** validated (376/376 checks), 28/28 Python↔Rust parity proven
- **0 local WGSL shaders** — all production shaders absorbed upstream

---

## Part 1: What groundSpring Consumes

### barracuda Module Usage by Domain

| Domain | barracuda Module | Functions Used | CPU/GPU | Origin Spring |
|--------|-----------------|----------------|---------|---------------|
| **Statistics** | `stats::*` | mean, rmse, mbe, mae, nse, r², IoA, hit_rate, percentile, std_dev | CPU + GPU | airSpring + groundSpring → S64 |
| **Correlation** | `stats::correlation` | pearson, spearman, covariance | CPU + GPU | neuralSpring S54 |
| **Distributions** | `stats::*` | norm_cdf, norm_ppf, chi2_decomposed | CPU | ToadStool core |
| **Bootstrap** | `stats::*` | bootstrap_mean, bootstrap_median, bootstrap_std, rawr_mean | CPU | groundSpring V15 → S66 |
| **Jackknife** | `stats::jackknife` | jackknife_mean_variance, `JackknifeMeanGpu` | CPU + GPU | groundSpring → S66 |
| **Diversity** | `stats::*` | shannon, simpson, pielou_evenness, chao1, bray_curtis, rarefaction_curve | CPU + GPU | wetSpring → S64 |
| **Evolution** | `stats::evolution` | kimura_fixation_prob, error_threshold, detection_power, detection_threshold | CPU | wetSpring + groundSpring |
| **Kinetics** | `stats::*` | hill, monod | CPU | wetSpring S68 |
| **Regression** | `stats::regression` | fit_linear, fit_quadratic, fit_exponential, fit_logarithmic | CPU | airSpring → S66 |
| **Moving window** | `stats::*` | moving_window_stats_f64 | CPU | airSpring |
| **Hydrology** | `stats::hydrology` | fao56_et0, hargreaves_et0, hargreaves_et0_batch, crop_coefficient, soil_water_balance | CPU | airSpring → S70+ |
| **Hydrology GPU** | `stats::hydrology::gpu` | `McEt0PropagateGpu`, `SeasonalPipelineF64` | GPU | airSpring → S72/S80 |
| **Spectral** | `spectral::*` | lyapunov_exponent/averaged, anderson_2d/3d/3d_correlated, anderson_4d, wegner_block_4d, anderson_sweep_averaged, find_w_c, level_spacing_ratio, almost_mathieu_hamiltonian, find_all_eigenvalues, lanczos, lanczos_eigenvalues, detect_bands | CPU + GPU | hotSpring S26 → S84 |
| **Linear algebra** | `linalg::*` | solve_f64_cpu, cholesky_f64, eigh_f64 | CPU + GPU | hotSpring |
| **Optimization** | `optimize::*` | brent, lbfgs_numerical | CPU | airSpring V035 → S84 |
| **Bio ops** | `ops::bio` | `BatchedMultinomialGpu`, `GillespieGpu`, `WrightFisherGpu` | GPU | wetSpring + neuralSpring → S58 |
| **Grid ops** | `ops::grid` | grid_search_3d | GPU | groundSpring → S71 |
| **Fused reduce** | `ops::fused_map_reduce_f64` | `FusedMapReduceF64` (shannon, simpson, l1_norm, sum_of_squares) | GPU | wetSpring + airSpring → S64 |
| **Batch ops** | `ops::batched_*` | `BatchedElementwiseF64`, `BatchedOdeRK4F64` | GPU | airSpring + wetSpring → S58 |
| **Numerical** | `numerical::*` | trapz, `BistableOde`, `MultiSignalOde` | CPU | wetSpring S58 |
| **ESN** | `esn_v2` | ESN new/train/predict | GPU | hotSpring + wetSpring |
| **Special** | `special::anderson_transport` | localization_length | CPU | wetSpring S52 |

### Primitive Count Summary

| Category | CPU | GPU | Total |
|----------|-----|-----|-------|
| Stats / metrics / distributions | 28 | 8 | 36 |
| Spectral / Anderson | 8 | 8 | 16 |
| Bio ops (Gillespie, WF, multinomial) | 0 | 6 | 6 |
| Hydrology | 5 | 4 | 9 |
| Optimization | 2 | 0 | 2 |
| Linear algebra | 1 | 2 | 3 |
| Numerical / ODE | 2 | 1 | 3 |
| ESN | 0 | 3 | 3 |
| **Total** | **44** (CPU) | **32** (GPU) | **76** |

---

## Part 2: Issues Found and Fixed

| Issue | Version | Fix |
|-------|---------|-----|
| `BatchedMultinomialGpu::sample` signature break (5 args → `BatchedMultinomialConfig`) | V67 | Updated 3 call sites with `BatchedMultinomialConfig { cumulative_probs: true, seed: None }` |
| `SeasonalGpuParams` private padding fields (`_pad0`, `_pad1`) | V67 | Workaround: `bytemuck::Zeroable::zeroed()` + field-by-field assignment |
| `SeasonalPipelineF64::dispatch` returns typed `SeasonalOutput`, not `Vec<f64>` | V67 | Map `barracuda::SeasonalOutput` → `groundspring::SeasonalOutput` field-by-field |
| `anderson_4d`, `wegner_block_4d` not re-exported from `barracuda::spectral` | V68 | Used full path `barracuda::spectral::anderson::anderson_4d` |
| pollster → `tokio_block_on` migration | V62 | All async GPU dispatch uses `barracuda::device::test_pool::tokio_block_on` |
| `WgpuDevice::new()` → `WgpuDevice::new_f64_capable()` | V62 | f64-capable device selection with fallback |

---

## Part 3: Cross-Spring Evolution Benchmarks

### V33 Three-Mode Benchmark (all 28 experiments)

| Mode | Total Time | Tests | vs Python |
|------|-----------|-------|-----------|
| Default (no barracuda) | 22,030ms | 279/279 | 11.5× faster |
| barracuda CPU | 22,828ms | 279/279 | +1.7% (free) |
| barracuda GPU | 9,798ms | 279/279 | **2.2× faster** |

**Star performers** (GPU speedup from cross-spring absorption):
- **Exp 009 quasiperiodic: 47.4×** — hotSpring Sturm tridiag (S26)
- **Exp 019 jackknife: 4.1×** — barracuda optimized jackknife (S64)
- **Exp 020 freeze-out: 1.7×** — barracuda chi² grid fit (S64)

### V55 Three-Mode Lib Tests

| Mode | Tests | Time | Speedup |
|------|-------|------|---------|
| Default | 333/333 | 4.30s | 1.0× |
| barracuda CPU | 333/333 | 4.26s | 1.0× (−1%) |
| barracuda GPU | 327/333* | 0.24s | **17.9×** |

*6 GPU failures are pre-existing f64 WGSL parser compatibility — fall back to CPU correctly.

### Key Insight: CPU Delegation is Free

barracuda CPU delegation adds ~1.7% overhead from function indirection. For
compute-heavy experiments (Anderson, RAWR), barracuda is actually *faster*
than local code due to optimized implementations. The real win is GPU
dispatch for embarrassingly parallel workloads.

---

## Part 4: Cross-Spring Evolution Learnings

### hotSpring Precision Shaders → groundSpring

| What Evolved | From | To | Impact |
|-------------|------|------|--------|
| DF64 core streaming | S58 (biomeGate QCD) | All Springs | f64-class precision on consumer GPUs |
| Spectral module (Anderson, Lanczos, Sturm) | S26 (Kachkovskiy) | groundSpring Exp 008-009, 012, 018 | **49.5× speedup** (Exp 009) |
| 4D Anderson + Wegner RG | S84 (condensed matter) | groundSpring tissue immunology (V68) | 4D tissue disorder modeling |
| `Fp64Strategy` auto-select | S58 | All GPU dispatch | Native vs DF64 routing |

### wetSpring Bio Shaders ↔ neuralSpring (Bidirectional)

| What Evolved | Direction | Impact |
|-------------|-----------|--------|
| Shannon, Simpson, Bray-Curtis | wetSpring → ToadStool → groundSpring | Diversity metrics for Exp 004, 016 |
| `BatchedMultinomialGpu` | groundSpring → neuralSpring metalForge → all | GPU occupancy for rare biosphere |
| `GillespieGpu`, `WrightFisherGpu` | wetSpring → neuralSpring → groundSpring | Stochastic bio simulation |
| ESN reservoir update | wetSpring → hotSpring → barracuda | Regime classification |

### airSpring Optimizer → groundSpring

| What Evolved | From | To | Impact |
|-------------|------|------|--------|
| L-BFGS numerical | V035 param fitting → S84 | freeze-out grid refinement (V68) | Sub-grid precision for QCD |
| Brent root-finding | V035 Richards PDE → S70+ | band edge refinement (V55) | 1e-12 precision band edges |
| FAO-56 hydrology chain | V035 → S70+ | fao56 module (V55) | ET₀ + Kc + water balance |
| `McEt0PropagateGpu` | V010 → S72 | MC uncertainty propagation (V67) | GPU Monte Carlo ET₀ |
| `SeasonalPipelineF64` | fused pipeline → S80 | seasonal step (V67) | Fused ET₀→Kc→WB→stress |

---

## Part 5: Paper Control Chain

### Open Data Provenance (33 experiments)

| # | Experiment | Open Data Source | Synthetic/Real | Control Status |
|---|-----------|-----------------|----------------|----------------|
| 001 | Sensor noise | Dong et al. 2020 (calibration data) | Synthetic benchmark | CPU ✓ GPU ✓ |
| 002 | Observation gap | ERA5/NOAA GHCND | Synthetic benchmark | CPU ✓ GPU pending |
| 003 | FAO-56 error | FAO Paper 56 | Synthetic benchmark | CPU ✓ GPU ✓ |
| 004 | Sequencing noise | Synthetic community | Synthetic benchmark | CPU ✓ GPU ✓ |
| 005 | Seismic | NMSZ synthetic | Synthetic benchmark | CPU ✓ GPU ✓ |
| 006 | Signal specificity | Massie 2012 PNAS | Synthetic (Gillespie) | CPU ✓ GPU ✓ |
| 007-013 | Bootstrap/RAWR/resampling | Wang 2021, Lee 2024 | Synthetic benchmark | CPU ✓ GPU partial |
| 014 | Drift selection | R. Anderson 2022 mBio | Wright-Fisher simulation | CPU ✓ GPU ✓ |
| 015 | Uncertainty bridge | Dong 2020 + Bourgain-Kachkovskiy | Cross-domain chain | CPU ✓ |
| 016-018 | Rare biosphere / quasispecies / band edge | R. Anderson 2015, Dolson 2023, Filonov-Kachkovskiy | Synthetic benchmark | CPU ✓ GPU ✓ |
| 019-021 | Jackknife / freeze-out / spectral recon | Bazavov 2016, 2025 | Synthetic benchmark | CPU ✓ GPU ✓ |
| 022-024 | ET₀→Anderson / no-till / aggregate | Cross-spring | Synthetic benchmark | CPU ✓ |
| 025-027 | WDM precision / size / vendor | MD simulation | Synthetic benchmark | CPU ✓ |
| 028 | NPU Anderson | AKD1000 DMA | Live hardware | CPU ✓ NPU ✓ |
| 029-032 | NUCLEUS (GHCND, NCBI, NUCLEUS, IRIS) | Live NOAA/NCBI/IRIS data | **Real data** | CPU ✓ (sovereign fallback) |
| 033 | Tissue Anderson (Paper 12) | Synthetic tissue lattice | Synthetic benchmark | CPU ✓ GPU ✓ (4D V68) |

### Three-Tier Hardware Validation Matrix

| Tier | Substrate | Status | Count |
|------|-----------|--------|-------|
| Tier 1: barracuda CPU | Safe Rust | **376/376 PASS** | 33 experiments |
| Tier 2: barracuda GPU | RTX 4070 / Titan V | **780/780 tests** | 15 wired modules |
| Tier 3: metalForge | CPU + GPU + NPU | **187/187 checks** | 30 workloads |

---

## Part 6: Action Items for ToadStool

### P0 — Breaking / Correctness

1. **`SeasonalGpuParams` constructor**: Add `SeasonalGpuParams::new(...)` to avoid `bytemuck::zeroed()` workaround for private padding fields.
2. **`anderson_4d`/`wegner_block_4d` re-export**: Add to `pub use anderson::{...}` in `spectral/mod.rs` for import parity with 1D/2D/3D.

### P1 — Performance / Evolution

3. **L-BFGS GPU variant**: Current `lbfgs_numerical` is CPU-only. Batched numerical gradient on GPU would benefit large-scale parameter sweeps across all Springs.
4. **Eigenvector GPU solver**: `spectral::find_all_eigenvalues` (Sturm) gives eigenvalues but not eigenvectors. A tridiag QL GPU eigenvector solver would unlock Exp 012 (transport) and Exp 017 (spin chain) GPU paths.
5. **Batched Brent GPU with custom closures**: `BrentGpu` currently supports only pre-defined functions (VanGenuchten, GreenAmpt, Polynomial). Supporting custom closures would unlock band_structure GPU refinement.

### P2 — Documentation / Quality

6. **BREAKING_CHANGES.md**: The `BatchedMultinomialGpu::sample` signature change broke 3 groundSpring call sites. A changelog of breaking API changes would help all Springs catch up faster.
7. **Cross-spring provenance tags**: Consider tagging each barracuda function with its origin Spring and session (e.g., `#[origin(hotSpring, S26)]`). groundSpring already documents this in handoffs but it would be powerful in the API docs.

### P3 — Future Evolution

8. **`BatchedStatefulF64` multi-day pipeline**: groundSpring's seasonal water balance could benefit from GPU-resident state across days. The API exists but needs a host-side loop pattern.
9. **Richards PDE GPU solver** (`RichardsGpu`): airSpring's infiltration model is a future groundSpring integration for coupled soil physics.
10. **`SubstratePipeline` integration**: groundSpring's metalForge already does pipeline dispatch; deeper integration with barracuda's `SubstratePipeline` / `InterconnectTopology` would enable first-class multi-GPU support.

---

## Provenance

| Item | Value |
|------|-------|
| groundSpring commit | `d6eb0c6` (V68) |
| ToadStool pin | S87 (`2dc26792`) |
| barracuda tests | 14,200+ |
| barracuda WGSL shaders | 844 |
| groundSpring tests | 780 |
| groundSpring validation checks | 376/376 |
| Python parity | 28/28 |
| Debt | Zero |

---

## Quality Gates

| Gate | Status |
|------|--------|
| `cargo fmt --check` | PASS |
| `cargo clippy --all-targets --all-features -- -W clippy::pedantic -W clippy::nursery` | PASS (zero warnings) |
| `cargo test --workspace` | 780 passed, 0 failed |
| `cargo doc --no-deps` | PASS |
| Zero unsafe | PASS |
| Zero TODO | PASS |
| Zero .unwrap() | PASS |
| Zero #[allow] without reason | PASS |

---

## Addendum: GPU Parity Buildout + Mixed-Hardware Pipeline (V68b)

### New GPU Parity Tests (7 added, 780 total)

| Test | Barracuda Op | What It Validates |
|------|-------------|-------------------|
| `gpu_mc_et0_propagation_parity` | `McEt0PropagateGpu` | MC ET₀ mean near FAO-56 Example 18 (3.88), determinism |
| `gpu_seasonal_pipeline_parity` | `SeasonalPipelineF64` | Multi-cell output (ET₀, Kc, ETc, θ, stress), determinism |
| `gpu_multinomial_occupancy_deterministic` | `BatchedMultinomialGpu` | Post-API-fix determinism, dominant species detection |
| `gpu_lbfgs_refine_improves_grid_fit` | `lbfgs_numerical` | Recovers T₀=160, κ₂=0.015 from exact data, chi²/dof < 1 |
| `gpu_tissue_4d_anderson_eigenvalues` | `anderson_4d` | 4D lattice (L=3, 81 sites), finite eigenvalues, LSR in (0,1) |
| `gpu_tissue_4d_wegner_rg_coarsen` | `wegner_block_4d` | Fine/coarse lattice sizes (L=4→L=2), dimension preservation |

### New Mixed-Hardware Pipeline Checks (42→57, 187 total)

| Section | Checks | What It Validates |
|---------|--------|-------------------|
| G: GPU→NPU PCIe Bypass | 8 | GPU→NPU→CPU pipeline routing, direct PCIe link, bypass savings, reverse NPU→GPU→CPU |
| H: NUCLEUS Coordination | 7 | `FullNucleus` health, capability composition, NPU discovery, degradation |

### toadStool action: GPU→NPU Unidirectional Streaming

groundSpring validated that GPU→NPU direct transfer via PCIe-low avoids the CPU
round-trip (GPU→CPU→NPU = 2 hops). For the `anderson_4d` → regime classification
pipeline, this means:
1. GPU computes 4D Anderson eigenvalues
2. NPU classifies regime (int8 DMA) directly from GPU memory
3. CPU only touches the final provenance store

This pattern generalizes to any GPU-compute → NPU-classify pipeline. ToadStool's
unidirectional streaming should support this GPU→NPU direct path natively.
