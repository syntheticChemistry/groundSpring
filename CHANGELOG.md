# Changelog

All notable changes to groundSpring follow [Keep a Changelog](https://keepachangelog.com/).

## [Unreleased]

### Complete BarraCUDA Rewiring (Phase 2a complete)
- **4 new CPU delegations**: `covariance`, `norm_cdf`, `norm_ppf`,
  `chi2_statistic`, `analytical_localization_length` — total **11 active**
- **New library functions**: `stats::covariance` (sample covariance),
  `stats::norm_cdf` (Φ(x) via A&S 7.1.26 erf), `stats::norm_ppf`
  (Φ⁻¹(p) via Acklam rational approximation), `stats::chi2_statistic`
  (Σ(O−E)²/E goodness-of-fit), `anderson::analytical_localization_length`
  (perturbative ξ(W,E) from hotSpring transport theory)
- **All have local implementations** + barracuda feature-gated delegation
- **Benchmarks**: 3-trial release-mode timing confirms <2% overhead for
  compute-heavy binaries (signal-specificity, RAWR, anderson)
- **Cross-spring evolution** documented in `specs/CROSS_SPRING_EVOLUTION.md`:
  traces provenance of every barracuda primitive groundSpring uses back to
  its origin spring (hotSpring precision, wetSpring bio, neuralSpring ML)
- **154 Rust tests** (131 unit + 14 proptest + 8 integration + 1 doc) + 119/119 validation checks PASS across local,
  barracuda, and barracuda-gpu modes
- Added `scripts/bench_barracuda_modes.sh` for reproducible benchmarking

### ToadStool Catch-Up Revalidation (Phase 2a++)
- **ToadStool baseline**: Verified against S62 + post-S62 DF64 expansion (Feb 25, 2026)
- **All 6 (later expanded to 11) barracuda delegations verified**: 119/119 PASS with `--features barracuda-gpu`
  - `pearson_r`, `spearman_r`, `sample_std_dev` via `barracuda::stats`
  - `bootstrap_mean` via `barracuda::stats::bootstrap_mean`
  - `lyapunov_exponent`, `lyapunov_averaged` via `barracuda::spectral`
- **New barracuda ops cataloged** (S59–S62): `anderson_3d_correlated`,
  `anderson_sweep_averaged`, `find_w_c`, `ridge_regression`, `PeakDetectF64`,
  `BandwidthTier` — available for future Kachkovskiy extension experiments
- **Noted**: barracuda has `bootstrap_mean_f64.wgsl` GPU shader (65 lines,
  xorshift32 PRNG, workgroup_size(256)) — GPU path for bootstrap exists
- Updated `specs/BARRACUDA_EVOLUTION.md` with S59–S62+ new ops table
- Updated V5 handoff with ToadStool catch-up section
- Updated `CONTROL_EXPERIMENT_STATUS.md` with Run 5 revalidation log

### Code Audit & Deep Debt Resolution (Phase 2a+)
- **CRITICAL FIX**: `barracuda::spectral::anderson::lyapunov_*` → `barracuda::spectral::lyapunov_*`
  (anderson submodule is private in barracuda; items re-exported at spectral level)
- **Idiomatic Rust**: `partial_cmp().unwrap_or()` → `f64::total_cmp` in bootstrap sort
- **DRY**: Extracted shared `percentile_ci()` helper in bootstrap module — both
  `bootstrap_mean_local` and `rawr_mean` delegate to it (was duplicated sort + CI code)
- **DRY**: `validate_rawr` `generate_normal` replaced with `Xorshift64::normal()` from library
  (was duplicate Box-Muller implementation)
- **Refactored**: validate_anderson, validate_fao56, validate_seismic — extracted helper
  functions from long `main()`, removed all `#[allow(clippy::too_many_lines)]`
- **Documentation**: Fixed phantom `bootstrap_mean_f64.wgsl` references in README and
  whitePaper (shader never existed; bootstrap delegates to barracuda CPU)
- **Documentation**: Added tolerance justification comments across validation binaries
- **Documentation**: Anderson seed divergence documented (barracuda `base_seed + r * 1000`
  vs local `base_seed + i` — Phase 2b alignment work)
- **Documentation**: validate_weather module doc clarified as analytical-only validator
- **Documentation**: `McResult` struct fields now documented in validate_fao56
- **Provenance**: Added `provenance_metadata()` and `write_benchmark()` to `control/common.py`
  for reproducible benchmark JSON generation with git commit + date
- **RAWR citation**: `30.0` fallback in bootstrap.rs documented as `-ln(9.4e-14)` guard
- `cargo fmt` clean (was failing in 6 files)
- `cargo clippy --all-targets -W pedantic -W nursery` zero warnings
- Workspace line coverage: 98.64% (cargo-llvm-cov)

### Barracuda CPU Delegation & Performance Benchmarks (Phase 2a)
- **Barracuda CPU delegation**: Wired 3 new functions to barracuda upstream:
  - `bootstrap::bootstrap_mean` → `barracuda::stats::bootstrap_mean` (`#[cfg(feature = "barracuda")]`)
  - `anderson::lyapunov_exponent` → `barracuda::spectral::lyapunov_exponent` (`#[cfg(feature = "barracuda-gpu")]`)
  - `anderson::lyapunov_averaged` → `barracuda::spectral::lyapunov_averaged` (`#[cfg(feature = "barracuda-gpu")]`)
- **New feature gate**: `barracuda-gpu` enables `barracuda/gpu` for spectral module access
- **Performance benchmarks**: `scripts/bench_rust_vs_python.py` — times Python vs Rust
  - Signal Specificity: **30.9×** faster
  - RAWR Resampling: **7.3×** faster
  - Anderson Localization: **29.8×** faster
  - **Total: 24.0× faster** (52.0s Python → 2.17s Rust)
- Updated `metalForge/ABSORPTION_MANIFEST.md` — 6 delegated (was 3)
- Updated `specs/BARRACUDA_EVOLUTION.md` — performance section, new module mappings
- Updated `CONTROL_EXPERIMENT_STATUS.md` — delegation and performance tables

### Paper Queue Experiment Buildout (Phase 1c)
- **Exp 006: Enzymatic Signal Specificity** — Gillespie SSA of c-di-GMP
  birth-death process (Massie et al. 2012 PNAS). 12/12 Python, 12/12 Rust.
  New `gillespie` module with `birth_death_ssa`, `steady_state_mean`,
  time-averaged mean/variance.  5 unit tests.
- **Exp 007: RAWR Resampling** — Standard bootstrap vs RAWR (Bayesian
  bootstrap) on Gaussian, skewed, and correlated test cases (Wang et al. 2021
  Bioinformatics/ISMB). 11/11 Python, 11/11 Rust.  New `bootstrap` module
  with `bootstrap_mean`, `rawr_mean`.  6 unit tests.
- **Exp 008: Anderson Localization** — Lyapunov exponents via transfer
  matrix method for 1D Anderson tight-binding model (Anderson 1958,
  Bourgain-Kachkovskiy 2018).  8/8 Python, 8/8 Rust.  New `anderson` module
  with `anderson_potential`, `lyapunov_exponent`, `localization_length`,
  `lyapunov_averaged`.  7 unit tests.
- Three new Python baselines: `control/signal_specificity/`,
  `control/rawr_resampling/`, `control/anderson_localization/`
- Three new Rust validation binaries: `validate-signal-specificity`,
  `validate-rawr`, `validate-anderson`
- Three new benchmark JSONs with full provenance and analytical predictions
- `tests/test_experiments.py` — 3 new integration tests (Exp 006-008)
- `scripts/run_all_baselines.sh` — 6 new entries (3 Python + 3 Rust)
- **Total**: 119/119 validation checks across 8 binaries, 154 Rust tests (131 unit + 14 proptest + 8 integration + 1 doc)

### Added
- `fao56` module — complete FAO-56 Penman-Monteith equation chain (Exp 003)
- `prng` module — deterministic Xorshift64 with Box-Muller normal sampling
- `stats::hit_rate` — precipitation occurrence agreement metric (Exp 002)
- `validate-weather` binary — Exp 002 weather model-observation gap validation
- `validate-fao56` binary — Exp 003 FAO-56 error propagation with Monte Carlo
- `barracuda` feature gate in `Cargo.toml` with working Tier A infrastructure
- `stats::pearson_r` — Pearson correlation coefficient, delegates to
  `barracuda::stats::pearson_correlation` when barracuda feature is enabled
- `stats::spearman_r` — Spearman rank correlation coefficient with tie handling,
  delegates to `barracuda::stats::correlation::spearman_correlation` when
  barracuda feature is enabled
- `stats::sample_std_dev` — Bessel-corrected sample std dev; delegates to
  `barracuda::stats::correlation::std_dev` when barracuda feature is enabled
- Comprehensive edge-case tests for `stats`, `rarefaction`, `validate` modules
- Determinism tests for `fao56`, `prng`, `rarefaction`
- Bitwise determinism test for Box-Muller normal variate (`prng::normal_deterministic_bitwise`)
- `serde_json` dependency for data-driven validation binaries
- `metalForge/` — Write → Absorb → Lean artifacts following hotSpring pattern
- `metalForge/ABSORPTION_MANIFEST.md` — module-by-module absorption inventory
- `metalForge/shaders/mc_et0_propagate.wgsl` — Tier C Monte Carlo FAO-56 kernel
- `metalForge/shaders/batched_multinomial.wgsl` — Tier C rarefaction kernel

### metalForge Evolution (following hotSpring)
- **`sample_std_dev` → barracuda CPU delegation**: `stats::sample_std_dev` now
  delegates to `barracuda::stats::correlation::std_dev` when the `barracuda`
  feature is enabled (joining `pearson_r` and `spearman_r` as leaned stats)
- **`batched_multinomial.wgsl` → production**: Evolved from commented pseudocode
  to 112-line production WGSL with xoshiro128** PRNG, binary search over
  cumulative probabilities, per-replicate state management, and uniform struct
  bindings matching barracuda conventions
- **`mc_et0_propagate.wgsl` → production**: Evolved from partially commented
  prototype to 149-line production WGSL with Box-Muller normal perturbation,
  full FAO-56 equation chain (noted as superseded by `Op::Fao56Et0` — MC
  wrapper remains valuable), and xoshiro PRNG matching barracuda
- **ABSORPTION_MANIFEST.md**: Complete rewrite with post-S62 status — 11 CPU
  leaned, 6 GPU pending adapter, 1 absorbed upstream, 2 WGSL production ready,
  full WGSL inventory with line counts and binding tables, and handoff checklist
- **metalForge/README.md**: Updated with current status table, WGSL conventions
  matching hotSpring pattern, and barracuda lean progress

### Deep Debt Evolution
- **`ValidationHarness` → generic `Write`**: Harness now writes to any
  `impl Write` destination via `new(name, writer)`.  `stdout(name)` provides
  the common case.  Output capture tests verify labels and totals.
- **Centralized cast helpers**: `cast::usize_f64`, `cast::u64_f64`,
  `cast::f64_usize` document the safety argument once, replacing 40+
  scattered `as f64` / `as usize` casts throughout `stats`, `rarefaction`,
  `seismic`, and `prng` modules.
- **Tightened workspace lints**: Removed blanket `cast_precision_loss`,
  `cast_possible_truncation`, `cast_sign_loss` allows from `Cargo.toml`.
  Cast lints are now targeted via `#[expect]` on individual statements in
  validation binaries.
- **`DailyWeatherInputs` → `Copy`**: Struct contains only `f64` fields;
  deriving `Copy` eliminates 4 `.clone()` allocations in the MC hot loop
  and sensitivity analysis (struct update now uses `..* base`).
- **`#[allow]` → `#[expect]`**: All remaining cast annotations in
  validation binaries evolved to `#[expect]` (warns if suppression
  becomes unnecessary).

### Changed
- **Data-driven validation:** all 5 validation binaries now load expected values
  from benchmark JSONs via `include_str!` + `serde_json` — single source of truth,
  eliminating hardcoded duplication between code and JSON provenance files
- `validate` module rewritten from global `AtomicU32` state to struct-based `ValidationHarness`
- All validation binaries migrated to `ValidationHarness` API
- `rarefaction::multinomial_sample` now uses shared `prng::Xorshift64` (identical output)
- `seismic::grid_search_inversion` — hoisted Vec allocation out of hot loop
- `missing_docs` lint promoted from `warn` to `deny`
- Benchmark JSON provenance updated with real commit SHA
- `specs/BARRACUDA_EVOLUTION.md` — major rewrite with Write→Absorb→Lean cycle, Tier A/B/C mapping
- `specs/BARRACUDA_REQUIREMENTS.md` — updated with fao56/prng modules and metalForge status
- `whitePaper/` — Phase 1 Rust results, methodology, and GPU evolution sections
- Root README and CONTRIBUTING updated with metalForge, new modules, and validation totals
- Modern idiomatic Rust: `f64::total_cmp` replaces `partial_cmp().unwrap_or(Equal)` (5 sites)
- Modern idiomatic Rust: `f64::midpoint` used throughout fao56 and validation
- Modern idiomatic Rust: `f64::from()` instead of lossy `as f64` casts in prng tests
- Modern idiomatic Rust: `.hypot()` instead of manual `.powi(2).sqrt()` in decompose
- Modern idiomatic Rust: `.mul_add()` for fused multiply-add where appropriate

### Fixed
- Clippy pedantic + nursery: zero errors, zero warnings
- `validate_seismic` — replaced `unwrap()` with `unwrap_or(Ordering::Equal)` for NaN safety
- Station code mismatch between `benchmark_seismic.json` ("WCI") and Rust binary
- Benchmark JSON `baseline_commit` fields updated from placeholder "initial" to real SHA
- `validate.rs` test no longer triggers `approx_constant` lint (used non-pi values)
- Coverage claim clarified: 99.7% is library-only (validation binaries are separate executables)

### Provenance
- All 5 benchmark JSONs now include DOI/references, `data_origin` field,
  `prng_algorithm` notation, and `real_data_accession` status for pending
  real-data phases
- `wateringHole/handoffs/GROUNDSPRING_TOADSTOOL_V3_FEB25_2026.md` — ToadStool
  catch-up handoff documenting S51-S62 absorption wave: FAO-56 superseded,
  Shannon entropy GPU-ready, population variance resolved, spearman_r wired,
  5 bio ODEs + NMF available. V1/V2 archived.
- `specs/BARRACUDA_EVOLUTION.md` — major update: Tier C FAO-56 marked
  superseded, new barracuda ops table, Phase 2a updated to 2 CPU wired

### Documentation
- `whitePaper/baseCamp/` created — 5 per-faculty research briefings
  (Bazavov, Waters, Liu, Kachkovskiy, R. Anderson) following wetSpring pattern
- `specs/PAPER_REVIEW_QUEUE.md` — added open data provenance audit (all 24
  papers use open data), three-tier control matrix (CPU/GPU/metalForge),
  barracuda kernel requirements summary updated for ToadStool S62 status
- `CONTROL_EXPERIMENT_STATUS.md` — three-tier control matrix,
  barracuda integration status updated post-S62, handoff V3 reference
- All docs synchronized to 154 Rust tests + 34 Python tests
- Root README directory structure updated with baseCamp/ and paper queue details
