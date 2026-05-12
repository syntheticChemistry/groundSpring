# groundSpring V138 Handoff — Deep Debt V3 + LTEE B4 Citrate Innovation

**Date**: May 12, 2026
**Version**: V138
**From**: groundSpring → all spring teams, primal teams, lithoSpore, projectNUCLEUS, foundation
**Previous**: V137 (Downstream Seeding Sprint)

---

## Summary

V138 completes the fourth LTEE reproduction (B4: Blount 2008/2012 citrate innovation), resolves remaining deep debt items, adds IPC stub test coverage, and synchronizes all documentation to current counts.

## LTEE B4: Blount et al. 2008/2012 Citrate Innovation (Exp 039)

**Scientific question**: Is the evolution of key innovations a single-step process, or does it require historical contingency (potentiating mutations)?

**Model**: Two-hit potentiation-actualization cascade. A population must first acquire a potentiating mutation (rate λ₁) before the actualizing mutation (rate λ₂) can produce the Cit+ phenotype. Replay experiments from earlier evolutionary timepoints demonstrate the contingency effect.

**Results**:
- Python baseline: 8/8 PASS
- Rust validator: 8/8 PASS (`validate_ltee_citrate`, `--format json` supported)
- Historical contingency demonstrated: replay probability from generation 0 = 0.0, increasing monotonically with generation number
- Two-hit analytical waiting time (520K gen) far exceeds single-hit mean (20K gen)
- Cit+ fraction consistent with rare innovation (2/12 at 60K gen with calibrated rates)

**Artifacts**: `control/ltee_citrate_innovation/expected_values.json` → lithoSpore module 4 (`ltee-citrate`)

## LTEE Status (All 4 Complete)

| ID | Paper | Exp | Checks | lithoSpore Module |
|----|-------|-----|--------|-------------------|
| B1 | Barrick 2009 (neutral mutation) | 037 | Py 8/8, Rust 8/8 | module 2 (`ltee-mutation`) |
| B2 | Wiser 2013 (fitness dynamics) | 036 | Py 9/9, Rust 10/10 | module 1 (`ltee-fitness`) |
| B3 | Good 2017 (clonal interference) | 038 | Py 7/7, Rust 7/7 | module 3 (`ltee-clonal`) |
| B4 | Blount 2008/2012 (citrate) | 039 | Py 8/8, Rust 8/8 | module 4 (`ltee-citrate`) |

All 4 `expected_values.json` files are validated and ready for lithoSpore BLAKE3-hash ingestion.

## Deep Debt Cleanup

- **experiment_catalog.json**: Reconciled with LTEE exps 036-039. Total: 38 experiments, 427 checks, 12 domains.
- **validate_nucleus_stack.rs**: Renamed `fake` → `missing_socket` (naming hygiene in validation code).
- **IPC stub unit tests**: Added `#[cfg(test)]` modules to 6 IPC stubs (barracuda, beardog, coralreef, nestgate, songbird, toadstool) — all verify tarpc trait compilation and role constant alignment with `primal_names::roles::*`.

## Documentation Synchronization

15+ files updated V135/V136/V137 → V138:
- CONTEXT.md, SECURITY.md, CONTROL_EXPERIMENT_STATUS.md
- docs/PRIMAL_GAPS.md, docs/VALIDATION_TIERS.md
- specs/README.md, specs/PRIMAL_INTERACTION_EVOLUTION.md, specs/BARRACUDA_EVOLUTION.md, specs/CROSS_SPRING_EVOLUTION.md
- metalForge/ABSORPTION_MANIFEST.md
- whitePaper/README.md, whitePaper/baseCamp/README.md, whitePaper/experiments/README.md
- wateringHole/README.md

Counts aligned: 38 experiments, 427 checks, 38 validation binaries, 1,123 tests, 31 benchmarks, 4 LTEE reproductions.

## Audit Findings (Clean)

| Dimension | Status |
|-----------|--------|
| Files >800L | None (largest 710L) |
| Unsafe code | Zero (all crates `#![forbid(unsafe_code)]`) |
| TODO/FIXME/todo!/unimplemented! | Zero in Rust sources |
| Mocks in production | Zero |
| Dead code | 2 `#[expect(dead_code)]`, both justified |
| Hardcoded primal names | Only `cfg(feature = ...)` (Cargo constraint) |

## For lithoSpore Team

Your modules 1–4 are fully unblocked. groundSpring's `control/ltee_*` directories contain:
- `benchmark_ltee_*.json` (model parameters + expected results)
- `expected_values.json` (computed values from Python baselines)
- Rust `validate_ltee_*` binaries for CI-quality validation

BLAKE3-hash and ingest when ready. No action needed from groundSpring.

## For projectNUCLEUS Team

All 38 `validate_*` binaries support `--format json` (V137 AutoSink). Structured NDJSON output ready for Tier 2 ingestion pipeline. Example:
```
cargo run --release --bin validate_ltee_citrate -- --format json
```

## For Foundation Team

All 4 LTEE `expected_values.json` files confirmed present and valid. Thread 5 (LTEE backbone) and Thread 7 (Anderson Math) targets reference groundSpring paths. Set `validated = true` on foundation targets as lithoSpore confirms ingestion.

## For Upstream Primal Teams

### NestGate (Critical)
- `specs/PRIMAL_INTERACTION_EVOLUTION.md` documents: `storage.put/get` and `data.*` providers "not available in this deploy" — binary needs update. This is the highest-priority upstream gap.
- groundSpring's Exp 029-032 (real GHCND, NCBI, IRIS, NUCLEUS) all have sovereign fallback paths, but full data pipeline requires NestGate content pipeline going live.

### coralReef
- IPC stub created (V136): `ipc/coralreef.rs` with `ShaderCompile` tarpc trait. Awaiting SM rebuild.

### All Primals
- groundSpring qualifies for Tier 4 IPC-first (`default = []`). All primal interactions via `CompositionContext` or `primal_names::discover_socket()`.

## For Sister Springs

### Patterns to absorb:
1. **LTEE reproduction pattern**: `control/<topic>/benchmark_*.json` (parameters) + `<topic>.py` (Python baseline) + `validate_ltee_*.rs` (Rust validator) + `expected_values.json` (output). Follow this for your queue items.
2. **`--format json` via `AutoSink`**: `ValidationHarness::from_args()` auto-selects text/NDJSON output. Pattern in `validate/sink.rs` + `validate/harness.rs`.
3. **IPC stub testing**: Each tarpc trait module gets a `#[cfg(test)]` that verifies the trait compiles and role constants align.

## LTEE Queue (Remaining for groundSpring)

| ID | Paper | Status |
|----|-------|--------|
| B6 | BioBricks burden 2024 *Nat Comms* | QUEUED |
| B7 | Tenaillon 2016 Epistasis *Nature* | QUEUED |
| B8 | Barrick & Waters 2025 Phage contingency | QUEUED |
| B9 | DFE evolution 2024 *Science* | QUEUED |

## Provenance

- Commit: V138 (`3039f63` + docs sync)
- Tests: 1,123 (zero failures)
- Clippy: zero warnings on all targets
- Benchmarks in provenance registry: 31
- Validation binaries: 38 (`validate_*`) + `validate_all` + `groundspring_guidestone`
