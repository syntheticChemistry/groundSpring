# Primal Interaction Evolution

**Last updated**: February 28, 2026 (V0 — first live NUCLEUS interaction from groundSpring)

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
| V5 | Graph pipelines | planned | biomeOS graph executor for multi-step science workflows |
| V6 | Pathway learning | planned | biomeOS learns optimal primal routing from usage patterns |

## Live NUCLEUS Interaction Map (V4)

```
groundSpring
 ├── biomeos::auto_connect()
 │   └── /run/user/1000/biomeos/neural-api.sock
 │
 ├── Tower (BearDog + Songbird)
 │   ├── topology.metrics ────── health check ................. ✅ LIVE
 │   ├── beacon.get_id ────────── beacon identity (48 bytes) .. ✅ LIVE
 │   └── crypto.hash ──────────── blake3 hash ................. ⚠  forward fails (params format)
 │
 ├── Node (ToadStool)
 │   ├── compute.health ───────── GPU status .................. ✅ LIVE
 │   ├── compute.capabilities ─── capability list (641 bytes) . ✅ LIVE
 │   ├── compute.version ──────── protocol version ............ ✅ LIVE
 │   └── compute.execute ──────── workload dispatch ........... ✗  not for physics (use barracuda)
 │
 ├── Squirrel (AI)
 │   └── ai.health ────────────── AI status (150 bytes) ....... ✅ LIVE
 │
 ├── Nest (NestGate) — requires Nest/Full NUCLEUS mode
 │   ├── storage.put ──────────── store validation results .... ○  not registered
 │   ├── storage.get ──────────── retrieve cached data ........ ○  not registered
 │   ├── data.ncbi_search ─────── NCBI SRA queries ........... ○  not registered
 │   ├── data.noaa_ghcnd ──────── NOAA weather data .......... ○  not registered
 │   ├── data.iris_stations ───── IRIS seismic stations ....... ○  not registered
 │   └── data.iris_events ─────── IRIS earthquake events ...... ○  not registered
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

## Next Steps

- [ ] Start NUCLEUS in Nest or Full mode to exercise NestGate storage/data
- [ ] Test BearDog crypto with correct params format for blake3_hash
- [ ] Wire Exp 029/030 sovereign data to also cache results via NestGate when live
- [ ] Create biomeOS graph pipeline for science workflows (Phase V5)
- [ ] Measure and record interaction latencies for pathway learning (Phase V6)
