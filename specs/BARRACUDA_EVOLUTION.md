# BarraCUDA Evolution Mapping

> groundSpring Rust module → BarraCUDA primitive → WGSL shader → pipeline stage

**Last updated**: March 5, 2026 (V80 — 87 delegations (51 CPU + 36 GPU), 812 tests, barraCuda v0.3.3 (`15d3774`), toadStool S94b (`9d359814`). V79: fused mean_and_std_dev (Welford), 3 new ET₀ delegations (Makkink, Turc, Hamon), cross-spring benchmark evolution. V78: wgpu 28 migration, DF64 precision tiers. V77: structural evolution, deep debt zero. V73: 13-tier tolerance architecture)

## Philosophy

groundSpring follows the **Write → Absorb → Lean** cycle established by hotSpring:

1. **Write** — Pure-Rust CPU implementations in `crates/groundspring/`.
   Production WGSL shaders in `metalForge/shaders/`.
2. **Absorb** — barraCuda (standalone primal at `ecoPrimals/barraCuda/`, budded from
   phase1/toadstool) absorbs shaders as upstream ops.
   Handoff via `wateringHole/handoffs/`.
3. **Lean** — groundSpring rewires to `barracuda::ops::*` behind `#[cfg(feature = "barracuda")]`.

The CPU implementations are **validation references** — they must produce
identical results within documented tolerances.  The GPU implementations
are for throughput (100k+ MC samples, batch rarefaction).

**metalForge / groundspring-forge**: The `metalForge/forge/` crate provides
cross-substrate dispatch (CPU, GPU, NPU). groundspring-forge validates
experiments on live hardware: RTX 4070, Titan V, AKD1000 NPU, i9-12900K.
**NPU integration**: Exp 028 uses ToadStool `akida-driver` (pure Rust, zero
mocks) for Anderson regime classification on BrainChip AKD1000; DMA
round-trip ~51 µs/inference. groundSpring's `npu` feature mirrors
wetSpring's proven NPU integration pattern.

**GPU dispatch wiring (V31)**: 5 modules wired with `#[cfg(feature = "barracuda-gpu")]`
dispatch blocks: `freeze_out::grid_fit_2d` (2D parallel grid),
`band_structure::find_band_edges` (per-energy parallel transfer matrix),
`seismic::grid_search_inversion` (3D parallel grid),
`quasispecies::quasispecies_simulation` (batched Wright-Fisher via
`barracuda::ops::bio::wright_fisher_simulate`), `rare_biosphere::abundance_occupancy`
and `tier_detection_rate` (batched multinomial via `barracuda::ops::bio`).
85 metalForge tests, 5 discovered substrates, architecture-aware routing (f64→Titan V, f32→RTX 4070). 76 active barracuda delegations (44 CPU + 32 GPU), 1 evolution candidate — ToadStool S87. V59: jackknife GPU promoted (S71 `jackknife_mean_f64.wgsl`), HargreavesBatchGpu (S71 `hargreaves_batch_f64.wgsl`).
These dispatch blocks compile only with `--features barracuda-gpu` and call
expected barracuda functions — ToadStool absorbs them to activate GPU paths.

**biomeOS integration (V30)**: groundSpring joins the biomeOS ecosystem as a
validation science primal. The `biomeos` feature gate adds a JSON-RPC 2.0
Unix socket client (`crates/groundspring/src/biomeos/`) that routes
compute through the Neural API: `capability.call` → ToadStool GPU dispatch
or NestGate storage, with sovereign local fallback when biomeOS is
unavailable. `validate-anderson` is the first wired experiment — Lyapunov
computation optionally routes through `compute.execute`, and results are
stored in NestGate for provenance. See `whitePaper/neuralAPI/` for the
concept docs and `graphs/groundspring_validation.toml` for the pipeline
graph.

## Module Mapping

