# groundSpring V139 — Upstream Ecosystem Handoff

**Date**: May 12, 2026
**From**: groundSpring (river delta)
**For**: primalSpring coordination, all primal teams, all spring teams
**Context**: Ecosystem Wave Sync response. Tier 2 unblocked. V139 is groundSpring's most composition-ready release.

---

## State Summary

| Metric | Value |
|--------|-------|
| Version | V139 |
| Tests | 1,123 (zero failures, zero clippy, zero unsafe) |
| Experiments | 38 (34 core + 4 LTEE B1-B4) |
| Validation checks | 427/427 PASS |
| Validation binaries | 38 (all with `--format json`) |
| Mathematical parity | 29/29 proven (Python ⇌ Rust) |
| guideStone | Level 4 |
| Tier 4 IPC-first | Compliant (`default = []`) |
| Tier 2 wiring | `toadstool.validate` + `toadstool.list_workloads` + `barracuda.precision.route` |
| barraCuda | v0.4.0 (optional, feature-gated) |
| LTEE | B1-B4 COMPLETE, B6-B9 QUEUED |

---

## What We Ship to Each Audience

### For primalSpring (L2 coordination)

1. **Tier 2 fully wired.** groundSpring is composition-ready with all three Pass 14 methods implemented:
   - `toadstool.validate` — workload pre-flight with graceful degradation
   - `toadstool.list_workloads` — auto-discover available workloads
   - `barracuda.precision.route` — precision tier advisory
