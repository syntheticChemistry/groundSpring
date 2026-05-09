# groundSpring V127 — Deep Debt Evolution + Docs Alignment Handoff

**Date**: May 9, 2026
**From**: groundSpring V127
**To**: primalSpring (upstream audit), all spring teams, all primal teams
**Quality**: 1,099 tests, zero clippy warnings on ALL targets (lib + bin + test), zero `fmt` diff, `cargo deny check` PASS
**Ecosystem**: barraCuda v0.3.13, toadStool S158+, coralReef Iteration 55+, primalSpring v0.9.25 pinned

---

## Executive Summary

V127 completes the deep debt cleanup initiated after the V126 eukaryotic UniBin evolution. All technical debt items identified by audit have been resolved or documented. The codebase is now at zero warnings on all targets including test code.

## Changes in V127

### 1. Logging Unification: `log` → `tracing`

Migrated 11 `log::` calls across 5 biomeos files to `tracing::`:
- `biomeos/resilience.rs` — `log::debug!` → `tracing::debug!`
- `provenance.rs` — `log::warn!` → `tracing::warn!`
- `biomeos/server.rs` — `log::error!` → `tracing::error!`
- `biomeos/registration.rs` — `log::info!`/`log::warn!` → `tracing::info!`/`tracing::warn!`
- `biomeos/health.rs` — `log::debug!` → `tracing::debug!`

**Removed `log` crate** from `Cargo.toml` entirely. Zero `log::` calls remain in the codebase.

**Library `eprintln!`** in `certification/composition.rs` converted to `tracing::warn!`/`tracing::info!`.

**Pattern for other springs**: If you still use `log::`, migrate to `tracing::` and remove the `log` dependency. The `tracing` crate is already required by `biomeos` and `certification` features.

### 2. Clippy Zero-Warning on All Targets

Fixed pre-existing warnings that only appeared when compiling test code (`--all-targets`):

- **5 `clippy::unwrap_used`** in `freeze_out/` test modules — added `#[expect(clippy::unwrap_used, reason = "...")]`
- **3 unused variables** in `stats/agreement/coefficient.rs` — moved declarations inside `#[cfg]` blocks
- **12 `clippy::float_cmp`** in test assertions — added `#[expect(clippy::float_cmp, reason = "...")]` where exact zero or constant aliases are compared
- **20+ `clippy::assertions_on_constants`** in tolerance/epsilon/constant guard tests — added `#[expect]` with reason explaining these are compile-time verification of constant ordering
- **2 collapsible-if** in `registry_sync.rs` — collapsed using `.and_then().filter()`
- **1 manual-assert** (panic-in-if) — converted to `assert!(...)`
- **3 `clippy::doc_markdown`** — added backticks to code references in doc comments
- **1 `clippy::cast_precision_loss`** in `exp095` — added `#[expect]` with reason

**Pattern for other springs**: Every `#[expect]` must have `reason = "..."`. Use `#[expect]` rather than `#[allow]` so the lint fires if the suppressed condition goes away.

### 3. Hardcoding Evolution

- `validate_nestgate_ncbi.rs`: Replaced `"localhost"` with named `DEFAULT_LOOPBACK` constant
- All `/run/user/` patterns verified behind `#[cfg(target_os = "linux")]` guards
- All socket discovery chains: env var → XDG → `/run/user/` (Linux) → `temp_dir()` (platform-agnostic)

### 4. Dependency Cleanup

- `log` crate removed (unified to `tracing`)
- `tracing` added to `certification` feature gate
- All remaining external deps verified current for their caret ranges

---

## Audit Findings (Clean)

| Category | Status |
|----------|--------|
| Unsafe code | Zero — `#![forbid(unsafe_code)]` in all crate roots |
| Bare `#[allow]` without reason | Zero |
| `#[deprecated]` without `note` | Zero (1 exists, has note) |
| TODO/FIXME/HACK in Rust source | Zero |
| Production mocks | Zero — only `fake_path` in test modules |
| `log::` usage | Zero — fully migrated to `tracing::` |
| `println!`/`eprintln!` in library | Zero — only in binaries and validation harness |
| Large files (>800L) | `groundspring_guidestone.rs` (833L) already absorbed into `certification/` |
| Hardcoded primal addresses | Zero — all use env-var discovery chains |
| Files >800L needing refactoring | None remaining |

---

## Upstream Primal Debt (for primal teams)

### barraCuda

