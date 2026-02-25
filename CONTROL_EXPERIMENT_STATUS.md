# groundSpring — Control Experiment Status

**Last updated**: February 25, 2026

## Experiment Register

| ID | Title | Domain | Phase 0 (Python) | Phase 1 (Rust) |
|----|-------|--------|:-----------------:|:--------------:|
| 001 | Sensor Noise Characterization | Agricultural (soil sensors) | 32/32 PASS | 36/36 PASS |
| 002 | Weather Model vs Observation | Meteorological | PASS (synthetic NOAA) | — |
| 003 | Error Propagation Through FAO-56 | Agricultural (ET₀) | PASS | — |
| 004 | Sequencing Depth & Taxonomic Noise | Biological (microbiome) | PASS | 15/15 PASS |
| 005 | Seismic Wave Propagation | Geological (seismology) | PASS | 9/9 PASS |

**Python Phase 0**: All 5 experiments passing
**Rust Phase 1**: 60/60 PASS across 3 validation binaries (decompose, rarefaction, seismic)
**pytest**: 31/31 PASS (unit tests, determinism tests, integration tests)

## Phase 0 — Python/NumPy/SciPy Baselines

### Exp 001: Sensor Noise Characterization (32/32 PASS)

**Question**: How much of the RMSE between factory-calibrated and field-measured soil moisture is systematic bias (correctable) vs random noise (irreducible)?

**Results**:
- CS616 in sand: NOISE-dominated (34.6% bias), noise floor 0.006 m³/m³
- CS616 in loamy sand: BIAS-dominated (59.2% bias), noise floor 0.021 m³/m³
- CS616 in sandy clay loam: NOISE-dominated (26.3% bias), noise floor 0.012 m³/m³
- EC5 across all soils: BIAS-dominated (62-77%), noise floor 0.004-0.020 m³/m³
- Site-specific calibration removes 50-80% of total sensor error

### Exp 002: Weather Model vs Observation (PASS)

**Question**: How does ERA5 reanalysis (Open-Meteo) differ from GHCND station observations (NOAA CDO)?

**Results** (synthetic NOAA mode — accuracy checks SKIPPED):
- Metrics computed for tmax, tmin, precipitation
- Seasonal decomposition shows winter has largest gaps
- **Pending**: Real NOAA CDO data for full accuracy validation

### Exp 003: Error Propagation Through FAO-56 (PASS)

**Question**: Given known sensor uncertainties, how does measurement noise propagate through the FAO-56 ET₀ chain?

**Results**:
- ET₀ uncertainty: 3.879 ± 0.142 mm/day (CV = 3.7%)
- 90% CI: [3.647, 4.118] mm/day
- Sensitivity ranking: humidity > radiation > temperature > wind
- Monte Carlo / analytical agreement: ratio ≈ 1.0

### Exp 004: Sequencing Depth & Taxonomic Noise (PASS)

**Question**: At what sequencing depth does 16S taxonomic assignment become stable?

**Results**:
- All phyla detected: ~100 reads
- Shannon convergence (5%): ~500 reads
- Genus saturation: ~5,000 reads
- Noise floor at 100k reads: ±0.004 Shannon, ±0.4 genera

### Exp 005: Seismic Wave Propagation (PASS)

**Question**: Can we locate an earthquake source from noisy P-wave arrivals?

**Results**:
- Clean inversion: 0.00 km error (perfect recovery)
- Noisy inversion (±0.5s): <30 km error
- Monte Carlo uncertainty: ±2 km horizontal (90th: ~4 km)
- Depth poorly constrained with surface-only stations

## Phase 1 — Rust Validation (hotSpring Pattern)

### validate-decompose (36/36 PASS)

Ports Exp 001 core algorithm to pure safe Rust.  Verifies:
- All 6 sensor-soil decompositions match analytically derived expected values
- Pythagorean identity (RMSE² = MBE² + σ²) holds to machine epsilon
- Noise floor reduction pythagorean holds

### validate-rarefaction (15/15 PASS)

Ports Exp 004 core algorithm to pure safe Rust.  Verifies:
- Shannon diversity analytical known values (uniform, single-species)
- Multinomial sampling determinism, total conservation, proportion accuracy
- Rarefaction convergence properties (monotonicity, high-depth completeness)

### validate-seismic (9/9 PASS)

Ports Exp 005 core algorithm to pure safe Rust.  Verifies:
- Haversine distance known values (zero, NY-London)
- Travel-time proportionality and known values
- Grid-search inversion recovers clean source exactly

## Test Infrastructure

| Suite | Tests | Type |
|-------|------:|------|
| `test_common.py` | 18 | Unit tests for shared statistical primitives |
| `test_determinism.py` | 7 | Rerun-identical verification for stochastic ops |
| `test_experiments.py` | 5 | Integration: each experiment returns exit code 0 |
| Rust `#[test]` | 26 | Unit tests for Rust library modules |
| **Total** | **56** | |

## Run Log

### Run 2 — February 25, 2026 (Phase 1 port)

```
Phase 0 (Python):
  Exp 001: Sensor Noise              32/32 PASS
  Exp 002: Observation Gap            PASS (synthetic SKIP)
  Exp 003: Error Propagation          PASS
  Exp 004: Sequencing Noise           PASS
  Exp 005: Seismic Inversion          PASS

Phase 1 (Rust):
  validate-decompose                 36/36 PASS
  validate-rarefaction               15/15 PASS
  validate-seismic                    9/9  PASS

pytest:
  test_common                        18/18 PASS
  test_determinism                    7/7  PASS
  test_experiments                    5/5  PASS
```

### Run 1 — February 16, 2026 (initial baselines)

```
Exp 001–005: 71/71 PASS (Python only)
```

## Evolution Roadmap

- **Phase 0**: Python/NumPy/SciPy baselines — **COMPLETE**
- **Phase 0+**: Real open data pipelines (NOAA CDO, IRIS waveforms) — pending API tokens
- **Phase 1a**: Rust CPU validation — **COMPLETE** (decompose, rarefaction, seismic)
- **Phase 1b**: Feature-gate BarraCUDA, Tier A rewire — see `specs/BARRACUDA_EVOLUTION.md`
- **Phase 2**: GPU acceleration, Tier B/C kernel development
- **neuralSpring bridge**: Export noise characterizations as labeled training data