2. **`roles::GPU_MATH` constant** added to `primal_names.rs` — barraCuda now has its own role distinct from `roles::COMPUTE` (ToadStool). Other springs may want to adopt this pattern.
3. **Bug fix surfaced**: `ipc/barracuda.rs` had a latent test asserting `roles::COMPUTE == "barracuda"` — wrong (it's `"toadstool"`). Any spring that copied our pre-V139 barracuda IPC stub should audit this.
4. **Deep debt zero**: No TODO/FIXME/HACK in Rust code. No `unsafe`. No `.unwrap()` in library. No mocks in production. No files >800L.

### For toadStool team

1. **We consume `toadstool.validate`**: Params `{workload_path, dry_run}` → Response `{valid, gpu_available, precision_tier, estimated_dispatch_time_ms, warnings, required_capabilities}`. Wired with graceful degradation via `try_validate_workload()`.
2. **We consume `toadstool.list_workloads`**: Wired with `try_list_workloads()`.
3. **Discovery**: Via `roles::COMPUTE` ("toadstool") → 5-tier socket discovery.
4. **Phase D readiness**: When `LocalDeviceFactory` and `try_local_dispatch()` activate, our metalForge workloads are ready.

### For barraCuda team

1. **We consume `barracuda.precision.route`**: Params `{domain, hardware_hint}` → Response `{recommended_tier, fma_safe, requires_compiler, hardware_hint}`. Wired with graceful degradation via `try_precision_route()`.
2. **Discovery**: Via new `roles::GPU_MATH` ("barracuda") → 5-tier socket discovery. This separates barraCuda math primitives from ToadStool orchestration at the role level.
3. **GAP-GS-011 (PRNG Phase 2b)**: Still deferred — `Xorshift64` → `xoshiro128**` migration requires your team's coordination. Our stochastic experiments use statistical validation (not bit-identical), so this isn't blocking.
4. **Hardcoded `"barracuda."` prefix**: Extracted to `LEGACY_COMPUTE_PREFIX` const in `dispatch/mod.rs`. Pre-v0.3.7 callers still supported; new callers should use bare `domain.operation` names.

### For coralReef team

1. **GAP-GS-002**: IPC stub at `ipc/coralreef.rs` with `ShaderCompile` trait (`compile_wgsl`, `targets`, `validate`). Ready to activate when SM rebuild ships. No blocking action on our side.

### For BearDog / Songbird teams

1. **GAP-GS-008 (Ionic runtime)**: Blocked on BearDog `crypto.sign_contract`.
2. **GAP-GS-009 (BTSP session crypto)**: Blocked on BearDog/barraCuda Phase 3.
3. **Songbird discovery**: We use `roles::DISCOVERY` → `find_primals` / `resolve`. Working correctly.

### For NestGate team

1. **Data pipeline stubs**: `ncbi_search`, `ncbi_fetch`, `noaa_ghcnd`, `iris_stations` in `ipc/nestgate.rs`.
2. **Cold seep FASTQ (PRJNA315684)**: Tier 1 dataset documented in `specs/TIER1_DATASET_ACQUISITION.md`. Bulk FASTQ download blocked on NestGate SRA evolution.
3. **Doc gap**: `iris_events` mentioned in `ipc/mod.rs` doc but no method on `DataPipeline` trait — minor doc drift.

### For spring teams (river delta)

1. **Tier 2 pattern reference**: Our `ipc/toadstool.rs` and `ipc/barracuda.rs` show the full pattern: tarpc trait + biomeOS JSON-RPC + `try_*` graceful degradation + `parse_jsonrpc_response()` helper. Copy this for your spring's Tier 2 wiring.
2. **`roles::GPU_MATH` adoption**: If your spring distinguishes ToadStool orchestration from barraCuda math, add `roles::GPU_MATH` to your `primal_names.rs`.
3. **Experiment catalog**: Our `experiments/results/experiment_catalog.json` pattern (with domain mapping, Python/Rust check counts, speedup ratios) may be useful for springs building their own catalog.
4. **LTEE handoff for lithoSpore**: `wateringHole/handoffs/GROUNDSPRING_LITHOSPORE_LTEE_HANDOFF_MAY12_2026.md` has the full B1-B4 ingestion guide.

### For lithoSpore

1. **Modules 1-4 unblocked**: B1 (mutation) → module 2, B2 (fitness) → module 1, B3 (clonal) → module 3, B4 (citrate) → module 4. All have `expected_values.json` with BLAKE3 checksums via `--format json`.
2. **Control directories ready for ingestion**: `control/ltee_fitness_dynamics/`, `control/ltee_neutral_mutation/`, `control/ltee_clonal_interference/`, `control/ltee_citrate_innovation/`.

### For foundation

1. **Thread 5 (LTEE backbone)**: Active, seeded with our B1-B4 data.
2. **Thread 7 (Anderson Math)**: Active. Our 29 measurement baselines feed both threads.
3. **Expected values**: All JSON files in `control/ltee_*/expected_values.json` are validated (Python + Rust).

### For projectNUCLEUS

1. **`--format json` on all 38 binaries**: NDJSON output via `AutoSink`. Ready for Tier 2 pipeline ingestion.
2. **`toadstool.validate` wired**: Workload pre-flight ready for NUCLEUS dispatch graphs.
3. **Workload TOMLs**: 30 metalForge workloads (24 GPU + 2 NPU + 2 CPU-only + 2 sovereign).

---

## Composition Patterns We've Validated

| Pattern | How We Use It |
|---------|---------------|
| **IPC-first (Tier 4)** | `default = []`, all primal calls via `CompositionContext` or IPC stubs |
| **Graceful degradation** | `try_validate_workload()` / `try_precision_route()` / `try_emit_audit_event()` — all return `Ok(None)` on missing primal |
| **Self-knowledge** | `primal_names::SELF_ID`, `roles::*` constants, 5-tier socket discovery |
| **`--format json`** | `AutoSink::from_args()` + `ValidationHarness` for structured NDJSON |
| **Feature gating** | `barracuda` optional, `biomeos` optional, `tarpc-ipc` optional, `certification`/`validation`/`unibin` gated |
| **Eukaryotic UniBin** | Single binary `groundspring_unibin` with `certify`/`validate`/`status`/`version` subcommands |
| **modular guidestone** | 5 NUCLEUS layer modules in `certification/`, 128L entry point (was 833L) |

---

## Open Gaps (all blocked upstream)

| Gap | Owner | Status |
|-----|-------|--------|
| GAP-GS-002 | coralReef | Stub, awaiting SM rebuild |
| GAP-GS-008 | BearDog | Ionic runtime not shipped |
| GAP-GS-009 | BearDog/barraCuda | BTSP Phase 3 not shipped |
| GAP-GS-011 | barraCuda | PRNG xoshiro128** migration deferred |
| LTEE B6-B9 | groundSpring | QUEUED, Exp TBD |
| Cold seep FASTQ | NestGate | SRA pipeline not shipped |
