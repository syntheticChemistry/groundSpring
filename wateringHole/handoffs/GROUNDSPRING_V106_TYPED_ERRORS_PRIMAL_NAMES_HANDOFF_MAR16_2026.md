# groundSpring V106 — Typed Errors + Primal Names Handoff

**Date**: March 16, 2026
**From**: groundSpring V106 (39 modules, 876+ tests, 102 delegations)
**To**: barraCuda / toadStool / biomeOS teams
**Authority**: wateringHole (ecoPrimals Core Standards)
**Supersedes**: GROUNDSPRING_V105_CODE_EVOLUTION_BARRACUDA_TOADSTOOL_HANDOFF_MAR15_2026.md

## Pins

- **barraCuda**: v0.3.5 (path dep `../../../barraCuda/crates/barracuda`)
- **toadStool**: S130+ (latest)
- **coralReef**: Iteration 10+

## Executive Summary

V106 absorbs two key patterns from the ecosystem review (wetSpring V119,
petalTongue v1.6.3) and eliminates all hardcoded primal name strings
from production code:

- **`primal_names.rs`** — centralized primal name constants (wetSpring pattern)
- **Typed `BiomeOsError`** — evolved from `BiomeOsError(String)` to a 7-variant enum
- **Zero hardcoded primal strings** — all IPC identifiers, socket paths, and env checks
  use `primal_names::*` constants
- **`niche.rs` rewired** — NICHE_ID and DEPENDENCIES delegate to `primal_names`

## Quality Gates

| Gate | Status |
|------|--------|
| `cargo fmt --all -- --check` | **PASS** (0 diff) |
| `cargo clippy --workspace --all-targets -D warnings -W pedantic` | **PASS** (0 warnings) |
| `cargo test --workspace` | **876+ passed, 0 failed** |
| `#[allow()]` in production | **0** (test modules use `#[allow()]`) |
| `unsafe` in application code | **forbidden** (`#![forbid(unsafe_code)]`) |
| `#![deny(clippy::expect_used, clippy::unwrap_used)]` | **enforced** (all 3 crate roots) |
| Files > 1000 LOC | **0** (largest: fao56/mod.rs @ 642) |
| Mocks in production | **0** |
| Hardcoded primal name strings in production | **0** |
| proptest property tests | **14** (stochastic invariants) |

## Part 1: `primal_names.rs` — Centralized Constants

Follows the wetSpring V119 pattern. Single source of truth for all primal
identifiers used in IPC discovery, capability routing, and socket paths.

```rust
// crates/groundspring/src/primal_names.rs
pub const SELF_ID: &str = "groundspring";
pub const BIOMEOS: &str = "biomeos";
pub const SONGBIRD: &str = "songbird";
pub const NESTGATE: &str = "nestgate";
pub const BEARDOG: &str = "beardog";
pub const TOADSTOOL: &str = "toadstool";
pub const CORALREEF: &str = "coralreef";
pub const PETALTONGUE: &str = "petaltongue";
pub const SQUIRREL: &str = "squirrel";
pub const BIOMEOS_SOCKET_DIR: &str = "biomeos";
```

### Rewiring

| Consumer | Before | After |
|----------|--------|-------|
| `niche.rs` NICHE_ID | `"groundspring"` | `crate::primal_names::SELF_ID` |
| `niche.rs` DEPENDENCIES | `("beardog", ...)` | `(crate::primal_names::BEARDOG, ...)` |
| `biomeos/mod.rs` is_enabled() | `.eq_ignore_ascii_case("biomeos")` | `.eq_ignore_ascii_case(crate::primal_names::BIOMEOS)` |
| `biomeos/server.rs` socket path | `.join("biomeos")` | `.join(crate::primal_names::BIOMEOS_SOCKET_DIR)` |
| `biomeos/interaction.rs` socket path | `.join("biomeos")` | `.join(crate::primal_names::BIOMEOS_SOCKET_DIR)` |
| `biomeos/discovery.rs` socket path | `.join("biomeos")` | `.join(crate::primal_names::BIOMEOS_SOCKET_DIR)` |
| `ipc.rs` socket path | `.join("biomeos")` | `.join(crate::primal_names::BIOMEOS_SOCKET_DIR)` |

## Part 2: Typed `BiomeOsError`

Evolved from `BiomeOsError(pub String)` — a wrapper struct — to a typed enum
with domain-specific variants for better error handling and pattern matching:

```rust
#[non_exhaustive]
pub enum BiomeOsError {
    Transport(String),      // connect, read, write, flush, timeout, bind
    Protocol(String),       // invalid JSON-RPC, missing fields, RPC error
    Serialization(String),  // invalid params JSON, compute params
    Registration(String),   // no capabilities registered
    Discovery(String),      // primal not found, health check failed
    Data(String),           // no results, empty response
    Other(String),          // migration path / uncategorized
}
```

### Migration

- **36+ construction sites** migrated across 8 production files + 1 test file
- `#[non_exhaustive]` ensures downstream consumers handle new variants gracefully
- `BiomeOsError::other()` provides backwards-compatible construction path
- `Display` impl prefixes each variant: `biomeOS transport: ...`, `biomeOS protocol: ...`

### Distribution by variant

| Variant | Count | Example message |
|---------|-------|----------------|
| Transport | 18 | `"biomeOS connect /run/user/1000/biomeos: Connection refused"` |
| Protocol | 5 | `"invalid JSON-RPC response: unexpected EOF"` |
| Serialization | 3 | `"invalid compute params: missing field 'method'"` |
| Registration | 1 | `"no capabilities registered"` |
| Discovery | 3 | `"primal not found: toadstool"` |
| Data | 4 | `"No NCBI results to seed community"` |
| Other | 1 | test-only |

## Part 3: Proptest Status

14 property-based tests already in place from V105 (`crates/groundspring/tests/proptest_invariants.rs`):
- Bootstrap CI monotonicity, symmetry, coverage
- Rarefaction monotonicity, idempotent-at-N
- Diversity index bounds (Shannon, Simpson)
- PRNG determinism and period detection

## Learnings

1. **`primal_names.rs` eliminates typo risk** — misspelling a constant is a compile error, misspelling a string literal is silent.
2. **Typed errors enable match-based recovery** — callers can now `match` on `BiomeOsError::Transport` vs `BiomeOsError::Discovery` and implement variant-specific retry/fallback logic.
3. **`#[non_exhaustive]`** protects downstream consumers when we add new error variants.
4. **Cross-spring pattern adoption accelerates convergence** — wetSpring V119's `primal_names.rs` pattern took <1 hour to absorb into groundSpring.

## For barraCuda/toadStool

No new absorption requests this version — V106 is an internal quality pass.
The existing V105 absorption list (102 delegations, 14 GPU-promotable modules)
remains active.

## For biomeOS

The typed `BiomeOsError` enum is now available for biomeOS orchestration code
to pattern-match on. If biomeOS wants to implement automatic retry for transport
failures vs immediate fail for discovery errors, the variants are there.
