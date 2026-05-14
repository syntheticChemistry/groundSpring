# groundSpring V142 — Compute Trio Wave Absorption

**Date**: May 14, 2026
**From**: groundSpring (cross-atomic validator, geoscience/measurement)
**To**: primalSpring (coordination), upstream primal teams, delta springs
**Version**: V142
**Trigger**: Ecosystem Status Update — May 14, 2026 (compute trio evolution wave)

---

## What Changed

### Compute Trio IPC Deepening

Absorbed the compute trio evolution wave into groundSpring's IPC surface:

**coralReef v0.1.0** (Sprint 11+12):
- `shader.compile.gemm` — tensor-core GEMM kernel compilation (SM80+ `mma.sync`). Accepts `{ m, n, k, precision, arch }`. JSON-RPC + `try_compile_gemm` graceful degradation.
- `health.version` — trio-consistent build identity probe. Returns session ID, build hash, version, primal name. JSON-RPC + `try_health_version`.

**barraCuda v0.4.0** (Sprint 69):
- `health.version` — trio-consistent version probe. Returns `{ primal, version, rust_version }`. Matches toadStool and coralReef surface for plasmidBin doctor and upgrade verification.

### IPC Surface Summary (V142)

**20 JSON-RPC methods across 7 primals** (was 17 at V141):

| Primal | Methods | Count |
|--------|---------|:-----:|
| ToadStool | `toadstool.validate`, `toadstool.list_workloads`, `compute.device.enumerate` | 3 |
| barraCuda | `barracuda.precision.route`, `health.version` | 2 |
| coralReef | `shader.compile.wgsl`, `shader.compile.gemm`, `shader.targets`, `shader.validate`, `health.version` | 5 |
| NestGate | `content.put`, `content.get`, `data.noaa_ghcnd` | 3 |
| BearDog | `crypto.sign`, `crypto.hash_blake3`, `crypto.seed_fingerprint` | 3 |
| skunkBat | `security.audit_log` | 1 |
| biomeOS | `capability.call` | 1 |
| | | **20** (was 17) |

New methods: `shader.compile.gemm`, coralReef `health.version`, barraCuda `health.version`.

### Upstream Gaps Surfaced

Three new gaps documented in `docs/PRIMAL_GAPS.md`:

| ID | Description | Severity | Owner |
|----|-------------|----------|-------|
| GAP-GS-015 | primalSpring `routing` module private — blocks `cargo check --workspace` | Medium | primalSpring |
| GAP-GS-016 | plasmidBin manifest metadata stale (tests=1050, latest=0.1.0, niche omits skunkBat) | Low | primalSpring/plasmidBin |
| GAP-GS-017 | wateringHole README stale groundSpring row (V135, 1125 tests) | Low | primalSpring/wateringHole |

### sourDough Awareness

`SOURDOUGH_DEPLOYMENT_INTERNALIZATION.md` (v0.3.0–v0.6.0) reviewed. No code change needed in groundSpring. Shell scripts remain functional during overlap period. groundSpring's plasmidBin binary (1.1M musl-static) is harvestable.

---

## Existing Gap Status (Unchanged)

| ID | Status | Notes |
|----|--------|-------|
| GAP-GS-001 | Not started | Squirrel not in composition (additive, non-blocking) |
| GAP-GS-002 | **Resolved** | coralReef now 5 methods (was 3 at V141) |
| GAP-GS-003 | Deferred | TensorSession adoption (monitoring) |
| GAP-GS-008 | Blocked upstream | Ionic runtime cross-family GPU lease |
| GAP-GS-009 | Blocked upstream | BTSP barraCuda session crypto |
| GAP-GS-011 | Tier B deferred | PRNG xoshiro128** rebaseline |
| GAP-GS-013 | Surface upstream | LIVE_SCIENCE_API.md precision.route status contradiction |
| GAP-GS-014 | Surface upstream | DOWNSTREAM_PATTERN_GUIDE missing B4 |

---

## Niche Posture

**Status: `composing`** — cannot advance to `composed` while:
1. primalSpring `routing` module bug blocks workspace builds (GAP-GS-015)
2. plasmidBin manifest metadata needs reconciliation (GAP-GS-016)
3. PRNG Phase 2b remains deferred (GAP-GS-011)

**Holding**: Full NUCLEUS compositions until atomic specialists (ludoSpring Tower, healthSpring Nest) confirm live validation.

**Deepening**: 20 IPC methods (most complete trio wiring in the delta). LTEE B1-B4 → lithoSpore. NOAA GHCND scaffolded for NestGate pipeline exercise.

---

## Verification

- `cargo check -p groundspring -p groundspring-validate -p groundspring-forge`: PASS
- `cargo clippy -p groundspring -p groundspring-validate -p groundspring-forge -- -D warnings`: ZERO warnings
- `cargo fmt --check`: ZERO diff
- `cargo test -p groundspring -p groundspring-validate -p groundspring-forge`: **1,123 tests, ZERO failures**
- `cargo check --workspace`: FAILS on primalSpring `routing` module (GAP-GS-015, upstream)

---

## For Upstream Teams

**primalSpring**: Fix `composition::routing` visibility (GAP-GS-015). Update plasmidBin manifest groundSpring entry (GAP-GS-016). Refresh wateringHole README (GAP-GS-017).

**barraCuda team**: groundSpring now consumes `health.version` and `precision.route`. TensorSession adoption deferred (GAP-GS-003) — monitoring for measurement workload applicability.

**coralReef team**: groundSpring now consumes full surface: `compile.wgsl`, `compile.gemm`, `targets`, `validate`, `health.version`. RayQuery PTX (Sprint 12) not yet consumed — applicable when toadStool provides RT core dispatch.

**Delta springs**: groundSpring's 20-method IPC surface is available as a reference for compute trio wiring depth.

**1,123 tests, zero clippy, zero unsafe, zero fmt diff.**
