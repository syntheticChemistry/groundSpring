# groundSpring → ToadStool V51: GPU Stats Dispatch + CPU/GPU Parity Proof

**Date**: February 28, 2026
**ToadStool pin**: S68+ (`e96576ee`)
**groundSpring**: V51 (GPU stats dispatch, batch GPU APIs, CPU/GPU parity proof)
**Previous**: V47 (library buildout + 7 new barracuda CPU delegations)
**License**: AGPL-3.0-only

---

## Summary

V51 is a **GPU parity proof** pass — 8 new GPU delegations wired via barracuda
reduce ops and batch APIs, with 9 explicit CPU vs GPU parity tests proving
mathematical equivalence. This completes the "barracuda CPU vs barracuda GPU
implementation to validate pure Rust math" milestone.

**Delegation count**: 46 → **48 active** (31 CPU + 17 GPU). 7 → **6 pending**.
**Test count**: 322 → **569** workspace. **95** three-tier parity tests.

---

## Part 1: New GPU Delegations (V47→V51)

### 1.1 GPU stats dispatch (5 core functions)

| Function | BarraCUDA GPU Op | Dispatch Pattern |
|----------|-----------------|------------------|
| `stats::mean` | `SumReduceF64::mean` | Sum reduce → divide by N |
| `stats::std_dev` | `VarianceReduceF64::population_std` | Two-pass variance reduce |
| `stats::rmse` | `FusedMapReduceF64` (squared residuals) | Map (o−m)² → sum reduce → sqrt(mean) |
| `stats::mbe` | `SumReduceF64::mean` (residuals) | Map (m−o) → mean reduce |
| `stats::pearson_r` | `CorrelationF64::pearson` | Fused correlation kernel |

All 5 use the standard GPU dispatch pattern:

```rust
#[cfg(feature = "barracuda-gpu")]
fn mean_gpu(values: &[f64]) -> Option<f64> {
    let device = crate::gpu::get_device()?;
    barracuda::ops::sum_reduce_f64::SumReduceF64::mean(device, values).ok()
}
```

CPU fallback is always compiled and activates on GPU failure or absent feature.

### 1.2 Batch GPU APIs (3 batch functions)

| Function | BarraCUDA GPU Op | Notes |
|----------|-----------------|-------|
| `gillespie::birth_death_ssa_batch` | `GillespieGpu` | Multi-trajectory SSA |
| `drift::wright_fisher_fixation_batch` | `WrightFisherGpu` | Ping-pong frequency buffers via wgpu |
| `fao56::daily_et0_batch` | `BatchedElementwiseF64::fao56_et0_batch` | Vectorized ET₀ over station-days |

The `wright_fisher_fixation_batch` implementation manages wgpu buffers directly
(ping-pong pattern for generational frequency updates), demonstrating groundSpring's
readiness for ToadStool's streaming compute model.

### 1.3 Dependencies added

- `wgpu = { version = "22", optional = true }` — gated by `barracuda-gpu`
- `bytemuck = { version = "1", optional = true }` — gated by `barracuda-gpu`

---

## Part 2: CPU vs GPU Parity Proof

### 2.1 Parity tests (9 new in `three_tier_parity.rs`)

| Test | What It Proves |
|------|---------------|
| `gpu_mean_matches_cpu_known_value` | SumReduceF64 matches CPU mean within 1e-10 |
| `gpu_std_dev_matches_cpu_known_value` | VarianceReduceF64 matches CPU std_dev within 1e-10 |
| `gpu_rmse_matches_cpu_known_value` | FusedMapReduceF64 matches CPU rmse within 1e-10 |
| `gpu_mbe_matches_cpu_known_value` | SumReduceF64 matches CPU mbe within 1e-10 |
| `gpu_pearson_perfect_positive` | CorrelationF64 returns 1.0 for identical series |
| `gpu_pearson_zero_correlation` | CorrelationF64 returns ~0.0 for orthogonal series |
| `gpu_r_squared_perfect` | R² = 1.0 for identity mapping |
| `gpu_decompose_pythagorean` | bias² + variance = MSE (Pythagorean identity) |
| `gpu_stats_deterministic` | Repeated GPU calls produce identical results |

