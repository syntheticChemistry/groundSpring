# groundSpring — The Dirty Differences

**Date**: February 27, 2026 | **License**: AGPL-3.0-or-later
**Status**: 28 experiments, 442 Rust tests (biomeos) / 410 (default) + 320 Python tests = 762 total, 288/288 validation checks (+ 49 metalForge), 32 active barracuda delegations (25 CPU + 7 GPU) + 9 pending ToadStool absorption, 19 metalForge workloads, 5 substrates (2 GPU + 1 NPU + 1 CPU + 1 GL), architecture-aware GPU routing (f64→Titan V, f32→RTX 4070), V35 Titan V / NAK adaptive GPU dispatch, four-mode CI

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
| 010: Bistable Phenotypic Switching | Biological | PASS | 9/9 PASS | Fernandez 2020 PNAS bifurcation |
| 011: Multi-Signal QS Integration | Biological | PASS | 8/8 PASS | Srivastava 2011 dual-signal integration |
| 012: Spin Chain Transport | Mathematics | 18/18 PASS | 18/18 PASS | Kachkovskiy 2016 wavepacket MSD, transport exponent |
| 013: Resampling Convergence | Statistics | 8/8 PASS | 8/8 PASS | Lee & Liu 2024 bootstrap convergence |
| 014: Drift vs Selection | Biological | 7/7 PASS | 7/7 PASS | R. Anderson 2022 Wright-Fisher, Kimura fixation |
| 015: Uncertainty Bridge | Cross-domain | 8/8 PASS | 8/8 PASS | Sensor noise → Anderson ξ propagation |
| 016: Rare Biosphere | Biological | 11/11 PASS | 10/10 PASS | Sequencing depth determines rare taxa signal boundary |
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

**Phase 1 total: 288/288 PASS across 28 validation binaries.**

## Library Modules

| Module | Purpose | GPU Tier |
|--------|---------|----------|
| `stats` | RMSE, MBE, R², IA, hit rate, mean, percentile, Pearson/Spearman, std, covariance, norm_cdf/ppf, χ² | 22 CPU delegated, GPU pending adapter |
| `decompose` | Bias-variance decomposition, noise floor | CPU-only (scalar) |
| `fao56` | FAO-56 Penman-Monteith equation chain | **Absorbed** (barracuda `Op::Fao56Et0`) |
| `prng` | Xorshift64 PRNG, Box-Muller normal | B (align to xoshiro) |
| `rarefaction` | Multinomial sampling, Shannon diversity, evenness | C (WGSL production ready) |
| `seismic` | Haversine, travel time, grid-search inversion | **GPU-ready** (V31 dispatch) |
| `gillespie` | Gillespie SSA for stochastic chemical kinetics | Pending (batch API needed, SSA serial) |
| `bootstrap` | Bootstrap + RAWR confidence intervals | A Lean (`barracuda::stats`) |
| `anderson` | Anderson localization, Lyapunov exponents, analytical ξ(W,E) | A Lean (`barracuda::spectral` + `special`) |
| `almost_mathieu` | Almost-Mathieu quasiperiodic localization, level spacing | A Lean (`barracuda::spectral`) |
| `transport` | Tridiag eigenvector solver (implicit QL), wavepacket MSD, transport exponent | B (adapt) |
| `drift` | Wright-Fisher fixation, Kimura fixation probability, neutral diversity trajectory | B (adapt) |
| `cast` | Centralized numeric casts with documented safety | N/A |
| `kinetics` | Hill-function kinetics (shared bistable + multi-signal) | A Lean (barracuda::stats::hill) |
| `validate` | Generic Write harness (hotSpring pattern) | N/A |
| `rare_biosphere` | Chao1, detection power/threshold, abundance-occupancy, singleton fraction | **GPU-ready** (V31 dispatch) |
| `quasispecies` | Eigen error threshold, master frequency, Wright-Fisher mutation simulation | **GPU-ready** (V31 dispatch) |
| `band_structure` | Transfer matrix, band edge detection, count bands, periodic Hamiltonian | **GPU-ready** (V31 dispatch) |
| `jackknife` | Jackknife variance, bias correction, leave-one-out resampling | CPU delegated |
| `freeze_out` | Freeze-out temperature inversion, hadron yield fitting | **GPU-ready** (V31 dispatch) |
| `spectral_recon` | Spectral function reconstruction from Euclidean correlators | GPU delegated (tikhonov_solve) |
| `npu` | NPU integration for Akida neuromorphic inference (behind `npu` feature) | NPU (AKD1000) |
| `groundspring-forge` | Hardware discovery and cross-substrate dispatch (19 workloads, 5 substrates) | metalForge crate |

## Quick Start

### Rust Phase 1

