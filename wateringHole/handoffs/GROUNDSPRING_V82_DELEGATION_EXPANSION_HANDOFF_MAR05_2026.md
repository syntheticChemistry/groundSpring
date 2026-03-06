# SPDX-License-Identifier: AGPL-3.0-only

# groundSpring V82 → Delegation Expansion + Deep Debt Audit Handoff

**Date:** 2026-03-05
**From:** groundSpring V82 (34 experiments, 395/395 checks, 824 workspace tests)
**To:** barraCuda team (absorption guidance), toadStool team (shader evolution)
**License:** AGPL-3.0-only
**Covers:** V81 → V82 (Thornthwaite ET₀, fit_all regression, smart refactoring, deep debt audit)
**Pins:** barraCuda `a4c20a5`, toadStool S95 (`d4817e2e`), coralReef `2e89541`

---

## Executive Summary

- groundSpring expanded from 88 → 91 barracuda delegations (54 CPU + 37 GPU)
- 3 new CPU delegations: `thornthwaite_et0`, `thornthwaite_heat_index`, `fit_all`
- Smart-refactored the 2 largest modules: `esn.rs` (816→3 files) and `fao56/mod.rs` (811→2 files)
- Deep debt audit: zero unsafe, zero production unwrap, zero TODO/FIXME, zero mocks, zero hardcoded paths
- 824 tests pass, zero clippy warnings, all 34 validation binaries green

---

## 1. New Delegations

| # | Function | Module | barraCuda Source | Notes |
|---|----------|--------|-----------------|-------|
| 89 | `thornthwaite_et0` | `fao56::et0_methods` | `stats::hydrology::thornthwaite_et0` | Monthly temp-based ET₀ (Thornthwaite 1948) |
| 90 | `thornthwaite_heat_index` | `fao56::et0_methods` | `stats::hydrology::thornthwaite_heat_index` | Annual heat index from 12 monthly temps |
| 91 | `fit_all` | `stats::regression` | `stats::regression::fit_all` | Unified 4-model regression (lin/quad/exp/log) |

All follow the established `if let Ok/Some` + CPU fallback pattern.

---

## 2. Smart Refactoring

### esn.rs (816 lines → 3 files)

| File | Lines | Content |
|------|-------|---------|
| `esn/mod.rs` | 79 | `RegimeLabel` enum, module doc, re-exports |
| `esn/brain.rs` | 536 | DriftAction, ConceptEdge, ClassificationUncertainty, MultiHeadUncertainty, edge detection, seed generation |
| `esn/classifier.rs` | 265 | GOE_R/POISSON_R constants, rule-based classifiers, spectral features, EsnClassifier (GPU) |

Split by domain: brain architecture (Nautilus lineage) vs spectral classification (Anderson lineage).

### fao56/mod.rs (811 lines → 2 files)

| File | Lines | Content |
|------|-------|---------|
| `fao56/mod.rs` | 628 | Core Penman-Monteith, Hargreaves, crop_coefficient, soil_water_balance, Example 18 |
| `fao56/et0_methods.rs` | 282 | Alternative methods: Makkink, Turc, Hamon, Thornthwaite + 20 tests |

Split by provenance: core FAO-56 (V1 original) vs alternative methods (V79 Exp 035 + V82 Thornthwaite).

---

## 3. Deep Debt Audit Results

| Category | Production | Test |
|----------|-----------|------|
| `unsafe` | 0 | 0 (workspace `forbid`) |
| `unwrap()` | 0 | 47 |
| `expect()` | 0 | 15 |
| `todo!()` / `unimplemented!()` / `panic!()` | 0 | 0 |
| Hardcoded paths/URLs | 0 | 0 |
| Production mocks | 0 | 0 |
| Primal hardcoding in logic | 0 | 0 |
| External C/FFI deps | 0 | wgpu backend only |
| Feature gate issues | 0 | 0 |
| Files > 1000 lines | 0 | 0 (max: 717) |

---

## 4. Full Delegation Inventory (91 Active)

### CPU Delegations (54) — `#[cfg(feature = "barracuda")]`

**Stats core**: mean, std_dev, sample_std_dev, mean_and_std_dev, percentile, norm_cdf, norm_ppf
**Agreement**: rmse, mae, mbe, r_squared, nash_sutcliffe, index_of_agreement, hit_rate
**Correlation**: pearson_r, spearman_r, covariance, pearson_full (fused)
**Regression**: fit_linear, fit_quadratic, fit_exponential, fit_logarithmic, **fit_all** (NEW)
**Bootstrap/RAWR**: bootstrap_mean, bootstrap_median, bootstrap_std, rawr_mean
**Jackknife**: jackknife_mean_variance
**Moving window**: moving_window_stats
**Diversity**: shannon, simpson, bray_curtis, chao1, rarefaction_curve
**Hydrology**: fao56_et0, hargreaves_et0, makkink_et0, turc_et0, hamon_et0, **thornthwaite_et0**, **thornthwaite_heat_index** (NEW), crop_coefficient, soil_water_balance
**Evolution**: kimura_fixation_prob, wright_fisher, quasispecies, error_threshold, detection_power, detection_threshold
**Kinetics**: hill, monod
**Numerical**: green_kubo (trapz)
**Spectral**: chi2_statistic, chi2_decomposed