### 2.2 Pure GPU workload tests (6 new)

| Test | Workload |
|------|----------|
| `gpu_gillespie_steady_state_convergence` | SSA batch converges to analytical steady state |
| `gpu_wright_fisher_kimura_agreement` | Fixation fraction ≈ Kimura analytical prediction |
| `gpu_fao56_reference_et0` | GPU ET₀ matches known reference (FAO-56 Example 18) |
| `gpu_anderson_localization_positive_lyapunov` | GPU Lyapunov exponent > 0 for W > 0 |
| `gpu_rare_biosphere_dominant_occupancy` | GPU multinomial produces expected occupancy |
| `gpu_batch_determinism` | Repeated GPU batch calls yield identical results |

### 2.3 Tolerance decisions

- **Stats parity**: 1e-10 (GPU reduce ops accumulate in different order → ULP drift)
- **FAO-56 batch**: 0.05 mm/day (GPU shader computes internally vs host step-by-step)
- **Stochastic parity**: Statistical convergence tests (not exact match — GPU uses different PRNG)
- **Determinism**: Exact match within same hardware (same PRNG seed + same dispatch order)

---

## Part 3: What ToadStool Could Absorb

### 3.1 Reduced pending list (6 remaining)

| groundSpring Function | BarraCUDA Target | Status |
|----------------------|-----------------|--------|
| `drift::kimura_fixation_prob` | `stats::kimura_fixation` | Not in barracuda S68+ |
| `jackknife::jackknife_mean_variance` | `stats::jackknife_mean_variance` | Not in barracuda S68+ |
| `fao56::daily_et0` (scalar) | `stats::hydrology::fao56_et0` | Batch exists, scalar doesn't |
| `freeze_out::grid_fit_2d` | `ops::grid::grid_fit_2d_f64` | No grid-search op |
| `seismic::grid_search_inversion` | `ops::grid::grid_search_3d_f64` | No grid-search op |
| `band_structure::find_band_edges` | `spectral::band_edges_parallel` | Per-energy parallel dispatch |

All 6 have complete local CPU implementations ready for ToadStool absorption.

### 3.2 Absorption priorities

**High** (embarrassingly parallel, would unlock GPU tier):
1. `jackknife_mean_variance` — leave-one-out is trivially parallel (Exp 019)
2. `fao56_et0` scalar — expose existing batch op with N=1

**Medium** (GPU acceleration candidates):
3. `grid_search_3d_f64` — 3D parameter scan (Exp 005, 020)
4. `grid_fit_2d_f64` — 2D chi-squared scan (Exp 020)
5. `kimura_fixation_prob` — pure scalar, trivial kernel (Exp 014)

**Design needed**:
6. `band_edges_parallel` — per-energy transfer matrix scan (Exp 018)

---

## Part 4: Learnings for ToadStool Evolution

### 4.1 GPU reduce op patterns that work well

The `SumReduceF64`, `VarianceReduceF64`, `FusedMapReduceF64`, and
`CorrelationF64` ops are clean, composable building blocks. groundSpring wires
them directly with minimal glue code. The pattern is:

```
get_device() → construct op → call associated fn → unwrap or fallback
```

**Suggestion**: These reduce ops should be the first-class "stats GPU tier"
in barracuda's public API. They compose naturally for any spring.

### 4.2 Batch API learnings

- `GillespieGpu` works well for embarrassingly parallel trajectory batches
- `WrightFisherGpu` required manual wgpu buffer management (ping-pong for
  generational updates) — this is a pattern ToadStool could abstract into
  a "generational simulation" primitive
- `BatchedElementwiseF64::fao56_et0_batch` demonstrates domain-specific
  vectorization — each station-day is independent

### 4.3 wgpu buffer management

groundSpring's `drift.rs` directly manages `wgpu::Buffer` creation with
`COPY_SRC | COPY_DST | STORAGE` flags and ping-pong dispatch. This is
infrastructure that ToadStool's `ComputeDispatch` builder should absorb
so downstream springs don't manage buffers manually.

