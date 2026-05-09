# Fossil: Validation Binaries (Prokaryotic Era)

**Fossilized**: May 9, 2026
**From**: `crates/groundspring-validate/src/`
**Superseded by**: `crates/groundspring/src/validation/scenarios/` + `groundspring_unibin validate`

## What This Was

35 standalone validation binaries, each with its own `fn main()`, compiled
separately and run individually. Each validated one specific experiment
domain (decompose, rarefaction, anderson, fao56, etc.).

## Why It Was Superseded

The eukaryotic UniBin pattern absorbs these into a `ScenarioRegistry` with
`ScenarioMeta` (id, track, tier, provenance). This enables:
- Filtered execution by tier (Rust/Live) and track
- Single binary deployment
- Unified reporting
- CI-friendly Tier 1 / Tier 2 distinction

## Binary List (at fossilization)

| Binary | Domain | Status |
|--------|--------|--------|
| validate_decompose | Noise decomposition | Absorbed → s_decompose |
| validate_rarefaction | Rarefaction curves | Absorbed → s_rarefaction |
| validate_anderson | Anderson localization | Absorbed → s_anderson |
| validate_fao56 | FAO-56 ET₀ | Absorbed → s_fao56 |
| validate_freeze_out | Freeze-out chi² | Absorbed → s_freeze_out |
| validate_bistable | Bistable switching | Absorbed → s_bistable |
| validate_seismic | Seismic travel-time | Absorbed → s_seismic |
| validate_drift | Wright-Fisher drift | Absorbed → s_drift |
| validate_jackknife | Jackknife resampling | Absorbed → s_jackknife |
| validate_all | Batch runner | Superseded by unibin |
| groundspring_guidestone | Certification L0-L4 | Absorbed → certification/ |
| (22 others) | Various domains | Remaining prokaryotic — future absorption |

## Note

The original binaries still exist in `crates/groundspring-validate/` for
backward compatibility during the transition period. This fossil record
documents the prokaryotic architecture before the eukaryotic UniBin absorbed
their function into organelle modules.
