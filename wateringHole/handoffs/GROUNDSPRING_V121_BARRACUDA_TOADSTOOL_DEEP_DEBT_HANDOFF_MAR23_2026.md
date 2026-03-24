# groundSpring V121 → barraCuda / toadStool Deep Debt + Absorption Handoff

**Date:** March 23, 2026
**From:** groundSpring V121
**To:** barraCuda team, toadStool team
**License:** AGPL-3.0-or-later
**Pins:** barraCuda v0.3.7, toadStool S158+, coralReef Iteration 55+

---

## Executive Summary

groundSpring V121 completes a comprehensive deep-debt audit and evolution sprint
begun in V119. This session centralized all inline tolerance literals into named
constants, hardened provenance to require `validation_script` and `command` fields,
added benchmark JSON round-trip testing, evolved `#[allow]` → `#[expect]` patterns,
synced all documentation to barraCuda v0.3.7, and updated MSRV to 1.87. All quality
gates pass: zero clippy (pedantic + nursery), zero fmt diff, zero doc warnings, zero
unsafe, zero unwrap/expect in production code, ≥92% library line coverage, 990+ tests.

This handoff documents: (1) what changed relevant to upstream, (2) patterns worth
absorbing, (3) current delegation inventory, (4) evolution priorities for the
toadStool/barraCuda team, and (5) cross-spring learnings.

---

## 1. What Changed in V121

### 1a. Tolerance Centralization (10 new named constants)

All inline float literals in validation binaries have been replaced with named
constants in `crates/groundspring-validate/src/tolerances.rs`:

| Constant | Value | Purpose | Provenance |
|----------|-------|---------|------------|
| `TOL_ET0_BASELINE` | 0.10 | ET₀ baseline comparison band | FAO-56 Table 2 |
| `SANITY_ES_KPA` | (1.8, 2.2) | Saturation vapor pressure bounds | FAO-56 Eq. 11, T≈20°C |
| `SANITY_EA_KPA` | (1.2, 1.6) | Actual vapor pressure bounds | FAO-56 Eq. 14, RH≈60-70% |
| `SANITY_P_KPA` | (99.0, 102.0) | Atmospheric pressure bounds | sea-level ± 2 kPa |
| `SANITY_U2_MS` | (1.5, 2.5) | 2m wind speed bounds | FAO-56 Table 4 |
| `SANITY_DAYLIGHT_HOURS` | (15.0, 17.0) | Summer daylight hours | 45°N solstice |
| `SANITY_MC_CV_PCT` | (1.0, 15.0) | Monte Carlo CV percentage | N=10000, σ/√N |
| `SANITY_VARIANCE_SUM` | (0.9, 1.1) | Variance partitioning sum | should ≈ 1.0 |
| `SANITY_PM_HARG_RATIO` | (0.3, 3.5) | PM/Hargreaves ratio bounds | FAO-56 Table 2 |
| `SANITY_PM_HARG_DIFF_MAX` | 10.0 | PM-Hargreaves absolute diff | mm/day ceiling |

**Impact on upstream**: This pattern (named constants with provenance citations) is
worth adopting in barraCuda test infrastructure. Currently barraCuda tests use bare
float literals; centralizing them would improve traceability.

### 1b. Provenance Hardening

`crates/groundspring-validate/src/provenance.rs`: `try_print_provenance_header()`
now **requires** `validation_script` and `command` fields in benchmark JSON
provenance blocks. Missing fields return `Err` instead of silently skipping.

New tests:
- `benchmark_json_round_trip`: Verifies JSON parse → serialize → reparse losslessness
  and required provenance field presence (`_source`, `_provenance.baseline_commit`,
  `_provenance.baseline_date`, `validation_script`, `command`)
- `provenance_requires_script_and_command`: Validates error on missing fields

**Impact on upstream**: If barraCuda/toadStool use benchmark JSONs for validation,
consider adopting the same required-field contract.

### 1c. Lint Evolution

- `#[allow(dead_code)]` → `#[expect(dead_code, reason = "...")]` in `ipc_error.rs`
  for `IpcResult<T>` under `cfg(not(feature = "tarpc-ipc"))`
- Zero `#[allow()]` remains in production code

### 1d. barraCuda Version Sync

