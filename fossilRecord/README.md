# groundSpring Fossil Record

**Date**: May 9, 2026
**Event**: Interstadial Primordial Extinction — Eukaryotic Evolution Wave

This directory preserves snapshots of pre-extinction (prokaryotic) code
patterns that were superseded during the eukaryotic UniBin evolution.
These are not dead code — they are provenance evidence.

## Fossils

Each subdirectory contains a `README.md` provenance marker documenting
what the prokaryotic pattern looked like and what superseded it. The actual
source code lives in its evolved location — fossils are provenance markers,
not source copies.

### `validate_binaries_prokaryotic_may2026/`

Provenance marker for the 35 standalone validation binaries. The binaries
still live at `crates/groundspring-validate/src/validate_*.rs` and are
additionally accessible via `groundspring_unibin validate`.

### `guidestone_prokaryotic_may2026/`

Provenance marker for the standalone `groundspring_guidestone` binary.
The guidestone was refactored (V131) into modular layers at
`crates/groundspring-validate/src/guidestone/{bare,tower,node,nest,cross}.rs`
with a thin binary orchestrator. Also accessible via `groundspring_unibin certify`.

### `experiment_crates_prokaryotic_may2026/`

Provenance marker for standalone experiment crate entry points (`exp094`,
`exp095`). These crates still exist at `experiments/` and are additionally
registered as validation scenarios with provenance metadata.

## Provenance

- **primalSpring**: v0.9.25 (Phase 60+ INTERSTADIAL)
- **groundSpring**: V126 (eukaryotic evolution)
- **Handoff**: `wateringHole/handoffs/GROUNDSPRING_V126_INTERSTADIAL_UNIBIN_HANDOFF_MAY09_2026.md`
- **License**: AGPL-3.0-or-later
