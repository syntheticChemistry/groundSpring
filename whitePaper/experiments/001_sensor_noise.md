# Exp 001: Sensor Noise Characterization

**Domain**: Agricultural sensing (soil moisture)
**Paper**: Dong et al. (2020) — factory calibration of CS616 and EC5 sensors
**Question**: How much sensor error is correctable bias vs irreducible noise?

## Data Source

Published factory calibration statistics (RMSE, MBE) for CS616 and EC5 sensors
across three Michigan soil types (sand, loamy sand, sandy clay loam).
Open data — digitized from peer-reviewed publication.

## Method

Bias-variance decomposition: RMSE² = MBE² + σ²(random).
Noise floor reduction: corrected RMSE after subtracting known bias.

## Key Result

EC5 sensors are **bias-dominated** (62-77% of error is systematic).
CS616 sensors have **mixed noise structure** — sand and clay soils are
noise-dominated, while loamy sand is bias-dominated. Site-specific
calibration removes 50-80% of total sensor error.

## Validation

| Phase | Checks | Binary |
|-------|--------|--------|
| Phase 0 (Python) | 32/32 | `control/sensor_noise/sensor_noise_decomposition.py` |
| Phase 1 (Rust) | 36/36 | `validate-decompose` |

## Barracuda Path

Tier A — `stats` functions delegate to `barracuda::stats` (CPU done).
GPU adapter needed for batch decomposition at scale.

## Modules

`stats`, `decompose`
