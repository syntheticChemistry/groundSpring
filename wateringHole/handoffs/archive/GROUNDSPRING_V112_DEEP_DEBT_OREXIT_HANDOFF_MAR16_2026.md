# groundSpring V112 → Ecosystem Deep Debt + OrExit Handoff

**Date**: March 16, 2026
**From**: groundSpring V112
**To**: barraCuda, toadStool, coralReef, ecosystem
**Authority**: Control experiment status + changelog
**Supersedes**: V110 (GROUNDSPRING_V110_ECOSYSTEM_ABSORPTION_HANDOFF_MAR16_2026.md)
**Pins**: barraCuda v0.3.5, toadStool S155b+, coralReef Iteration 49+
**License**: AGPL-3.0-or-later

## Executive Summary

V112 completes two sprints of deep debt evolution since V110:

### V111 — Deep Debt Evolution
1. `thiserror` for all error types — `InputError`, `BiomeOsError`, `IpcError` evolved from manual Display+Error impls
2. `DispatchOutcome` enum — structured RPC error classification (Ok, ProtocolError, ApplicationError)
3. Safe casts — 25+ `as` casts replaced with `crate::cast` helpers or documented `#[expect(reason)]`
4. Config injection — env var reads evolved to `_with_env` DI pattern for testability
5. Dead code removal — redundant `compute_execute`, `storage_put`, `storage_get` in interaction.rs

### V112 — Ecosystem Absorption + OrExit
6. `OrExit<T>` trait — zero-boilerplate exit pattern for validation binaries (wetSpring V123 / healthSpring V31)
7. `parse_benchmark()` — 28 validation binaries migrated from let-else boilerplate
8. `BenchFieldError` evolved to `thiserror::Error` derive
9. Generic `socket_env_var()` / `address_env_var()` — primal discovery helpers (sweetGrass v0.7.18)
10. Provenance trio — `RHIZOCRYPT`, `LOAMSPINE`, `SWEETGRASS` in `primal_names.rs`
11. Validate crate decoupled from barracuda (`default-features = false`)
12. Hardcoded `/tmp/test.sock` → `tempfile::tempdir()`

## Quality Gates

| Gate | Status |
|------|--------|
| `cargo test` (605 unit + 24 integration) | PASS |
| `cargo clippy -D warnings` (core) | 0 warnings |
| `cargo clippy -D warnings` (validate) | 0 warnings |
| `cargo fmt --check` | 0 diff |
| Validation binaries (29/29) | PASS at all 3 tiers |
| metalForge checks (140) | PASS |
| barracuda delegations | 102 (61 CPU + 41 GPU) |

## Part 1: Error Type Evolution (thiserror)

### What Changed
All three primary error types now use `thiserror::Error` derive:
- `InputError` — `LengthMismatch`, `InsufficientData`, `OutOfRange` with structured fields
- `BiomeOsError` — `Transport`, `Dispatch`, `Capability`, `Socket`, `Protocol` with `#[non_exhaustive]`
- `IpcError` — evolved from newtype `String` to enum: `Connect`, `Transport`, `Remote`, `Discovery`

### Why This Matters for barraCuda/toadStool
Downstream consumers get structured error variants instead of opaque strings. Match on `IpcError::Connect` vs `IpcError::Transport` to distinguish connection failures from protocol errors.

## Part 2: DispatchOutcome for RPC

### What Changed
New `DispatchOutcome` enum classifies JSON-RPC responses:
- `Ok(String)` — successful result
- `ProtocolError { code, message }` — JSON-RPC layer errors (-32700 to -32600)
- `ApplicationError { code, message }` — business logic errors

`is_method_not_found()` enables graceful capability probing.

### Why This Matters for toadStool
Springs can probe for capabilities and gracefully fall back without treating "method not found" as a hard error. toadStool should ensure `compute.dispatch.*` methods return standard JSON-RPC error codes.

## Part 3: OrExit<T> and Validation Patterns

### What Changed
- `OrExit<T>` trait on `Result<T, E>` and `Option<T>` — `.or_exit("message")` prints to stderr and exits with code 1
- `parse_benchmark(json_str)` — one-call benchmark JSON parsing
- 28 validation binaries migrated, eliminating 4 lines of boilerplate each

### Recommendation for Ecosystem
All springs with validation binaries should absorb this pattern. It's already in wetSpring V123 and healthSpring V31.

## Part 4: Generic Primal Discovery

### What Changed
- `socket_env_var("groundspring")` → `"GROUNDSPRING_SOCKET"`
- `address_env_var("nestgate")` → `"NESTGATE_ADDRESS"`
- Eliminates per-primal discovery constants

### Recommendation for toadStool
toadStool's `ComputeScheduler` should use generic discovery rather than hardcoded primal socket paths.

## Part 5: Safe Cast and DI Patterns

### What Changed
- `crate::cast` helpers: `usize_f64`, `u64_f64`, `f64_usize` with saturation semantics
- `_with_env` DI pattern: all env var reads accept an injected reader for testing
- 25+ metalForge casts documented with `#[expect(reason)]`

## Part 6: Outstanding Evolution Requests for barraCuda

| Priority | Request | Context |
|----------|---------|---------|
| P1 | `GemmF64` transpose flags | `spectral_recon.rs` keeps local `mat_transpose_mul` |
| P1 | Tridiag eigenvectors | `transport.rs` uses local QL; barracuda Sturm is eigenvalues-only |
| P1 | GPU FFT (real + complex) | Would enable full spectral pipeline on GPU |
| P2 | PRNG alignment (`xoshiro128**` ↔ `xorshift64`) | Bitwise GPU–CPU reproducibility |
| P2 | Parallel 3D grid dispatch | `seismic.rs`, `freeze_out.rs` grid searches |
| P3 | Unified `ComputeScheduler` | metalForge manually routes CPU/GPU/NPU |
| P3 | `erfc` large-x stability | hotSpring Exp 046 flagged cancellation issue |

## Part 7: What groundSpring Does NOT Need from barraCuda

- **Chao1** — stays local (Chao 1984 formula; barracuda uses Chao & Chiu 2016)
- **Small matrix transpose** — n_omega ≤ 200; dispatch overhead exceeds benefit
- **PRNG** — local `xorshift64` is intentional for baseline reproducibility

---

**groundSpring V112 | 39 modules | 35 experiments | 912+ tests | 102 delegations | AGPL-3.0-or-later**
