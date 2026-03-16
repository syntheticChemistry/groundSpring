# groundSpring V105 — Code Evolution + barraCuda/toadStool Absorption Handoff

**Date**: March 15, 2026
**From**: groundSpring V105 (38 modules, 936 tests, 102 delegations)
**To**: barraCuda / toadStool teams
**Authority**: wateringHole (ecoPrimals Core Standards)
**Supersedes**: GROUNDSPRING_V104_DEEP_DEBT_BARRACUDA_ABSORPTION_HANDOFF_MAR15_2026.md

## Pins

- **barraCuda**: v0.3.5 (path dep `../../../barraCuda/crates/barracuda`)
- **toadStool**: S130+ (latest)
- **coralReef**: Iteration 10+

## Executive Summary

V105 is a deep code evolution pass — modern idiomatic Rust, complete
error handling, smart module refactoring, and typed IPC client implementation.
Key changes:

- **`#![deny(clippy::expect_used, clippy::unwrap_used)]`** enforced across all 3 crates
- **`print_provenance_header`** evolved from panicking to `Result`-returning API
- **`freeze_out.rs`** (715 LOC) smart-refactored into 4 domain-aligned submodules
- **Typed tarpc IPC client** implemented with runtime socket discovery
- **Platform-agnostic paths**: `/tmp` → `std::env::temp_dir()`
- **Env-configurable** node names in metalForge validation binaries
- **Shared Python tolerances** mirror `groundspring::tol` for Rust↔Python parity
- **Provenance automation**: `et0_methods.py` baseline_commit `"pending"` → `git_commit_hash()`

## Quality Gates

| Gate | Status |
|------|--------|
| `cargo fmt --all -- --check` | **PASS** (0 diff) |
| `cargo clippy --workspace --all-targets -D warnings -W pedantic` | **PASS** (0 warnings) |
| `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` | **PASS** (0 warnings) |
| `cargo test --workspace` | **936 passed, 0 failed** |
| `#[allow()]` in production | **0** (test modules use `#[allow()]`; production uses `#[expect(reason)]`) |
| `unsafe` in application code | **forbidden** (`#![forbid(unsafe_code)]`) |
| `#![deny(clippy::expect_used, clippy::unwrap_used)]` | **enforced** (all 3 crate roots) |
| Files > 1000 LOC | **0** (largest: fao56/mod.rs @ 642) |
| Mocks in production | **0** |
| Hardcoded `/tmp` paths | **0** (all use `std::env::temp_dir()`) |
| Hardcoded primal/node names in production | **0** (all env-configurable or `FAMILY_ID`) |

## Part 1: Panic-Free Production Code

All 3 crate roots now enforce `#![deny(clippy::expect_used, clippy::unwrap_used)]`.
Production code is forbidden from using `.unwrap()` or `.expect()` unless explicitly
opted-in with `#[expect(clippy::expect_used, reason = "...")]`.

Validation binary helpers (`f64_field`, `usize_field`, etc.) use `#[expect()]` with
documented reason: "compile-time JSON; missing field is a programmer error". The
underlying `get_*` functions return `Result<T, BenchFieldError>`.

`print_provenance_header` evolved:
- **New**: `try_print_provenance_header()` → `BenchResult<()>` (pure error handling)
- **Convenience**: `print_provenance_header()` wraps with documented `#[expect()]`
- Pattern: Result-returning core + panicking convenience for validation binaries

### Action for barraCuda

Consider adopting `#![deny(clippy::expect_used, clippy::unwrap_used)]` in barraCuda
crate roots. The pattern: deny at crate level, `#[expect(reason)]` for justified
cases, `#[allow()]` only in `#[cfg(test)]` modules.

## Part 2: Smart Module Refactoring

`freeze_out.rs` (715 LOC, the largest file) refactored into 4 domain-aligned submodules:

| Module | LOC | Responsibility |
|--------|-----|----------------|
| `freeze_out/curve.rs` | 78 | Forward model + chi-squared evaluation |
| `freeze_out/grid.rs` | 196 | 2D grid search + L-BFGS refinement (CPU + GPU) |
| `freeze_out/chi2.rs` | 113 | Decomposed chi-squared analysis |
| `freeze_out/nelder_mead.rs` | 138 | Multi-start Nelder-Mead (GPU batched) |
| `freeze_out/mod.rs` | 198 | Types, config, re-exports, tests |