```bash
cargo test --workspace                         # 410 tests, all PASS
cargo test --workspace --features biomeos      # 442 tests (adds biomeOS client + integration)
cargo clippy --workspace -- -D warnings        # zero warnings × 4 modes
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
cargo llvm-cov --workspace          # 99.37% workspace line coverage
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

Barracuda CPU delegation is free. Barracuda-GPU adds the Sturm tridiag
eigenvalue solver (from hotSpring S26 spectral), giving **47.7× speedup**
for Exp 009. Cross-spring evolution (hotSpring precision, wetSpring bio-stats,
airSpring metrics, neuralSpring dispatch) validated by 32 active delegations
(25 CPU + 7 GPU), with 9 pending ToadStool absorption (3 CPU + 6 GPU).
19 metalForge workloads route across 5 substrates (GPU/NPU/CPU) with architecture-aware routing.

## Evolution Path

```
Python baseline (Phase 0)  →  Rust validation (Phase 1)  →  GPU acceleration (Phase 2)  →  Mixed hardware (Phase 3)
   NumPy/SciPy                    Pure safe Rust                BarraCUDA / ToadStool            metalForge dispatch
     ✓ Complete                     ✓ 288/288 PASS               ◐ 32 active (25 CPU +             19 workloads
   11.5× slower than Rust           28/28 parity proven            7 GPU) + 9 pending              5 substrates, arch-aware

     Write locally              →  Hand off to barracuda      →  Lean on upstream              →  Cross-substrate parity
     (metalForge shaders)          (wateringHole/handoffs/)       (rewire to barracuda ops)        (metalForge forge crate)
```

**Lean progress**: 32 functions delegate to barracuda with graceful sovereign fallback.
25 CPU delegated via `#[cfg(feature = "barracuda")]`,
7 GPU delegated via `#[cfg(feature = "barracuda-gpu")]`.
9 additional delegations pending ToadStool absorption (commented out with `TODO(toadstool)`):
3 CPU (`kimura_fixation`, `jackknife_mean_variance`, `fao56_et0`) +
6 GPU (`grid_fit_2d`, `grid_search_3d`, `band_edges_parallel`, `wright_fisher_simulate`,
`batched_multinomial_occupancy`, `batched_multinomial_tier_rate`).
Two production WGSL shaders ready for ToadStool absorption.

See `specs/BARRACUDA_EVOLUTION.md` for the full GPU promotion mapping.
See `metalForge/` for absorption-ready shaders and the manifest.

## How groundSpring Relates to Other Springs

| Spring | What It Validates | What groundSpring Adds |
|--------|-------------------|------------------------|
| hotSpring | Clean nuclear math (f64, GPU) | How AME2020 mass uncertainties propagate to model predictions |
| airSpring | FAO-56 ET₀, soil calibration | The REAL sensor noise — quantifying factory vs field calibration |
| wetSpring | Microbiome taxonomy, PFAS detection | Sequencing error rates, mass spec noise floors |
| neuralSpring (future) | ML surrogates, transfer learning | groundSpring provides labeled dirty data; NPU dispatch via metalForge |

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
│   └── npu_anderson/              # Exp 028: NPU Anderson regime classification
├── crates/
│   ├── groundspring/                # Phase 1 Rust library (26 modules incl. npu)
│   └── groundspring-validate/       # 28 validation binaries (hotSpring pattern)
├── metalForge/                      # Write → Absorb → Lean artifacts
│   ├── forge/                       # groundspring-forge crate: hardware discovery, dispatch
│   ├── npu/akida/                   # AKD1000 NPU integration, HARDWARE.md
│   ├── ABSORPTION_MANIFEST.md       # Module-by-module absorption inventory
│   └── shaders/                     # Production WGSL shaders for ToadStool absorption
├── .github/workflows/ci.yml         # GitHub Actions CI
├── wateringHole/                    # Handoff directory
├── specs/
│   ├── BARRACUDA_EVOLUTION.md       # Module → GPU promotion mapping + PRNG roadmap
│   ├── BARRACUDA_REQUIREMENTS.md    # GPU kernel gap analysis
│   └── PAPER_REVIEW_QUEUE.md        # 28 papers, three-tier control matrix, open data audit
├── whitePaper/                      # Study, methodology, baseCamp, experiments
│   ├── baseCamp/                    # Per-faculty research briefings (6 faculty)
│   ├── experiments/                 # Per-experiment summaries (001-028)
├── tests/                           # Python test suite (21 experiments)
├── Cargo.toml                       # Rust workspace (barracuda feature gate)
├── CONTRIBUTING.md
├── CHANGELOG.md
└── LICENSE                          # AGPL-3.0-or-later
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

*Initialized: February 16, 2026 | Phase 1 complete: February 25, 2026 | Full-suite parity: February 26, 2026 | V21 complete barracuda rewiring + dual-mode CI: February 26, 2026 | V22 experiment buildout (016-018): February 26, 2026 | V23 experiment buildout (019-021): February 26, 2026 | V26 metalForge live hardware: February 27, 2026 | V27 docs + handoff audit: February 27, 2026 | V30 biomeOS Neural API: February 27, 2026 | V31 GPU dispatch wiring + metalForge workloads: February 27, 2026 | V32 ToadStool S68+ catch-up + forward declaration cleanup: February 27, 2026 | V33 delegation count expansion (32 active, 25 CPU + 7 GPU): February 27, 2026 | V35 Titan V / NAK adaptive GPU dispatch + architecture-aware routing: February 27, 2026*