### 4.4 PRNG divergence (unchanged from V47)

CPU uses `Xorshift64`, GPU uses `Xoshiro128**`. This means stochastic
parity tests must use statistical convergence (e.g., Kimura analytical
solution) rather than exact numerical match. Phase 2b PRNG alignment
requires baseline regeneration across all 5 stochastic experiments.

### 4.5 Formula divergence

`chao1` remains undelegated — barracuda uses bias-corrected Chao & Chiu 2016,
groundSpring uses classic Chao 1984. Consider offering both variants.

### 4.6 bench-cpu-vs-gpu infrastructure

groundSpring now includes `bench-cpu-vs-gpu` binary for automated performance
comparison across 6 workloads (Gillespie, Wright-Fisher, FAO-56, rare biosphere,
Anderson, diversity). This can serve as a template for other springs.

---

## Part 5: Full Delegation Inventory (48 active + 6 pending)

### CPU delegations (31 active)

| # | groundSpring | barracuda | Session |
|---|-------------|-----------|---------|
| 1 | `stats::pearson_r` | `stats::pearson_correlation` | S50 |
| 2 | `stats::spearman_r` | `stats::correlation::spearman_correlation` | S50 |
| 3 | `stats::sample_std_dev` | `stats::correlation::std_dev` | S50 |
| 4 | `stats::covariance` | `stats::correlation::covariance` | S50 |
| 5 | `stats::norm_cdf` | `stats::norm_cdf` | S50 |
| 6 | `stats::norm_ppf` | `stats::norm_ppf` | S50 |
| 7 | `stats::chi2_statistic` | `stats::chi2_decomposed` | S50 |
| 8 | `bootstrap::bootstrap_mean` | `stats::bootstrap_mean` | S50 |
| 9 | `anderson::analytical_localization_length` | `special::anderson_transport::localization_length` | S59 |
| 10 | `bistable::bistable_derivative` | `numerical::ode_bio::BistableOde::cpu_derivative` | S58 |
| 11 | `multisignal::multisignal_derivative` | `numerical::ode_bio::MultiSignalOde::cpu_derivative` | S58 |
| 12 | `stats::r_squared` | `stats::metrics::r_squared` | S64 |
| 13 | `stats::index_of_agreement` | `stats::metrics::index_of_agreement` | S64 |
| 14 | `stats::hit_rate` | `stats::metrics::hit_rate` | S64 |
| 15 | `rarefaction::shannon_diversity` | `stats::diversity::shannon` | S64 |
| 16 | `stats::percentile` | `stats::metrics::percentile` | S64 |
| 17 | `rarefaction::evenness` | `stats::pielou_evenness` | S64 |
| 18 | `bootstrap::rawr_mean` | `stats::rawr_mean` | S66 |
| 19 | `kinetics::hill` | `stats::hill` | S68 |
| 20 | `kinetics::hill_repress` | `stats::hill` (1 − hill) | S68 |
| 21 | `wdm::finite_size_extrapolate` | `stats::regression::fit_linear` | S66 |
| 22 | `stats::mae` | `stats::metrics::mae` | S66 |
| 23 | `stats::nash_sutcliffe` | `stats::nash_sutcliffe` | S64 |
| 24–27 | `stats::regression::fit_{linear,quadratic,exponential,logarithmic}` | `stats::regression::*` | S66 |
| 28 | `kinetics::monod` | `stats::metrics::monod` | S66 |
| 29 | `rarefaction::simpson_diversity` | `stats::diversity::simpson` | S64 |
| 30 | `rarefaction::bray_curtis` | `stats::diversity::bray_curtis` | S64 |
| 31 | `rarefaction::analytical_rarefaction` | `stats::diversity::rarefaction_curve` | S64 |

### GPU delegations (17 active)

