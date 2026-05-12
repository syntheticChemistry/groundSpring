# groundSpring V128 — Tier 4 IPC-First + River Delta Evolution

**Date**: May 10, 2026
**Version**: V128
**Tests**: 1,101 (Rust) + 287 (Python)
**Clippy**: 0 warnings on all targets
**Build**: Clean workspace (default = no barracuda)

---

## Summary

V128 implements the primalSpring post-interstadial river delta evolution
guidance. The core change is Tier 4 IPC-first: `barracuda` is no longer a
default dependency. All 284 `barracuda::` references are behind
`#[cfg(feature = "barracuda")]` and the `local` feature enables direct
library linkage when needed.

## Changes

### Tier 4 IPC-First

- `default = []` in `Cargo.toml` (was `default = ["barracuda"]`)
- `local = ["barracuda"]` feature added for opt-in library path
- All 284 barracuda references already properly feature-gated
- `PRIMAL_PROOF_IPC_MAPPING.md` updated to reflect Tier 4 status

### biomeOS v3.51 Absorption

- `composition_status()` in `biomeos::health` — calls `composition.status`,
  returns `CompositionStatus { active_users, primal_health, resource_pressure }`
- `register_methods()` in `biomeos::registration` — calls `method.register`
  (GAP-09) to dynamically register 16 `measurement.*` methods

### skunkBat Audit Logging (JH-5)

All 6 deploy graphs now include `security.audit_log` nodes:
- `groundspring_deploy` — niche deploy event
- `groundspring_validation` — validation result
- `groundspring_cross_substrate` — cross-substrate parity
- `groundspring_nucleus_local` — NUCLEUS bootstrap
- `groundspring_tower_bootstrap` — Tower bootstrap
- `groundspring_nucleus_node` — node validation

### CI Cross-Sync

- Registry sync tests updated for canonical 400+ methods (extracts 400)
- Tool count test added: validates exactly 16 measurement.* tools
- 8 registry sync tests pass

### Documentation

- `CONTEXT.md` rewritten: eukaryotic UniBin architecture, correct method
  names, certification/validation/ipc/fossilRecord, primalSpring v0.9.25
- `README.md`: V128 status
- `CHANGELOG.md`: V128 entry

## Upstream Debt (for primalSpring)

- Handoff `SPRING_NUCLEUS_AUDIT_MAY2026.md` lists groundSpring as V124 /
  guideStone L0 — stale. Actual: V128, guideStone L4.
- Scorecard says 965+ tests — actual: 1,101.
- `measurement.*` domain not in canonical registry (by design — niche-scoped).

## Downstream Patterns

- **IPC-first pattern**: `default = []` + `local` feature for library opt-in
  is the reference pattern for other springs' Tier 4 migration
- **skunkBat audit in deploy graphs**: `security.audit_log` with
  `fallback = "skip"` is the non-blocking pattern
- **biomeOS v3.51 absorption**: `composition.status` and `method.register`
  client code in `biomeos/` module is reusable

## License

AGPL-3.0-or-later
