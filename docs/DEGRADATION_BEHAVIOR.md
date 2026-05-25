# groundSpring — IPC Degradation Behavior

**Date**: May 25, 2026
**Version**: V146
**Gate**: eastGate (12/12 NUCLEUS ALIVE, Songbird federation :7700)
**Pattern**: `has_capability()` before `call()`. Never gate science behind primal availability.
**Reference**: `infra/wateringHole/PROVENANCE_TRIO_INTEGRATION_GUIDE.md`

---

## Design Principle

groundSpring's 16 `measurement.*` capabilities are **purely local** — they
run deterministic Rust math with no IPC dependencies. All primal interactions
are **enrichment** (provenance, audit logging, precision routing) and degrade
gracefully when primals are unreachable.

---

## Per-Primal Degradation Table

| Primal | Role | When Unreachable | Mechanism | Science Impact |
|--------|------|------------------|-----------|----------------|
| **biomeOS** | Orchestrator | Registration skipped; server still runs | `warn!` in `groundspring_primal.rs` | None — measurement handlers work locally |
| **NestGate** | Storage | `nest.store` / `nest.commit` → `Ok(None)` → legacy path or skip | `try_content_put`, `try_content_get` → `Ok(None)` | None — results computed locally, storage is enrichment |
| **BearDog** | Security | Crypto hash/sign unavailable; audit events skipped | `try_crypto_sign`, `try_crypto_hash_blake3` → `Ok(None)` | None — provenance enrichment only |
| **Songbird** | Discovery | No socket discovery; falls back to env vars / tmp paths | 5-tier discovery chain in `primal_names::discover_socket` | None — direct socket paths still work |
| **ToadStool** | Compute | Workload validation / precision routing unavailable | `try_validate_workload`, `try_precision_route` → `Ok(None)` | None — local math is the default path |
| **barraCuda** | GPU Math | GPU acceleration unavailable; pure Rust fallback | Feature-gated (`barracuda` optional); CPU path always compiled | None — Tier 4 IPC-first means CPU is default |
| **coralReef** | Compiler | Shader compilation unavailable | `try_compile_gemm`, `try_health_version` → `Ok(None)` | None — GPU shaders are acceleration, not correctness |
| **skunkBat** | Audit | JH-5 audit events not recorded | `try_emit_audit_event` → `Ok(None)` | None — audit is non-blocking enrichment |
| **rhizoCrypt** | Provenance DAG | DAG session creation fails | `start_session` → `Err` → `warn!`, validation continues | None — session is enrichment |
| **loamSpine** | Attestation | Provenance attestation skipped | `commit_session` → `Err` → `warn!` | None — attestation is enrichment |
| **sweetGrass** | Attribution | Attribution recording skipped | `record_attribution` → `Err` → `warn!` | None — attribution is enrichment |

---

## Signal Dispatch Degradation

| Signal | Primary Path | Fallback | Ultimate Degradation |
|--------|-------------|----------|---------------------|
| `nest.store` | `ctx.dispatch("nest.store")` | `content.put` via NestGate JSON-RPC | Skip — `Ok(None)` |
| `nest.commit` | `ctx.dispatch("nest.commit")` | `provenance.session_dehydrate` via JSON-RPC | Skip — `warn!`, return `Ok(session_id)` |
| `primal.announce` | `primal.announce` JSON-RPC | `capability.register` + `method.register` legacy | `Err` only if total registrations = 0 |

---

## Registration Degradation

| Step | Behavior on Failure |
|------|---------------------|
| `primal.announce` | Falls back to legacy `capability.register` per-capability |
| `capability.register` (per-cap) | `warn!`; continues to next capability |
| `method.register` (per-method) | `warn!`; continues to next method |
| All registrations fail | `Err(BiomeOsError::Registration)` — server still runs, `warn!` logged |

---

## Provenance Lifecycle Degradation

```
run_lifecycle(socket, experiment_id, result_json)
  ├─ Try nest.store signal dispatch
  │   ├─ Success → return Ok(session_id)
  │   └─ Fail/None → fall through to legacy
  ├─ start_session → Err → propagated (only hard failure)
  ├─ store_result → Err → warn! (non-fatal)
  ├─ commit_session → nest.commit signal → legacy dehydrate → warn!
  └─ record_attribution → Err → warn! (non-fatal)
```

**Key rule**: `start_session` failure is the only hard gate in the legacy
path. Signal dispatch (`nest.store`) bypasses session creation entirely.
When signals are available, no hard gates exist.

---

## Known Gaps

| Gap | Description | Status |
|-----|-------------|--------|
| `resilient_call` unwired | `biomeos/resilience.rs` defines `resilient_call` with retry/circuit-breaker but no IPC module calls it | Deferred — current `try_*` pattern sufficient for enrichment calls |
| Songbird client missing | `ipc/songbird.rs` defines only `tarpc::service` trait; no JSON-RPC client or `try_*` wrapper | Low — Songbird discovery uses env vars / socket probing instead |
| `et0_propagation` dependency metadata | `OPERATION_DEPENDENCIES` previously claimed `data.noaa_ghcnd` — handler is purely local | Fixed (V145) |

---

## Cross-Gate Degradation (Wave 50 Covalent HPC)

When running in a meshed multi-gate environment (Songbird TCP :7700):

| Scenario | Behavior |
|----------|----------|
| Remote gate unreachable | `discovery.peers` returns empty; `capability.call` falls back to local gate | 
| Remote NestGate down | `nest.sync` graph skips replication; local artifacts remain authoritative |
| Cross-subnet peers | Requires router config or TURN relay; Songbird logs unreachable peers as `warn!` |
| Remote barraCuda busy | `toadstool.compute` yields to gate owner; local CPU fallback activates |

**Key rule**: Cross-gate is always enrichment. Local science never depends on
remote gate availability. The `nest.sync` graph is for backup staging, not
correctness.
