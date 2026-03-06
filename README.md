# groundSpring — The Dirty Differences

**Date**: March 6, 2026 | **License**: AGPL-3.0-only
**Status**: V89 — 34 modules, 35 experiments, 500+ Rust workspace tests + 261 Python provenance tests, 395/395 validation checks (340 core + 55 NUCLEUS) + 187 metalForge checks, 93 active barracuda delegations (56 CPU + 37 GPU) — barraCuda v0.3.3+ (`ed82625`), toadStool S128b (`22d1a2c7`), coralReef Phase 9 (`b7f8ab4`). tarpc 0.35→0.37 aligned with barraCuda. `barracuda::ops` gate evolved from `barracuda` to `barracuda-gpu` (barraCuda ops now GPU-only). `domain-esn` feature wired. Rust 2024 `unsafe_code` model: workspace `deny` + lib.rs `#![forbid]`. All collapsible_if and unfulfilled lint expectations resolved. `cargo fmt` + `cargo clippy --workspace --all-features` zero warnings. All CPU tests pass; GPU tests blocked by barraCuda `Fp64Strategy` regression (`ed82625`)

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
| 013: Resampling Convergence | Statistics | 8/8 PASS | 8/8 PASS | Lee & Liu 2024 bootstrap convergence |
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
| 034: ET₀ Methods | Agricultural (FAO-56) | 15/15 PASS | 19/19 PASS | 5-method ET₀ cross-validation: PM, Hargreaves, Makkink, Turc, Hamon |

**Phase 1 total: 395/395 PASS across 34 validation binaries** (340 core + 55 NUCLEUS via `--features biomeos`).

## Library Modules

| Module | Purpose | GPU Tier |
|--------|---------|----------|
| `stats::agreement` | RMSE, MAE, MBE, NSE, R², IA, hit rate (R²/NSE deduplicated via shared `coefficient_of_efficiency`) | **GPU dispatched** (rmse, mbe via FusedMapReduceF64/SumReduceF64) + CPU delegated |
| `stats::metrics` | mean, std_dev, sample_std_dev, percentile | **GPU dispatched** (mean via SumReduceF64, std_dev via VarianceReduceF64) + CPU delegated |
| `stats::correlation` | Pearson/Spearman correlation, covariance | **GPU dispatched** (pearson_r via CorrelationF64) + CPU delegated |
| `stats::distributions` | norm_cdf, norm_ppf, χ² | 3 CPU delegated |
| `stats::regression` | Linear, quadratic, exponential, logarithmic fits | 4 CPU delegated |
| `decompose` | Bias-variance decomposition, noise floor | CPU-only (scalar) |
| `fao56` | FAO-56 Penman-Monteith equation chain | **Absorbed** (barracuda `Op::Fao56Et0`) + **GPU batch** (BatchedElementwiseF64) |
| `prng` | Xorshift64 PRNG, Box-Muller normal | B (align to xoshiro) |
| `rarefaction` | Multinomial sampling, Shannon/Simpson diversity, Bray-Curtis, evenness, analytical rarefaction | C (WGSL production ready) |
| `seismic` | Haversine, travel time, grid-search inversion | **GPU-ready** (V31 dispatch) |
| `gillespie` | Gillespie SSA for stochastic chemical kinetics | **GPU dispatched** (batch via GillespieGpu) |
| `bootstrap` | Bootstrap (mean/median/std) + RAWR confidence intervals | A Lean (`barracuda::stats`) |
| `anderson` | Anderson localization, Lyapunov exponents, analytical ξ(W,E), 2D/3D eigenvalues, disorder sweep | A Lean (`barracuda::spectral` + `special`) |
| `almost_mathieu` | Almost-Mathieu quasiperiodic localization, level spacing | A Lean (`barracuda::spectral`) |
| `linalg` | Tridiag eigensolver (implicit QL with Wilkinson shifts) — shared by transport + band_structure | B (adapt) |
| `transport` | Wavepacket MSD, transport exponent (re-exports `linalg::tridiag_eigh` for compat) | B (adapt) |
| `error` | Typed input validation errors (`InputError`: `LengthMismatch`, `InsufficientData`, `OutOfRange`) | N/A |
| `drift` | Wright-Fisher fixation, Kimura fixation probability, neutral diversity trajectory | **CPU delegated** (kimura_fixation_prob S70+) + **GPU batch** (WrightFisherGpu) |
| `cast` | Centralized numeric casts with documented safety | N/A |
| `kinetics` | Hill + Monod kinetics (shared bistable + multi-signal) | A Lean (barracuda::stats::hill, monod) |
| `validate` | Generic Write harness (hotSpring pattern) | N/A |
| `rare_biosphere` | Chao1, detection power/threshold, abundance-occupancy, singleton fraction | **GPU-ready** (V31 dispatch) |
| `quasispecies` | Eigen error threshold, master frequency, Wright-Fisher mutation simulation | **GPU-ready** (V31 dispatch) |
| `band_structure` | Transfer matrix, band edge detection, count bands, periodic Hamiltonian | **GPU-ready** (V31 dispatch) |
| `jackknife` | Jackknife variance, bias correction, leave-one-out resampling | **CPU delegated** (jackknife_mean_variance S70+) |
| `freeze_out` | Freeze-out temperature inversion, hadron yield fitting | **GPU-ready** (V31 dispatch) |
| `spectral_recon` | Spectral function reconstruction from Euclidean correlators | GPU delegated (tikhonov_solve) |
| `biomeos` | biomeOS Neural API client: JSON-RPC 2.0, capability routing, NestGate storage (behind `biomeos` feature) | N/A |
| `nestgate` | NestGate data pipeline: NCBI/NOAA providers, provenance key schemas, cache-through (behind `biomeos` feature) | N/A |
| `esn` | Echo State Network regime classification: `EsnClassifier` (barracuda-gpu), rule-based `classify_by_spacing_ratio`, `spectral_features` | **GPU dispatched** (barracuda-gpu ESN) + CPU rule-based |
| `lanczos` | Sparse eigensolver for 2D/3D Anderson: `sparse_eigenvalues`, `eigenvalues_from_csr` (barracuda-gpu only) | **GPU dispatched** (barracuda spectral Lanczos) |
| `npu` | NPU integration for Akida neuromorphic inference (behind `npu` feature) | NPU (AKD1000) |
| `groundspring-forge` | Hardware discovery, cross-substrate dispatch, `PCIe` topology, multi-stage pipeline, NUCLEUS atomics, remote NUCLEUS discovery (26 workloads, 5+ substrates, 120 tests) | metalForge crate |

