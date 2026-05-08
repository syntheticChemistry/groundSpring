# groundSpring Primal Composition Gaps

**Spring:** groundSpring V124
**Proto-nucleate:** `downstream_manifest.toml` (spring_name = "groundspring")
**Particle profile:** balanced (Node + Nest atomic)
**Domain:** geoscience / measurement
**Date:** April 27, 2026
**Last audited:** May 8, 2026 (guideStone L4, 5 gaps resolved, 6 remaining — tolerance unified, composition parity achieved)
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

### GAP-GS-002: coralReef Not Wired

- **Primal:** coralReef
- **Severity:** Low
- **Status:** Deferred
- **Description:** coralReef (sovereign shader compiler) is listed in
  `depends_on` in the downstream manifest but is not wired via IPC.
  barraCuda currently handles shader compilation internally. When
  coralReef's JSON-RPC compile API stabilizes, groundSpring should route
  shader compilation through `shader.*` capability.
- **Action:** Wire when coralReef API stabilizes. Track via
  `specs/BARRACUDA_EVOLUTION.md`.

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
  `specs/BARRACUDA_REQUIREMENTS.md` to v0.3.13. Historical `tol.rs`
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

---

## Resolved Gaps

| ID | Description | Resolution | Date |
|----|-------------|------------|------|
| GAP-GS-004 | Niche YAML capability drift (8→16) | YAML + deploy graph + tower bootstrap synced | Apr 27, 2026 |
| GAP-GS-005 | Deploy graph verb mismatches | All verbs fixed to match IPC contracts | Apr 27, 2026 |
| GAP-GS-006 | metalForge tolerance duplication | Unified via `groundspring::tol` delegation | May 8, 2026 |
| GAP-GS-007 | barraCuda version refs (0.3.7→0.3.13) | Active specs/graphs updated | Apr 27, 2026 |
| GAP-GS-010 | compute_capabilities() wrong capability | Fixed to `compute.capabilities` | Apr 27, 2026 |

---

## Handback Protocol

1. Document gap in this file with severity and upstream reference.
2. If the gap requires primal evolution: PR to `primalSpring/docs/PRIMAL_GAPS.md`.
3. If the gap requires graph evolution: PR to `primalSpring/graphs/downstream/`.
4. If the gap surfaced a new pattern: handoff to `ecoPrimals/infra/wateringHole/handoffs/`.
