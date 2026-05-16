# groundSpring V143 — Wave 17 Signal Adoption

**Date**: May 16, 2026
**From**: groundSpring (cross-atomic validator, geoscience/measurement)
**To**: primalSpring (coordination), upstream primal teams, delta springs
**Version**: V143
**Trigger**: primalSpring Wave 17 — Neural API Signal Elevation

---

## What Changed

### 1. `primal.announce` Registration (Wave 17)

**Before** (legacy 3-call pattern):
```
capability.register(domain=measurement, caps=[16], mappings=...)
capability.register(cap=measurement.noise_decomposition, ...)  × 16
method.register(primal=groundspring, methods=[16], ...)
```
18+ RPC calls for startup registration.

**After** (`announce_or_register`):
```
primal.announce(primal=groundspring, socket=..., methods=[16],
                capabilities=[measurement], version=0.1.0,
                lifecycle={state: running})
```
Single RPC call. Falls back to legacy pattern on older biomeOS.

**Implementation**: `biomeos::announce_or_register()` in `registration.rs`. Exported from `biomeos` module. Called from `groundspring_primal.rs` server startup.

### 2. `nest.store` Signal Dispatch (Wave 17)

**Before** (legacy 4-call provenance sequence in `run_lifecycle`):
```
provenance.session_create → storage.put → provenance.session_dehydrate → contribution.recordDehydration
```
4 sequential capability-routed RPC calls.

**After** (`nest.store` signal dispatch):
```
dispatch("nest.store", { content: <base64>, author: "groundspring", metadata: {...} })
```
Single signal dispatch. biomeOS executes the `nest_store.toml` graph: NestGate `content.put` → rhizoCrypt `dag.event.append` → loamSpine `spine.seal` → sweetGrass `braid.create`.

**Implementation**: `ipc::nestgate::nest_store_dispatch()` with automatic fallback to `content.put` if `CompositionContext` signal dispatch is unavailable. `provenance::run_lifecycle()` now prefers signal path, with `run_lifecycle_legacy()` as fallback.

### 3. GAP-GS-015 Resolved

`cargo check --workspace` passes against primalSpring HEAD (Wave 17). The `composition::routing` module items are now re-exported: `ALL_CAPS`, `BTSP_EXTRA_CAPS`, `capability_to_primal`, etc.

### 4. Lint Cleanup

- `validate_ltee_fitness.rs`: Stale `#[expect(clippy::too_many_lines)]` removed (function no longer triggers).
- `validate_resampling_conv.rs`: `#[expect]` → `#[allow]` for `too_many_lines` to fix `unfulfilled_lint_expectations`.

---

## Signal Adoption Map

| Signal | groundSpring Usage | Status |
|--------|-------------------|--------|
| `nest.store` | LTEE provenance lifecycle (`run_lifecycle`) | **Wired** — dispatch + fallback |
| `nest.commit` | Session finalization (server shutdown) | Candidate — next wave |
| `primal.announce` | Server startup registration | **Wired** — announce + fallback |
| `node.compute` | Not applicable (groundSpring is measurement, not compute-heavy) | N/A |
| `tower.publish` | Not applicable (no signed result publication flow) | N/A |
| `tower.authenticate` | Not applicable (no session authentication flow) | N/A |

groundSpring's statistical method APIs (`measurement.*`) remain as `ctx.call()` — only orchestration sequences collapse to signals.

---

## IPC + Signal Surface (V143)

**20 JSON-RPC methods + 2 signal dispatch paths across 7 primals:**

| Category | Methods |
|----------|---------|
| ToadStool (3) | `toadstool.validate`, `toadstool.list_workloads`, `compute.device.enumerate` |
| barraCuda (2) | `barracuda.precision.route`, `health.version` |
| coralReef (5) | `shader.compile.wgsl`, `shader.compile.gemm`, `shader.targets`, `shader.validate`, `health.version` |
| NestGate (3) | `content.put`, `content.get`, `data.noaa_ghcnd` |
| BearDog (3) | `crypto.sign`, `crypto.hash_blake3`, `crypto.seed_fingerprint` |
| skunkBat (1) | `security.audit_log` |
| biomeOS (1) | `capability.call` |
| **Signals** (2) | `nest.store` (provenance lifecycle), `primal.announce` (registration) |

---

## Glacial Priorities (from audit)

| Priority | Status |
|----------|--------|
| Pull primalSpring HEAD for 451-method registry sync | **Done** — builds clean |
| Replace registration with `ctx.announce()` | **Done** — `announce_or_register` |
| Identify `nest.store` / `nest.commit` candidates in LTEE pipelines | **Done** — `nest.store` wired in `run_lifecycle` |
| LTEE B1-B3 → lithoSpore modules 1-3 handoff | In progress (CATHEDRAL pipeline) |
| Threads 5+7 → foundation maintenance | Active |

---

## Verification

- `cargo check --workspace`: **PASS** (GAP-GS-015 fix confirmed)
- `cargo clippy -p groundspring -p groundspring-validate -p groundspring-forge -- -D warnings`: **ZERO**
- `cargo fmt --check`: **ZERO diff**
- `cargo test -p groundspring -p groundspring-validate -p groundspring-forge`: **1,123 tests, ZERO failures**

**1,123 tests, zero clippy, zero unsafe, zero fmt diff.**
