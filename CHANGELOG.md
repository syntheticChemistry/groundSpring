# Changelog

All notable changes to groundSpring follow [Keep a Changelog](https://keepachangelog.com/).

## [Unreleased]

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
- **ABSORPTION_MANIFEST.md**: Complete rewrite with post-S62 status — 3 CPU
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
- All docs synchronized to 90 unit tests + 1 doc test
- Root README directory structure updated with baseCamp/ and paper queue details
