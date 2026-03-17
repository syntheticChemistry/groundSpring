# Changelog

All notable changes to groundSpring follow [Keep a Changelog](https://keepachangelog.com/).

## [Unreleased]

### V113 Ecosystem Absorption + Resilience (Mar 16, 2026)

#### GemmF64 Transpose Delegation (barraCuda v0.3.5)
- `spectral_recon::tikhonov_solve` now delegates `KᵀK` and `KᵀG` matrix setup
  to `barracuda::ops::linalg::GemmF64::execute_gemm_ex` with `trans_a=true`
  when GPU is available
- CPU fallback via local `mat_transpose_mul` / `mat_transpose_vec` retained
- Eliminates P1 GemmF64 transpose gap identified in V112 handoff

#### Exit Code Constants (sweetGrass v0.7.19 Pattern)
- `exit_code` module: `SUCCESS` (0), `GENERAL_ERROR` (1), `CONFIG_ERROR` (78),
  `NETWORK_ERROR` (76) per UNIBIN_ARCHITECTURE_STANDARD
- `OrExit<T>` now uses `exit_code::GENERAL_ERROR` instead of hardcoded `1`

#### IPC Resilience (petalTongue v1.6.6 / rhizoCrypt v0.13 Pattern)
- `RetryPolicy` — exponential backoff with configurable max retries, delay cap
- `CircuitBreaker` — Closed/Open/HalfOpen states with failure threshold and cooldown
- 8 unit tests for retry and circuit breaker

#### 4-Format Capability Parsing (airSpring V0.8.7 Pattern)
- `extract_capabilities` now handles `{"result": [...]}` wrapper (JSON-RPC result)
- Recursive unwrapping for `{"capabilities": {"capabilities": [...]}}` double-nesting
- 3 new tests (result wrapper, double-nested, result with objects)

#### Deep Debt
- `#[allow(dead_code)]` → `#[expect(reason)]` in `protocol.rs` (DispatchOutcome)
- Hardcoded `"biomeos"` in `niche.rs` → `crate::primal_names::BIOMEOS`
- `BenchFieldError` to `thiserror::Error` (V112 continuation)

#### Quality Gates
- 618 unit tests + 24 integration tests pass, 0 clippy warnings, 0 fmt diff

### V112 Ecosystem Absorption + OrExit (Mar 16, 2026)

#### `OrExit<T>` Trait (wetSpring V123 / healthSpring V31 Pattern)
- New `OrExit<T>` trait in `groundspring-validate` — `.or_exit(msg)` on `Result` and `Option`
- `parse_benchmark()` helper replaces repeated `let Ok(bench) = serde_json::from_str else { eprintln!; return 1 }` boilerplate
- 28 validation binaries migrated to `parse_benchmark()`; dead `serde_json::Value` imports and stale `#[expect]` attributes cleaned
- `BenchFieldError` evolved from manual `Display`+`Error` impls to `thiserror::Error` derive

#### Generic Primal Discovery (sweetGrass v0.7.18 Pattern)
- `socket_env_var(primal_name)` → `"{UPPER_NAME}_SOCKET"` — generic env var construction
- `address_env_var(primal_name)` → `"{UPPER_NAME}_ADDRESS"` — generic address env var
- Tests for both helpers

#### Provenance Trio
- Added `RHIZOCRYPT`, `LOAMSPINE`, `SWEETGRASS` constants to `primal_names.rs`

#### Infrastructure
- `groundspring-validate` now uses `default-features = false` for groundspring dependency — decouples from barracuda compilation for clippy
- `thiserror = "2"` added to validate crate
- Hardcoded `/tmp/test.sock` in `server.rs` tests replaced with `tempfile::tempdir()`
- metalForge cast evolution: stale `#[expect]` cleaned for casts that don't trigger on 64-bit

#### Quality Gates
- 605 unit tests + 24 integration tests pass, 0 clippy warnings, 0 fmt diff
- Validate crate: 0 clippy warnings with `-D warnings`

### V110 Cross-Ecosystem Absorption (Mar 16, 2026)

#### `#[expect(reason)]` Migration (wetSpring V122 Pattern)
- All 95 `#[allow(clippy::unwrap_used, clippy::expect_used)]` replaced with
  `#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test assertions use unwrap/expect for clarity")]`
- Covers 83 crate files + 12 metalForge files
- Zero `#[allow()]` remaining in entire codebase — stale suppressions now
  produce compile warnings

#### Python Tolerance Mirror (healthSpring V29 Pattern)
- Created `control/tolerances.py` with all 28 constants from `tol::*` (13 tiers),
  `tolerances.rs` (validation-specific), and `eps::*` (epsilon guards)
- `control/common.py` now re-exports from `tolerances.py` for backward compatibility
- Includes validation-specific constants: `TOL_RAREFACTION_PROP`, `TOL_REGIME`,
  `TOL_GRID_MATCH`, `TOL_MONOTONIC_SLACK`, `TOL_ET0`, `THRESHOLD_GOOD_R2`,
  `THRESHOLD_GOOD_IA`, `THRESHOLD_LARGE_GAMMA`, `ET0_PLAUSIBLE_MIN_MM`,
  `ET0_PLAUSIBLE_MAX_MM`, `EPS_SAFE_DIV_STRICT`

#### Structured Tracing (airSpring v0.8.4 Pattern)
- Added `tracing` + `tracing-subscriber` as optional deps behind `biomeos` feature
- Primal binary (`groundspring_primal.rs`) converted from `eprintln!` to
  structured `tracing::info!`/`tracing::warn!`/`tracing::error!` with key-value fields
- `RUST_LOG` env var controls log level (default: `info`)

#### toadStool `compute.dispatch.*` Direct Dispatch (ludoSpring V22 Pattern)
- `dispatch_submit()` — submit GPU workload directly to toadStool
- `dispatch_result()` — poll for dispatched job result
- `dispatch_capabilities()` — query GPU dispatch capabilities
- All use capability-based discovery (no hardcoded primal names)

#### Dual-Format Capability Parsing (neuralSpring S156 Pattern)
- `discover_by_capability()` now handles both flat array and nested object
  capability response formats via `extract_capabilities()`
- 6 new unit tests covering flat array, nested objects (name/capability keys),
  wrapped object, empty, and invalid JSON

#### Infrastructure
- `deny.toml` created: `wildcards=deny`, vulnerability/yanked deny, full
  license allowlist (aligned with airSpring v0.8.4)
- CI: `cross-compile` job for `aarch64-unknown-linux-gnu` (ecoBin compliance)
- CI: `deny` job via `EmbarkStudios/cargo-deny-action@v2`
- Fixed Rust 2024 pattern matching in `stats/agreement.rs`
- Workspace `Cargo.toml` comment clarified re: `temp-env` / `unsafe_code`

#### Quality Gates
- 912+ tests pass (default workspace), 0 clippy warnings, 0 fmt diff
- `cargo doc -D warnings`: 0 warnings
- Zero `#[allow()]` in entire codebase
- Zero `std::env::set_var`/`remove_var` usage (Rust 2024 safe)
- License: AGPL-3.0-or-later (SCYBORG trio)

### V109 Deep Debt Resolution + Smart Refactoring (Mar 16, 2026)

#### Zero-Panic Validation Binaries
- All 28 `serde_json::from_str(BENCHMARK).expect(...)` converted to
  `let Ok(bench) = ... else { eprintln!("FATAL: ..."); return 1; }`
- `validate_notill_sampling.rs` `panic!()` eliminated — fully Result-based
- `validate_nucleus_stack.rs` `expect()` → `let Some(...)` pattern
- `validate_et0_methods.rs` seasonal array `expect()` → `let Some(...)` pattern
- Zero `panic!()` calls remain in any validation binary

#### Named Constants — Physical Bounds
- `ET0_PLAUSIBLE_MIN_MM` (0.01) and `ET0_PLAUSIBLE_MAX_MM` (15.0) with
  FAO-56 provenance, replacing bare literals in `validate_et0_methods.rs`

#### Smart Module Refactoring (4 modules, not just line-splitting)
- `groundspring-validate/lib.rs` (647→506 LOC): extracted `tolerances.rs`
  (106 LOC) and `provenance.rs` (71 LOC) as coherent submodules
- `stats/regression.rs` (624→4 files): `linear.rs`, `quadratic.rs`,
  `nonlinear.rs`, `mod.rs` — split by algorithm family
- `fao56/mod.rs` (642→47 LOC): `daily.rs`, `hargreaves.rs`, `crop_soil.rs`
  — split by ET₀ method domain
- `fao56/pipeline.rs` (623→3 files): `monte_carlo.rs`, `seasonal.rs`,
  `mod.rs` — split by pipeline concern

#### Hardcoding Evolution
- `"biomeos-neural-api.sock"` → `primal_names::LEGACY_NEURAL_API_SOCK`
- Documented `mat_transpose_mul`/`mat_transpose_vec` non-delegation
  rationale in `spectral_recon.rs` (small matrices, Cholesky delegated)

#### Python Dependency Pinning
- Upper bounds added: `numpy>=1.24,<2.0`, `scipy>=1.10,<2.0` etc.
  Prevents PRNG drift from silent major-version upgrades

#### Quality Gates
- 878 tests pass (no-default-features), 0 clippy warnings, 0 fmt diff
- `cargo doc -D warnings`: 0 warnings
- Zero files > 1000 LOC (largest: 705 LOC)
- Zero `panic!()` in validation binaries
- License: AGPL-3.0-or-later (SCYBORG trio)

### V108 Deep Debt + Absorption Evolution (Mar 16, 2026)

#### License Correction
- AGPL-3.0-only → AGPL-3.0-or-later across all 302 source files, LICENSE, Cargo.toml
  (SCYBORG Provenance Trio: AGPL-3.0-or-later + ORC + CC-BY-SA 4.0)

#### barraCuda CPU Delegation
- `std_dev` and `mean_and_std_dev` now delegate to `barracuda::stats::welford::WelfordState`
  when the `barracuda` feature is enabled (CPU path), with local `welford_population` fallback

#### Tolerance Centralization
- Test assertions in `tissue_anderson/compartments.rs` use `crate::tol::ANALYTICAL`
  instead of bare `1e-10` literals
- `tissue_anderson/geometry.rs` on-site energy magic numbers extracted to named constants
  with provenance comments

#### Typed Capability-Based Discovery
- New public functions in `biomeos/interaction.rs`: `discover_by_capability`,
  `compute_execute`, `storage_put`, `storage_get`
- Runtime-only capability discovery — no compile-time primal knowledge

#### Python Provenance Enrichment
- All 29 benchmark JSONs enriched with `python_version` and `numpy_version` in `_provenance`

#### Validation Binary Evolution
- `validate_band_edge.rs` refactored to Result-based error handling with `BenchResult`
- Determinism check uses `groundspring::tol::DETERMINISM` instead of `f64::EPSILON`

#### Quality Gates
- 906 tests pass, 0 clippy warnings (pedantic+nursery), 0 fmt diff
- License: AGPL-3.0-or-later (SCYBORG trio aligned)
- `cargo doc -D warnings`: 0 warnings

### V107 Release Profile + Niche + Tolerance Provenance (Mar 16, 2026)

#### License Alignment
- AGPL-3.0-only → AGPL-3.0-or-later (SCYBORG Provenance Trio, 302 files)

#### Release Profile Optimization
- `lto = true`, `codegen-units = 1`, `strip = true` in workspace Cargo.toml

#### Enriched niche.rs
- `operation_dependencies()` and `cost_estimates()` const functions with structured `OperationDeps` and `CostEstimate` types (ludoSpring V19 pattern)

#### Tolerance Provenance Citations
- All 13 `tol::` constants now have mathematical provenance, source citations, and validation binary references

#### Bare Literal Elimination
- Extracted ~15 named constants from `tissue_anderson/compartments.rs`, `tissue_anderson/geometry.rs`, `anderson/spectral.rs`, `multisignal.rs`, `bistable.rs`

#### Feature-Gated Dead Code
- Spectral constants properly gated behind their respective features

#### Quality Gates
- 906 tests pass, 0 clippy warnings (pedantic+nursery), 0 fmt diff

### V106 primal_names + Typed BiomeOsError (Mar 16, 2026)

#### primal_names Module
- Created `primal_names.rs` — centralized primal name constants (wetSpring V119 pattern)
- Rewired all biomeOS socket paths, discovery, and env checks to use `primal_names::*` constants
- Zero hardcoded primal name strings in production code

#### Typed BiomeOsError
- Evolved `BiomeOsError(String)` → typed enum with `Transport`, `Protocol`, `Serialization`, `Registration`, `Discovery`, `Data`, `Other` variants

#### Documentation
- 38 → 39 modules across README, CONTRIBUTING, specs, whitePaper, baseCamp
- V105 → V106 version bumps in all status lines and deploy graph
- Added `primal_names` to Library Modules table and module listings

#### Quality
- Already had proptest (14 property tests from V105)
- 936 tests, 395/395 validation checks

### V105 Deep Code Evolution (Mar 15, 2026)

#### Panic-Free Production Code
- `#![deny(clippy::expect_used, clippy::unwrap_used)]` enforced across all 3 crate roots
- 9 validation helpers annotated with `#[expect(clippy::expect_used, reason = "...")]`
- `print_provenance_header` evolved: new `try_print_provenance_header()` returns `BenchResult<()>`
- 88 test modules annotated with `#[allow(clippy::unwrap_used, clippy::expect_used)]`

#### Smart Module Refactoring
- `freeze_out.rs` (715 LOC) → 4 domain-aligned submodules: `curve.rs`, `grid.rs`, `chi2.rs`, `nelder_mead.rs`
- Largest file reduced from 715 to 642 LOC (`fao56/mod.rs`)

#### Typed IPC Client
- `ipc.rs`: `GroundSpringClient` with `connect_unix()`, `connect_discovered()`, typed methods
- Runtime socket discovery: `GROUNDSPRING_IPC_SOCKET` → `$XDG_RUNTIME_DIR` → `temp_dir()`
- `IpcError` type with `Display` and `Error` impls

#### Platform Agnosticism
- `biomeos/server.rs`: `/tmp` → `std::env::temp_dir()`
- metalForge validation binaries: hardcoded node names → `GROUNDSPRING_TEST_TOWER`/`GROUNDSPRING_TEST_NODE` env vars
- `et0_methods.py`: `"baseline_commit": "pending"` → `git_commit_hash()`

#### Tolerance Evolution
- `validate_quasispecies.rs`: bare `0.05` → `TOL_RAREFACTION_PROP`
- `validate_notill_sampling.rs`: bare `1e-12` → `groundspring::tol::EXACT`
- `control/common.py`: 15 named tolerance constants mirroring `groundspring::tol`
- `control/common.py`: `git_commit_hash()` shared utility for provenance

### V104 Deep Debt Resolution + Ecosystem Alignment (Mar 15, 2026)

#### Named Constants with Physical Provenance
- `gpu.rs`: `F64_REDUCTION_SMOKE_TOL` — GPU smoke-test sanity threshold (1%)
- `esn/classifier.rs`: `LYAPUNOV_EXTENDED_THRESHOLD`, `ESN_RESERVOIR_SIZE`, `ESN_SPECTRAL_RADIUS`, `ESN_CONNECTIVITY`, `ESN_LEAK_RATE` — validated against hotSpring Exp 015/022
- `esn/brain.rs`: `SHARP_BOUNDARY_RATIO`, `BOUNDARY_EXPLORE_FACTOR` — Nautilus Shell heuristic
- `dispatch.rs`: `DEFAULT_ENERGY`, `DEFAULT_CONFIDENCE`, `DEFAULT_ELEVATION_M`, `DEFAULT_RHMAX_PCT`, `DEFAULT_RHMIN_PCT`, `DEFAULT_REGIME_MARGIN` — FAO-56/RMT/Bazavov provenance
- `tissue_anderson/drug_scoring.rs`: 12 pharmacokinetic constants — Lipinski boundary, topical delivery literature
- `freeze_out.rs`: `NM_SIMPLEX_SCALE` — Nelder-Mead simplex perturbation (2σ grid cell)

#### Capability-Based Discovery
- `dispatch.rs`: replaced `"groundspring"` string literals with `crate::biomeos::FAMILY_ID`
- `biomeos/server.rs`: socket filename format string uses `FAMILY_ID` constant
- All test assertions updated to reference `FAMILY_ID`

#### License Alignment
- 143+ files updated from `AGPL-3.0-or-later` to `AGPL-3.0-or-later` (scyBorg Provenance Trio Guidance)
- Cargo.toml workspace `license` field, pyproject.toml, README, CONTRIBUTING, binary `version` output, all SPDX headers

#### Documentation Debt
- Broken rustdoc link `[serve]` → `[serve_one]` fixed (zero doc warnings)
- 15+ docs updated from V99/V102 → V104 (specs, whitePaper, neuralAPI, CONTROL_EXPERIMENT_STATUS, graphs)
- Capability surface rewritten: `science.*` → `measurement.*` with correct JSON-RPC examples
- `scripts/three_mode_benchmark.sh` binary count corrected (27 → 29)
- `.gitignore` updated with `rust_results.json`
- `PrecisionRoutingAdvice` count corrected (21 → 11)
- graphs/*.toml version references updated to V104

#### Quality Gates
- 936 tests, 0 failures
- 0 clippy warnings (pedantic + nursery)
- 0 doc warnings
- 0 fmt diff

### V103 Deep Debt Audit + Idiomatic Evolution (Mar 15, 2026)

#### Code Quality — Zero Debt
- Replaced `eprintln!` with `log::error!` in `biomeos/server.rs` (structured logging)
- Fixed `clippy::let_unit_value` in `biomeos/server.rs` test
- Zero clippy warnings with `--all-features` (pedantic + nursery verified)

#### Named Constants with Provenance
- `dispatch.rs`: 10 new `const` values (`DEFAULT_REGULARIZATION`, `DEFAULT_TAU_STEP`, `DEFAULT_OMEGA_STEP`, `DEFAULT_SIGMA`, `DEFAULT_T0_LO/HI/STEP`, `DEFAULT_K2_LO/HI/STEP`) replacing magic numbers, each with physical provenance
- `esn/classifier.rs`: `ESN_READOUT_REGULARIZATION` extracted with `hotSpring` validation link
- `esn/brain.rs`: bare `1e-15` replaced with `crate::eps::LOG_FLOOR`
- `lib.rs`: new `eps::LOG_FLOOR` centralized for near-zero guards in log/entropy paths
- `validate_tissue_anderson.rs`: 7 named thresholds with Paper 12 provenance (Gonzales)
- `validate_et0_methods.rs`: `TOL_ET0` centralized to `groundspring-validate::lib.rs`

#### Smart Refactoring
- Extracted `biomeos/interaction.rs` from `biomeos/mod.rs` (683 → 531 LOC): primal discovery, `DiscoveredPrimal`, `direct_primal_rpc`, `proprioception`, `topology`
- Assessed `freeze_out.rs` (706), `bootstrap.rs` (625), `fao56/mod.rs` (641) — cohesive units, no fragmentation warranted

#### Doc Fixes
- `clippy::doc_markdown`: backtick-wrapped `MeV`, `d_eff`, `W_c` in doc comments
- `clippy::too_long_first_doc_paragraph`: rewrote `TOL_ET0` doc header

### V102 Niche Deployment via biomeOS Graph Composition (Mar 14, 2026)

#### Architecture — Spring as Niche
- groundSpring is now a deployable niche that biomeOS composes from a graph
- UniBin binary: `groundspring server/status/version` — the minimum contract for niche deployment
- Measurement domain: `measurement.*` capability namespace with 8 capabilities and semantic mappings
- Deploy graph: `graphs/groundspring_deploy.toml` — canonical 5-phase niche deployment (Tower → dependencies → groundSpring → validate → provenance)
- Niche YAML: `niches/groundspring-measurement.yaml` — BYOB definition with organisms, interactions, customization
- Neural API automation: graphs execute via `neural_api.execute_graph` with topological sort, parallel phases, and rollback

#### New Modules (behind `biomeos` feature gate)
- `biomeos::server` — UDS socket binding, JSON-RPC accept loop, non-blocking serve
- `dispatch` — Semantic method routing: `measurement.*` → library functions, `health.check`, `capability.list`, `lifecycle.status`
- `provenance` — Provenance Trio lifecycle wrappers: `start_session`, `commit_session`, `record_attribution`

#### Measurement Domain Evolution
- Renamed `SCIENCE_CAPABILITIES` → `MEASUREMENT_CAPABILITIES` (legacy alias preserved with deprecation)
- Added `MEASUREMENT_DOMAIN`, `MEASUREMENT_MAPPINGS` constants
- Two-phase registration: domain-level `capability.register` with semantic mappings, then per-capability registration
- 8 measurement methods: `noise_decomposition`, `anderson_validation`, `parity_check`, `et0_propagation`, `regime_classification`, `uncertainty_budget`, `spectral_features`, `freeze_out`

#### Graph Evolution (all 6 graphs)
- Created `groundspring_deploy.toml`: canonical niche deploy graph following `SPRING_AS_NICHE_DEPLOYMENT_STANDARD`
- Updated all 5 existing graphs: `science.*` → `measurement.*` capability names, V87 → V102 version pins
- Provenance Trio wired at graph level (session_create → session_dehydrate → recordDehydration) with `fallback = "skip"`
- Hardcoded primal lists replaced with capability-based discovery (`discover_by_capability = true`)
- Direct `rpc_call target = "nestgate"` evolved to capability-based `storage.put` routing

#### Binary
- `[[bin]] name = "groundspring"` in Cargo.toml (feature-gated `biomeos`)
- Server: bind UDS socket → register capabilities → JSON-RPC accept loop → graceful shutdown
- Status: connect to own socket, call `health.check`, print result
- Version: print version, domain, capabilities, license, family_id

#### Verification
- `cargo fmt --all -- --check`: PASS
- `cargo clippy --features biomeos -- -D warnings`: PASS (0 warnings)
- `cargo test --features biomeos --lib`: PASS
- All default-feature tests: PASS (908)

### V101 Deep Debt Evolution + DRY + Capability-Based Discovery (Mar 14, 2026)

#### Code evolution
- Fixed barracuda `ESNConfig` API drift — struct update syntax (`..Default::default()`) absorbs new SGD fields
- Extracted `chi2_freeze_out()` shared helper in `freeze_out.rs` — 4 copies → 1 (−17 lines)
- Extracted `r_squared_from_residuals()` in `stats/regression.rs` — 3 copies → 1 (−17 lines)
- Extracted `validate_bootstrap_inputs()` and `const fn from_barracuda_ci()` in `bootstrap.rs` — 4 copies → 1 each (−12 lines)
- Net −52 lines across 8 files; zero behavioral change

#### Primal sovereignty
- `validate_nucleus_stack.rs`: replaced hardcoded `["beardog", "toadstool", "squirrel"]` with `biomeos::discover_primals()` runtime discovery
- `validate_node`: removed hardcoded `"toadstool"` direct RPC fallbacks — pure capability-based routing
- `validate_ai`: removed hardcoded `"squirrel"` fallback — pure capability routing via `ai.health`
- `biomeos/mod.rs`: removed primal names from API doc comments — agnostic "compute provider" language

#### Documentation
- Enhanced `validate_weather.rs` provenance table documenting mathematical identity behind each analytical check
- Added tolerance margin note to `validate_et0_methods.rs` `TOL_ET0` constant
- Updated all stale version references: V98/V97→V101 across specs/, whitePaper/, CONTROL_EXPERIMENT_STATUS
- Aligned test counts to canonical 908 default-feature (936 across features) + 287 Python across all docs
- Crafted V101 toadStool/barraCuda absorption handoff

#### Verification
- `cargo fmt --all -- --check`: PASS
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: PASS (0 warnings)
- `cargo doc --workspace --all-features --no-deps`: PASS (0 warnings)
- `cargo test --workspace`: 908/908 PASS

### V100 Deep Debt Audit + Documentation Sync (Mar 14, 2026)

#### Build & CI
- Fixed build-breaking `akida-driver` path dependency (lowercase vs camelCase on case-sensitive FS)
- Scoped `cargo fmt` to groundSpring packages only (prevents upstream formatting drift)
- Added `validate-et0-methods` to CI validation step
- Added `validate-tissue-anderson` to three-tier parity script (29 binaries)

#### Code Quality
- Eliminated 4 silent `unwrap_or(0.0)` fallbacks in `validate_weather.rs` → `expect()`
- Removed hardcoded `"beardog"` primal name from `biomeos/mod.rs` — capability-based discovery
- Replaced bare `1e-10` literal in `rare_biosphere.rs` test with `tol::ANALYTICAL`
- Removed avoidable `clone()` in `freeze_out.rs` test
- Added provenance doc comments to all metalForge tolerance constants
- Fixed 4 `rustfmt` violations in `biomeos/mod.rs` and `gpu.rs`
- Fixed `clippy::doc_markdown` lints in metalForge validation binaries

#### Documentation
- Updated all 5 graph TOMLs from V87/S96c to V100/S130+
- Updated wateringHole README to V100 with V99+V100 as active handoffs
- Fixed stale V96 handoff reference in CONTROL_EXPERIMENT_STATUS.md
- Updated whitePaper README, baseCamp README, experiments README to V100
- Updated ecoPrimals/whitePaper/gen3/baseCamp version line
- Corrected test count: 908 default-feature (936 across all feature gates)
- Created V100 deep debt handoff for toadStool/barraCuda team

### V99 Live NUCLEUS Integration + Direct Primal Discovery (Mar 8, 2026)

#### biomeOS Client Evolution
- Evolved `health()` to try `neural_api.get_metrics` first, fall back to `topology.metrics`
- Added `discover_primals()` — scans biomeOS socket directory for live primal sockets
- Added `primal_health(name)` — direct health check to individual primals
- Added `direct_primal_rpc(name, method, params)` — bypass Neural API routing
- Added `proprioception()` and `topology()` Neural API queries
- Fixed duplicate toadstool discovery (tarpc vs jsonrpc socket dedup)

#### NUCLEUS Validation (Full mode — BearDog + Songbird + ToadStool + Squirrel + Neural API)
- First live NUCLEUS connection from groundSpring: `auto_connect()` → CONNECTED
- 4 primals discovered via socket scan (beardog, songbird, toadstool, squirrel)
- Neural API: 3/3 methods respond (metrics, topology, proprioception)
- Direct primal health: BearDog v0.9.0, ToadStool v0.1.0, Squirrel — all healthy
- 40/40 NUCLEUS experiment checks PASS (Exp 029, 030, 031, 032)
- NestGate: binary version mismatch (no `daemon` subcommand) — needs P1 rebuild

#### Exp 031 Evolution
- Phase A: socket + primal discovery (was offline, now CONNECTED + 4 primals)
- Phase B: Neural API health, proprioception, topology (all live)
- Phase B2 (new): direct primal health checks (BearDog, ToadStool, Squirrel)
- Phase C: compute via direct ToadStool fallback (health, version respond)
- Phase D: AI via direct Squirrel fallback (healthy)
- 16/16 checks PASS

#### Quality Gates
- 936 tests PASS (0 fail, `--features biomeos`)
- clippy pedantic+nursery: 0 warnings
- 39/42 validation binaries PASS (3 pre-existing metalForge GPU tier issues)

### V98 Upstream Rewire + Cross-Spring Evolution Benchmark (Mar 8, 2026)

#### Upstream Pin Updates
- barraCuda `2a6c072` → `a898dee` (deep debt: typed errors, named constants, test resilience, lint compliance)
- toadStool S130 (`88a545df`) → S130+ (`bfe7977b`, clippy pedantic clean, unsafe audit, dependency audit, spring sync — all 5 springs confirm zero API breakage)
- coralReef Iteration 7 (`72e6d13`) → Iteration 10 (`d29a734`, AMD E2E GPU dispatch verified, conditional branch fix in `translate_if` + multi-pred RA merge — unlocks f64 shared-memory reduction shaders via sovereign path)

#### Three-Tier Benchmark (29 validation binaries, release mode)
- Local CPU: **12.5s** (baseline)
- BarraCUDA CPU: **18.4s** (dispatch overhead on small workloads)
- BarraCUDA GPU: **9.9s** (**1.27× faster** than local)
- Workspace test: local 50.3s → barracuda-CPU 29.0s (**1.73× faster**)
- 396 Python correctness tests PASS (1 timing-only flake)
- Kokkos parity: Anderson γ=0.1579, bootstrap CI verified

#### Cross-Spring Shader Provenance Documented
- Full provenance map: 784 WGSL shaders traced across 5 springs
- hotSpring: precision (DF64, Sturm tridiag → 47.7× speedup)
- wetSpring: bio (Gillespie, diversity, alignment)
- neuralSpring: stats (chi², KL, correlation)
- airSpring: hydrology (seasonal pipeline, Hargreaves)
- groundSpring: spectral (Anderson, chi² → all springs), f64 bug → PrecisionRoutingAdvice
- Evolution timeline: 10 key events from Feb 2026 → Mar 8 2026
- Cross-spring flow matrix: all 5 springs both contribute and consume

#### Validation
- 936 tests, 0 failed (unchanged — no API breakage)
- Clippy pedantic + nursery: 0 warnings
- 29/29 validation binaries PASS at all 3 tiers
- 140 metalForge tests PASS

### V97 GPU Smoke Test + Three-Tier Parity Proven (Mar 7, 2026)

#### GPU Precision Routing — Runtime Verification
- **Runtime f64 reduction smoke test**: `f64_reductions_safe()` now computes `mean([1.0; 4])` on GPU and verifies the result is 1.0 (cached in `OnceLock`, runs once per process). Detects GPUs where driver profile says `F64Native` but naga/SPIR-V workgroup shared-memory f64 reductions silently produce zeros (observed on RTX 4070 Ada Lovelace)
- **21 GPU dispatch paths guarded**: changed 11 stochastic/reduction call sites from `get_device()` to `get_device_f64_safe()` — `mean_gpu`, `std_dev_gpu`, `mbe_gpu`, `mc_et0_gpu`, `seasonal_step_gpu`, `gillespie_gpu`, `bootstrap_gpu`, `multinomial_gpu`, `rare_biosphere_gpu` (×2), `wf_batch_gpu`, `seismic_grid_gpu`
- 10 deterministic per-element GPU ops (eigensolvers, Cholesky, ODE batch, elementwise ET₀) remain on `get_device()` — they don't use workgroup shared memory and work correctly
- Clippy nursery warning silenced: `tuple_array_conversions` in `mc_mean_variance_gpu`

#### Three-Tier Parity
- **29/29 validation binaries PASS at all three tiers**: default CPU, barracuda-CPU, barracuda-GPU
- **936 workspace tests** (was 925), 0 failed
- **382 Python correctness tests** pass (3 timing-only skips)
- **Kokkos parity benchmark**: Anderson γ = 0.1579, stats reductions, bootstrap — all verified
- **metalForge**: 140 tests + 8 validation binaries PASS (NPU hardware absent — expected)
- **Titan V spot-check**: 7/7 PASS with `WGPU_ADAPTER_NAME=NVIDIA TITAN V` — confirms GPU math correct on native f64 hardware

#### Files Changed
- `crates/groundspring/src/gpu.rs` — `f64_reduction_smoke_test()` + `OnceLock<bool>` cache
- `crates/groundspring/src/stats/metrics.rs` — `mean_gpu`, `std_dev_gpu` → `get_device_f64_safe()`
- `crates/groundspring/src/stats/agreement.rs` — `mbe_gpu` → `get_device_f64_safe()`
- `crates/groundspring/src/fao56/pipeline.rs` — MC ET₀, seasonal, `mc_mean_variance_gpu`
- `crates/groundspring/src/gillespie.rs` — Gillespie SSA batch
- `crates/groundspring/src/bootstrap.rs` — bootstrap mean GPU
- `crates/groundspring/src/rarefaction/sampling.rs` — multinomial batch
- `crates/groundspring/src/rare_biosphere.rs` — occupancy + tier detection
- `crates/groundspring/src/drift/mod.rs` — Wright-Fisher GPU
- `crates/groundspring/src/seismic.rs` — grid search argmin

### V96 Upstream Rewire + Precision Routing (Mar 7, 2026)

#### Precision Routing
- **`PrecisionRoutingAdvice` wired**: `gpu::precision_routing()` queries barraCuda `GpuDriverProfile` for hardware-appropriate f64 routing (F64Native / F64NativeNoSharedMem / Df64Only / F32Only)
- **Public API**: `groundspring::gpu_precision_routing()` re-exported with `#[cfg(feature = "barracuda-gpu")]` gate
- Provenance: originated in groundSpring V84-V85, absorbed into toadStool S128, re-exported from barraCuda `device::driver_profile`, now round-tripped back as first-class API

#### Upstream Pin Updates
- barraCuda `0bd401f` → `2a6c072` (module decomposition, shader.compile.* IPC alignment, lint hardening, LSCFRK integrators, DF64 bug fix)
- toadStool S129 → S130 (`88a545df`, cross-spring shader rewiring, coralReef proxy, provenance tracking)
- coralReef Phase 11 → Iteration 7 (`72e6d13`, safety boundary, ioctl layout tests, CFG domain-split)

#### Precision Routing Wiring (V96b)
- 11 GPU dispatch paths now check `PrecisionRoutingAdvice` via `get_device_f64_safe()`:
  `pearson_full_gpu`, `pearson_r_gpu`, `covariance_gpu`, `mean_and_std_dev_gpu`,
  `coefficient_of_efficiency_gpu`, `rmse_gpu`, `mae_gpu`, `jackknife_mean_gpu`,
  `simpson_diversity_gpu`, `shannon_diversity_gpu`, `autocorrelation_gpu`
- Added `gpu::f64_reductions_safe()` and `gpu::get_device_f64_safe()` helpers
- Skips GPU dispatch when `F64NativeNoSharedMem` or `F32Only` (naga shared-mem zeros bug)
- Three-tier parity tests: 51/51 PASS (physics 27, stats 24)
- Cross-spring benchmark validated with provenance annotations

#### Cross-Spring Shader Evolution
- Updated `specs/CROSS_SPRING_EVOLUTION.md` with modern provenance (708 shaders, 5 springs)
- Documented origin → consumer matrix, precision evolution timeline, and evolution examples

#### Quality
- 925 tests (was 907), 102 delegations (61 CPU + 41 GPU), all quality gates pass
- Fixed stale `#[expect(clippy::cast_precision_loss)]` in `validate_real_ghcnd_et0.rs`
- Collapsed nested `if let` in `stats/correlation.rs` per clippy `collapsible_if`
- Doc sync: README, CONTROL_EXPERIMENT_STATUS, BARRACUDA_EVOLUTION updated

#### Handoff
- V96 handoff: upstream rewire, precision routing integration, delegation summary, evolution requests
- V95 handoff archived

### V95 coralReef Breakthrough + Doc Sync + Handoff (Mar 7, 2026)

#### coralReef Phase 11
- **Push buffer encoding fixed**: `mthd_incr` (Kepler+ Type 1 INCR header) had `count` and `method/4` fields transposed — produced `0x20000001` instead of `0x20010000` for SET_OBJECT. PBDMA interpreted data words as illegal Type 0 headers → `[PBENTRY]` fault
- **NVIF constants aligned**: `ROUTE_NVIF=0x00`, `ROUTE_HIDDEN=0xFF`, `OWNER_NVIF=0x00`, `OWNER_ANY=0xFF` — matched to Mesa `nvif/ioctl.h`
- **Sovereign GPU method dispatch proven**: SET_OBJECT, INVALIDATE_SHADER_CACHES, SET_SHADER_LOCAL_MEMORY_WINDOW all surviving on Titan V via DRM EXEC path without NAK/NVK/Vulkan
- Discovery method: NVK ioctl trace via LD_PRELOAD spy → cross-referenced `NVC0_FIFO_PKHDR_SQ` in Mesa `nv_push.h` → found field swap

#### Documentation
- README bumped to V95; coralReef Phase 10 → 11
- specs/README test counts corrected (824→907 Rust, 375→261 Python)
- BARRACUDA_EVOLUTION.md: V95 coralReef breakthrough noted
- whitePaper baseCamp and ecoPrimals gen3 baseCamp updated
- CONTRIBUTING.md: validation binary count fixed (33→34)
- CONTROL_RUN_LOG.md: version reference updated

#### Handoff
- V95 handoff: comprehensive coralReef breakthrough + push buffer root cause + sovereign pipeline E2E status + barraCuda/toadStool evolution requests (P0: QMD CBUF binding, Fp64Strategy; P1: CoralReefDevice backend, absorption tracker)
- V94 handoff archived

### V94 Ecosystem Sync + API Evolution + Shannon Delegation (Mar 7, 2026)

#### Ecosystem Sync
- Synced against barraCuda `0bd401f` (cross-spring evolution + API debt elimination), toadStool S129, coralReef Phase 10
- Verified Fp64Strategy regression still present in `SumReduceF64`/`VarianceReduceF64` (no `Fp64Strategy` branching in 0bd401f)
- toadStool absorption tracker stale at V85 — updated V94 handoff corrects to 907 tests, 102 delegations

#### New Delegation
- `barracuda::stats::shannon` in `drift::neutral_diversity_trajectory`: Shannon diversity computation now delegates to barraCuda CPU (absorbed S70+). Local CPU fallback retained for no-barracuda builds
- Delegation count: 102 (61 CPU + 41 GPU), up from 101

#### API Evolution
- `CorrelationFull::r_squared()` and `CorrelationFull::covariance()`: convenience methods mirroring barraCuda's `CorrelationResult` (0bd401f). 4 new tests
- `covariance_gpu`: simplified to use `CorrelationResult::covariance()` directly instead of manual `r * sqrt(var_x * var_y)`

#### Handoff
- New V94 handoff supersedes V93; documents absorption tracker delta (V85→V94), confirms P0 Fp64Strategy open, notes coralReef Phase 10 shader compilation status (Anderson Lyapunov SM70 PASS)
- V93 handoff archived

### V93 Smart Refactoring + FFT Wiring + Coverage Expansion (Mar 7, 2026)

#### Smart Module Splits
- **rarefaction.rs** (743 lines) → `rarefaction/mod.rs` (200) + `rarefaction/diversity.rs` (325) + `rarefaction/sampling.rs` (259): diversity indices (Simpson, Shannon, Bray-Curtis, Pielou) separated from multinomial sampling engine and rarefaction orchestration. Full backward compat via `pub use` re-exports
- **drift.rs** (669 lines) → `drift/mod.rs` (496) + `drift/monitor.rs` (192): `DriftMonitor` advisory system extracted from Wright-Fisher/Kimura simulation. Monitor is reusable by any evolutionary optimizer
- **tissue_anderson/mod.rs** (670 lines) → `mod.rs` (427) + `geometry.rs` (258): skin layers, cell types, disorder functions, and potential generation extracted from simulation functions. Types are the building blocks; simulation is the consumer

#### FFT Wiring
- `spectral_recon::fft_power_spectrum()`: computes |FFT(G)|² for lattice correlator spectral analysis (Bazavov 2025). GPU path delegates to `barracuda::ops::fft::Fft1DF64` via `tokio_block_on`. CPU fallback uses O(N²) DFT. 3 new tests
- Delegation count: 101 (60 CPU + 41 GPU), up from 100

#### Coverage Expansion
- 16 new tests: fao56/equations.rs (16 tests covering all PM building-block functions)
- 10 new tests: tissue_anderson/compartments.rs (preset constructors)
- 12 new tests: tissue_anderson/sweeps.rs (barrier disruption, dimensional duality)
- 3 new tests: spectral_recon (FFT power spectrum)
- Total: 903 Rust workspace tests, 0 failures

### V91 Complete Ecosystem Rewire + Cross-Spring Evolution (Mar 7, 2026)

#### New Delegations
- **AutocorrelationF64 GPU**: `wdm::autocorrelation()` delegates to `barracuda::ops::autocorrelation_f64_wgsl::AutocorrelationF64` for GPU-accelerated ACF computation. `wdm::optimal_block_size()` uses ACF to estimate integrated autocorrelation time → recommended jackknife block size. Provenance: hotSpring MD VACF → barraCuda S128
- **Empirical Spectral Density**: `anderson::empirical_spectral_density()` delegates to `barracuda::stats::spectral_density`. Provenance: neuralSpring V69 → barraCuda
- **PeakDetectF64**: `anderson::detect_transition()` delegates to `barracuda::ops::peak_detect_f64::PeakDetectF64` for GPU peak detection in disorder sweep derivatives to find Anderson metal-insulator transition W_c
- **CovarianceF64 GPU**: `stats::covariance()` GPU path now uses `barracuda::ops::covariance_f64_wgsl::CovarianceF64::sample_covariance()` directly instead of deriving from `CorrelationF64::correlation_full`
- **Marchenko-Pastur**: `anderson::marchenko_pastur_upper()` delegates to `barracuda::stats::spectral_density::marchenko_pastur_bounds()`. `anderson::spectral_diagnostics_auto()` convenience wrapper added

#### Benchmarks
- 5 new workloads in `bench-cpu-vs-gpu`: Autocorrelation (10k, lag 200), Covariance (5k pairs), Spectral diagnostics (1k eigs), ESD (5k eigs, 50 bins), Optimal block size (5k AR(1))
- Total: 21 benchmark workloads

#### Cross-Spring Evolution
- New `specs/CROSS_SPRING_SHADER_EVOLUTION.md`: full provenance tree showing how WGSL shaders flow between springs via barraCuda absorption. Documents which spring created each shader, when it was absorbed, and which springs now benefit

#### Tests
- 14 new tests: autocorrelation (5), empirical spectral density (2), transition detection (2), Marchenko-Pastur (2), spectral_diagnostics_auto (1), covariance (1), optimal block size (1)
- Total: 807 tests, 0 failures

#### Delegation Count
- 100 active delegations (59 CPU + 41 GPU), up from 93

### V90 Deep Debt Execution (Mar 7, 2026)

#### Unsafe Code Eliminated
- All `unsafe { std::env::set_var() }` in `tests/biomeos_integration.rs` replaced with `temp_env::with_var()` / `temp_env::with_vars()` (RAII-based). `#[allow(unsafe_code)]` attributes removed. `#![forbid(unsafe_code)]` now fully honoured workspace-wide

#### Validation Binary Migration
- 11 binaries migrated from inline mean/variance/mbe/rmse to `groundspring::stats::*` (→ barracuda delegation): validate_real_ghcnd_et0, validate_uncertainty_bridge, validate_fao56, validate_et0_anderson, validate_aggregate_stability, validate_signal_specificity, validate_vendor_parity, validate_precision_drift, validate_quasispecies

#### Tolerance Documentation
- 6 bare numeric tolerances documented with mathematical justifications: eigenvalue band fraction, eigenvalue percentage, Xoshiro mean, γ parity, PM/Hargreaves ratio, mean absolute difference, band edges

#### Coverage Expansion
- 18 new tests: fao56/pipeline.rs (6, coverage 0→99.6%), bootstrap.rs (5, +8%), stats/regression.rs (5, +12%), anderson.rs (2)
- Overall line coverage: 91.55%

### V89 Rewire to barraCuda/toadStool/coralReef Evolution (Mar 6, 2026)

#### Breaking API Rewire
- **tarpc 0.35 → 0.37**: Aligned with barraCuda workspace. `#[tarpc::service]` traits unchanged; tarpc 0.37 no longer triggers `clippy::too_many_arguments` internally, so the unfulfilled `#[expect]` was removed from `ipc.rs`
- **`barracuda::ops` GPU-gated**: barraCuda moved `pub mod ops` behind `#[cfg(feature = "gpu")]`. `rarefaction::multinomial_sample` delegation re-gated from `barracuda` to `barracuda-gpu`; CPU fallback now compiles correctly without GPU feature
- **`domain-esn` feature**: barraCuda's `esn_v2` module now requires `domain-esn` feature. Added to groundSpring's `barracuda-gpu` feature

#### Code Quality
- **Rust 2024 unsafe model**: `set_var`/`remove_var` are unsafe in edition 2024. Workspace lint changed from `forbid` to `deny`; all three `lib.rs` files assert `#![forbid(unsafe_code)]` for production code. Test file uses `#[allow(unsafe_code)]` with documented SAFETY comments
- **Collapsible if**: 8 `collapsible_if` warnings resolved across `fao56/mod.rs`, `biomeos/discovery.rs`, `validate_real_ghcnd_et0.rs`, `validate_iris_seismic.rs`, `validate_anderson.rs`, `validate_nucleus_pipeline.rs` using Rust 2024 `let` chains

#### Pin Updates
- **barraCuda**: `e1184f3` → `ed82625` (Fp64Strategy wired into SumReduceF64/VarianceReduceF64)
- **toadStool**: S96c `d77fc546` → S128b `22d1a2c7` (f64 shared-memory routing, PrecisionRoutingAdvice, sovereign_binary_capable, shader compilation IPC)
- **coralReef**: Phase 6 `849fedd` → Phase 9 `b7f8ab4` (sovereign pipeline complete, zero C dependencies, NVIDIA SM70-89 + AMD RDNA2)

#### Known Issue
- **GPU test regression**: 6 GPU-dispatched tests fail with barraCuda `ed82625` (`Fp64Strategy` integration into `SumReduceF64`/`VarianceReduceF64`). All 500 CPU tests pass. Filed as evolution request in V89 handoff

#### Validation
- 500+ tests pass (476 lib + 24 integration), 0 failures (CPU-only)
- `cargo fmt` + `cargo clippy --workspace --all-features` zero warnings
- 261/261 Python provenance tests pass

### V88 Deep Audit + Evolution (Mar 6, 2026)

#### Quality Evolution
- **Structured logging**: Added `log = "0.4"` to groundspring + forge. All `eprintln!` in library code evolved to `log::warn!` (`biomeos/mod.rs`, `nucleus.rs`)
- **Formal provenance schema**: New `specs/PROVENANCE_SCHEMA.md` — defines required vs optional benchmark JSON fields, enforcement mechanisms, stochastic experiment rules
- **Auto-discovery drift guard**: `regenerate_benchmarks.sh` now uses `find control -name 'benchmark_*.json'` instead of hardcoded 29-element array. New experiments are automatically picked up
- **Fixed `benchmark_et0_methods.json`**: Added missing `_doi` field (`10.1016/S0378-3774(98)00053-4`). Python provenance tests: 261/261 PASS (was 260/261)
- **Provenance comments**: Documented analytical derivations for all hardcoded expected values in tests (`band_structure`, `multisignal`, `bistable`, `seismic`)
- **Named constants**: Extracted inline Tikhonov regularization strengths to `LAMBDA_NOISELESS`, `LAMBDA_NOISY`, `LAMBDA_PARITY` in `spectral_recon.rs` and `validate_gpu_tier/spectral.rs`

#### Documentation
- **Root docs**: Updated README (35 experiments, 395/395, V88 status), CONTRIBUTING (34 modules), CONTROL_EXPERIMENT_STATUS (93 delegations), specs/README
- **whitePaper**: Updated baseCamp README, neuralAPI status, CROSS_SPRING_EVOLUTION date, experiments README
- **wateringHole**: New "What This Is" and "Conventions" sections, naming convention aligned to actual pattern, V88 handoff
- **Archive**: 6 local + 9 ecoPrimals wateringHole superseded handoffs moved to archive/
- **gen3 baseCamp**: Updated groundSpring version to V88 with structured logging and provenance schema

#### Handoff
- **New**: `GROUNDSPRING_V88_DEEP_AUDIT_EVOLUTION_HANDOFF_MAR06_2026.md` — full codebase audit results, PRNG alignment roadmap, test coverage gaps, evolution requests for barraCuda/toadStool/coralReef
- **Copied** to `ecoPrimals/wateringHole/handoffs/`

#### Validation
- 824+ tests pass, 0 failures
- `cargo fmt` + `cargo clippy --workspace -- -D warnings` + `cargo doc` all clean
- 261/261 Python provenance tests pass

### V87 Tier B Resolution + Cross-Spring Delegation Completion (Mar 6, 2026)

#### New Delegations
- **`rarefaction::multinomial_sample`** CPU-delegated to `barracuda::ops::bio::multinomial_sample_cpu` — cumulative prob adapter, Xorshift64 RNG via closure. Cross-spring round-trip: groundSpring V62 → toadStool → barraCuda S93 → groundSpring V87
- **`anderson::anderson_potential`** CPU-delegated to `barracuda::spectral::anderson_potential` — documented PRNG divergence (Xorshift64 vs LcgRng, distributional parity)

#### Tier B Resolution
- **5 stale entries resolved**: `freeze_out::grid_fit_2d`, `seismic::grid_search_inversion`, `rare_biosphere::abundance_occupancy`, `rare_biosphere::tier_detection_rate`, `gillespie::birth_death_ssa` — all already wired in V42-V68
- **2 CPU-by-design**: `quasispecies::quasispecies_simulation` (per-gen mutation thinning overhead), `band_structure::find_band_edges` coarse scan (data-dependent matrix chains)
- **New batch API**: `quasispecies_simulation_batch` for multi-replicate trajectories
- **Delegation count**: 91 → 93 (56 CPU + 37 GPU), 0 evolution candidates remaining

#### Documentation
- **`specs/BARRACUDA_EVOLUTION.md`**: Tier B table fully resolved, delegation count updated, V87 timeline entries
- **`wateringHole/CROSS_SPRING_SHADER_EVOLUTION.md`**: Bidirectional provenance narratives (hotSpring precision → all; wetSpring ↔ groundSpring bio; airSpring → physics; neuralSpring → dispatch); delegation lineage #44-45
- **New handoff**: `GROUNDSPRING_V87_TIER_B_RESOLUTION_HANDOFF_MAR06_2026.md`

#### Validation
- 804+ tests pass in both default and barracuda modes
- 0 clippy warnings (pedantic) in both modes

### V86 DF64 Reduce Wiring + Full Stats Benchmark (Mar 6, 2026)

#### barraCuda Evolution (`e1184f3`)
- **Fp64Strategy wired into SumReduceF64 and VarianceReduceF64**: Consumer GPUs (Hybrid strategy) now route to DF64 shaders instead of broken f64 workgroup shared memory shaders
- **New WGSL shaders**: `sum_reduce_f64_via_df64.wgsl` (sum/max/min) and `variance_reduce_f64_via_df64.wgsl` (Welford variance) — f64 I/O, DF64 internal shared memory
- **BootstrapMeanGpu**: Confirmed no workgroup shared memory — no DF64 routing needed

#### Benchmark Results
- **Precision parity PROVEN**: 4/5 kernels bitwise identical across Python, Kokkos CUDA, and Rust CPU
- **Speed**: Rust CPU 21× faster than Python, Kokkos CUDA 73× faster
- **Energy**: Kokkos 74.6× less energy than Python, Rust CPU 35.1× less
- **DF64 projection**: anderson_lyapunov on RTX 4070 — DF64 ≈ 12,556 µs vs Kokkos 34,741 µs (2.8×)

#### Documentation
- **Commit pins updated**: barraCuda `cf1602c` → `e1184f3` across all specs, docs, and benchmarks
- **New handoff**: `GROUNDSPRING_V86_FULL_STATS_HANDOFF_MAR06_2026.md` — DF64 reduce wiring, full 4-tier benchmark results, energy comparison, GPU pipeline root cause analysis

#### Known Issues
- GPU reduce ops still return 0 — root cause is in `compile_shader_f64` pipeline (SovereignCompiler/SPIRV passthrough), not shader selection. Even pre-existing DF64-wired `VarianceF64` returns 0 on tested hardware

### V85 coralReef Sovereign Compilation + Docs Cleanup (Mar 6, 2026)

#### Documentation
- **Root docs refresh**: README, CONTROL_EXPERIMENT_STATUS, CONTROL_RUN_LOG updated to V85 with coralReef `849fedd` pin
- **whitePaper/baseCamp**: Updated validation summary with V85 sovereign compilation achievement
- **ecoPrimals/whitePaper/gen3/baseCamp**: Updated groundSpring provenance line (V83 → V85) and cross-spring provenance
- **experiments/README.md**: Updated barraCuda pin to include coralReef `849fedd`
- **wateringHole/README.md**: Added V85 toadStool/barraCuda evolution handoff and coralReef sovereign handoff to active list
- **specs/**: Updated version references in BARRACUDA_EVOLUTION, CROSS_SPRING_EVOLUTION, PAPER_REVIEW_QUEUE to V85
- **metalForge/ABSORPTION_MANIFEST.md**: Updated version and coralReef pin
- **Cross-spring shader docs**: Updated wateringHole and whitePaper CROSS_SPRING_EVOLUTION to V85
- **CONTROL_RUN_LOG.md**: Added historical note (V75–V85 tracked via CHANGELOG and handoffs)
- **benchmark_cross_spring.rs**: Updated evolution banner from V82 to V85
- **New handoff**: `GROUNDSPRING_V85_TOADSTOOL_BARRACUDA_EVOLUTION_HANDOFF_MAR06_2026.md` — comprehensive evolution handoff for toadStool/barraCuda/coralReef teams with f64 pipeline analysis, sovereign compilation findings, and per-primal evolution requests

#### Fixed (in coralReef — commit `849fedd`)
- **CFG edge loss in opt_jump_thread**: `translate_if` now emits conditional branches (`@!cond BRA reject`), preventing `rewrite_cfg` from losing structural edges to reject blocks. Fixes compilation of shaders with 3+ `workgroupBarrier()` calls
- **Multi-predecessor RA merge**: Register allocator now merges SSA→register mappings from ALL predecessors at merge points (not just `pred[0]`)
- **OpBar encoding**: BAR.SYNC instruction encoder fields now populated (src, reduction op, barrier mode, predicate)

#### Achieved
- **Sovereign f64 reduction compilation**: coralReef compiles the exact f64 shared-memory reduction shaders that fail through `naga → SPIR-V → NVK/NAK` to native SM70 (Titan V) and SM89 (RTX 4070) binaries
- **6/6 shader compilation**: basic f64, storage r/w, shared_mem simple, 2-barrier, 3-barrier, 8-step unrolled reduction — all compile for both GPU architectures

#### Documented
- **Handoff**: `CORALREEF_SOVEREIGN_COMPILATION_HANDOFF_MAR06_2026.md` — detailed findings, remaining gaps (coralDriver, BAR.SYNC encoding, f64 instruction emission), and evolution guidance

#### Remaining Gaps
- **coralDriver**: Native binaries compile but cannot be submitted to GPU yet (no cubin ELF wrapper or userspace driver)
- **BAR.SYNC opex**: nvdisasm reports undefined opex table value 0x10 — barrier count field encoding needs Volta reference
- **f64 instructions**: Basic shaders disassemble as FMUL/FADD (f32) instead of DMUL/DADD (f64)
- **Loop compilation**: Loop-based tree reduction hits `opt_instr_sched_prepass` assertion (unrolled works)
- **Uniform buffers**: `var<uniform>` bindings not yet supported in compute prologue

### V84 GPU Validation Discovery (Mar 6, 2026)

#### Added
- **GPU validation handoff**: `GROUNDSPRING_V84_GPU_VALIDATION_HANDOFF_MAR06_2026.md` — comprehensive dual-GPU probe results, f64 pipeline diagnostics, and evolution guidance for barraCuda/coralReef/toadStool

#### Fixed
- **CoralCompiler tokio panic**: `barraCuda/coral_compiler.rs` — `spawn_coral_compile()` now checks `tokio::runtime::Handle::try_current()` before spawning, preventing panic in synchronous contexts (e.g., groundSpring GPU tests without Tokio runtime)
- **Device selection strategy**: `gpu.rs` — switched from `new_f64_capable()` (could select Titan V/NVK with broken compute) to `new()` (high-performance discrete GPU, proprietary driver preferred)

#### Discovered
- **f64 WGSL shared memory pipeline issue**: All f64 reduction shaders with `var<workgroup> shared_data: array<f64, 256>` return 0 on both GPUs (RTX 4070 proprietary + Titan V NVK). Root cause: naga/SPIR-V compilation pipeline for f64 workgroup shared memory. Simple f64 arithmetic (basic_f64 probe) works; complex shaders with barriers + tree reduction fail
- **RTX 4070 (SM89 Ada)**: DF64 path green (tensor matmul, DF64 add/sub, FHE NTT), f64 builtins 3/9 native (sqrt, fma, abs/min/max), Fp64Strategy correctly detected as Hybrid
- **Titan V (SM70 Volta)**: Same f64 shared memory issue via NVK/NAK. f64 probe runs without system freeze (previous crash was heavy compute, not simple probes). Fp64Strategy detected as Native but broken through NVK

#### Quality
- `cargo test --workspace`: PASS (824 tests, 0 failures) — CPU path unaffected
- `cargo test --features barracuda-gpu`: 17/32 pass (14 fail = f64 reduction returning 0)
- `cargo clippy --workspace -- -D warnings`: PASS (0 warnings)
- `validate_gpu` (barraCuda): 6/6 pass on RTX 4070 (DF64, tensor matmul, FHE NTT)

### V83 Dependency Catch-Up + Pin Refresh (Mar 6, 2026)

#### Changed
- **barraCuda pin**: `a4c20a5` → `e1184f3` — deep debt resolved (JSON-RPC 2.0 compliance, unsafe elimination, zero-copy docs), GpuView persistent buffers, AutocorrelationF64 GPU op, CoralCompiler for coralReef integration, anderson_lyapunov f32/f64 WGSL shaders, fft_radix2_f64 shader, Kokkos parity benchmarks started. 708 shaders, 3,471+ tests
- **toadStool pin**: S95 (`d4817e2e`) → S96c (`d77fc546`) — HardwareFingerprint with sovereign capability detection, SubstrateCapabilityKind (12 variants), 5 god file splits (<1000 LOC), crates/api/ fossilized, V4L2 unsafe documented. 18,028 tests
- **coralReef pin**: `2e89541` → `1e048be` — `nak/` → `codegen/` vendor-neutral rename, pluggable `Frontend` trait (NagaFrontend default), Phase 5.5 naming evolution complete, Phase 6 multi-vendor in progress. 672 tests (up from 390)

#### Ecosystem — Cross-Spring Evolution Observations
- **barraCuda e1184f3**: New GPU primitives (AutocorrelationF64 for time-series, GpuView<T> for zero-copy GPU-resident computation, anderson_lyapunov f32/f64 shaders for transfer-matrix Lyapunov), CoralCompiler for coralReef native binary integration, DF64 naga rewrite validated with compound assignments, 15 ops gain Fp64Strategy-based shader selection
- **toadStool S96c**: Sovereign pipeline infrastructure (HardwareFingerprint estimated_tflops, SubstrateType expanded to 8 variants including NPU/TPU/FPGA), capability-scored discovery with `is_sovereign_capable()`, safe allocation limits for NVK
- **coralReef 1e048be**: Vendor-neutral architecture evolution — `codegen/` replaces `nak/`, `TranscendentalOp` replaces `MuFuOp`, pluggable Frontend trait decouples shader language from compiler core. NVIDIA backend complete, AMD/Intel backends in progress

#### Quality
- `cargo check --workspace`: PASS (clean build with updated deps)
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS (0 warnings)
- `cargo test --workspace`: PASS (824 tests, 0 failures)
- No API changes required — all 91 delegations verified compatible

### V82 Delegation Expansion + Smart Refactoring (Mar 5, 2026)

#### Added
- **Thornthwaite ET₀ delegation**: `fao56::thornthwaite_et0()` and `fao56::thornthwaite_heat_index()` delegate to `barracuda::stats::hydrology`. Monthly temperature-based ET₀ for climate classification (Thornthwaite 1948)
- **`fit_all` unified regression**: `stats::fit_all()` runs linear, quadratic, exponential, and logarithmic fits in one call, delegates to `barracuda::stats::regression::fit_all`. Useful for automated model comparison
- **12 new tests**: 8 Thornthwaite (heat index, ET₀ positivity, edge cases, temperature/daylight scaling), 4 fit_all (multi-model, best R², empty/insufficient data)

#### Changed
- **Delegation count**: 88 → 91 active delegations (54 CPU + 37 GPU) — +2 from Thornthwaite, +1 from fit_all
- **Test count**: 812 → 824 workspace tests

#### Refactored
- **`esn.rs` smart-split**: 816 lines → `esn/brain.rs` (brain architecture: DriftAction, ConceptEdge, uncertainty), `esn/classifier.rs` (regime classification, spectral features, EsnClassifier), `esn/mod.rs` (shared RegimeLabel, re-exports). Semantic split by domain, not mechanical line count
- **`fao56/mod.rs` smart-split**: 811 lines → `fao56/mod.rs` (624, core Penman-Monteith + Hargreaves) + `fao56/et0_methods.rs` (alternative ET₀: Makkink, Turc, Hamon, Thornthwaite). All submodules under 625 lines

#### Quality
- `cargo fmt --check`: PASS
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS (0 warnings)
- `cargo test --workspace`: PASS (824 tests, 0 failures)
- No files > 720 lines (down from 816 max)
- Deep debt audit: 0 unsafe, 0 production unwrap, 0 TODO/FIXME, 0 production mocks, 0 hardcoded paths

### V81 Modern Rewire + Cross-Spring Validation (Mar 5, 2026)

#### Added
- **`BootstrapMeanGpu` GPU dispatch**: `bootstrap_mean()` now dispatches to barraCuda's `BootstrapMeanGpu` for parallel resample computation when `barracuda-gpu` is enabled. Falls back to CPU `barracuda::stats::bootstrap_mean`, then to local Xorshift64 implementation
- **coralReef cloned**: Sovereign shader compiler (`ecoPrimals/coralReef`) cloned via SSH — 390 tests, Phase 5 complete, WGSL/SPIR-V → native SM70+ binary with f64 transcendentals

#### Fixed
- **`freeze_out::lbfgs_refine` feature gate**: barraCuda moved `optimize` module behind `gpu` feature; updated groundSpring gate from `barracuda` to `barracuda-gpu` for L-BFGS refinement and associated constants

#### Ecosystem — Cross-Spring Evolution Observations
- **coralReef 2e89541**: Absorbed groundSpring patterns (BTreeMap deterministic serialization, unsafe removal, silent-default audit, cross-spring provenance doc-comments in `lower_f64/`). Smart-refactored `poly.rs` by algorithm family (exp2/log2/trig), evolved `AtomType` panics to `Option<AtomType>`
- **barraCuda a4c20a5**: Fused reduction shaders (Welford mean+variance, 5-accumulator Pearson), DF64 three-tier precision (f32/DF64/f64), TensorContext pooled buffers, subgroup capability detection — all inherited via path dependency
- **toadStool S95**: 18,028 tests (from 5,369 at S94b), clippy pedantic clean, full audit execution. 144 ComputeDispatch ops migrated, full primal decoupling. Shaders transferred to standalone barraCuda (S93). neuralSpring pinned at V85/S127
- **Cross-spring shader provenance**: hotSpring DF64 → all springs get f64-class precision on consumer GPUs; wetSpring bio (Smith-Waterman, Gillespie) → neuralSpring → groundSpring delegation; airSpring L-BFGS → groundSpring freeze-out refinement; groundSpring RAWR → wetSpring rarefaction CIs; groundSpring Anderson sweep → ESN training data cross-spring

#### Benchmarks
- **27/27 cross-spring checks passed** (benchmark-cross-spring)
- **CPU vs GPU benchmark**: 16 workloads profiled. GPU path inactive (nouveau Titan V freeze deferred; RTX 4070 proprietary driver pending). Batch dispatch parity confirmed on CPU fallback
- **coralReef 390 tests green**, clippy/fmt/doc clean

#### Quality
- `cargo fmt --check`: PASS
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS
- `cargo test --workspace`: PASS (812+ tests, 27 validation binaries, 0 failures)

### V80 Fused Ops Rewire + barraCuda/ToadStool Catch-Up (Mar 5, 2026)

#### Added
- **Fused `correlation_full` GPU dispatch**: New `stats::pearson_full()` function and `CorrelationFull` struct expose barraCuda's 5-accumulator `CorrelationF64::correlation_full` shader — returns mean_x, mean_y, var_x, var_y, and Pearson r in a single GPU dispatch (no intermediate readbacks). DF64 precision tier auto-selected on consumer GPUs via `Fp64Strategy::Hybrid`
- **Covariance GPU path**: `stats::covariance()` now has a GPU dispatch via `correlation_full`, deriving sample covariance from population covariance with Bessel correction. Was CPU-only
- **5 new tests**: `pearson_full` perfect positive/negative, empty, constant, and agreement-with-`pearson_r` tests

#### Changed
- **Welford single-pass CPU stats**: `std_dev`, `sample_std_dev`, and `mean_and_std_dev` CPU fallbacks now use `welford_population()` — numerically stable single-pass algorithm replacing the two-pass mean-then-variance pattern
- **barraCuda catch-up**: Validated against barraCuda HEAD (`15d3774`) — chi_squared GPU feature gate fix, DF64 precision tiers, TensorContext migration inherited as free upgrades via path dependency
- **Delegation count**: 85 → 87 active delegations (51 CPU + 36 GPU) — +2 GPU from `correlation_full` wiring (pearson_full, covariance)
- **Test count**: 807 → 812 workspace tests

#### Ecosystem
- **ToadStool S94b review**: Full primal decoupling confirmed, barraCuda standalone verified, V68 groundSpring work fully absorbed
- **coralReef awareness**: New ecosystem primal (sovereign Rust shader compiler) at Phase 2. groundSpring assigned Level 4 (driver/memory/queue) per sovereign compute roadmap

#### Quality
- `cargo fmt --check`: PASS
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS
- `cargo doc --workspace --no-deps`: PASS
- `cargo test --workspace`: PASS (812 tests)
- Python Phase 0: 29 experiments, 390 checks PASS
- Rust Phase 1: 34 binaries, 395/395 PASS

### V79 Exp 035 Multi-Method ET₀ + Delegation Strengthening (Mar 5, 2026)

#### Added
- **Exp 035: Multi-Method ET₀ Cross-Validation** — New experiment comparing five ET₀ methods (Penman-Monteith, Hargreaves, Makkink, Turc, Hamon) at the FAO-56 Example 18 reference site. Python control (15/15 PASS) + Rust validation binary (19/19 PASS). Validates the full pipeline: Python baseline → pure Rust math → barracuda CPU delegation
- **Python control**: `control/et0_methods/et0_methods.py` — 5-method comparison with seasonal variation, input sensitivity analysis (Makkink radiation CV, Hamon temperature CV), and cross-method agreement checks
- **Rust validation binary**: `validate-et0-methods` — matches Python baselines within 0.005 mm/day tolerance (trig intermediate rounding differences documented)
- **Benchmark JSON**: `control/et0_methods/benchmark_et0_methods.json` — full provenance for Rust validation

#### Changed
- **Seismic `origin_time_and_rms`**: Now delegates mean computation to `crate::stats::mean` (which uses barracuda CPU when enabled), strengthening the delegation chain
- **Delegation count**: 84 → 85 active delegations (51 CPU + 34 GPU) — seismic mean delegation
- **Test count**: 806 → 807 workspace tests

#### Quality
- `cargo fmt --check`: PASS
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS
- `cargo doc --workspace --no-deps`: PASS
- `cargo test --workspace`: PASS (807 tests)
- Python Phase 0: 29 experiments, 390 checks PASS
- Rust Phase 1: 34 validation binaries, 395/395 PASS

### V78 Modern Rewiring + Cross-Spring Benchmark Evolution (Mar 5, 2026)

#### Added
- **Fused `mean_and_std_dev`**: New `stats::mean_and_std_dev()` function uses `VarianceF64::mean_variance()` (Welford single-pass GPU shader) when `barracuda-gpu` is enabled, replacing the 2-dispatch pattern. Wired into `rarefaction::rarefaction_at_depth` (saves 2 GPU dispatches) and `gillespie::birth_death_ssa_batch_{cpu,gpu}` (saves 1 dispatch each)
- **Makkink ET₀**: `fao56::makkink_et0()` — radiation-only method, delegates to `barracuda::stats::hydrology::makkink_et0` (airSpring → barraCuda v0.3.2)
- **Turc ET₀**: `fao56::turc_et0()` — temperature + radiation + humidity method, delegates to `barracuda::stats::hydrology::turc_et0` (airSpring → barraCuda v0.3.2)
- **Hamon ET₀**: `fao56::hamon_et0()` — temperature + daylight hours method, delegates to `barracuda::stats::hydrology::hamon_et0` (airSpring → barraCuda v0.3.2)
- **16 new unit tests**: 12 for Makkink/Turc/Hamon individual behavior + 1 cross-method comparison, 3 determinism tests

#### Changed
- **Cross-spring benchmark**: Updated `benchmark_cross_spring.rs` to barraCuda v0.3.3 / toadStool S94b state — added fused mean+variance benchmark, ET₀ method comparison table, Phase 5 evolution timeline (DF64 tiers, fused shaders, TensorContext, new ET₀ ops), 8 new provenance entries in shader table
- **Delegation count**: 81 → 84 active delegations (50 CPU + 34 GPU) — 3 new CPU delegations (Makkink, Turc, Hamon) + 1 fused GPU optimization

#### Quality
- `cargo fmt --check`: PASS
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS
- `cargo doc --workspace --no-deps`: PASS
- `cargo test --workspace`: PASS (806 tests, up from 790)
- Deep debt zero maintained
- Cross-spring provenance tracked for all new delegations

### V77 wgpu 28 Migration + barraCuda v0.3.3 Sync (Mar 5, 2026)

#### Changed
- **wgpu 22 → 28**: Bumped `wgpu` dependency from v22 to v28 in both `crates/groundspring/Cargo.toml` and `metalForge/forge/Cargo.toml`, synchronized with barraCuda v0.3.3 (`4629bdd`)
- **`validate_metalforge_titan_v.rs` wgpu 28 API migration**: `entry_point: Some("main")`, `set_bind_group(0, Some(&bg), &[])`, `Instance::new(&desc)`, async `enumerate_adapters` via `tokio_block_on`, `PollType::Wait` (replaces `Maintain::Wait`), `DeviceDescriptor` new fields (`experimental_features`, `trace`), removed `request_device` trace path argument
- **`probe.rs` wgpu 28 API migration**: `Instance::new(&desc)`, async `enumerate_adapters` via `barracuda::device::test_pool::tokio_block_on`

#### Free Upgrades (inherited from barraCuda v0.3.3)
- **DF64 precision tiers**: 15 barracuda ops auto-select f64/DF64/f32 per GPU hardware via `Fp64Strategy`. groundSpring's delegations to `variance_f64`, `correlation_f64`, `covariance_f64`, `beta_f64` etc. get ~10× throughput on consumer GPUs
- **Fused reduction shaders**: Single-pass Welford mean+variance, 5-accumulator Pearson correlation
- **TensorContext pooled buffers**: Stats ops use pipeline/buffer caching
- **DF64 naga rewriter fix**: NAK compound assignment bug fixed — Titan V DF64 works correctly
- **`sourdough-core` removed**: Broken path dependency eliminated from barracuda

#### Pins
- **barraCuda**: v0.3.3 (`4629bdd`)
- **toadStool**: S94b (`9d359814`)

#### Quality
- `cargo fmt --check`: PASS
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS
- `cargo doc --workspace --no-deps`: PASS
- `cargo test --workspace`: PASS (790 tests)
- Deep debt zero maintained

### V76 Structural Evolution + Deep Debt Zero + Absorption Handoff (Mar 5, 2026)

#### Changed
- **`validate_gpu_tier.rs` → domain-split binary**: 895-line monolith refactored into `validate_gpu_tier/{main,stats,spectral,bio}.rs` (58+167+290+422 lines). Split follows scientific domain boundaries: stats (metrics/regression/bootstrap/jackknife), spectral (Anderson/Almost-Mathieu/Tikhonov/eigendecomp/PRNG), bio (diversity/kinetics/ODE/Gillespie/Wright-Fisher/FAO-56/tissue)
- **`groundspring_forge::nucleus` module**: Extracted shared `discover_uid()`, `biomeos_socket_dir()`, and `NucleusHarness` from `validate_nucleus_pipeline.rs` and `validate_nestgate_ncbi.rs` into forge library — eliminates ~120 lines of duplication with 4 unit tests
- **Observation-gap benchmark parity chain**: `validate_weather.rs` now loads `benchmark_observation_gap.json` via `include_str!` and validates acceptance criteria (temperature R², RMSE range, precipitation hit rate) against synthetic data — 21/21 checks (up from 14/14)
- **`unwrap()` elimination**: All `unwrap()` calls in production binaries replaced with `if let` / graceful error handling
- **Tolerance constant migration**: Bare float literals in `validate_gpu_tier.rs` replaced with `tol::ANALYTICAL`, `tol::EXACT`, `tol::CDF_APPROX`, `tol::RECONSTRUCTION`, `tol::INTEGRATION`
- **Provenance headers**: Runtime provenance prints added to 3 validation binaries missing them
- **Clippy fixes**: `cast_precision_loss` (u128→f64 via `as_secs_f64()`), `suboptimal_flops` (→ `mul_add`), `doc_markdown` (backticks), `too_many_lines` (`#[expect]`), `manual_midpoint` (→ `f64::midpoint`)

#### Quality
- `cargo fmt --check`: PASS
- `cargo clippy --workspace -- -D warnings`: PASS
- `cargo doc --workspace --no-deps`: PASS
- `cargo test --workspace`: PASS (790 tests)
- All files < 1000 lines (largest: `bio.rs` at 422)
- Zero TODO/FIXME/HACK/STUB/MOCK in Rust source
- Zero `unwrap()` in production code
- Zero unsafe code

### V74 Deep Debt + ToadStool/barraCuda Catch-Up + Full Validation Benchmark (Mar 4, 2026)

#### Fixed
- **Clippy pedantic `let_and_return`**: `wdm.rs` map closure simplified to direct expression
- **Clippy pedantic `too_many_lines`**: `drift.rs` `wf_batch_gpu` refactored — extracted `wf_generate_prng_state` and `wf_readback_fixations` helpers
- **`eps::UNDERFLOW` dead code**: integrated into `linalg.rs` QL convergence detection against subnormal values

#### Changed
- **CI clippy pedantic enforcement**: all `cargo clippy` commands in `.github/workflows/ci.yml` now include `-W clippy::pedantic` across default, barracuda, and all-features modes
- **Tolerance deduplication**: `groundspring-validate` re-exports 7 core tolerance constants from `groundspring::tol` instead of defining local duplicates
- **`freeze_out.rs` magic numbers → named constants**: 7 L-BFGS constants (behind `#[cfg(feature = "barracuda")]`) + 3 Nelder-Mead constants (behind `#[cfg(feature = "barracuda-gpu")]`) + PRNG seed
- **`fao56/pipeline.rs` magic numbers → named constants**: `RH_MIN_FLOOR_PCT`, `RH_MAX_CEIL_PCT`, `RHMAX_FLOOR_PCT`, `WIND_SPEED_FLOOR_KMH` for Monte Carlo clamp bounds
- **`biomeos/mod.rs` magic numbers → named constants**: `DEFAULT_CONNECT_TIMEOUT_SECS`, `DEFAULT_READ_TIMEOUT_SECS`
- **`benchmark_cross_spring.rs`**: updated to ToadStool S93, barraCuda v0.3.1; added S89 budding and S90-S93 evolution timeline entries; provenance table expanded with barraCuda standalone and D-DF64 transfer
- **ABSORPTION_MANIFEST.md**: updated from V61 (32 delegations) to V74 (81 delegations) — full inventory with CPU/GPU breakdown, Tier B/C resolution
- **BARRACUDA_EVOLUTION.md**: V74 header + fresh benchmark table (28 binaries × 2 modes)
- **CROSS_SPRING_SHADER_EVOLUTION.md**: V74 benchmark results, S87–S93 + barraCuda budding section, universal precision architecture documented, cross-spring provenance for speedups

#### Quality
- `cargo fmt --check`: PASS
- `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic`: PASS (default + barracuda)
- `cargo doc --workspace --no-deps`: PASS
- `cargo test --workspace`: PASS (790 tests)
- `cargo test --workspace --features barracuda`: PASS
- **Validation binaries (default)**: 28/28 PASS, 21.7s total (release)
- **Validation binaries (barracuda CPU)**: 28/28 PASS, 19.6s total (release, **−10%**)
- **Cross-spring benchmark**: 23/23 PASS, 4.5s
- **barraCuda**: v0.3.1 (standalone primal, `f6895ca`)
- **toadStool**: S93 (`9319668d`)
- **Delegations**: 81 active (47 CPU + 34 GPU), unchanged

### V73 Tolerance Architecture + Epsilon Guards + Idiomatic Evolution (Mar 4, 2026)

#### Added
- **13-tier named tolerance module (`tol::`)**: `DETERMINISM` (1e-15), `STRICT` (1e-14), `EXACT` (1e-12), `ANALYTICAL` (1e-10), `INTEGRATION` (1e-8), `CDF_APPROX` (1e-6), `ROUNDTRIP` (1e-5), `RECONSTRUCTION` (1e-4), `LITERATURE` (0.001), `DECOMPOSITION` (0.005), `STOCHASTIC` (0.01), `NORM_2PCT` (0.02), `EQUILIBRIUM` (0.1) — each with scientific justification
- **Production epsilon guard module (`eps::`)**: `SAFE_DIV` (1e-10), `SSA_FLOOR` (1e-15, behind `barracuda-gpu` feature), `UNDERFLOW` (1e-300) — replaces inline magic numbers in drift, gillespie, anderson
- `tol` module now `pub` for integration test and downstream crate access

#### Changed
- **~170 bare float tolerance literals → named `tol::` constants** across 35 library modules and 6 integration test files — every assertion now carries semantic meaning
- **3 inline epsilon guards → `eps::` constants**: `drift.rs` division guard, `gillespie.rs` SSA floor, `anderson.rs` underflow guard
- **`f64::midpoint` applied**: overflow-safe midpoint in Spearman rank tie handling (`stats/correlation.rs`)
- **18 explicit `return` statements → tail expressions**: idiomatic Rust in cfg-gated functions across `stats/agreement.rs`, `stats/metrics.rs`, `stats/distributions.rs`, `kinetics.rs`, `rarefaction.rs`, `band_structure.rs`, `fao56/pipeline.rs`, `almost_mathieu.rs`
- **`biomeos/discovery.rs` capability evolution**: `NUCLEUS_SOCKET_NAMES` → `CAPABILITY_SOCKET_NAMES` with `find_capability_socket` fallback scan

#### Removed
- `eps::SAFE_DIV_STRICT` (unused outside validate crate; validate crate has its own `EPS_SAFE_DIV_STRICT`)

#### Quality
- `cargo fmt --check`: PASS
- `cargo clippy --workspace -- -D warnings`: PASS
- `cargo doc --workspace --no-deps`: PASS
- `cargo test --workspace`: PASS (790 tests)
- `cargo llvm-cov`: 97.25% line coverage (target 90%)
- Zero bare tolerance literals in library test code (all use `tol::*`)
- Zero inline epsilon guards in library production code (all use `eps::*`)

### V72 Deep Audit + Debt Evolution + Idiomatic Maturation (Mar 3, 2026)

#### Fixed
- **Clippy failure**: `missing_const_for_fn` in `freeze_out::nelder_mead_multi_start` — extracted shared `const fn validate_config_lengths()` used by both `grid_fit_2d` and `nelder_mead_multi_start` (DRY + const-correct)
- **Silent data loss**: `unwrap_or(0.0)` / `unwrap_or("")` in GHCND and IRIS JSON parsing replaced with `let Some(...) else { continue }` — invalid records now skipped instead of injected with zeroed defaults
- **Non-deterministic iteration**: `HashMap` → `BTreeMap` for GHCND temperature maps in `validate_real_ghcnd_et0.rs` — DOY assignment now reproducible across runs
- **Provenance enforcement**: `print_provenance_header` now `expect()`s mandatory `_source`, `baseline_commit`, `baseline_date` instead of defaulting to "unknown" — malformed benchmark JSON fails loudly

#### Changed
- **Bare `unwrap()` → descriptive `expect()`**: 10 sites across `bench_cpu_vs_gpu.rs`, `three_tier_parity_gpu.rs`, `three_tier_parity_bio.rs`, `validate_drift.rs`, `validate_transport.rs`, `drift.rs`
- **Hardcoded constants → documented `mod station`**: GHCND station params (ID, lat, alt, weather defaults) in `validate_real_ghcnd_et0.rs` now live in a documented `station` module with NOAA source URLs
- **Idiomatic iterator**: `find_band_edges` in `band_structure.rs` evolved from manual stateful `for` loop to `.map().collect()` + `.windows(2).filter_map()` pattern
- **Python CI coverage enforced**: Added `pytest --cov=control --cov-fail-under=80` to CI workflow

#### Added
- **4 analytical ODE tests**: exponential decay (`e^{-t}`), simple harmonic oscillator (energy conservation), coupled rotation (`sin/cos`), logistic growth (`1/(1+9e^{-t})`) — `ode` module coverage from 2 → 6 tests
- **3 benchmark JSON `data_origin` fields**: `sensor_noise`, `precision_drift`, `vendor_parity` — all 28/28 benchmarks now have complete provenance

#### Quality
- `cargo fmt --check`: PASS
- `cargo clippy --all-targets -D warnings`: PASS (was FAIL before V72)
- `cargo doc -D warnings`: PASS
- `cargo test --workspace`: PASS (all tests)
- 28/28 validation binaries: PASS (exit 0)
- 27/27 Python experiments: PASS (1 skip: NPU hardware)
- 252/252 Python baseline integrity: PASS

### V71 barraCuda 0.3.1 Pin + Ecosystem Maturation (Mar 3, 2026)

#### Changed
- **barraCuda pin**: `b53c3de` (v0.2.1) → `f6895ca` (v0.3.1) — tarpc parity, 73 new tests, IPC E2E, debt cleanup
- **toadStool pin**: S87 (`2dc26792`) → S93 (`9319668d`) — complete untangle, DF64 transfer, sovereignty audit
- **wateringHole sync**: Pulled 16 new handoffs documenting budding, untangle, DF64 transfer, spring absorptions
- **Maturity assessment**: barraCuda now Nascent-Stable (builds in CI, full toadStool untangle, 2,965 tests, version-aligned at 0.3.1)
- **Zero breaking impact**: 0.3.1 tarpc type changes (MatmulResult, FheResult, DispatchResult) do not affect groundSpring
- **Architecture clarified**: akida-driver stays with toadStool permanently (hardware, not math); barraCuda owns precision/quantization path (fp64 → int4)
- **Handoff**: V71 budding maturation handoff with ecosystem guidance

#### Quality
- 786+ tests passing, 0 failed
- `cargo clippy --all-features -- -W clippy::pedantic -W clippy::nursery`: zero warnings
- barraCuda toadStool dependency fully removed upstream (toadstool_integration.rs deleted, npu/ops deleted)
- barraCuda version alignment issue resolved (0.3.1 consistent across Cargo.toml, CHANGELOG, spec)

### V70 barraCuda Budding — Standalone Primal (Mar 3, 2026)

#### Changed
- **barracuda dependency**: Rewired from `phase1/toadstool` to standalone `barraCuda` primal at `ecoPrimals/barraCuda/`
- **New S80-S87 primitive delegations**: StatefulPipeline, BatchedEncoder, batched_nelder_mead_gpu, device-lost resilience, spectral diagnostics
- **Handoff**: Updated wateringHole handoff for barraCuda budding from phase1/toadstool to standalone primal

---

For entries prior to V70 (Feb 25 – Mar 2, 2026), see [CHANGELOG_ARCHIVE.md](CHANGELOG_ARCHIVE.md).
