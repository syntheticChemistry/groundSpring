# groundSpring — Control Experiment Run Log

Historical run log extracted from CONTROL_EXPERIMENT_STATUS.md.
See [CONTROL_EXPERIMENT_STATUS.md](CONTROL_EXPERIMENT_STATUS.md) for the current experiment register and status.

> **Note**: V75+ runs are documented in CHANGELOG.md and per-version handoffs
> in `wateringHole/handoffs/`. This log covers the structured run format used
> through V74. Current status: V144, 1,123 lib tests (default), 427/427 checks, 110 delegations (67 CPU + 43 GPU), 20 IPC methods + 2 signal dispatch paths across 7 primals.

## Run Log

### Run 43 (V74 Deep Debt + ToadStool/barraCuda Catch-Up + Full Validation Benchmark, Mar 4, 2026)

- `cargo fmt --check`: PASS
- `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic`: PASS (0 warnings, default + barracuda)
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`: PASS (0 warnings)
- `cargo test --workspace` (default): 790 tests, all PASS
- `cargo test --workspace --features barracuda`: all PASS
- **Validation binaries (default)**: 28/28 PASS, 21.7s total (release mode)
- **Validation binaries (barracuda CPU)**: 28/28 PASS, 19.6s total (release mode, **−10%**)
- **Cross-spring benchmark**: 23/23 PASS, 4.5s (barraCuda v0.3.1, ToadStool S93)
- **Notable speedups**: FAO-56 −78%, spectral-recon −81%, freeze-out −67%, jackknife −57%
- **barraCuda**: v0.3.1 (standalone primal, `f6895ca`)
- **toadStool**: S93 (`9319668d`)
- **Key changes**: clippy pedantic CI, deep debt (wdm/drift/linalg), tolerance deduplication, ABSORPTION_MANIFEST V74 (32→81 delegations), benchmark cross-spring updated to S93, full cross-spring provenance documented
- **Handoff**: V74 Deep Debt + ToadStool/barraCuda Catch-Up + Benchmark

### Run 42 (V73 Tolerance Architecture + Idiomatic Evolution, Mar 4, 2026)

- `cargo fmt --check`: PASS
- `cargo clippy --workspace -- -D warnings`: PASS (0 warnings)
- `cargo doc --workspace --no-deps`: PASS (0 warnings)
- `cargo test --workspace` (default): 790 tests, all PASS
- `cargo llvm-cov --workspace --ignore-filename-regex '(validate_|bench_|bin/)' --fail-under-lines 90`: 97.25% line coverage
- **barraCuda**: v0.3.1 (standalone primal)
- **ToadStool**: S93
- **Key changes**: 13-tier `tol::` module, `eps::` module, ~170 bare literals → named constants, `f64::midpoint`, 18 tail expression refactors, capability-based discovery evolution
- **Handoff**: V73 Tolerance Architecture + Absorption Guide

### Run 41 (V72 Deep Audit + Debt Evolution, Mar 3, 2026)

- `cargo fmt --check`: PASS
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: PASS (0 warnings)
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`: PASS (0 warnings)
- `cargo test --workspace` (default): 786+ tests, all PASS
- `cargo test --workspace --features barracuda-gpu`: all PASS
- **barraCuda**: v0.3.1 (standalone primal)
- **ToadStool**: S93
- **Handoff**: V72 Deep Audit + Debt Evolution, all gates green, 786+ tests

### Run 40 (V69 Cross-Spring Evolution Parity + Universal Precision Audit, Mar 2, 2026)

- `cargo fmt --check`: PASS
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: PASS (0 warnings)
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`: PASS (0 warnings)
- `cargo test --workspace` (default): 783 tests, all PASS (up from 780)
- `cargo test --workspace --features barracuda-gpu`: 785 tests, all PASS
- **ToadStool pin**: S86 → S87 (`2dc26792`) — FHE shader fix, async-trait reclassification, unsafe audit
- **NEW** 5 cross-spring evolution parity tests:
  - `gpu_shannon_diversity_cross_spring_parity` (wetSpring S64 → `FusedMapReduceF64`)
  - `gpu_simpson_diversity_cross_spring_parity` (wetSpring S64 → `FusedMapReduceF64`)
  - `gpu_seismic_grid_search_cross_spring_parity` (groundSpring + `ComputeDispatch`)
  - `gpu_anderson_2d_eigenvalues_cross_spring_parity` (hotSpring S59 Lanczos, `#[cfg(gpu)]`)
  - `gpu_anderson_3d_eigenvalues_cross_spring_parity` (hotSpring S59 WDM, `#[cfg(gpu)]`)
- **ENHANCED** `benchmark_cross_spring`: 4-phase evolution timeline (Foundation → Absorption → Universal Precision → Modern Wiring), expanded provenance table (+6 S72-S87 ops)
- **DOCUMENTED** S67-S68 universal precision architecture: "Math is universal, precision is silicon"
- **UPDATED** all docs: V68→V69, S86→S87, stale S79 refs, graph TOMLs, 780→783 tests
- **Handoff**: V69 S87 pin + universal precision audit + cross-spring parity

### Run 36 (V63+ Paper 12 Tissue Anderson + GPU Tier Expansion, Mar 2, 2026)

