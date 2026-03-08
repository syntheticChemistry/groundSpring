<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# groundSpring V98 → toadStool/barraCuda/coralReef Upstream Rewire Handoff

**Date**: March 8, 2026
**From**: groundSpring V98 (936 tests, 102 delegations, 0 failures, three-tier parity proven)
**To**: barraCuda team, toadStool team, coralReef team
**Supersedes**: V97 handoff (Mar 7, 2026)
**Synced against**: barraCuda `a898dee`, toadStool S130+ (`bfe7977b`), coralReef Iteration 10 (`d29a734`)
**License**: AGPL-3.0-only

## Executive Summary

groundSpring V98 is an **upstream rewire release** that catches up to the
latest barraCuda (deep debt: typed errors, named constants, lint compliance),
toadStool S130+ (clippy pedantic clean, unsafe audit, dependency audit,
spring sync confirming zero API breakage for all 5 springs), and coralReef
Iteration 10 (AMD E2E GPU dispatch verified, conditional branch fix in
`translate_if` + multi-pred RA merge).

**What changed since V97:**
- **Pin updates**: barraCuda `2a6c072` → `a898dee`, toadStool `88a545df` →
  `bfe7977b`, coralReef `72e6d13` → `d29a734`
- **Zero API breakage**: all 936 tests pass without code changes
- **Zero new clippy warnings**
- **Three-tier parity intact**: 29/29 validation binaries PASS at all 3 tiers

*This handoff is unidirectional: groundSpring → ecosystem. No response expected.*

---

## 1. Upstream Changes Absorbed

### barraCuda `2a6c072` → `a898dee` (2 commits)

| Commit | Summary |
|--------|---------|
| `d7ba7f3` | Doc updates: accurate counts post-audit |
| `a898dee` | Deep debt: typed errors (`BarracudaError` hierarchy), named constants (epsilon guards, workgroup sizes), test resilience (deterministic seeds), lint compliance |

**Impact on groundSpring**: None — barraCuda's error types flow through
`Result` chains that groundSpring already handles via `Option`-based GPU
fallback (`if let Ok(...)`). Named constants improve readability but don't
change behavior.

### toadStool `88a545df` → `bfe7977b` (4 commits, S130+)

| Commit | Summary |
|--------|---------|
| `4e575b86` | Clippy pedantic clean, doc update, debris cleanup |
| `a7262515` | Spring sync: all 5 springs confirm zero API breakage |
| `73123cda` | Deep debt: unsafe audit (3 `unsafe fn` → safe alternatives), dependency audit (`flate2`/`procfs` pure Rust), hardcoding evolution, coverage expansion (~200 new tests) |
| `bfe7977b` | Clean root docs, update test counts (19,777 tests), fix stale TESTING.md |

**Impact on groundSpring**: None — groundSpring uses toadStool via
`akida-driver` (NPU path) which is unchanged.

### coralReef `72e6d13` → `d29a734` (2 commits, Iterations 9-10)

| Commit | Summary |
|--------|---------|
| `d5b51c5` | Iteration 9: E2E wiring, push buffer fix, debt reduction |
| `d29a734` | Iteration 10: AMD E2E GPU dispatch verified (RDNA2/GFX1030), docs updated, 990 tests (953 passing, 37 ignored for hardware) |

**Impact on groundSpring**: The conditional branch fix in `translate_if` +
multi-pred RA merge **unlocks f64 shared-memory reduction shaders via the
sovereign path**. Shaders like `sum_reduce_f64.wgsl`, `chi_squared_f64.wgsl`,
`softmax_f64.wgsl`, `layer_norm_f64.wgsl` with 3+ `workgroupBarrier()` calls
now compile to native SM70/SM89 binaries through coralReef. This is the same
class of shader that produces zeros via naga/SPIR-V on RTX 4070 — coralReef
provides a sovereign alternative that bypasses naga entirely.

---

## 2. V97 GPU Smoke Test — Status Update

The P0 action from V97 (reclassify Ada Lovelace as `F64NativeNoSharedMem`)
remains **open**. groundSpring's runtime smoke test continues to protect
against the naga zeros bug. When barraCuda absorbs this fix, groundSpring
can remove the smoke test and rely on the upstream classification.

The coralReef sovereign path (now fixed in Iteration 10) provides an
alternative: when coralReef is running, f64 shared-memory reductions can
compile via coralReef instead of naga, potentially bypassing the bug entirely.
This is a P2 evolution opportunity for when toadStool's `shader.compile.*`
proxy is wired end-to-end.

---

## 3. Quality Gates

```
cargo fmt --check                         → PASS
cargo clippy --workspace --all-targets    → 0 warnings (pedantic + nursery)
cargo test --workspace                    → 936 passed, 0 failed
cargo test --workspace --features barracuda → 936 passed, 0 failed
validation binaries (barracuda-gpu)       → 29/29 PASS
metalForge tests                          → 140 passed, 0 failed
```

---

## 4. Delegation Inventory (unchanged)

102 active delegations (61 CPU + 41 GPU). No new delegations — this release
focuses on upstream pin sync.

Full inventory in `specs/BARRACUDA_EVOLUTION.md`.

---

## 5. Evolution Requests (carried from V97)

| Priority | Request | Team | Status |
|----------|---------|------|--------|
| **P0** | Reclassify Ada Lovelace + proprietary driver as `F64NativeNoSharedMem` | barraCuda | Open |
| **P1** | Runtime GPU reduction smoke test in `ComputeDispatch` | toadStool | Open |
| **P2** | GPU tridiagonal eigenvector solver | barraCuda | Open |
| **P2** | End-to-end coralReef sovereign path for f64 reductions | toadStool + coralReef | New — enabled by Iteration 10 fix |
