# groundSpring Primal Composition Gaps

**Spring:** groundSpring V144
**Proto-nucleate:** `downstream_manifest.toml` (spring_name = "groundspring")
**Particle profile:** balanced (Node + Nest atomic)
**Domain:** geoscience / measurement
**Date:** April 27, 2026
**Last audited:** May 16, 2026 (V144 — Wave 20 schema standardization. `capability.list` canonical envelope. `nest.commit` signal dispatch. Registry 452. 20 IPC methods + 3 signal paths)
**License:** AGPL-3.0-or-later

---

## Purpose

This document tracks gaps discovered during groundSpring's NUCLEUS composition
wiring. Gaps are handed back to primalSpring for ecosystem-wide refinement
via PRs to `primalSpring/docs/PRIMAL_GAPS.md` and `graphs/downstream/`.

---

## Active Gaps

### GAP-GS-001: Squirrel Not in Composition

- **Primal:** Squirrel
- **Severity:** Low
- **Status:** Not started
- **Description:** Squirrel is not in the niche YAML, deploy graph, or
  `CONSUMED_CAPABILITIES`. The `primal_names::roles::ASSISTANT` constant
  exists and a scaffolded test in `biomeos_integration.rs` references
  `ai.health`, but no `ai.*` capabilities are declared or consumed.
  groundSpring's measurement domain is standalone — AI is additive, not
  required. Adding Squirrel requires new niche entries, graph nodes, and
  `CONSUMED_CAPABILITIES` for `inference.*`.
- **Action:** Add Squirrel when neuralSpring native inference matures.
  No blocker in groundSpring code.

### GAP-GS-002: coralReef Shader IPC

- **Primal:** coralReef
- **Severity:** Low
- **Status:** Resolved (May 14, 2026 — compute trio wave)
- **Description:** coralReef (sovereign shader compiler) was listed in
  `depends_on` but not wired via IPC. coralReef FECS stability proof
  shipped (Sprint 7, 4,790 tests).
- **Resolution:** Full `coralReef` surface wired in `ipc/coralreef.rs`:
  `shader.compile.wgsl` (source, target, sm_version),
  `shader.compile.gemm` (m, n, k, precision, arch — Sprint 11 tensor-core),
  `shader.targets`, `shader.validate`, `health.version` (trio-consistent).
  All with biomeOS JSON-RPC helpers and `try_*` graceful degradation.

### GAP-GS-003: TensorSession Not Adopted

- **Primal:** barraCuda
- **Severity:** Low
- **Status:** Deferred
- **Description:** barraCuda's `TensorSession` fused multi-op pipeline API
  is not yet used in groundSpring. The codebase fuses at the individual op
  level (`FusedMapReduceF64`, seasonal pipeline types). Adopting
  TensorSession would enable fused GPU dispatch for multi-step measurement
  pipelines (e.g. decompose + bootstrap + rarefaction as a single session).
- **Action:** Monitor barraCuda TensorSession stabilization; wire when
  stable for measurement workloads.

### GAP-GS-004: Niche YAML Capability Drift

- **Primal:** groundSpring (self)
- **Severity:** Medium
- **Status:** Resolved (April 27, 2026)
- **Description:** `niches/groundspring-measurement.yaml` listed 8
  capabilities while `niche.rs` declares 16. The deploy graph
  `capabilities_provided` also listed only 8 + health.
- **Resolution:** Synchronized YAML to 16 capabilities, deploy graph
  `capabilities_provided` to 16 + 2 health, tower bootstrap registration
  to 16 + 2 health.

### GAP-GS-005: Deploy Graph Verb Mismatch

- **Primal:** groundSpring (self)
- **Severity:** Medium
- **Status:** Resolved (April 27, 2026)
- **Description:** Several deploy graphs had method name inconsistencies.
- **Resolution:** Fixed `measurement.validate_suite` → `measurement.parity_check`,
  `measurement.parity_report` → `measurement.uncertainty_budget`,
  `storage.store` → `storage.put`, `registry.register` → `capability.register`.
  All graph verbs now match actual IPC contracts.

