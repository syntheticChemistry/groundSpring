# Fossil: Experiment Crates (Prokaryotic Era)

**Fossilized**: May 9, 2026
**From**: `experiments/exp094_composition_parity/`, `experiments/exp095_measurement_niche/`
**Superseded by**: `crates/groundspring/src/validation/scenarios/` + `groundspring_unibin validate`

## What These Were

Two standalone experiment crates, each with their own `Cargo.toml` and
`src/main.rs`, exercising NUCLEUS composition parity and measurement
niche registration.

## Why They Were Superseded

The eukaryotic validation framework absorbs experiment logic into scenario
modules with `ScenarioMeta` provenance tracking. The experiment crates
remain as workspace members for backward compatibility, but their core
logic now lives in the validation registry.

## Experiment Crates at Fossilization

### exp094_composition_parity
- **Focus**: Full NUCLEUS composition parity (Tower + Node + Nest + cross-atomic)
- **Uses**: `primalspring::composition::CompositionContext`
- **Absorbed into**: `s_composition_parity` scenario (Tier: Live, Track: CompositionParity)

### exp095_measurement_niche
- **Focus**: Measurement niche registration and capability discovery
- **Uses**: `primalspring::composition::CompositionContext`, `groundspring::niche`
- **Absorbed into**: Certification L2 discovery checks

## Note

The original experiment crates still exist in `experiments/` for backward
compatibility. This fossil record documents the standalone crate architecture
before absorption into the eukaryotic validation registry.
