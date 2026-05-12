# groundSpring V137 — Downstream Seeding Sprint Handoff

**Date**: May 12, 2026
**Version**: V137
**Gate**: Downstream Seeding Sprint (projectNUCLEUS Tier 2 + foundation)

---

## What Was Done

### `--format json` on All Validation Binaries

All 37 `validate_*` binaries now support `--format json` for structured NDJSON output, enabling projectNUCLEUS Tier 2 pipeline ingestion.

| Binary | Default | `--format json` |
|--------|---------|-----------------|
| `validate_ltee_fitness` | `[PASS] label: detail` | `{"type":"check","status":"pass","label":"...","detail":"..."}` |
| `validate_ltee_neutral` | same | same |
| `validate_ltee_clonal` | same | same |
| *(all 34 others)* | same | same |

**Implementation**: `AutoSink` enum in `validate/sink.rs` wraps `WriteSink<Stdout>` (text) or `NdjsonSink<Stdout>` (JSON). `ValidationHarness::from_args()` checks `std::env::args()` for `--format json` or `--format=json`. Zero runtime cost in default text mode. No trait objects.

**Usage**:
```bash
# Default human-readable output
cargo run --bin validate_ltee_clonal

# Structured JSON for projectNUCLEUS
cargo run --bin validate_ltee_clonal -- --format json
```

### Foundation Readiness

All 3 LTEE expected values JSONs confirmed present:

| Module | Path | Paper | Status |
|--------|------|-------|--------|
| 1 (`ltee-fitness`) | `control/ltee_fitness_dynamics/expected_values.json` | Wiser 2013 | Ready |
| 2 (`ltee-mutation`) | `control/ltee_neutral_mutation/expected_values.json` | Barrick 2009 | Ready |
| 3 (`ltee-clonal`) | `control/ltee_clonal_interference/expected_values.json` | Good 2017 | Ready |

Foundation Thread 5 targets can set `validated = true` for these paths.
Thread 7 (Anderson Math) targets reference groundSpring's 29 measurement baselines.

---

## For projectNUCLEUS Team

The `--format json` flag produces one NDJSON line per check:
- `{"type":"check","status":"pass"|"fail","label":"...","detail":"..."}`
- `{"type":"section","name":"..."}`
- `{"type":"summary","text":"..."}`

**Ingestion pattern**: pipe binary output through `jq 'select(.type == "check")'` to extract check results. Exit code is 0 (all pass) or 1 (any fail).

All 37 binaries are in `crates/groundspring-validate/` (35 default + 2 feature-gated). The 3 LTEE binaries (`validate_ltee_fitness`, `validate_ltee_neutral`, `validate_ltee_clonal`) are the immediate candidates for Tier 2 workload wiring.

---

## For lithoSpore Team

No changes from V136 — B1/B2/B3 expected values JSONs are unchanged and ready for BLAKE3-hash ingestion.

---

## For Other Springs

The `AutoSink` + `from_args()` pattern is reusable:
1. Add `AutoSink` to your `validate/sink.rs` (or equivalent)
2. Change `ValidationHarness::stdout(name)` to `ValidationHarness::from_args(name)` in each binary
3. Default type parameter on `ValidationHarness` should be `AutoSink` for bare-type helper function ergonomics

This is the upstream directive for all springs with validation binaries to prepare for Tier 2 ingestion.

---

## Test Summary

- Rust workspace: all tests pass, zero clippy warnings, zero unsafe
- LTEE reproductions: **B1 COMPLETE, B2 COMPLETE, B3 COMPLETE**
- Validation binaries: **37** with `--format json` support
- Foundation: 3/3 expected values JSONs verified
