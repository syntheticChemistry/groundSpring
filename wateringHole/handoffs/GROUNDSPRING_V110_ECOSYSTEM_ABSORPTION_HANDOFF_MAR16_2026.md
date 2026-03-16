# groundSpring V110 — Cross-Ecosystem Absorption Handoff

**Date**: March 16, 2026
**From**: groundSpring V110 (39 modules, 912+ tests, 102 delegations)
**To**: barraCuda / toadStool / coralReef teams, All Springs
**Authority**: wateringHole (ecoPrimals Core Standards)
**Supersedes**: GROUNDSPRING_V109_DEEP_DEBT_SMART_REFACTOR_HANDOFF_MAR16_2026.md
**Pins**: barraCuda v0.3.5, toadStool S155b+, coralReef Iteration 49+
**License**: AGPL-3.0-or-later (SCYBORG Provenance Trio)

---

## Executive Summary

V110 is a cross-ecosystem absorption sprint. Every spring completed deep debt
sprints on March 16, 2026 — groundSpring reviewed all spring handoffs and
primal evolution, then absorbed the best patterns ecosystem-wide. No new
science or new delegations — this version raises the quality floor and wires
forward-compatible infrastructure.

Key changes:
1. **Zero `#[allow()]` in entire codebase** — 95 files migrated to `#[expect(reason)]`
   with curated reason strings (wetSpring V122 pattern)
2. **Python tolerance mirror** — `control/tolerances.py` with all 28 Rust constants,
   grouped by domain with provenance (healthSpring V29 pattern)
3. **Structured tracing** — primal binary uses `tracing` crate with key-value fields
   instead of `eprintln!` (airSpring v0.8.4 pattern)
4. **toadStool `compute.dispatch.*`** — 3 direct dispatch methods for sub-frame GPU
   workloads (ludoSpring V22 pattern)
5. **Dual-format capability parsing** — handles both flat-array and nested-object
   `capability.list` responses (neuralSpring S156 compatibility)
6. **`deny.toml`** — `wildcards=deny`, vulnerability/yanked audit (airSpring v0.8.4 alignment)
7. **aarch64 cross-compile CI** — ecoBin compliance for ARM targets

---

## Quality Gates (V110)

| Gate | Status |
|------|--------|
| cargo test | 912+ passed, 0 failed |
| cargo clippy (pedantic + nursery) | 0 warnings |
| cargo fmt --check | 0 diff |
| cargo doc -D warnings | 0 warnings |
| `#[allow()]` in codebase | 0 (all migrated to `#[expect(reason)]`) |
| `std::env::set_var` usage | 0 (Rust 2024 safe) |
| Files > 1000 LOC | 0 |
| `deny.toml` wildcards | deny |
| aarch64 cross-compile | wired in CI |
| License | AGPL-3.0-or-later |

---

## Part 1: `#[expect(reason)]` Migration

### What Changed

Every `#[allow(clippy::unwrap_used, clippy::expect_used)]` on test modules has
been replaced with:

```rust
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
```

This covers 83 crate files + 12 metalForge forge files = 95 total.

### Why This Matters for barraCuda/toadStool

`#[expect()]` is strictly superior to `#[allow()]`:
- Stale suppressions produce `unfulfilled_lint_expectations` warnings
- The `reason` field documents *why* the suppression exists
- Automated cleanup: `cargo clippy --message-format=json` detects stale ones

**Recommendation**: All primals should adopt this as ecosystem standard. See
wetSpring V122 handoff for the full reason dictionary and migration tooling.

---

## Part 2: Python Tolerance Mirror

### What Changed

Created `control/tolerances.py` with all 28 constants from:
- `groundspring::tol::*` (13 tiers with mathematical provenance)
- `groundspring-validate::tolerances` (validation-specific: `TOL_RAREFACTION_PROP`,
  `TOL_REGIME`, `TOL_GRID_MATCH`, `TOL_MONOTONIC_SLACK`, `TOL_ET0`, thresholds)
- `groundspring::eps::*` (epsilon guards: `EPS_SAFE_DIV`, `EPS_SAFE_DIV_STRICT`,
  `EPS_LOG_FLOOR`, `EPS_UNDERFLOW`)
- Physical bounds: `ET0_PLAUSIBLE_MIN_MM`, `ET0_PLAUSIBLE_MAX_MM`

`control/common.py` re-exports core constants for backward compatibility.

### Pattern Worth Absorbing

healthSpring V29, ludoSpring V22, and wetSpring V121 all independently created
`tolerances.py` mirrors. This is now a de facto ecosystem standard. Any spring
with Python baselines should have a single-source-of-truth tolerance module.

---

## Part 3: toadStool Direct Dispatch

### What Changed