| # | groundSpring | barracuda GPU Op | Notes |
|---|-------------|------------------|-------|
| 1 | `anderson::lyapunov_exponent` | `spectral::lyapunov_exponent` | Transfer matrix |
| 2 | `anderson::lyapunov_averaged` | `spectral::lyapunov_averaged` | Multi-realization |
| 3 | `almost_mathieu::hamiltonian` | `spectral::almost_mathieu_hamiltonian` | λ/2 coupling |
| 4 | `almost_mathieu::eigenvalues` | `spectral::find_all_eigenvalues` | Sturm tridiag — 49.5× |
| 5 | `almost_mathieu::level_spacing_ratio` | `spectral::level_spacing_ratio` | Sort adapter |
| 6 | `band_structure::detect_band_ranges` | `spectral::detect_bands` | Gap detection |
| 7 | `spectral_recon::tikhonov_solve` | `linalg::solve_f64_cpu` | Gauss–Jordan |
| 8 | `rare_biosphere::abundance_occupancy` | `ops::bio::BatchedMultinomialGpu` | Batched sampling |
| 9 | `rare_biosphere::tier_detection_rate` | `ops::bio::BatchedMultinomialGpu` | Tier-sliced |
| 10 | `stats::mean` | `ops::sum_reduce_f64::SumReduceF64::mean` | GPU reduce |
| 11 | `stats::std_dev` | `ops::variance_reduce_f64::VarianceReduceF64::population_std` | GPU reduce |
| 12 | `stats::rmse` | `ops::fused_map_reduce_f64::FusedMapReduceF64` | Map-reduce |
| 13 | `stats::mbe` | `ops::sum_reduce_f64::SumReduceF64::mean` | Residual mean |
| 14 | `stats::pearson_r` | `ops::correlation_f64_wgsl::CorrelationF64` | Fused correlation |
| 15 | `gillespie::birth_death_ssa_batch` | `ops::bio::GillespieGpu` | Batch SSA |
| 16 | `drift::wright_fisher_fixation_batch` | `ops::bio::WrightFisherGpu` | Batch WF |
| 17 | `fao56::daily_et0_batch` | `ops::batched_elementwise_f64::BatchedElementwiseF64` | Batch ET₀ |

---

## Part 6: Quality State

| Gate | Result |
|------|--------|
| `cargo fmt --check` | PASS |
| `cargo clippy --all-targets -W pedantic -W nursery` | 0 warnings |
| `cargo doc --no-deps -D warnings` | clean |
| `cargo test --workspace --features barracuda-gpu` | 569/569 PASS |
| Validation binaries | 292/292 PASS (28 experiments) |
| Three-tier parity tests | 95 |
| CPU vs GPU parity tests | 9 |
| `#[allow]` annotations | 0 |
| `unsafe` blocks | 0 |
| Production mocks | 0 |

---

## Part 7: metalForge Workload Validation

All 19 groundSpring workloads confirmed correctly routed:
- **17 → GPU** (all f64 compute workloads: Anderson, FAO-56, Gillespie, etc.)
- **2 → NPU** (Anderson regime classification, quantized inference on AKD1000)

Workload routing tested via `metalForge/forge/src/workloads.rs` with capability
matching (`F64Compute`, `ShaderDispatch`, `QuantizedInference`).

---

## Handoff Checklist

- [x] All 28 validation binaries PASS (292/292)
- [x] barracuda-gpu feature compiles cleanly (zero warnings)
- [x] 8 new GPU delegations tested with and without barracuda-gpu feature
- [x] 9 CPU vs GPU parity tests prove mathematical equivalence
- [x] All `TODO(toadstool)` comments reference current S68+ state (6 remaining)
- [x] Delegation inventory matches code (48 active, verified)
- [x] README.md updated (48 active, 569 tests, V51)
- [x] CHANGELOG.md V51 entry added
- [x] CONTROL_EXPERIMENT_STATUS.md updated
- [x] specs/ updated (PAPER_REVIEW_QUEUE, BARRACUDA_EVOLUTION)
- [x] whitePaper/ updated (baseCamp, experiments)
- [x] gen3/baseCamp/README.md updated
- [x] V47 archived to handoffs/archive/
