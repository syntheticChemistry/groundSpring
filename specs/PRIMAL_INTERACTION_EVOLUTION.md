# Primal Interaction Evolution

**Last updated**: May 13, 2026 (V141 — 17 IPC methods across 7 primals: ToadStool (validate, list_workloads, device.enumerate), barraCuda (precision.route), coralReef (compile.wgsl, targets, validate), NestGate (content.put/get, data.noaa_ghcnd), BearDog (crypto.sign/hash_blake3/seed_fingerprint), skunkBat (security.audit_log), biomeOS (capability.call). Tier 4 IPC-first, LTEE B1–B4 COMPLETE with BLAKE3 ingestion manifest, 1,123 tests)

This document tracks the evolution of groundSpring's interaction with the
ecoPrimals ecosystem through biomeOS and the NUCLEUS Neural API, mirroring
the shader evolution pattern in `CROSS_SPRING_EVOLUTION.md`.

## Evolution Phases

| Phase | Strategy | Status | Description |
|-------|----------|--------|-------------|
| V0 | Sovereign isolation | **retired** | Pure local computation, no primal awareness |
| V1 | Socket discovery | **active** | `biomeos.rs` discovers Neural API socket, health check |
| V2 | Capability routing | **active** | `capability_call()` routes through biomeOS translations |
| V3 | Data pipelines | **active** | NestGate `data.*` providers (NCBI, NOAA, IRIS) |
| V4 | Multi-primal workflows | **active** | Exp 031 exercises Tower+Node+Nest+Squirrel |
| V4.1 | Direct primal discovery | **active** | `discover_primals()` + `primal_health()` + `direct_primal_rpc()` |
| V4.2 | Adaptive health | **active** | `health()` tries `neural_api.get_metrics` then `topology.metrics` |
| V5 | Graph pipelines | planned | biomeOS graph executor for multi-step science workflows |
| V6 | Pathway learning | planned | biomeOS learns optimal primal routing from usage patterns |

## Live NUCLEUS Interaction Map (V4.2)

```
groundSpring
 ├── biomeos::auto_connect()
 │   └── /run/user/1000/biomeos/neural-api.sock ─── ✅ CONNECTED
 │
 ├── biomeos::discover_primals()
 │   ├── beardog.sock ──────── ✅ 4 primals discovered
 │   ├── songbird.sock
 │   ├── toadstool.sock
 │   └── squirrel.sock
 │
 ├── Neural API (biomeOS orchestrator)
 │   ├── neural_api.get_metrics ──── health + system stats .... ✅ LIVE
 │   ├── neural_api.get_topology ─── primal connections ....... ✅ LIVE
 │   ├── neural_api.get_proprioception ── self-awareness ...... ✅ LIVE
 │   └── capability.call ────────────── routing ............... ✗  not in this binary version
 │
 ├── Direct Primal: BearDog (Security)
 │   └── health ──────────────── crypto status ................ ✅ LIVE (v0.9.0)
 │
 ├── Direct Primal: ToadStool (Compute)
 │   ├── toadstool.health ───── GPU status ................... ✅ LIVE (v0.1.0)
 │   ├── toadstool.version ──── protocol version ............. ✅ LIVE
 │   └── compute.execute ────── workload dispatch ............ ✗  not for physics (use barracuda)
 │
 ├── Direct Primal: Squirrel (AI)
 │   └── squirrel.health ────── AI bridge status ............. ✅ LIVE
 │
 ├── Direct Primal: Songbird (Network)
 │   └── songbird.sock ──────── TCP :3492 + IPC .............. ✅ LIVE
 │
 ├── Nest (NestGate) — binary version mismatch (no `daemon` subcommand)
 │   ├── storage.put/get ─────── not available in this deploy . ⚠  binary needs update
 │   └── data.* providers ────── not available ................ ⚠  binary needs update
 │
 └── Sovereign Fallback
     ├── Local Anderson localization .......................... ✅ always works
     ├── Synthetic weather data (Exp 029) ..................... ✅ always works
     └── Synthetic community data (Exp 030) .................. ✅ always works
```

## Capability Registry Alignment