Split philosophy: **domain boundaries, not arbitrary line counts**. Each submodule
owns a complete concept. `cfg(feature)` blocks stay with their domain logic.
Shared types (`GridFitConfig`, `GridFitResult`) live in `mod.rs`.

### Action for barraCuda

When absorbing `freeze_out_scan` (P1.4 from V104), the submodule structure maps
directly to barraCuda ops: `curve` → scalar ops (stay local), `grid` → parallel
dispatch, `chi2` → `barracuda::stats::chi2`, `nelder_mead` → `barracuda::optimize`.

## Part 3: Typed IPC Client

`ipc.rs` now has a complete `GroundSpringClient`:

```rust
// Runtime socket discovery — no hardcoded paths
let client = GroundSpringClient::connect_discovered().await?;
let result = client.anderson_validation(3.5, 1000, "f64".into()).await?;
```

Discovery chain: `GROUNDSPRING_IPC_SOCKET` → `$XDG_RUNTIME_DIR/biomeos/*.sock` →
`<temp_dir>/groundspring-ipc.sock`. Zero hardcoded paths.

Typed methods: `anderson_validation`, `noise_decomposition`, `parity_check`,
`et0_propagation`. All return `IpcResult<String>` with proper error types.

### Action for toadStool

The tarpc client is ready for integration testing when toadStool exposes a
tarpc server endpoint. Current JSON-RPC 2.0 remains the primary protocol;
tarpc activates when both endpoints support it.

## Part 4: Platform Agnosticism

| Before | After |
|--------|-------|
| `PathBuf::from("/tmp")` | `std::env::temp_dir()` |
| `TowerAtomic::new("eastgate")` | `TowerAtomic::new(&env("GROUNDSPRING_TEST_TOWER", "eastgate"))` |
| `NodeAtomic::with_inventory("strandgate", inv)` | `NodeAtomic::with_inventory(&env("GROUNDSPRING_TEST_NODE", "strandgate"), inv)` |
| `"baseline_commit": "pending"` | `"baseline_commit": git_commit_hash()` |

### Action for barraCuda / toadStool

Audit for hardcoded `/tmp` or node names in your codebases. The env-fallback
pattern (`std::env::var("X").unwrap_or_else(|| default.into())`) is the standard.

## Part 5: Shared Python Tolerance Module

`control/common.py` now exports 15 named tolerance constants mirroring
`groundspring::tol`:

```python
TOL_DETERMINISM = 1e-15   # matches groundspring::tol::DETERMINISM
TOL_EXACT = 1e-12         # matches groundspring::tol::EXACT
TOL_ANALYTICAL = 1e-10    # matches groundspring::tol::ANALYTICAL
# ... 12 more tiers
```

Plus `git_commit_hash()` for automated provenance in Python baselines.

### Action for ecosystem

Other springs should adopt the shared tolerance vocabulary. The constant names
and values are now canonical: use `TOL_{TIER}` in Python, `tol::{TIER}` in Rust.

## Delegation Inventory (unchanged from V104)

| Tier | Count | Notes |
|------|-------|-------|
| CPU delegations | 61 | All active |
| GPU delegations | 41 | All active |
| **Total** | **102** | |
| Tier B remaining | 1 | PRNG xorshift64 → xoshiro128** |

## P1 Absorption Opportunities (carried from V104)

1. **Batch uncertainty budget** → `barracuda::stats::uncertainty_budget`
2. **Batch regime classification** → `barracuda::stats::regime_classification`
3. **Spectral reconstruction defaults** → `barracuda::spectral::tikhonov_solve` config
4. **Freeze-out grid scan defaults** → `barracuda::inverse::freeze_out_scan` config

## Part 6: Self-Knowledge Module (`niche.rs`)

New `niche.rs` module (airSpring pattern) — single source of truth for:

- `NICHE_ID` — identity constant (replaces scattered `"groundspring"` literals)
- `CAPABILITIES` (8) — all measurement capabilities
- `SEMANTIC_MAPPINGS` (8) — operation → JSON-RPC method
- `DEPENDENCIES` (4) — required/optional primals with descriptions
- `COST_ESTIMATES` (8) — per-capability timing + GPU hints for Pathway Learner
- `CONSUMED_CAPABILITIES` (13) — what groundSpring calls on other primals
- `DELEGATION_COUNT` — (61 CPU, 41 GPU)
- `FEATURE_GATES` (4) — barracuda, barracuda-gpu, biomeos, npu

