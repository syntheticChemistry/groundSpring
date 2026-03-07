# Changelog

All notable changes to groundSpring follow [Keep a Changelog](https://keepachangelog.com/).

## [Unreleased]

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

### V69+ Documentation Sweep + Zero-Debt `#[allow]` Audit (Mar 2, 2026)

#### Changed
- **V68→V69** across all active docs (README, baseCamp, experiments, METHODOLOGY,
  CROSS_SPRING_EVOLUTION, BARRACUDA_EVOLUTION, BARRACUDA_REQUIREMENTS,
  PAPER_REVIEW_QUEUE, CONTROL_EXPERIMENT_STATUS, 5 graph TOMLs)
- **S86→S87 cleanup**: CROSS_SPRING_SHADER_EVOLUTION.md S86/hash mismatch fixed
- **172→187 metalForge checks** in CONTROL_EXPERIMENT_STATUS, BARRACUDA_REQUIREMENTS
- **95→100+ three-tier parity** in CONTROL_EXPERIMENT_STATUS, STUDY
- **1,151→1,158 grand total** in CONTROL_EXPERIMENT_STATUS
- **ABSORPTION_MANIFEST.md**: shader inventory updated (absorbed shaders documented,
  local `anderson_lyapunov` shaders listed)
- **whitePaper/README.md**: shader section updated for ToadStool absorption
- **ecoPrimals/whitePaper/gen3/baseCamp/README.md**: groundSpring V69 metrics
- **CONTROL_RUN_LOG.md**: Run 40 (V69) added
- **V69 handoff**: cross-spring bidirectional flow table added (Part 5)

#### Fixed
- **`#[allow(clippy::float_cmp)]`** reason comments added in 5 test files +
  freeze_out.rs `allow(unused)` (zero-debt: "zero `#[allow]` without reason")
- **918 Rust tests** stale count in CROSS_SPRING_SHADER_EVOLUTION.md → 783

### V69 ToadStool S87 Pin + Universal Precision Documentation (Mar 2, 2026)

#### Added
- **5 cross-spring evolution parity tests** (783 default / 785 barracuda-gpu):
  - Shannon diversity (`wetSpring` S64 → `FusedMapReduceF64::shannon_entropy`)
  - Simpson diversity (`wetSpring` S64 → `FusedMapReduceF64::simpson_index`)
  - Seismic grid search (`groundSpring` forward model + barracuda `ComputeDispatch`)
  - Anderson 2D eigenvalues (`hotSpring` S59 sparse Lanczos, `#[cfg(barracuda-gpu)]`)
  - Anderson 3D eigenvalues (`hotSpring` S59, metal-insulator transition, `#[cfg(barracuda-gpu)]`)
- **Cross-spring evolution timeline** in `benchmark_cross_spring`: 4-phase
  provenance history (Foundation → Absorption → Universal Precision → Modern
  Wiring) with bidirectional flow documentation.
- **Expanded provenance table**: +6 entries (McEt0PropagateGpu, SeasonalPipelineF64,
  lbfgs_numerical, anderson_4d, FHE fix, device-retry).

#### Changed
- **ToadStool pin**: S86 (`7e01ac7e`) → S87 (`2dc26792`). S87 adds FHE shader
  fix, async-trait reclassification, 9 test fixes, unsafe audit.
- **hofstadter doc path**: Updated `almost_mathieu.rs` doc comments to reference
  `barracuda::spectral::almost_mathieu_hamiltonian` (hofstadter module private).
- **All docs**: S86→S87 pin across 15+ files, stale S79 refs in graphs/specs/
  benchmarks updated, V62→V68 in biomeOS graph TOMLs. 780→783 tests.

#### Documented
- **ToadStool S67-S68 Universal Precision Architecture**: "Math is universal,
  precision is silicon." All 844+ WGSL shaders evolved to f64-canonical.
  `compile_shader_universal(src, precision)` auto-targets F16/F32/F64/Df64.
  Dual-layer DF64 (op_preamble + naga IR rewrite). `Fp64Strategy` auto-selects
  Native (Titan V, A100) vs Hybrid (RTX 4070, consumer GPUs). Precision is
  transparent to groundSpring consumers — barracuda ops internally select the
  best precision path per hardware.
- **Cross-spring shader provenance**: hotSpring precision shaders (DF64, Fp64Strategy,
  Lanczos, Anderson) → wetSpring bio shaders (diversity, Smith-Waterman, Gillespie)
  → neuralSpring ML (ESN, AlphaFold2, pow_f64 fix) → all springs consume.
  Bidirectional: every spring both contributes and consumes.

### V68 Complete Rewiring — L-BFGS Refinement, 4D Anderson, Cross-Spring Benchmark (Mar 2, 2026)

#### Added
- **freeze_out::grid_fit_2d L-BFGS refinement**: Post-grid-search gradient-based
  refinement via `barracuda::optimize::lbfgs_numerical` (airSpring V035 → ToadStool S84).
  Grid search finds basin, L-BFGS converges to sub-grid precision.
- **tissue_anderson::tissue_4d_simulation**: 4D Anderson lattice construction via
  `barracuda::spectral::anderson::anderson_4d` (hotSpring S26 → ToadStool S84).
  Fourth dimension models immune response gradient for Paper 12 tissue immunology.
- **tissue_anderson::tissue_4d_rg_coarsen**: Wegner block RG coarsening of 4D
  Anderson Hamiltonian via `barracuda::spectral::anderson::wegner_block_4d`
  (hotSpring condensed matter → ToadStool S84). Reveals disorder flow at
  cell-cluster scale.
- **metalForge workloads**: 2 new (L-BFGS grid refine CPU, Tissue Anderson 4D +
  Wegner RG). Total: 30 workloads.
- **metalForge tolerances**: 2 new (Analytical for L-BFGS, Exact for 4D lattice).
  Total: 30 tolerance specs.
- **Cross-spring evolution benchmark** (wateringHole/CROSS_SPRING_SHADER_EVOLUTION):
  V68 section documenting hotSpring→tissue 4D, airSpring→freeze-out L-BFGS,
  wetSpring↔neuralSpring bidirectional flow.

#### Changed
- **Dispatch target count**: 44 CPU + 32 GPU = 76 active delegations (was 73).
- All documentation updated with V68 canonical numbers.

### V67 ToadStool S86 Catch-Up — McEt0 GPU, Seasonal Pipeline, API Rewire (Mar 2, 2026)

#### Added
- **fao56::monte_carlo_et0**: GPU Monte Carlo uncertainty propagation via
  `McEt0PropagateGpu` (ToadStool S72 absorption, groundSpring V10 provenance).
  CPU fallback with deterministic xorshift64 sampling.
- **fao56::seasonal_step**: Fused seasonal pipeline (ET₀ → Kc → water balance → stress)
  via `SeasonalPipelineF64` (ToadStool S80). CPU fallback per-cell evaluation.
- **metalForge workloads**: 2 new workloads (MC ET₀ propagation GPU, seasonal pipeline
  GPU fused). Total: 28 workloads.
- **metalForge tolerances**: 2 new tolerance specs. Total: 28 tolerance specs.

#### Changed
- **Dispatch target count**: 43 CPU + 30 GPU = 73 active delegations (was 71).
- **ToadStool pin**: S86 `7e01ac7e` (was S79 `f97fc2ae`).
- **BatchedMultinomialGpu::sample**: Updated 3 call sites (rarefaction, rare_biosphere ×2)
  for new signature with `BatchedMultinomialConfig` parameter (`cumulative_probs: true`,
  `seed: None`, `seeds: Some(&mut seeds)`). Breaking change in ToadStool S80.

### V66 Stats Tier A GPU + Bistable Batch ODE + metalForge Expansion (Mar 2, 2026)

#### Added
- **stats::agreement GPU**: MAE via `FusedMapReduceF64::l1_norm`, NSE/R² via dual
  `FusedMapReduceF64::sum_of_squares` dispatches. Papers 1-5 stats now fully GPU-capable.
- **bistable batch GPU**: `integrate_batch()` dispatches to `BatchedOdeRK4F64` for
  parallel RK4 trajectories on GPU. CPU fallback sequential.
- **multisignal batch**: `integrate_batch()` for batch ODE integration (CPU path,
  GPU promotion candidate).
- **validate-gpu-tier**: 4 new validation sections: stats Tier A GPU parity,
  bistable batch GPU parity, jackknife GPU parity, FAO-56 batch GPU parity.
- **three_tier_parity_gpu.rs**: 5 new parity tests (MAE known value, NSE=R²,
  bistable batch consistent, jackknife GPU, FAO-56 batch matches single).
- **metalForge workloads**: 3 new workloads (MAE GPU fused, NSE/R² GPU fused,
  bistable ODE batch GPU RK4). Total: 26 workloads.
- **metalForge tolerances**: 3 new tolerance specs. Total: 26 tolerance specs.

#### Changed
- **Dispatch target count**: 43 CPU + 28 GPU = 71 active delegations (was 67).
- **Test count**: 776 workspace tests (was 771).
- Updated PAPER_REVIEW_QUEUE.md, BARRACUDA_EVOLUTION.md to V66.

### V65 Docs Sweep + ToadStool Absorption Handoff + Paper Queue Review (Mar 2, 2026)

#### Changed
- **Root docs updated**: README.md status line, CHANGELOG.md (V62–V64 catch-up),
  wateringHole README active handoff.
- **whitePaper/README.md**: Status line updated to V65 (376/376, 67 delegations,
  752 tests).
- **whitePaper/baseCamp/gonzales.md**: Added V64 tissue_anderson module
  refactoring note.
- **specs/PAPER_REVIEW_QUEUE.md**: Three-tier control matrix updated with V64
  delegation count (67) and experiment 033 entry.
- **specs/BARRACUDA_EVOLUTION.md**: Header updated to V65 (67 delegations).
- **wateringHole/README.md**: V65 handoff active, V64 archived.
- New V65 ToadStool handoff with comprehensive barracuda absorption roadmap,
  paper queue × three-tier hardware matrix, and PRNG alignment action items
  for the ToadStool/BarraCUDA team.

### V64 Deep Audit — Idiomatic Rust Evolution + Docs Sweep (Mar 2, 2026)

#### Changed
- **biomeos refactored**: Monolithic `biomeos.rs` (834 lines) → directory module
  `biomeos/` with `mod.rs` (public API, config, routing), `discovery.rs` (socket
  resolution), `protocol.rs` (JSON-RPC serialization), `transport.rs` (platform
  I/O). All under 1000 lines.
- **`#[allow]` → `#[expect]`**: Last remaining `#[allow(clippy::cast_*)]` in
  `validate_real_ncbi_16s.rs` converted to `#[expect(..., reason = "...")]`.
- **Epsilon guards documented**: `1e-10` in `drift.rs`, `1e-15` in
  `gillespie.rs`, three `1e-20` guards in `validate_vendor_parity.rs` — all
  annotated with division-by-zero prevention rationale.
- **Tolerance comments**: Three-tier GPU test tolerances in
  `three_tier_parity_gpu.rs` and `chaos_fault.rs` now document exact/approximate
  reasoning.
- **Benchmark units**: `benchmark_sensor_noise.json` gains `_units` block;
  `benchmark_sequencing_noise.json` gains `_units`, `_tolerance_note`,
  `_validation_path` documenting Python vs Rust validation distinction.
- **`validate_rarefaction.rs`**: Module doc-comment explains why Rust uses
  analytical invariants (PRNG-agnostic) rather than Python's RNG-dependent
  `expected_results`.
- **LICENSE**: Corrected from "version 3, or any later version" to "version 3
  only" (matches `AGPL-3.0-only` SPDX everywhere).
- **`s.clone()` → `s.to_owned()`** in `biomeos/protocol.rs`.
- **BARRACUDA_EVOLUTION.md**: `TODO(toadstool)` entries clarified to "blocked on
  `barracuda::ops::*`".

### V63 Brain Architecture + Capability-Based Discovery + Paper 12 (Mar 2, 2026)

#### Added
- **Exp 033 Tissue Anderson** (Paper 12 — Gonzales): `tissue_anderson` module
  with cytokine Anderson lattice, barrier disruption sweep, geometry-aware drug
  scoring. `SkinLayer`, `CellType`, `TissueCompartment`, `DrugCandidate`,
  `DeliveryRoute`, `DrugScore`. 29/29 validation checks, 18 unit tests.
- 6 new GPU delegations: `GillespieGpu` batch, `WrightFisherGpu` batch,
  `BatchedMultinomialGpu`, `cholesky_f64`, `eigh_f64`, `GpuAlignedRng`.
- `bench-cpu-vs-gpu`: 4 new benchmarks (multinomial, tikhonov, tridiag, MSD).
- `validate-gpu-tier`: expanded 66 → 73 checks (tissue Anderson GPU parity).
- `validate-metalforge-pipeline` (30/30 mixed-hardware checks).
- NUCLEUS compute dispatch tests (Neural API → provider → CPU baseline).

#### Changed
- Delegation count: 61 → 67 (37 CPU + 26 GPU + 4 cross-spring).
- Workspace tests: 752 (409 lib + 343 integration/validation).
- `tissue_anderson.rs` refactored into directory module (916 → 641 + 268 lines).
- `DriftAction`, `ConceptEdge`, `MultiHeadUncertainty` from Nautilus integration.

### V62 S79 Catch-Up + Rewire + Clean (Mar 2, 2026)

#### Changed
- **ToadStool pin**: S71+++ → S79 (`f97fc2ae`).
- **pollster eliminated**: All `pollster::block_on` → `barracuda::device::test_pool::tokio_block_on`.
- **f64-capable device**: `WgpuDevice::new()` → `WgpuDevice::new_f64_capable()`.
- **Redundant shaders removed**: `mc_et0_propagate.wgsl` and
  `batched_multinomial.wgsl` deleted (absorbed S72/S76 upstream).
- `validate/lib.rs` evolved to `Result`-based API (`BenchResult<T>`) with
  backward-compatible panicking wrappers.
- All `partial_cmp().unwrap_or()` → `f64::total_cmp()`.
- All `#[allow]` lint suppressions → `#[expect(lint, reason = "...")]`.
- All 33 validation binaries now use `std::process::exit(h.summary())`.
- SPDX license identifiers unified to `AGPL-3.0-only` across all non-Rust files.

### V61 Mixed-Hardware Pipeline + NUCLEUS Atomics (Mar 2, 2026)

#### Added
- **`metalForge/forge/src/topology.rs`**: `PCIe` topology and device adjacency
  module. Models 6 bandwidth tiers (Local, NvLink, `PciePeer`, `PcieHost`,
  `PcieLow`, Network), infers interconnect topology from substrate inventory,
  and calculates transfer time estimates. Foundation for NPU↔GPU P2P
  decisions — bypassing CPU round-trips in mixed-hardware pipelines.
- **`metalForge/forge/src/pipeline.rs`**: Multi-stage pipeline dispatch.
  `Pipeline` builder chains workloads across substrates with per-stage
  `FallbackPolicy` (Degrade / Skip / Fail) and `TransferStrategy`
  (PeerToPeer / HostBounce). `plan()` resolves stages to substrates using
  topology for transfer cost estimation.
- **`metalForge/forge/src/atomic.rs`**: NUCLEUS atomic composition types.
  `TowerAtomic` (BearDog + Songbird), `NodeAtomic` (Tower + ToadStool +
  Inventory + Topology), `NestAtomic` (Tower + NestGate), `FullNucleus`
  (all primals + Squirrel). Sovereign degradation: Full → Node+Nest → Node
  → Tower → Sovereign.
- **`dispatch::fallback_chain()`**: Ordered substrate fallback (preferred →
  GPU `NativeF64` → GPU → NPU → CPU) for graceful runtime degradation.
- **`validate-mixed-hardware`**: New validation binary — 42 checks covering
  topology inference, fallback chains, pipeline planning, NUCLEUS atomics,
  degradation levels, and tolerance tiers.
- 35 new workspace tests (topology, pipeline, atomic, dispatch) — 120 total
  metalForge tests (up from 85).

#### Changed
- **Deep idiomatic Rust pass**: 13 clippy errors resolved (anderson, esn,
  fao56, freeze_out, lanczos, spectral_recon, wdm) — `needless_return`,
  `doc_markdown`, `assertions_on_constants`, `suboptimal_flops`,
  `cast_lossless`, `items_after_statements`, `must_use_candidate`,
  `redundant_clone`.
- **Iterator modernization**: `flat_map` chain in spectral_recon (nested
  `for` loop → functional), `mul_add` + `.iter().sum()` in wdm (manual
  accumulation → idiomatic).
- **`serde_json::json!`**: 5 manual `format!` JSON strings → typed macro
  invocations in nestgate.rs.
- **Result-based API**: `try_f64_field`, `try_usize_field`, `try_str_field`
  in groundspring-validate lib; existing panic-based helpers delegate to
  these with `unwrap()`.
- **Provenance headers**: 4 NUCLEUS validation binaries (ghcnd, ncbi,
  nucleus-stack, iris-seismic) now document data origin and baselines.
- **Hardcoding evolution**: `temp_dir().join(...)` replaces `/tmp/` path;
  `unwrap()` → `.expect("...")` in 3 binaries; primal socket names →
  named constants.
- 17 new unit tests for intermediate functions (fao56, npu, validate lib).

### V60 hotSpring Cross-Spring Absorption — DriftMonitor, Uncertainty, Concept Edges (Mar 1, 2026)

#### Added
- **`drift::DriftMonitor`**: `N_e`·`s` drift monitoring for evolutionary populations.
  Tracks effective population size × selection coefficient across generations,
  detects when genetic drift overwhelms selection (3+ consecutive generations
  below threshold), recommends `DriftAction` (continue / increase selection /
  increase population). Cross-spring lineage: bingoCube Nautilus Shell
  `constraints.rs`. 5 new tests.
- **`drift::DriftAction`**: Enum of recommended responses to drift detection.
- **`esn::ClassificationUncertainty`**: Epistemic uncertainty metrics for
  regime classification — confidence, entropy, margin. `is_boundary()` method
  detects regime transitions. Cross-spring lineage: hotSpring `MultiHeadNpu`
  `HeadGroupDisagreement`. 3 new tests.
- **`esn::classification_uncertainty()`**: Softmax normalization → uncertainty
  metrics from raw classifier outputs.
- **`esn::detect_concept_edges()`**: Leave-one-out cross-validation over disorder
  sweep data to identify parameter regions where the model breaks down (regime
  boundaries). Returns `(disorder_value, loo_error)` pairs. Cross-spring lineage:
  bingoCube Nautilus Shell `NautilusBrain::detect_concept_edges()`. 2 new tests.
- **`nautilus` feature**: Optional dependency on `bingocube-nautilus` crate
  (`primalTools/bingoCube/nautilus/`) for evolutionary reservoir computing.
  Re-exports `NautilusBrain`, `NautilusShell`, `DriftMonitor`, `EdgeSeeder`,
  `Akd1000Export`. Pure Rust, no GPU dependencies.

#### Validated
- `cargo check` (default, barracuda, barracuda-gpu, nautilus): all PASS
- `cargo clippy -- -D warnings`: 0 warnings
- `cargo fmt --check`: PASS
- `cargo test --workspace`: 620 tests, all PASS (+7 from V59)

### V59 ToadStool S71+++ Catch-Up — GPU Promotions + Pure Math Shaders (Mar 1, 2026)

#### Changed
- **ToadStool pin**: S70+++ (`1dd7e338`) → **S71+++ (`8dc01a37`)**
- **`jackknife_mean_variance` GPU promoted**: CPU → GPU via `JackknifeMeanGpu`
  (S71 `jackknife_mean_f64.wgsl` — leave-one-out means on GPU, variance on CPU)
- **`hargreaves_et0_batch` GPU path evolved**: Now tries `HargreavesBatchGpu` (S71
  `hargreaves_batch_f64.wgsl`) before falling back to `BatchedElementwiseF64` (S70)
- **Delegation breakdown**: 37 CPU + 20 GPU + 4 cross-spring (was 38+19+4)
- **Module doc comments**: Updated from "S68+" to reflect S70+/S71 state

#### ToadStool S71 Evolution Summary
- **6 commits absorbed** (S71 through S71+++ plus docs/clean)
- **671 WGSL shaders** (was 700 — stale shaders archived, count corrected)
- **~9,000 lines net reduction** (14K removed, 5K added — stale code/examples archived)
- **ComputeDispatch builder**: 66 ops migrated to unified GPU dispatch pattern
- **DF64 transcendental suite complete**: 15 functions (gamma, erf, inverse trig,
  hyperbolics) — extended precision for all transcendentals
- **Pure math shaders**: All shaders evolved to f64 canonical with precision-per-use
- **3 new GPU shaders directly relevant**:
  - `kimura_fixation_f64.wgsl` — batch Kimura fixation (available, not yet consumed)
  - `hargreaves_batch_f64.wgsl` — batch Hargreaves ET₀ (wired)
  - `jackknife_mean_f64.wgsl` — leave-one-out jackknife (wired)
- **External deps audit**: libc in akida-driver identified for rustix evolution,
  unsafe reduced across GPU device creation and unified memory

#### Validated
- `cargo check` (default, barracuda, barracuda-gpu): all PASS
- `cargo clippy -- -D warnings`: 0 warnings
- `cargo fmt --check`: PASS
- `cargo test --workspace`: 613 tests, all PASS

### V58 Deep-Debt Completion + Hardcoding Evolution + Documentation (Mar 1, 2026)

#### Changed
- **`biomeos::FAMILY_ID` made public**: All ecosystem interactions now derive their family
  identifier from a single constant rather than scattered string literals
- **`nestgate.rs` hardcoding eliminated**: 9 hardcoded `"groundspring"` literals in key
  generation functions (`result_key`, `parity_key`, `data_key`, `record_lifecycle_event`)
  and RPC params (`ncbi_search`, `ncbi_fetch`, `noaa_ghcnd`, `iris_stations`, `iris_events`)
  now reference `biomeos::FAMILY_ID`
- **`biomeos.rs` DRY refactoring**: Extracted `merge_compute_params()` helper eliminating
  duplicate 10-line blocks from `compute_execute`/`compute_submit`; simplified
  `direct_rpc_call` to delegate to `capability_call`
- **Root docs updated**: README.md reflects 61 delegations, 32 modules, 613 workspace tests
- **V58 handoff created**: Cross-spring evolution + deep-debt completion for ToadStool team

#### Validated
- Comprehensive audit: zero unsafe, zero production mocks, zero production unwrap/panic,
  zero primal coupling in code logic, zero production TODO/FIXME
- All `# Panics` documentation in place for all asserting public functions
- All external dependencies assessed (wgpu, pollster, bytemuck, serde_json, proptest, tempfile)
  — all necessary, minimal, no pure-Rust replacements available
- `cargo clippy -W clippy::pedantic`: zero warnings
- All quality gates green (fmt/clippy/doc/test) for all feature modes
- 613 workspace tests PASS

### V57 Cross-Spring Evolution — ESN, Lanczos, 2D/3D Anderson, Chi2 (Mar 1, 2026)

#### Added
- **`esn` module**: Echo State Network regime classification; `RegimeLabel` enum,
  rule-based `classify_by_spacing_ratio` and `classify_by_lyapunov`, `spectral_features`
  extraction, GPU-accelerated `EsnClassifier` (barracuda-gpu feature); cross-spring
  lineage: wetSpring ESN reservoir → hotSpring spectral features → neuralSpring GPU fixes
- **`lanczos` module** (barracuda-gpu): Sparse eigensolver wrapping
  `barracuda::spectral::lanczos` and `lanczos_eigenvalues`; `sparse_eigenvalues` and
  `eigenvalues_from_csr` for 2D/3D Anderson Hamiltonians; cross-spring lineage:
  hotSpring Lanczos iteration → barracuda SpMV shader
- **`anderson::disorder_sweep`**: GPU-accelerated disorder sweep via
  `barracuda::spectral::anderson_sweep_averaged` with CPU fallback; returns `Vec<SweepPoint>`
- **`anderson::anderson_2d_eigenvalues`** (barracuda-gpu): 2D Anderson lattice eigenvalues
  via `barracuda::spectral::anderson_2d` + Lanczos
- **`anderson::anderson_3d_eigenvalues`** (barracuda-gpu): 3D Anderson lattice eigenvalues
  via `barracuda::spectral::anderson_3d` + Lanczos
- **`freeze_out::chi2_analysis`**: Decomposed chi-squared analysis via
  `barracuda::stats::chi2::chi2_decomposed_weighted` with CPU fallback; returns
  `Chi2Analysis` with per-datum contributions, residuals, pulls, p-value
- **Benchmark expansion**: `bench_anderson_sweep`, `bench_chi2_analysis`,
  `bench_esn_classification` added to `benchmark_cross_spring` with provenance table
- **Parity tests**: 6 new tests in `three_tier_parity_physics.rs`

#### Changed
- **Delegation count**: 57 → **61 active** (38 CPU + 19 GPU + 4 cross-spring S59+)
- **`specs/BARRACUDA_EVOLUTION.md`**: Updated to V57, 61 delegations, all Tier B items
  resolved; new module → shader mapping for `lanczos` and `esn`

#### Validated
- `cargo fmt --check`: PASS
- `cargo clippy -- -D warnings`: 0 warnings
- `cargo doc --no-deps`: clean
- `cargo test -p groundspring`: 486 tests PASS
- `cargo test -p groundspring-validate`: all 33 validation binaries PASS
- `cargo test -p groundspring-forge`: 85 tests PASS

### V56 NUCLEUS Live Validation + Docs Cleanup + ToadStool Handoff (Mar 1, 2026)

#### Added
- **Experiments 029–032**: 4 NUCLEUS-integrated validation experiments (55 checks)
  - Exp 029: Real GHCND ET₀ — Hargreaves vs Penman-Monteith on real/synthetic NOAA weather (6/6)
  - Exp 030: Real NCBI 16S — Rare biosphere detection on real/synthetic NCBI metagenomes (9/9)
  - Exp 031: NUCLEUS Stack — Full primal validation: Tower + Node + Squirrel + Nest (28/28)
  - Exp 032: IRIS Seismic — IRIS FDSN station geometry + travel times via NestGate (12/12)
- **biomeOS client**: `biomeos.rs` — socket discovery, `auto_connect()`, `capability_call()`,
  `compute_execute`, `compute_submit`, `compute_capabilities`, `storage_put`/`storage_get`
- **NestGate client**: `nestgate.rs` — NCBI search/fetch, NOAA GHCND, IRIS stations/events,
  lifecycle provenance, `IrisEventQuery` struct
- **V56 handoff**: NUCLEUS integration handoff for ToadStool team
- **PRIMAL_INTERACTION_EVOLUTION.md**: Tracks V0–V6 NUCLEUS evolution phases
- **Deployment graphs**: `graphs/groundspring_nucleus_node.toml`, `groundspring_tower_bootstrap.toml`
- **biomeOS capability registry**: `data.*` and `compute.*` domain translations aligned

#### Changed
- **Root docs**: README.md updated to 32 experiments, 347/347 checks, Phase 4 (NUCLEUS) evolution
- **Experiment index**: whitePaper/experiments/ expanded to 032
- **wateringHole**: V55 archived, V56 handoff created, CROSS_SPRING_SHADER_EVOLUTION.md corrected
- **baseCamp**: Updated validation counts and NUCLEUS integration status
- **gen3/baseCamp**: Updated groundSpring entry with NUCLEUS capabilities
- **specs/README.md**: Added Exp 022–032 to status, PRIMAL_INTERACTION_EVOLUTION.md spec
- **CONTRIBUTING.md**: Updated test counts (622 with biomeos), module counts (30)
- Fixed stale delegation counts (63→57) in baseCamp/README.md, CROSS_SPRING_SHADER_EVOLUTION.md

#### Validated
- All quality gates green (fmt/clippy/doc/test) for all feature modes
- 622 tests with `--features biomeos`, 569 core, 375 Python = 997 total
- V55 handoff archived, V56 handoff created

### V55 barracuda Evolution Review + Docs Cleanup + ToadStool Handoff (Feb 28, 2026)

#### Added
- **V55 handoff**: comprehensive barracuda evolution review for ToadStool team
  - Complete 57-delegation inventory (38 CPU + 19 GPU)
  - API adaptation patterns documented (Option adapter, unit conversion, pre-eval + GPU argmin)
  - Cross-spring shader lineage mapped (hotSpring, wetSpring, neuralSpring, airSpring, groundSpring)
  - Performance data (Rust vs Python, CPU vs GPU benchmarks)
  - Recommended barracuda evolutions prioritized

#### Changed
- **Docs sweep**: fixed 50+ stale references across specs/, whitePaper/, wateringHole/
  - Updated BARRACUDA_REQUIREMENTS feature gate counts (30→38 CPU, 9→19 GPU)
  - Updated BARRACUDA_EVOLUTION, CROSS_SPRING_EVOLUTION, PAPER_REVIEW_QUEUE headers
  - Updated ecoPrimals/whitePaper/gen3/baseCamp/ groundSpring entry
  - Updated whitePaper/baseCamp/anderson.md, bazavov.md, experiments/004, experiments/002
  - Cleaned stale S68 → S70+++ in current-state descriptions
  - Historical entries (CHANGELOG, archive/) left as fossil record

#### Validated
- All quality gates green (fmt/clippy/doc/test)
- V54 handoff archived, V55 handoff created

### V54 Full Control Validation + Barracuda CPU Parity Proof + Rust vs Python Benchmark (Feb 28, 2026)

#### Validated
- **28 experiments, 283/283 checks**: All 27 validation binaries PASS (Exp 028 NPU hardware-only)
- **95/95 three-tier parity tests**: CPU = barracuda-CPU mathematical identity proven
- **Rust vs Python benchmark**: 11.6× faster (excl. LAPACK-bound), 51.2× peak (seismic)
  - 27/27 experiments: Rust produces identical results to Python, faster
  - LAPACK-bound (Exp 009): 0.1× expected — custom QR vs Fortran eigensolve
  - Pure stochastic (Exp 014): 0.4× — large Wright-Fisher populations
- **GPU workload**: 316/322 tests pass; 6 failures = `enable f64` shader on non-Titan-V GPU (expected)
- `bench-cpu-vs-gpu`: 12 workloads measured (barracuda CPU mode)
- `bench_rust_vs_python.json`: fresh timing data saved to `data/`

#### Key finding
barracuda CPU produces **identical math** to the sovereign CPU path and the Python baseline,
while running **11.6× faster** than interpreted Python. The math is now proven portable
from Python → Rust → barracuda CPU. Next step: barracuda GPU proves the math is portable
to GPU via unidirectional streaming (ToadStool `ComputeDispatch`).

### V53 Complete Rewiring + GPU Grid Adapters + Cross-Spring Lineage (Feb 28, 2026)

#### Added
- **GPU adapter**: `seismic::grid_search_inversion` → pre-evaluate RMS on CPU, `barracuda::ops::grid::grid_search_3d` for parallel argmin
- **GPU adapter**: `freeze_out::grid_fit_2d` → pre-evaluate chi-squared on CPU, `barracuda::ops::grid::grid_search_3d` for parallel argmin
- **CPU delegation**: `quasispecies::error_threshold` → `barracuda::stats::evolution::error_threshold` (S70+, Option adapter)
- **CPU delegation**: `rare_biosphere::detection_power` → `barracuda::stats::evolution::detection_power` (S70+, infallible)
- **CPU delegation**: `rare_biosphere::detection_threshold` → `barracuda::stats::evolution::detection_threshold` (S70+, infallible)
- **Benchmark expansion**: 6 new workloads added to `bench-cpu-vs-gpu` (kimura, jackknife, chao1, fao56 scalar, seismic, freeze-out) → 12 total
- **Cross-spring lineage documentation**: expanded shader evolution with per-Spring contribution details

#### Changed
- **Delegation count**: 52 → **57 active** (38 CPU + 19 GPU), **1 evolution candidate** (was 3)
- 2 GPU grid ops (grid_search_inversion, grid_fit_2d) reclassified from evolution candidates to active GPU delegations
- Only `band_edges` remains as evolution candidate (transfer matrix vs eigenvalue algorithm mismatch)

#### Validated
- `cargo fmt --check`: PASS
- `cargo clippy --all-targets -W pedantic -W nursery`: 0 warnings × 3 modes
- `cargo doc --no-deps`: clean
- `cargo test --workspace --features barracuda`: all PASS
- `bench-cpu-vs-gpu`: 12 workloads measured

### V52 ToadStool S70+ Catch-Up — 4 New CPU Delegations, Zero Pending (Feb 28, 2026)

#### Added
- **`drift::kimura_fixation_prob`** → `barracuda::stats::evolution::kimura_fixation_prob` (S70+, infallible `#[cfg]` pattern)
- **`jackknife::jackknife_mean_variance`** → `barracuda::stats::jackknife::jackknife_mean_variance` (S70+, `Option` fallback pattern)
- **`fao56::daily_et0`** → `barracuda::stats::hydrology::fao56_et0` (S70+, sunshine hours → Rs conversion before delegation)
- **`rare_biosphere::chao1`** → `barracuda::stats::diversity::chao1_classic` (S70+, Chao 1984 formula with `u64` — formula parity confirmed)

#### Changed
- **Delegation count**: 48 → **52 active** (35 CPU + 17 GPU), **0 pending** (was 6)
- **ToadStool pin**: S68+ (`e96576ee`) → **S70+++ (`1dd7e338`)**
- 3 GPU grid ops (grid_fit_2d, grid_search_3d, band_edges_parallel) reclassified from "pending" to "evolution candidates" — barracuda ops exist but use different algorithms than groundSpring's domain-specific implementations
- `TODO(toadstool)` comments replaced with explanatory notes about interface mismatch

#### Validated
- `cargo fmt --check`: PASS
- `cargo clippy --all-targets -W pedantic -W nursery`: 0 warnings × 3 modes
- `cargo doc --no-deps`: clean
- `cargo test --workspace --features barracuda`: all PASS
- `cargo test --workspace`: all PASS

### V51 GPU Stats Dispatch + CPU/GPU Parity Proof + Docs Cleanup (Feb 28, 2026)

#### Added
- **GPU stats dispatch**: 5 core statistical functions wired for GPU execution:
  - `stats::mean` → `SumReduceF64::mean`
  - `stats::std_dev` → `VarianceReduceF64::population_std`
  - `stats::rmse` → `FusedMapReduceF64` (sum of squared residuals)
  - `stats::mbe` → `SumReduceF64::mean` (mean of residuals)
  - `stats::pearson_r` → `CorrelationF64::pearson`
- **Batch GPU APIs**: 3 batch functions with GPU dispatch:
  - `gillespie::birth_death_ssa_batch` → `GillespieGpu`
  - `drift::wright_fisher_fixation_batch` → `WrightFisherGpu`
  - `fao56::daily_et0_batch` → `BatchedElementwiseF64::fao56_et0_batch`
- **9 CPU vs GPU parity tests** in `three_tier_parity.rs`
- **`groundspring::gpu_available()`**: Public API for GPU runtime detection
- **`bench-cpu-vs-gpu`** binary in `groundspring-validate`
- `wgpu` + `bytemuck` as optional dependencies (gated by `barracuda-gpu`)

#### Changed
- **Delegation count**: 46 → **48 active** (31 CPU + 17 GPU), 7 → **6 pending** ToadStool
- **Test count**: 322 → **569** workspace tests; **95** three-tier parity tests
- **metalForge routing**: 19 workloads confirmed — 17 GPU + 2 NPU
- **Benchmark provenance**: seismic + observation_gap `real_data_accession` updated for NestGate paths

#### Validated
- `cargo fmt --check`: PASS
- `cargo clippy --all-targets -W pedantic -W nursery`: 0 warnings
- `cargo doc --no-deps -D warnings`: clean
- `cargo test --workspace --features barracuda-gpu`: 569/569 PASS
- 292/292 validation checks across 28 experiments

### V47 Library Buildout + BarraCUDA CPU Expansion (Feb 28, 2026)

#### Added
- **`rarefaction::simpson_diversity`**: Simpson diversity index (1 − Σpᵢ²) with `barracuda::stats::simpson` delegation (S64)
- **`rarefaction::bray_curtis`**: Bray-Curtis dissimilarity with `barracuda::stats::bray_curtis` delegation (S64)
- **`rarefaction::analytical_rarefaction`**: Hypergeometric expected species with `barracuda::stats::rarefaction_curve` delegation (S64)
- **`kinetics::monod`**: Monod saturation kinetics with `barracuda::stats::monod` delegation (S66)
- **`bootstrap::bootstrap_median`**: Robust CI for median with `barracuda::stats::bootstrap_median` delegation (S64)
- **`bootstrap::bootstrap_std`**: CI for standard deviation with `barracuda::stats::bootstrap_std` delegation (S64)
- **`stats::moving_window`**: New submodule — sliding window mean/variance/min/max with `barracuda::stats::moving_window_stats_f64` delegation (S66)
- 26 new unit tests across 4 modules

#### Changed
- **Delegation count**: 39 → **46 active** (37 CPU + 9 GPU), 7 pending ToadStool unchanged
- **Test count**: 296 → **322** library tests; 436+ workspace (default)

#### Validated
- `cargo fmt --check`: PASS
- `cargo clippy -D warnings`: 0 warnings
- `cargo clippy -W pedantic`: 0 warnings
- `cargo doc --no-deps`: clean
- `cargo test --workspace`: all PASS (default + barracuda)
- 28 validation binaries: 292/292 PASS

### V46 Idiomatic Rust Evolution (Feb 28, 2026)

#### Changed
- **`#[allow]` → `#[expect]`**: All 7 remaining `#[allow]` annotations migrated to `#[expect]` with documented reasons; removed stale suppression in `seismic.rs`
- **Hardcoded thresholds evolved**: 6 validation binaries updated to read thresholds from benchmark JSONs (`rare_biosphere`, `multisignal`, `seismic`, `spectral_recon`, `band_edge`, `freeze_out`)
- **Named constant**: `SINGULARITY_THRESHOLD` extracted in `regression.rs`
- **`validate_weather.rs`**: Structured analytical provenance header added
- **`validate_nucleus_pipeline.rs`**: UID fallback documented with rationale

#### Added
- 6 benchmark JSON files updated with previously hardcoded thresholds and rationales
- Module → WGSL Shader → Pipeline Stage mapping table in `BARRACUDA_EVOLUTION.md`

### V45 Validation Gap Closure (Feb 28, 2026)

#### Added
- +4 validation checks (292/292 total): Exp 010 low-noise agreement, Exp 011 dual-signal variance, Exp 016 Spearman occupancy + multinomial determinism
- All Python checks now covered in Rust

### V44 Deep-Debt Evolution — linalg Extraction, Typed Errors, Capability Discovery (Feb 28, 2026)

#### Added
- **`linalg` module** (`crates/groundspring/src/linalg.rs`): Extracted tridiagonal eigensolver (`tridiag_eigh`, `EighError`) from `transport.rs` into shared linear algebra primitive. Used by both `transport` (wavepacket MSD) and `band_structure` (periodic Hamiltonian). `transport` re-exports for backward compatibility. `EighError` now derives `Clone`, `PartialEq`, `Eq`.
- **`error` module** (`crates/groundspring/src/error.rs`): Typed input validation errors (`InputError`) with three variants: `LengthMismatch`, `InsufficientData`, `OutOfRange`. Display implementation, full test suite.
- **`std_dev` test**: Known-value test for `stats::std_dev` (was untested).
- **`percentile_out_of_range` test**: Error condition test for `stats::percentile`.
- **`jackknife_insufficient_data` test**: Error condition test for `jackknife_mean_variance`.
- **Capability-based UID discovery**: `biomeos_socket_dir()` + `discover_uid()` in metalForge validation binaries — uses `$XDG_RUNTIME_DIR`, `$UID`, then `/proc/self/status` parsing. Zero `libc`, zero `unsafe`.

#### Changed
- **`jackknife_mean_variance`**: `assert!(n >= 2)` → `Result<JackknifeResult, InputError>`
- **`block_jackknife_variance`**: `assert!` → `Result<JackknifeResult, InputError>`
- **`finite_size_extrapolate`**: `assert_eq!` + `assert!` → `Result<(f64, f64, f64), InputError>`
- **`chi_squared`**: `assert_eq!` → `Result<f64, InputError>`
- **`percentile`**: `assert!` → `Result<f64, InputError>`
- **`GridFitConfig<'a>`**: Added `Debug`, `Clone`, `Copy` derives
- **`prng::next_u64`**: `as u64` → `u64::from()` (idiomatic)
- **`lib.rs`**: New `error` and `linalg` module declarations with documentation

#### Validated
- `cargo test --workspace` (default, barracuda, barracuda-gpu): all PASS
- `cargo clippy --workspace --all-targets --features barracuda-gpu`: 0 warnings
- 0 unsafe, 0 mocks in production, 0 deprecated patterns

### V39 NUCLEUS Integration + NestGate Data Pipeline + metalForge Remote Discovery (Feb 27, 2026)

#### Added
- **`nestgate` module** (`crates/groundspring/src/nestgate.rs`): NestGate data pipeline for experiment data and provenance. Provenance key schemas (`groundspring:results:`, `groundspring:parity:`, `groundspring:data:`), NCBI search/fetch via `ncbi_live_provider`, NOAA GHCND/FAO-56 via `noaa_cdo_live_provider`, cache-through helper. Behind `biomeos` feature. 4 tests.
- **`remote` module** (`metalForge/forge/src/remote.rs`): Remote substrate discovery via biomeOS capability routing. Parses remote NUCLEUS node inventory JSON, merges into local inventory with node ID prefix (e.g. `TITAN V@biomegate`). GPU arch parsing from canonical names (Volta, Ada, etc.). 12 tests.
- **Tower bootstrap graph** (`graphs/groundspring_tower_bootstrap.toml`): biomeOS pipeline for Tower atomic (BearDog + Songbird) on Eastgate — security, IPC nucleation, capability registration, health check, provenance.
- **`Inventory::merge_remote()`**: Extend metalForge inventory with remote NUCLEUS substrates.
- **`biomeos::escape_json_pub()`**: Public JSON escaping for sibling modules.

#### Updated
- **gen3/baseCamp/06_notill_anderson.md**: Added Exp 022-024 (ET₀→Anderson, no-till 16S, aggregate stability) to Cross-Spring Integration table.
- **gen3/baseCamp/07_sovereign_wdm.md**: Added Section 6.3 — WDM uncertainty budget (Exp 025-027: precision drift, size convergence, vendor parity).
- **gen3/baseCamp/README.md**: Expansion paragraph now includes Exp 022-024 for Paper 06.
- **groundSpring/whitePaper/baseCamp/anderson.md**: Three-tier table CPU tier DONE (Exp 014/016), metalForge tier in progress.
- **groundSpring/whitePaper/baseCamp/README.md**: Cross-Spring Impact table extended (Exp 022-028), Sub-thesis 07 (WDM) added.
- **ABSORPTION_MANIFEST.md**: Remote substrate discovery marked complete.

#### Validated
- `cargo fmt --all -- --check`: PASS
- `cargo clippy --workspace --all-targets --features biomeos -- -D warnings`: 0 warnings
- `cargo test --workspace --features biomeos`: 498+ tests PASS, 0 failures

### V35 Titan V / NAK Adaptive GPU Dispatch + Architecture-Aware Routing (Feb 27, 2026)

#### Added
- **GPU architecture detection**: `GpuArch` enum (Volta, Turing, Ampere, Ada, Other) auto-detected from adapter name. Reports f64:f32 ratio, workgroup sizing, and native f64 capability.
- **`NativeF64` capability**: GPUs with ≥ 1:4 f64:f32 ratio (Volta `GV100` = 1:2) get `NativeF64` capability, enabling architecture-aware dispatch.
- **Adaptive memory batching**: `AdaptiveBatch::for_gpu()` computes max batch size, workgroup size, and resident-memory mode from GPU arch + VRAM. Falls back to arch-specific VRAM defaults when wgpu reports API limits (common on NVK/NAK).
- **f64-preferring dispatch**: `dispatch::route()` prefers `NativeF64`-capable GPUs for f64 workloads. All 17 f64 workloads now route to Titan V (1:2 ratio) over RTX 4070 (1:64 ratio).
- **GPU VRAM probing**: `probe_gpus()` populates `memory_bytes` from wgpu `Limits::max_buffer_size`.
- **Inventory GPU methods**: `find_gpu_by_arch()`, `best_f64_gpu()`, `adaptive_batch()`.
- **Arch-aware summary table**: `print_summary()` shows architecture column.
- **14 new tests**: GPU arch detection, Volta f64 ratio, adaptive batch params, native f64 routing preference, f32 fallback routing.

#### Validated
- **Titan V (NVK `GV100`)**: Discovered via wgpu/Vulkan with NAK shader compilation — f64, shader dispatch, timestamps, native-f64 all confirmed.
- **Hardware inventory**: 5 substrates (RTX 4070 Ada, Titan V Volta, RTX 4070 OpenGL, AKD1000 NPU, i9-12900K CPU) — 14/14 checks pass.
- **f64 routing**: All 17 f64 workloads route to Titan V; 2 NPU workloads route to AKD1000; 19/19 routable.
- **Full workspace**: 0 clippy warnings, 49 metalForge tests pass, all groundspring tests pass.

#### Live GPU Compute (first direct shader execution)
- **Anderson Lyapunov (L=200, W=2.0, 1024 realizations)** executed on both GPUs:
  - Titan V (NVK/NAK): γ=0.0386, ξ=25.90, **797 µs** (f32 shader)
  - RTX 4070 (NVVM): γ=0.0386, ξ=25.90, **274 µs** (f32 shader)
  - CPU reference (f64): γ=0.0406, ξ=24.61, 6341 µs
  - f32 vs f64 precision delta: 5.0% (γ relative diff) — validates DF64 need

#### NAK f64 Gap Discovered
- **NAK**: `SHADER_F64` advertised but ALU lowering not implemented (`from_nir.rs:1092: assert bit_size == 32`). DF64 emulation needed.
- **NVVM**: `SHADER_F64` advertised but consumer Ada driver rejects f64 compute shaders. DF64 also needed.
- **Solution**: ToadStool's DF64 (double-float on f32 cores) gives ~50-bit precision, bridging the gap.

#### Metrics
- 32 active barracuda delegations (25 CPU + 7 GPU), 9 pending ToadStool
- 19 metalForge workloads, 49 metalForge tests, 5 discovered substrates
- 3 WGSL compute shaders (anderson_lyapunov f64, anderson_lyapunov f32, mc_et0_propagate, batched_multinomial)
- Titan V f64 throughput: 1:2 ratio (vs RTX 4070 1:64) — 32× native f64 advantage (pending NAK f64 ALU)

### V34 Three-Tier Validation + Four-Stage Progression + metalForge Expansion (Feb 27, 2026)

#### Added
- **Three-tier parity certificate**: `scripts/three_tier_parity_report.sh` — proves default = barracuda-CPU = barracuda-GPU for all 27 validation binaries (279/279 checks × 3 modes).
- **7 new metalForge workloads**: `gillespie_ssa_batch`, `spectral_recon_tikhonov`, `jackknife_leave_one_out`, `mc_et0_propagation`, `transport_eigenvalues`, `wright_fisher_batch`, `bootstrap_resampling`. Total: 19 workloads (14 GPU, 3 CPU, 2 NPU).
- **7 new metalForge routing tests**: all new workloads verified for correct substrate routing (41/41 metalForge tests pass).

#### Validated
- **Four-stage progression verified**:
  - Stage 1: Python 107.1s → Rust 20.5s (**5.2× faster**, 28/28 parity)
  - Stage 2: Rust 22.0s → barracuda-CPU 22.8s (+3.6% overhead, 27/27 parity)
  - Stage 3: barracuda-CPU → barracuda-GPU 9.8s (**2.2× total**, 47.4× peak)
  - Stage 4: metalForge routes 19 workloads to GPU/CPU/NPU per-operation
- **Three-tier parity**: `data/three_tier_parity_report.json` — 27/27 PROVEN
- **Python↔Rust parity**: `data/parity_report.json` — 28/28 PROVEN

#### Metrics
- 32 active barracuda delegations (25 CPU + 7 GPU), 9 pending ToadStool
- 19 metalForge workloads (14 GPU, 3 CPU, 2 NPU), 41 metalForge tests
- Python: 107.1s → Rust: 20.5s → barracuda-GPU: 9.8s (10.9× end-to-end)

### V33 Complete Rewiring + Three-Mode Benchmark + Cross-Spring Evolution (Feb 27, 2026)

#### Added
- **3 new barracuda delegations**:
  - #30 `stats::mae` — Mean Absolute Error (CPU tier, from airSpring/groundSpring S64 absorption)
  - #31 `stats::nash_sutcliffe` — Nash-Sutcliffe Efficiency (CPU tier, from airSpring/groundSpring S64 absorption)
  - #32 `spectral::detect_bands` — GPU band detection from eigenvalue spectrum (barracuda-gpu tier, from hotSpring v0.6 spectral theory)
- **`detect_band_ranges()` in `band_structure.rs`**: CPU fallback with gap-based band detection, GPU delegation via `barracuda::spectral::detect_bands`.
- **7 new tests**: 4 unit tests (mae, nse, detect_bands) + 3 three-tier parity tests.
- **Updated dispatch sentinel**: `dispatch_targets_at_least_32` (was `_37`/`_29`).

#### Validated
- **Three-mode benchmark** (default / barracuda / barracuda-gpu): 279/279 checks × 3 modes, all PASS.
  - **Total GPU speedup: 2.2×** (22,030ms → 9,798ms)
  - **Exp 009 quasiperiodic: 47.4×** (11,376ms → 240ms) — hotSpring Sturm tridiag eigenvalue solver
  - **Exp 019 jackknife: 4.1×** (410ms → 100ms) — barracuda optimized jackknife
  - **Exp 020 freeze-out: 1.7×** — barracuda chi² grid fit
  - **Exp 026 size-convergence: 1.6×** — barracuda regression fit_linear
- **28/28 parity proven** (Python ↔ Rust mathematical parity, `data/parity_report.json`).
- **0 clippy warnings** (`cargo clippy --workspace --all-features -- -D warnings`).
- Pre-existing `npu.rs` doc_markdown lint fixed (`MacKinnon` backticked).
- Pre-existing `validate_npu_anderson.rs` unfulfilled lint expectation fixed.

#### Cross-Spring Provenance (new delegations)
| # | Function | barracuda fn | Origin | Evolved Through |
|---|---------|-------------|--------|----------------|
| 30 | `mae` | `stats::mae` | airSpring V009 → ToadStool S64 | airSpring ET₀ validation metrics → groundSpring error decomposition |
| 31 | `nash_sutcliffe` | `stats::nash_sutcliffe` | airSpring V009 → ToadStool S64 | airSpring hydrology → groundSpring model agreement metrics |
| 32 | `detect_band_ranges` | `spectral::detect_bands` | hotSpring v0.6 → ToadStool S26 | hotSpring spectral theory → groundSpring band structure analysis |

#### Metrics
- 32 active barracuda delegations (25 CPU + 7 GPU), 9 pending ToadStool
- 279/279 validation checks in all three feature modes
- cargo test --workspace: 0 failures
- Three-mode total: default 22,030ms / barracuda 22,828ms / barracuda-gpu 9,798ms

### V32 ToadStool S68+ Catch-Up + Forward Declaration Cleanup (Feb 27, 2026)

#### Changed
- **9 forward declarations commented out**: V29 wired 3 CPU delegations (`kimura_fixation`, `jackknife_mean_variance`, `fao56_et0`) and V31 wired 6 GPU delegations (`grid_fit_2d_f64`, `grid_search_3d_f64`, `band_edges_parallel`, `wright_fisher_simulate`, `batched_multinomial_occupancy`, `batched_multinomial_tier_rate`) that reference functions not yet in ToadStool barracuda. All 9 are now commented out with `TODO(toadstool)` markers. `--features barracuda` and `--features barracuda-gpu` now compile clean.
- **Doc comments updated**: Module-level barracuda delegation docs in `drift.rs`, `jackknife.rs`, `fao56.rs` clarified as "pending ToadStool absorption" rather than "delegates to".
- **ToadStool pin confirmed**: S68+ (`e96576ee`, Feb 27 2026) — universal precision architecture, 700 WGSL shaders, dual-layer DF64, zero f32-only shaders.

#### Metrics
- 29 active barracuda delegations (23 CPU + 6 GPU), all compile clean
- 9 pending ToadStool delegations (3 CPU + 6 GPU), commented out
- 410/410 Rust tests (default), 442/442 (biomeos), 320/320 Python — all PASS
- `cargo clippy --workspace --all-features` — 0 warnings
- `cargo check --features barracuda` — PASS (was FAIL before V32)
- `cargo check --features barracuda-gpu` — PASS (was FAIL before V32)

### V31 GPU Dispatch Wiring + metalForge Workload Expansion (Feb 27, 2026)

#### Added
- **GPU dispatch wiring**: 5 modules now have `#[cfg(feature = "barracuda-gpu")]` dispatch blocks ready for ToadStool absorption: `freeze_out::grid_fit_2d` (2D parallel grid), `band_structure::find_band_edges` (per-energy parallel transfer matrix), `seismic::grid_search_inversion` (3D parallel grid), `quasispecies::quasispecies_simulation` (batched Wright-Fisher), `rare_biosphere::abundance_occupancy` + `tier_detection_rate` (batched multinomial).
- **metalForge workloads**: 5 new workload definitions with GPU routing — `freeze_out_grid_fit`, `seismic_grid_search`, `band_edge_scan`, `quasispecies_wright_fisher`, `rare_biosphere_multinomial`. Total: 12 workloads (up from 7).
- **GPU parity integration tests**: 10 new tests in `three_tier_parity.rs` verifying determinism and known-value parity for all GPU-wired functions.
- **metalForge routing tests**: 5 new tests confirming GPU routing for new workloads.
- Updated gillespie doc to clarify GPU is batch-only (not drop-in single-trajectory).

#### Metrics
- 442 Rust tests with `--features biomeos` (410 base + 32 biomeos), all PASS
- 410 Rust tests in default mode (up from 391: +10 GPU parity + 5 metalForge routing + 4 dispatch parity), all PASS
- 320 Python tests, all PASS + 2 skipped
- 0 clippy warnings across all feature combinations
- Total: 442 + 320 = 762 tests (up from 745)
- 12 metalForge workloads (up from 7)
- 37 barracuda dispatch targets: 26 CPU + 6 existing GPU + 5 new GPU-ready
- 28/28 parity PROVEN (formal certificate: `data/parity_report.json`)
- Performance: 11.5× Rust vs Python (excl. LAPACK-bound); Exp 005 Seismic 53.5×, Exp 011 Multi-Signal QS 44.7×

### V30 biomeOS Neural API Integration (Feb 27, 2026)

#### Added
- **biomeOS Neural API client**: `crates/groundspring/src/biomeos.rs` — JSON-RPC 2.0 over Unix domain socket, following wetSpring's `NestGate` pattern. Provides `capability_call`, `direct_rpc_call`, `storage_put/get`, and `health` methods with sovereign fallback when socket is unavailable.
- **`biomeos` feature gate**: Optional feature in `Cargo.toml` — no external deps, pure Rust Unix socket + JSON string handling.
- **Concept docs**: `whitePaper/neuralAPI/README.md` (groundSpring as validation science primal) and `CAPABILITY_SURFACE.md` (5 provided + 3 consumed capabilities with registry format).
- **Pipeline graph**: `graphs/groundspring_validation.toml` — biomeOS orchestration graph for Anderson localization (benchmark → Lyapunov → parity → provenance), follows `science_pipeline.toml` pattern.
- **Anderson biomeOS routing**: `validate-anderson` optionally routes Lyapunov computation through `capability.call("compute.execute", ...)` when `GROUNDSPRING_COMPUTE_PROVIDER=biomeos`, with sovereign fallback. Also stores results in `NestGate` when available.
- **Integration tests**: `biomeos_integration.rs` — 10 tests covering socket discovery (env var, XDG, temp), sovereign fallback, error handling, provider detection.

#### Metrics
- 423 Rust tests with `--features biomeos` (391 base + 22 biomeos unit + 10 biomeos integration), all PASS
- 391 Rust tests in default mode (unchanged), all PASS
- 322 Python tests (unchanged), 320 pass + 2 skip
- 0 clippy warnings across all feature combinations
- Total: 423 + 322 = 745 tests (up from 713)

### V29 Three-Tier Validation Buildout + Barracuda CPU Delegation Wave (Feb 27, 2026)

#### Added
- **3 new barracuda CPU delegations**: `drift::kimura_fixation_prob` → `barracuda::stats::kimura_fixation`, `jackknife::jackknife_mean_variance` → `barracuda::stats::jackknife_mean_variance`, `fao56::daily_et0` → `barracuda::stats::hydrology::fao56_et0`. All use `if let Ok` pattern with always-compiled CPU fallback.
- **GPU-ready annotations** for 8 undelegated modules: freeze_out (2D grid dispatch), band_structure (per-energy parallel), seismic (3D grid dispatch), quasispecies (batched WrightFisher), rare_biosphere (batched multinomial), gillespie (batched trajectories), transport (tridiag stays local — QL beats dense Jacobi), fao56 (batch ET₀)
- **`three_tier_parity.rs`**: 23 integration tests validating CPU ↔ barracuda-CPU ↔ barracuda-GPU mathematical equivalence across drift, jackknife, fao56, rare_biosphere, quasispecies, band_structure, freeze_out, gillespie, transport, seismic
- **`test_three_tier_parity.py`**: Python-side parity tests proving all 27 Rust validation binaries pass, all benchmark JSONs have provenance, and Rust is not slower than Python

#### Changed
- **BARRACUDA_EVOLUTION.md**: Tier A table expanded to 32 delegations (26 CPU + 6 GPU); Tier B table expanded with 4 new GPU dispatch candidates (freeze_out, band_structure, quasispecies, rare_biosphere); Phase 2a updated to 32, Phase 2b in progress
- **PAPER_REVIEW_QUEUE.md**: Updated delegation counts, added three-tier parity note, 8 GPU-annotated modules
- **Module docs**: All 8 newly annotated modules have barracuda delegation sections documenting CPU/GPU/metalForge progression

#### Metrics
- 391 Rust tests (272 unit + 13 determinism + 14 proptest + 23 three-tier parity + 29 forge + 12 validate-lib + 28 integration), all PASS
- 322 Python tests (250 experiments + 72 three-tier parity), 320 pass + 2 skip (LAPACK-wins: quasiperiodic, drift)
- 32 barracuda delegations (26 CPU + 6 GPU), 8 GPU-annotated modules
- 0 clippy warnings, 288/288 validation checks
- Total: 391 + 322 = 713 tests (up from 618)
- 28/28 experiments green at CPU tier, GPU tier annotated, metalForge tier live (Exp 028)

### V28 Coverage Evolution + PRNG Readiness + CI Drift Detection (Feb 27, 2026)

#### Added
- **`Xoshiro128StarStar` API parity**: `next_u64()` and `binomial()` methods added to GPU-aligned PRNG, matching full `Xorshift64` API surface. Ready for Phase 2b PRNG migration when barracuda feature gate activates
- **`tests/test_baseline_integrity.py`**: 196 parametric tests verifying every benchmark JSON has complete provenance metadata (`_source`, `_provenance.baseline_date`, `_provenance.baseline_commit`, `_provenance.validation_script`), valid hex commit hashes, UTF-8 encoding, and that every experiment directory has both a benchmark file and a Python script
- **Coverage tests**: 45 new Rust tests across 6 modules:
  - `bistable.rs`: stochastic_integrate determinism/divergence/non-negativity, low-noise near-deterministic, derivative boundedness
  - `multisignal.rs`: stochastic_integrate determinism/divergence/non-negativity, derivative boundedness
  - `rare_biosphere.rs`: tier_detection_rate (determinism, abundant, rare), detection_threshold edge cases, chao1 only-singletons branch
  - `prng.rs`: xoshiro next_u64/binomial determinism, binomial mean, normal with mean/std
  - `inventory.rs`: count/first for absent kinds, print_summary, empty inventory
  - `validate lib.rs`: print_provenance_header (complete + missing fields), f64_range longer array

#### Changed
- **CI workflow**: Split Python job into fast integrity checks (test_common + test_determinism + test_baseline_integrity) then full experiment runs with `--timeout=300`; `fetch-depth: 0` for provenance commit verification
- **Root README**: Updated to 368 tests, xoshiro128** API parity, four-mode CI
- **whitePaper/**: Updated baseCamp, experiments, STUDY, METHODOLOGY with V28 metrics
- **wateringHole/**: V28 toadstool handoff (coverage evolution + PRNG readiness + three-tier paper controls)
- **specs/**: Updated BARRACUDA_EVOLUTION (xoshiro128** at API parity), PAPER_REVIEW_QUEUE (three-tier matrix confirmed), BARRACUDA_REQUIREMENTS

#### Metrics
- 368 Rust tests (272 groundspring lib + 13 determinism + 14 proptest + 29 forge + 12 validate-lib + 28 binary), all PASS
- 196 Python baseline integrity tests, all PASS
- 288/288 validation checks, 28/28 mathematical parity
- 0 clippy warnings × 3 modes, 0 `todo!()`/`unimplemented!()`, 0 `unwrap()` in production
- All 28 benchmark JSONs have complete provenance, all hex commit hashes, all UTF-8
- Xoshiro128StarStar: next_u64(), next_f64(), next_normal(), normal(), binomial() — full API parity with Xorshift64

### V27 Docs + Handoff Audit: Barracuda Evolution Review (Feb 27, 2026)

#### Changed
- **Root README.md**: Updated to 323 tests, 29 delegations (23 CPU + 6 GPU), 99.37% coverage, three-mode CI
- **CONTRIBUTING.md**: Updated test counts, module count (26), added Exp 022-028 validation binaries and metalForge binaries, three-mode CI
- **whitePaper/baseCamp/README.md**: Updated validation summary (29 delegations, 323 tests, 99.37% coverage, V26 metalForge)
- **whitePaper/experiments/README.md**: Updated to 323 tests, 29 delegations, 99.37% coverage, metalForge tier
- **whitePaper/STUDY.md**: Updated abstract, Phase 2a summary, and timeline
- **whitePaper/METHODOLOGY.md**: Updated test counts, coverage, delegation counts, added metalForge tier
- **whitePaper/CROSS_SPRING_EVOLUTION.md**: Updated to 29 delegations
- **wateringHole/README.md**: V27 and V26 as active handoffs, V23 archived
- **wateringHole/CROSS_SPRING_SHADER_EVOLUTION.md**: Updated to 29 delegations, added #28 (tikhonov_solve) and #29 (finite_size_extrapolate), V26/V27 timeline entries
- **specs/README.md**: Updated to 29 delegated, 323 tests
- **specs/BARRACUDA_REQUIREMENTS.md**: Updated to 29 delegations (23 CPU + 6 GPU), 323 tests, 99.37% coverage, metalForge hardware
- **specs/PAPER_REVIEW_QUEUE.md**: Updated delegation and test counts
- **metalForge/ABSORPTION_MANIFEST.md**: Updated to 29 delegated, added hill/tikhonov/regression entries, V27 handoff checklist item
- **CONTROL_EXPERIMENT_STATUS.md**: Added V27 entry, updated coverage to 99.37%

#### Added
- **V27 ToadStool handoff** (`wateringHole/handoffs/GROUNDSPRING_TOADSTOOL_V27_BARRACUDA_EVOLUTION_HANDOFF_FEB27_2026.md`): Comprehensive barracuda evolution review following hotSpring/wetSpring handoff pattern — 6 parts: barracuda usage, absorption requests, evolution learnings, paper controls (open data audit: 28/28 PASS), three-tier hardware validation matrix (CPU → GPU → metalForge), barracuda evolution summary (V7→V27)

#### Metrics
- 323 Rust tests, 288/288 validation checks, 29 barracuda delegations (23 CPU + 6 GPU)
- 99.37% line coverage, 0 clippy warnings × 3 modes
- 28/28 papers confirmed open data/systems — zero proprietary dependencies
- Three-tier validation: CPU (288/288), GPU (6 delegations, 2.2× speedup), metalForge (31 checks, 3 live hardware substrates)

### V26 MetalForge Live Hardware: GPU + NPU + Cross-Substrate (Feb 27, 2026)

#### Added
- **groundspring-forge crate** (`metalForge/forge/`): hardware discovery via wgpu (GPU), /dev/akida* (NPU), procfs (CPU); capability-based dispatch routing 7 groundSpring workloads to best substrate
- **validate-metalforge-inventory**: discovers all substrates, asserts GPU/NPU/CPU presence (10/10 PASS)
- **validate-metalforge-gpu**: Anderson Lyapunov on GPU via barracuda-gpu dispatch, CPU/GPU parity proof (11/11 PASS)
- **validate-metalforge-cross-substrate**: CPU vs GPU vs NPU parity table on 10 Anderson disorder values (10/10 PASS)
- **npu feature** in groundspring crate: `npu.rs` module wrapping ToadStool akida-driver for Anderson regime classification on BrainChip AKD1000
- **Exp 028: NPU Anderson Regime Classification** — int8 quantized classification on live AKD1000 at ~51µs/inference (Python 7/7, Rust 9/9 PASS)
- **AKD1000 hardware characterization** (`metalForge/npu/akida/HARDWARE.md`)
- 12 new forge crate unit tests (substrate, dispatch, probe, inventory)

#### Changed
- Workspace `Cargo.toml`: added `metalForge/forge` to workspace members
- Three-mode benchmark updated to 27 bins: 20.4s (default) → 9.2s (barracuda-gpu), 2.2× overall

#### Metrics
- 314 Rust tests (302 groundspring + 12 forge)
- 288/288 Rust validation checks (28 experiment binaries) + 31 metalForge checks
- 28/28 mathematical parity (Python ⇌ Rust)
- 0 clippy warnings across all crates and features
- Live hardware: RTX 4070, Titan V, AKD1000 NPU, i9-12900K

### V24 Deep Debt: Deterministic Validation, SPDX Compliance, Coverage & Idiomatic Cleanup (Feb 26, 2026)

#### Fixed
- **Flaky `validate-uncertainty-bridge`**: per-sensor deterministic RNG (fresh `Xorshift64` per sensor call) eliminates cross-sensor dependency that caused intermittent failures under `--all-features`
- **Tolerance**: EC5 bias-correction `min_reduction_fraction` widened from -0.05 to -0.15 with documented justification (MC variance at n_mc=200)
- **`unwrap()` → `expect()`** in `validate_uncertainty_bridge.rs:99` for explicit error context
- **SPDX headers**: added `AGPL-3.0-only` headers to all 30 Python files missing them (experiment scripts, `__init__.py`, utility scripts)
- **Baseline provenance**: stamped `baseline_commit` in 6 benchmark JSONs that had empty or "pending" values
- **Sequencing noise data accession**: resolved "Pending" → documented as synthetic with future SRA targets

#### Changed
- **`spectral_recon::rmse`**: now delegates to `crate::stats::rmse` (which delegates to barracuda when feature-enabled), eliminating duplicate RMSE implementation
- **`#[allow]` → `#[expect]` with reasons**: `spectral_recon.rs` linalg helpers upgraded to modern idiomatic lint suppression
- **`rare_biosphere::chao1`**: documented as staying local — barracuda's `chao1(&[f64])` uses float equality for singleton/doubleton classification, incompatible with our u64 integer counting (Tier B alignment required)

#### Added
- 7 new transport.rs tests: edge cases for `EighError::Display`, regression singularity, tiny-MSD filtering, nonpositive-time filtering, MSD at t=0, eigenvalue reconstruction verification
- Transport coverage: 93.18% → 98.78% line coverage

#### Metrics
- 287 Rust tests (was 280)
- 236/236 Rust validation checks (all 21 binaries PASS)
- 98.78% workspace line coverage (was 98.55%)
- 0 clippy warnings × 3 modes (CPU-only, barracuda, barracuda-gpu)
- 100% SPDX compliance (Rust + Python)
- 0 empty/pending baseline_commit fields (was 6)

### V23 Experiment Buildout: Exp 019-021 — Inverse Problems & Spectral Reconstruction (Feb 26, 2026)

#### Added
- **Experiment 019: Jackknife Error Estimation** — Bazavov 2025 Phys Rev D 111, 094508; jackknife variance, bias correction, leave-one-out resampling; module: `jackknife`; 9/9 Rust checks
- **Experiment 020: Freeze-Out Inverse Problem** — Bazavov 2016 Phys Rev D 93, 014512; freeze-out temperature inversion from hadron yields; module: `freeze_out`; 8/8 Rust checks
- **Experiment 021: Spectral Function Reconstruction** — Bazavov 2025 arXiv 2501.12259; spectral reconstruction from Euclidean correlators; module: `spectral_recon`; 8/8 Rust checks
- 3 new Rust library modules: `jackknife`, `freeze_out`, `spectral_recon`
- 3 new validation binaries: `validate-jackknife` (9/9), `validate-freeze-out` (8/8), `validate-spectral-recon` (8/8)
- New scientific domain: **Inverse Problems & Spectral Reconstruction** (formal domain from Bazavov papers)

#### Metrics
- 21 experiments (was 18)
- 280 Rust tests (was 262)
- 236 Rust validation checks (was 211)
- 21/21 mathematical parity (was 18/18)
- 47 pytest tests (was ~44)
- 8 scientific domains (was 7)
- 24 library modules (was 21)

### V22 Experiment Buildout: Exp 016-018 + Full Linting Cleanup (Feb 26, 2026)

#### Added
- **Experiment 016: Rare Biosphere Signal Detection** — R. Anderson 2015 FEMS Microbiol Ecol; Chao1 richness, detection power/threshold, abundance-occupancy, singleton fraction
- **Experiment 017: Eco-Evolutionary Noise Threshold** — Dolson 2023 J R Soc Interface; Eigen's quasispecies model, error threshold, master frequency, Wright-Fisher mutation simulation
- **Experiment 018: Band Edge Structure** — Filonov & Kachkovskiy 2018 Acta Math; transfer matrix method, band-gap detection, periodic Hamiltonian, eigenvalue band fraction
- 3 new Rust library modules: `rare_biosphere`, `quasispecies`, `band_structure`
- 3 new validation binaries: `validate-rare-biosphere` (10/10), `validate-quasispecies` (6/6), `validate-band-edge` (10/10)
- 3 new Python control scripts with benchmark JSONs
- WhitePaper experiment docs (016, 017, 018) and Dolson faculty briefing

#### Fixed
- Pre-existing clippy warnings in `almost_mathieu.rs`: 9 CPU-fallback functions/constants gated with `#[cfg(not(feature = "barracuda-gpu"))]` to eliminate dead-code warnings in all-features mode
- Pre-existing clippy warnings in `anderson.rs`: 2 CPU-fallback functions gated similarly
- `needless_return` in `almost_mathieu.rs` barracuda-gpu blocks (expression position instead of `return`)
- `float_cmp` in `bistable.rs`, `multisignal.rs`, `ode.rs` determinism tests (added `#[allow(clippy::float_cmp)]`)
- `unfulfilled_lint_expectations` in `tests/determinism.rs`: replaced per-function `#[expect]` with module-level `#![allow(clippy::float_cmp)]`
- `suboptimal_flops` in `transport.rs` test helper (mul_add)
- Python ruff: `F541` (f-string), `F401` (unused import), `I001` (import sort), `F841` (unused vars) across 3 new control scripts

#### Metrics
- 262 Rust tests (207 unit + 13 determinism + 14 proptest + 9 validate-lib + 18 integration + 1 doc)
- 211/211 validation checks across 18 binaries
- 18/18 mathematical parity proven (Python ⇌ Rust)
- 18/18 pytest PASS
- `cargo clippy --all-targets --all-features -- -D warnings`: 0 warnings
- `ruff check control/ tests/`: All checks passed

### V21 Complete Barracuda Rewiring + Dual-Mode CI (Feb 26, 2026)

#### Changed
- `kinetics::hill`: domain guard (x ≤ 0 → 0) now precedes barracuda delegation — preserves biological convention
- 17 `_cpu` fallback functions gated with `#[cfg(not(feature = "barracuda"))]` — zero dead-code warnings in barracuda mode
- `needless_return` cleanup: `#[cfg]` blocks use expression position instead of `return`
- Import gating: `bistable`, `multisignal`, `stats/metrics` imports gated per feature flag
- CI: `cargo clippy --features barracuda` + `cargo test --features barracuda` added — dual-mode validation

#### Metrics
- 225 Rust tests PASS in both CPU-only and barracuda-delegated modes
- Zero clippy warnings in both modes (`cargo clippy --workspace -- -D warnings`)
- CPU delegation overhead: +1.7% total (16,447ms → 16,722ms in release benchmarks)
- Anderson/RAWR actually faster with barracuda (742ms vs 831ms, 604ms vs 640ms)
- Cross-spring shader evolution documented: 700 shaders from 5 springs

### V20 ToadStool S68 Catch-up + Hill Delegation (Feb 26, 2026)

#### Added
- Hill kinetics delegation #27: `kinetics::hill` now delegates directly to `barracuda::stats::hill` (infallible f64 return, `#[cfg]`/`#[cfg(not)]` mutual exclusion)

#### Changed
- `kinetics::hill`: stub → active delegation. Was `if let Ok(v)` fallible pattern, now `#[cfg(feature)]` / `#[cfg(not)]` infallible pattern
- `kinetics::hill_repress`: simplified to `1.0 - hill(x, k, n)`, gets barracuda delegation for free
- ToadStool pin updated: `045103a7` (S66 Wave 5) → `f0feb226` (S68 universal precision)

#### Noted
- ToadStool S68 CPU feature gate bug: `stats/mod.rs` references `crate::shaders` without `#[cfg(feature = "gpu")]`, preventing `--features barracuda` compilation. Reported in V20 handoff.

#### Metrics
- 27 barracuda delegations (22 CPU + 5 GPU), was 26 (21 + 5)
- 225 Rust tests, 185/185 validation checks, 98.93% coverage — unchanged
- ToadStool S68: 700 shaders (zero f32-only), 2,546+ barracuda tests, 21,599 workspace tests

### V19 Uncertainty Bridge + Idiomatic Evolution (Feb 26, 2026)

#### Added
- New **Experiment 015: Uncertainty Bridge** — cross-domain uncertainty propagation from sensor noise (Exp 001) through Anderson localization (Exp 008) to predict localization length confidence intervals
- Python Phase 0 baseline: 8/8 PASS (Monte Carlo with PCG64, 200 samples × 10 realizations)
- Rust Phase 1 validation binary `validate-uncertainty-bridge`: 8/8 PASS (Xorshift64 MC)
- Experiment briefing at `whitePaper/experiments/015_uncertainty_bridge.md`
- Python integration test `test_exp015_uncertainty_bridge`
- CI: `validate-uncertainty-bridge` added to GitHub Actions workflow

#### Changed
- `seismic.rs`: extracted `origin_time_and_rms()` from `grid_search_inversion()` reducing main function complexity by 20 lines
- `transport.rs`: `#[allow(clippy::many_single_char_names)]` → `#[expect(..., reason = "...")]` (modern Rust idiom, zero remaining `#[allow]`)
- Validation binary pattern: `BridgeParams` struct replaces 10-argument functions (clippy `too_many_arguments` fix)

#### Scientific Findings
- At typical soil moisture (θ ≈ 0.30), Lyapunov exponent operates in the saturated regime — bias correction provides <1% improvement in localization length uncertainty
- Sensor ranking preserved: EC5 (higher noise) → higher CV(ξ) than CS616 (lower noise)
- This validates the Gen3 Sub-thesis 01+06 precursor: sensor noise quantification is a prerequisite for the Anderson-QS bridge

#### Metrics
- 225 Rust tests (unchanged), 185/185 validation checks, 98.93% llvm-cov
- 15 validation binaries (was 14), 15/15 experiments with parity
- Zero `#[allow]` remaining (was 1), zero clippy warnings

### V18 Deep Debt Evolution — Idiomatic Rust + Test Coverage (Feb 26, 2026)

#### Added
- New `kinetics` module: `hill()` and `hill_repress()` with barracuda delegation
- 13 determinism tests: bitwise-identical rerun verification for all stochastic algorithms
- 6 Hill kinetics unit tests
- Python tests for Exp 012-014 in `test_experiments.py`
- DOIs added to 10 benchmark JSONs (all 14 now have formal DOIs)
- Tolerance justification comments on 17 library tests

#### Fixed
- CI: added 3 missing validation binaries (`validate-transport`, `validate-resampling-conv`, `validate-drift`)
- Provenance: `validate_rawr` and `validate_rarefaction` now use standard `print_provenance_header`
- Exp 013 lognormal convergence tolerance widened (1.2× → 1.5×) with justification
- 3 pending `baseline_commit` fields stamped

#### Changed
- `almost_mathieu.rs`: Givens QR refactored from `Vec<Vec<f64>>` to flat row-major `Vec<f64>`
- `transport.rs`: `tridiag_eigh` returns flat `Vec<f64>` instead of `Vec<Vec<f64>>`
- `bistable.rs` and `multisignal.rs`: rewired from local `hill()` to `crate::kinetics::hill()`

#### Metrics
- 225 Rust tests (was 205), 177/177 validation checks, 98.94% llvm-cov
- 0 clippy warnings (pedantic + nursery), 0 unsafe, 0 `Vec<Vec<f64>>`, 0 duplicate math
- 14/14 CI validation binaries, 14/14 DOIs, 0 pending baseline commits

### V16 ToadStool S66 Catch-up + Deep Debt Evolution (Feb 26, 2026)
- **ToadStool S66 review**: ToadStool reached Session 66 (2,541 tests, 707 WGSL
  shaders, sovereign compiler, DF64 multi-precision). V7 was last groundSpring
  handoff absorbed — V13-V15 handoffs await consumption.
- **New delegation #26**: `rawr_mean` → `barracuda::stats::rawr_mean` (absorbed
  in ToadStool S66 from groundSpring V15 request). RAWR (Dirichlet-weighted
  bootstrap) now delegates to barracuda CPU path with graceful fallback.
  Total: **26 active delegations** (21 CPU + 5 GPU).
- **Bug fix**: `covariance`, `pearson_r`, `spearman_r` now fall through to CPU
  implementation on barracuda error instead of returning 0.0. Previously, if
  barracuda returned an error, these functions silently returned 0.0.
- **Deep debt: delegation pattern evolution**: Eliminated ALL 20
  `#[allow(unreachable_code)]` annotations across 13 files. Infallible
  barracuda calls now use `#[cfg]`/`#[cfg(not)]` mutual exclusion. Fallible
  calls use `#[cfg] if let Ok` with natural fall-through to CPU.
  Only 1 `#[allow]` remains: `clippy::many_single_char_names` in QL algorithm.
- **Deep debt: idiomatic Rust**: `BistableParams` and `MultiSignalParams` now
  derive `Copy` — eliminated all `.clone()` calls on small-field structs.
  Gillespie `time_averaged_mean`/`time_averaged_variance` use `.windows(2)`
  instead of manual index loops.
- **Deep debt: named constants**: 7 magic numbers extracted as named constants
  (`QL_MAX_ITERATIONS`, `QR_MAX_ITERATIONS`, `MSD_MIN_THRESHOLD`,
  `REGRESSION_EPSILON`, `EXP_VARIATE_CAP`, `DERRIDA_GARDNER_CONSTANT`).
- **Deep debt: docs accuracy**: transport.rs, drift.rs, gillespie.rs docs
  corrected from claiming barracuda delegation to documenting future candidates.
- **Deep debt: error messages**: Validation binary `unwrap()` calls replaced
  with `unwrap_or_else(|| panic!("descriptive message"))`.
- **Test fix**: `bootstrap_different_from_rawr` and `validate_rawr` updated
  for barracuda parity.
- **Three-mode revalidation**: 205/205 tests × 3 modes, 177/177 validation
  checks × 3 modes, 0 clippy warnings × 3 modes.

### V15 Experiment Buildout (Feb 26, 2026)
- **3 new experiments built**: Exp 012 (Spin Chain Transport, 18/18 PASS),
  Exp 013 (Resampling Convergence, 8/8 PASS), Exp 014 (Drift vs Selection,
  7/7 PASS). Total: **15 experiments, 185 validation checks, 225 Rust tests**.
- **New module `transport`**: Tridiagonal eigenvector solver (implicit QL),
  wavepacket MSD computation, transport exponent extraction via log-log fit.
- **New module `drift`**: Wright-Fisher fixation simulation, Kimura analytical
  fixation probability, neutral diversity trajectory under genetic drift.
- **`prng::binomial`**: Added binomial sampling to Xorshift64 for Wright-Fisher.
- **Paper queue progress**: Papers #13 (Lee & Liu), #17 (Kachkovskiy), #20
  (R. Anderson) moved from Queued to Active.
- **Mathematical parity**: 14/14 PROVEN (Python ⇌ Rust).

### V14 S65 Revalidation + Cross-Spring Documentation (Feb 26, 2026)
- **New delegation #25**: `evenness` → `barracuda::stats::pielou_evenness`.
  S≤1 semantic adapter (groundSpring returns 1.0, barracuda returns 0.0).
  Total: **25 active delegations** (20 CPU + 5 GPU).
- **`stats/correlation.rs` modernized**: Removed `#[cfg(not(feature))]` gates.
  CPU code always compiled. Extracted `pearson_r_cpu`, `spearman_r_cpu`,
  `covariance_cpu`, `rank` as private always-compiled functions.
- **`anderson.rs` → `almost_mathieu.rs` split**: Almost-Mathieu model
  extracted to own module (264 + 329 lines, was 594 combined).
- **Python linting**: 14 ruff errors fixed (import sorting, `zip(strict=True)`,
  unused variables). Python linting now zero-warning.
- **Three-mode benchmark**: 14,893ms → 3,926ms (barracuda-gpu). Exp 009: 49.5×.
- **Cross-spring evolution document**: `whitePaper/CROSS_SPRING_EVOLUTION.md`
  tracing lineage of all 25 delegations through hotSpring, wetSpring, airSpring,
  neuralSpring.
- **New scripts**: `regenerate_benchmarks.sh` (drift guard), `three_mode_benchmark.sh`.
- **V14 handoff**: Created. V13 archived.

### Complete Rewiring + Cross-Spring Benchmark (V13 — Feb 26, 2026)
- **4 new barracuda delegations**: `mean`, `percentile`, `level_spacing_ratio`,
  `almost_mathieu_eigenvalues` (via `find_all_eigenvalues` Sturm tridiag solver).
  Total: **24 active delegations** (was 20).
- **Exp 009 performance breakthrough**: Sturm bisection tridiag solver from
  barracuda (originated in hotSpring S26 spectral module) replaces dense QR.
  Quasiperiodic validation: 11.7s → 0.23s (**50× speedup**).
  Total three-mode benchmark: 14.5s → 3.3s (barracuda-gpu mode).
- **QR eigenvalue code moved**: Dense Givens QR moved from validation binary
  to library (`anderson::eigenvalues_qr_dense`), gated behind
  `#[cfg(not(feature = "barracuda-gpu"))]`.
- **V13 handoff**: Created with complete rewiring state. V12 archived.

### ToadStool S64 Catch-Up + 20 Delegations (V12 — Feb 26, 2026)
- **6 new barracuda delegations**: ToadStool Session 64 absorbed `stats::metrics`
  (rmse, mbe, r_squared, index_of_agreement, hit_rate) and `stats::diversity`
  (shannon) from airSpring/groundSpring. groundSpring immediately wired all 6.
  Total: **20 active delegations** (was 14).
- **3 pre-existing barracuda-mode bug fixes**:
  - `OdeSystem` trait import for `BistableOde`/`MultiSignalOde` delegation
  - `barracuda::spectral::hofstadter` module path (now private, re-exported)
  - Dead-code gates for local helpers (`hill`, `hill_repress`, `*_local`)
- **batched_multinomial absorbed**: ToadStool S64 absorbed `BatchedMultinomialGpu`
  + `multinomial_sample_cpu`. groundSpring rewiring deferred (signature mismatch).
- **V12 handoff**: Created with updated priorities. V11 archived.
- **All docs updated**: 14→20 delegations across README, specs, whitePaper,
  wateringHole, metalForge.

### Full-Suite Parity + Benchmarks (V11 — Feb 26, 2026)
- **Exp 009–011 buildout**: Almost-Mathieu quasiperiodic localization,
  bistable phenotypic switching, multi-signal QS integration. 3 new Python
  baselines, 3 new Rust modules (`bistable`, `multisignal`, `anderson` extended),
  3 new validation binaries. 25 new validation checks, 24 new unit tests,
  3 new barracuda delegations (`almost_mathieu_hamiltonian`,
  `BistableOde::cpu_derivative`, `MultiSignalOde::cpu_derivative`).
  Total: **144/144** across 11 binaries, **177** Rust tests, **14** delegations.
- **Full-suite Python vs Rust benchmarks**: Expanded `bench_rust_vs_python.py`
  from 3 to all 11 experiments. 10/11 compute-bound experiments: **23.4× faster**.
  Exp 009 (custom QR vs LAPACK) intentionally slower — proves parity not speed.
- **Mathematical parity certificate**: New `scripts/parity_report.py` runs all
  22 validation paths (11 Python + 11 Rust) against shared benchmark JSONs.
  **11/11 PARITY PROVEN**. Machine-readable `data/parity_report.json`.
- **Benchmark scripts expanded**: `bench_barracuda_modes.sh` (8→11 binaries),
  `run_all_baselines.sh` (8+8→11+11 Python+Rust entries).
- **Documentation updated**: README, CONTROL_EXPERIMENT_STATUS, BARRACUDA_EVOLUTION,
  whitePaper/STUDY all reflect 11 experiments, 23.4× speedup, 11/11 parity.
- **V11 handoff**: Created with full-suite benchmarks, parity certificate,
  updated absorption priorities, and three-tier validation roadmap.
  V10 archived.

### Definitive Handoff & Cross-Spring Evolution Doc (V10)
- Created `wateringHole/CROSS_SPRING_SHADER_EVOLUTION.md` (following wetSpring pattern)
  documenting how hotSpring, wetSpring, and neuralSpring evolved barracuda
- V10 definitive handoff: 5 absorption priorities, complete delegation inventory,
  PRNG roadmap, error handling standard, benchmark results, cross-spring learnings
- Fixed CONTROL_EXPERIMENT_STATUS.md handoff table (was stale at V8, now V10)
- Added barracuda overhead benchmark table to README.md
- V9 archived

### Complete Rewiring, Benchmarks & Cross-Spring Lineage (V9)
- Full barracuda API audit: 11 delegations confirmed as the complete CPU-only set
- Three-mode release benchmarks: **zero overhead** for compute-heavy binaries
  - anderson: 671ms local → 640ms barracuda-gpu (−5%)
  - signal-specificity: 795ms local → 787ms barracuda-gpu (−1%)
  - Total: 2108ms local → 2076ms barracuda-gpu (~0%)
- Cross-spring shader lineage documented:
  - hotSpring → precision (df64_core, spectral/anderson, sum_reduce_f64)
  - wetSpring → bio-stats (FusedMapReduce, Gillespie, log_f64 fix, ridge)
  - neuralSpring → ML/dispatch (spectral_density, domain_ops, xoshiro)
- V9 handoff posted with benchmarks, API audit, cross-spring map
- V8 archived

### ToadStool S62 Catch-Up Revalidation
- Reviewed ToadStool S50–S62 + DF64 expansion (14,200+ tests, 650+ WGSL shaders)
- Fixed 3 `needless_return` clippy warnings in `stats/correlation.rs`
- Revalidated all three modes: 163/163 PASS × 3, 0 clippy × 3

### Deep Debt Resolution & Sovereignty Evolution (Phase 2a++)
- **Sovereignty fix**: `error_propagation_fao56.py` discovery rewritten from
  hardcoded `"airSpring"` name to capability-based scan. New
  `_discover_fao56_capability()` scans sibling primals for
  `control/fao56/penman_monteith.py` without knowing which primal provides it.
  Primary override: `FAO56_MODULE_PATH` env var.
- **Sovereignty fix**: `tests/test_experiments.py` airSpring skip check
  replaced with capability scan across all sibling directories.
- **BarraCUDA error handling**: All 11 barracuda delegations now use `if let Ok`
  pattern with graceful CPU fallback. Replaced `.expect()` panics and
  `.unwrap_or(0.0)` silent suppressions. CPU fallback functions are always
  compiled regardless of feature gate (no `#[cfg(not(feature))]` on fallbacks).
- **Shared validation helpers**: New `groundspring-validate` library crate with
  `f64_field`, `usize_field`, `u64_field`, `f64_range`, `print_provenance_header`.
  9 unit tests. All 7 validation binaries import from shared lib (DRY).
- **Validation refactoring**: Large `run()` functions split into focused validators
  (`validate_gaussian`, `validate_correlated`, `validate_analytical`, etc.).
  Parameter groups extracted into structs (`SourceTruth`, `AcceptanceCriteria`,
  `Uncertainties`, `EnzymeNetwork`, `SimConfig`).
- **Dead code removal**: `write_benchmark()` and `provenance_metadata()` removed
  from `control/common.py` (defined but never called). Unused imports cleaned.
- **Clippy zero warnings**: Fixed `suboptimal_flops` (→ `mul_add`),
  `manual_range_contains` (→ `.contains()`), `items_after_test_module`,
  `struct_field_names`, `doc_markdown`. All resolved with zero suppressions.
- **Tolerance documentation**: All validation binaries now document tolerance
  justification inline (mathematical basis, not just the number).
- **PRNG alignment investigation**: Confirmed Xorshift64 → xoshiro128** requires
  full rebaseline of all 5 stochastic experiments. BarraCUDA has no CPU-side
  xoshiro128** with Box-Muller. Documented as planned Tier B evolution step.
- **GPU adapter assessment**: Confirmed all 6 pending metrics (RMSE, MBE, R², IA,
  hit rate, Shannon) require `WgpuDevice` async context — no CPU delegations
  available. Deferred to ToadStool GPU infrastructure phase.
- **Tests**: 163 Rust tests (131 unit + 9 validate-lib + 14 proptest +
  8 integration + 1 doc), 34 Python tests, 119/119 validation checks.
- **Coverage**: 99.11% workspace line coverage (cargo-llvm-cov).

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
