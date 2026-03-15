# groundSpring V104 — Deep Debt Resolution + barraCuda Absorption Handoff

**Date**: March 15, 2026
**From**: groundSpring team
**To**: barraCuda / toadStool / coralReef teams
**Authority**: wateringHole (ecoPrimals Core Standards)
**Supersedes**: GROUNDSPRING_V103_BARRACUDA_TOADSTOOL_ABSORPTION_HANDOFF_MAR15_2026.md

## Pins

- **barraCuda**: v0.3.5 (local path `../../../barraCuda/crates/barracuda`)
- **toadStool**: S130+ (latest)
- **coralReef**: Iteration 10+

## Executive Summary

V104 is a comprehensive deep-debt resolution pass. All production magic numbers
now have named constants with physical provenance. Hardcoded primal identifiers
are replaced with `FAMILY_ID` constant references and capability-based discovery.
License aligned ecosystem-wide to `AGPL-3.0-or-later` per scyBorg guidance.
Stale version references (V99–V102) updated across 15+ docs. Capability surface
evolved from legacy `science.*` to `measurement.*` domain. Zero warnings across
fmt, clippy (pedantic+nursery), and rustdoc.

## Quality Gates

| Gate | Status |
|------|--------|
| `cargo fmt --all -- --check` | **PASS** (0 diff) |
| `cargo clippy --workspace --all-targets --all-features` | **PASS** (0 warnings) |
| `cargo doc --workspace --all-features --no-deps` | **PASS** (0 warnings) |
| `cargo test --workspace` | **936 passed, 0 failed** |
| `#[allow()]` in production | **0** |
| `unsafe` in application code | **forbidden** (`#![forbid(unsafe_code)]`) |
| Files > 1000 LOC | **0** (largest: freeze_out.rs @ 715) |
| Mocks in production | **0** |

## Delegation Inventory (unchanged)

| Tier | Count | Notes |
|------|-------|-------|
| CPU delegations | 61 | All active |
| GPU delegations | 41 | All active |
| **Total** | **102** | |
| Tier B remaining | 1 | PRNG xorshift64 → xoshiro128** |

## Part 1: Named Constants with Physical Provenance

All production magic numbers extracted to documented constants:

| File | Constants Added | Provenance |
|------|----------------|------------|
| `gpu.rs` | `F64_REDUCTION_SMOKE_TOL` | GPU smoke test sanity threshold (1%) |
| `esn/classifier.rs` | `LYAPUNOV_EXTENDED_THRESHOLD` | Almost-Mathieu λ=0.5 benchmark (Exp 009) |
| `esn/classifier.rs` | `ESN_RESERVOIR_SIZE`, `ESN_SPECTRAL_RADIUS`, `ESN_CONNECTIVITY`, `ESN_LEAK_RATE` | hotSpring Exp 015/022 validation |
| `esn/brain.rs` | `SHARP_BOUNDARY_RATIO`, `BOUNDARY_EXPLORE_FACTOR` | Nautilus Shell `constraints.rs` heuristic |
| `dispatch.rs` | `DEFAULT_ENERGY`, `DEFAULT_CONFIDENCE`, `DEFAULT_ELEVATION_M`, `DEFAULT_RHMAX_PCT`, `DEFAULT_RHMIN_PCT`, `DEFAULT_REGIME_MARGIN` | FAO-56 standard, RMT, Bazavov 2016 |
| `tissue_anderson/drug_scoring.rs` | `MW_BLOCKED_DA` through `PENETRATION_SMALL`, `BIOAVAIL_STRATUM_CORNEUM`, `BARRIER_BLOCK_FACTOR`, `SYSTEMIC_*`, `MIN_REACHABLE_PENETRATION` | Lipinski Rule of Five, topical delivery literature |
| `freeze_out.rs` | `NM_SIMPLEX_SCALE` | Nelder-Mead simplex perturbation (2σ grid cell) |

### Action for barraCuda

The constant provenance pattern (named constant + doc comment citing paper/benchmark)
is now standard across groundSpring. When absorbing batch primitives (P1 below),
carry the provenance comments upstream. Constants like `ESN_READOUT_REGULARIZATION`
should appear in barraCuda's absorbed API with matching documentation.

## Part 2: Capability-Based Discovery

- `dispatch.rs`: `"primal": "groundspring"` → `crate::biomeos::FAMILY_ID`
- `dispatch.rs`: `"name": "groundspring"` → `crate::biomeos::FAMILY_ID`
- `biomeos/server.rs`: socket filename format string uses `FAMILY_ID` constant
- All test assertions updated to reference `FAMILY_ID`, not string literals
- Capability surface doc migrated from legacy `science.*` to `measurement.*`

### Action for toadStool

No toadStool changes needed. groundSpring's self-identification is now
fully constant-driven. toadStool's capability-based dispatch already
handles `FAMILY_ID` correctly at the protocol level.

## Part 3: License Alignment

All code (143 `.rs`/`.toml`/`.wgsl` files) updated from `AGPL-3.0-only` to
`AGPL-3.0-or-later` per scyBorg Provenance Trio Guidance. Cargo.toml workspace
`license` field updated. README, CONTRIBUTING, binary `version` output updated.

### Action for barraCuda / toadStool

Verify your license fields align with scyBorg guidance. barraCuda V035 handoff
explicitly changed TO `-only`; the guidance document says `-or-later`. The
ecosystem should converge. groundSpring and wetSpring now use `-or-later`.