Three new functions in `biomeos/interaction.rs`:
- `dispatch_submit(op, params)` → `compute.dispatch.submit`
- `dispatch_result(job_id)` → `compute.dispatch.result`
- `dispatch_capabilities()` → `compute.dispatch.capabilities`

All use capability-based discovery — no hardcoded primal names.

### What toadStool Needs to Know

These methods discover toadStool at runtime via `discover_by_capability()` and
call the `compute.dispatch.*` IPC methods directly, bypassing Neural API
capability routing for lower latency. This follows ludoSpring V22's sub-frame
dispatch pattern.

groundSpring's 102 existing delegations via `#[cfg(feature)]` remain unchanged.
The dispatch methods are additive — they enable a future path where groundSpring
can submit GPU validation workloads directly to toadStool.

---

## Part 4: Dual-Format Capability Parsing

### What Changed

`discover_by_capability()` previously used naive `body.contains(capability)`
which broke on nested-object responses. Now uses `extract_capabilities()` that
handles:
- **Flat array**: `["compute.execute", "storage.put"]`
- **Nested objects (name key)**: `[{"name": "compute.execute", "version": "1.0"}]`
- **Nested objects (capability key)**: `[{"capability": "compute.execute"}]`
- **Wrapped object**: `{"capabilities": ["compute.execute"]}`

### Why This Matters

neuralSpring S156 changed the capability response format from flat arrays to
nested objects with version metadata. Without this fix, groundSpring's
capability-based discovery would silently fail when talking to S156+ biomeOS.

6 unit tests cover all formats including edge cases.

---

## Part 5: barraCuda Primitive Consumption (V110 — 102 Active, Unchanged)

No new delegations in V110. The 102 active delegations (61 CPU + 41 GPU) from
V109 remain intact and passing at all three tiers.

### Unwired Primitives (candidates for future absorption)

When barraCuda `GemmF64` gains transpose flags:
- `mat_transpose_mul` / `mat_transpose_vec` in `spectral_recon.rs` (currently
  local — small matrices, Cholesky already delegated via barracuda)

### Evolution Request: GemmF64 Transpose

P2 request: Add `transpose_a: bool` / `transpose_b: bool` flags to
`barracuda::linalg::GemmF64`. This would let groundSpring delegate the
Tikhonov regularization matrix operations that are currently local.

---

## Part 6: Patterns Worth Absorbing Ecosystem-Wide

| Pattern | Source | Benefit |
|---------|--------|---------|
| `#[expect(reason)]` | wetSpring V122 | Stale suppression detection, documented rationale |
| `tolerances.py` mirror | healthSpring V29 | Rust↔Python constant parity, single source of truth |
| `tracing` in binaries | airSpring v0.8.4 | Structured logging, `RUST_LOG` control, key-value fields |
| `compute.dispatch.*` | ludoSpring V22 | Sub-frame GPU dispatch, bypasses capability routing |
| `deny.toml` wildcards | airSpring v0.8.4 | Prevents wildcard dep versions, supply chain audit |
| aarch64 CI | airSpring v0.8.4 | ecoBin ARM compliance, catches platform-specific issues |
| Dual-format capabilities | neuralSpring S156 | Forward-compatible discovery as biomeOS evolves |

---

## Part 7: Learnings Relevant to barraCuda/toadStool Evolution

### For barraCuda

1. **`#[expect(reason)]` migration** is straightforward — bulk `sed` + `cargo fmt`.
   Zero compile issues. Recommend absorbing across all barraCuda crates.
2. **`deny.toml`** with `wildcards=deny` catches accidental `*` version specs in
   Cargo.toml. Already present in airSpring's barraCuda fork; should be in
   the canonical barraCuda repo.

### For toadStool

1. **`compute.dispatch.*`** is now consumed by both ludoSpring V22 and groundSpring
   V110. The API surface should be considered stable enough for documentation.
2. **Capability response format** should settle on one format (flat array vs nested).
   Both ludoSpring and groundSpring handle both, but consistency would simplify
   all consumers.

### For All Springs

1. **`temp-env`** crate is the Rust 2024-safe way to manipulate env vars in tests.
   groundSpring has zero direct `set_var`/`remove_var` calls.
2. **aarch64 cross-compile** in CI catches assumptions about pointer width, endianness,
   and platform-specific APIs early.

---

## What Springs Should Know

- groundSpring V110 is a quality/alignment sprint, not a science sprint
- Zero new experiments, zero new delegations, zero new modules
- All 35 experiments remain PASS, all 395 validation checks PASS
- The 102 delegations are unchanged and proven at three tiers
- Next science sprint will focus on absorbing `compute.dispatch.*` for GPU
  validation workloads and potentially wiring `mat_transpose_mul` when
  `GemmF64` gains transpose flags