- `cargo fmt --check`: PASS
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS (0 warnings)
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`: PASS (0 warnings)
- `cargo test --workspace` (default): 743 tests, all PASS (up from 725)
- **NEW** `validate_tissue_anderson`: 29/29 PASS — 10 validation scenarios (Exp 033+034)
- **NEW** `tissue_anderson` module: cytokine Anderson lattice, barrier disruption sweep, dimensional duality, geometry-aware drug scoring (6-drug AD panel), 18 unit tests
- **UPDATED** `validate_gpu_tier`: 73/73 PASS (was 66) — 7 tissue Anderson parity checks + Wright-Fisher param fix (s=0.05→s=0.0 for neutral drift)
- **UPDATED** `CONTROL_EXPERIMENT_STATUS.md`: Exp 033/034 registered, 376/376 total validation checks
- **UPDATED** `specs/PAPER_REVIEW_QUEUE.md`: Paper 12 (Anderson Localization in Immunological Signaling) added
- **UPDATED** `specs/BARRACUDA_EVOLUTION.md`: V63 delegation history + tissue_anderson mapping

### Run 35 (V63 Experiment Buildout + GPU Dispatch Validation + metalForge Pipeline, Mar 2, 2026)

- `cargo fmt --check`: PASS
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS (0 warnings)
- `cargo doc --workspace --no-deps`: PASS (0 warnings)
- `cargo test --workspace` (default): 725 tests, all PASS (up from 668)
- Validation binaries (core): 292/292 PASS (unchanged)
- **NEW** `validate_metalforge_pipeline`: 30/30 PASS — 3-stage NPU→GPU→CPU pipeline, P2P transfer strategy, all FallbackPolicy paths, NodeAtomic planning, sovereign degradation, timing instrumentation
- **Phase 1 — 6 GPU delegations wired**:
  - `gillespie::birth_death_ssa_batch` → `GillespieGpu` batch dispatch with 3-mode parity test
  - `drift::wright_fisher_fixation_batch` → `WrightFisherGpu` device acquisition + buffer management
  - `rarefaction::multinomial_sample_batch` → `BatchedMultinomialGpu` with cumulative probability adapter
  - `spectral_recon::tikhonov_solve` → `barracuda::linalg::cholesky_f64` GPU with CPU fallback chain
  - `linalg::tridiag_eigh_barracuda` → `barracuda::linalg::eigh_f64` (Jacobi rotation validation path)
  - `prng::Xoshiro128StarStar` + `GpuAlignedRng` type alias — aligned with `barracuda::ops::prng_xoshiro_wgsl`
- **Phase 2 — bench + parity expansion**:
  - `bench_cpu_vs_gpu` expanded: `bench_multinomial_batch`, `bench_tikhonov`, `bench_tridiag_eigh`, `bench_transport_msd`
  - `validate_gpu_tier` expanded: 6 new parity functions (Gillespie, Wright-Fisher, multinomial, Cholesky, tridiag eigh, PRNG stream)
- **Phase 3 — NUCLEUS compute dispatch**:
  - `validate_nucleus_pipeline` expanded: `validate_compute_execute_anderson`, `validate_compute_submit_batch`, `validate_compute_roundtrip` — Neural API → provider → validate vs local CPU baseline
- **Phase 4 — metalForge mixed-hardware pipeline**:
  - `validate_metalforge_pipeline` (NEW binary): 10 validation scenarios, 30 checks — Anderson regime pipeline, NPU→GPU PCIe P2P/PcieLow transfer, full 3-stage pipeline, Degrade/Skip/Fail fallback, GPU-only chain (zero transfer), CPU-only sovereign degradation, timing instrumentation, NodeAtomic integration
  - AKD1000 NPU correctly identified as PcieLow (PCIe 2.0 x1) with HostBounce fallback
- **Quality**: 0 unsafe, 0 unwrap in library, 0 mocks in production, 0 hardcoded primal names, all `#[expect]` with reasons
- **ToadStool pin**: S79 (`f97fc2ae`) — barracuda GPU delegation fully wired

### Run 34 (V61 Mixed-Hardware Pipeline + NUCLEUS Atomics + Deep Debt, Mar 2, 2026)

- `cargo fmt --check`: PASS
- `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic -W clippy::nursery`: PASS (0 warnings)
- `cargo clippy --workspace --all-targets --features barracuda-gpu -- -D warnings`: PASS (0 warnings)
- `cargo doc --workspace --no-deps`: PASS (0 warnings)
- `cargo test --workspace` (default): 668 tests, all PASS (374 lib + 13 validate + 120 forge + 109 parity + 52 integration)
- `python3 -m pytest tests/`: 375/375 PASS + 3 skipped
- Validation binaries (core): 292/292 PASS (27 binaries)
- metalForge validation: 130/132 PASS (2 expected NPU absence failures)
- **NEW** `validate_mixed_hardware`: 42/42 PASS — topology, fallback chains, pipeline, atomics, degradation, tolerances
- **metalForge infrastructure**: `topology.rs` (PCIe device adjacency, 6 bandwidth tiers), `pipeline.rs` (multi-stage dispatch with fallback), `atomic.rs` (NUCLEUS compositions with sovereign degradation), `dispatch::fallback_chain()` (ordered substrate selection)
- **Deep idiomatic pass**: 13 clippy errors, iterator chains, `serde_json::json!`, `mul_add`, Result-based validate API, provenance headers, hardcoding evolution
- **Coverage**: 668 tests (up from 620), 120 metalForge tests (up from 85), 42 new mixed-hardware checks

### Run 33 (V47 Deep Debt: Hardcode Evolution + #[expect] Migration, Feb 28, 2026)

