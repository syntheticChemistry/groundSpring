# groundSpring V100 — Deep Debt Audit + Evolution Handoff

**Date**: March 14, 2026
**From**: groundSpring (V100)
**To**: toadStool/barraCuda, coralReef, biomeOS
**Pins**: barraCuda v0.3.5, toadStool S130+ (`bfe7977b`), coralReef Iteration 10 (`d29a734`)
**License**: AGPL-3.0-only

---

## Summary

V100 is a deep audit and debt resolution pass. No new experiments — this
release hardens the existing 35 experiments, 102 delegations, and 19,658
lines of Rust across 133 source files.

- **Build-breaking bug fixed**: `akida-driver` path dependency used
  `phase1/toadstool` (lowercase) but the actual directory is
  `phase1/toadStool` (camelCase). No `cargo` command could run on
  case-sensitive filesystems.
- **4 rustfmt violations fixed** in `biomeos/mod.rs` and `gpu.rs`.
- **Silent fallback eliminated**: `validate_weather.rs` used
  `unwrap_or(0.0)` for 4 benchmark JSON fields — silently defaulting to
  zero on schema drift. Replaced with `.expect()` to fail loudly.
- **Hardcoded primal name removed**: `biomeos/mod.rs` special-cased
  `"beardog"` health method. Now tries both qualified and bare methods
  for all primals — capability-based, zero primal-specific knowledge.
- **Tolerance provenance**: All metalForge tolerance constants now have
  doc comments citing mathematical justification and literature source.
- **Bare literals eliminated**: `rare_biosphere.rs` test used bare
  `1e-10`; replaced with `tol::ANALYTICAL`.
- **Avoidable clone removed**: `freeze_out.rs` test cloned `obs` when
  `&obs` sufficed for both parameters.
- **CI scoped**: `cargo fmt` now checks only groundSpring packages, not
  path dependencies (prevents toadStool formatting drift from blocking CI).
- **Test count aligned**: CONTRIBUTING.md now says `908 default-feature
  tests (936 across all feature gates)` instead of misleading `936`.

### Verification

| Check | Result |
|-------|--------|
| `cargo check --workspace` | PASS |
| `cargo fmt` (3 packages) | PASS |
| `cargo clippy -D warnings` | PASS (0 warnings) |
| `RUSTDOCFLAGS="-D warnings" cargo doc` | PASS |
| `cargo test --workspace` | PASS (908 tests + 11 doc-tests) |
| Python `pytest tests/` | PASS (287 tests) |

---

## Current Delegation Inventory

**102 active delegations** (61 CPU + 41 GPU). No new delegations this
version. Full inventory in `specs/BARRACUDA_EVOLUTION.md`.

### CPU Delegations (61)

Stats (25): `mean`, `std_dev`, `percentile`, `rmse`, `mae`,
`nash_sutcliffe`, `mbe`, `r_squared`, `index_of_agreement`, `hit_rate`,
`bootstrap_mean`, `rawr_mean`, `bootstrap_median`, `bootstrap_std`,
`jackknife_mean_variance`, `pearson_correlation`, `spearman_correlation`,
`covariance`, `norm_cdf`, `norm_ppf`, `chi2_decomposed`,
`moving_window_stats_f64`, `shannon`, `simpson`, `pielou_evenness`.

Regression (5): `fit_linear`, `fit_quadratic`, `fit_exponential`,
`fit_logarithmic`, `fit_all`.

Diversity (4): `bray_curtis`, `rarefaction_curve`, `chao1_classic`,
`detection_power`.

Evolution (3): `detection_threshold`, `kimura_fixation_prob`,
`multinomial_sample_cpu`.

Linalg (2): `solve_f64_cpu`, `cholesky_f64`.

Anderson (7): `lyapunov_exponent`, `lyapunov_averaged`,
`analytical_localization_length`, `anderson_2d_correlated`,
`anderson_sweep_averaged`, `find_w_c`, `find_all_eigenvalues`.

Spectral (6): `spectral_bandwidth`, `spectral_condition_number`,
`classify_spectral_phase`, `marchenko_pastur_bounds`,
`empirical_spectral_density`, `almost_mathieu_hamiltonian`.

Numerical/pipeline (9): `trapz`, `hargreaves_et0`, `makkink_et0`,
`turc_et0`, `hamon_et0`, `StatefulPipeline`, `WaterBalanceState`,
`SeasonalGpuParams`, `SeasonalPipelineF64`.

### GPU Delegations (41)

Ops: `SumReduceF64`, `VarianceReduceF64`, `VarianceF64`,
`FusedMapReduceF64`, `CorrelationF64`, `CovarianceF64`,
`AutocorrelationF64`, `Fft1DF64`, `PeakDetectF64`.

Bio: `GillespieGpu`, `BatchedMultinomialGpu`, `WrightFisherGpu`.

ET₀/pipeline: `HargreavesBatchGpu`, `BatchedElementwiseF64`,
`McEt0PropagateGpu`, `SeasonalPipelineF64`.

Device: `WgpuDevice`, `PrecisionRoutingAdvice`, `GpuDriverProfile`.

---