- **59 files** use `barracuda::` (281 occurrences) — this is the largest primal dependency
- Feature-gated via `default = ["barracuda"]` — the optional pattern other springs should adopt
- `barracuda/gpu` feature has a pre-existing conditional compilation issue: `device` module referenced without `gpu` feature in `tolerances/precision.rs`
- **IPC mapping**: 30+ `barracuda::` library calls documented in `PRIMAL_PROOF_IPC_MAPPING.md` with JSON-RPC equivalents
- **Evolution**: Transition from library calls to IPC-first (`CompositionContext::from_live_discovery_with_fallback()` + `ctx.call()`) is the Tier 3 target

### ToadStool

- 14 files reference ToadStool (36 occurrences), primarily in IPC definitions and metalForge
- `ipc/toadstool.rs` defines `OrchestrationService` trait for compute dispatch
- No blocking debt — clean IPC surface

### NestGate

- 14 files reference NestGate (51 occurrences)
- HTTP API validation in `validate_nestgate_ncbi.rs` with full env-var discovery chain
- `ipc/nestgate.rs` defines `StorageService` and `DataPipeline` traits
- Storage lifecycle (`put/get/list/delete`) exercised in composition tests

### BearDog

- 9 files reference BearDog (15 occurrences)
- `ipc/beardog.rs` defines `CryptoService` trait
- Hash verification used in composition certification (Layer 3 Tower)

### Songbird

- 8 files reference Songbird (17 occurrences)
- `ipc/songbird.rs` defines `DiscoveryService` trait
- Primal resolution exercised in composition certification

---

## Patterns for Downstream Springs to Absorb

### 1. Zero-Warning Clippy on All Targets

groundSpring now passes `cargo clippy --workspace --all-targets` with zero warnings. Key patterns:
- Use `#[expect(lint, reason = "...")]` instead of `#[allow]`
- Scope `#[expect]` to the narrowest item (function, not module)
- For constant-verification tests, use `#[expect(clippy::assertions_on_constants, reason = "...")]`
- For exact-zero f64 comparisons in tests, use `#[expect(clippy::float_cmp, reason = "...")]`

### 2. Unified Tracing

Replace all `log::` with `tracing::`. Remove `log` dependency. The `tracing` ecosystem (subscriber, spans, structured fields) is the ecosystem standard.

### 3. Env-Var Discovery Chain Pattern

```
EXPLICIT_SOCKET > BIOMEOS_SOCKET_DIR > XDG_RUNTIME_DIR > /run/user/ (Linux) > temp_dir()
```

Never hardcode primal addresses. Use named constants for protocol defaults (ports, loopback).

### 4. Tolerance Architecture

groundSpring's 13-tier tolerance system (`groundspring::tol`) is the canonical reference. Other springs should reference these constants rather than defining their own.

---

## Composition Patterns for NUCLEUS

- **CompositionContext** used in 16 files (66 occurrences)
- **`ctx.call()`** used in 4 files (23 occurrences) — certification, guidestone, exp094, exp095
- **Two-tier validation**: Tier 1 (Rust-only, CI-safe) and Tier 2 (live NUCLEUS required)
- **Certification organelle**: `certification/bare.rs` (5 properties) + `certification/composition.rs` (Layers 2-4)
- **Deployment via biomeOS**: `biomeos/server.rs` → Unix socket → JSON-RPC 2.0 → Neural API

---

## Doc Alignment (V127)

All documents aligned to V127 / May 9, 2026 / 1,099 tests:
- README.md, CONTEXT.md, CONTRIBUTING.md, CONTROL_EXPERIMENT_STATUS.md
- CONTROL_RUN_LOG.md, specs/README.md, docs/PRIMAL_GAPS.md
- sporeprint/validation-summary.md, whitePaper/baseCamp/README.md
- wateringHole/README.md, fossilRecord/README.md
- PRIMAL_PROOF_IPC_MAPPING.md (fixed `storage.store` → `storage.put`)
- whitePaper/baseCamp/liu.md (fixed `rawr_mean` delegation status)
- whitePaper/STUDY.md (fixed "28 experiments" → "35 experiments")

---

## Remaining Gaps (for upstream primalSpring audit)

1. **Registry cross-sync**: `capability_registry.toml` header still references primalSpring V0.3.0 — needs alignment with v0.9.25
2. **Tier 3 IPC-first**: 281 `barracuda::` library calls still use direct linking — IPC transition is the next evolution target
3. **exp095 orphaned from UniBin**: exp095 is documented as "absorbed into certification L2" in fossil record, but has no named scenario in `build_registry()` — clarify coverage or add scenario
4. **pytest failures**: 67/400 pytest failures (Kokkos binary name issues) — build-environment dependent, not a code bug

---

**License**: AGPL-3.0-or-later
