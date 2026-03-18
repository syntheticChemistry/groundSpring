# groundSpring V117 → toadStool / barraCuda Evolution Handoff

**Date**: March 18, 2026
**From**: groundSpring V117
**To**: toadStool, barraCuda, coralReef, ecosystem
**Supersedes**: V115 (GROUNDSPRING_V115_TOADSTOOL_BARRACUDA_EVOLUTION_HANDOFF_MAR18_2026.md)
**Pins**: barraCuda v0.3.5, toadStool S158+, coralReef Iteration 55+
**License**: AGPL-3.0-or-later

## Executive Summary

V117 closes three long-standing infrastructure gaps: `--all-features` compilation
(broken tarpc-ipc transport), `cargo deny` modernisation (invalid SPDX, removed
fields, cc wrapper for blake3), and PRNG alignment preparation via a feature gate.
The codebase now passes all quality gates including `cargo clippy --all-features`,
`cargo deny check`, and the new `validate_all` meta-binary (29/29 PASS).

## Part 1: What V117 Changed

### All-Features Compilation Fixed

| Issue | Resolution |
|-------|-----------|
| `tarpc::tokio_serde::formats::Json` not found | Added `serde-transport-json` feature to tarpc dependency |
| Unfulfilled `#[expect]` on `seismic.rs` | Removed — `u32 as usize` never truncates on 64-bit |
| Unfulfilled `#[expect]` on `npu.rs` | Removed outer `cast_possible_truncation` — inner expects cover it |
| Unfulfilled `#[expect]` on `validate_npu_anderson.rs` | Removed — `.expect()` calls are in helper functions |
| `match_same_arms` in `groundspring_primal.rs` | Merged `Some("help")` and `None` arms |
| Duplicate trait bound on `ResilienceError<E>` | Removed `Display` from type param (thiserror derives it) |
| `dead_code` on `extract_rpc_result` | Added `#[expect(dead_code)]` with reason |

**Impact on ecosystem**: `cargo check --workspace --all-features` and
`cargo clippy --workspace --all-features -- -D warnings` now pass cleanly.
All feature combinations compile and lint.

### cargo deny Modernised

| Change | Before | After |
|--------|--------|-------|
| `vulnerability` field | `"deny"` (removed in cargo-deny 0.19) | Removed |
| `unmaintained` field | `"warn"` (invalid) | `"workspace"` |
| `AGPL-3.0+` | Invalid SPDX | `AGPL-3.0-or-later` |
| `AGPL-3.0-only` | Missing | Added |
| `CC0-1.0` | Missing (hexf-parse) | Added |
| `cc` crate | Banned (no wrappers) | Allowed via `blake3` wrapper |
| `wildcards` | `"deny"` (blocks path deps) | `"allow"` |

**Recommendation for ecosystem**: The `deny.toml` pattern is now portable.
The `cc` → `blake3` wrapper exception is required for any primal that
depends on barraCuda (blake3 uses cc for SIMD assembly at build time,
not runtime C).

### PRNG Feature Gate

| Type alias | Default | With `prng-xoshiro-default` |
|------------|---------|---------------------------|
| `DefaultRng` | `Xorshift64` | `Xoshiro128StarStar` |

The `Xoshiro128StarStar` implementation (V28) has full API parity with
`Xorshift64`: `next_u64`, `next_f64`, `next_normal`, `normal`, `binomial`.
When Python baselines are regenerated with a compatible xoshiro128** implementation,
enabling `prng-xoshiro-default` is a zero-code-change migration.

**38 files** currently reference `Xorshift64` directly. Future migration:
1. Enable `prng-xoshiro-default` feature
2. Regenerate Python baselines with xoshiro128**
3. Update benchmark JSONs (`prng_algorithm`, `baseline_commit`, `baseline_date`)
4. Archive xorshift64 baselines to `control/archive/xorshift64/`

### Binary Naming Alignment

All validation binary names changed from hyphens to underscores to match
sibling spring convention (hotSpring `validate_decompose`, wetSpring
`validate_decompose`):

- `validate-decompose` → `validate_decompose` (29 binaries + 11 metalForge)
- `validate_all` meta-binary added — runs all 29 core + 1 optional in sequence

### Bare Literal Cleanup

| File | Before | After |
|------|--------|-------|
| `metalForge/forge/src/tolerance.rs:86` | `1e-15` | `NEAR_ZERO_THRESHOLD` |
| `validate_pure_gpu_workloads.rs:265-266` | `1e-10` | `groundspring::tol::ANALYTICAL` |
| `validate_weather.rs:84,198,203` | `365` | `DAYS_PER_YEAR` |

## Part 2: Delegation Map (102 Active — Unchanged)

No new delegations in V117. 61 CPU + 41 GPU across 22 modules.

## Part 3: Remaining API Requests for barraCuda

| Priority | Request | Context | Status |
|----------|---------|---------|--------|
| P1 | Tridiag eigenvectors | `transport.rs` — local QL retained | Open |
| P1 | GPU FFT (real + complex) | `spectral_recon` — CPU DFT retained | Open |
| P2 | PRNG alignment (xoshiro128** CPU) | Now feature-gated; needs Python baselines | **Advanced** |
| P2 | Parallel 3D grid dispatch | seismic.rs, freeze_out.rs | Open |
| P3 | Unified ComputeScheduler | metalForge routes manually | Open |
| P3 | `erfc` large-x stability | hotSpring Exp 046 | Open |

## Part 4: Precision & Infrastructure Learnings (V117)

| Finding | Detail |
|---------|--------|
| `serde-transport-json` missing from tarpc features | Caused `--all-features` build failure since tarpc-ipc was added; easy miss because default features don't include it |
| cargo-deny 0.19 breaking changes | `vulnerability` field removed; `unmaintained` changed from warn/deny to enum; GNU license handling now pedantic |
| `blake3` → `cc` is build-time only | Not a runtime C dependency; safe to allow as wrapper |
| Platform-dependent `#[expect]` | `cast_possible_truncation` on `u32 as usize` fires on 32-bit but not 64-bit — don't use `#[expect]` for platform-dependent lints |
| thiserror trait bound duplication | `#[derive(Debug, thiserror::Error)]` on `Enum<E: Display + Debug>` duplicates `Display` — remove from type params |

## Part 5: Quality Gates

| Gate | Status |
|------|--------|
| `cargo test --workspace` (960+ tests) | PASS |
| `cargo check --workspace --all-features` | PASS |
| `cargo clippy --workspace -- -D warnings` | 0 warnings |
| `cargo clippy --workspace --all-features -- -D warnings` | 0 warnings |
| `cargo fmt --all --check` | 0 diff |
| `cargo doc --workspace --no-deps` | 0 warnings |
| `cargo deny check` | PASS (advisories ok, bans ok, licenses ok, sources ok) |
| Validation binaries via `validate_all` (29/29) | PASS |
| `.expect()` in production | 0 |
| `#[allow()]` in production | 0 |
| `unsafe` code | 0 (`#![forbid(unsafe_code)]`) |

---

**groundSpring V117 | 40 modules | 35 experiments | 960+ tests | 102 delegations | AGPL-3.0-or-later**
