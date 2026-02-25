# Exp 002: Weather Model vs Observation

**Domain**: Meteorology
**Source**: Open-Meteo ERA5 reanalysis + NOAA CDO station data
**Question**: How does gridded reanalysis differ from point station readings?

## Data Source

Open-Meteo API (free, no token) for ERA5 reanalysis.
NOAA CDO API for station observations (synthetic mode used pending CDO token).
Open data — both APIs publicly accessible.

## Method

Side-by-side comparison of temperature and precipitation.
Bias-variance decomposition of model-observation gap.
Seasonal analysis (DJF/MAM/JJA/SON), precipitation hit-rate analysis.

## Key Result

The model-observation gap is **representation noise dominated** — most of
the difference between a 10km grid cell and a point station is spatial
variability, not systematic bias. Bias correction alone cannot close the gap.

## Validation

| Phase | Checks | Binary |
|-------|--------|--------|
| Phase 0 (Python) | PASS (synthetic) | `control/observation_gap/observation_gap.py` |
| Phase 1 (Rust) | 13/13 | `validate-weather` |

## Barracuda Path

Tier A — stats delegates to barracuda CPU. Full GPU pending adapter.

## Modules

`stats`, `decompose`, `prng`