## Part 4: Documentation Debt Resolved

- Broken rustdoc link `[serve]` → `[serve_one]` fixed (zero doc warnings)
- 15+ docs updated from V99/V102 → V104 (specs, whitePaper, neuralAPI)
- Capability surface rewritten: `science.*` → `measurement.*` with correct
  JSON-RPC examples matching the actual dispatch API
- `CONTROL_EXPERIMENT_STATUS.md` active handoff pointer updated
- `scripts/three_mode_benchmark.sh` binary count corrected (27 → 29)
- `.gitignore` updated with `rust_results.json`

## Part 5: Coverage Baseline

First `cargo-llvm-cov` measurement: **74.45% line / 80.95% function** coverage
(workspace total). Hardware-gated metalForge binaries (GPU/NPU required) inflate
the uncovered count. Library code coverage is estimated ~85%+.

## P1 Absorption Opportunities (from V103, still active)

These batch primitives are ready for barraCuda absorption:

1. **Batch uncertainty budget**: `bootstrap_mean` + `jackknife_mean_variance` →
   `barracuda::stats::uncertainty_budget`. Constants: `DEFAULT_CONFIDENCE = 0.95`.

2. **Batch regime classification**: `spectral_features` + `classify_by_spacing_ratio` →
   `barracuda::stats::regime_classification`. Constants: `GOE_R = 0.5307`,
   `POISSON_R = 0.3863`, `LYAPUNOV_EXTENDED_THRESHOLD = 0.005`,
   `ESN_READOUT_REGULARIZATION = 1e-6`.

3. **Spectral reconstruction defaults**: `tikhonov_solve` with manual params →
   `barracuda::spectral::tikhonov_solve` with default config.
   Constants: `DEFAULT_REGULARIZATION = 1e-4`, `DEFAULT_TAU_STEP = 0.1`,
   `DEFAULT_OMEGA_STEP = 0.2`.

4. **Freeze-out grid scan defaults**: `grid_fit_2d` with manual ranges →
   `barracuda::inverse::freeze_out_scan` with config. Constants:
   `DEFAULT_T0_LO/HI/STEP`, `DEFAULT_K2_LO/HI/STEP`, `NM_SIMPLEX_SCALE = 2.0`.

## P2: Streaming Primitive (planned)

Streaming primitive for continuous biomeOS mode to avoid per-call buffer
allocation. Entry point: `TensorSession` adoption for fused multi-op pipelines.

## P3: PRNG Alignment (planned)

`prng::Xorshift64` → xoshiro128** when barraCuda absorbs it. Full rebaseline
of all stochastic experiments needed.

## Learnings for barraCuda / toadStool Evolution

1. **Constant provenance pattern works well.** Named constants with doc comments
   citing paper/benchmark make code self-documenting and audit-friendly.
   Recommend adopting this pattern for all absorbed primitives.

2. **`#[expect()]` with reason is superior to `#[allow()]`.** The Rust 2024 edition's
   `#[expect()]` attribute forces a documented reason string and warns when the
   suppression becomes unnecessary. Recommend migrating barraCuda from any
   remaining `#[allow()]` to `#[expect(reason = "...")]`.

3. **Capability-based discovery eliminates hardcoded names.** groundSpring now has
   zero literal primal name strings in production code. All self-identification
   flows through `FAMILY_ID`. This pattern should be standard for all primals.

4. **Tolerance tiers centralized early pay dividends.** The 12-tier `tol` module
   eliminates tolerance debates in code review. barraCuda should consider
   a shared tolerance vocabulary for absorbed primitives.

5. **`ValidationHarness` hotSpring pattern scales well.** 46 validation binaries
   all follow the same `check_approx/check_range/check_true` → `summary()` →
   `exit(0/1)` pattern. The harness should be promoted to a barraCuda shared
   crate if multiple primals need it.

6. **License alignment needs ecosystem coordination.** The scyBorg guidance says
   `AGPL-3.0-or-later` but some primals use `-only`. A single wateringHole
   decision should resolve this for all primals.

## Verification

```bash
cargo fmt --all -- --check       # 0 diff
cargo clippy --workspace --all-targets --all-features  # 0 warnings
cargo doc --workspace --all-features --no-deps  # 0 warnings
cargo test --workspace           # 936 passed, 0 failed
```

## Files Changed

150 files, 330 insertions, 197 deletions. Key substantive changes:

- `crates/groundspring/src/gpu.rs` — `F64_REDUCTION_SMOKE_TOL`
- `crates/groundspring/src/esn/classifier.rs` — 6 named constants
- `crates/groundspring/src/esn/brain.rs` — 2 named constants
- `crates/groundspring/src/dispatch.rs` — 6 defaults + FAMILY_ID
- `crates/groundspring/src/tissue_anderson/drug_scoring.rs` — 12 named constants
- `crates/groundspring/src/freeze_out.rs` — `NM_SIMPLEX_SCALE`
- `crates/groundspring/src/biomeos/server.rs` — doc link fix + FAMILY_ID
- 143 files — SPDX `AGPL-3.0-only` → `AGPL-3.0-or-later`
- 15+ docs — version refs V99/V102 → V104
- `whitePaper/neuralAPI/CAPABILITY_SURFACE.md` — full rewrite `science.*` → `measurement.*`