- `cargo fmt --check`: PASS
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS (0 warnings)
- `cargo clippy --workspace --all-targets -- -W clippy::pedantic`: PASS (0 warnings)
- `cargo doc --workspace --no-deps`: PASS (0 warnings)
- `cargo test --workspace` (default): 452 tests, 292/292 validation checks, all PASS
- **Hardcode evolution**: 8 validation binaries evolved to read thresholds from benchmark JSON instead of inline magic numbers:
  - `validate_rare_biosphere`: `0.2` Spearman → `exp.spearman_occupancy_min`; `0.95` detection → `exp.detection_power_target`
  - `validate_multisignal`: `1e-10` determinism → `exp.determinism_tolerance`; `1.5` variance → `exp.dual_signal_variance_ratio_max`
  - `validate_seismic`: `5520/5620` NY-London → `haversine_reference.ny_london_range` + lat/lon from JSON
  - `validate_spectral_recon`: `opt_idx = 2` → `exp.optimal_lambda_index`; `0.5` ratio → `exp.regularization_tradeoff_ratio_min`
  - `validate_band_edge`: `0.05` edge tol → `exp.edge_tolerance`; `0.01` slack → `exp.gap_monotonicity_slack`; `0.95` fraction → `exp.eigenvalue_band_fraction_min`; `0.05` margin → `exp.eigenvalue_band_margin`
  - `validate_freeze_out`: `0.5` slack → `exp.noise_degradation_slack`
  - `validate_weather`: Added structured analytical provenance header
  - `validate_nucleus_pipeline`: Documented `"1000"` UID fallback with rationale
- **Benchmark JSON evolution**: 6 JSONs updated with documented threshold fields + rationale strings
- **`#[allow]` → `#[expect]` migration**: All 7 remaining `#[allow]` annotations migrated to `#[expect]` with `reason` parameters. Migration caught 1 stale suppression in `seismic.rs` (lints no longer fire — dead `#[allow]` removed)
- **Named constants**: `SINGULARITY_THRESHOLD` extracted in `regression.rs` (was magic `1e-30`)
- **Specs evolution**: `specs/BARRACUDA_EVOLUTION.md` — added explicit Module → WGSL Shader → Pipeline Stage mapping table (27 entries covering all library modules)
- **Code quality gate**: 0 unsafe, 0 unwrap in library, 0 `#[allow]` annotations (all `#[expect]`), 0 mocks in production, 0 stale lint suppressions, 6 external deps all justified

### Run 32 (V46 Idiomatic Rust Evolution, Feb 28, 2026)

- `cargo fmt --check`: PASS
- `cargo clippy --workspace --all-targets`: PASS (0 warnings)
- `cargo doc --workspace --no-deps`: PASS (0 warnings)
- `cargo test --workspace` (default): 296 unit tests, 292/292 validation checks, all PASS
- **stats/agreement.rs**: Domain split from metrics.rs — 7 paired-observation error/agreement metrics extracted
- **R²/NSE dedup**: `r_squared_cpu` and `nash_sutcliffe_cpu` were identical implementations; now share `coefficient_of_efficiency` helper
- **Iterator modernization**: `level_spacing_ratio_cpu` rewritten from `for i in 0..n-2` to `.windows(3).fold()`
- **Hardcode evolution**: `NESTGATE_DEFAULT_PORT` constant extracted (was magic `8090` in 3 places)
- **Full audit completed**: 0 unsafe, 0 unwrap/expect in library, 0 mocks in production, 0 &String/&Vec params, 0 Box<dyn Error> in public APIs, 0 dead code allows, 6 external deps all justified
- Large files reviewed: regression.rs (401), rare_biosphere.rs (439), biomeos.rs (495) — all domain-focused and cohesive, no artificial split needed
- FAO-56 magic numbers: intentionally inline with equation citations per standards verification practice
- TODO(toadstool) blocks (0): zero remaining

### Run 31 (V45 Validation Gap Closure, Feb 28, 2026)

- `cargo fmt --all -- --check`: PASS
- `cargo clippy --workspace --all-targets`: PASS (0 warnings)
- `cargo doc --workspace --no-deps`: PASS (0 warnings)
- `cargo test --workspace` (default): all PASS
- Validation checks: 292/292 PASS (28 binaries, was 288)
- **Exp 010 Bistable** (+1 check → 10/10): Added Part 6 low-noise agreement — stochastic c-di-GMP with σ=0.01 stays within 0.3 of deterministic attractor
- **Exp 011 Multi-Signal** (+1 check → 9/9): Added Part 4 dual-signal variance advantage — dual-signal σ(c-di-GMP) ≤ 1.5× CAI-1-only σ, confirming signal integration reduces noise
- **Exp 016 Rare Biosphere** (+2 checks → 12/12): Added Spearman ρ(abundance, occupancy) > 0.2 (positive association with rank-tied community) and multinomial determinism (same-seed exact reproducibility)
- **Refactor**: `validate_bistable.rs` extracted `SimCtx` struct + `validate_stochastic()` to stay under clippy `too_many_lines` limit
- Total Rust validation checks: 292 (was 288)

### Run 30 (V44 Deep-Debt Evolution, Feb 28, 2026)

