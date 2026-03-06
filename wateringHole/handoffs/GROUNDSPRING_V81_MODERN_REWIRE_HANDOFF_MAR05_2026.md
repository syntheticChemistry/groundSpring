# groundSpring V81 — Modern Rewire Handoff

**From**: groundSpring V81
**To**: ToadStool / BarraCUDA / coralReef teams
**Date**: March 5, 2026
**Pins**: barraCuda `a4c20a5` | toadStool S94b `bc89fa61` | coralReef `2e89541`

---

## Summary

V81 completes the modern rewiring cycle begun at V77 (wgpu 28). This version
adds `BootstrapMeanGpu` GPU dispatch, fixes a feature-gate mismatch for L-BFGS
refinement, clones the coralReef sovereign shader compiler, and validates the
full cross-spring benchmark suite (27/27 checks).

**Counts**: 88 delegations (51 CPU + 37 GPU), 812+ workspace tests, 390 Python
tests, zero clippy warnings, zero unsafe.

---

## 1. What Changed

### 1a. BootstrapMeanGpu GPU Dispatch (NEW — delegation #88)

`bootstrap_mean()` in `crates/groundspring/src/bootstrap.rs` now dispatches to
`barracuda::stats::bootstrap::BootstrapMeanGpu` when the `barracuda-gpu` feature
is enabled.

**Fallback chain**:
```
barracuda-gpu → BootstrapMeanGpu::dispatch (GPU parallel resampling)
barracuda     → barracuda::stats::bootstrap_mean (CPU)
default       → local Xorshift64 (sovereign)
```

The GPU path computes resample means on-device, then `percentile_ci` runs on
CPU to extract the confidence interval. No barraCuda API changes needed —
`BootstrapMeanGpu` already exists and works as documented.

### 1b. freeze_out Feature Gate Fix

`barracuda::optimize` (L-BFGS, Brent, etc.) is gated behind `#[cfg(feature = "gpu")]`
in barraCuda. groundSpring's `freeze_out::lbfgs_refine` was previously gated
under `#[cfg(feature = "barracuda")]` which caused `E0432: unresolved import`
when `barracuda-gpu` was not enabled.

**Fix**: All L-BFGS constants and the `lbfgs_refine_barracuda` function are now
gated with `#[cfg(feature = "barracuda-gpu")]`. The sovereign grid-search
fallback remains always-available.

### 1c. coralReef Ecosystem Integration

Cloned `ecoPrimals/coralReef` (renamed from coralNAK). Validated 390 tests PASS.
coralReef has completed Phases 1–5: full SPIR-V frontend, f64 transcendental
lowering (sqrt, rcp, exp2, log2, sin, cos), optimization passes, register
allocation, and binary encoding.

Cross-spring pattern absorption confirmed: coralReef adopted `BTreeMap` for
deterministic IR serialization, removed `unsafe unwrap_unchecked`, and added
cross-spring provenance doc-comments in `lower_f64/`.

### 1d. V80 Recap (for completeness)

- Fused `correlation_full` GPU dispatch (5-accumulator single-pass Pearson)
- Welford single-pass CPU stats (`std_dev`, `sample_std_dev`, `mean_and_std_dev`)
- Covariance GPU path derived from `CorrelationF64::correlation_full`
- barraCuda `makkink_et0` coefficient fix (−0.012 → −0.12)

---

## 2. barraCuda Usage Inventory (88 delegations)

### CPU Delegations (51)

All under `#[cfg(feature = "barracuda")]` with sovereign fallback:

| Module | Functions | barraCuda Path |
|--------|-----------|----------------|
| `stats/metrics` | mean, std_dev, sample_std_dev, mean_and_std_dev, variance, sample_variance, median, percentile, skewness, kurtosis, coefficient_of_variation | `barracuda::stats::*` |
| `stats/correlation` | pearson, spearman, covariance, pearson_full | `barracuda::stats::correlation::*` |
| `regression` | linear_regression, r_squared | `barracuda::stats::regression::*` |
| `rarefaction` | chao1, rarefaction_curve | `barracuda::stats::diversity::*` |
| `rare_biosphere` | chao1_richness | `barracuda::stats::diversity::chao1` |
| `bootstrap` | bootstrap_mean, rawr_mean | `barracuda::stats::bootstrap_mean` / `rawr_mean` |
| `jackknife` | jackknife_mean, jackknife_resamples | `barracuda::stats::jackknife::*` |
| `fao56` | fao56_et0 | `barracuda::stats::hydrology::fao56_et0` |
| `et0_methods` | hargreaves_et0, makkink_et0, turc_et0, hamon_et0 | `barracuda::stats::hydrology::*` |
| `seismic` | stats::mean | `barracuda::stats::mean` |
| `drift` | wright_fisher_step, kimura_fixation | `barracuda::stats::evolution::*` |
| `quasispecies` | quasispecies_frequencies | `barracuda::stats::evolution::quasispecies_frequencies` |
| `kinetics` | hill_coefficient | `barracuda::stats::hill_coefficient` |
| `decompose` | seasonal_decompose | `barracuda::stats::decompose::*` |
| `linalg` | mat_vec_mul, solve_tridiagonal, cholesky_solve | `barracuda::linalg::*` |

### GPU Delegations (37)

All under `#[cfg(feature = "barracuda-gpu")]` with CPU fallback:

| Module | Dispatch | barraCuda GPU Type |
|--------|----------|--------------------|
| `stats/metrics` | mean_and_std_dev | `MeanAndStdDevGpu` (Welford fused) |
| `stats/correlation` | pearson_full, covariance | `CorrelationF64` (5-acc fused) |
| `rarefaction` | rarefaction_gpu | `RarefactionGpu` |
| `bootstrap` | bootstrap_mean | `BootstrapMeanGpu` **(V81 NEW)** |
| `jackknife` | jackknife_mean | `JackknifeMeanGpu` |
| `anderson` | anderson_transport | `AndersonGpu` |
| `drift` | wright_fisher_gpu | `WrightFisherGpu` |
| `freeze_out` | lbfgs_refine | `lbfgs_numerical` **(V81 gate fix)** |
| `gillespie` | gillespie_ssa_gpu | `GillespieGpu` |
| `wdm` | green_kubo_* | `GreenKuboGpu` |
| `esn` | esn_gpu | `EsnGpu` |
| `lanczos` | sparse_eigenvalues | `SparseLanczosGpu` |
| + 25 metalForge workloads | various | Various GPU dispatchers |

---

## 3. Cross-Spring Evolution Notes

### What groundSpring Observed

| Origin | Contribution | Where It Landed |
|--------|-------------|-----------------|
| hotSpring | Precision shaders (DF64, f64 transcendentals) | barraCuda DF64 tiers, coralReef lower_f64/ |
| hotSpring | Sturm eigensolver (49.5× speedup) | barraCuda `tridiag_eigh`, groundSpring `anderson` |
| wetSpring | Bio-stats (diversity, rarefaction, QS kinetics) | barraCuda `stats/diversity`, `stats/evolution` |
| wetSpring | Signal integration (multi-signal QS) | barraCuda `bio/batched_multinomial` |
| neuralSpring | ESN dispatch, LSTM patterns | barraCuda `ops/esn`, groundSpring `esn` module |
| airSpring | ET₀ methods (Makkink, Turc, Hamon) | barraCuda `stats/hydrology` |
| groundSpring | RAWR, metalForge topology, L-BFGS | barraCuda `stats/bootstrap`, `multi_gpu/*`, `optimize/lbfgs_gpu` |
| coralReef | BTreeMap determinism, silent-default audit | coralReef IR (absorbed from groundSpring patterns) |

### What Each Spring Gets Back

