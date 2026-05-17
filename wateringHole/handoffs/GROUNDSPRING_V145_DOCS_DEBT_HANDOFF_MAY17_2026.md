# groundSpring V145 — Docs Sweep + Deep Debt Resolution Handoff

**Date**: May 17, 2026
**From**: groundSpring
**To**: primalSpring (coordination), delta springs, upstream primals
**Version**: V145 (hardcoded-names evolution + docs cleanup)
**Registry**: 452 methods

---

## What Changed This Cycle

### Code Evolution
- **Hardcoded primal names → role constants**: `certification/composition.rs` and
  `guidestone/tower.rs` replaced string literals (`"security"`, `"discovery"`,
  `"compute"`, `"storage"`) with `primal_names::roles::*` constants. Zero
  runtime coupling to primal identities — only role-based discovery.
- **`#[allow]` with reason**: All bare `#[allow(...)]` attributes now carry
  `reason = "..."` per ecosystem lint standard.

### Documentation Debt Resolved
- **Python test count**: Reconciled 287 → 294 across CONTEXT.md,
  whitePaper/baseCamp/README.md, CONTROL_EXPERIMENT_STATUS.md.
- **PAPER_REVIEW_QUEUE Phase 1 line**: Fixed stale "427/427, 38 experiments"
  to "461/461, 39 experiments". Added B6 to LTEE summary line.
- **B7 (Tenaillon 2016) deferred to Tier 2**: Parallelism statistics require
  real 264-genome data (SRA PRJNA294072). Synthetic simulation produces
  parameter-guessing, not reproduction. Marked as DEFERRED in queue.
- **neuralAPI CAPABILITY_SURFACE.md**: Added `nest.commit` signal documentation
  (was missing — only 2 of 3 signal paths were documented).
- **sporeprint/validation-summary.md**: Fixed "29 validators" → 39, date
  updated to May 17.
- **CONTROL_EXPERIMENT_STATUS.md**: Fixed stale pytest counts (400/314 → 294),
  updated active handoff reference to V145.
- **PRIMAL_GAPS.md**: Date clarified (initial April 27, last audited May 17).

### Deploy Graph Evolution (from Wave 20 debt resolution)
- All 6 NUCLEUS deploy graphs updated: `provenance_commit` node uses
  `nest.commit` as primary capability with `provenance.session_dehydrate`
  as fallback. STATUS headers updated to V145/barraCuda v0.4.0.
- Notebook guideStone level corrected (3 → 4).

---

## Cumulative V145 State

| Metric | Value |
|--------|-------|
| Rust tests | 1,123 |
| Python tests | 294 |
| Experiments | 39 (Exp 001–040, no 034) |
| Validation checks | 461/461 |
| metalForge checks | 138 |
| LTEE reproductions | 5 (B1–B4, B6) |
| lithoSpore modules | 5 (BLAKE3 manifest) |
| IPC methods | 20 |
| Signal dispatch paths | 3 (nest.store, nest.commit, primal.announce) |
| Primals wired | 7 |
| guideStone Level | 4 |
| Deploy graphs | 6 |
| Registry sync | 452 |
| barraCuda version | v0.4.0 |
| primalSpring version | v0.9.25 |
| Unsafe code | 0 |
| Clippy warnings | 0 |
| Production mocks | 0 |
| `.unwrap()` in library | 0 |
| `#[allow]` without reason | 0 |
| Files > 800 lines | 0 |
| TODO/FIXME/HACK markers | 0 |

---

## Patterns for Absorption

### For primalSpring
- **Role constants pattern**: `primal_names::roles::*` gives every spring a
  single source of truth for ecosystem role identifiers. Other springs should
  adopt this to eliminate hardcoded primal name strings in composition code.
- **B7 deferral rationale**: Synthetic parallelism statistics are not meaningful
  without real genomic targets. This applies to any LTEE paper whose core
  result depends on specific gene identities (vs aggregate statistics like
  fitness dynamics or DFE shapes).

### For delta springs
- **`#[allow]` with `reason`**: Ecosystem lint standard. Every bare `#[allow]`
  should carry a reason string explaining why the lint is suppressed.
- **pytest count reconciliation**: If your docs cite pytest counts, verify
  against actual `pytest --co -q | wc -l` — Kokkos benchmark tests inflate
  the "collected" count but fail due to missing binaries.

### For upstream primals
- **NestGate**: `nest.commit` is wired in groundSpring's deploy graphs.
  When NestGate ships session finalization, the signal path is ready.
- **ToadStool**: `toadstool.validate` surface exercised in composition
  certification. groundSpring can be a pre-flight validation consumer
  when workload routing goes live.

---

## LTEE Queue Status

| ID | Paper | Status | Notes |
|----|-------|--------|-------|
| B1 | Barrick et al. 2009 | COMPLETE | lithoSpore module 2 |
| B2 | Wiser et al. 2013 | COMPLETE | lithoSpore module 1 |
| B3 | Good et al. 2017 | COMPLETE | lithoSpore module 3 |
| B4 | Blount et al. 2008/2012 | COMPLETE | lithoSpore module 4 |
| B6 | BioBrick Burden 2024 | COMPLETE | lithoSpore module 5 |
| B7 | Tenaillon et al. 2016 | DEFERRED | Tier 2 — needs real 264-genome data |
| B8 | Barrick & Waters 2025 | QUEUED | Low priority — phage bet-hedging |
| B9 | DFE Evolution 2024 | QUEUED | Medium — pure statistical fitting |

**Next science target**: B9 (DFE fitting) — principled statistical exercise
using groundSpring's existing `model_selection`, `jackknife`, and distribution
infrastructure. No synthetic parameter guessing required.

---

## Remaining Gaps (groundSpring-owned)

| Gap | Owner | Status |
|-----|-------|--------|
| GAP-GS-001: Squirrel not in composition | Upstream (Squirrel) | Blocked |
| GAP-GS-003: TensorSession not adopted | Upstream (neuralSpring) | Deferred |
| Eigenvector side of transport (Paper 17) | groundSpring | GPU-blocked (coralReef) |
| LTEE B7 real data | groundSpring | Tier 2 — awaiting NestGate SRA pipeline |
| LTEE B9 DFE fitting | groundSpring | Next science target |

---

## Archive Summary

This handoff supersedes: `GROUNDSPRING_V145_SCIENCE_EXPANSION_HANDOFF_MAY16_2026.md`
(moved to `archive/`).

Previous handoffs in archive: V123–V144 (see `wateringHole/handoffs/archive/`).