### GPU Delegations (37) — `#[cfg(feature = "barracuda-gpu")]`

**Correlation**: CorrelationF64 (pearson_full), CovarianceF64
**Stats**: MeanVarianceF64, WeightedDotF64
**Hydrology**: HargreavesBatchGpu, McEt0PropagateGpu, SeasonalPipelineF64
**Bootstrap**: BootstrapMeanGpu
**Spectral**: Anderson 1D/2D/3D/4D, Lyapunov, AlmostMathieu, HofstadterButterfly
**Bio**: GillespieBatchGpu, OdeBatchGpu, WrightFisherGpu, MultinomialGpu
**Grid search**: GridSearchGpu
**Optimization**: NelderMeadGpu, L-BFGS
**Linalg**: SolveF64, CholeskyF64, EighF64, TridiagonalSolveF64, GenEighF64
**ESN**: EsnClassifier (via barracuda::esn_v2::ESN)

---

## 5. What barraCuda Should Know

### Delegation candidates we investigated but deferred

| Candidate | Reason for deferral |
|-----------|-------------------|
| `RawrWeightedMeanGpu` | Algorithm mismatch: GPU uses standard bootstrap (resampling indices), groundSpring RAWR uses Bayesian bootstrap (Exp(1) weights). Different semantics. |
| `BootstrapMedianGpu` | Does not exist in barraCuda yet. CPU path is sufficient for now. |
| `BootstrapStdGpu` | Does not exist in barraCuda yet. |
| `CubicSpline` | No interpolation use case in current experiments. |
| `Sobol/LHS sampling` | No sensitivity analysis experiments yet. Candidate for future work. |

### Pattern evolution: Thornthwaite delegation

Thornthwaite's `heat_index` function is a pure delegation (no CPU fallback needed — barraCuda's version is the reference implementation). The pattern uses:
```rust
#[cfg(feature = "barracuda")]
{ barracuda::stats::hydrology::thornthwaite_heat_index(monthly_temps) }
#[cfg(not(feature = "barracuda"))]
{ /* local implementation */ }
```

This is cleaner than the `if let Some/Ok` pattern for functions that never return `None/Err`.

### Cross-method ET₀ validation

groundSpring now validates 6 ET₀ methods against each other (Exp 035):
PM, Hargreaves, Makkink, Turc, Hamon, Thornthwaite. All produce results within (0, 20) mm/day for the FAO-56 Example 18 reference conditions.

---

## 6. What toadStool Should Know

### Smart refactoring patterns

groundSpring's `esn.rs` and `fao56/mod.rs` refactors demonstrate the "smart split" approach:
- Split by **domain** (brain vs classifier), not by **line count**
- Shared types stay in `mod.rs`, domain-specific logic moves to submodules
- All tests move with their functions — no orphaned tests
- Re-exports preserve the public API (no breaking changes)

### Cross-spring shader provenance update

The Thornthwaite delegation adds to the hydrology shader lineage:
```
airSpring → barraCuda hydrology → groundSpring fao56
```

Full ET₀ method coverage: PM (FAO-56 standard), Hargreaves (radiation-based),
Makkink (Dutch standard), Turc (Mediterranean), Hamon (simplest),
Thornthwaite (temperature-only, climate classification).

---

## 7. Recommended Next Steps

### For groundSpring
1. **Experiment buildout**: Papers 12 (eigenvector GPU), 23/24 (full GPU chain) remain partially wired
2. **Sensitivity analysis**: Wire Sobol sampling from barraCuda for Exp 003 uncertainty propagation
3. **coralReef integration**: When coralDriver matures, test SM70 binary compilation for Titan V workloads

### For barraCuda
1. **RAWR GPU alignment**: Consider adding a Bayesian bootstrap (Exp(1) weights) GPU kernel to match RAWR semantics
2. **BootstrapMedianGpu / BootstrapStdGpu**: Extend the BootstrapMeanGpu pattern to median and std
3. **Eigenvector solver**: GPU tridiagonal eigenvector extraction would complete Paper 12's GPU chain

### For toadStool
1. **Thornthwaite shader**: If hydrology batch workloads grow, a `thornthwaite_batch_f64.wgsl` would be useful
2. **fit_all batch**: Batch regression fitting across multiple datasets — common in multi-experiment validation

---

## 8. Validation Certificate

```
groundSpring V82
  cargo fmt --check:                              PASS
  cargo clippy --workspace --all-targets -D warn: PASS (0 warnings)
  cargo test --workspace:                         PASS (824 tests, 0 failures)
  Experiments:                                    34/34 validated (395/395 checks)
  Delegations:                                    91 active (54 CPU + 37 GPU)
  barraCuda pin:                                  a4c20a5
  toadStool pin:                                  S95 (d4817e2e)
  coralReef pin:                                  2e89541
  Deep debt:                                      Zero (0 unsafe, 0 unwrap, 0 TODO)
  Max file size:                                  717 lines (freeze_out.rs)
```
