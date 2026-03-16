# groundSpring V107 — License + Niche + Tolerance Provenance Handoff

**Date**: March 16, 2026
**From**: groundSpring V107 (39 modules, 906 tests, 102 delegations)
**To**: barraCuda / toadStool / biomeOS teams
**Authority**: wateringHole (ecoPrimals Core Standards)
**Supersedes**: GROUNDSPRING_V106_TYPED_ERRORS_PRIMAL_NAMES_HANDOFF_MAR16_2026.md

## Pins

- **barraCuda**: v0.3.5 (path dep `../../../barraCuda/crates/barracuda`)
- **toadStool**: S155b (latest)
- **coralReef**: Iteration 49+

## Executive Summary

V107 aligns groundSpring with the latest ecosystem standards and absorbs
patterns from the cross-spring review (ludoSpring V19, wateringHole):

- **License: AGPL-3.0-only** (302 files migrated from `-or-later`)
- **Release profile optimization** (`lto`, `codegen-units = 1`, `strip`)
- **Enriched niche.rs** with structured `OperationDeps` + `CostEstimate` (ludoSpring pattern)
- **Tolerance provenance** — all 13 `tol::` constants with mathematical derivation,
  source citations, and validation binary references
- **Bare literal elimination** — ~15 named constants extracted from 5 production files
- **Zero dead code warnings** — feature-gated constants for `#[cfg]`-dependent paths

## Quality Gates

| Gate | Status |
|------|--------|
| `cargo fmt --all -- --check` | **PASS** (0 diff) |
| `cargo clippy --workspace --all-targets -D warnings -W pedantic` | **PASS** (0 warnings) |
| `cargo test --workspace` | **906 passed, 0 failed** |
| `#[allow()]` in production | **0** |
| `unsafe` in application code | **forbidden** (`#![forbid(unsafe_code)]`) |
| `#![deny(clippy::expect_used, clippy::unwrap_used)]` | **enforced** (all 3 crate roots) |
| Files > 1000 LOC | **0** (largest: three_tier_parity_gpu.rs @ 705) |
| Mocks in production | **0** |
| Hardcoded primal name strings in production | **0** |
| TODO/FIXME/HACK in .rs source | **0** |
| Bare numeric literals in production | **0** (all named or in `tol::`/`eps::`) |
| License | **AGPL-3.0-only** (ecosystem aligned) |
| proptest property tests | **14** |

## Part 1: License Alignment

All 302 files migrated from `AGPL-3.0-or-later` to `AGPL-3.0-only`, matching
the wateringHole STANDARDS_AND_EXPECTATIONS and the Squirrel v0.1.0-alpha.3
alignment. SPDX headers and `Cargo.toml` `license` field updated consistently.

## Part 2: Release Profile

```toml
[profile.release]
lto = true
codegen-units = 1
strip = true
```

Validation binaries now produce smaller, faster binaries. Pattern adopted from
rhizoCrypt's release profile optimization.

## Part 3: Enriched Niche Self-Knowledge

Two new `const fn` functions follow ludoSpring V19's enriched niche pattern:

```rust
pub const fn operation_dependencies() -> &'static [OperationDeps] { ... }
pub const fn cost_estimates() -> &'static [CostEstimate] { ... }
```

`OperationDeps` declares required/optional inputs and consumed capabilities
per method. `CostEstimate` provides scheduling metadata (estimated_ms,
gpu_beneficial, peak_memory_bytes, deterministic). biomeOS Pathway Learner
uses these for input validation and resource allocation.

## Part 4: Tolerance Provenance Citations

All 13 `tol::` constants now carry:
- Mathematical derivation (why this value, not another)
- Source citation (Abramowitz & Stegun, Hansen, IEEE 754, CLT)
- Validation binary reference (which experiment validates this tier)

Example:
```rust
/// CDF/erf approximation (A&S 7.1.26, two-layer composition).
///
/// Provenance: Abramowitz & Stegun formula 7.1.26 has max error
/// 1.5e-7; our chi² CDF compounds erf twice, giving ~1e-6.
/// Source: Abramowitz & Stegun (1964), §7.1.26.
/// Validated: `validate_decompose`, `validate_freeze_out`.
pub const CDF_APPROX: f64 = 1e-6;
```

## Part 5: Bare Literal Elimination

| File | Constants Extracted |
|------|--------------------|
| `tissue_anderson/compartments.rs` | 9 (cell fractions, infiltration slopes) |
| `tissue_anderson/geometry.rs` | 2 (on-site energies) |
| `anderson/spectral.rs` | 3 (phase thresholds, peak prominence) |
| `multisignal.rs` | 1 (default half-saturation) |
| `bistable.rs` | 1 (default half-saturation) |

## Learnings

1. **License `-or-later` → `-only`** is a one-line `sed` across 302 files, but the
   compliance signal matters for downstream consumers and package managers.
2. **Tolerance provenance** turns constants into documentation — reviewers can
   now trace any tolerance from its value back to the mathematics and the experiment.
3. **`const fn` niche metadata** enables compile-time evaluation for biomeOS scheduling.
4. **Feature-gating constants** eliminates dead-code warnings without suppression attributes.

## For barraCuda/toadStool

No new absorption requests this version. The V105/V106 absorption list
(102 delegations, 14 GPU-promotable modules) remains active.

## Ecosystem Alignment Status

| Standard | groundSpring V107 | Ecosystem |
|----------|-------------------|-----------|
| License | AGPL-3.0-only | Aligned |
| Edition | 2024 | Aligned (with Squirrel, petalTongue) |
| unsafe | `#![forbid(unsafe_code)]` | Aligned |
| Lints | pedantic + nursery, 0 warnings | Aligned |
| niche.rs | Yes (enriched) | Aligned (airSpring, ludoSpring, neuralSpring) |
| primal_names | Yes | Aligned (wetSpring V119) |
| Typed errors | Yes (BiomeOsError enum) | Aligned (petalTongue v1.6.3) |
| proptest | Yes (14 tests) | Aligned (sweetGrass, ludoSpring) |
| Release profile | Yes (lto + strip) | Aligned (rhizoCrypt) |
