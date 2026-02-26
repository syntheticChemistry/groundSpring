# groundSpring — The Dirty Differences

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

**Phase 1 total: 144/144 PASS across 11 validation binaries.**

## Library Modules

| Module | Purpose | GPU Tier |
|--------|---------|----------|
| `stats` | RMSE, MBE, R², IA, hit rate, mean, percentile, Pearson/Spearman, std, covariance, norm_cdf/ppf, χ² | 15 CPU delegated, GPU pending adapter |
| `decompose` | Bias-variance decomposition, noise floor | CPU-only (scalar) |
| `fao56` | FAO-56 Penman-Monteith equation chain | **Absorbed** (barracuda `Op::Fao56Et0`) |
| `prng` | Xorshift64 PRNG, Box-Muller normal | B (align to xoshiro) |
| `rarefaction` | Multinomial sampling, Shannon diversity, evenness | C (WGSL production ready) |
| `seismic` | Haversine, travel time, grid-search inversion | B (adapt) |
| `gillespie` | Gillespie SSA for stochastic chemical kinetics | GPU-ready (`GillespieGpu`) |
| `bootstrap` | Bootstrap + RAWR confidence intervals | A Lean (`barracuda::stats`) |
| `anderson` | Anderson localization, Lyapunov exponents, analytical ξ(W,E) | A Lean (`barracuda::spectral` + `special`) |
| `cast` | Centralized numeric casts with documented safety | N/A |
| `validate` | Generic Write harness (hotSpring pattern) | N/A |

## Quick Start

### Rust Phase 1

```bash
cargo test --workspace          # 177 tests (131 unit + 9 validate-lib + 14 proptest + 8 integration + 1 doc)
cargo clippy --workspace        # zero warnings
cargo fmt --check               # clean

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
```

### Python Phase 0

```bash
pip install -e ".[dev]"
python3 -m pytest tests/ -v       # ~129 checks
ruff check control/ tests/        # zero errors
mypy control/ tests/              # zero errors
```

### Test Coverage

```bash
cargo llvm-cov --workspace          # 99.11% workspace line coverage
```

## Performance: Rust vs Python

Median of 3 trials, all 11 experiments (Feb 26, 2026):

| Experiment | Python (s) | Rust (s) | Speedup |
|---|---|---|---|
| Exp 001: Sensor Noise | 0.64 | 0.11 | **5.7×** |
| Exp 002: Observation Gap | 0.28 | 0.07 | **4.4×** |
| Exp 003: Error Propagation | 0.36 | 0.10 | **3.8×** |
| Exp 004: Sequencing Noise | 0.14 | 0.08 | **1.8×** |
| Exp 005: Seismic Inversion | 7.63 | 0.12 | **63.6×** |
| Exp 006: Signal Specificity | 26.78 | 0.88 | **30.5×** |
| Exp 007: RAWR Resampling | 4.64 | 0.63 | **7.3×** |
| Exp 008: Anderson Localization | 21.98 | 0.73 | **29.9×** |
| Exp 009: Quasiperiodic | 0.65 | 0.23 * | **2.8×** |
| Exp 010: Bistable Switching | 3.58 | 0.19 | **18.5×** |
| Exp 011: Multi-Signal QS | 4.30 | 0.09 | **46.2×** |
| **Total** | **70.98** | **3.23** | **22.0×** |

\* Exp 009 with barracuda-gpu (Sturm tridiag solver from hotSpring S26).
Without barracuda: 11.7s (custom QR). The Sturm solver is **50× faster**.

**Mathematical parity**: All 11 experiments proven — both languages validate
against the same shared benchmark JSONs. See `data/parity_report.json`.

Run benchmarks: `python3 scripts/bench_rust_vs_python.py`
Run parity report: `python3 scripts/parity_report.py`

## BarraCUDA Delegation Performance

| Mode | Total (ms) | Quasiperiodic (ms) | Delta |
|------|-----------|-------------------|-------|
| Local (no features) | 14,530 | 11,717 | baseline |
| Barracuda CPU | 14,282 | 11,355 | ~0% overhead |
| **Barracuda-GPU** | **3,274** | **234** | **−77% (4.4× faster)** |