## Quick Start

### Rust Phase 1

```bash
cargo test --workspace                         # 824 tests, all PASS
cargo test --workspace --features biomeos      # ~822 tests (NUCLEUS client active)
cargo test --workspace --features barracuda-gpu # 824 tests (GPU dispatch active)
cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic -W clippy::nursery
cargo fmt --check                              # clean

# Barracuda-delegated mode (validates cross-spring math)
cargo test --workspace --features barracuda
cargo test --workspace --features barracuda-gpu

# NPU mode (BrainChip AKD1000)
cargo test --workspace --features npu          # npu module + Exp 028

# metalForge live hardware binaries
cargo run --bin validate-metalforge-inventory
cargo run --bin validate-metalforge-gpu
cargo run --bin validate-metalforge-cross-substrate
cargo run --bin validate-metalforge-titan-v

# Validation binaries (hotSpring pattern: exit 0 = pass, exit 1 = fail)
cargo run --bin validate-decompose
cargo run --bin validate-rarefaction
cargo run --bin validate-seismic
cargo run --bin validate-weather
cargo run --bin validate-fao56
cargo run --bin validate-signal-specificity
cargo run --bin validate-rawr
cargo run --bin validate-anderson
cargo run --bin validate-quasiperiodic
cargo run --bin validate-bistable
cargo run --bin validate-multisignal
cargo run --bin validate-transport
cargo run --bin validate-resampling-conv
cargo run --bin validate-drift
cargo run --bin validate-uncertainty-bridge
cargo run --bin validate-rare-biosphere
cargo run --bin validate-quasispecies
cargo run --bin validate-band-edge
cargo run --bin validate-jackknife
cargo run --bin validate-freeze-out
cargo run --bin validate-spectral-recon
cargo run --bin validate-npu-anderson
cargo run --bin validate-et0-anderson
cargo run --bin validate-notill-sampling
cargo run --bin validate-aggregate-stability
cargo run --bin validate-precision-drift
cargo run --bin validate-size-convergence
cargo run --bin validate-vendor-parity
cargo run --bin validate-tissue-anderson

# NUCLEUS / biomeOS validation (requires biomeos feature, NUCLEUS optional)
cargo run --features biomeos --bin validate-real-ghcnd-et0
cargo run --features biomeos --bin validate-real-ncbi-16s
cargo run --features biomeos --bin validate-nucleus-stack
cargo run --features biomeos --bin validate-iris-seismic
```

