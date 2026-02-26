# groundSpring → ToadStool/BarraCUDA V20 — S68 Catch-up + Hill Delegation

**Date**: February 26, 2026
**From**: groundSpring (noise validation + environmental modeling)
**To**: ToadStool/BarraCUDA team
**Previous**: V19 (uncertainty bridge + zero #[allow])
**ToadStool HEAD**: `f0feb226` (S68 — universal precision, zero f32-only shaders)
**groundSpring HEAD**: `67c1ab1` (V19)
**License**: AGPL-3.0-or-later

---

## Executive Summary

- **Hill kinetics delegation wired** — `kinetics::hill` now delegates directly to `barracuda::stats::hill` (infallible `f64` return) via `#[cfg]`/`#[cfg(not)]` mutual exclusion. Delegation #27 is live. `hill_repress` rewired to `1.0 - hill(x, k, n)`.
- **ToadStool S67-S68 reviewed** — 19 commits: universal precision architecture (S67), all 291 f32 shaders evolved to f64 canonical (S68), zero f32-only shaders remaining, 12 stale docs archived, 122 shader tests.
- **CPU-only feature gate bug found** — `barracuda::stats::mod.rs` lines 51-58 reference `crate::shaders::precision` without `#[cfg(feature = "gpu")]`, preventing `--features barracuda` compilation without GPU deps. groundSpring's CPU-only path (no features) still works fine; barracuda feature gate needs ToadStool fix before delegation testing.
- **226 tests, 185/185 checks, 98.93% coverage** — all green without barracuda features.

---

## Part 1: ToadStool S67-S68 Evolution Review

### S67: Universal Precision Architecture

- `compile_shader_universal(source, precision)` for f32/f64/df64
- `Precision::Df64` variant added
- `downcast_f64_to_f32()` and `downcast_f64_to_f32_with_transcendentals()`
- 12 universal shader templates

### S68: Precision Bottleneck Execution

- **291 f32 shaders** evolved to f64 canonical across 11 waves
- **Zero f32-only shaders remaining** — all use `LazyLock` downcast
- Dual-layer precision: `op_preamble` + naga IR rewrite
- F16 downcast with sentinel protection (±65504.0 clamping)
- 5 duplicate shader pairs merged
- 12 stale documentation files archived (-2,156 lines)
- 122 shader tests (unit + e2e + chaos + fault)
- `println!` → `tracing::info!` (14 sites), magic numbers → constants (5)

### Impact on groundSpring

| Aspect | Before (S66) | After (S68) | Impact |
|--------|-------------|-------------|--------|
| Shaders | 707, some f32-only | 700, all f64-canonical | groundSpring f64 math always uses native precision |
| Precision | Mixed f32/f64 | Universal f64 with downcast | No precision surprises in delegated code |
| Tests | 2,541 | 2,546+ | More coverage of paths groundSpring delegates to |
| hill() | Existed, untested from gS | Tested (4 tests) | Ready for delegation |

---

## Part 2: What groundSpring Rewired

### Hill Kinetics Delegation (#27)

```rust
// Before (V19 — stubbed, expected Result):
#[cfg(feature = "barracuda")]
{
    if let Ok(v) = barracuda::stats::hill(x, k, n) {
        return v;
    }
}
hill_cpu(x, k, n)

// After (V20 — direct, infallible):
#[cfg(feature = "barracuda")]
return barracuda::stats::hill(x, k, n);
#[cfg(not(feature = "barracuda"))]
hill_cpu(x, k, n)
```

`hill_repress` simplified to `1.0 - hill(x, k, n)`, getting barracuda delegation for free through the activating form.

**Delegation count**: 27 active (22 CPU + 5 GPU), up from 26 (21 + 5).

---

## Part 3: CPU Feature Gate Bug

ToadStool S68 introduced a compile-time issue for CPU-only consumers:

```rust
// crates/barracuda/src/stats/mod.rs:51-58
pub const WGSL_BOOTSTRAP_MEAN_F64: &str = include_str!("../shaders/special/bootstrap_mean_f64.wgsl");

pub static WGSL_HISTOGRAM: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    crate::shaders::precision::downcast_f64_to_f32_with_transcendentals(include_str!(
        "../shaders/stats/histogram_f64.wgsl"
    ))
});
```

These reference `crate::shaders` which is gated behind `#[cfg(feature = "gpu")]`. Without the `gpu` feature, `cargo check --features barracuda` fails with:

```
error[E0433]: could not find `shaders` in the crate root
```

Similarly in `numerical/mod.rs:64`.

**Impact**: groundSpring cannot test barracuda CPU delegation until this is fixed. Our no-features path (pure CPU) is unaffected and all 226 tests pass.

**Recommended fix**: Gate these constants behind `#[cfg(feature = "gpu")]`:

```rust
#[cfg(feature = "gpu")]
pub const WGSL_BOOTSTRAP_MEAN_F64: &str = ...;

#[cfg(feature = "gpu")]
pub static WGSL_HISTOGRAM: std::sync::LazyLock<String> = ...;
```

---

## Part 4: Updated Delegation Matrix

| # | Function | Module | Tier | Status |
|---|----------|--------|------|--------|
| 1-15 | stats metrics (rmse, mbe, r², etc.) | stats/metrics | CPU | Active |
| 16-20 | correlation (pearson, spearman, cov, std_dev, norm_cdf/ppf) | stats/correlation | CPU | Active |
| 21 | chi2_statistic | stats/chi2 | CPU | Active |
| 22-23 | bootstrap_mean, rawr_mean | bootstrap | CPU | Active |
| 24-25 | lyapunov_exponent, lyapunov_averaged | spectral | CPU | Active |
| 26 | analytical_localization_length | spectral | CPU | Active |
| **27** | **hill** | **stats/metrics** | **CPU** | **NEW (V20)** |
| G1-G3 | almost_mathieu_hamiltonian, eigenvalues, level_spacing_ratio | spectral | GPU | Active |
| G4-G5 | bistable_derivative, multisignal_derivative | ops/ode | GPU | Active |

---

## Part 5: Action Items for ToadStool

1. **Fix CPU feature gate** — `stats/mod.rs` lines 51-58 and `numerical/mod.rs` line 64 need `#[cfg(feature = "gpu")]` gating. This blocks all CPU-only spring consumers (groundSpring, airSpring) from testing barracuda delegation.

2. **Consume V18-V19 handoffs** — Still pending from groundSpring V7 (last consumed). Key items:
   - `kinetics::hill` delegation is now live (#27) — verify from ToadStool side
   - Flat buffer convention for eigenvector data (row-major `Vec<f64>`)
   - Determinism test pattern for stochastic algorithms
   - Exp 015 uncertainty bridge MC loop as batch dispatch candidate

3. **Eigenvector gap** — `transport::tridiag_eigh` returns flat eigenvectors. Barracuda's Sturm solver only handles eigenvalues. When eigenvector support is added, groundSpring can delegate immediately.

---

## Verification Commands

```bash
cargo test --workspace                        # 226 tests, all pass
cargo clippy --workspace -- -D warnings       # zero warnings
cargo run --bin validate-uncertainty-bridge    # 8/8 PASS

# Barracuda delegation (BLOCKED by ToadStool feature gate bug):
# cargo test --features barracuda             # fails: shaders module not found
```

---

*groundSpring V20 | February 26, 2026 | ToadStool S68 catch-up | 15 experiments, 226 tests, 185/185 checks, 27 delegations (22 CPU + 5 GPU)*
