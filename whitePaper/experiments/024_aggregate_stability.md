# Experiment 024: Aggregate Stability Measurement Noise

**Domain**: Cross-spring (soil physics + Anderson localization)
**Paper**: Nimmo & Perkins (2002); Kemper & Rosenau (1986); Bourgain & Kachkovskiy (2018)
**Phase 0**: 8/8 PASS (Python)
**Phase 1**: 8/8 PASS (Rust)
**Barracuda**: Uses decompose + stats modules

## Question

How precisely must aggregate stability (WSA) be measured to distinguish Anderson localization regimes (d_eff = 2 vs d_eff = 3)?

## Method

1. Two soil states: tilled (WSA=0.35, d_eff≈2) and no-till (WSA=0.75, d_eff≈3)
2. Simulate measured WSA with bias (0.02) and random noise (σ=0.05)
3. Map WSA → d_eff via linear calibration (d_eff = 2.5×WSA + 1.125)
4. Bias-variance decomposition of measurement error
5. Regime discrimination: can we distinguish d_eff=2 from d_eff=3?

## Key Results

- Tilled d_eff: 2.02 ± 0.14 (CV: 0.071)
- No-till d_eff: 3.06 ± 0.12 (CV: 0.039)
- Regimes distinguishable (non-overlapping 95% intervals)
- Noise floor (0.12-0.14) well below regime gap (1.0)
- Bias fraction: 0.02-0.21 (varies by soil state)

## Files

| File | Description |
|------|-------------|
| `control/aggregate_stability/aggregate_stability.py` | Python baseline |
| `control/aggregate_stability/benchmark_aggregate_stability.json` | Benchmark config |
| `crates/groundspring-validate/src/validate_aggregate_stability.rs` | Rust validation binary |

## Cross-Spring

Extends Exp 001 (sensor noise decomposition) methodology to soil structure.
Contributes to baseCamp Sub-thesis 06.