### Python Phase 0

```bash
pip install -e ".[dev]"
python3 -m pytest tests/ -v       # 28 experiments
ruff check control/ tests/        # zero errors
mypy control/ tests/              # zero errors
```

### Test Coverage

```bash
cargo llvm-cov --workspace          # 97.25% library line coverage (target 90%)
```

## Performance: Rust vs Python

Median of 3 trials across all 28 experiments (Feb 27, 2026). See `data/bench_rust_vs_python.json` for full data.

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

**Mathematical parity**: 28/28 PROVEN — both languages validate against the
same shared benchmark JSONs. See `data/parity_report.json`.

Run benchmarks: `python3 scripts/bench_rust_vs_python.py`
Run parity report: `python3 scripts/parity_report.py`

## BarraCUDA Delegation Performance

| Mode | Total (ms) | Quasiperiodic (ms) | Delta |
|------|-----------|-------------------|-------|
| Local (no features) | 20,366 | 11,648 | baseline |
| Barracuda CPU | 21,512 | 12,734 | ~0% overhead |
| **Barracuda-GPU** | **9,236** | **244** | **−55% (2.2× faster)** |

Barracuda CPU delegation is free. Barracuda-GPU adds Sturm tridiag
eigenvalue solver (from hotSpring S26 spectral), GPU reduce ops
(FusedMapReduceF64, SumReduceF64, VarianceReduceF64, CorrelationF64),
and batch dispatch APIs (GillespieGpu, WrightFisherGpu, BatchedElementwiseF64),
giving **47.7× speedup** for Exp 009. Cross-spring evolution validated
by 76 active delegations (44 CPU + 32 GPU), including
ESN regime classification, Lanczos sparse eigensolver, 2D/3D/4D Anderson eigenvalues,
L-BFGS refinement, Wegner RG coarsening, and decomposed chi-squared analysis from
hotSpring/wetSpring lineage. 30 metalForge workloads route across 5 substrates
(24 GPU + 2 NPU + 2 CPU-only) with architecture-aware routing.

## Evolution Path

```
Phase 0 (Python)  →  Phase 1 (Rust)  →  Phase 2 (GPU)  →  Phase 3 (Hardware)  →  Phase 4 (NUCLEUS)
  NumPy/SciPy         Pure safe Rust     BarraCUDA/ToadStool   metalForge dispatch    biomeOS Neural API
  ✓ Complete          ✓ 376/376 PASS     ◐ 84 active           30 workloads           Tower+Node+Squirrel
  11.5× slower        33/33 experiments    (50+34)              24 GPU + 2 NPU + 2 CPU-only         NestGate data pipes
                      824 workspace tests                       PCIe topology          NUCLEUS atomics
                                                                Pipeline dispatch      Sovereign degradation

  Write locally    →  Hand off          →  Lean on upstream   →  Cross-substrate     →  Primal orchestration
  (metalForge)       (wateringHole/)       (barracuda ops)       (metalForge forge)    (biomeOS graphs)
```

