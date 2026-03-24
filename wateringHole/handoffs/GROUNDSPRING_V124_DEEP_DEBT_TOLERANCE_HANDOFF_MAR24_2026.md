# groundSpring V124 — Deep Debt Resolution + Tolerance Hardening Handoff

**Date**: March 24, 2026
**From**: groundSpring V124
**To**: barraCuda, toadStool, coralReef, all spring teams, biomeOS
**barraCuda**: v0.3.7 | **toadStool**: S158+ | **coralReef**: Iteration 55+

---

## Executive Summary

V124 is a deep debt resolution release focused on tolerance hardening,
output safety, capability-based discovery, and CI hardening. No new
experiments or delegations — this release strengthens the foundation.

**Key changes for downstream consumers:**
- `groundspring::eps` is now a **public** module (was `pub(crate)`)
- New `eps::SAFE_DIV_STRICT` (1e-20) constant for strict near-zero guards
- NDJSON validation output now RFC 8259 safe (JSON injection hardened)
- `validate_all` meta-validator distinguishes hardware-unavailable (exit 2) from failure (exit 1)
- 29 validator exit-code integration tests added
- All bare float literals replaced with named `tol::`/`eps::` constants

---

## What Changed (for primals and springs)

### 1. `eps` Module Now Public

**Impact**: barraCuda, all springs

The `groundspring::eps` module is now `pub` (was `pub(crate)`). Downstream
crates can reference `groundspring::eps::SAFE_DIV`, `eps::LOG_FLOOR`,
`eps::SSA_FLOOR`, `eps::SAFE_DIV_STRICT`, `eps::UNDERFLOW` directly.

**New constant**: `eps::SAFE_DIV_STRICT = 1e-20` — strict near-zero guard
for diffusion coefficients (~1e-15 m²/s floor), MSD log-log filters, and
similar quantities where `SAFE_DIV` (1e-10) is too generous.

**Pattern for springs**: If your validation code uses bare `1e-20` for
division safety, replace with `groundspring::eps::SAFE_DIV_STRICT` or
define your own re-export.

### 2. Named Tolerance Constants

**Impact**: All springs (pattern to absorb)

Every bare float literal in library and validation code now references
a named constant from `tol::` or `eps::`. Examples:

| Module | Old | New |
|--------|-----|-----|
| `drift/mod.rs` | `1e-10` | `eps::SAFE_DIV` |
| `drift/mod.rs` | `1e-15` | `tol::DETERMINISM` |
| `freeze_out/grid.rs` | `1e-12` | `tol::EXACT` |
| `freeze_out/grid.rs` | `1e-15` | `tol::DETERMINISM` |
| `freeze_out/grid.rs` | `1e-4` | `tol::RECONSTRUCTION` |
| `band_structure.rs` | `1e-12` | `tol::EXACT` |
| `transport.rs` | `1e-20` | `eps::SAFE_DIV_STRICT` |
| `dispatch/defaults.rs` | `1e-4` | `tol::RECONSTRUCTION` |
| `validate_fao56.rs` | `f64::EPSILON` | `TOL_DETERMINISM` |
| `validate_et0_methods.rs` | `f64::EPSILON` | `TOL_DETERMINISM` |
| `validate_rare_biosphere.rs` | `f64::EPSILON` | `TOL_DETERMINISM` |

**`ValidationHarness`**: `check_relative` and `check_abs_or_rel` now use
`RELATIVE_DENOM_GUARD` (= `tol::DETERMINISM`) instead of bare `1e-15`.

**Recommendation for other springs**: Audit your codebase for bare float
literals in comparisons. Replace with named constants from your own `tol`
module or reference `groundspring::tol::*` via re-export.

### 3. NDJSON Injection Hardening

**Impact**: Any consumer of `NdjsonSink` output

