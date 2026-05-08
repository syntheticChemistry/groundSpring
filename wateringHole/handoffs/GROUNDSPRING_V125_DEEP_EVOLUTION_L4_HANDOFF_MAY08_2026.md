# groundSpring V125 — guideStone Level 4 + Deep Debt Evolution Handoff

**Date**: May 8, 2026
**From**: groundSpring V125
**To**: All primal teams, all spring teams, primalSpring, biomeOS
**barraCuda**: v0.3.13 | **toadStool**: S158+ | **coralReef**: Iteration 55+
**guideStone**: Level 4 (bare + NUCLEUS composition parity)

---

## Executive Summary

groundSpring advances from guideStone Level 3 to **Level 4** — full NUCLEUS
composition parity validated through 2 composition experiment crates (exp094,
exp095), deep Rust debt eliminated, and documentation unified to canonical truth.
This handoff documents: (1) the L3→L4 evolution, (2) deep debt resolved,
(3) patterns learned for all teams, (4) remaining gaps, (5) downstream integration.

**Quality certificate**: 965+ tests, 0 failures, 0 clippy warnings,
0 unsafe, 0 TODO/FIXME, 0 production mocks, 0 hardcoded primal addresses.
29 new `#[cfg(test)]` modules added. 34 notebooks live on primals.eco.
All deploy graph versions, provenance blocks, and capability registrations
synchronized to V125.

---

## 1. guideStone L3 → L4 Evolution

### What changed

| Area | L3 (V124) | L4 (V125) |
|------|-----------|-----------|
| guideStone binary | 5 bare + 6 NUCLEUS IPC | Same — still operational |
| Composition crates | 0 | 2 (exp094_composition_parity, exp095_measurement_niche) |
| Test modules | Partial | 29 new `#[cfg(test)]` modules across freeze_out, stats, dispatch, biomeos, fao56, esn, eps, tol, gpu, ipc |
| Tolerance system | Duplicated (forge + groundspring) | Unified — forge delegates to `groundspring::tol` |
| Logging | Mixed `log` + `tracing` | All `tracing` |
| Platform guards | Partial | `#[cfg(target_os = "linux")]` on all `/run/user/` paths |
| Niche YAML | 8/16 cost_estimates | 16/16 cost_estimates, consumed caps aligned to niche.rs |
| Deploy graphs | V123 provenance, 8-cap registration | V125 provenance, 16-cap registration |
| sporePrint | Not published | 5 notebooks + 29 baselines live on primals.eco |

### Composition experiment crates

- **exp094_composition_parity**: Validates full NUCLEUS composition (Tower +
  Node + Nest + cross-atomic pipeline) via `primalspring` composition API
- **exp095_measurement_niche**: Validates measurement niche registration,
  capability routing, and health contract

---

## 2. Deep Debt Resolved

### A1: Tolerance Unification (GAP-GS-006 Resolved)

`metalForge/forge/src/tolerance.rs` `ToleranceTier::relative_tolerance()` now
delegates to canonical `groundspring::tol` constants instead of hardcoded values.
Single source of truth — `groundspring::tol::{EXACT, ANALYTICAL, STOCHASTIC, QUANTIZED}`.

**Pattern for other springs**: If your forge or GPU layer defines its own
tolerance constants, refactor to import from your library crate's `tol` module.

### A2: Logging Standardization

`metalForge/forge` migrated from `log` to `tracing` (`probe.rs`, `nucleus.rs`).
The `log` dependency is removed from `forge/Cargo.toml`.

**Pattern**: All ecosystem crates should use `tracing` for structured observability.

### A3: Capability-Based Discovery

`validate_nestgate_ncbi.rs` socket registry lookup now uses
`groundspring::primal_names::roles::STORAGE` constant instead of literal
`"nestgate"`. Environment-variable discovery chain is canonical.

**Pattern**: Use `primal_names::roles::*` constants for substring matches,
never literal primal names.

### A4: Platform Guards

`#[cfg(target_os = "linux")]` guard added to `/run/user/` UID enumeration in
`nucleus.rs`. All platform-specific paths are now gated.

### A5: Test Coverage Expansion

