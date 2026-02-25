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
| 002: Observation Gap | Meteorological | PASS/SKIP | — | Reanalysis model vs station readings |
| 003: Error Propagation | Agricultural | PASS | — | How sensor noise becomes ET₀ uncertainty |
| 004: Sequencing Noise | Biological | PASS | 15/15 PASS | Taxonomic reliability vs sequencing depth |
| 005: Seismic Waves | Geological | PASS | 9/9 PASS | Source localization from noisy arrivals |

## Quick Start

### Python Phase 0

```bash
pip install numpy scipy pandas requests
bash scripts/run_all_baselines.sh
```

### Rust Phase 1

```bash
cargo test --workspace
cargo clippy --workspace   # zero warnings required

# Validation binaries (hotSpring pattern: exit 0 = pass, exit 1 = fail)
cargo run --bin validate-decompose
cargo run --bin validate-rarefaction
cargo run --bin validate-seismic
```

### Full Suite (Python + Rust + pytest)

```bash
bash scripts/run_all_baselines.sh
```

## Evolution Path

```
Python baseline (Phase 0)  →  Rust validation (Phase 1)  →  GPU acceleration (Phase 2)
   NumPy/SciPy                    Pure safe Rust                BarraCUDA / ToadStool
   ✓ Complete                     ✓ Core algorithms             ◻ Feature-gated
```

See `specs/BARRACUDA_EVOLUTION.md` for the module-by-module GPU promotion mapping.

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
│   ├── groundspring/                # Phase 1 Rust library (zero deps, safe)
│   └── groundspring-validate/       # Validation binaries (hotSpring pattern)
├── tests/                           # pytest suite (unit, determinism, integration)
├── scripts/
│   ├── run_all_baselines.sh         # Full validation (Python + Rust + pytest)
│   └── download_iris.py             # IRIS seismic data (free, public)
├── specs/
│   ├── BARRACUDA_REQUIREMENTS.md    # GPU kernel requirements
│   ├── BARRACUDA_EVOLUTION.md       # Module → GPU promotion mapping
│   └── PAPER_REVIEW_QUEUE.md        # Future experiment candidates
├── whitePaper/
├── pyproject.toml                   # Python tooling (ruff, mypy, pytest)
├── Cargo.toml                       # Rust workspace
├── CONTRIBUTING.md
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

*Initialized: February 16, 2026 | Phase 1 (Rust): February 25, 2026*
