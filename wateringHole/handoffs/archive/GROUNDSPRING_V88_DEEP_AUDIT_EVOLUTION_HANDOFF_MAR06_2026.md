# SPDX-License-Identifier: AGPL-3.0-only

# groundSpring V88 → toadStool / barraCuda / coralReef Deep Audit Handoff

**Date**: 2026-03-06
**From**: groundSpring V88 — 35 experiments, 395/395 checks, 824+ tests, 93 delegations (56 CPU + 37 GPU), 261/261 provenance tests
**To**: barraCuda, toadStool, coralReef
**License**: AGPL-3.0-only
**Pins**: barraCuda `e1184f3`, toadStool S96c (`d77fc546`), coralReef `849fedd`

---

## Executive Summary

- Full codebase audit completed: zero TODO/FIXME/unsafe/unwrap in production, all files <1000 lines, `cargo fmt` + `cargo clippy --workspace -- -D warnings` + `cargo doc` + `cargo test` all clean
- Structured logging evolved: `eprintln!` → `log::warn!` in all library code
- Formal provenance schema documented in `specs/PROVENANCE_SCHEMA.md`
- Benchmark drift guard now auto-discovers `control/*/benchmark_*.json` (no hardcoded list)
- Missing `_doi` in `benchmark_et0_methods.json` fixed — 261/261 Python provenance tests pass
- All inline magic numbers documented with analytical derivations
- PRNG alignment (xorshift64 → xoshiro128**) remains the primary evolution blocker

---

## Part 1: Quality Gates — All Green

| Gate | Status |
|------|--------|
| `cargo fmt --check` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS (pedantic + nursery) |
| `cargo doc --workspace --no-deps` | PASS (0 warnings) |
| `cargo test --workspace` | PASS (824+ tests, 0 failures) |
| `unsafe_code = "forbid"` | Enforced (workspace lint) |
| `missing_docs = "deny"` | Enforced (workspace lint) |
| Python `test_baseline_integrity.py` | 261/261 PASS |
| Files > 1000 lines | 0 (max 743: `rarefaction.rs`) |
| TODO/FIXME/HACK in production | 0 |
| `unwrap()` in production | 0 |
| `panic!()` in production | 0 |

---

## Part 2: Delegation Inventory (93 active)

**56 CPU delegations**: stats (mean, std_dev, percentile, pearson_r, spearman_r, covariance, rmse, mae, mbe, nash_sutcliffe, r_squared, index_of_agreement, hit_rate, norm_cdf, norm_ppf, chi2_decomposed, fit_linear, fit_quadratic, fit_exponential, fit_logarithmic, fit_all, bootstrap_mean, bootstrap_median, bootstrap_std, rawr_mean, jackknife_mean_variance, moving_window_stats_f64), bio (multinomial_sample_cpu, kimura_fixation_prob, error_threshold, chao1_classic, detection_power, detection_threshold, hill, monod), spectral (anderson_potential, lyapunov_exponent), linalg (eigh_f64), numerical (trapz), hydrology (fao56_et0, hargreaves_et0, crop_coefficient, soil_water_balance, makkink_et0, turc_et0, hamon_et0, thornthwaite_et0)

**37 GPU delegations**: stats GPU ops (SumReduceF64, VarianceReduceF64, CorrelationF64, FusedMapReduceF64), bio GPU (GillespieGpu, WrightFisherGpu, BatchedMultinomialGpu, BootstrapMeanGpu, JackknifeMeanGpu), spectral GPU (lyapunov_averaged, anderson_sweep_averaged, anderson_2d, anderson_3d, anderson_3d_correlated, anderson_4d, wegner_block_4d, find_w_c, classify_spectral_phase), optimize (brent, lbfgs_numerical, batched_nelder_mead_gpu, grid_search_3d), linalg (solve_f64_cpu, cholesky_f64), hydrology GPU (HargreavesBatchGpu, McEt0PropagateGpu, SeasonalPipelineF64, BatchedElementwiseF64, BatchedOdeRK4F64)

---

## Part 3: Evolution Requests

### barraCuda action: PRNG alignment (Tier B — blocking)

groundSpring's `prng::Xorshift64` diverges from barraCuda's `xoshiro128**`. This blocks:
1. CPU↔GPU PRNG stream parity
2. Stochastic baseline regeneration
3. Cross-spring PRNG determinism

**Request**: Provide a `barracuda::prng::Xorshift64` compatibility shim, or document the migration path for springs to adopt xoshiro128** and regenerate all stochastic baselines.

### barraCuda action: Test coverage for GPU-gated paths

