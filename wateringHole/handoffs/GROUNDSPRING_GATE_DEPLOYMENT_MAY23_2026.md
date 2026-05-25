# groundSpring Gate Deployment Status — eastGate (May 25, 2026)

**Spring**: groundSpring V146
**Gate**: eastGate
**Hardware**: i9-12900, RTX 4070 + Akida NPU, 32GB DDR5
**Co-residents**: primalSpring (coordinator), neuralSpring
**NUCLEUS Composition**: Full NUCLEUS (13 primals) + `groundspring_cell.toml` overlay
**Status**: OPERATIONAL — covalent mesh sound-off complete

---

## Gate Configuration

### Proto-Nucleate Parameters (from `downstream_manifest.toml`)

```toml
spring_name = "groundspring"
owner = "groundSpring"
domain = "geoscience"
particle_profile = "balanced"
fragments = ["tower_atomic", "node_atomic", "nest_atomic"]
depends_on = ["beardog", "songbird", "coralreef", "toadstool", "barracuda", "nestgate"]
validation_capabilities = ["tensor.matmul", "stats.mean", "compute.dispatch", "storage.store", "crypto.hash"]
```

### Primal Dependencies (6 of 13)

| Primal | Atomic | groundSpring Usage |
|--------|--------|-------------------|
| BearDog | Tower | `crypto.sign`, `crypto.hash_blake3`, `crypto.seed_fingerprint` |
| Songbird | Tower | Discovery, `ipc.resolve` for capability routing |
| coralReef | Node | `shader.compile.wgsl`, `shader.compile.gemm`, `shader.targets`, `shader.validate`, `health.version` |
| ToadStool | Node | `toadstool.validate`, `toadstool.list_workloads`, `compute.device.enumerate` |
| barraCuda | Node | `barracuda.precision.route`, `health.version` |
| NestGate | Nest | `content.put`, `content.get`, `data.noaa_ghcnd`, `nest.store` (signal), `nest.commit` (signal) |

### Additional IPC (wired but not in proto-nucleate `depends_on`)

| Primal | Method | Notes |
|--------|--------|-------|
| skunkBat | `security.audit_log` | Tower defense — present in all compositions |
| rhizoCrypt | via `nest.store` signal | DAG event append (collapsed into signal) |
| loamSpine | via `nest.store` signal | Permanent ledger seal (collapsed into signal) |
| sweetGrass | via `nest.commit` signal | Attribution braid (collapsed into signal) |

---

## Deployment Validation Plan

### Phase 1: NUCLEUS Startup (plasmidBin v2026.05.23)

```bash
cd $ECOPRIMALS/infra/plasmidBin
./nucleus_launcher.sh --family-id irongate-gs --composition nucleus
```

Expected: 13/13 primals healthy, Songbird registry seeded.

### Phase 2: groundSpring Primal-Proof Validation

```bash
cd $ECOPRIMALS/springs/groundSpring
cargo run --bin groundspring_unibin -- validate --format json
cargo run --bin groundspring_guidestone
```

Expected: 1,123 tests pass locally. GuideStone self-validates against live IPC.

### Phase 3: Live IPC Parity

```bash
cargo run --bin groundspring_unibin -- server --family-id irongate-gs
```

Expected: groundSpring serves 16 `measurement.*` capabilities via Songbird registration.
Other gate residents (primalSpring, ludoSpring) can discover and call groundSpring methods.

---

## Current State

### What Works

- All 6 deploy graphs carry Dark Forest gate metadata (`secure_by_default`, `uds_only`, `MethodGate`)
- Registry cross-sync at 458 methods (primalSpring v0.9.27)
- 20 IPC methods wired across 7 primals with `try_*` graceful degradation
- 3 signal dispatch paths (`nest.store`, `nest.commit`, `primal.announce`)
- GuideStone Level 4 self-validation (bare + NUCLEUS layers)
- All IPC uses runtime discovery — no hardcoded socket paths

### What Needs Live Validation

- [ ] NUCLEUS launcher starts 6 required primals on eastGate hardware
- [ ] GuideStone discovers primals via Songbird and validates IPC parity
- [ ] `crypto.sign` produces valid Ed25519 signatures from BearDog
- [ ] `barracuda.precision.route` returns precision strategy for `stats.mean`
- [ ] `content.put` + `content.get` roundtrip succeeds via NestGate
- [ ] `nest.store` signal dispatch collapses multi-call provenance chain
- [ ] Multi-spring contention: groundSpring + neuralSpring concurrent on same NUCLEUS
- [ ] `toadstool.validate` confirms workload viability for measurement domain
- [ ] Akida NPU discovery via `compute.device.enumerate` (eastGate unique hardware)

### Gaps Found (handback to primalSpring)

| # | Gap | Severity | Notes |
|---|-----|----------|-------|
| 1 | Proto-nucleate `validation_capabilities` uses abstract names | Low | `storage.store` vs actual `content.put`; Songbird routes abstract→concrete. Not a bug, but documentation gap. |
| 2 | `NUCLEUS_SPRING_ALIGNMENT.md` shows groundSpring at V135/gS 0 | Stale | Actual: V146/gS L4. Stale in upstream doc. |
| 3 | No `fetch_primals.sh` script (actual: `fetch.sh` in plasmidBin) | Naming | Audit blurb references `fetch_primals.sh --all`; actual infra uses `fetch.sh`. |

---

## Multi-Domain Composition Notes (eastGate)

eastGate hosts 3 springs:
- **primalSpring**: Coordinator — 49 scenarios, registry validation, graph orchestration
- **neuralSpring**: ML inference — Squirrel pipeline, weight persistence, NestGate heavy
- **groundSpring**: Measurement — Node + Nest balanced, science-local math

Expected interaction patterns:
- All three register with Songbird (separate capability domains: `coordination.*`, `inference.*`, `measurement.*`)
- No capability name collisions (domains are disjoint)
- Socket namespace: each spring uses its own UDS path under `/run/user/$UID/biomeos/`
- Resource contention risk: barraCuda GPU (RTX 4070) shared between neuralSpring ML inference and groundSpring measurement dispatch
- NPU opportunity: Akida NPU available for int8-quantized workloads (neuralSpring primary consumer, groundSpring optional)

---

## Deployment Readiness Checklist

- [x] Dark Forest gate metadata on all deploy graphs
- [x] Registry sync 458 (primalSpring v0.9.27)
- [x] `primal.announce` / `announce_or_register` for registration
- [x] UDS-only transport (no TCP hardcoding)
- [x] GuideStone Level 4 (self-validating)
- [x] `try_*` wrappers for graceful degradation (primals absent → `Ok(None)`)
- [x] plasmidBin binaries present (13/13 in `primals/x86_64-unknown-linux-musl/`)
- [x] Gate declared: eastGate (shared NUCLEUS with primalSpring)
- [x] Cell graph: `plasmidBin/cells/groundspring_cell.toml`
- [x] Songbird federation port 7700 (cross-gate LAN discovery)
- [x] Covalent mesh sound-off complete (Wave 48)
- [ ] Live `capability.call` cross-gate verified (eastGate → biomeGate physics, southGate biology)
- [ ] Multi-spring concurrent validation (with neuralSpring on eastGate)
