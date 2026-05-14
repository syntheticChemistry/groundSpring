# groundSpring — The Dirty Differences

[![CI](https://github.com/ecoPrimals/groundSpring/actions/workflows/ci.yml/badge.svg)](https://github.com/ecoPrimals/groundSpring/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/badge/coverage-%E2%89%A592%25-brightgreen)](https://github.com/ecoPrimals/groundSpring/actions/workflows/ci.yml)
[![License: AGPL-3.0-or-later](https://img.shields.io/badge/license-AGPL--3.0--or--later-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2024_edition-orange.svg)](https://www.rust-lang.org/)
[![ecoBin](https://img.shields.io/badge/ecoBin-compliant-green.svg)](https://github.com/ecoPrimals/wateringHole/blob/main/ECOBIN_ARCHITECTURE_STANDARD.md)

**Date**: May 13, 2026 | **License**: AGPL-3.0-or-later | **MSRV**: Rust 1.87 (2024 edition)
**Status**: V142 — Compute trio wave absorption. `shader.compile.gemm` (coralReef Sprint 11, tensor-core GEMM). `health.version` trio-consistent probe wired on both barraCuda (Sprint 69) and coralReef. **20 IPC methods across 7 primals**. sourDough deployment internalization tracked. 3 new upstream gaps surfaced (GAP-GS-015/016/017). Tier 4 IPC-first. 41 modules, 38 experiments (34 core + 4 LTEE reproductions), **1,123 tests** (zero clippy, zero fmt diff), 427/427 validation checks + 138 metalForge checks + 29 validator integration tests. **guideStone Level 4**. LTEE B1-B4 complete → lithoSpore modules 1-4 with BLAKE3-anchored ingestion manifest. Zero unsafe, zero mocks, zero `.unwrap()` in library code. plasmidBin binary: 1.1M stripped. barraCuda v0.4.0, coralReef v0.1.0, primalSpring v0.9.25.

**The gap between what models predict and what instruments measure.**

groundSpring is the reality layer in the [ecoPrimals](https://github.com/ecoPrimals) ecosystem. Where other springs validate clean science — hotSpring (nuclear math), airSpring (FAO-56 equations), wetSpring (taxonomy pipelines) — groundSpring lives in the space where those models meet the physical world.

**Core question**: "How do things actually look, and why is it different from what we expected?"

```
Clean models (other springs) → Noisy measurements (groundSpring) → Adapted models (neuralSpring)
```

## The Five Pillars

1. **Signal vs Noise** — Distinguishing real phenomena from measurement artifacts. Sensor drift, calibration error, environmental interference. When airSpring's soil moisture sensor reads 0.32 instead of 0.30, is that real heterogeneity or instrument error?

2. **Inverse Problems** — From observations back to causes. Where did an earthquake start, from its emanations? What is a star's composition, from its light frequencies? What contaminant entered the watershed, from downstream sensor readings?

3. **Sensing Systems** — The physics of measurement itself. How do different instruments see the same phenomenon differently? A thermometer, a satellite, and a reanalysis model all "measure" temperature differently. Color and size have different meaning depending on the detector.

4. **Temporal Dynamics** — How systems drift over time. Sensor degradation. Seasonal baselines. Long-term climate trends vs short-term weather noise. The geological clock vs the agricultural clock vs the astronomical clock.

5. **Spatial Propagation** — How signals travel through media. Seismic waves through rock. Light through atmosphere (extinction, redshift). Moisture through soil. Contaminants through aquifers. The medium distorts the message.

## Current Status

| Experiment | Domain | Phase 0 (Python) | Phase 1 (Rust) | Key Question |
|------------|--------|:-----------------:|:--------------:|--------------|
| 001: Sensor Noise | Agricultural | 32/32 PASS | 36/36 PASS | Bias vs variance in soil moisture sensors |
| 002: Observation Gap | Meteorological | PASS/SKIP | 13/13 PASS | Reanalysis model vs station readings |
| 003: Error Propagation | Agricultural | PASS | 15/15 PASS | How sensor noise becomes ET₀ uncertainty |
| 004: Sequencing Noise | Biological | PASS | 15/15 PASS | Taxonomic reliability vs sequencing depth |
| 005: Seismic Waves | Geological | PASS | 9/9 PASS | Source localization from noisy arrivals |
| 006: Signal Specificity | Biological | 12/12 PASS | 12/12 PASS | c-di-GMP signal vs noise in enzyme network |
| 007: RAWR Resampling | Statistics | 11/11 PASS | 11/11 PASS | Bayesian bootstrap vs naive bootstrap |
| 008: Anderson Localization | Mathematics | 8/8 PASS | 8/8 PASS | Lyapunov exponents in disordered media |
| 009: Almost-Mathieu Quasiperiodic | Mathematics | PASS | 8/8 PASS | Aubry-André metal-insulator transition |
| 010: Bistable Phenotypic Switching | Biological | PASS | 10/10 PASS | Fernandez 2020 PNAS bifurcation |
| 011: Multi-Signal QS Integration | Biological | PASS | 9/9 PASS | Srivastava 2011 dual-signal integration |
| 012: Spin Chain Transport | Mathematics | 18/18 PASS | 18/18 PASS | Kachkovskiy 2016 wavepacket MSD, transport exponent |
| 013: Resampling Convergence | Statistics | 10/10 PASS | 8/8 PASS | Lee & Liu 2024 bootstrap convergence |
| 014: Drift vs Selection | Biological | 7/7 PASS | 7/7 PASS | R. Anderson 2022 Wright-Fisher, Kimura fixation |
| 015: Uncertainty Bridge | Cross-domain | 8/8 PASS | 8/8 PASS | Sensor noise → Anderson ξ propagation |
| 016: Rare Biosphere | Biological | 11/11 PASS | 12/12 PASS | Sequencing depth determines rare taxa signal boundary |
| 017: Quasispecies Threshold | Evolutionary | 9/9 PASS | 6/6 PASS | Eigen's error threshold predicts mutation-driven information collapse |
| 018: Band Edge Structure | Mathematical | 8/8 PASS | 10/10 PASS | Transfer matrix reproduces tight-binding band-gap structure |
| 019: Jackknife Error Estimation | Inverse Problems & Spectral Reconstruction | 9/9 PASS | 9/9 PASS | Bazavov 2025 Phys Rev D 111, 094508 — jackknife variance, bias correction |
| 020: Freeze-Out Inverse Problem | Inverse Problems & Spectral Reconstruction | 8/8 PASS | 8/8 PASS | Bazavov 2016 Phys Rev D 93, 014512 — freeze-out temperature from hadron yields |
| 021: Spectral Function Reconstruction | Inverse Problems & Spectral Reconstruction | 8/8 PASS | 8/8 PASS | Bazavov 2025 arXiv 2501.12259 — spectral reconstruction from correlators |
| 022: ET₀ → Anderson Propagation | Cross-spring (FAO-56 + Anderson) | 7/7 PASS | 7/7 PASS | Humidity-dominated ET₀ error → localization length CV |
| 023: No-Till vs Tilled Sampling | Cross-spring (microbiome + soil) | 7/7 PASS | 7/7 PASS | Saturation depth by soil management regime |
| 024: Aggregate Stability Noise | Cross-spring (soil physics) | 8/8 PASS | 8/8 PASS | WSA measurement precision vs Anderson regime discrimination |
| 025: f32 vs f64 Precision Drift | WDM MD | 7/7 PASS | 7/7 PASS | Green-Kubo f32 accumulation bias |
| 026: System-size Convergence | WDM MD | 7/7 PASS | 7/7 PASS | Transport coefficient finite-size extrapolation |
| 027: GPU Vendor Parity | WDM MD | 7/7 PASS | 7/7 PASS | Cross-vendor transport coefficient agreement |
| 028: NPU Anderson Regime | Hardware (NPU) | 7/7 PASS | 9/9 PASS | Anderson regime classification on AKD1000 via int8 DMA |
| 029: Real GHCND ET₀ | Cross-spring (NOAA) | — | 6/6 PASS | Hargreaves vs Penman-Monteith on real/synthetic weather via NestGate |
| 030: Real NCBI 16S | Biological (NCBI) | — | 9/9 PASS | Rare biosphere detection on real/synthetic NCBI 16S metagenomes |
| 031: NUCLEUS Stack | Infrastructure | — | 28/28 PASS | Full NUCLEUS primal validation: Tower + Node + Squirrel + Nest |
| 032: IRIS Seismic | Geological (IRIS) | — | 12/12 PASS | IRIS FDSN station geometry + travel times via NestGate |
| 033: Tissue Anderson | Immunological (Paper 12) | — | 29/29 PASS | Cytokine Anderson lattice + geometry-aware drug scoring |
| 035: ET₀ Methods | Agricultural (FAO-56) | 15/15 PASS | 19/19 PASS | 5-method ET₀ cross-validation: PM, Hargreaves, Makkink, Turc, Hamon |

**Phase 1 total: 427/427 PASS across 38 validation binaries** (340 core + 55 NUCLEUS + 32 LTEE via `--features biomeos`). All public APIs return `Result` — zero panicking entry points.

## Library Modules

| Module | Purpose | GPU Tier |
|--------|---------|----------|
| `stats::agreement` | RMSE, MAE, MBE, NSE, R², IA, hit rate (R²/NSE deduplicated via shared `coefficient_of_efficiency`) | **GPU dispatched** (rmse, mbe via FusedMapReduceF64/SumReduceF64) + CPU delegated |
| `stats::metrics` | mean, std_dev, sample_std_dev, percentile | **GPU dispatched** (mean via SumReduceF64, std_dev via VarianceReduceF64) + CPU delegated |
| `stats::correlation` | Pearson/Spearman correlation, covariance | **GPU dispatched** (pearson_r via CorrelationF64, covariance via CovarianceF64) + CPU delegated |
| `stats::distributions` | norm_cdf, norm_ppf, χ² | 3 CPU delegated |
| `stats::regression` | Linear, quadratic, exponential, logarithmic, power-law, hyperbolic fits; `fit_all` (6 models) | 6 CPU delegated |
| `stats::model_selection` | AIC, BIC, RSS, `compare_models` — information-criterion model selection | N/A |
| `decompose` | Bias-variance decomposition, noise floor | CPU-only (scalar) |
| `fao56` | FAO-56 Penman-Monteith equation chain | **Absorbed** (barracuda `Op::Fao56Et0`) + **GPU batch** (BatchedElementwiseF64) |
| `prng` | Xorshift64 PRNG, Box-Muller normal | B (DefaultRng aligned) |
| `rarefaction` | Multinomial sampling, Shannon/Simpson diversity, Bray-Curtis, evenness, analytical rarefaction | C (WGSL production ready) |
| `seismic` | Haversine, travel time, grid-search inversion | **GPU-ready** (V31 dispatch) |
| `gillespie` | Gillespie SSA for stochastic chemical kinetics | **GPU dispatched** (batch via GillespieGpu) |
| `bootstrap` | Bootstrap (mean/median/std) + RAWR confidence intervals | A Lean (`barracuda::stats`) |
| `anderson` | Anderson localization, Lyapunov exponents, analytical ξ(W,E), 2D/3D eigenvalues, disorder sweep, spectral diagnostics, empirical spectral density, Marchenko-Pastur bounds, transition detection | A Lean (`barracuda::spectral` + `special` + `stats::spectral_density` + `ops::peak_detect_f64`) |
| `almost_mathieu` | Almost-Mathieu quasiperiodic localization, level spacing | A Lean (`barracuda::spectral`) |
| `linalg` | Tridiag eigensolver (implicit QL with Wilkinson shifts) — shared by transport + band_structure | B (adapt) |
| `transport` | Wavepacket MSD, transport exponent (re-exports `linalg::tridiag_eigh` for compat) | B (adapt) |
| `error` | Typed input validation errors (`InputError`: `LengthMismatch`, `InsufficientData`, `OutOfRange`) | N/A |
| `drift` | Wright-Fisher fixation, Kimura fixation probability, neutral diversity trajectory | **CPU delegated** (kimura_fixation_prob S70+) + **GPU batch** (WrightFisherGpu) |
| `cast` | Centralized numeric casts with documented safety | N/A |
| `kinetics` | Hill + Monod kinetics (shared bistable + multi-signal) | A Lean (barracuda::stats::hill, monod) |
| `validate` | Modular validation harness: `sink` (trait + stdout/null/NDJSON) + `harness` (check methods + summary) — hotSpring pattern | N/A |
| `rare_biosphere` | Chao1, detection power/threshold, abundance-occupancy, singleton fraction | **GPU-ready** (V31 dispatch) |
| `quasispecies` | Eigen error threshold, master frequency, Wright-Fisher mutation simulation | **GPU-ready** (V31 dispatch) |
| `band_structure` | Transfer matrix, band edge detection, count bands, periodic Hamiltonian | **GPU-ready** (V31 dispatch) |
| `jackknife` | Jackknife variance, bias correction, leave-one-out resampling | **CPU delegated** (jackknife_mean_variance S70+) |
| `freeze_out` | Freeze-out temperature inversion, hadron yield fitting | **GPU-ready** (V31 dispatch) |
| `spectral_recon` | Spectral function reconstruction from Euclidean correlators | GPU delegated (tikhonov_solve) |
| `biomeos` | biomeOS Neural API client: JSON-RPC 2.0, capability routing, UDS server, NestGate storage (behind `biomeos` feature) | N/A |
| `dispatch` | JSON-RPC method dispatch: `measurement.*` semantic routing to library functions (behind `biomeos` feature) | N/A |
| `provenance` | Provenance Trio lifecycle: session create, dehydrate, attribute via capability calls (behind `biomeos` feature) | N/A |
| `nestgate` | NestGate data pipeline: NCBI/NOAA providers, provenance key schemas, cache-through (behind `biomeos` feature) | N/A |
| `esn` | Echo State Network regime classification: `EsnClassifier` (barracuda-gpu), rule-based `classify_by_spacing_ratio`, `spectral_features` | **GPU dispatched** (barracuda-gpu ESN) + CPU rule-based |
| `lanczos` | Sparse eigensolver for 2D/3D Anderson: `sparse_eigenvalues`, `eigenvalues_from_csr` (barracuda-gpu only) | **GPU dispatched** (barracuda spectral Lanczos) |
| `npu` | NPU integration for Akida neuromorphic inference (behind `npu` feature) | NPU (AKD1000) |
| `niche` | Self-knowledge module: capabilities, dependencies, cost estimates, feature gates, consumed capabilities (airSpring pattern) | N/A |
| `primal_names` | Centralized primal name constants for socket paths, discovery, env checks (wetSpring V119 pattern); zero hardcoded strings in production | N/A |
| `groundspring-forge` | Hardware discovery, cross-substrate dispatch, `PCIe` topology, multi-stage pipeline, NUCLEUS atomics, remote NUCLEUS discovery (26 workloads, 5+ substrates, 120 tests) | metalForge crate |

## Quick Start

### Rust Phase 1

```bash
cargo test --workspace                         # 1,123 tests, all PASS
cargo test --workspace --all-features          # all feature paths tested
cargo test --workspace --features biomeos      # NUCLEUS client active
cargo test --workspace --features barracuda-gpu # GPU dispatch active
cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic -W clippy::nursery
cargo fmt --check                              # clean

# Barracuda-delegated mode (validates cross-spring math)
cargo test --workspace --features barracuda
cargo test --workspace --features barracuda-gpu

# NPU mode (BrainChip AKD1000)
cargo test --workspace --features npu          # npu module + Exp 028

# metalForge live hardware binaries
cargo run --bin validate_metalforge_inventory
cargo run --bin validate_metalforge_gpu
cargo run --bin validate_metalforge_cross_substrate
cargo run --bin validate_metalforge_titan_v

# Run all validation binaries at once (meta-binary)
cargo run --bin validate_all

# Individual validation binaries (hotSpring pattern: exit 0 = pass, exit 1 = fail)
cargo run --bin validate_decompose
cargo run --bin validate_rarefaction
cargo run --bin validate_seismic
cargo run --bin validate_weather
cargo run --bin validate_fao56
cargo run --bin validate_signal_specificity
cargo run --bin validate_rawr
cargo run --bin validate_anderson
cargo run --bin validate_quasiperiodic
cargo run --bin validate_bistable
cargo run --bin validate_multisignal
cargo run --bin validate_transport
cargo run --bin validate_resampling_conv
cargo run --bin validate_drift
cargo run --bin validate_uncertainty_bridge
cargo run --bin validate_rare_biosphere
cargo run --bin validate_quasispecies
cargo run --bin validate_band_edge
cargo run --bin validate_jackknife
cargo run --bin validate_freeze_out
cargo run --bin validate_spectral_recon
cargo run --bin validate_npu_anderson
cargo run --bin validate_et0_anderson
cargo run --bin validate_notill_sampling
cargo run --bin validate_aggregate_stability
cargo run --bin validate_precision_drift
cargo run --bin validate_size_convergence
cargo run --bin validate_vendor_parity
cargo run --bin validate_tissue_anderson
cargo run --bin validate_et0_methods

# NUCLEUS / biomeOS validation (requires biomeos feature, NUCLEUS optional)
cargo run --features biomeos --bin validate_real_ghcnd_et0
cargo run --features biomeos --bin validate_real_ncbi_16s
cargo run --features biomeos --bin validate_nucleus_stack
cargo run --features biomeos --bin validate_iris_seismic
```

### Python Phase 0

```bash
pip install -e ".[dev]"
python3 -m pytest tests/ -v       # 29 experiments (287 tests)
ruff check control/ tests/        # zero errors
mypy control/ tests/              # zero errors
```

### Test Coverage

```bash
cargo llvm-cov --workspace --lib    # ≥92% library line coverage (target 90%)
```

## Performance: Rust vs Python

Median of 3 trials across all 28 experiments (Feb 27, 2026). Generate full data: `python3 scripts/bench_rust_vs_python.py`.

| Experiment | Python (s) | Rust (s) | Speedup |
|---|---|---|---|
| Exp 001: Sensor Noise | 0.38 | 0.07 | **5.3×** |
| Exp 002: Observation Gap | 0.27 | 0.08 | **3.6×** |
| Exp 003: Error Propagation | 0.34 | 0.08 | **4.4×** |
| Exp 004: Sequencing Noise | 0.14 | 0.09 | **1.5×** |
| Exp 005: Seismic Inversion | 7.42 | 0.14 | **53.5×** |
| Exp 006: Signal Specificity | 26.51 | 0.86 | **31.0×** |
| Exp 007: RAWR Resampling | 4.54 | 0.64 | **7.1×** |
| Exp 008: Anderson Localization | 21.96 | 0.77 | **28.6×** |
| Exp 009: Quasiperiodic | 0.65 | 11.32 * | **0.1×** |
| Exp 010: Bistable Switching | 3.26 | 0.18 | **18.1×** |
| Exp 011: Multi-Signal QS | 4.25 | 0.10 | **44.7×** |
| Exp 012: Spin Chain Transport | 0.92 | 0.31 | **3.0×** |
| Exp 013: Resampling Convergence | 1.36 | 0.13 | **10.4×** |
| Exp 014: Drift vs Selection | 0.42 | 1.14 | **0.4×** |
| Exp 015: Uncertainty Bridge | 1.32 | 0.12 | **11.1×** |
| Exp 016: Rare Biosphere | 0.38 | 0.20 | **1.9×** |
| Exp 017: Quasispecies Threshold | 0.12 | 0.09 | **1.3×** |
| Exp 018: Band Edge Structure | 0.23 | 0.11 | **2.1×** |
| Exp 019: Jackknife Estimation | 0.12 | 0.07 | **1.7×** |
| Exp 020: Freeze-Out Inverse | 0.36 | 0.07 | **5.1×** |
| Exp 021: Spectral Recon | 0.12 | 0.07 | **1.7×** |
| Exp 022: ET₀ Anderson | 0.87 | 0.10 | **8.6×** |
| Exp 023: No-Till Sampling | 0.11 | 0.09 | **1.3×** |
| Exp 024: Aggregate Stability | 0.14 | 0.09 | **1.6×** |
| Exp 025: Precision Drift | 27.93 | 3.18 | **8.8×** |
| Exp 026: Size Convergence | 0.12 | 0.07 | **1.6×** |
| Exp 027: Vendor Parity | 0.14 | 0.12 | **1.1×** |
| Exp 028: NPU Anderson | 0.12 | 0.08 | **1.5×** |
| **Total** | **104.49** | **20.35** | **5.1×** |
| **Total (excl. LAPACK-bound)** | **103.84** | **9.04** | **11.5×** |

\* Exp 009/014: Rust custom QR/Wright-Fisher vs NumPy LAPACK/SciPy. Barracuda-gpu
(Sturm tridiag from hotSpring S26) closes the gap: **47.7× speedup** for Exp 009.

**Mathematical parity**: 29/29 PROVEN — both languages validate against the
same shared benchmark JSONs. Generate report: `python3 scripts/parity_report.py`.

Run benchmarks: `python3 scripts/bench_rust_vs_python.py`
Run parity report: `python3 scripts/parity_report.py`

## BarraCUDA Delegation Performance

### Validation Binary Benchmark (29 binaries, release mode, March 8 2026)

| Mode | Wall time (s) | Δ vs local |
|------|--------------|------------|
| Local (no features) | 12.5 | baseline |
| BarraCUDA CPU | 18.4 | +47% (dispatch overhead on small workloads) |
| **BarraCUDA GPU** | **9.9** | **−21% (1.27× faster)** |

### Workspace Test Benchmark (1,123 tests, release mode)

| Mode | Wall time (s) | Δ vs local |
|------|--------------|------------|
| Local (no features) | 50.3 | baseline |
| BarraCUDA CPU | 29.0 | **−42% (1.73× faster)** |

110 active delegations (67 CPU + 43 GPU). Cross-spring evolution powers
this: hotSpring precision shaders (DF64 core, Sturm tridiag — **47.7× for Exp 009**),
wetSpring bio shaders (Gillespie SSA, diversity fusion, Bray-Curtis),
neuralSpring stats (chi-squared, KL divergence, matrix correlation),
airSpring hydrology (seasonal pipeline, Hargreaves ET₀), and groundSpring
spectral shaders (Anderson Lyapunov, uncertainty propagation).
`PrecisionRoutingAdvice` + runtime f64 smoke test (V97) guards all 21 GPU dispatch paths
via `get_device_f64_safe()` — hardware-aware routing with empirical f64 reduction verification.

### Cross-Spring Shader Evolution

Every spring contributes shaders that benefit the entire ecosystem through barraCuda:

```
hotSpring (precision)  ─────┐
  df64_core, Sturm tridiag, │    All absorbed into barraCuda v0.4.0
  stress_virial, CG kernels ├──► 784 WGSL shaders, f64-canonical
                             │    with f16/f32/f64/Df64 per hardware
wetSpring (bio)  ────────────┤
  smith_waterman, gillespie, │    toadStool S158+ routes hardware
  fused_map_reduce, HMM      │    coralReef compiles to native GPU binary
                             │
neuralSpring (ML)  ──────────┤    groundSpring consumes 110 ops:
  chi_squared, KL_divergence,│    67 CPU delegated, 43 GPU dispatched
  matrix_correlation, ESN    │
                             │
airSpring (hydrology)  ──────┤
  hargreaves, seasonal_pipe, │
  moving_window, Brent root  │
                             │
groundSpring (spectral)  ────┘
  anderson_lyapunov, welford,
  chi_squared → ALL springs
  f64 bug → PrecisionRoutingAdvice
```

## Evolution Path

```
Phase 0 (Python)  →  Phase 1 (Rust)  →  Phase 2 (GPU)  →  Phase 3 (Hardware)  →  Phase 4 (NUCLEUS)
  NumPy/SciPy         Pure safe Rust     BarraCUDA/ToadStool   metalForge dispatch    biomeOS Neural API
  ✓ Complete          ✓ 427/427 PASS     ◐ 110 active          30 workloads           Tower+Node+Squirrel
  11.5× slower        38/38 experiments    (67+43)              24 GPU + 2 NPU + 2 CPU-only         NestGate data pipes
                      1,123 workspace tests                      PCIe topology          NUCLEUS atomics
                                                                Pipeline dispatch      Sovereign degradation

  Write locally    →  Hand off          →  Lean on upstream   →  Cross-substrate     →  Primal orchestration
  (metalForge)       (wateringHole/)       (barracuda ops)       (metalForge forge)    (biomeOS graphs)
```

**Lean progress**: 110 functions delegate to barracuda with graceful sovereign fallback.
67 CPU delegated via `#[cfg(feature = "barracuda")]`, 43 GPU dispatched via
`#[cfg(feature = "barracuda-gpu")]`. V97: runtime f64 reduction smoke test
ensures GPU correctness — detects naga/SPIR-V zeros bug, graceful CPU fallback.
All local shaders absorbed upstream; only 2 unique `anderson_lyapunov*.wgsl`
reference shaders remain in metalForge. 13-tier tolerance architecture, all gates green.

**NUCLEUS progress**: biomeOS Neural API integration via `#[cfg(feature = "biomeos")]`.
Tower (BearDog) health + beacon, Node (ToadStool) compute capabilities, Squirrel AI
health — all validated live. NestGate data pipelines (NCBI, NOAA GHCND, IRIS FDSN)
wired with sovereign fallback to synthetic data.

See `specs/BARRACUDA_EVOLUTION.md` for GPU promotion mapping.
See `specs/PRIMAL_INTERACTION_EVOLUTION.md` for NUCLEUS evolution.
See `metalForge/` for absorption-ready shaders and the manifest.

## How groundSpring Relates to Other Springs

| Spring | What It Validates | What groundSpring Adds |
|--------|-------------------|------------------------|
| hotSpring | Clean nuclear math (f64, GPU) | How AME2020 mass uncertainties propagate to model predictions |
| airSpring | FAO-56 ET₀, soil calibration | The REAL sensor noise — quantifying factory vs field calibration |
| wetSpring | Microbiome taxonomy, PFAS detection | Sequencing error rates, mass spec noise floors |
| neuralSpring (future) | ML surrogates, transfer learning | groundSpring provides labeled dirty data; NPU dispatch via metalForge |
| biomeOS / NUCLEUS | Primal orchestration, data acquisition | groundSpring validates Tower+Node+Squirrel+Nest through Neural API |

## Directory Structure

```
groundSpring/
├── control/                         # Phase 0 Python experiments
│   ├── common.py                    # Shared statistical primitives
│   ├── sensor_noise/                # Exp 001: bias-variance decomposition
│   ├── observation_gap/             # Exp 002: model vs station
│   ├── error_propagation/           # Exp 003: Monte Carlo through FAO-56
│   ├── sequencing_noise/            # Exp 004: taxonomic noise floor
│   ├── seismic/                     # Exp 005: wave propagation + source inversion
│   ├── signal_specificity/          # Exp 006: c-di-GMP Gillespie SSA
│   ├── rawr_resampling/             # Exp 007: RAWR vs bootstrap
│   ├── anderson_localization/       # Exp 008: Anderson localization Lyapunov
│   ├── quasiperiodic/               # Exp 009: Almost-Mathieu Quasiperiodic
│   ├── bistable_switching/          # Exp 010: Bistable phenotypic switching
│   ├── multisignal_qs/             # Exp 011: Multi-signal QS integration
│   ├── spin_transport/             # Exp 012: Spin chain transport (Kachkovskiy 2016)
│   ├── resampling_convergence/     # Exp 013: Resampling convergence (Lee & Liu 2024)
│   ├── drift_selection/            # Exp 014: Drift vs selection (R. Anderson 2022)
│   ├── uncertainty_bridge/         # Exp 015: Sensor noise → Anderson ξ uncertainty
│   ├── rare_biosphere/            # Exp 016: Rare biosphere signal detection
│   ├── quasispecies_threshold/    # Exp 017: Eco-evolutionary noise threshold
│   ├── band_edge/                 # Exp 018: Band edge structure
│   ├── jackknife_estimation/      # Exp 019: Jackknife error estimation (Bazavov 2025)
│   ├── freeze_out_inverse/        # Exp 020: Freeze-out inverse problem (Bazavov 2016)
│   ├── spectral_recon/            # Exp 021: Spectral function reconstruction (Bazavov 2025)
│   ├── et0_anderson_propagation/   # Exp 022: ET₀ → Anderson uncertainty
│   ├── notill_sampling/            # Exp 023: No-till vs tilled 16S sampling
│   ├── aggregate_stability/        # Exp 024: Aggregate stability noise
│   ├── precision_drift/            # Exp 025: f32 vs f64 precision drift
│   ├── size_convergence/           # Exp 026: System-size convergence
│   ├── vendor_parity/              # Exp 027: GPU vendor parity
│   ├── npu_anderson/               # Exp 028: NPU Anderson regime classification
│   └── et0_methods/                # Exp 035: Multi-method ET₀ cross-validation
├── crates/
│   ├── groundspring/               # Phase 1 Rust library (41 modules incl. rawr, esn, lanczos, tissue_anderson, biomeos, nestgate, npu, primal_names)
│   └── groundspring-validate/      # 38 validation binaries (hotSpring pattern)
├── metalForge/                     # Write → Absorb → Lean artifacts
│   ├── forge/                      # groundspring-forge crate: hardware discovery, dispatch, topology, pipeline, atomics, remote
│   ├── npu/akida/                  # AKD1000 NPU integration, HARDWARE.md
│   ├── ABSORPTION_MANIFEST.md      # Module-by-module absorption inventory
│   └── shaders/                    # Production WGSL shaders for ToadStool absorption
├── graphs/                         # biomeOS pipeline graphs (deploy, Tower, Node, cross-substrate, validation)
├── niches/                         # BYOB niche YAML definitions (groundspring-measurement)
├── .github/workflows/ci.yml        # GitHub Actions CI
├── wateringHole/                   # Handoff directory (V127 current)
├── specs/
│   ├── BARRACUDA_EVOLUTION.md      # Module → GPU promotion mapping + PRNG roadmap
│   ├── BARRACUDA_REQUIREMENTS.md   # GPU kernel gap analysis
│   ├── CROSS_SPRING_EVOLUTION.md   # Cross-spring shader provenance
│   ├── PRIMAL_INTERACTION_EVOLUTION.md # NUCLEUS Neural API evolution (V0–V6)
│   ├── LAN_DEPLOYMENT_READINESS.md # LAN HPC readiness assessment
│   └── PAPER_REVIEW_QUEUE.md       # 30 papers, three-tier control matrix, open data audit
├── whitePaper/                     # Study, methodology, baseCamp, experiments
│   ├── baseCamp/                   # Per-faculty research briefings (7 faculty)
│   ├── experiments/                # Per-experiment summaries (001-035)
├── tests/                          # Python test suite (29 experiments + parity)
├── Cargo.toml                      # Rust workspace (barracuda feature gate)
├── CONTRIBUTING.md
├── CHANGELOG.md
└── LICENSE                         # AGPL-3.0-or-later
```

## Hardware Gate

Same as all ecoPrimals springs:

| Component | Specification |
|-----------|--------------|
| CPU | Intel i9-12900K (16C/24T, 5.2 GHz) |
| RAM | 64 GB DDR5-4800 |
| GPU | NVIDIA GeForce RTX 4070 (12 GB VRAM) |
| GPU | NVIDIA Titan V (12 GB HBM2) |
| NPU | BrainChip AKD1000 (80 NPs, 10 MB SRAM, PCIe 2.0 x1) |
| Storage | 1 TB NVMe SSD |
| OS | Pop!_OS 22.04 (Ubuntu-based) |

## License

AGPL-3.0-or-later — See [LICENSE](LICENSE)

---

## Version Timeline

| Version | Date | Milestone |
|---------|------|-----------|
| Init | Feb 16 | Repository initialized |
| Phase 1 | Feb 25 | 29 experiments PASS in Rust |
| V21 | Feb 26 | Complete barraCuda rewiring |
| V26 | Feb 27 | metalForge live hardware (RTX 4070, Titan V, AKD1000) |
| V39 | Feb 27 | NUCLEUS integration + NestGate data pipelines |
| V53 | Feb 28 | 57 active delegations + GPU grid adapters |
| V68 | Mar 2 | L-BFGS refinement, 4D Anderson |
| V76 | Mar 5 | Structural evolution, deep debt zero |
| V85 | Mar 6 | coralReef sovereign compilation (SM70/SM89) |
| V97 | Mar 7 | Three-tier parity proven (29/29 binaries × 3 tiers) |
| V110 | Mar 16 | Cross-ecosystem absorption, `#[expect(reason)]` migration |
| V115 | Mar 18 | Zero panicking APIs, ecoBin compliance, 930+ tests |
| V118 | Mar 19 | RPC expansion (16 caps), 110 delegations, 960+ tests |
| V119 | Mar 22 | Deep audit + cross-ecosystem absorption, 990+ tests, ≥92% coverage |
| V120 | Mar 23 | Deep audit execution: dispatch refactored, forbid(unsafe_code) on 50 binaries, DeviceCapabilities |
| V121 | Mar 23 | Deep debt + ecosystem absorption: tolerance centralization, provenance hardening, MSRV 1.87 |
| V122 | Mar 24 | Cast evolution + module extraction: lib.rs 607→182, validate/lib.rs 769→226, 20+ bare casts→named helpers |
| V123 | Mar 24 | Cross-ecosystem absorption + provenance, 1020+ tests, 41 modules |
| V124 | Apr 27 | guideStone L3, 16 caps synced, 6 deploy graphs, deep debt + tolerance hardening, 965+ tests |
| V125 | May 8 | guideStone L4, deep debt evolution, tolerance unification, 29 new test modules, composition crates, sporePrint notebooks |
| V131 | May 11 | Deep debt evolution + docs cleanup + handoffs, 1,101 tests |
| V134 | May 11 | LTEE B2 (Wiser 2013 power-law fitness) + B1 (Barrick 2009 neutral mutation), 1,119 tests |
| V135 | May 11 | Deep debt: modular validate harness, self-knowledge dispatch, dep decoupling, 1,125 tests |

Part of [ecoPrimals](https://github.com/syntheticChemistry) · [wateringHole](https://github.com/ecoPrimals/wateringHole) · AGPL-3.0-or-later
