# groundSpring V144 — Wave 20 Schema Standardization + E2E Validation

**Date**: May 16, 2026
**From**: groundSpring (cross-atomic validator, geoscience/measurement)
**To**: primalSpring (coordination), upstream primal teams, delta springs
**Version**: V144
**Trigger**: primalSpring Wave 20 — Schema Standardization + E2E Validation

---

## Wave 20 Checklist (groundSpring)

- [x] **`capability.list` canonical envelope**: Added `"primal"` and `"count"` fields. Response is now `{ "primal": "groundspring", "domain": "measurement", "capabilities": [...], "count": 16 }`. `domain` retained as allowed extra field.
- [x] **Registry sync**: Cross-check test updated to reference 452 methods (Wave 20: `primal.list` added).
- [x] **`nest.commit` signal dispatch**: Wired for LTEE session finalization. Collapses 5-primal sequential graph into single `dispatch("nest.commit", { session_id })`.
- [ ] **`--provenance-dir`**: Future — when foundation workloads call groundSpring validate binaries with this flag.

---

## What Changed

### 1. `capability.list` Canonical Envelope

`dispatch/lifecycle.rs` — `capability_list()` now returns the Wave 20 canonical shape:

```json
{
  "primal": "groundspring",
  "domain": "measurement",
  "capabilities": ["measurement.noise_decomposition", "measurement.anderson_validation", ...],
  "count": 16
}
```

Test `capability_list_has_canonical_envelope` verifies: `primal` is string, `count` is u64, `count == capabilities.len()`.

### 2. `nest.commit` Signal Dispatch

`ipc::nestgate::nest_commit_dispatch(session_id)` — dispatches `nest.commit` via `CompositionContext`. biomeOS executes the `nest_commit.toml` graph: rhizoCrypt `event.append` → bearDog `crypto.sign` → nestGate `content.put` → loamSpine `session.commit` → sweetGrass `braid.create`.

`provenance::commit_session()` — now prefers `nest.commit` signal, falling back to legacy `provenance.session_dehydrate`.

### 3. Registry Sync

Cross-check test comment updated from 413 to 452 (Wave 20: `primal.list` canonical schema added). Threshold remains `>= 401` (production methods extracted from TOML, excludes test fixtures).

---

## Signal Surface (V144)

| Signal | Usage | Status |
|--------|-------|--------|
| `primal.announce` | Server startup registration | Wired (V143) |
| `nest.store` | LTEE provenance lifecycle | Wired (V143) |
| `nest.commit` | Session finalization / dehydration | **Wired (V144)** |

All three signals have automatic fallback to legacy multi-call patterns.

## IPC + Signal Surface (V144)

**20 JSON-RPC methods + 3 signal dispatch paths across 7 primals.**

---

## Verification

- `cargo check --workspace`: PASS
- `cargo clippy -D warnings`: ZERO
- `cargo fmt --check`: ZERO diff
- `cargo test`: **1,123 tests, ZERO failures**