**Lean progress**: 93 functions delegate to barracuda with graceful sovereign fallback.
56 CPU delegated via `#[cfg(feature = "barracuda")]`, 37 GPU dispatched via
`#[cfg(feature = "barracuda-gpu")]`. V82: Thornthwaite ET₀ + heat_index, fit_all
regression, esn/fao56 smart-refactored, deep debt audit clean.
V81: BootstrapMeanGpu GPU dispatch, freeze_out gate fix, coralReef cloned, 27/27 cross-spring checks.
V80: fused `correlation_full` GPU (5-accumulator single-pass), Welford single-pass
CPU stats, covariance GPU path. V78: modern rewiring — fused `mean_and_std_dev`,
3 new ET₀ delegations. V77: wgpu 28, DF64 precision tiers. V76: deep debt zero.
V73: 13-tier tolerance architecture. All gates green. All local shaders absorbed
upstream; only 2 unique `anderson_lyapunov*.wgsl` reference shaders remain in metalForge.

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
│   └── npu_anderson/               # Exp 028: NPU Anderson regime classification
├── crates/
│   ├── groundspring/               # Phase 1 Rust library (34 modules incl. esn, lanczos, tissue_anderson, biomeos, nestgate, npu)
│   └── groundspring-validate/      # 34 validation binaries (hotSpring pattern)
├── metalForge/                     # Write → Absorb → Lean artifacts
│   ├── forge/                      # groundspring-forge crate: hardware discovery, dispatch, topology, pipeline, atomics, remote
│   ├── npu/akida/                  # AKD1000 NPU integration, HARDWARE.md
│   ├── ABSORPTION_MANIFEST.md      # Module-by-module absorption inventory
│   └── shaders/                    # Production WGSL shaders for ToadStool absorption
├── graphs/                         # biomeOS pipeline graphs (Tower bootstrap, Node, cross-substrate)
├── .github/workflows/ci.yml        # GitHub Actions CI
├── wateringHole/                   # Handoff directory (V85 current)
├── specs/
│   ├── BARRACUDA_EVOLUTION.md      # Module → GPU promotion mapping + PRNG roadmap
│   ├── BARRACUDA_REQUIREMENTS.md   # GPU kernel gap analysis
│   ├── CROSS_SPRING_EVOLUTION.md   # Cross-spring shader provenance
│   ├── PRIMAL_INTERACTION_EVOLUTION.md # NUCLEUS Neural API evolution (V0–V6)
│   ├── LAN_DEPLOYMENT_READINESS.md # LAN HPC readiness assessment
│   └── PAPER_REVIEW_QUEUE.md       # 30 papers, three-tier control matrix, open data audit
├── whitePaper/                     # Study, methodology, baseCamp, experiments
│   ├── baseCamp/                   # Per-faculty research briefings (7 faculty)
│   ├── experiments/                # Per-experiment summaries (001-033)
├── tests/                          # Python test suite (28 experiments)
├── Cargo.toml                      # Rust workspace (barracuda feature gate)
├── CONTRIBUTING.md
├── CHANGELOG.md
└── LICENSE                         # AGPL-3.0-only
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

AGPL-3.0-only — See [LICENSE](LICENSE)

---

*Initialized: February 16, 2026 | Phase 1 complete: February 25, 2026 | Full-suite parity: February 26, 2026 | V21 complete barracuda rewiring: February 26, 2026 | V26 metalForge live hardware: February 27, 2026 | V30 biomeOS Neural API: February 27, 2026 | V35 Titan V / NAK adaptive GPU dispatch: February 27, 2026 | V39 NUCLEUS integration + NestGate data pipeline + metalForge remote discovery: February 27, 2026 | V53 complete rewiring + GPU grid adapters — 57 active: February 28, 2026 | V56 NUCLEUS live validation + ToadStool handoff: March 1, 2026 | V58 cross-spring evolution + deep-debt completion: March 1, 2026 | V63 brain architecture + capability-based discovery + Paper 12: March 2, 2026 | V68 complete rewiring — L-BFGS refinement, 4D Anderson: March 2, 2026 | V72 deep audit + debt evolution — zero clippy, BTreeMap determinism: March 3, 2026 | V73 tolerance architecture + epsilon guards + idiomatic evolution: March 4, 2026 | V74 deep debt + clippy pedantic CI + ToadStool/barraCuda full catch-up: March 4, 2026 | V76 structural evolution + deep debt zero + toadstool/barracuda absorption handoff: March 5, 2026 | V77 wgpu 28 migration + barraCuda v0.3.3 sync + DF64 precision tiers: March 5, 2026 | V78 modern rewiring + fused ops + ET₀ methods + cross-spring benchmark evolution: March 5, 2026 | V80 fused correlation_full GPU + Welford CPU stats + covariance GPU: March 5, 2026 | V81 BootstrapMeanGpu + freeze_out gate fix + coralReef sovereign compiler: March 5, 2026 | V84 GPU validation — RTX 4070 + Titan V dual-GPU probed, f64 shared memory issue found: March 6, 2026 | V85 coralReef sovereign compilation — CFG/RA fixes, f64 reduction compiles to native SM70/SM89: March 6, 2026*
