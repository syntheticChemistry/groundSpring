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

| Experiment | Domain | Status | Key Question |
|------------|--------|--------|--------------|
| 001: Sensor Noise | Agricultural | Phase 0 | Bias vs variance in soil moisture sensors |
| 002: Observation Gap | Meteorological | Phase 0 | Reanalysis model vs station readings |
| 003: Error Propagation | Agricultural | Phase 0 | How sensor noise becomes ET0 uncertainty |
| 004: Sequencing Noise | Biological | Phase 0 | Taxonomic reliability vs sequencing depth |
| 005: Seismic Waves | Geological | Phase 0 | Source localization from noisy arrivals |

## Quick Start

```bash
# Install dependencies
pip install -r control/requirements.txt

# Run all Phase 0 baselines
bash scripts/run_all_baselines.sh
```

## How groundSpring Relates to Other Springs

| Spring | What It Validates | What groundSpring Adds |
|--------|-------------------|------------------------|
| hotSpring | Clean nuclear math (f64, GPU) | How AME2020 mass uncertainties propagate to model predictions |
| airSpring | FAO-56 ET0, soil calibration | The REAL sensor noise — quantifying factory vs field calibration |
| wetSpring | Microbiome taxonomy, PFAS detection | Sequencing error rates, mass spec noise floors |
| neuralSpring (future) | ML surrogates, transfer learning | groundSpring provides labeled dirty data neuralSpring learns from |

**Key distinction**: airSpring asks "what is ET0 for this field?" groundSpring asks "how confident are we in that number given what the sensors actually reported?" neuralSpring asks "can we learn a model that adapts this answer to a new field we've never seen?"

## Research Context

groundSpring draws on research from all existing spring PIs:

- **Dr. Younsuk Dong (airSpring, MSU BAE)** — Dong et al. (2020) is a groundSpring study: comparing factory calibrations to field reality across soil types. The RMSE/IA/MBE framework is groundSpring's statistical language.

- **Dr. A. Daniel Jones (wetSpring, MSU Biochem)** — Non-targeted PFAS screening is an inverse/detection problem in noisy mass spec data. His ML work for PFAS in Michigan drinking water is groundSpring methodology.

- **Dr. Michael Murillo (hotSpring, MSU Physics)** — Langevin thermostat = stochastic noise modeling. AME2020 uncertainty quantification is error propagation. SparsitySampler learns where noise matters most.

- **Smallwood & Cahill (wetSpring, Sandia)** — Pond crash forensics = anomaly detection in biological time series. When does normal variation become a crash signal?

### New Directions (No PI Yet)

- **Seismology / Geophysics** — Earthquake source inversion, ambient noise tomography
- **Astronomical Observation** — Stellar classification from noisy spectra, atmospheric correction
- **Remote Sensing** — Satellite calibration, the gap between what a sensor sees and ground truth
- **Instrument Characterization** — Cross-calibration across sensor types

## Cross-Spring Use Cases

- **New Mexico pistachios / California almonds**: airSpring has Michigan models. groundSpring quantifies how different NM/CA conditions are from Michigan. neuralSpring adapts. groundSpring says "here's how much to distrust the prediction outside Michigan."

- **Fruit blight / white fungus in bat caves**: wetSpring identifies microbes. airSpring models environment. groundSpring asks "how reliable is this signal? Is the blight signature real or a sequencing artifact?"

- **Soil microbiome health**: wetSpring (microbial ID) + airSpring (soil-plant-atmosphere) + groundSpring (measurement uncertainty, spatial heterogeneity, temporal variability). groundSpring says "this sample represents a 10m radius, not the whole field."

## Directory Structure

```
groundSpring/
├── control/                      # Phase 0 baselines
│   ├── sensor_noise/             # Exp 001: bias-variance decomposition
│   ├── observation_gap/          # Exp 002: model vs station
│   ├── error_propagation/        # Exp 003: Monte Carlo through FAO-56
│   ├── sequencing_noise/         # Exp 004: taxonomic noise floor
│   └── seismic/                  # Exp 005: wave propagation + source inversion
├── scripts/
│   ├── run_all_baselines.sh
│   └── download_iris.py          # IRIS seismic data (free, public)
├── whitePaper/
├── data/                         # Downloaded data (not committed)
├── CONTROL_EXPERIMENT_STATUS.md
├── README.md
└── LICENSE
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

AGPL-3.0-or-later

---

*Initialized: February 16, 2026*
