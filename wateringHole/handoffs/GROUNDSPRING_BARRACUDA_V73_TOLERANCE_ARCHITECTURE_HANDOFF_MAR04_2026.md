# SPDX-License-Identifier: AGPL-3.0-only

# groundSpring → barraCuda/toadStool: V73 Tolerance Architecture + Idiomatic Evolution

**Date:** 2026-03-04
**From:** groundSpring V73
**To:** barraCuda team, toadStool team, ecoPrimals ecosystem
**Supersedes:** V72 (deep audit)
**barraCuda version:** v0.3.1 (standalone)
**groundSpring tests:** 790 passed, 0 failed
**License:** AGPL-3.0-only

---

## Executive Summary

- 13-tier named tolerance architecture (`tol::`) eliminates all bare float
  literals from test assertions — every tolerance carries semantic meaning
- Production epsilon guard module (`eps::`) replaces inline magic numbers in
  drift, gillespie, anderson production code
- ~170 bare tolerance replacements across 35 library modules + 6 integration
  test files
- Idiomatic Rust: `f64::midpoint`, 18 `return` → tail expressions, capability
  discovery evolution
- 97.25% library line coverage (target 90%), 790 workspace tests, all gates pass

---

## Part 1: Tolerance Architecture (Recommended for All Springs)

groundSpring now maintains 13 named tolerance tiers, each scientifically
justified. This pattern eliminates the "what does 1e-10 mean here?" problem.

| Constant | Value | Mathematical Regime |
|----------|-------|---------------------|
| `DETERMINISM` | 1e-15 | Same seed, same path, IEEE 754 rounding only |
| `STRICT` | 1e-14 | Compensated arithmetic / extended precision |
| `EXACT` | 1e-12 | Summation-only f64 paths |
| `ANALYTICAL` | 1e-10 | One transcendental (sqrt, ln) ~1 ULP |
| `INTEGRATION` | 1e-8 | ODE RK4 O(dt⁴) accumulation |
| `CDF_APPROX` | 1e-6 | CDF/erf approximation (A&S 7.1.26) |
| `ROUNDTRIP` | 1e-5 | CDF↔PPF round-trip (both approximations) |
| `RECONSTRUCTION` | 1e-4 | Spectral Tikhonov roundtrip RMSE |
| `LITERATURE` | 0.001 | Published 3–4 significant figures |
| `DECOMPOSITION` | 0.005 | Pythagorean identity RMSE² = MBE² + σ² |
| `STOCHASTIC` | 0.01 | O(1/√N) sampling noise |
| `NORM_2PCT` | 0.02 | ~2% normalization / integral tolerance |
| `EQUILIBRIUM` | 0.1 | ODE steady-state / measurement precision |

### Pattern for barraCuda/toadStool adoption

```rust
pub mod tol {
    pub const EXACT: f64 = 1e-12;
    pub const ANALYTICAL: f64 = 1e-10;
    // ... each with doc comment explaining the regime
}
```

Test code uses `tol::ANALYTICAL` instead of `1e-10`. When a tolerance changes
tier (e.g., GPU introduces an extra transcendental), you rename the constant
reference — the intent is visible in the diff.

### Production epsilon guards

Separate from test tolerances, `eps::` handles division safety:

| Constant | Value | Purpose |
|----------|-------|---------|
| `SAFE_DIV` | 1e-10 | Prevent NaN in `x / y.max(eps::SAFE_DIV)` |
| `SSA_FLOOR` | 1e-15 | Gillespie SSA steady-state guard |
| `UNDERFLOW` | 1e-300 | Condition number / matrix element guard |

---

## Part 2: Idiomatic Rust Patterns Applied

### `f64::midpoint` for overflow-safe averages

Spearman rank tie handling now uses `f64::midpoint(a, b)` instead of
`(a + b) / 2.0`. While overflow is unlikely for rank values, the intent
is clearer and the pattern is worth adopting in barraCuda's stats module.

### Tail expressions over explicit `return`

18 cfg-gated functions converted from:
```rust
#[cfg(feature = "barracuda")]
{ return barracuda::stats::rmse(a, b); }
#[cfg(not(feature = "barracuda"))]
{ return local_rmse(a, b); }
```
to:
```rust
#[cfg(feature = "barracuda")]
{ barracuda::stats::rmse(a, b) }
#[cfg(not(feature = "barracuda"))]
{ local_rmse(a, b) }
```

### Capability-based socket discovery

`discovery.rs` evolved from `NUCLEUS_SOCKET_NAMES` (hardcoded primal name) to
`CAPABILITY_SOCKET_NAMES` with `find_capability_socket()` fallback that scans
the directory for any `.sock` file. More robust for ecosystem evolution.

---

## Part 3: barraCuda Primitives Consumed (81 — unchanged)

No new delegations in V73. All 81 primitives (47 CPU + 34 GPU) from V72
remain active. See V72 handoff Part 2 for the full inventory.

---

## Part 4: Recommendations for barraCuda Evolution

### Tolerance module

barraCuda should adopt its own `tol::` module for internal tests. The tiers
may differ (GPU introduces additional error sources), but the pattern of
named, justified constants is the key insight.

### `eps::` guards

barraCuda's GPU kernels face the same division-by-zero risks. A shared
`eps::SAFE_DIV` in the Rust host code prevents NaN propagation before data
reaches the shader.

### `f64::midpoint`

Any statistics code computing means of two values should use `f64::midpoint`
for overflow safety. This is particularly relevant in GPU reduce ops where
partial sums can be large.

---

## Part 5: Quality Summary

| Gate | Status |
|------|--------|
| `cargo fmt --check` | PASS |
| `cargo clippy --workspace -- -D warnings` | PASS |
| `cargo doc --workspace --no-deps` | PASS |
| `cargo test --workspace` | PASS (790 tests) |
| `cargo llvm-cov` | 97.25% line coverage |
| 33/33 validation binaries | PASS (exit 0) |
| 28/28 Python parity | PASS |
| Zero bare tolerance literals in tests | PASS |
| Zero inline epsilon guards in lib | PASS |
| Zero unsafe | PASS |
| Zero todo!() / unimplemented!() | PASS |
| Zero production mocks | PASS |
| All files < 1000 lines | PASS |
| AGPL-3.0-only SPDX | PASS |

---

## Part 6: Provenance

| Metric | V72 | V73 | Change |
|--------|-----|-----|--------|
| barraCuda pin | v0.3.1 | v0.3.1 | Unchanged |
| Active delegations | 81 | 81 | Unchanged |
| Tests | 786+ | 790 | +4 (new tol module tests) |
| Named tolerance tiers | 9 (validate only) | 13 (library-wide) | +4 tiers, ~170 sites migrated |
| Production eps guards | 0 (inline) | 3 (named) | New `eps::` module |
| Library line coverage | ~97% | 97.25% | Stable |
| Bare float literals in tests | ~170 | 0 | Eliminated |

---

*groundSpring V73 — 790 tests, 33 validation binaries, 81 barracuda delegations.
Every tolerance has a name. Every name has a reason.*