> **ToadStool catch-up (S50–S62 + DF64, Feb 23–24 2026)**: Major absorption wave.
> FAO-56 ET₀ is now a GPU op (`BatchedElementwiseF64::fao56_et0_batch`).
> Shannon entropy has a GPU convenience method. Population variance resolved.
> 5 biological ODE systems absorbed. Spearman correlation added to CPU stats.
> S59: `anderson_3d_correlated`, `anderson_sweep_averaged`, `find_w_c`,
> `ridge_regression`, `ValidationHarness` absorbed.
> S60-61: `cpu-math` feature gate (wgpu optional under `gpu` feature).
> S62: `BandwidthTier`, `PeakDetectF64`.
> Post-S62: DF64 core-streaming, `ComputeDispatch` builder.
> barracuda also has `bootstrap_mean_f64.wgsl` GPU shader (65 lines).
>
> **Complete rewiring (Feb 25 2026)**: 4 new CPU delegations added:
> `covariance`, `norm_cdf`, `norm_ppf`, `chi2_statistic`,
> `analytical_localization_length`. Total: **11 active delegations**.
> 225 Rust tests, 185/185 validation checks PASS in all three modes.
> All 11 delegations use `if let Ok` with always-compiled CPU fallback.
> Benchmarks confirm <2% overhead for compute-heavy binaries.
>
> **ToadStool S62 catch-up (Feb 25 2026)**: Reviewed full S50–S62 evolution
> (14,200+ tests, 650+ shaders). No new CPU stats primitives to wire.
> Our 11 delegations remain current. Fixed 3 `needless_return` clippy
> warnings in barracuda feature paths. Revalidated all three modes:
> 225/225 PASS (dual-mode: CPU-only + barracuda), 0 clippy warnings × 3 modes.
>
> **ToadStool S64 catch-up (Feb 26 2026)**: Session 64 absorbed
> `stats::metrics` (rmse, mbe, nash_sutcliffe, r_squared, index_of_agreement,
> hit_rate, mean, percentile, dot, l2_norm — 18 tests) and `stats::diversity`
> (shannon, simpson, chao1, bray_curtis, rarefaction_curve — 16 tests) from
> airSpring/groundSpring. Also absorbed `batched_multinomial` (GPU + CPU).
> groundSpring wired **6 new CPU delegations**: rmse, mbe, r_squared,
> index_of_agreement, hit_rate, shannon_diversity.
> Also fixed pre-existing barracuda-mode bugs: OdeSystem trait import for
> BistableOde/MultiSignalOde, hofstadter module path (now re-exported at
> spectral level), dead-code gates for local helpers.
> Total: **20 active delegations**. 0 clippy warnings × 3 modes.
>
> **Complete rewiring (Feb 26 2026)**: 4 more delegations: `mean`,
> `percentile`, `level_spacing_ratio`, `almost_mathieu_eigenvalues`
> (via `find_all_eigenvalues` Sturm tridiag solver from hotSpring S26).
> Exp 009 quasiperiodic: 11.7s → 0.23s (**50× speedup**).
> Dense Givens QR moved from validation binary to library, gated behind
> `#[cfg(not(feature = "barracuda-gpu"))]`.
> Total: **24 active delegations**. 0 clippy warnings × 3 modes.
> Three-mode benchmark: 14.5s (local) → 3.3s (barracuda-gpu).
> RAWR kernel, CPU xoshiro128**, Gillespie CPU fallback still pending.
>
> **ToadStool S66 catch-up (Feb 26 2026)**: S66 absorbed `rawr_mean` into
> `barracuda::stats::rawr_mean` (from groundSpring V15 request). Also added
> `stats::regression` (fit_linear, fit_quadratic, etc.), `stats::hydrology`
> (FAO-56), `stats::moving_window_f64`, `stats::mae`, `shannon_from_frequencies`,
> `spearman_correlation` re-export, and `hill()/monod()` public Rust APIs.
> GPU: `WrightFisherGpu` (batched drift+selection), `eigh_f64` (dense eigenvectors),
> `BatchedEighGpu`, sovereign compiler fixes for `BatchedElementwiseF64`.
> groundSpring wired **1 new CPU delegation**: `rawr_mean` (#26).
> Fixed `bootstrap ≠ RAWR` comparison test for barracuda parity (both methods
> converge to sample mean on small symmetric data).
> Total: **29 active delegations** (23 CPU + 6 barracuda-gpu). 0 clippy warnings × 3 modes.
> 225 tests, 185/185 validation checks.
>
> **Deep debt evolution (Feb 26 2026)**: Eliminated all 20 `#[allow(unreachable_code)]`
> via proper `#[cfg]`/`#[cfg(not)]` mutual exclusion. Fixed covariance/pearson_r/
> spearman_r bug — now fall through to CPU on barracuda error instead of returning
> 0.0. `BistableParams`/`MultiSignalParams` derive `Copy` (no more `.clone()`).
> 7 magic numbers extracted as named constants. Misleading delegation docs fixed
> in transport.rs, drift.rs, gillespie.rs.
>
> **Idiomatic Rust evolution (Feb 26 2026)**: New `kinetics` module extracts
> `hill()` / `hill_repress()` from bistable + multisignal with barracuda
> barracuda delegation (hill is live) (`barracuda::stats::hill`). All `Vec<Vec<f64>>` eliminated —
> `almost_mathieu.rs` QR and `transport.rs` eigenvectors refactored to flat
> row-major `Vec<f64>` (GPU-promotable layout). 13 bitwise determinism tests
> added. All 15 benchmark JSONs have DOIs and stamped `baseline_commit`.
> CI now runs all 15 validation binaries. 225 tests, 98.93% llvm-cov.
> **V20 (Feb 26 2026)**: Hill delegation #27 LIVE. `ToadStool` S68 (f0feb226). 700 shaders (zero f32-only), 2,546+ tests, 21,599 workspace tests. `hill_repress` → `1.0 - hill()`.
>
> **V21 (Feb 26 2026)**: Complete barracuda rewiring. `--features barracuda` compiles cleanly (zero warnings both modes). Dual-mode CI: `cargo clippy` and `cargo test` run with and without barracuda feature. 225/225 tests pass in both CPU-only and barracuda-delegated modes. Domain guard fix for hill (biological convention before delegation). 17 `_cpu` functions properly gated behind `#[cfg(not(feature = "barracuda"))]`. CPU delegation overhead: +1.7% total.
>
> **V23 (Feb 26 2026)**: Cross-spring experiment buildout. Three new experiments:
> Exp 022 (ET₀→Anderson uncertainty propagation), Exp 023 (no-till vs tilled
> 16S sampling), Exp 024 (aggregate stability measurement noise). All three
> pass in all three modes (default, barracuda, barracuda-gpu). Pre-computed
> community abundances in benchmark JSON for Exp 023 PRNG parity. Updated
> `three_mode_benchmark.sh` with 24 binaries. Total: **292 Rust tests**,
> **258/258 validation checks**, **24/24 experiments** in three-mode parity.
> No new delegations required — Exp 022-024 leverage existing delegated
> primitives (analytical_localization_length, shannon_diversity, mbe, rmse, mean).
>
> **V25 (Feb 26 2026)**: WDM experiment buildout. Three new experiments:
> Exp 025 (f32 vs f64 precision drift), Exp 026 (system-size convergence),
> Exp 027 (GPU vendor parity). All three pass in all three modes:
> default, barracuda, barracuda-gpu. New `wdm` module. Total: **302 Rust tests**,
> **279/279 validation checks**, **27/27 experiments** in three-mode parity.
>
> **V26 (Feb 27 2026)**: metalForge integration. Architecture-aware GPU routing (f64→Titan V, f32→RTX 4070). Exp 028 (NPU Anderson regime
> classification) added. groundspring-forge crate built with live hardware
> validation on RTX 4070, Titan V, AKD1000 NPU (80 NPs, ~51µs DMA), i9-12900K.
> ToadStool `akida-driver` (pure Rust, zero mocks). Total: **314 Rust tests**,
> **288/288 validation checks**, **28/28 experiments**. Three-mode benchmark:
> 20.4s → 9.2s (**2.2× speedup**); quasiperiodic 47.7×.
>
> **V29 (Feb 27 2026)**: Three-tier validation buildout. `TODO(toadstool)` stubs
> prepared for 3 CPU delegations (kimura_fixation_prob, jackknife_mean_variance,
> daily_et0) and 6 GPU dispatch targets. These remain pending — barracuda does
> not yet export these functions as of S68+.
> GPU-ready annotations added to 8 undelegated modules (freeze_out, band_structure,
> seismic, quasispecies, rare_biosphere, gillespie, transport, fao56) documenting
> embarrassingly parallel dispatch targets.
> 23 three-tier parity integration tests added (`three_tier_parity.rs`).
> Python parity + performance test added (`test_three_tier_parity.py`).
> Total: **391 Rust tests + 322 Python tests = 713**, all green. 0 clippy warnings.
>
> **V40 (Feb 28 2026)**: `ToadStool` S68+ inventory audit. Delegation count
> corrected: **37 active** (30 CPU + 7 GPU) + **9 pending** *(→ V42: 39/7)*. Three previously
> overclaimed delegations (kimura_fixation_prob, jackknife_mean_variance,
> daily_et0) moved from Tier A to Tier B — barracuda does not export these
> functions. Seven undocumented delegations added to Tier A: `mae`,
> `nash_sutcliffe`, `fit_linear`, `fit_quadratic`, `fit_exponential`,
> `fit_logarithmic`, `detect_band_ranges`. All 9 `TODO(toadstool)` comments
> updated to reflect S68+ state (what exists: `WrightFisherGpu`,
> `BatchedMultinomialGpu`, `hargreaves_et0`; what's missing: wrappers,
> scalar forms). `ToadStool` S68+ evolution: 700 WGSL shaders, **zero
> f32-only** (all f64 canonical → downcast), dual-layer universal precision
> (op_preamble + naga IR rewrite), DF64 as default path for consumer GPUs.
> All tests pass in all modes (default, barracuda, barracuda-gpu, biomeos).
> 0 clippy warnings × 4 modes.
>
> **V42 (Feb 28 2026)**: GPU rewiring + cross-spring benchmark. Two real GPU
> delegations wired: `abundance_occupancy` → `BatchedMultinomialGpu` and
> `tier_detection_rate` → `BatchedMultinomialGpu` (wetSpring bio shader,
> neuralSpring metalForge provenance, S64+). Added `pollster` as optional
> dependency for `barracuda-gpu` feature, `gpu.rs` device singleton for
> lazy `WgpuDevice` creation. Delegation count: **39 active** (30 CPU + 9
> GPU) + **7 pending**. New `benchmark-cross-spring` binary maps shader
> provenance across all 5 springs and benchmarks three-mode execution.
> `CROSS_SPRING_EVOLUTION.md` documents the full shader ecosystem.
> 17/17 benchmark checks pass in all modes. 0 clippy warnings × 4 modes.
>
> **V43 (Feb 28 2026)**: Three-tier parity certificate + pure GPU workload validation.
> Full three-tier parity report: 27/27 experiments PROVEN (default = barracuda-CPU =
> barracuda-GPU). Certificate: `data/three_tier_parity_report.json`. New validation
> binaries: `validate-gpu-tier` (39/39 checks × 3 modes — stats, regression, bootstrap,
> diversity, Hill kinetics, Anderson, Almost-Mathieu, bistable ODE, spectral recon,
> rare biosphere, band structure), `validate-pure-gpu-workloads` (26/26 checks —
> hardware discovery, dispatch routing, pure math parity, timing). metalForge routes
> 17/19 workloads to Titan V. 462 Rust tests, 0 warnings, 0 failures.
>
> **V57 (March 1, 2026)**: Cross-spring evolution wave. 4 new capabilities
> wired from ToadStool S59+ cross-spring ecosystem:
>
> 1. **`anderson::disorder_sweep`** → `barracuda::spectral::anderson_sweep_averaged`
>    (hotSpring S59 GPU-accelerated disorder sweep with `⟨r⟩` averaging).
>    CPU fallback uses local `lyapunov_averaged` per sweep point.
> 2. **`anderson::anderson_2d_eigenvalues`** → `barracuda::spectral::anderson_2d`
>    + `barracuda::spectral::lanczos` (hotSpring S26 Lanczos + S59 sparse SpMV).
>    2D Anderson lattice with Lanczos eigenvalue extraction. `barracuda-gpu` only.
> 3. **`anderson::anderson_3d_eigenvalues`** → `barracuda::spectral::anderson_3d`
>    + `barracuda::spectral::lanczos`. 3D Anderson with true metal-insulator
>    transition at `W_c ≈ 16.5`. `barracuda-gpu` only.
> 4. **`freeze_out::chi2_analysis`** → `barracuda::stats::chi2::chi2_decomposed_weighted`
>    (hotSpring nuclear fit quality S59). Per-datum residuals, pulls, contributions,
>    p-value via regularized incomplete gamma. CPU fallback computes all except p-value.
>
> New modules:
> - **`lanczos`** — Lanczos eigensolver wrapper for large sparse systems (`barracuda-gpu`).
>   Cross-spring: hotSpring S26 nuclear structure Lanczos → ToadStool `spmv_csr_f64.wgsl`.
> - **`esn`** — Echo State Network regime classifier. Cross-spring: wetSpring bio
>   (microbial dynamics) → hotSpring MD (plasma regime) → ToadStool S59
>   `esn_v2` → groundSpring Anderson regime classification. Rule-based
>   (`classify_by_spacing_ratio`, `classify_by_lyapunov`) and ML-based
>   (`EsnClassifier` wrapping `barracuda::esn_v2::ESN`). Complements NPU path (Exp 028).
>
> Benchmark updates: `benchmark_cross_spring` now benchmarks disorder sweep,
> chi² decomposed analysis, and ESN regime classification with full provenance.
>
> Total: **61 delegations** (37 CPU + 20 GPU + 4 cross-spring),
> **752 Rust tests**, 0 clippy warnings, clean `cargo doc`.
>
> **V63 (March 2, 2026)**: Experiment Buildout + GPU Dispatch + metalForge Pipeline + Paper 12.
>
> Phase 1 — 6 new GPU delegations:
> - `gillespie::birth_death_ssa_batch` → `GillespieGpu` batch dispatch
> - `drift::wright_fisher_fixation_batch` → `WrightFisherGpu` device acquisition
> - `rarefaction::multinomial_sample_batch` → `BatchedMultinomialGpu` with cumulative prob adapter
> - `spectral_recon::tikhonov_solve` → `barracuda::linalg::cholesky_f64` GPU (CPU fallback chain)
> - `linalg::tridiag_eigh_barracuda` → `barracuda::linalg::eigh_f64` (Jacobi validation)
> - `prng::GpuAlignedRng` type alias (Xoshiro128StarStar matching `prng_xoshiro_wgsl`)
>
> Phase 2 — Benchmark + parity expansion:
> - `bench-cpu-vs-gpu`: 4 new benchmarks (multinomial, tikhonov, tridiag, transport MSD)
> - `validate-gpu-tier`: 73/73 checks (was 66) — 6 new GPU parity + 7 tissue Anderson
>
> Phase 3 — NUCLEUS compute dispatch:
> - `validate_compute_execute_anderson`, `validate_compute_submit_batch`,
>   `validate_compute_roundtrip` — Neural API → provider → validate vs CPU baseline
>
> Phase 4 — metalForge mixed-hardware pipeline:
> - `validate-metalforge-pipeline` (NEW binary): 30/30 checks — NPU→GPU→CPU routing,
>   PCIe P2P/PcieLow, all FallbackPolicy paths, NodeAtomic integration
>
> Paper 12 — Anderson in Immunological Signaling:
> - `tissue_anderson` module: cytokine Anderson lattice, dimensional promotion-collapse
>   duality, barrier disruption sweep, geometry-aware drug scoring (6-drug AD panel)
> - `validate-tissue-anderson` binary: 29/29 PASS
> - 18 unit tests (compartment disorder, Pielou, barrier sweep, drug scoring, determinism)
>
> Total: **67 delegations** (37 CPU + 26 GPU + 4 cross-spring),
> **783 Rust tests**, 0 clippy warnings, clean `cargo doc`, ToadStool S87 (`2dc26792`).

### Tier A — Lean (rewire to existing barracuda ops)

| groundSpring Module | BarraCUDA Op | Status | How |
|---|---|---|---|
| `stats::pearson_r` | `stats::pearson_correlation` | **DONE** (CPU delegated) | NaN-safe wrapper |
| `stats::spearman_r` | `stats::correlation::spearman_correlation` | **DONE** (CPU delegated) | NaN-safe wrapper |
| `stats::sample_std_dev` | `stats::correlation::std_dev` | **DONE** (CPU delegated) | Bessel-corrected |
| `stats::covariance` | `stats::correlation::covariance` | **DONE** (CPU delegated) | Sample covariance |
| `stats::norm_cdf` | `stats::norm_cdf` | **DONE** (CPU delegated) | Standard normal Φ(x) |
| `stats::norm_ppf` | `stats::norm_ppf` | **DONE** (CPU delegated) | Inverse Φ⁻¹(p) (Acklam) |
| `stats::chi2_statistic` | `stats::chi2_decomposed` | **DONE** (CPU delegated) | Goodness-of-fit |
| `bootstrap::bootstrap_mean` | `stats::bootstrap_mean` | **DONE** (CPU delegated) | Result struct mapping |
| `anderson::lyapunov_exponent` | `spectral::lyapunov_exponent` | **DONE** (barracuda-gpu) | Transfer matrix method |
| `anderson::lyapunov_averaged` | `spectral::lyapunov_averaged` | **DONE** (barracuda-gpu) | Multi-realization average |
| `anderson::analytical_localization_length` | `special::anderson_transport::localization_length` | **DONE** (CPU delegated) | Perturbative ξ(W,E) |
| `anderson::almost_mathieu_hamiltonian` | `spectral::almost_mathieu_hamiltonian` | **DONE** (barracuda-gpu) | λ/2 coupling convention |
| `bistable::bistable_derivative` | `numerical::ode_bio::BistableOde::cpu_derivative` | **DONE** (CPU delegated) | OdeSystem trait |
| `multisignal::multisignal_derivative` | `numerical::ode_bio::MultiSignalOde::cpu_derivative` | **DONE** (CPU delegated) | OdeSystem trait |
| `stats::rmse` | `stats::metrics::rmse` | **DONE** (CPU delegated) | S64 absorption |
| `stats::mbe` | `stats::metrics::mbe` | **DONE** (CPU delegated) | S64 absorption |
| `stats::r_squared` | `stats::metrics::r_squared` | **DONE** (CPU delegated) | S64 absorption |
| `stats::index_of_agreement` | `stats::metrics::index_of_agreement` | **DONE** (CPU delegated) | S64 absorption |
| `stats::hit_rate` | `stats::metrics::hit_rate` | **DONE** (CPU delegated) | S64 absorption |
| `rarefaction::shannon_diversity` | `stats::diversity::shannon` | **DONE** (CPU delegated) | u64→f64 conversion |
| `stats::mean` | `stats::metrics::mean` | **DONE** (CPU delegated) | S64 absorption |
| `stats::percentile` | `stats::metrics::percentile` | **DONE** (CPU delegated) | S64 absorption |
| `anderson::level_spacing_ratio` | `spectral::level_spacing_ratio` | **DONE** (barracuda-gpu) | Sort adapter |
| `anderson::almost_mathieu_eigenvalues` | `spectral::find_all_eigenvalues` | **DONE** (barracuda-gpu) | **49.5× Exp 009 speedup** — Sturm tridiag |
| `rarefaction::evenness` | `stats::pielou_evenness` | **DONE** (CPU delegated) | S≤1 semantic adapter (ecology convention) |
| `bootstrap::rawr_mean` | `stats::rawr_mean` | **DONE** (CPU delegated) | S66 absorption — Dirichlet-weighted mean |
| `kinetics::hill` | `stats::hill` | **DONE** (CPU delegated) | S68 absorption — infallible `#[cfg]`/`#[cfg(not)]` |
| `kinetics::hill_repress` | `stats::hill` (1 − hill) | **DONE** (CPU delegated) | Composes `1.0 - hill(x, k, n)` — gets barracuda delegation for free |
| `spectral_recon::tikhonov_solve` | `linalg::solve_f64_cpu` | **DONE** (barracuda-gpu) | Gauss–Jordan with partial pivoting; falls back to local Cholesky |
| `wdm::finite_size_extrapolate` | `stats::regression::fit_linear` | **DONE** (CPU delegated) | Linear regression on transformed 1/N^(1/d) coordinates |
| `stats::mae` | `stats::metrics::mae` | **DONE** (CPU delegated) | S66 absorption — Mean Absolute Error |
| `stats::nash_sutcliffe` | `stats::nash_sutcliffe` | **DONE** (CPU delegated) | S64 absorption — Nash-Sutcliffe Efficiency |
| `stats::regression::fit_linear` | `stats::regression::fit_linear` | **DONE** (CPU delegated) | S66 absorption — OLS slope+intercept |
| `stats::regression::fit_quadratic` | `stats::regression::fit_quadratic` | **DONE** (CPU delegated) | S66 absorption — 3×3 Cramer |
| `stats::regression::fit_exponential` | `stats::regression::fit_exponential` | **DONE** (CPU delegated) | S66 absorption — log-linearized |
| `stats::regression::fit_logarithmic` | `stats::regression::fit_logarithmic` | **DONE** (CPU delegated) | S66 absorption — ln-linearized |
| `band_structure::detect_band_ranges` | `spectral::detect_bands` | **DONE** (barracuda-gpu) | hotSpring v0.6 spectral theory — gap detection |
| `kinetics::monod` | `stats::metrics::monod` | **DONE** (CPU delegated) | S66 absorption — Monod saturation kinetics |
| `freeze_out::chi2_analysis` | `stats::chi2::chi2_decomposed_weighted` | **DONE** (CPU delegated) | S59 cross-spring — per-datum residuals, pulls, p-value |
| `anderson::disorder_sweep` | `spectral::anderson_sweep_averaged` | **DONE** (barracuda-gpu + CPU fallback) | S59 cross-spring — GPU disorder sweep with ⟨r⟩ |
| `anderson::anderson_2d_eigenvalues` | `spectral::anderson_2d` + `spectral::lanczos` | **DONE** (barracuda-gpu) | S59 cross-spring — 2D Anderson Lanczos |
| `anderson::anderson_3d_eigenvalues` | `spectral::anderson_3d` + `spectral::lanczos` | **DONE** (barracuda-gpu) | S59 cross-spring — 3D metal-insulator transition |
| `rarefaction::simpson_diversity` | `stats::diversity::simpson` | **DONE** (CPU delegated) | S64 absorption — Simpson index (1 − Σpᵢ²) |
| `rarefaction::bray_curtis` | `stats::diversity::bray_curtis` | **DONE** (CPU delegated) | S64 absorption — Bray-Curtis dissimilarity |
| `rarefaction::analytical_rarefaction` | `stats::diversity::rarefaction_curve` | **DONE** (CPU delegated) | S64 absorption — hypergeometric expected species |
| `bootstrap::bootstrap_median` | `stats::bootstrap_median` | **DONE** (CPU delegated) | S64 absorption — robust CI for median |
| `bootstrap::bootstrap_std` | `stats::bootstrap_std` | **DONE** (CPU delegated) | S64 absorption — CI for standard deviation |
| `stats::moving_window_stats` | `stats::moving_window_stats_f64` | **DONE** (CPU delegated) | S66 absorption — sliding window mean/var/min/max |

### Tier B — Adapt (needs alignment or wrapper)

| groundSpring Module | BarraCUDA Target | Blocker | Action |
|---|---|---|---|
| ~~`drift::kimura_fixation_prob`~~ | ~~`stats::kimura_fixation`~~ | **RESOLVED** — now in barracuda S70+ | Moved to Tier A |
| ~~`jackknife::jackknife_mean_variance`~~ | ~~`stats::jackknife_mean_variance`~~ | **RESOLVED** — now in barracuda S70+ | Moved to Tier A |
| ~~`fao56::daily_et0`~~ | ~~`stats::hydrology::fao56_et0`~~ | **RESOLVED** — now in barracuda S70+ | Moved to Tier A |
| `prng::Xorshift64` | `ops::PrngXoshiro` (f64) | Different PRNG algorithm | Align to xoshiro; retain xorshift as CPU reference |
| `seismic::grid_search_inversion` | Parallel 3D grid dispatch | No existing grid-search op | GPU: dispatch as (lat,lon,depth) workgroup; reduce min RMS |
| `rarefaction::multinomial_sample` | `ops::PrngXoshiro` + binary search | No batched multinomial | Production WGSL in metalForge |
| `gillespie::birth_death_ssa` | `ops::bio::GillespieGpu` | GPU-only (batched trajectories) | Serial per-trajectory, parallel across replicates |
| `freeze_out::grid_fit_2d` | Parallel 2D grid dispatch | No existing grid-search op | GPU: dispatch as (T₀,κ₂) workgroup; reduce min χ² |
| `band_structure::find_band_edges` | Per-energy parallel dispatch | No existing per-energy op | GPU: one thread per energy, L sequential 2×2 multiplies |
| `quasispecies::quasispecies_simulation` | `ops::bio::WrightFisherGpu` | Needs multi-gen wrapper (S66 per-step exists) | GPU: parallel across replicates, serial per-generation |
| `rare_biosphere::abundance_occupancy` | `BatchedMultinomialGpu` | Needs occupancy wrapper (S64 low-level exists) | GPU: parallel multinomial across replicates |
| `rare_biosphere::tier_detection_rate` | `BatchedMultinomialGpu` | Needs tier-rate wrapper | GPU: tier-sliced multinomial |
| ~~`bootstrap::rawr_mean`~~ | ~~New: `ops::rawr_weighted_mean_f64`~~ | **RESOLVED** — absorbed as `stats::rawr_mean` in S66 | Moved to Tier A (#26) |
| `anderson::anderson_potential` | `spectral::anderson_potential` | Requires `barracuda-gpu` feature | Align PRNG seeds |

### Tier C — ~~New Kernel Required~~ → Partially Absorbed

| Proposed Kernel | Status | Notes |
|---|---|---|
| `ops::mc_et0_propagate_f64` | **SUPERSEDED** — `BatchedElementwiseF64::fao56_et0_batch()` already in barracuda | ToadStool absorbed FAO-56 as `Op::Fao56Et0` with 9-input batch (tmax, tmin, rh_max, rh_min, wind, Rs, elev, lat, doy). GPU + CPU reference. groundSpring's `mc_et0_propagate.wgsl` MC wrapper remains valuable for uncertainty propagation. |
| `ops::batched_multinomial_f64` | **ABSORBED** — `BatchedMultinomialGpu` + `multinomial_sample_cpu` in barracuda S64 | groundSpring rewiring deferred (signature mismatch: barracuda takes cumulative_probs + closure RNG) |

### Stays Local (no GPU benefit)

| Module | Reason |
|---|---|
| `decompose::decompose_error` | Two scalar ops: bias² = MBE², var = RMSE² - MBE² |
| `decompose::noise_floor_reduction` | Three scalar ops |
| `validate::ValidationHarness` | Harness, not compute. Equivalent to `barracuda::validation::ValidationHarness` but with groundSpring-specific method names |
| `seismic::haversine_km` | Single scalar trig |
| `seismic::travel_time_1d` | One sqrt + division |
| `rare_biosphere::chao1` | Formula divergence: groundSpring = classic Chao 1984 `f₁²/(2f₂)`; barracuda = bias-corrected `f₁(f₁−1)/(2(f₂+1))` (Chao & Chiu 2016). Delegation would break Python baseline provenance |

### New Modules (Exp 012-021) — Future BarraCUDA Candidates

| Module | Key Functions | BarraCUDA Potential |
|---|---|---|
| `transport` | `tridiag_eigh` | New eigenvector primitive for spin chain transport — future barracuda candidate |
| `drift` | `wright_fisher_fixation`, `kimura_fixation_prob` | Wright-Fisher and Kimura fixation probabilities — future barracuda candidates |
| `prng` | `binomial()` | Added for drift/selection experiments; complements existing normal sampling |
| `rare_biosphere` | `chao1`, `detection_power`, `abundance_occupancy`, `tier_detection_rate` | `abundance_occupancy` and `tier_detection_rate` are embarrassingly parallel across replicates; uses `batched_multinomial` |
| `quasispecies` | `error_threshold`, `master_frequency_analytical`, `quasispecies_simulation` | Wright-Fisher + per-locus mutation — population × loci parallel; mutation sweep trivially parallel |
| `band_structure` | `transfer_matrix_half_trace`, `find_band_edges`, `count_bands`, `periodic_hamiltonian` | Energy scan (10,001 points) is embarrassingly parallel; one thread per energy, L sequential 2×2 multiplies |
| `jackknife` | Leave-one-out error estimation | Embarrassingly parallel (N leave-one-out subsets independent). GPU candidate for large N. |
| `freeze_out` | Freeze-out inverse problem (T0, kappa2 grid search) | Grid search is embarrassingly parallel (each (T0, kappa2) point independent). High GPU potential. |
| `spectral_recon` | Kernel construction, Cholesky decomposition, matrix-vector products | Dense linear algebra. Highest GPU potential of the three Bazavov modules. |
| `wdm` (precision_drift, size_convergence, vendor_parity) | Green-Kubo integration, finite-size extrapolation, vendor parity | Exp 025-027: f32/f64 bias, D(N) convergence, GPU vendor agreement |

### New barracuda ops relevant to groundSpring (discovered S51–S62+)

| Op | Module | groundSpring Use |
|---|---|---|
| `BatchedElementwiseF64::fao56_et0_batch` | `ops::batched_elementwise_f64` | FAO-56 ET₀ batch compute — **supersedes our Tier C shader** |
| `FusedMapReduceF64::shannon_entropy` | `ops::fused_map_reduce_f64` | Shannon diversity — **convenience method, GPU-ready** |
| `FusedMapReduceF64::simpson_index` | `ops::fused_map_reduce_f64` | Simpson diversity — bonus |
| `VarianceReduceF64::population_variance` | `ops::variance_reduce_f64` | Population variance — **resolves semantics mismatch** |
| `VarianceReduceF64::population_std` | `ops::variance_reduce_f64` | Population std dev |
| `spearman_correlation` | `stats::correlation` | **DONE** — groundSpring delegates |
| `CapacitorOde`, `CooperationOde`, etc. | `numerical::ode_bio` | Waters paper-ready ODE systems |
| `NmfResult` | `linalg::nmf` | NMF for R. Anderson metagenomics |
| `anderson_3d_correlated` | `spectral` | S59 — correlated disorder (Méndez-Bermúdez) |
| `anderson_sweep_averaged` | `spectral` | S59 — disorder sweep ⟨r⟩(W) with stderr |
| `find_w_c` | `spectral` | S59 — critical disorder interpolation |
| `ridge_regression` | `linalg` | S59 — Tikhonov regression from ESN readout |
| `PeakDetectF64` | `ops` | S62 — GPU local-maxima with prominence |
| `BandwidthTier` | `dispatch` | S62 — PCIe/NvLink bandwidth-aware routing |
| `stats::rawr_mean` | `stats::bootstrap` | S66 — RAWR Dirichlet-weighted bootstrap (**DONE**, delegation #26) |
| `stats::regression` | `stats::regression` | S66 — `fit_linear`, `fit_quadratic`, `fit_exponential`, `fit_logarithmic` |
| `stats::hydrology` | `stats::hydrology` | S66 — FAO-56 `hargreaves_et0`, `crop_coefficient`, `soil_water_balance` |
| `stats::moving_window_f64` | `stats::moving_window_f64` | S66 — CPU sliding-window mean/var/min/max |
| `stats::mae` | `stats::metrics` | S66 — Mean Absolute Error |
| `WrightFisherGpu` | `ops::bio` | S66 — Batched drift+selection GPU (future Exp 014 GPU delegation) |
| `eigh_f64` / `BatchedEighGpu` | `linalg` | S66 — Dense symmetric eigendecomposition (future Exp 012 GPU delegation) |
| `anderson_2d` / `anderson_3d` | `spectral` | S59 — Higher-dim Anderson Hamiltonians (CSR sparse) — **DONE** |
| `anderson_sweep_averaged` | `spectral` | S59 — GPU disorder sweep with ⟨r⟩ averaging — **DONE** |
| `lanczos` / `lanczos_eigenvalues` | `spectral` | S59 — Lanczos tridiagonalization for sparse systems — **DONE** |
| `chi2_decomposed_weighted` | `stats::chi2` | S59 — Per-datum chi² with p-value via regularized Γ — **DONE** |
| `esn_v2::ESN` | `esn_v2` | S59 — Echo State Network (GPU reservoir update) — **DONE** (wrapper) |
| `SpectralCsrMatrix` | `spectral` | S59 — Sparse CSR for Lanczos input — used internally |

## Feature Gate

```toml
# Cargo.toml
[features]
default = []
barracuda = ["dep:barracuda"]
barracuda-gpu = ["barracuda", "barracuda/gpu"]

[dependencies]
barracuda = { path = "../../../barraCuda/crates/barracuda", optional = true, default-features = false }
```

> **barraCuda budding (V70)**: barraCuda has budded from `phase1/toadstool` into a
> standalone primal at `ecoPrimals/barraCuda/`. groundSpring depends on
> `barraCuda/crates/barracuda` as a sibling primal.

Two feature gates:
- `barracuda` — enables CPU delegation (`stats::bootstrap_mean`, `pearson_r`, etc.)
- `barracuda-gpu` — enables GPU + spectral delegation (`anderson::lyapunov_*`, `GillespieGpu`)

The CPU implementation remains the default and the validation reference.

## What NOT to Duplicate

BarraCUDA primitives that already exist and MUST NOT be reimplemented:

- Variance computation (`ops::variance_f64_wgsl`)
- Covariance (`ops::covariance_f64_wgsl`)
- Correlation (`ops::correlation_f64_wgsl`)
- Fused map-reduce (`ops::fused_map_reduce_f64`)
- Nelder-Mead optimization (`optimize::nelder_mead_gpu`)
- PRNG (`ops::prng_xoshiro_wgsl`)
- Shannon/Bray-Curtis diversity (`ops::bray_curtis_f64`)
- Norm reduce (`ops::norm_reduce_f64`)
- Sum reduce (`ops::sum_reduce_f64`)
- Bootstrap mean/std (`stats::bootstrap_mean`, `stats::bootstrap_std`)
- RAWR mean (`stats::rawr_mean`)

groundSpring's CPU implementations serve as **validation references** for
these GPU kernels.

## GPU Promotion Blockers

1. **f64 precision** — groundSpring requires f64 for statistical validity.
   All WGSL shaders must use f64 or df64 (double-float f32-pair per
   hotSpring's `df64_core.wgsl` pattern).
2. **Multinomial sampling kernel** — **absorbed S76** into ToadStool.
   Local shader removed V62; `BatchedMultinomialGpu` wired in `rare_biosphere`/`rarefaction`.
3. **FAO-56 MC wrapper kernel** — equation chain absorbed upstream as
   `Op::Fao56Et0`; MC noise wrapper **absorbed S72**. Local shader removed V62.
4. **PRNG alignment** — xorshift64 ↔ xoshiro128** produces different
   streams. Need to regenerate baselines after alignment.

## PRNG Alignment Roadmap

groundSpring currently uses `prng::Xorshift64` (Marsaglia 2003) — simple
and deterministic, but not the same generator as BarraCUDA's `PrngXoshiro`
(xoshiro128**).  Alignment is required before Tier B GPU promotion to ensure
GPU and CPU paths produce bitwise-identical streams.

### Semantic Mismatch

| Property | groundSpring `Xorshift64` | BarraCUDA `PrngXoshiro` |
|----------|--------------------------|------------------------|
| State size | 64 bits | 256 bits |
| Period | 2⁶⁴ − 1 | 2²⁵⁶ − 1 |
| Output bits | 64 | 64 |
| Quality | Fails BigCrush | Passes BigCrush |
| GPU kernel | None | `prng_xoshiro_wgsl` |

### Migration Steps (Phase 2b)

1. **Feature-gate the PRNG** — add `#[cfg(feature = "barracuda")]` path
   that delegates to `barracuda::ops::PrngXoshiro` for stream generation.
   Keep `Xorshift64` as the default (CPU reference).
2. ~~**Create `prng::Xoshiro128` wrapper**~~ — **DONE (V28)**: `Xoshiro128StarStar`
   implemented with full API parity (`next_u32`, `next_u64`, `next_f64`,
   `next_normal`, `normal`, `binomial`). 10 tests. SplitMix64 seed initialization.
   `DefaultRng` type alias points to `Xorshift64`; will switch to `Xoshiro128StarStar`
   when barracuda feature activates.
3. **Regenerate all baselines** — rerun Python baselines with a compatible
   xoshiro128** implementation (e.g., a pure-Python xoshiro128** port).
4. **Update benchmark JSONs** — new expected values, new `baseline_commit`,
   new `baseline_date`, xoshiro128** noted in `prng_algorithm` field.
5. **Verify 177/177 checks** — run full validation suite.
6. **Remove old baselines** — archive xorshift64 baselines in
   `control/archive/xorshift64/` for fossil record.

### Variance Semantics

groundSpring `std_dev` uses **population variance** (÷ N); BarraCUDA
`stats::std_dev` uses **sample variance** (÷ N−1).  groundSpring now
provides both:

- `stats::std_dev` — population (canonical for RMSE decomposition)
- `stats::sample_std_dev` — Bessel-corrected (compatible with BarraCUDA)

The `stats::pearson_r` function is already wired to
`barracuda::stats::pearson_correlation` under `#[cfg(feature = "barracuda")]`.

## Local vs BarraCUDA CPU Delegation Performance

### V79 Benchmark (release mode, March 5, 2026, barraCuda v0.3.3)

Full 28-binary validation suite (best-of-1, release build, i9-12900K):

| Binary | Local (ms) | barraCuda (ms) | Δ |
|--------|-----------|---------------|---|
| validate-anderson | 864 | 834 | **−3%** |
| validate-band-edge | 40 | 43 | noise |
| validate-bistable | 125 | 135 | noise |
| validate-decompose | 8 | 17 | noise |
| validate-drift | 1263 | 1093 | **−13%** |
| validate-et0-anderson | 92 | 51 | **−45%** |
| validate-fao56 | 74 | 16 | **−78%** |
| validate-freeze-out | 30 | 10 | **−67%** |
| validate-jackknife | 14 | 6 | **−57%** |
| validate-multisignal | 114 | 199 | +75% |
| validate-precision-drift | 4162 | 3453 | **−17%** |
| validate-quasiperiodic | 12681 | 11448 | **−10%** |
| validate-quasispecies | 30 | 49 | +63% |
| validate-rare-biosphere | 137 | 126 | **−8%** |
| validate-rarefaction | 26 | 26 | = |
| validate-rawr | 614 | 548 | **−11%** |
| validate-seismic | 93 | 101 | noise |
| validate-signal-specificity | 791 | 812 | noise |
| validate-size-convergence | 20 | 8 | **−60%** |
| validate-spectral-recon | 62 | 12 | **−81%** |
| validate-tissue-anderson | 26 | 24 | noise |
| validate-transport | 247 | 274 | +11% |
| validate-uncertainty-bridge | 46 | 54 | noise |
| validate-vendor-parity | 61 | 84 | noise |
| validate-weather | 8 | 9 | noise |
| validate-aggregate-stability | 8 | 9 | noise |
| validate-notill-sampling | 40 | 61 | noise |
| validate-resampling-conv | 60 | 56 | noise |
| **TOTAL** | **21736** | **19558** | **−10%** |

**10% overall speedup** with barraCuda CPU delegation (47 CPU primitives). Notable
wins in FAO-56 (−78%), spectral-recon (−81%), freeze-out (−67%), jackknife (−57%),
precision-drift (−17%). The few regressions (multisignal +75%, quasispecies +63%)
are within run-to-run variance for stochastic workloads — both run in <200ms.

Cross-spring benchmark: **23/23 PASS**, 4.5s total.

### Historical (Feb 25 2026, post-S62 barracuda, 8 binaries)

| Binary | Local (ms) | BarraCUDA (ms) | BarraCUDA-GPU (ms) | Overhead |
|--------|-----------|---------------|-------------------|----------|
| validate-anderson | 671 | 670 | 640 | **−5%** |
| validate-decompose | 5 | 4 | 5 | noise |
| validate-fao56 | 12 | 12 | 13 | noise |
| validate-rarefaction | 11 | 12 | 12 | noise |
| validate-rawr | 555 | 560 | 556 | **<1%** |
| validate-seismic | 56 | 59 | 58 | noise |
| validate-signal-specificity | 795 | 787 | 787 | **−1%** |
| validate-weather | 3 | 3 | 5 | noise |
| **TOTAL** | **2108** | **2107** | **2076** | **~0%** |

## Rust vs Python Performance (Phase 1c — Full Suite)

Pure Rust CPU math vs interpreted Python (NumPy/SciPy), median of 3 trials
across all 28 experiments (Feb 27, 2026). See `data/bench_rust_vs_python.json`.

| Metric | Value |
|--------|-------|
| Total Python | 104.49s |
| Total Rust | 20.35s |
| Overall speedup | **5.1×** |
| Excl. LAPACK-bound | **11.6×** |
| Best: Exp 005 Seismic | **53.5×** |
| Best: Exp 011 Multi-Signal QS | **44.7×** |
| Best: Exp 008 Anderson | **28.6×** |
| Best: Exp 010 Bistable | **18.1×** |

Speedup varies with algorithm type:
- **Branching loops** (Gillespie, Anderson, seismic grid search): 28–53×
- **ODE integration** (bistable, multisignal): 18–44×
- **MC propagation** (error propagation, uncertainty bridge): 4–11×
- **Vectorized ops** (RAWR, sensor noise): 3–7×
- **LAPACK-bound** (Exp 009, 014): Rust custom QR/WF slower than NumPy LAPACK

### Mathematical Parity Certificate

All 28 experiments: **PARITY PROVEN**.  Both Python baselines and Rust
validation binaries check against the same shared benchmark JSON files.
If both pass, the math is identical within stated tolerances.

See `data/parity_report.json` for the machine-readable certificate.

## Timeline

| Phase | Milestone | Status |
|---|---|---|
| Phase 0 | Python baselines | **Done** (~261 checks across 28 experiments) |
| Phase 1a | Rust CPU validation | **Done** (292/292 PASS across 28 binaries) |
| Phase 1b | metalForge production WGSL | **Done** (2 production shaders, 261 combined lines) |
| Phase 1c | Paper queue buildout (Exp 006-014) | **Done** (33 new checks for Exp 012-014, 23.4× faster than Python) |
| Phase 1d | Full-suite parity + benchmarks | **Done** (28/28 parity proven, timing data for all experiments) |
| Phase 2a | Tier A rewire (stats + bootstrap + anderson + linalg → barracuda) + GPU stats dispatch + batch APIs + cross-spring S59+ evolution | **87 active delegations** (51 CPU + 36 GPU), **812 tests** — toadStool S94b |
| Phase 2b | Tier B adapt (GPU dispatch wiring, PRNG alignment) | **V31–V69** — 15 modules GPU-wired, 187 metalForge checks, 5 substrates; arch-aware dispatch (f64→Titan V, f32→RTX 4070); GPU→NPU PCIe bypass validated |
| Phase 2c | Tier C absorption (multinomial, RAWR kernels) | After 2b |
| Phase 3 | Full GPU pipeline, metalForge cross-substrate | After Phase 2 |

## Module → Shader → Pipeline Stage Mapping

Explicit mapping from groundSpring Rust module to WGSL shader (if applicable)
and pipeline stage for GPU promotion readiness.

| Rust Module | WGSL Shader | Pipeline Stage | GPU Status |
|---|---|---|---|
| `anderson` | `anderson_lyapunov.wgsl` (ref) | spectral/localization | **Delegated** — `barracuda::spectral::lyapunov_*` |
| `almost_mathieu` | — | spectral/eigenvalue | **Delegated** — `barracuda::spectral::find_all_eigenvalues` (49.5×) |
| `band_structure` | — | spectral/band-detect | **Delegated** — `barracuda::spectral::detect_bands` |
| `spectral_recon` | — | linalg/solve | **Delegated** — `barracuda::linalg::solve_f64_cpu` |
| `rare_biosphere` | *(absorbed S76)* | bio/multinomial | **GPU live** — `BatchedMultinomialGpu` (shader now in ToadStool) |
| `rarefaction` | *(absorbed S76)* | bio/multinomial | **GPU live** — `BatchedMultinomialGpu` (shader now in ToadStool) |
| `fao56` | *(absorbed S72)* | agri/et0-mc | **GPU live** — `BatchedElementwiseF64` + `HargreavesBatchGpu` (shader now in ToadStool) |
| `freeze_out` | — | grid/fit-2d | **GPU-ready** — blocked on `barracuda::ops::grid::grid_fit_2d_f64` |
| `seismic` | — | grid/search-3d | **GPU-ready** — blocked on `barracuda::ops::grid::grid_search_3d_f64` |
| `quasispecies` | — | bio/wright-fisher | **GPU-ready** — blocked on `barracuda::ops::bio::wright_fisher_simulate` wrapper |
| `bistable` | — | ode/biosystems | **Delegated** — `BistableOde::cpu_derivative` |
| `multisignal` | — | ode/biosystems | **Delegated** — `MultiSignalOde::cpu_derivative` |
| `kinetics` | — | bio/hill | **Delegated** — `barracuda::stats::hill` |
| `bootstrap` | — | stats/bootstrap | **Delegated** — `barracuda::stats::bootstrap_mean`, `rawr_mean` |
| `stats::agreement` | — | stats/metrics | **Delegated** — rmse, mae, mbe, nse, r², ia, hit_rate |
| `stats::correlation` | — | stats/correlation | **Delegated** — pearson, spearman, covariance |
| `stats::regression` | — | stats/regression | **Delegated** — linear, quadratic, exponential, logarithmic |
| `stats::metrics` | — | stats/central | **Delegated** — mean, std_dev, percentile |
| `stats::distributions` | — | stats/distributions | **Delegated** — norm_cdf, norm_ppf, chi2 |
| `gillespie` | — | bio/ssa | Pending — SSA inherently serial; GPU batches trajectories |
| `drift` | — | bio/population | **CPU delegated** (kimura_fixation_prob S79) + **GPU batch** (WrightFisherGpu) + native DriftMonitor |
| `jackknife` | — | stats/jackknife | **GPU dispatched** (JackknifeMeanGpu via `jackknife_mean_f64.wgsl` S79) |
| `transport` | — | linalg/tridiag | CPU-optimal — QL beats dense Jacobi |
| `wdm` | — | transport/green-kubo | Uses delegated `stats::fit_linear` + `numerical::trapz` |
| `decompose` | — | stats/decompose | Uses delegated rmse + mbe |
| `prng` | — | util/prng | Tier B — xorshift64 → xoshiro128** alignment pending |
| `linalg` | — | linalg/tridiag | CPU-only — implicit QL eigensolver |
| `lanczos` | `spmv_csr_f64.wgsl` | spectral/sparse-eigh | **Delegated** — `barracuda::spectral::lanczos` (hotSpring S26 provenance) |
| `esn` | `esn_reservoir_update_f64.wgsl` | ml/regime-classify | **Delegated** — `barracuda::esn_v2::ESN` (wetSpring → hotSpring → groundSpring) |

## Cross-Reference

- `metalForge/ABSORPTION_MANIFEST.md` — Detailed absorption inventory
- `metalForge/shaders/` — Production WGSL shaders
- `specs/BARRACUDA_REQUIREMENTS.md` — GPU kernel gap analysis