## Absorption Opportunities for barraCuda

### No new absorption requests

All 102 delegations are wired and validated. The local math that remains
in groundSpring is justified:

| Module | Local Math | Reason |
|--------|-----------|--------|
| `spectral_recon.rs` | Small matrix ops (`mat_transpose_mul`, etc.) | <100 elements — GPU dispatch overhead exceeds gain |
| `linalg.rs` | `tridiag_eigh` (QL iteration) | QL outperforms dense Jacobi for tridiagonal; `tridiag_eigh_barracuda` exists for cross-validation |
| `almost_mathieu.rs` | `eigenvalues_qr_dense` (Givens QR) | CPU fallback when `barracuda-gpu` disabled |

### Quality signals for upstream confidence

- `#![forbid(unsafe_code)]` on both crates
- Zero `unwrap()` in production code (908+ test `unwrap`s are intentional)
- All 5 production `clone()` calls justified (Arc sharing or pre-consumption)
- 28/28 benchmark JSONs have complete `_provenance` with commit SHA
- `tests/test_baseline_integrity.py` enforces provenance schema (261 tests)
- No FASTQ/mzML/MS2 parsers (no streaming I/O concern)
- Largest file: 724 lines (37% tests) — well under 1000-line max
- All tolerance values use named `tol::*` constants or have inline justification

---

## Findings Relevant to toadStool/barraCuda Evolution

### 1. akida-driver Path Case Sensitivity

The `akida-driver` optional dependency referenced `phase1/toadstool/`
(lowercase) but the directory is `phase1/toadStool/` (camelCase). On
case-sensitive filesystems (standard Linux), this prevented ALL cargo
commands from running. **toadStool action**: audit all cross-primal path
references for case sensitivity.

### 2. `cargo fmt --all` Checks Path Dependencies

When groundSpring runs `cargo fmt --all`, it also checks formatting in
toadStool source files (because `akida-driver` is a path dep). Any
toadStool formatting drift blocks groundSpring CI. **Resolution**:
groundSpring now uses `-p` to scope fmt checks. **toadStool action**:
consider running `cargo fmt` in your own CI so downstream springs don't
discover formatting issues.

### 3. Primal Health Method Inconsistency

BearDog responds to bare `"health"` while other primals use qualified
`"{primal}.health"`. groundSpring previously special-cased BearDog.
We now try both methods for all primals. **biomeOS action**: standardize
health method naming across all primals — either all bare or all
qualified.

### 4. PRNG Alignment Still Pending

CPU uses `Xorshift64`; GPU uses `xoshiro128**`. Stochastic validation
binaries match statistically (within `tol::STOCHASTIC`) but not bitwise.
This is documented and accepted. When barraCuda adds CPU `xoshiro128**`,
groundSpring can align for bitwise reproducibility.

---

## Evolution Readiness (GPU Promotion Tiers)

### Tier A — Ready for Direct GPU Shader Promotion

Already fully delegating to barraCuda GPU ops: `stats/*`, `bootstrap`,
`rarefaction`, `anderson`, `drift`, `seismic`, `freeze_out`, `jackknife`,
`rare_biosphere`, `quasispecies`.

### Tier B — Needs Minor Adaptation

| Module | Blocker | Effort |
|--------|---------|--------|
| `spectral_recon` | FFT wired but Tikhonov local (small matrices) | Low — only worth it at scale |
| `fao56` | `BatchedElementwiseF64` wired; batch pipeline partial | Low |
| `wdm` | Regression delegated; finite-size scaling local | Low |

### Tier C — New Shader Required

| Module | Needed | Complexity |
|--------|--------|------------|
| `tissue_anderson` | 3D compartmented geometry kernel | Medium |
| `esn` | Reservoir dynamics kernel | Medium |

---

## Validation Chain Position

```
Python Phase 0 (29 exp) → Rust Phase 1 (34 binaries, 395/395)
→ barraCuda CPU (3-tier parity) → barraCuda GPU (3-tier parity)
→ metalForge (140 checks) → NUCLEUS (V99 live)
→ Deep debt audit (V100, this handoff)
```

---

## Primitive Consumption Summary (V100)

No change from V99. Full table in `specs/BARRACUDA_EVOLUTION.md`.

| Category | Count | Source |
|----------|-------|--------|
| CPU stats | 25 | `barracuda::stats::*` |
| CPU regression | 5 | `barracuda::stats::regression::*` |
| CPU diversity | 4 | `barracuda::stats::diversity::*` |
| CPU linalg | 2 | `barracuda::linalg::*` |
| CPU anderson | 7 | `barracuda::anderson::*` |
| CPU spectral | 6 | `barracuda::spectral::*` |
| CPU numerical | 1 | `barracuda::numerical::trapz` |
| CPU pipeline | 9 | `barracuda::pipeline::*` |
| GPU ops | 9 | `barracuda::ops::*` |
| GPU bio | 3 | `barracuda::ops::bio::*` |
| GPU ET₀ | 4 | `barracuda::ops::*` |
| GPU device | 3 | `barracuda::device::*` |
| **Total** | **102** | **61 CPU + 41 GPU** |