- `cargo fmt --all -- --check`: PASS
- `cargo clippy --workspace --all-targets --features barracuda-gpu`: PASS (0 warnings)
- `cargo test --workspace` (default): all PASS
- `cargo test --workspace --features barracuda`: all PASS
- `cargo test --workspace --features barracuda-gpu`: all PASS
- **New module `linalg`**: Tridiagonal eigensolver (`tridiag_eigh`, `EighError`) extracted from `transport.rs`. Shared by `transport` + `band_structure`. Re-exported from `transport` for backward compat.
- **New module `error`**: `InputError` enum (`LengthMismatch`, `InsufficientData`, `OutOfRange`) with `Display`, full test suite.
- **5 APIs evolved**: `jackknife_mean_variance`, `block_jackknife_variance`, `finite_size_extrapolate`, `chi_squared`, `percentile` — all from `assert!` to `Result<T, InputError>`.
- **Derives added**: `GridFitConfig` +`Debug`/`Clone`/`Copy`; `EighError` +`Clone`/`PartialEq`/`Eq`.
- **Idiomatic cast**: `prng::next_u64` — `as u64` → `u64::from()`.
- **Capability discovery**: Hardcoded `/run/user/1000/` → runtime UID from `$XDG_RUNTIME_DIR` / `$UID` / `/proc/self/status`.
- **New tests**: `std_dev_known_value`, `percentile_out_of_range`, `jackknife_insufficient_data`, `InputError` unit tests, `EighError` derive tests.
- V44 handoff: `wateringHole/handoffs/GROUNDSPRING_TOADSTOOL_V44_DEEP_DEBT_EVOLUTION_HANDOFF_FEB28_2026.md`

### Run 29 (baseCamp Update + NUCLEUS/NestGate/metalForge Extension, Feb 27, 2026)

- `cargo fmt --all -- --check`: PASS
- `cargo clippy --workspace --all-targets --features biomeos -- -D warnings`: PASS (0 warnings)
- `cargo test --workspace --features biomeos`: 498+ tests PASS, 0 failures
- gen3/baseCamp/06_notill_anderson.md: Added Exp 022-024 (ET₀→Anderson propagation, no-till vs tilled 16S, aggregate stability noise)
- gen3/baseCamp/07_sovereign_wdm.md: Added Section 6.3 — WDM uncertainty budget (Exp 025-027: f32/f64 drift, size convergence, vendor parity)
- gen3/baseCamp/README.md: Added Exp 022-024 to expansion paragraph
- groundSpring/whitePaper/baseCamp/anderson.md: Three-tier table updated (CPU tier DONE for Exp 014/016)
- groundSpring/whitePaper/baseCamp/README.md: Cross-Spring Impact table extended (Exp 022-028), Sub-thesis 07 (WDM) added
- New graph: `graphs/groundspring_tower_bootstrap.toml` — Tower atomic (BearDog + Songbird) for Eastgate
- New module: `crates/groundspring/src/nestgate.rs` — NestGate data pipeline (NCBI/NOAA via biomeOS, provenance key schemas, cache-through, 4 tests)
- New module: `metalForge/forge/src/remote.rs` — Remote substrate discovery via biomeOS capability routing (parse, merge, 12 tests)
- Extended: `metalForge/forge/src/inventory.rs` — `merge_remote()` method for NUCLEUS node substrates
- Extended: `biomeos.rs` — public `escape_json_pub()` for sibling modules
- ABSORPTION_MANIFEST.md: Remote substrate discovery marked complete

### Run 28 (V38 Code Quality Evolution, Feb 27, 2026)

