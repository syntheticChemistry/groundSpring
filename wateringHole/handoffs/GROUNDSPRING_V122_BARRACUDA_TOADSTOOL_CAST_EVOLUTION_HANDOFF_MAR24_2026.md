# groundSpring V122 → barraCuda / toadStool Cast Evolution Handoff

**Date**: March 24, 2026
**From**: groundSpring V122
**To**: barraCuda team, toadStool team
**Pins**: barraCuda v0.3.7, toadStool S158+, coralReef Iteration 55+

---

## Summary

groundSpring V122 completes a **cast evolution and module extraction** cycle.
The primary theme is evolving bare `as` casts into centralized, documented,
type-safe cast helpers — eliminating 20+ `#[expect(clippy::cast_possible_truncation)]`
blocks and making the safety argument once per conversion type.

This handoff documents patterns worth absorbing into barraCuda and toadStool,
plus continuing evolution priorities from V121.

---

## 1. Cast Module Architecture (pattern worth absorbing)

### What we built

`groundspring::cast` — a public module of named numeric cast functions:

| Function | Conversion | Use case |
|----------|-----------|----------|
| `usize_f64(n)` | `usize → f64` | Collection lengths in math |
| `u64_f64(n)` | `u64 → f64` | Rarefaction counts, PRNG |
| `f64_usize(x)` | `f64 → usize` | Index from float rank |
| `usize_u32(v, label)` | `usize → Result<u32>` | Checked GPU dispatch params |
| `usize_u64(v)` | `usize → u64` | Infallible widening |
| `f64_f32(v)` | `f64 → f32` | GPU shader inputs |
| `u64_u32_truncate(v)` | `u64 → u32` | PRNG seed low-32-bit extraction |
| `u64_usize(v)` | `u64 → usize` | Binomial results, RNG index |
| `u32_f64(v)` | `u32 → f64` | Always exact |
| `i32_f64(v)` | `i32 → f64` | Always exact |
| `f64_u32(v)` | `f64 → u32` | Debug-asserted truncation |
| `u32_usize(v)` | `u32 → usize` | Always exact |
| `f64_i32(v)` | `f64 → i32` | Debug-asserted truncation |
| `usize_i32(v)` | `usize → i32` | Debug-asserted sign conversion |

### Why this matters for barraCuda

barraCuda GPU dispatch code has the same pattern: `usize` workgroup counts
passed to `wgpu` APIs that expect `u32`. Every spring reinvents this.
**Recommendation**: absorb a `barracuda::cast` module that all springs can
depend on. The `usize_u32(value, label) → Result<u32, String>` pattern is
particularly valuable for GPU dispatch — it catches overflow before silent
truncation.

### `u64_u32_truncate` — new pattern

PRNG seed generation universally does `rng.next_u64() as u32` to extract
low-32-bit entropy. This is intentional, not accidental truncation. The named
function documents that intent. **Recommendation**: barraCuda's PRNG module
should provide this or a `Seed32::from_u64(v)` wrapper.

---

## 2. Module Extraction Results

| File | Before | After | Extracted |
|------|--------|-------|-----------|
| `lib.rs` | 607 LOC | 182 LOC | `cast.rs` (213), `tol.rs` (112), `eps.rs` (30) |
| `validate/lib.rs` | 769 LOC | 226 LOC | `accessors.rs` (305) |

These are **smart refactors** — each extracted module is a self-contained
domain with its own tests. The cast module is `pub` to enable cross-crate
use (`groundspring-validate` now calls `groundspring::cast::u64_usize`).

---

## 3. Epsilon Guards (eps module)

`eps::SSA_FLOOR` (1e-15) is now unconditional — available in all builds,
not just `barracuda-gpu`. This is a general SSA steady-state guard for
Gillespie simulation, not GPU-specific.

**Pattern worth absorbing**: barraCuda's `ops::bio::GillespieGpu` could
expose a `SSA_FLOOR` or `STEADY_STATE_GUARD` constant alongside its API.

---

## 4. Dependency Cleanup

