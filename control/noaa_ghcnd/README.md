# NOAA GHCND — NestGate Pipeline Exercise

**Status**: SCAFFOLDED (awaiting NestGate deployment)
**Experiment**: 040 (proposed)
**Spring**: groundSpring V141

## Purpose

First real-dataset ingestion through the NestGate CAS (Content-Addressed Storage) pipeline. NOAA GHCND daily weather observations are public domain, well-documented, and deterministic — the ideal first NestGate pipeline exercise.

## Pipeline Flow

```
NestGate data.noaa_ghcnd  →  CSV observations
        ↓
groundSpring validate     →  range checks, completeness, TMAX > TMIN
        ↓
NestGate content.put      →  BLAKE3-hashed storage with provenance
        ↓
lithoSpore / foundation   →  downstream consumption
```

## Dataset

- **Station**: USW00094728 (Central Park, New York, NY)
- **Period**: 2024-01-01 to 2024-12-31
- **Elements**: TMAX, TMIN, PRCP, SNOW, SNWD
- **Expected records**: 360+ (< 10 missing days)

## Validation Rules

| Rule | Threshold |
|------|-----------|
| TMAX range (°C) | [-20, 45] |
| TMIN range (°C) | [-30, 35] |
| PRCP range (mm) | [0, 300] |
| TMAX > TMIN | always |
| Completeness | ≥ 360/366 records |

## Dependencies

- NestGate `data.noaa_ghcnd` method (awaiting deployment)
- NestGate `content.put` / `content.get` for CAS storage
- groundSpring IPC wiring in `crates/groundspring/src/ipc/nestgate.rs`

## Files

- `pipeline_config.toml` — Machine-readable pipeline configuration
- `README.md` — This file