`biomeos/mod.rs` constants (`FAMILY_ID`, `MEASUREMENT_DOMAIN`,
`MEASUREMENT_CAPABILITIES`, `MEASUREMENT_MAPPINGS`) now delegate to
`crate::niche::*`. Zero duplication.

### Action for toadStool / barraCuda

The `niche.rs` pattern should become standard for all springs. It makes a
niche self-describing at compile time — biomeOS can introspect capabilities,
dependencies, and scheduling hints without runtime discovery. Consider
promoting the pattern to a shared trait or macro in barraCuda.

## P2: TensorSession Adoption (planned)

Streaming primitive for continuous biomeOS mode. Entry point: fused multi-op
pipelines in `fao56/pipeline.rs` and `bootstrap/`.

## P3: PRNG Alignment (planned)

`prng::Xorshift64` → xoshiro128** when barraCuda absorbs it.

## Learnings for barraCuda / toadStool Evolution

1. **`#![deny(expect_used, unwrap_used)]` catches real bugs.** During this pass,
   we found no actual unwrap-in-production violations — but the enforcement
   prevents future regressions. The deny-at-root + expect-with-reason pattern
   is strictly superior to ad-hoc auditing.

2. **Smart refactoring beats mechanical splitting.** `freeze_out.rs` split along
   domain boundaries (curve/grid/chi2/nelder_mead) rather than arbitrary 300-line
   chunks. Each module owns imports, `cfg(feature)` gates, and tests. This maps
   directly to barraCuda absorption units.

3. **Result-returning core + panicking convenience is the right pattern** for
   validation harness code. Library consumers get `Result`; validation binaries
   get ergonomic `.expect()` with documented `#[expect()]`.

4. **Shared tolerance vocabulary pays compound interest.** Python and Rust now
   share identical named constants. Tolerance debates become "which tier?" not
   "what number?". Recommend barraCuda absorb `tol::*` as a shared crate.

5. **Runtime socket discovery is non-negotiable.** Zero hardcoded paths. The
   env-var fallback chain (`EXPLICIT` → `XDG_RUNTIME_DIR` → `temp_dir()`) works
   on Linux, macOS, and Windows. All primals should converge on this pattern.

6. **`std::env::temp_dir()` over `/tmp`.** Simple but critical for cross-platform
   sovereign compute. macOS uses `/private/tmp`, Windows uses `%TEMP%`.

7. **Self-knowledge modules make niches introspectable.** `niche.rs` holds
   capabilities, dependencies, costs, and feature gates in one file. biomeOS
   can read this at compile time for scheduling and dependency resolution.
   airSpring pioneered the pattern; groundSpring adopted it. All springs should.

## Verification

```bash
cargo fmt --all -- --check       # 0 diff
cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic  # 0 warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps  # 0 warnings
cargo test --workspace           # 936 passed, 0 failed
```

## Files Changed

~120 files across all 3 crates + Python:

- `crates/groundspring/src/niche.rs` — self-knowledge module (capabilities, deps, costs)
- `crates/groundspring/src/lib.rs` — `#![deny(clippy::expect_used, clippy::unwrap_used)]` + `pub mod niche`
- `crates/groundspring-validate/src/lib.rs` — same deny + Result-based provenance
- `metalForge/forge/src/lib.rs` — same deny
- `crates/groundspring/src/freeze_out/` — 4 new submodules (was single 715-LOC file)
- `crates/groundspring/src/ipc.rs` — `GroundSpringClient` with typed methods
- `crates/groundspring/src/biomeos/server.rs` — `temp_dir()`, doc updates
- `crates/groundspring-validate/src/validate_quasispecies.rs` — named tolerances
- `crates/groundspring-validate/src/validate_notill_sampling.rs` — named tolerances
- `metalForge/forge/src/bin/validate_mixed_hardware.rs` — env-configurable names
- `metalForge/forge/src/bin/validate_metalforge_pipeline.rs` — env-configurable names
- `control/common.py` — `git_commit_hash()` + 15 tolerance constants
- `control/et0_methods/et0_methods.py` — automated provenance
- 88 test modules — `#[allow(clippy::unwrap_used)]` annotation