`NdjsonSink` now escapes `"`, `\`, newlines, and ASCII control characters
in all string fields (`label`, `detail`, `name`, `text`) before
interpolation. This prevents structural injection via crafted labels.

The `json_escape()` function is intentionally minimal (no serde_json dep)
and handles RFC 8259 escaping. 5 adversarial tests verify the hardening.

**Pattern for springs**: If you have structured output sinks (NDJSON, JSON
logs), audit for unescaped string interpolation. The `format!` + raw
string template pattern is unsafe for JSON — always escape or use a
serializer.

### 4. validate_all Exit-Code Protocol

**Impact**: CI systems, toadStool, biomeOS

`validate_all` now uses a `RunResult` enum:
- **Exit 0**: All checks pass
- **Exit 1**: Validation failure (real bug)
- **Exit 2**: Hardware unavailable (GPU, NPU, NUCLEUS absent)

Optional binaries only count as "skip" when exit code is exactly 2.
Any other non-zero exit (including from optional binaries) is a hard fail.

**Recommendation**: If your spring has hardware-dependent validators,
adopt exit code 2 for hardware-unavailable to integrate cleanly with
meta-validators.

### 5. Capability-Based Discovery

**Impact**: NestGate, biomeOS, toadStool

`validate_nestgate_ncbi` no longer uses a hardcoded fallback port (8090)
or localhost default. Discovery chain:
1. `NESTGATE_URL` (full URL)
2. `NESTGATE_ADDRESS` (host:port)
3. `NESTGATE_HOST` + `NESTGATE_PORT` (individual)
4. biomeOS socket registry (`socket-registry.json`)

Fails with exit code 2 if no discovery path succeeds.

`nucleus.rs` UID discovery now has 4 tiers (`$UID` → `/proc/self/status`
→ `id -u` → `/run/user/` enumeration) and panics with a descriptive
message instead of silently falling back to "1000".

**Pattern**: No primal should hardcode another primal's address. Discover
at runtime or fail closed.

### 6. Clippy Debt Cleared

- 3 stale `#[expect(clippy::cast_possible_truncation)]` removed (compiler
  is now smart enough to know `usize as u64` doesn't truncate on 64-bit)
- 2 unnecessary `as u64` casts removed where type was already `u64`
- Modern let-chain syntax adopted for collapsible-if patterns

---

## Quality Certificate

| Metric | Value |
|--------|-------|
| `cargo fmt --check --all` | PASS (zero diffs) |
| `cargo clippy --all-features -D warnings -W clippy::pedantic -W clippy::nursery` | PASS (zero warnings) |
| `cargo doc -D warnings` | PASS (zero warnings) |
| `cargo test --workspace` | PASS (all tests, zero failures) |
| Validation checks | 395/395 PASS |
| Validator integration tests | 29/29 PASS |
| Library line coverage | ≥92% |
| `#[allow()]` in production | Zero |
| `unsafe` in application code | Zero (`#![forbid(unsafe_code)]`) |
| Bare float literals in comparisons | Zero (all named) |
| NDJSON injection surface | Hardened (RFC 8259 escaping) |

---

## Evolution Items (for primals)

### For barraCuda
- `eps::SAFE_DIV_STRICT` (1e-20) — consider absorbing as a barraCuda-level
  constant if other springs need strict division guards
- Named tolerance pattern is mature — barraCuda's own internal thresholds
  could follow the same `tol::`/`eps::` architecture

### For toadStool
- Exit code 2 protocol for hardware-unavailable — adopt ecosystem-wide
- UID discovery should be a shared utility (currently in `groundspring-forge::nucleus`)

### For coralReef
- No direct changes; sovereign compilation path remains on roadmap

### For Springs (wetSpring, hotSpring, neuralSpring, airSpring, etc.)
- **Absorb the named-constant pattern**: audit bare `1e-N` literals, replace with named `tol::`/`eps::` constants
- **Absorb NDJSON hardening**: if you use `NdjsonSink` or similar structured output, audit for injection
- **Absorb exit-code protocol**: exit 0/1/2 for pass/fail/hardware-unavailable
- **Absorb capability-based discovery**: no hardcoded addresses in validators

### For biomeOS
- `validate_all` + exit-code integration tests provide a meta-validation pattern for orchestrated CI
- Socket registry discovery is now a fallback path in NestGate validation — confirms registry format