All documentation references updated from `v0.3.5` → `v0.3.7`:
- `metalForge/ABSORPTION_MANIFEST.md`
- `wateringHole/CROSS_SPRING_SHADER_EVOLUTION.md`
- `specs/BARRACUDA_EVOLUTION.md`
- `specs/BARRACUDA_REQUIREMENTS.md`
- `specs/CROSS_SPRING_EVOLUTION.md`
- `specs/README.md`
- `specs/PAPER_REVIEW_QUEUE.md`
- `whitePaper/METHODOLOGY.md`
- `whitePaper/STUDY.md`
- `whitePaper/experiments/README.md`
- `CONTROL_EXPERIMENT_STATUS.md`
- `README.md`

---

## 2. Patterns Worth Absorbing

### 2a. Named Tolerance Constants with Provenance

groundSpring now has 13 tolerance tiers (`tol::*`) + 10 validation-specific sanity
bounds + epsilon guards (`eps::*`). Every constant has:
- A documented physical or mathematical justification
- A provenance citation (paper, equation number, or commit)
- A clear regime label (deterministic, analytical, stochastic, etc.)

**Recommendation for barraCuda**: Consider a `barracuda::test::tol` module that
centralizes GPU test tolerances with similar provenance tracking.

### 2b. `#[expect(reason)]` Over `#[allow]`

Rust 1.87+ supports `#[expect(lint, reason = "...")]` which is strictly better than
`#[allow(lint)]`: it warns if the lint expectation becomes unfulfilled (dead code
becomes used, etc.). groundSpring has fully migrated. Recommended for barraCuda
codebase.

### 2c. Provenance-Required Benchmark JSONs

The `validation_script` + `command` requirement means every benchmark can be
traced to the exact Python command that generated it. This closes the
"how was this baseline created?" question permanently.

### 2d. ValidationHarness (from V120, carried forward)

`check_relative()` and `check_abs_or_rel()` complement `check()` for broader
floating-point validation patterns. Worth considering as a shared cross-spring
validation primitive.

---

## 3. Current Delegation Inventory

**Total**: 110 active delegations (67 CPU + 43 GPU)

### CPU Delegations (67) — `#[cfg(feature = "barracuda")]`

| Category | Count | Key Ops |
|----------|-------|---------|
| Stats core | 15 | mean, std_dev, variance, pearson_r, covariance, rmse, mbe, mae, nse, r², ia, hit_rate, percentile, sample_std_dev, mean_and_std_dev |
| Distributions | 3 | norm_cdf, norm_ppf, chi2_statistic |
| Bootstrap/RAWR | 2 | bootstrap_mean, rawr_mean |
| Spectral | 8 | anderson potential, analytical ξ, disorder sweep, spectral_density, peak_detect, almost_mathieu, hofstadter, eigvalues |
| Bio | 7 | hill, monod, gillespie, bray_curtis, shannon, chao1, wright_fisher |
| Regression | 6 | fit_linear, fit_quadratic, fit_exponential, fit_logarithmic, fit_all, ridge_regression |
| FAO-56 | 4 | fao56_et0, hargreaves, thornthwaite, thornthwaite_heat_index |
| Inverse | 3 | jackknife, tikhonov_solve, fft_power_spectrum |
| Other | 19 | WelfordState, error_threshold, detection_power, kimura, rarefaction, etc. |

### GPU Delegations (43) — `#[cfg(feature = "barracuda-gpu")]`

| Category | Count | Key Ops |
|----------|-------|---------|
| Stats GPU | 11 | SumReduceF64 (mean, mbe), VarianceReduceF64 (std_dev), CorrelationF64, CovarianceF64, MAE, NSE, R² |
| Batch GPU | 6 | GillespieGpu, WrightFisherGpu, BootstrapMeanGpu, BatchedElementwiseF64, McEt0PropagateGpu, SeasonalPipelineF64 |
| Spectral GPU | 8 | anderson_lyapunov, anderson_3d, BatchedEighGpu (Sturm), Lanczos, fused_map_reduce variants |
| Grid GPU | 5 | freeze_out grid_fit_2d, band_edges, seismic grid_search, quasispecies batch, rare_biosphere batch |
| ESN GPU | 1 | EsnClassifier (barracuda-gpu feature) |
| Other GPU | 12 | Fao56Batch, HargreavesBatch, tikhonov, detect_bands, etc. |

