# Exp 032: IRIS Seismic via NUCLEUS

## Domain
Geological (IRIS) — IRIS FDSN station geometry + seismic travel times

## Question
Can groundSpring's seismic modules (haversine, travel times, grid-search inversion)
produce correct results when driven by real IRIS FDSN station metadata obtained
through NestGate's IRIS data provider?

## Method
- Query IRIS FDSN for seismic stations via NestGate `data.iris_stations` (if NUCLEUS live)
- Fall back to synthetic NMSZ (New Madrid Seismic Zone) stations when NUCLEUS unavailable
- Compute: inter-station distances (haversine), P-wave travel times, event queries
- Validate: distance symmetry, travel time positivity, reasonable magnitudes

## Results
- 12/12 validation checks PASS
- Inter-station distances physically reasonable (> 0 km for distinct stations)
- Distance matrix symmetric (d(A,B) = d(B,A))
- Travel times positive and proportional to distance
- P-wave velocity consistent with crustal average (~6.0 km/s)
- Event queries return valid results or gracefully degrade
- Provenance stored to NestGate when available

## Validation
- Rust: `validate-iris-seismic` (requires `--features biomeos`)
- Sovereign fallback: synthetic NMSZ stations when NUCLEUS offline

## Cross-Spring
- Uses `seismic::haversine_km`, `seismic::travel_time_1d`, `nestgate::iris_stations`
- NestGate IRIS FDSN data provider via biomeOS Neural API
- Extends Exp 005 (seismic waves) with real station geometry

## Key Finding
Real IRIS station geometry produces inter-station distances and travel times consistent
with the synthetic NMSZ baseline from Exp 005, confirming that the seismic modules
are physically correct at continental scale.
