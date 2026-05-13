# groundSpring V140 — Tier 2 Convergence Wave Handoff

**Date**: May 13, 2026
**From**: groundSpring (river delta)
**For**: primalSpring coordination, upstream primal teams, lithoSpore
**Context**: Response to Tier 2 Convergence Wave (May 13, 2026). All code changes verified: 1,123 tests, zero clippy, zero unsafe.

---

## What We Did

### 1. Aligned `toadstool.list_workloads` with upstream contract

The LIVE_SCIENCE_API.md specifies `{ "filter": "active" }` as a parameter. Our V139 wire sent empty params. Fixed:
- tarpc trait: `async fn list_workloads(filter: String)` (was parameterless)
- JSON-RPC helper: now sends `{ "filter": filter }` to match S245+ contract
- `try_list_workloads(filter)` updated accordingly

### 2. Wired coralReef `shader.compile.wgsl` (closed GAP-GS-002)

coralReef FECS stability proof shipped (Sprint 7, 4,790 tests). Updated `ipc/coralreef.rs`:
- `ShaderCompile` trait params aligned to upstream: `compile_wgsl(source, target, sm_version)`
  - Was: `compile_wgsl(source, entry_point)` — incorrect params from pre-FECS stub
- Added biomeOS JSON-RPC helpers: `compile_wgsl()`, `try_compile_wgsl()` with graceful degradation
- Updated status from "Stub — awaiting SM rebuild" to "Wired — FECS Sprint 7"
- Updated `ipc/mod.rs` doc to reflect coralReef is no longer a stub

### 3. Packaged LTEE B1-B4 to lithoSpore standard

Created `tolerances.toml` for all 4 LTEE control directories:
- `control/ltee_fitness_dynamics/tolerances.toml` — B2 (Wiser 2013) → module 1
- `control/ltee_neutral_mutation/tolerances.toml` — B1 (Barrick 2009) → module 2
- `control/ltee_clonal_interference/tolerances.toml` — B3 (Good 2017) → module 3
- `control/ltee_citrate_innovation/tolerances.toml` — B4 (Blount 2008/2012) → module 4

Each contains `[meta]` (paper_id, experiment, litho_module, spring) and `[tolerances]` with named tolerance thresholds. lithoSpore can now consume these alongside `expected_values.json`.

### 4. Validated plasmidBin deployment

Successfully built musl-static binary:
```
target: x86_64-unknown-linux-musl
binary: groundspring_unibin
size: 1.3M (stripped, static-pie, LTO)
commands: version ✓, validate --list ✓ (10 scenarios, 38 validations)
```

Note: Springs are not in `plasmidBin/sources.toml` (only primals ship there). Spring binaries are built locally from source for projectNUCLEUS dispatch.

### 5. Confirmed `toadstool.validate` + `barracuda.precision.route` already aligned

Both wires from V139 match the LIVE_SCIENCE_API.md contract exactly. No changes needed.

---

## Gaps Surfaced Upstream (handback to primalSpring)

### GAP-GS-013: LIVE_SCIENCE_API.md `precision.route` status contradiction

`primalSpring/docs/LIVE_SCIENCE_API.md` line 184 says `barracuda.precision.route` is **NOT IMPLEMENTED**, but the Tier 2 Convergence Wave blurb says **IMPLEMENTED (649 tests)**. One is stale.

### GAP-GS-014: DOWNSTREAM_PATTERN_GUIDE missing groundSpring B4

`primalSpring/docs/DOWNSTREAM_PATTERN_GUIDE.md` lists groundSpring LTEE as "B1-B3 DONE" with 1,125 tests. Actual state: **B1-B4 DONE** (Exp 039 citrate innovation, V138), **1,123 tests** (V139+).

---

## Tier 2 IPC Surface (complete)

| Method | Module | Status | Contract Match |
|--------|--------|--------|---------------|
| `toadstool.validate` | `ipc/toadstool.rs` | Wired (V139) | Exact match |
| `toadstool.list_workloads` | `ipc/toadstool.rs` | Wired (V140) | `filter` param added |
| `barracuda.precision.route` | `ipc/barracuda.rs` | Wired (V139) | Exact match |
| `shader.compile.wgsl` | `ipc/coralreef.rs` | Wired (V140) | Params realigned |
| `compute.execute` | `ipc/barracuda.rs` | Wired (pre-V130) | Existing |
| `compute.submit` | `ipc/barracuda.rs` | Wired (pre-V130) | Existing |
| `compute.capabilities` | `ipc/barracuda.rs` | Wired (pre-V130) | Existing |

All methods have `try_*` graceful degradation wrappers for when primals are not discovered.
