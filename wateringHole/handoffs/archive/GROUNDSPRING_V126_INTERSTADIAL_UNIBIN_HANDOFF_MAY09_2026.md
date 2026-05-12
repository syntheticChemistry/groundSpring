# groundSpring V126 — Interstadial Eukaryotic UniBin Handoff

**Date**: May 9, 2026
**From**: groundSpring V126
**To**: All primal teams, all spring teams, primalSpring, biomeOS
**barraCuda**: v0.3.13 | **toadStool**: S158+ | **coralReef**: Iteration 55+
**primalSpring**: v0.9.25 (pinned)
**guideStone**: Level 4, Tier 1 → Tier 2 target

---

## Executive Summary

groundSpring completes its eukaryotic evolution in response to the primalSpring
v0.9.25 interstadial primordial extinction wave. This is the first delta spring
to absorb the full UniBin pattern:

- **certification/ organelle**: Guidestone properties 1-5 (bare) + Layers 2-4
  (NUCLEUS composition parity) absorbed as a library module.
- **validation/scenarios/ registry**: 10 scenarios across 10 tracks with
  `ScenarioMeta` provenance, `Tier` filtering (Rust/Live), and `Track` taxonomy.
- **groundspring_unibin binary**: Single deployable with `certify`, `validate`,
  `status`, `version` subcommands.
- **IPC tree**: `src/ipc/` with per-primal modules (barraCuda, ToadStool,
  NestGate, BearDog, Songbird).
- **Fossil record**: 3 dated prokaryotic snapshots in `fossilRecord/`.

**Quality certificate**: 1,006+ tests, 0 failures, 0 clippy warnings (lib),
0 unsafe, 0 bare `#[allow]`/`#[expect]` without reason, 0 TODO/FIXME/HACK/DEBT.

---

## What Changed (V125 → V126)

### New Modules

| Module | Feature | Purpose |
|--------|---------|---------|
| `src/certification/mod.rs` | `certification` | Certification engine entry point |
| `src/certification/bare.rs` | `certification` | Properties 1-5, no IPC |
| `src/certification/composition.rs` | `certification` | Layers 2-4, live NUCLEUS |
| `src/validation/mod.rs` | `validation` | Scenario framework |
| `src/validation/scenarios/registry.rs` | `validation` | ScenarioMeta + Registry |
| `src/validation/scenarios/s_*.rs` | `validation` | 10 absorbed scenarios |
| `src/ipc/mod.rs` | `tarpc-ipc` | IPC module root |
| `src/ipc/barracuda.rs` | `tarpc-ipc` | ComputeDispatch trait |
| `src/ipc/toadstool.rs` | `tarpc-ipc` | OrchestrationService trait |
| `src/ipc/nestgate.rs` | `tarpc-ipc` | StorageService + DataPipeline |
| `src/ipc/beardog.rs` | `tarpc-ipc` | CryptoService trait |
| `src/ipc/songbird.rs` | `tarpc-ipc` | DiscoveryService trait |
| `src/ipc/client.rs` | `tarpc-ipc` | GroundSpringClient |
| `src/ipc/discovery.rs` | `tarpc-ipc` | Socket discovery |
| `src/bin/groundspring_unibin.rs` | `unibin` | UniBin CLI |

### New Documents

| Document | Purpose |
|----------|---------|
| `PRIMAL_PROOF_IPC_MAPPING.md` | 30+ barracuda:: → JSON-RPC mappings |
| `docs/VALIDATION_TIERS.md` | Two-tier validation model |
| `fossilRecord/README.md` | Fossil record index |
| `fossilRecord/*/README.md` | Per-fossil provenance |

### Lint Fixes

- 3 `#![expect]` without `reason` → fixed with domain-specific reasons

---

## Validation Tier Status

### Tier 1 (Rust — CI-Safe)

9 scenarios covering all measurement domains:
- noise-decomposition, ecology, condensed-matter, agricultural-science,
  statistical-fitting, dynamical-systems, geophysics, population-genetics,
  resampling

All pass without IPC or deployed primals.

### Tier 2 (Live — NUCLEUS Required)

1 scenario (composition-parity) exercising full NUCLEUS via
`CompositionContext::from_live_discovery_with_fallback()`.

Certification L0 (bare) passes. L2-L4 require deployed primals and produce
skip (not fail) when primals are unavailable.

---

## For Other Spring Teams

### Pattern to Replicate

1. **Pin primalSpring v0.9.25** — path dep in Cargo.toml
2. **Create `certification/`** — absorb your guidestone as a library module
3. **Create `validation/scenarios/`** — absorb representative experiments with ScenarioMeta
4. **Create unibin binary** — certify/validate/status/version subcommands with clap
5. **Expand IPC tree** — per-primal modules in `src/ipc/`
6. **Fossilize** — snapshot prokaryotic binaries to `fossilRecord/`
7. **Create `PRIMAL_PROOF_IPC_MAPPING.md`** — map library calls to JSON-RPC

### groundSpring-Specific Innovations

- **Optional barracuda feature gate**: `default = ["barracuda"]` with path dep.
  This is the transitional pattern for sovereign NUCLEUS deployment.
- **10-track domain taxonomy**: Domain-specific tracks (ecology, condensed-matter,
  etc.) for filtered validation.
- **1,006+ test count**: Comprehensive coverage across all measurement domains.

---

## Remaining Gaps (V126)

| Gap | Blocked On | Priority |
|-----|-----------|----------|
| Tier 2 → Tier 3 (full primal-proof IPC parity) | primal-proof feature implementation | Next wave |
| 22 remaining prokaryotic binaries | Scenario absorption (future waves) | Medium |
| Live deploy graph validation | plasmidBin deployment | Tier 2 exercise |
| Registry cross-sync test | primalSpring capability_registry.toml | Medium |

---

## License

AGPL-3.0-or-later