### GAP-GS-006: metalForge Tolerance System Duplication

- **Primal:** groundSpring (self)
- **Severity:** Low
- **Status:** Resolved (May 8, 2026)
- **Description:** `metalForge/forge/src/tolerance.rs` defines a
  `ToleranceTier` enum with numeric table that parallels
  `groundspring::tol` but uses a completely separate type hierarchy.
  Neither imports from the other. This creates a maintenance risk where
  tolerance tiers could drift between the library and forge.
- **Resolution:** Unified — forge `ToleranceTier::relative_tolerance()` now
  delegates to `groundspring::tol::{EXACT, ANALYTICAL, STOCHASTIC, QUANTIZED}`.
  A new `tol::QUANTIZED` constant was added for the NPU int8 tier.

### GAP-GS-007: barraCuda Version Documentation Drift

- **Primal:** barraCuda
- **Severity:** Low
- **Status:** Resolved (April 27, 2026)
- **Description:** Active specs and deploy graphs referenced barraCuda
  v0.3.7 while path dependency resolves to v0.3.12.
- **Resolution:** Updated all active graph STATUS headers and
  `specs/BARRACUDA_REQUIREMENTS.md` to v0.4.0. Historical `tol.rs`
  contract pins retain original version annotations (they document when
  the contract was established).

### GAP-GS-008: IONIC-RUNTIME Cross-Family GPU Lease

- **Primal:** BearDog / Songbird
- **Severity:** Medium (ecosystem-wide)
- **Status:** Blocked upstream
- **Description:** The proto-nucleate documents ionic bonding for cross-
  FAMILY_ID GPU lease. BearDog's `crypto.sign_contract` and ionic
  propose/accept/seal protocol are not yet implemented.
- **Upstream ref:** `primalSpring/docs/PRIMAL_GAPS.md` IONIC-RUNTIME item.

### GAP-GS-009: BTSP-BARRACUDA-WIRE Session Crypto

- **Primal:** barraCuda / BearDog
- **Severity:** Medium (ecosystem-wide)
- **Status:** Blocked upstream
- **Description:** barraCuda session creation does not yet use full BTSP
  stream encryption (Phase 3). groundSpring's tensor work is in-process
  via Rust crate import, so this gap only affects multi-process barraCuda
  IPC scenarios.
- **Upstream ref:** `primalSpring/docs/PRIMAL_GAPS.md` BTSP-BARRACUDA-WIRE.

### GAP-GS-010: `biomeos::compute::compute_capabilities()` Wrong Capability

- **Primal:** groundSpring (self)
- **Severity:** Medium
- **Status:** Resolved (April 27, 2026)
- **Description:** `biomeos/compute.rs` `compute_capabilities()` called
  `resource.health.check` instead of a compute capabilities listing API.
- **Resolution:** Fixed to call `compute.capabilities` which matches the
  ToadStool compute provider's actual API.

### GAP-GS-011: PRNG Rebaseline for GPU Alignment

- **Primal:** barraCuda
- **Severity:** Low
- **Status:** Tier B — deferred
- **Description:** groundSpring uses `Xorshift64` PRNG. GPU alignment
  requires migration to xoshiro128** to match barraCuda's WGSL PRNG.
  This requires a full rebaseline of all 29 stochastic experiments since
  determinism tests depend on the current RNG state sequence.
- **Action:** Coordinate with barraCuda team. Execute rebaseline when
  PRNG migration is ecosystem-wide priority.

### GAP-GS-013: primalSpring LIVE_SCIENCE_API.md `precision.route` Status Contradiction

- **Primal:** primalSpring (documentation)
- **Severity:** Low
- **Status:** Surface upstream
- **Description:** `primalSpring/docs/LIVE_SCIENCE_API.md` line 184 lists
  `barracuda.precision.route` as **NOT IMPLEMENTED**, but the Tier 2
  Convergence Wave blurb (May 13, 2026) says **IMPLEMENTED (649 tests)**.
  One of these is stale. groundSpring's wire assumes IMPLEMENTED.
- **Action:** Handback to primalSpring for doc reconciliation.

