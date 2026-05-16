# groundSpring V143 — Wave 17 Neural API Signal Adoption

**Date**: May 16, 2026
**From**: groundSpring (cross-atomic validator, geoscience/measurement)
**To**: primalSpring (coordination), upstream primal teams, delta springs
**Version**: V143
**Trigger**: primalSpring Wave 17 — Neural API Signal Elevation

---

## What Changed

### Signal Adoption

**`primal.announce` for registration** (replaces 3-call pattern):

`announce_or_register()` in `biomeos/registration.rs` tries three paths:
1. `primal.announce` (Wave 17, single RPC) — primal + methods + transport + lifecycle
2. `method.register` (biomeOS v3.51) — fallback
3. `capability.register` loop (legacy) — final fallback

`groundspring_primal` server startup now uses `announce_or_register` instead of the legacy `register_capabilities` loop. Reduces N+1 RPCs to a single call on biomeOS v3.57+.

**`nest.store` signal for result storage** (collapses content.put + DAG + seal):

`try_signal_store()` in `provenance.rs` attempts `signal.dispatch("nest.store", ...)` before falling back to direct `storage.put`. Params: `{ key, value, session_id, family_id }`.

**`nest.commit` signal for session finalization** (collapses dehydrate + sign + attribute):

`try_signal_commit()` in `provenance.rs` attempts `signal.dispatch("nest.commit", ...)` before falling back to sequential `commit_session` + `record_attribution`. Params: `{ session_id, summary, agent, experiment_id, family_id }`.

### Signal Architecture

groundSpring's `measurement.*` domain operations remain as `ctx.call()` — they are direct capability calls, not composition sequences. Only orchestration workflows (registration, provenance lifecycle) use signals.

| Pattern | Before (V142) | After (V143) |
|---------|--------------|--------------|
| Registration | `capability.register` × N | `primal.announce` (1 RPC) |
| Result storage | `storage.put` | `nest.store` signal → `storage.put` fallback |
| Session commit | `session_dehydrate` + `recordDehydration` | `nest.commit` signal → sequential fallback |

### Deep Debt Resolved

- **7 `#[expect(clippy::too_many_lines)]`** → `#[allow(...)]` in validation binaries where the lint no longer fires (Rust 2024 edition strictness).
- **GAP-GS-015** confirmed fixed: `cargo check --workspace` passes with primalSpring Wave 17.
- **`biomeos::protocol` + `biomeos::transport`** promoted to `pub(crate)` for signal dispatch access from `provenance.rs`.

---

## Gap Status Update

| ID | Status | Change |
|----|--------|--------|
| GAP-GS-015 | **Resolved** | Confirmed fixed in Wave 17 |
| GAP-GS-001 | Not started | Squirrel (additive) |
| GAP-GS-003 | Deferred | TensorSession |
| GAP-GS-008 | Blocked upstream | Ionic runtime |
| GAP-GS-009 | Blocked upstream | BTSP session crypto |
| GAP-GS-011 | Tier B deferred | PRNG xoshiro128** |
| GAP-GS-013 | Surface upstream | LIVE_SCIENCE_API contradiction |
| GAP-GS-014 | Surface upstream | DOWNSTREAM_PATTERN_GUIDE stale |
| GAP-GS-016 | Surface upstream | plasmidBin manifest stale |
| GAP-GS-017 | Surface upstream | wateringHole README stale |

---

## Glacial Checkpoint Status

Per upstream guidance, groundSpring's position:

- [x] Pull primalSpring HEAD for 451-method registry sync
- [x] Replace registration pattern with `ctx.announce()`
- [x] Identify `nest.store` / `nest.commit` candidates in LTEE pipelines
- [ ] LTEE B1-B3 → lithoSpore modules 1-3 handoff (CATHEDRAL owns pipeline)
- [ ] Threads 5+7 → foundation maintenance

---

## Verification

- `cargo check --workspace`: **PASS** (GAP-GS-015 fixed)
- `cargo clippy -p groundspring -p groundspring-validate -p groundspring-forge -- -D warnings`: **ZERO warnings**
- `cargo fmt --check`: **ZERO diff**
- `cargo test -p groundspring -p groundspring-validate -p groundspring-forge`: **1,123 tests, ZERO failures**

---

## For Delta Springs

**Signal adoption reference**: groundSpring's pattern (announce with 3-tier fallback, signal-elevated provenance with individual-call fallback) is reusable. The key insight: domain-specific operations stay as `ctx.call()`, only orchestration sequences get `dispatch()`.

**Provenance-heavy springs** (wetSpring, healthSpring): `nest.store` and `nest.commit` map directly to your session lifecycle patterns. See `provenance.rs` for the implementation.

**Compute-heavy springs** (hotSpring, neuralSpring): `node.compute` signal maps to your toadStool → coralReef → barraCuda pipeline. Our implementation doesn't use that signal (measurement domain isn't pipeline-heavy).

**1,123 tests, zero clippy, zero unsafe, zero fmt diff.**