Barracuda CPU delegation is free. Barracuda-GPU adds the Sturm tridiag
eigenvalue solver (from hotSpring S26 spectral), giving **50× speedup**
for Exp 009. Cross-spring evolution (hotSpring precision, wetSpring bio-stats,
airSpring metrics, neuralSpring dispatch) validated by 24 barracuda delegations.

## Evolution Path

```
Python baseline (Phase 0)  →  Rust validation (Phase 1)  →  GPU acceleration (Phase 2)
   NumPy/SciPy                    Pure safe Rust                BarraCUDA / ToadStool
     ✓ Complete                     ✓ 144/144 PASS               ◐ 24 delegated (19 CPU + 5 GPU), 2 WGSL ready
   23× slower than Rust             11/11 parity proven           barracuda-gpu: anderson, ODE, hamiltonian

     Write locally              →  Hand off to barracuda      →  Lean on upstream
     (metalForge shaders)          (wateringHole/handoffs/)       (rewire to barracuda ops)
```

**Lean progress**: 24 functions delegate to barracuda with graceful fallback —
`pearson_r`, `spearman_r`, `sample_std_dev`, `covariance`, `norm_cdf`, `norm_ppf`,
`chi2_statistic`, `bootstrap_mean`, `lyapunov_exponent`, `lyapunov_averaged`,
`analytical_localization_length`, `almost_mathieu_hamiltonian`, `bistable_derivative`,
`multisignal_derivative`, `rmse`, `mbe`, `r_squared`, `index_of_agreement`,
`hit_rate`, `shannon_diversity`, `mean`, `percentile`, `level_spacing_ratio`,
`almost_mathieu_eigenvalues`. 19 CPU delegated via `#[cfg(feature = "barracuda")]`,
5 GPU delegated via `#[cfg(feature = "barracuda-gpu")]`. FAO-56 equation chain
absorbed upstream. Two production WGSL shaders ready for ToadStool absorption.

See `specs/BARRACUDA_EVOLUTION.md` for the full GPU promotion mapping.
See `metalForge/` for absorption-ready shaders and the manifest.

## How groundSpring Relates to Other Springs

| Spring | What It Validates | What groundSpring Adds |
|--------|-------------------|------------------------|
| hotSpring | Clean nuclear math (f64, GPU) | How AME2020 mass uncertainties propagate to model predictions |
| airSpring | FAO-56 ET₀, soil calibration | The REAL sensor noise — quantifying factory vs field calibration |
| wetSpring | Microbiome taxonomy, PFAS detection | Sequencing error rates, mass spec noise floors |
| neuralSpring (future) | ML surrogates, transfer learning | groundSpring provides labeled dirty data neuralSpring learns from |

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
│   └── multisignal_qs/             # Exp 011: Multi-signal QS integration
├── crates/
│   ├── groundspring/                # Phase 1 Rust library (11 modules)
│   └── groundspring-validate/       # 11 validation binaries (hotSpring pattern)
├── metalForge/                      # Write → Absorb → Lean artifacts
│   ├── ABSORPTION_MANIFEST.md       # Module-by-module absorption inventory
│   └── shaders/                     # Production WGSL shaders for ToadStool absorption
├── .github/workflows/ci.yml         # GitHub Actions CI
├── wateringHole/                    # Handoff directory
├── specs/
│   ├── BARRACUDA_EVOLUTION.md       # Module → GPU promotion mapping + PRNG roadmap
│   ├── BARRACUDA_REQUIREMENTS.md    # GPU kernel gap analysis
│   └── PAPER_REVIEW_QUEUE.md        # 27 papers, three-tier control matrix, open data audit
├── whitePaper/                      # Study, methodology, baseCamp, experiments
│   ├── baseCamp/                    # Per-faculty research briefings (5 faculty)
│   ├── experiments/                 # Per-experiment summaries (001-011)
├── tests/                           # Python test suite (~129 checks)
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
| Storage | 1 TB NVMe SSD |
| OS | Pop!_OS 22.04 (Ubuntu-based) |

## License

AGPL-3.0-or-later — See [LICENSE](LICENSE)

---

*Initialized: February 16, 2026 | Phase 1 complete: February 25, 2026 | Full-suite parity: February 26, 2026*