- `cargo fmt --all -- --check`: PASS (24 formatting diffs resolved)
- `cargo clippy --workspace --all-targets`: PASS (22 warnings resolved → 0)
- `cargo doc --workspace --no-deps`: PASS
- `cargo test --workspace`: 438/438 PASS
- Validation checks: 288/288 PASS (28 binaries)
- Python baseline integrity: 222/222 PASS
- Clippy fixes: `abs_diff`, `cast_lossless` → `f64::from()`, `mul_add`, bitwise determinism tests
- CI hardened: `--all` for fmt, `--all-targets` for clippy, `--fail-under-lines 90` for coverage
- CI expanded: 6 missing validation binaries added (et0-anderson, notill-sampling, aggregate-stability, precision-drift, size-convergence, vendor-parity)
- Copyright: 10 metalForge `.rs` files now have `Copyright (C) 2026 ecoPrimals / Squirrel Team`
- Tolerances: 8 named constants (`TOL_EXACT` through `TOL_REGIME`) with mathematical justifications; 6 validation binaries updated
- chao1 doc: clarified formula divergence (classic Chao 1984 vs barracuda's bias-corrected Chao & Chiu 2016)
- Delegation audit: 39 active (V42 GPU rewiring), 7 pending ToadStool, 2 GPU ops wired (BatchedMultinomialGpu)

### Run 27 (V30 biomeOS Neural API Integration, Feb 27, 2026)

- `cargo fmt --check`: PASS
- `cargo clippy --workspace --features biomeos`: PASS (0 warnings)
- `cargo test --workspace`: 391/391 PASS (default mode, unchanged)
- `cargo test --workspace --features biomeos`: 423/423 PASS (+22 biomeos unit + 10 biomeos integration)
- Validation checks: 288/288 PASS (28 binaries)
- Python pytest: 322 collected, 320 pass + 2 skip (unchanged)
- New feature: `biomeos` — JSON-RPC 2.0 Unix socket client for biomeOS Neural API
- Anderson biomeOS routing: `validate_anderson` optionally routes through `capability.call("compute.execute")`
- Docs: `whitePaper/neuralAPI/` (concept + capability surface), `graphs/groundspring_validation.toml` (pipeline graph)
- Total: 423 Rust (biomeos) + 322 Python = 745 tests

### Run 26 (V29 Three-Tier Validation Buildout, Feb 27, 2026)

- `cargo fmt --check`: PASS
- `cargo clippy --workspace`: PASS (0 warnings)
- `cargo test --workspace`: 391/391 PASS
- Validation checks: 288/288 PASS (28 binaries)
- Python pytest: 322 collected, 320 pass + 2 skip (250 experiments + 72 three-tier parity)
- Three-tier parity: 23/23 Rust integration tests PASS
- Barracuda delegations: 39 active (30 CPU + 9 GPU), 7 pending ToadStool (V42 GPU rewiring)
- GPU-annotated modules: 8 (freeze_out, band_structure, seismic, quasispecies, rare_biosphere, gillespie, transport, fao56)
- New CPU delegations: drift::kimura_fixation_prob, jackknife::jackknife_mean_variance, fao56::daily_et0

### Run 24 (V26 MetalForge Live Hardware, Feb 27, 2026)

- `cargo fmt --check`: PASS
- `cargo clippy --workspace`: PASS (0 warnings)
- `cargo test --workspace`: 314/314 PASS
- Validation checks: 288/288 PASS (28 binaries)
- MetalForge checks: 31/31 PASS (inventory 10/10, GPU 11/11, cross-substrate 10/10)
- Python pytest: 52/52 PASS (28 experiments)
- Three-mode benchmark: 279/279 × 3 modes = all PASS
- Added: Exp 028 NPU Anderson (9/9), groundspring-forge crate (12 tests), npu module (8 tests)
- Live hardware: Titan V (Volta, native f64 @ 1:2), RTX 4070 (Ada), AKD1000 NPU (80 NPs, ~51µs/inference), i9-12900K

### Run 23 (V25 Experiment Buildout: Exp 025-027, Feb 26, 2026)

- `cargo fmt --check`: PASS
- `cargo clippy --all-targets --all-features -- -D warnings`: PASS (0 warnings)
- `cargo test --workspace`: 302/302 PASS (234 unit + 13 determinism + 14 proptest + 9 validate-lib + 27 integration + 2 doc)
- Validation checks: 279/279 PASS (27 binaries)
- Python pytest: 50/50 PASS (Exp 001-027)
- `ruff check control/ tests/`: PASS (0 warnings)
- Added: Exp 025 f32 vs f64 Precision Drift (7/7), Exp 026 System-size Convergence (7/7), Exp 027 GPU Vendor Parity (7/7)
- New modules: `wdm` (precision_drift, size_convergence, vendor_parity)

### Run 22 (V24 Experiment Buildout: Exp 022-024, Feb 26, 2026)

- `cargo fmt --check`: PASS
- `cargo clippy --all-targets --all-features -- -D warnings`: PASS (0 warnings)
- `cargo test --workspace`: 290/290 PASS (207 unit + 13 determinism + 14 proptest + 9 validate-lib + 24 integration + 1 doc)
- Validation checks: 258/258 PASS (24 binaries)
- Python pytest: 50/50 PASS (Exp 001-024)
- `ruff check control/ tests/`: PASS (0 warnings)
- Added: Exp 022 ET₀ → Anderson Propagation (7/7), Exp 023 No-Till vs Tilled Sampling (7/7), Exp 024 Aggregate Stability Noise (8/8)
- New modules: none (uses fao56, anderson, rarefaction, rare_biosphere, decompose, stats)

### Run 21 (V23 Experiment Buildout: Exp 019-021, Feb 26, 2026)

- `cargo fmt --check`: PASS
- `cargo clippy --all-targets --all-features -- -D warnings`: PASS (0 warnings)
- `cargo test --workspace`: 280/280 PASS (207 unit + 13 determinism + 14 proptest + 9 validate-lib + 21 integration + 1 doc)
- Validation checks: 236/236 PASS (21 binaries)
- Python pytest: 47/47 PASS (Exp 001-021)
- `ruff check control/ tests/`: PASS (0 warnings)
- Added: Exp 019 Jackknife Error Estimation (9/9), Exp 020 Freeze-Out Inverse Problem (8/8), Exp 021 Spectral Function Reconstruction (8/8)
- New modules: `jackknife`, `freeze_out`, `spectral_recon`
- New domain: Inverse Problems & Spectral Reconstruction (Bazavov papers)

### Run 20 (V22 Experiment Buildout: Exp 016-018, Feb 26, 2026)

- `cargo fmt --check`: PASS
- `cargo clippy --all-targets --all-features -- -D warnings`: PASS (0 warnings)
- `cargo test --workspace`: 280/280 PASS (222 unit + 13 determinism + 14 proptest + 9 validate-lib + 21 integration + 1 doc)
- Validation checks: 236/236 PASS (21 binaries)
- Python pytest: 21/21 PASS (Exp 001-021)
- `ruff check control/ tests/`: PASS (0 warnings)
- Added: Exp 016 Rare Biosphere (10/10), Exp 017 Quasispecies Threshold (6/6), Exp 018 Band Edge Structure (10/10)
- New modules: `rare_biosphere`, `quasispecies`, `band_structure`
- Pre-existing clippy warnings cleaned: cfg gates for barracuda-gpu dead code, float_cmp in determinism tests, mul_add in transport

### Run 19 (V21 Complete Barracuda Rewiring + Dual-Mode CI, Feb 26, 2026)

- **Dual-mode validation**: CI now runs `cargo clippy` and `cargo test` both with and without `--features barracuda`. 225/225 tests pass in both CPU-only and barracuda-delegated modes.
- `--features barracuda` compiles cleanly (zero warnings both modes).
- Domain guard fix for hill: biological convention applied before delegation.
- 17 `_cpu` functions properly gated behind `#[cfg(not(feature = "barracuda"))]`.
- CPU delegation overhead: +1.7% total.

### Run 18 (V19 Uncertainty Bridge, Feb 26, 2026)

- `cargo fmt --check`: PASS
- `cargo clippy --workspace -- -D warnings`: PASS (0 warnings)
- `cargo doc --workspace --no-deps`: PASS
- `cargo test --workspace`: 225/225 PASS (173 unit + 13 determinism + 14 proptest + 9 validate-lib + 15 integration + 1 doc/unused)
- Validation checks: 185/185 PASS (15 binaries)
- `cargo llvm-cov`: 99.37% line coverage
- Python pytest: 37/37 PASS (Exp 001-015)
- Added: Exp 015 Uncertainty Bridge (8/8 PASS), validate_uncertainty_bridge binary
- Zero `#[allow]` remaining (transport.rs fix)

### Run 17 (V18 Deep Debt Evolution, Feb 26, 2026)

- `cargo fmt --check`: PASS
- `cargo clippy --workspace -- -D warnings`: PASS (0 warnings)
- `cargo doc --workspace --no-deps`: PASS
- `cargo test --workspace`: 225/225 PASS (173 unit + 13 determinism + 14 proptest + 9 validate-lib + 15 integration + 1 doc/unused)
- Validation checks: 177/177 PASS (14 binaries)
- `cargo llvm-cov`: 98.94% line coverage
- Python pytest: 37/37 PASS (Exp 001-011)
- Added: kinetics module, flat buffers, 13 determinism tests, DOIs, CI completeness

### Run 16 — February 26, 2026 (V16 ToadStool S66 catch-up + rewiring)

```
ToadStool S66 review: 2,541 tests, 707 WGSL shaders, sovereign compiler.
  V44 is current. ToadStool pinned at S68+ (e96576ee).
  S66 absorbed rawr_mean from V15 request.

New delegation #26: rawr_mean → barracuda::stats::rawr_mean (CPU)
  Total: 26 active delegations (21 CPU + 5 GPU)

Test fix: bootstrap_different_from_rawr and validate_rawr RAWR comparison
  updated for barracuda parity (compare CI widths instead of exact estimates).

Three-mode revalidation:
  default:       205/205 tests PASS, 177/177 checks PASS
  barracuda:     205/205 tests PASS, 177/177 checks PASS
  barracuda-gpu: 205/205 tests PASS, 177/177 checks PASS
  clippy:        0 warnings × 3 modes

New S66 capabilities documented (not yet wired):
  WrightFisherGpu, eigh_f64, stats::regression, stats::hydrology,
  stats::moving_window_f64, stats::mae
```

### Run 15 — February 26, 2026 (V15 Experiment Buildout: Exp 012–014)

```
3 new experiments built:
  Exp 012: Spin Chain Transport (Kachkovskiy 2016)    18/18 PASS  transport.rs
  Exp 013: Resampling Convergence (Lee & Liu 2024)    8/8  PASS  bootstrap
  Exp 014: Drift vs Selection (R. Anderson 2022)       7/7  PASS  drift.rs

New modules:
  transport   tridiag_eigh, wavepacket_msd, transport_exponent
  drift       wright_fisher_fixation, kimura_fixation_prob, neutral_diversity_trajectory

prng::binomial   Added for Wright-Fisher sampling

Totals:
  15 experiments, 185/185 validation checks
  205 Rust tests (167 unit + 14 proptest + 9 validate-lib + 14 integration + 1 doc)
  15 validation binaries
  Mathematical parity: 15/15 PROVEN (Python ⇌ Rust)

Paper queue: Papers #13, #17, #20 moved Queued → Active
```

### Run 14 — February 26, 2026 (V14 S65 revalidation + cross-spring documentation)

```
New delegation #25: evenness → barracuda::stats::pielou_evenness
  S≤1 semantic adapter (groundSpring returns 1.0, barracuda returns 0.0)
  Total: 25 active delegations (20 CPU + 5 GPU)

Code quality:
  anderson.rs → almost_mathieu.rs split (594 → 264 + 329 lines)
  stats/correlation.rs modernized (CPU always compiled)
  Python: 14 ruff errors fixed (zip(strict=True), unused vars)
  Python linting: zero-warning

Three-mode benchmark (release, single pass):
  Binary                   Local(ms)  Barracuda(ms)  Barra-GPU(ms)
  validate_decompose             82           71            560
  validate_rarefaction            70           99            102
  validate_seismic              141          128            171
  validate_weather                65           71             97
  validate_fao56                  79           80            106
  validate_signal_specificity    854          858            898
  validate_rawr                  619          625            651
  validate_anderson              745          745            774
  validate_quasiperiodic      11986        11867            242
  validate_bistable              167          222            207
  validate_multisignal            85          118            118
  TOTAL                       14893        14884           3926

Three-mode validation:
  190/190 Rust tests PASS × 3 modes
  144/144 validation checks × 3 modes
  0 clippy warnings × 3 modes
  37/37 Python tests PASS

New artifacts:
  whitePaper/CROSS_SPRING_EVOLUTION.md   Cross-spring lineage for all 25 delegations
  scripts/regenerate_benchmarks.sh       Benchmark drift guard
  scripts/three_mode_benchmark.sh        Automated three-mode timing

Handoff V14 posted (V13 archived)
```

### Run 11 — February 26, 2026 (Full-suite parity + benchmarks)

```
Benchmark expansion:
  bench_rust_vs_python.py     3 → 11 experiments (full suite)
  bench_barracuda_modes.sh    8 → 11 binaries (full suite)
  run_all_baselines.sh        8+8 → 11+11 experiments (Python + Rust)

New scripts:
  parity_report.py            Formal Python⇌Rust parity certificate
  data/parity_report.json     Machine-readable parity certificate
  data/bench_rust_vs_python.json  Updated with all 11 experiments

Parity certificate:
  11/11 experiments: PARITY PROVEN
  Python baselines + Rust validation both pass against same benchmark JSONs
  Python checks: ~129    Rust checks: 144/144

Performance (median of 3 trials):
  10/11 experiments: Rust 1.8×–63.6× faster than Python
  1/11 (Exp 009):   custom QR vs LAPACK — parity proven, LAPACK faster
  Total (excl. LAPACK-bound): 23.4× Rust speedup
```

### Run 10 — February 25, 2026 (Exp 009–011: quasiperiodic, bistable, multisignal)

```
Phase 0 (Python):
  Exp 009: Almost-Mathieu Quasiperiodic    8/8  PASS
  Exp 010: Bistable Phenotypic Switching  10/10 PASS
  Exp 011: Multi-Signal QS Integration   9/9  PASS

Phase 1 (Rust):
  validate_quasiperiodic                  8/8  PASS
  validate_bistable                      9/9  PASS
  validate_multisignal                   8/8  PASS

New experiments:
  control/quasiperiodic/                  Almost-Mathieu Hamiltonian
  control/bistable_switching/             BistableOde phenotypic switching
  control/multisignal_qs/                MultiSignalOde QS integration

Barracuda delegations (+3):
  almost_mathieu_hamiltonian              barracuda-gpu (Exp 009)
  BistableOde::cpu_derivative           barracuda (Exp 010)
  MultiSignalOde::cpu_derivative        barracuda (Exp 011)

Totals:
  11 experiments, 144/144 validation checks
  Rust tests: 177 (153 lib + 9 validate-lib + 14 proptest + 11 integration + 1 doc)
  Python checks: ~129
  Barracuda delegations: 14
```

### Run 7 — February 25, 2026 (Deep debt resolution & sovereignty evolution)

```
Phase 1 (Rust) — local mode:
  8/8 binaries, 119/119 PASS

Sovereignty:
  error_propagation_fao56.py    capability-based discovery (no hardcoded primal names)
  test_experiments.py           capability scan for FAO-56 skip check

BarraCUDA error handling:
  All 11 delegations             .expect() / .unwrap_or() → if let Ok + CPU fallback
  CPU fallbacks                  always compiled (no #[cfg(not(feature))] guard)

Shared validation helpers (DRY):
  groundspring-validate lib.rs   f64_field, usize_field, u64_field, f64_range, print_provenance_header
  9 unit tests for validate-lib  (was 0% coverage)

Validation refactoring:
  validate_seismic               SourceTruth + AcceptanceCriteria structs
  validate_fao56                 Uncertainties struct, split run()
  validate_rawr                  validate_gaussian/skewed/correlated/determinism
  validate_signal_specificity    EnzymeNetwork + SimConfig structs, split run()

Dead code removal:
  control/common.py              write_benchmark(), provenance_metadata() removed (unused)

Clippy: 0 warnings
Rust tests: 163/163 PASS (131 unit + 9 validate-lib + 14 proptest + 8 integration + 1 doc)
Python tests: 34/34 PASS
Coverage: 99.11% (cargo-llvm-cov)
```

### Run 9 — February 25, 2026 (Complete rewiring + benchmarks + cross-spring lineage)

```
Complete barracuda API audit:
  All CPU-accessible functions reviewed
  11 delegations confirmed as the complete set
  6 remaining metrics (rmse, mbe, r², IoA, hit_rate, shannon) require WgpuDevice
  No new CPU-only primitives available to wire

Three-mode benchmarks (release, best-of-3):
  Binary                   Local(ms)  BarraCUDA(ms)  BarraCUDA-GPU(ms)
  validate_anderson            671         670             640
  validate_decompose             5           4               5
  validate_fao56                12          12              13
  validate_rarefaction          11          12              12
  validate_rawr                555         560             556
  validate_seismic              56          59              58
  validate_signal_specificity  795         787             787
  validate_weather               3           3               5
  TOTAL                       2108        2107            2076
  Overhead: ~0% (compute-heavy <1%, signal-spec -1%, anderson -5%)

Cross-spring lineage documented:
  hotSpring → precision (df64_core, spectral/anderson, sum_reduce_f64)
  wetSpring → bio-stats (FusedMapReduce, Gillespie, log_f64 fix, ridge)
  neuralSpring → ML/dispatch (spectral_density, domain_ops, xoshiro)

Validation (all three modes):
  163/163 Rust tests PASS × 3 modes
  119/119 validation checks × 3 modes
  0 clippy warnings × 3 modes
  34/34 Python tests PASS

Handoff V9 posted (V8 archived)
```

### Run 8 — February 25, 2026 (ToadStool catch-up revalidation)

```
ToadStool baseline: S50–S62 + DF64 expansion (Feb 23-24, 2026)
  14,200+ tests, 650+ WGSL shaders, shader-first architecture

Review findings:
  No new CPU stats primitives added since our S62 baseline
  Our 11 delegations remain current and complete
  ToadStool has since absorbed both shaders (batched_multinomial S76, mc_et0_propagate S72)

Code fix:
  correlation.rs  3× needless_return in barracuda cfg blocks → removed

Three-mode validation:
  Local:          163/163 PASS, 0 clippy warnings
  Barracuda:      163/163 PASS, 0 clippy warnings
  Barracuda-GPU:  163/163 PASS, 0 clippy warnings
```

### Run 6 — February 25, 2026 (Complete rewiring + benchmarks)

```
Phase 1 (Rust) — local mode:
  8/8 binaries, 119/119 PASS

Phase 1 (Rust) — barracuda-gpu mode (11 delegated):
  8/8 binaries, 119/119 PASS

New delegations wired (5 new):
  stats::covariance          → barracuda::stats::correlation::covariance
  stats::norm_cdf            → barracuda::stats::norm_cdf
  stats::norm_ppf            → barracuda::stats::norm_ppf
  stats::chi2_statistic      → barracuda::stats::chi2_decomposed
  anderson::analytical_ξ     → barracuda::special::anderson_transport::localization_length

Rust tests: 154/154 PASS (131 unit + 14 proptest + 8 integration + 1 doc)
Clippy: 0 warnings (pedantic + nursery)

Benchmarks (best-of-3, release mode):
  Local total:          2573 ms
  Barracuda-GPU total:  2721 ms (+6%)
  Compute-heavy delta:  <2% overhead (signal-specificity, RAWR, anderson)
```

### Run 5 — February 25, 2026 (ToadStool catch-up revalidation)

```
Phase 1 (Rust) — local mode:
  8/8 binaries, 119/119 PASS

Phase 1 (Rust) — barracuda-gpu mode:
  8/8 binaries, 119/119 PASS (11 delegated functions, all correct)

ToadStool baseline: S62 + DF64 expansion (Feb 24-25, 2026)
  S59: anderson_3d_correlated, sweep_averaged, find_w_c, ridge_regression
  S60-61: cpu-math feature gate, SpMM, TransE
  S62: BandwidthTier, PeakDetectF64
  Post-S62: DF64 core-streaming, ComputeDispatch builder

Verified:
  cargo test --features barracuda-gpu     154/154 PASS (131 unit + 14 proptest + 8 integration + 1 doc)
  cargo clippy --features barracuda-gpu   0 warnings (pedantic + nursery)
  barracuda has bootstrap_mean_f64.wgsl   GPU path available
```

### Run 4 — February 25, 2026 (Code audit & deep debt resolution)

```
Phase 1 (Rust):
  validate_decompose                 36/36 PASS
  validate_rarefaction               15/15 PASS
  validate_seismic                    9/9  PASS
  validate_weather                   13/13 PASS
  validate_fao56                     15/15 PASS
  validate_signal_specificity        12/12 PASS
  validate_rawr                      11/11 PASS
  validate_anderson                   8/8  PASS

Fixes:
  barracuda::spectral::anderson::*  → barracuda::spectral::* (E0603 fix)
  cargo fmt                          6 files reformatted
  cargo clippy (pedantic + nursery)  0 warnings (was 3: too_many_lines × 3)
  bootstrap sort                     partial_cmp().unwrap_or() → f64::total_cmp
  validate_rawr generate_normal      duplicate Box-Muller → library Xorshift64::normal()
  bootstrap percentile_ci            extracted shared helper (DRY)
  validate_anderson main()           extracted disorder_sweep() + thouless_and_localization()
  validate_fao56 main()              extracted validate_monte_carlo() + validate_sensitivity()
  validate_seismic main()            extracted validate_forward_model() + validate_inversion()
  control/common.py                  added provenance_metadata() + write_benchmark()
  phantom bootstrap_mean_f64.wgsl    removed from README + whitePaper
```

### Run 3 — February 25, 2026 (Paper queue experiment buildout)

```
Phase 0 (Python):
  Exp 001: Sensor Noise              32/32 PASS
  Exp 002: Observation Gap            PASS (synthetic SKIP)
  Exp 003: Error Propagation          PASS
  Exp 004: Sequencing Noise           PASS
  Exp 005: Seismic Inversion          PASS
  Exp 006: Signal Specificity        12/12 PASS
  Exp 007: RAWR Resampling           11/11 PASS
  Exp 008: Anderson Localization      8/8  PASS

Phase 1 (Rust):
  validate_decompose                 36/36 PASS
  validate_rarefaction               15/15 PASS
  validate_seismic                    9/9  PASS
  validate_weather                   13/13 PASS
  validate_fao56                     15/15 PASS
  validate_signal_specificity        12/12 PASS
  validate_rawr                      11/11 PASS
  validate_anderson                   8/8  PASS

pytest:
  test_common                        18/18 PASS
  test_determinism                    7/7  PASS
  test_experiments                    8/8  PASS (3 new)
```

### Run 2 — February 25, 2026 (Phase 1 port)

```
Phase 0 (Python):
  Exp 001: Sensor Noise              32/32 PASS
  Exp 002: Observation Gap            PASS (synthetic SKIP)
  Exp 003: Error Propagation          PASS
  Exp 004: Sequencing Noise           PASS
  Exp 005: Seismic Inversion          PASS

Phase 1 (Rust):
  validate_decompose                 36/36 PASS
  validate_rarefaction               15/15 PASS
  validate_seismic                    9/9  PASS
  validate_weather                   13/13 PASS
  validate_fao56                     15/15 PASS

pytest:
  test_common                        18/18 PASS
  test_determinism                    7/7  PASS
  test_experiments                    5/5  PASS
```

### Run 1 — February 16, 2026 (initial baselines)

```
Exp 001–005: 71/71 PASS (Python only)
```