---

## 4. Evolution Priorities for toadStool/barraCuda

### P0 — Blocking / High-Value

| Priority | Description | groundSpring Impact |
|----------|-------------|---------------------|
| **PRNG alignment** | `DefaultRng` Xoshiro alignment with `prng.rs` Xorshift64 | Enables deterministic GPU reproducibility for MC experiments |
| **Lanczos at scale** | CSR SpMV + Lanczos eigensolver for large Anderson matrices | Enables 2D/3D Anderson at realistic system sizes |

### P1 — Evolution Opportunities

| Priority | Description | groundSpring Impact |
|----------|-------------|---------------------|
| Sparse matrix-vector product (SpMV) | CSR-format SpMV shader | Inner loop of Lanczos, required for spectral methods at scale |
| `GpuDriverProfile` removal | Complete deprecation after all consumers migrate | groundSpring clean since V120 (`DeviceCapabilities` adopted) |
| Named tolerance module | `barracuda::test::tol` with provenance | Cross-spring quality improvement |
| `#[expect(reason)]` migration | Replace `#[allow]` in barraCuda codebase | Aligns with Rust 2024 edition best practices |

### P2 — Long Horizon (Phase 3)

| Priority | Description | Notes |
|----------|-------------|-------|
| RAWR GPU kernel | Weighted resampling with random walks | Medium effort, needs PRNG alignment first |
| Matrix exponentiation | General exp(iHt) for transport analysis | Medium effort, Cayley exists for SU(3) |
| Sobol indices | Parallel quasi-random sampling | Enables sensitivity analysis at scale |

---

## 5. Cross-Spring Learnings

### 5a. Tolerance Architecture as Ecosystem Pattern

groundSpring's 3-tier tolerance architecture (13 `tol::*` + validation sanity
bounds + `eps::*` guards) has proven effective across 35 experiments and 395
validation checks. The key insight: **tolerances are not just numbers — they are
claims about algorithm behavior that need provenance tracking**. Every tolerance
should answer: "why this value and not a tighter one?"

### 5b. Provenance Hardening Closes Audit Loops

Making `validation_script` and `command` required (not optional) in benchmark
JSONs means every validation check can be traced to the exact Python command
that generated the baseline. This was the last remaining gap in the provenance
chain: `published paper → Python script → benchmark JSON → Rust validator`.

### 5c. `#[expect]` vs `#[allow]` at Scale

After migrating 95+ files from `#[allow]` to `#[expect(reason)]`, the benefits
are clear: unfulfilled expectations are immediately flagged by clippy, preventing
stale suppressions from accumulating. The `reason` string also serves as
documentation for why the suppression exists.

### 5d. Documentation Drift Detection

This audit found `v0.3.5` references in 20+ active doc files after barraCuda
moved to `v0.3.7`. Recommendation: add a CI step that greps for stale version
strings (or use a `VERSION` constant that docs reference).

---

## 6. Quality Certificate

| Gate | Status |
|------|--------|
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic -W clippy::nursery` | 0 warnings |
| `cargo doc --workspace --no-deps` | PASS (0 warnings) |
| `cargo test --workspace` | 990+ tests, 0 failures |
| `cargo deny check` | PASS |
| Library line coverage | ≥92% |
| TODOs/FIXMEs in library code | 0 |
| Unsafe code in library | 0 |
| `#[allow()]` in production code | 0 |
| `unwrap()`/`expect()` in production code | 0 (workspace-denied) |
| Validation checks | 395/395 PASS (340 core + 55 NUCLEUS) |
| Mathematical parity | 29/29 PROVEN |
| Three-tier parity | 29/29 × 3 tiers |

---

## 7. Items Carried Forward from V120

- `GpuDriverProfile` can be removed from barraCuda public API (all known consumers migrated)
- `ValidationHarness` patterns (`check_relative`, `check_abs_or_rel`) available for cross-spring adoption
- Dispatch refactoring pattern (semantic decomposition, not naive splitting) documented as reference

---

*groundSpring V121 — AGPL-3.0-or-later — ecoPrimals ecosystem*
