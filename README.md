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

**Phase 1 total: 88/88 PASS across 5 validation binaries.**

## Library Modules

| Module | Purpose | GPU Tier |
|--------|---------|----------|
| `stats` | RMSE, MBE, R², IA, hit rate, Pearson/Spearman, std | 3 CPU leaned, 6 GPU pending |
| `decompose` | Bias-variance decomposition, noise floor | CPU-only (scalar) |
| `fao56` | FAO-56 Penman-Monteith equation chain | **Absorbed** (barracuda `Op::Fao56Et0`) |
| `prng` | Xorshift64 PRNG, Box-Muller normal | B (align to xoshiro) |
| `rarefaction` | Multinomial sampling, Shannon diversity, evenness | C (WGSL production ready) |
| `seismic` | Haversine, travel time, grid-search inversion | B (adapt) |
| `validate` | Generic Write harness (hotSpring pattern) | N/A |

## Quick Start

### Rust Phase 1

```bash
cargo test --workspace          # 90 unit + 1 doc test
cargo clippy --workspace        # zero warnings
cargo fmt --all -- --check      # clean

# Validation binaries (hotSpring pattern: exit 0 = pass, exit 1 = fail)
cargo run --bin validate-decompose
cargo run --bin validate-rarefaction
cargo run --bin validate-seismic
cargo run --bin validate-weather
cargo run --bin validate-fao56
```

### Python Phase 0

```bash
pip install numpy scipy pandas requests
python3 control/sensor_noise/sensor_noise_decomposition.py
python3 control/sequencing_noise/sequencing_noise.py
python3 control/seismic/seismic_inversion.py
```

### Test Coverage

```bash
cargo llvm-cov --workspace --lib    # 99.7% library line coverage
```

## Evolution Path

```
Python baseline (Phase 0)  →  Rust validation (Phase 1)  →  GPU acceleration (Phase 2)
   NumPy/SciPy                    Pure safe Rust                BarraCUDA / ToadStool
     ✓ Complete                     ✓ 88/88 PASS                 ◐ 3 CPU leaned, 2 WGSL ready

     Write locally              →  Hand off to barracuda      →  Lean on upstream
     (metalForge shaders)          (wateringHole/handoffs/)       (rewire to barracuda ops)
```

**Lean progress**: `pearson_r`, `spearman_r`, `sample_std_dev` delegate to barracuda CPU.
FAO-56 equation chain absorbed upstream (`BatchedElementwiseF64::fao56_et0_batch`).
Two production WGSL shaders ready for ToadStool absorption.

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
│   └── seismic/                     # Exp 005: wave propagation + source inversion
├── crates/
│   ├── groundspring/                # Phase 1 Rust library (7 modules, 99.7% library line coverage)
│   └── groundspring-validate/       # 5 validation binaries (hotSpring pattern)
├── metalForge/                      # Write → Absorb → Lean artifacts
│   ├── ABSORPTION_MANIFEST.md       # Module-by-module absorption inventory
│   └── shaders/                     # Production WGSL shaders for ToadStool absorption
├── specs/
│   ├── BARRACUDA_EVOLUTION.md       # Module → GPU promotion mapping + PRNG roadmap
│   ├── BARRACUDA_REQUIREMENTS.md    # GPU kernel gap analysis
│   └── PAPER_REVIEW_QUEUE.md        # 19 papers, three-tier control matrix, open data audit
├── whitePaper/                      # Study results and methodology
│   ├── baseCamp/                    # Per-faculty research briefings (5 faculty)
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

*Initialized: February 16, 2026 | Phase 1 complete: February 25, 2026*