### GAP-GS-014: DOWNSTREAM_PATTERN_GUIDE Missing groundSpring B4

- **Primal:** primalSpring (documentation)
- **Severity:** Low
- **Status:** Surface upstream
- **Description:** `primalSpring/docs/DOWNSTREAM_PATTERN_GUIDE.md` lists
  groundSpring LTEE as "B1-B3 DONE" with 1,125 tests. Actual: B1-B4
  DONE (Exp 039 citrate innovation), 1,123 tests.
- **Action:** Handback to primalSpring for doc update.

### GAP-GS-015: primalSpring `routing` Module Visibility Bug

- **Primal:** primalSpring
- **Severity:** Medium (blocks `cargo check --workspace`)
- **Status:** Resolved (May 16, 2026 — Wave 17)
- **Description:** `primalSpring/ecoPrimal/src/coordination/mod.rs:315`
  referenced `crate::composition::routing::capability_to_primal` but
  `composition/mod.rs:42` declared `mod routing;` (private).
- **Resolution:** primalSpring Wave 17 re-exports `ALL_CAPS`, `BTSP_EXTRA_CAPS`,
  `capability_to_primal`, `capability_to_primal_typed`, `method_to_capability_domain`
  from `composition/mod.rs`. `cargo check --workspace` passes. Verified May 16, 2026.

### GAP-GS-016: plasmidBin Manifest Metadata Stale

- **Primal:** plasmidBin (infra)
- **Severity:** Low
- **Status:** Surface upstream
- **Description:** `infra/plasmidBin/manifest.toml` lists groundSpring
  with `tests = 1050` (actual: 1,123), `latest = "0.1.0"` (stale).
  `niche-groundspring` omits `skunkBat` while `atomics.nucleus` includes
  it (10 primals). `barracuda_depth = "calling"` is unique and unclear.
- **Action:** Handback to primalSpring for manifest reconciliation.

### GAP-GS-017: wateringHole README Stale groundSpring Row

- **Primal:** wateringHole (infra)
- **Severity:** Low
- **Status:** Surface upstream
- **Description:** `infra/wateringHole/README.md` still lists groundSpring
  as V135, 1,125 tests, "coralReef IPC, PRNG GPU alignment deferred".
  Actual: V142, 1,123 tests, coralReef IPC fully wired (5 methods),
  20 IPC methods across 7 primals.
- **Action:** Handback to primalSpring for table refresh.

---

## Resolved Gaps

| ID | Description | Resolution | Date |
|----|-------------|------------|------|
| GAP-GS-004 | Niche YAML capability drift (8→16) | YAML + deploy graph + tower bootstrap synced | Apr 27, 2026 |
| GAP-GS-005 | Deploy graph verb mismatches | All verbs fixed to match IPC contracts | Apr 27, 2026 |
| GAP-GS-006 | metalForge tolerance duplication | Unified via `groundspring::tol` delegation | May 8, 2026 |
| GAP-GS-007 | barraCuda version refs (0.3.7→0.3.13) | Active specs/graphs updated | Apr 27, 2026 |
| GAP-GS-010 | compute_capabilities() wrong capability | Fixed to `compute.capabilities` | Apr 27, 2026 |
| GAP-GS-002 | coralReef not wired via IPC | Full surface: `compile.wgsl`, `compile.gemm`, `targets`, `validate`, `health.version` | May 14, 2026 |
| GAP-GS-015 | primalSpring `routing` module private | Re-exported via `composition/mod.rs`, workspace builds pass | May 16, 2026 |
| GAP-GS-012 | `barracuda.rs` test asserted `roles::COMPUTE == "barracuda"` | Added `roles::GPU_MATH`, fixed tests, Tier 2 methods | May 12, 2026 |

---

## Handback Protocol

1. Document gap in this file with severity and upstream reference.
2. If the gap requires primal evolution: PR to `primalSpring/docs/PRIMAL_GAPS.md`.
3. If the gap requires graph evolution: PR to `primalSpring/graphs/downstream/`.
4. If the gap surfaced a new pattern: handoff to `ecoPrimals/infra/wateringHole/handoffs/`.