29 new `#[cfg(test)]` modules added:
- **freeze_out**: chi2, curve, grid, nelder_mead
- **stats/agreement**: coefficient, hit_rate, efficiency, error_metrics, willmott
- **dispatch**: defaults, extract, lifecycle, measurement
- **biomeos**: registration, compute, health, storage, transport
- **core**: eps, tol, fao56/constants, fao56/pipeline, esn, gpu, ipc

---

## 3. For primalSpring (Upstream)

### Open gaps (docs/PRIMAL_GAPS.md — 6 remaining)

| ID | Gap | Status |
|----|-----|--------|
| GAP-GS-001 | Squirrel not in niche YAML / graphs / CONSUMED_CAPABILITIES | Not started |
| GAP-GS-002 | coralReef not wired (shader compile still internal to barraCuda) | Deferred |
| GAP-GS-003 | barraCuda TensorSession not adopted | Deferred |
| GAP-GS-008 | IONIC-RUNTIME / cross-family GPU lease (BearDog/Songbird) | Blocked upstream |
| GAP-GS-009 | BTSP session crypto for barraCuda IPC | Blocked upstream |
| GAP-GS-011 | PRNG rebaseline (Xorshift64 → xoshiro for GPU alignment) | Tier B deferred |

### Registry cross-sync

groundSpring has 16 `measurement.*` capabilities registered in
`capability_registry.toml`. No CI test yet validates against primalSpring's
canonical 389-method `config/capability_registry.toml`. This is a universal
evolution target for all springs.

### barraCuda optional = true

groundSpring still links `barracuda` as a mandatory path dep (with `optional = true`
in Cargo.toml but `default = ["barracuda"]`). For sovereign NUCLEUS deployment,
the default feature set should not require barraCuda on disk. IPC-first pattern
is the target.

---

## 4. For All Spring Teams

### Patterns to absorb

1. **Tolerance unification**: Define canonical tolerance constants in your library
   crate's `tol` module. Have GPU/forge layers import from there — never duplicate.
2. **`tracing` over `log`**: Standardize on `tracing` for structured observability.
3. **Platform guards**: `#[cfg(target_os = "linux")]` on all `/run/user/`, `/proc/`,
   and similar Linux-only paths.
4. **`primal_names::roles::*`**: Never use literal primal name strings in socket
   lookups or registry queries.
5. **`#[cfg(test)]` modules**: Every `.rs` file should have a test module. Test
   constants, structural properties, error paths — even for modules that need
   live NUCLEUS connections.
6. **sporePrint pattern**: Create `experiments/results/*.json` frozen data, then
   5 notebooks loading that data. Push to main with `notify-sporeprint.yml`.

### Composition experiment replication

primalSpring's `exp095_proto_nucleate_template` is the scaffold for replicating
the composition parity experiment. Each spring should validate Tower + Node + Nest
+ cross-atomic pipeline for their measurement niche.

---

## 5. For Downstream (foundation, sporeGarden, projectNUCLEUS)

### foundation integration (completed)

- `data/targets/thread07_anderson_targets.toml`: 18 groundSpring-specific
  validation targets for Anderson Mathematics
- `deploy/foundation_validate.sh`: Now scans `workloads/groundspring/` in
  addition to `thread*` directories

### sporeGarden integration (completed)

- `workloads/groundspring/`: 4 TOML workloads mirroring foundation
  (gs-validate-all, gs-guidestone, gs-bench-gpu, gs-python-baselines)
- `PHASES.md`: groundSpring added to spring science hubs list

### sporePrint (live)

- 5 notebooks + 29 baselines rendering on primals.eco
- `sporeprint/validation-summary.md` is the canonical stats source

---

## 6. Quality Certificate

| Metric | Value |
|--------|-------|
| Rust tests | 965+ |
| Python provenance tests | 287 |
| Validation checks | 395/395 PASS |
| Clippy warnings | 0 (pedantic + nursery) |
| Unsafe blocks | 0 |
| TODO/FIXME | 0 |
| Library coverage | ≥92% |
| Deploy graphs | 6 (all V125) |
| Composition crates | 2 |
| Notebooks | 34 (5 sporePrint + 29 baselines) |
| barraCuda delegations | 110 (67 CPU + 43 GPU) |
| PRIMAL_GAPS | 5 resolved, 6 remaining |