- **groundSpring** benefits from barraCuda's fused reduction shaders (evolved from
  hotSpring precision needs) and DF64 precision tiers
- **coralReef** benefits from groundSpring's Rust quality patterns and will
  eventually provide the sovereign WGSL→binary compiler path
- **toadStool** has absorbed 76+ groundSpring ops and provides the orchestration
  layer (ComputeDispatch) that all springs use

---

## 4. Recommended Next Steps

### For barraCuda

1. **BootstrapMedianGpu / BootstrapStdGpu**: groundSpring has `bootstrap_median`
   and `bootstrap_std` but they still use CPU-only paths. GPU versions would
   complete the bootstrap family
2. **RAWR GPU**: `rawr_mean` has CPU delegation but no GPU dispatch yet.
   `RawrWeightedMeanGpu` exists as a shader but isn't wired in groundSpring
3. **Regression GPU batch**: `BatchLinearRegressionGpu` exists in barraCuda but
   groundSpring's scalar API doesn't use it yet — consider batched interface

### For toadStool

1. **SPRING_ABSORPTION_TRACKER**: Update from V68 → V81. New delegations:
   `BootstrapMeanGpu` (V81), fused `correlation_full` (V80)
2. **ComputeDispatch coverage**: 144 ops migrated, ~139 legacy remaining.
   groundSpring's 37 GPU delegations are a validation surface for ComputeDispatch
3. **coralReef integration**: When coralReef produces native binaries,
   toadStool will need a `CoralDispatch` path alongside `WgpuDispatch`

### For coralReef

1. **Naga upgrade**: groundSpring uses naga via wgpu 28; coralReef's SPIR-V
   frontend should track the same naga version for IR compatibility
2. **DF64 lowering**: barraCuda DF64 shaders are a validation target for
   coralReef's f64 lowering. The DF64 utilization strategy (idle f32 cores →
   ~48-bit precision) should be a first-class code path in coralReef
3. **AMD/Intel backends**: groundSpring has dual GPU hardware (Titan V + RTX 4070).
   When coralReef adds AMD/Intel backends, groundSpring can validate
   cross-vendor parity

---

## 5. Validation Certificate

```
cargo check --workspace                                    PASS
cargo clippy --workspace --all-targets -- -D warnings      PASS (0 warnings)
cargo test --workspace                                     812+ tests PASS
cargo test --workspace --features barracuda                812+ tests PASS
cargo run --bin benchmark-cross-spring                     27/27 checks PASS
cargo run --bin bench-cpu-vs-gpu                           16 workloads profiled
coralReef cargo test --workspace                           390 tests PASS
```

**Note**: `--features barracuda-gpu` deferred — Titan V (nouveau/NVK) causes
system freeze under GPU compute. RTX 4070 GPU paths validated via metalForge
where stable.

---

## 6. File Changes in V81

| File | Change |
|------|--------|
| `crates/groundspring/src/bootstrap.rs` | Added `bootstrap_mean_gpu` dispatch |
| `crates/groundspring/src/freeze_out.rs` | Gate fix: `barracuda` → `barracuda-gpu` for L-BFGS |
| `CHANGELOG.md` | V81 entry |
| `README.md` | V81 status, 88 delegations, coralReef |
| `CONTROL_EXPERIMENT_STATUS.md` | V81 counts |
| `wateringHole/README.md` | V81 handoff active, V80 archived |
| `whitePaper/baseCamp/README.md` | V81 summary |
| `whitePaper/experiments/README.md` | 88 delegations, 812+ tests |
| `metalForge/forge/src/bin/benchmark_cross_spring.rs` | Updated provenance strings |
| `wateringHole/handoffs/SOVEREIGN_PIPELINE_CROSS_PRIMAL_HANDOFF_MAR05_2026.md` | coralReef 2e89541 update |

---

*groundSpring V81 — BootstrapMeanGpu + freeze_out gate fix + coralReef ecosystem integration. March 5, 2026.*