| groundSpring semantic | biomeOS translation | NestGate actual method | Status |
|----------------------|--------------------|-----------------------|--------|
| `storage.put` | `storage.put` → `storage.store` | `storage.store` | ✅ aligned |
| `storage.get` | `storage.get` → `storage.retrieve` | `storage.retrieve` | ✅ aligned |
| `storage.store` | `storage.store` → `storage.store` | `storage.store` | ✅ alias |
| `storage.retrieve` | `storage.retrieve` → `storage.retrieve` | `storage.retrieve` | ✅ alias |
| `compute.execute` | `compute.execute` → `execute_workload` | ToadStool native | ✅ aligned |
| `compute.submit` | `compute.submit` → `submit_workload` | ToadStool native | ✅ aligned |
| `data.ncbi_search` | → `nestgate.data.ncbi_search` | `data.ncbi_search` | ✅ aligned |
| `data.noaa_ghcnd` | → `nestgate.data.noaa_ghcnd` | `data.noaa_ghcnd` | ✅ aligned |
| `data.iris_stations` | → `nestgate.data.iris_stations` | `data.iris_stations` | ✅ aligned |
| `data.iris_events` | → `nestgate.data.iris_events` | `data.iris_events` | ✅ aligned |

## Experiment Interaction Matrix

| Experiment | NUCLEUS Required | Primals Used | Sovereign Fallback |
|-----------|-----------------|--------------|-------------------|
| Exp 001–028 | No | None (pure local) | N/A |
| Exp 029: GHCND ET₀ | Optional | NestGate (data.noaa_ghcnd) | Synthetic weather |
| Exp 030: NCBI 16S | Optional | NestGate (data.ncbi_search) | Synthetic community |
| Exp 031: NUCLEUS Stack | Required for live | BearDog, Songbird, ToadStool, Squirrel, NestGate | All paths degrade gracefully |
| Exp 032: IRIS Seismic | Optional | NestGate (data.iris_stations, data.iris_events) | Synthetic NMSZ stations |

## Key Architecture Decisions

### 1. Compute through barracuda, not RPC

ToadStool provides GPU primitives (shader dispatch, workload scheduling),
not high-level physics operations. groundSpring uses barracuda directly
for GPU-accelerated computation:

```
groundSpring physics code
    └── barracuda::stats::diversity::chao1_classic()
        └── barracuda WGSL shader dispatch (wgpu)
            └── GPU hardware
```

The Neural API `compute.execute` is for ToadStool-native workloads
(containers, WASM modules, GPU job scheduling), not for forwarding
arbitrary physics computations.

### 2. Sovereign-first design

Every groundSpring experiment works without NUCLEUS. Live data from
NestGate adds ecological realism; NUCLEUS storage adds provenance;
but the science runs regardless.

### 3. Adaptive capability testing

Exp 031 doesn't hardcode which primals must be running — it queries
what's available and validates each path, passing gracefully when
a primal is absent.

## V99 Key Changes (March 8, 2026)

### Evolved Health Check (adaptive multi-method)
- `health()` now tries `neural_api.get_metrics` first (current binary),
  falls back to `topology.metrics` (future alias support).
- Previously hard-coded to `topology.metrics` which failed against live stack.

### Direct Primal Discovery
- `discover_primals()` scans `$XDG_RUNTIME_DIR/biomeos/` for primal sockets.
- `primal_health(name)` checks individual primals directly (not through Neural API).
- `direct_primal_rpc(name, method, params)` bypasses Neural API routing.
- Deduplicates tarpc/jsonrpc socket variants.

### Validation Binary Evolution
- Exp 031 now exercises both Neural API and direct primal paths.
- Phase B2 (Direct Primal Health) validates BearDog, ToadStool, Squirrel directly.
- Phase C/D fall back to direct primal calls when `capability.call` is unavailable.

### Live NUCLEUS Results (Full mode, Family ID `8ff3b864a4bc589a`)
- 4 primals discovered: beardog, songbird, toadstool, squirrel
- Neural API: 3/3 methods respond (metrics, topology, proprioception)
- Direct health: 3/3 primals healthy (beardog v0.9.0, toadstool v0.1.0, squirrel)
- NestGate: binary version mismatch (no `daemon` subcommand) — needs P1 rebuild
- 40/40 NUCLEUS experiment checks PASS (Exp 029, 030, 031, 032)
- 1,123 unit/integration tests PASS (0 fail, `--features biomeos`)

## Next Steps

- [x] Register groundSpring capabilities once Neural API supports it (DONE V118 — 16 capabilities)
- [x] Test `capability.call` after Neural API binary update (DONE V120 — `capability_call_typed`)
- [ ] Update NestGate binary to version with `daemon` subcommand (P1)
- [ ] Wire Exp 029/030 to cache results via NestGate when live
- [ ] Create biomeOS graph pipeline for science workflows (Phase V5)
- [ ] Measure and record interaction latencies for pathway learning (Phase V6)
- [ ] Cross-spring NUCLEUS experiments (wetSpring diversity × groundSpring noise)