groundSpring's lowest-coverage modules are all GPU-feature-gated:
- `bootstrap.rs` — 49.8% (GPU path untested)
- `stats/regression.rs` — 55.0%
- `stats/moving_window.rs` — 57.9%
- `stats/correlation.rs` — 74.1%

**Request**: Consider exposing a mock/stub GPU device for CI testing, or document the recommended pattern for testing GPU-gated code paths without hardware.

### toadStool action: `log` crate integration

groundSpring now uses `log` for structured logging. toadStool binaries should initialize a log subscriber (e.g. `env_logger`) so groundSpring warnings are visible in pipeline runs.

### coralReef action: No changes requested

coralReef Phase 6 is stable. Gap to execution remains coralDriver (Phase 7).

---

## Part 4: What V88 Changed

| Change | Impact |
|--------|--------|
| Added `log = "0.4"` to groundspring + forge | Structured logging replaces eprintln |
| `eprintln!` → `log::warn!` in `biomeos/mod.rs`, `nucleus.rs` | Library code no longer writes to stderr |
| Added `_doi` to `benchmark_et0_methods.json` | 261/261 provenance tests pass |
| Provenance comments on hardcoded expected values | band_structure, multisignal, bistable, seismic |
| Named Tikhonov λ constants | spectral_recon, validate_gpu_tier/spectral |
| Auto-discovery in `regenerate_benchmarks.sh` | New experiments picked up automatically |
| `specs/PROVENANCE_SCHEMA.md` | Formal schema for benchmark JSON provenance |
| Updated experiment count: 35 experiments, 395/395 | README, CONTRIBUTING, CONTROL_EXPERIMENT_STATUS |

---

## Part 5: Cross-Spring Learnings

### For barraCuda

- **`if let Ok` + CPU fallback**: All 93 delegations follow this pattern. CPU fallback is always compiled (no `#[cfg(not(feature))]` guard). This pattern should be documented as the canonical delegation pattern.
- **Tolerance tiers**: groundSpring's 13-tier tolerance system (`tol::DETERMINISM` through `tol::EQUILIBRIUM`) works well. Each tier has documented justification. barraCuda should consider exporting a similar tiered tolerance module.
- **Provenance chain**: groundSpring's `paper → Python → JSON → Rust → pass/fail` chain is machine-auditable. barraCuda's GPU tests should adopt benchmark JSON provenance.

### For toadStool

- **metalForge workloads**: 30 workloads with capability-based routing work well. `dispatch::route()` selects substrate by capability match, not by name.
- **NucleusHarness**: `metalForge/forge/src/nucleus.rs` provides `finish() -> bool` for binaries that manage their own exit code. Consider absorbing into toadStool's harness.

---

## Part 6: Test Coverage Report

| Component | Line Coverage | Notes |
|-----------|--------------|-------|
| groundspring library | ~90%+ (most modules) | GPU paths drag down average |
| metalForge forge library | ~95%+ | Strong |
| Validation binaries | 0% (by design) | Run as integration, not unit-tested |
| **Workspace total** | 74.46% | Inflated by 0% binaries |

### Per-module gaps (below 80%)

| Module | Coverage | Root Cause |
|--------|----------|------------|
| `bootstrap.rs` | 49.8% | GPU batch path |
| `stats/regression.rs` | 55.0% | barracuda delegation |
| `stats/moving_window.rs` | 57.9% | barracuda delegation |
| `stats/correlation.rs` | 74.1% | GPU correlation |
| `fao56/et0_methods.rs` | 72.5% | barracuda hydrology |
| `fao56/mod.rs` | 79.8% | GPU batch |
| `fao56/pipeline.rs` | 79.5% | GPU pipeline |
| `freeze_out.rs` | 79.6% | GPU optimizer |

---

## Part 7: Evolution Tier Map (unchanged from V87)

| Tier | Modules | Status |
|------|---------|--------|
| **A (Lean)** | anderson, almost_mathieu, band_structure, bistable, bootstrap, drift, esn, fao56, freeze_out, gillespie, jackknife, kinetics, lanczos, linalg, multisignal, quasispecies, rare_biosphere, rarefaction, seismic, spectral_recon, stats/*, tissue_anderson, transport, wdm | 93 delegations active |
| **B (Adapt)** | prng (xorshift→xoshiro alignment) | Blocked on PRNG migration |
| **Stays Local** | decompose (scalar), ode (generic), validate (harness), error (types) | By design |

---

*This handoff is unidirectional: groundSpring → barraCuda / toadStool / coralReef. No response expected.*
