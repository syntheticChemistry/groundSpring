# groundSpring — Control Experiment Status

**Last updated**: February 16, 2026

## Experiment Register

| ID | Title | Domain | Data Source | Status | Tests |
|----|-------|--------|-------------|--------|-------|
| 001 | Sensor Noise Characterization | Agricultural (soil sensors) | Dong et al. (2020) via airSpring | **PASS** | 32/32 |
| 002 | Weather Model vs Observation | Meteorological | Open-Meteo + NOAA CDO (synthetic) | **PASS** | 5/5 |
| 003 | Error Propagation Through FAO-56 | Agricultural (ET0) | Monte Carlo simulation | **PASS** | 8/8 |
| 004 | Sequencing Depth & Taxonomic Noise | Biological (microbiome) | Simulated rarefaction | **PASS** | 16/16 |
| 005 | Seismic Wave Propagation | Geological (seismology) | Synthetic NMSZ scenario | **PASS** | 10/10 |

**Grand Total: 71/71 PASS (0 FAIL)**

## Phase 0 — Python/NumPy/SciPy Baselines

### Exp 001: Sensor Noise Characterization (32/32 PASS)

**Question**: How much of the RMSE between factory-calibrated and field-measured soil moisture is systematic bias (correctable) vs random noise (irreducible)?

**Results**:
- CS616 in sand: NOISE-dominated (34.6% bias), noise floor 0.006 m³/m³
- CS616 in loamy sand: BIAS-dominated (59.2% bias), noise floor 0.021 m³/m³
- CS616 in sandy clay loam: NOISE-dominated (26.3% bias), noise floor 0.012 m³/m³
- EC5 across all soils: BIAS-dominated (62-77%), noise floor 0.004-0.020 m³/m³
- Site-specific calibration removes 50-80% of total sensor error

### Exp 002: Weather Model vs Observation (5/5 PASS)

**Question**: How does ERA5 reanalysis (Open-Meteo) differ from GHCND station observations (NOAA CDO)?

**Results** (synthetic NOAA mode — methodology demonstration):
- Metrics computed for tmax, tmin, precipitation
- Seasonal decomposition shows winter has largest gaps
- Bias-variance decomposition framework validated
- **Pending**: Real NOAA CDO data for production results (token available in testing-secrets)

### Exp 003: Error Propagation Through FAO-56 (8/8 PASS)

**Question**: Given known sensor uncertainties, how does measurement noise propagate through the FAO-56 ET₀ chain?

**Results**:
- ET₀ uncertainty: 3.879 ± 0.142 mm/day (CV = 3.7%)
- 90% CI: [3.647, 4.118] mm/day
- Sensitivity ranking: humidity (65.6%) > radiation (20.1%) > temperature (10.0%) > wind (4.3%)
- Monte Carlo / analytical agreement: ratio = 1.009 (first-order Taylor is adequate)

### Exp 004: Sequencing Depth & Taxonomic Noise (16/16 PASS)

**Question**: At what sequencing depth does 16S taxonomic assignment become stable?

**Results**:
- All phyla detected: 100 reads
- Shannon convergence (5%): 500 reads
- Genus saturation: 5,000 reads
- Noise floor at 100k reads: ±0.004 Shannon, ±0.4 genera
- Genera detection and Shannon diversity are monotonically increasing

### Exp 005: Seismic Wave Propagation (10/10 PASS)

**Question**: Can we locate an earthquake source from noisy P-wave arrivals?

**Results**:
- Clean inversion: 0.00 km error (perfect recovery)
- Noisy inversion (±0.5s): 0.9 km horizontal, 7.7 km depth error
- Monte Carlo uncertainty: ±2.1 km horizontal (90th: 3.9 km), ±8.5 km depth
- Depth is poorly constrained with surface-only stations
- 3 stations → 28 km error; 5 stations → <1 km error; 7 stations → <1 km error

## Run Log

### Run 1 — February 16, 2026

All 5 experiments executed sequentially via `scripts/run_all_baselines.sh`.

```
Exp 001: Sensor Noise Characterization       32/32 PASS
Exp 002: Weather Model vs Observation          5/5  PASS (synthetic NOAA)
Exp 003: Error Propagation Through FAO-56      8/8  PASS
Exp 004: Sequencing Depth & Taxonomic Noise   16/16 PASS
Exp 005: Seismic Wave Propagation             10/10 PASS
─────────────────────────────────────────────────────────
TOTAL                                         71/71 PASS
```

## Evolution Roadmap

- **Phase 0**: Python/NumPy/SciPy baselines ← **COMPLETE (71/71)**
- **Phase 0+**: Real open data pipelines (NOAA CDO, IRIS waveforms)
- **Phase 1**: BarraCUDA Rust port (noise-aware primitives for ToadStool)
- **neuralSpring bridge**: Export noise characterizations as labeled training data