- **`deny.toml`**: Removed stale `ring` license clarification. `ring` is
  banned for ecoBin compliance; the clarification was dead weight from a
  transitive dep that no longer exists.

---

## 5. Continuing Evolution Priorities (from V121)

### P0 — Critical alignment

| Item | Status | Notes |
|------|--------|-------|
| PRNG alignment (xoshiro128** in WGSL vs xorshift64 in Rust) | Open | Different algorithms give different streams; validation relies on tolerance, not bitwise identity |
| Lanczos at scale (N > 4096) | Open | `barracuda::spectral::lanczos` works but memory grows; chunked approach needed |

### P1 — Important

| Item | Status | Notes |
|------|--------|-------|
| Sparse SpMV (`CsrMatrix::spmv`) for Anderson 2D/3D | Open | Currently dense fallback; sparse would enable N=10000+ |
| `GpuDriverProfile` deprecation | Open | V120 migrated to `DeviceCapabilities`; old struct still in barraCuda |
| Named tolerances in barraCuda test suite | Open | barraCuda tests use bare float literals; adopt `tol::` pattern |
| `#[expect(reason)]` migration in barraCuda | Open | groundSpring is at zero `#[allow]`; barraCuda still has many |
| Cast module in barraCuda | **NEW** | Absorb `barracuda::cast` from groundSpring pattern |

### P2 — Desired

| Item | Status | Notes |
|------|--------|-------|
| RAWR GPU dispatch | Open | `rawr_mean` is CPU; batch bootstrap pattern exists |
| Matrix exponentiation GPU | Open | For drift/selection at large N |
| Sobol sensitivity indices | Open | For FAO-56 MC pipeline |

---

## 6. Quality Certificate

| Gate | Result |
|------|--------|
| `cargo check --workspace` | PASS (0 warnings) |
| `cargo clippy --workspace` | PASS (0 warnings) |
| `cargo fmt --check` | PASS (0 diffs) |
| `cargo doc --workspace` | PASS (0 warnings) |
| `cargo test --workspace` | PASS (1000+ tests, 0 failures) |
| `cargo deny check` | PASS |
| Validation checks | 395/395 PASS |
| metalForge checks | 140/140 PASS |
| Math parity | 29/29 PROVEN |
| Library coverage | ≥92% |
| `unsafe` in production | Zero (`#![forbid(unsafe_code)]`) |
| `#[allow]` in production | Zero |
| `TODO`/`FIXME` in production | Zero |
| `unwrap`/`expect` in production | Zero (workspace deny) |

---

## 7. Delegation Inventory

**110 active delegations** (67 CPU + 43 GPU), unchanged from V121.

No new delegations in V122 — this was a hygiene cycle. The 20+ cast evolutions
change _how_ existing delegations pass parameters (type-safe casts), not _which_
operations are delegated.

---

## 8. Cross-Spring Learnings

### For barraCuda
1. **Cast module pattern**: centralized, documented, tested. Eliminates scatter
   of `#[expect(cast_possible_truncation)]` across 10+ files
2. **`u64_u32_truncate` for PRNG seeds**: makes intentional truncation explicit
3. **`eps::SSA_FLOOR` alongside `GillespieGpu` API**: guards should live near
   the operations they protect
4. **Module extraction threshold**: ~600 LOC triggers extraction of
   self-contained sub-domains (cast, tolerances, accessors)

### For toadStool
1. **`usize_u32` for dispatch params**: GPU workgroup sizes, buffer counts, and
   dispatch dimensions should use checked conversion, not bare `as u32`
2. **Feature-gated `dead_code`**: `#[cfg_attr(not(feature = "x"), expect(dead_code, reason = "..."))]`
   is the modern pattern for code used only under optional features

### For all springs
1. **`pub mod cast`** over `pub(crate) mod cast`: cross-crate validation
   binaries need the same cast helpers as the library
2. **Smart refactoring**: extract when a module has a clear domain boundary,
   not when it crosses an arbitrary LOC threshold
