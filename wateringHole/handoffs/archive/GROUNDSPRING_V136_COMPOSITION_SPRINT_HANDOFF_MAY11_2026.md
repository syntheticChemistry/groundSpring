# groundSpring V136 — Primal Composition Sprint Handoff

**Date**: May 11, 2026
**Version**: V136
**Gate**: Primal Composition Sprint (Interstadial → Stadial)

---

## What Was Done

### LTEE B3 Complete (Good et al. 2017 — Clonal Interference)

Third LTEE reproduction, completing the groundSpring-owned B1/B2/B3 critical path.

| Aspect | Detail |
|--------|--------|
| Paper | Good BH et al. (2017) "Dynamics of molecular evolution" *Nature* 551:45-50 |
| Python baseline | 7/7 PASS — Wright-Fisher with 4 population sizes (100, 1K, 10K, 100K) |
| Rust validator | 7/7 PASS — `validate_ltee_clonal` binary, deterministic Xorshift64 PRNG |
| Key result | Fixation prob monotonically decreases with N; log-fitness rate sublinear (ratio 9.0 for 10× N) |
| Artifacts | `control/ltee_clonal_interference/expected_values.json` → lithoSpore module 3 (`ltee-clonal`) |

### IPC Surface Completion

- **tarpc `GroundSpringScience`**: 8 → 16 methods. Now covers all `niche::CAPABILITIES`. Missing methods added: `uncertainty_budget`, `spectral_features`, `drift`, `band_edge`, `rare_biosphere`, `gillespie`, `bistable`, `quasispecies`.
- **coralReef IPC stub**: New `ipc/coralreef.rs` with `ShaderCompile` trait. Awaiting coralReef SM rebuild.

### Doc Reconciliation (Upstream Audit Response)

Addressed all findings from primalSpring Composition Sprint audit:
- LTEE B1/B2 status: STARTED → **COMPLETE** in PAPER_REVIEW_QUEUE
- Duplicate `#` numbering in completed reproductions table: fixed
- Stale `v0.3.7` → `v0.3.13` across 5 living docs
- Stale `1,101` → `1,125` across 5 living docs
- Stale `V133` → `V135` across 4 living docs

---

## For lithoSpore Team

Three expected values JSONs are now ready for absorption:

| Module | JSON Path | Paper | Checks |
|--------|-----------|-------|--------|
| 1 (`ltee-fitness`) | `control/ltee_fitness_dynamics/expected_values.json` | Wiser 2013 | 9/9 Py, 10/10 Rust |
| 2 (`ltee-mutation`) | `control/ltee_neutral_mutation/expected_values.json` | Barrick 2009 | 8/8 Py, 8/8 Rust |
| 3 (`ltee-clonal`) | `control/ltee_clonal_interference/expected_values.json` | Good 2017 | 7/7 Py, 7/7 Rust |

Integration pattern: parse JSON, validate against lithoSpore's own model output, gate on tolerance thresholds.

---

## For Other Springs

### LTEE Pattern
The B3 reproduction follows the same pattern as B2+B1:
1. Benchmark JSON in `control/ltee_<name>/benchmark_ltee_<name>.json` (provenance, model params, expected results)
2. Python baseline script producing expected_values.json
3. Rust validator binary using `ValidationHarness` + `Xorshift64` PRNG

**PRNG note**: Python uses `numpy.random.default_rng` (PCG64); Rust uses `Xorshift64`. Values will differ — we validate statistical properties, not bit-identical trajectories.

### coralReef IPC
When coralReef ships the SM rebuild, all springs can use the `ShaderCompile` trait pattern from `groundSpring/ipc/coralreef.rs` as a reference for wiring `shader.*` methods.

---

## Remaining Queue

| Item | Status | Notes |
|------|--------|-------|
| LTEE B4 (Blount 2008 — citrate innovation) | QUEUED | Rare event statistics |
| LTEE B6 (BioBricks burden 2024) | QUEUED | Anderson Wc analogy |
| LTEE B8 (Barrick & Waters 2025 — phage contingency) | QUEUED | Bet-hedging statistics |
| coralReef IPC activation | BLOCKED | Awaiting SM rebuild |
| PRNG Phase 2b (GPU seed stride) | BLOCKED | barraCuda team deliverable |

---

## Test Summary

- Rust workspace: **1,125** tests, zero clippy warnings, zero unsafe
- Python provenance: **287** tests
- Validation binaries: **36** (35 + new `validate_ltee_clonal`)
- LTEE reproductions: **B1 COMPLETE, B2 COMPLETE, B3 COMPLETE**
- Provenance registry: **30** benchmarks (was 29)
