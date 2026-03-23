# groundSpring V120 → barraCuda / toadStool Deep Audit Execution Handoff

**Date:** March 23, 2026
**From:** groundSpring V120
**To:** barraCuda team, toadStool team
**License:** AGPL-3.0-or-later

---

## Executive Summary

groundSpring V120 completes the deep audit execution begun in V119. The codebase now has
`#![forbid(unsafe_code)]` on all 50 binary entry points, a fully refactored dispatch module,
expanded validation harness, and zero clippy warnings across all feature combinations. This
handoff documents: (1) structural changes relevant to upstream, (2) deprecated API migration,
(3) patterns worth absorbing, (4) updated evolution priorities, and (5) items carried forward
from V119.

---

## 1. What Changed in V120

### 1a. `dispatch.rs` refactored into `dispatch/` module

The 823-line monolithic dispatch module was refactored into 4 focused submodules:

| Submodule | LOC | Responsibility |
|-----------|-----|----------------|
| `dispatch/mod.rs` | 257 | Router, re-exports, module-level tests |
| `dispatch/defaults.rs` | 101 | Named dispatch constants (regularization, window sizes, rates) |
| `dispatch/extract.rs` | 60 | JSON-RPC parameter extraction helpers |
| `dispatch/lifecycle.rs` | 78 | Health, capability, status methods |
| `dispatch/measurement.rs` | 388 | Domain-specific measurement implementations |

**Impact on upstream**: None — internal refactoring, no API change. Demonstrates the
"smart refactoring" pattern (semantic decomposition, not naive line-count splitting).

### 1b. `GpuDriverProfile` → `DeviceCapabilities` migration

`crates/groundspring/src/gpu.rs` now uses `barracuda::device::capabilities::DeviceCapabilities`
instead of the deprecated `barracuda::device::driver_profile::GpuDriverProfile`.

**Impact on upstream**: Confirms that `GpuDriverProfile` can be removed from barraCuda's
public API once all downstream consumers have migrated. groundSpring is now clean.

### 1c. `ValidationHarness` expanded

Two new comparison methods added:

- `check_relative(label, computed, expected, rel_tol)` — relative tolerance comparison
- `check_abs_or_rel(label, computed, expected, abs_tol, rel_tol)` — passes if either
  absolute or relative tolerance is satisfied

**Pattern for upstream**: These complement the existing `check(label, computed, expected,
abs_tol)` and could be useful as a shared validation primitive in `barracuda::test` or
as a cross-spring pattern via wateringHole.

### 1d. `#![forbid(unsafe_code)]` on all binary entry points

All 50 binary files now have `#![forbid(unsafe_code)]` after the copyright header:
- 37 validation binaries (`validate_*.rs`)
- 3 benchmark binaries (`bench_*.rs`)
- 10 metalForge binaries (`validate_metalforge_*.rs`, `validate_gpu_tier`, etc.)

This was already on the 3 library crate roots. Now the entire surface is covered.

### 1e. `capability_call_typed` added to biomeOS module

New public function for typed JSON-RPC capability invocation:

```rust
pub fn capability_call_typed(socket: &str, method: &str, params: Value) -> Result<Value, BiomeOsError>
```

Uses the previously dead-code `extract_rpc_result` — now a production code path.

### 1f. Release-mode CI validation

New `validate-release` CI job runs 7 validation binaries under `--release` (LTO +
`codegen-units=1`). Catches release-only numeric divergence (e.g., FMA reordering,
LTO inlining changing float evaluation order).

---

## 2. Upstream Issues (carried from V119, still open)

### 2a. Compile failure without `gpu` feature (P0 — unchanged)

```
error[E0433]: crate::ops::lattice::cpu_complex::Complex64
  --> barracuda/src/special/plasma_dispersion.rs:23

error[E0433]: crate::linalg::eigh::eigh_f64
  --> barracuda/src/spectral/stats.rs:142
```

**Fix needed**: Gate these imports behind `#[cfg(feature = "gpu")]` or extract
CPU-accessible parts from behind the `gpu` gate.

### 2b. `cast` module parity (P1 — unchanged)

groundSpring (and airSpring, wetSpring, healthSpring) all maintain local `cast` modules
with identical helpers. Consider promoting the full set to `barracuda::cast`.

### 2c. `GpuDriverProfile` deprecation cleanup (NEW — P2)

groundSpring has migrated to `DeviceCapabilities`. If all other consumers are migrated,
the deprecated `GpuDriverProfile` type can be removed.

---

## 3. Patterns Worth Absorbing (NEW in V120)

### 3a. Smart Module Refactoring Pattern

The `dispatch.rs` → `dispatch/` refactoring demonstrates semantic decomposition:
- Group by **responsibility** (lifecycle vs measurement vs parameter extraction), not by line count
- Keep the router (`mod.rs`) thin — it dispatches to submodules
- Named constants in their own file (`defaults.rs`) — easy to audit, easy to evolve
- Test colocation: each submodule can have its own `#[cfg(test)]` module

### 3b. Release-Mode CI for Numeric Code

Validation binaries should be tested under `--release` to catch FMA/LTO-induced
floating-point divergence. groundSpring's CI now does this for 7 key validators:
`validate_decompose`, `validate_anderson`, `validate_fao56`, `validate_transport`,
`validate_drift`, `validate_spectral_recon`, `validate_et0_methods`.

### 3c. `check_relative` / `check_abs_or_rel` Pattern

For validation where absolute tolerance doesn't scale well (e.g., values spanning
many orders of magnitude), relative or hybrid abs-or-rel comparison is cleaner.

---

## 4. Updated Evolution Priorities

### For barraCuda

| Priority | Item | Status |
|----------|------|--------|
| **P0** | Fix non-`gpu` compile (plasma_dispersion + spectral/stats imports) | **Open** |
| **P1** | Promote full cast module to `barracuda::cast` | **Open** |
| **P1** | RAWR GPU kernel (Bayesian weighted resampling) | **Open** |
| **P2** | Remove deprecated `GpuDriverProfile` (groundSpring migrated) | **New** |
| **P2** | Sparse matrix-vector product (CSR SpMV) for Lanczos | **Open** |
| **P2** | Matrix exponentiation (general case) | **Open** |

### For toadStool

| Priority | Item | Status |
|----------|------|--------|
| **P1** | Absorb groundSpring's 2 remaining `anderson_lyapunov*.wgsl` reference shaders | **Open** |
| **P2** | Fused spectral pipeline (Lyapunov + level statistics in one dispatch) | **Open** |

### Cross-Spring GPU Sharing (unchanged from V119)

| Kernel | groundSpring Use | Also Benefits |
|--------|-----------------|---------------|
| Jackknife GPU | Leave-one-out resampling (Bazavov precision) | neuralSpring |
| Anderson 4D | 4D localization for lattice QCD proxy | hotSpring |
| Batched Nelder-Mead | Multi-start optimization | airSpring |
| Rarefaction GPU | Multinomial + diversity in one dispatch | wetSpring |

---

## 5. Quality Metrics (V120)

| Metric | Value |
|--------|-------|
| barraCuda delegations | 110 (67 CPU + 43 GPU) |
| Validation checks | 395/395 PASS |
| Library coverage | ≥92% |
| Workspace tests | 990+ |
| Clippy (default features) | 0 warnings (pedantic + nursery) |
| Clippy (all features) | 0 warnings (pedantic + nursery, -D warnings) |
| `cargo doc --all-features` | 0 warnings |
| Unsafe code | 0 — `#![forbid(unsafe_code)]` on **all 53 entry points** (3 lib roots + 50 binaries) |
| `.unwrap()` in library | 0 |
| `publish = false` | 3/3 crates |
| MSRV | 1.85 (Rust 2024 edition) |
| Files > 1000 LOC | 0 |
| Release-mode CI | 7 validation binaries under LTO |

---

## 6. Delegation Inventory Summary (unchanged from V119)

**67 CPU delegations** across: statistics (20+), regression (8), correlation (6),
distributions (3), special functions (4+), linear algebra (6), optimization (5),
biology (8), hydrology (7).

**43 GPU delegations** across: reduce ops (8), matrix ops (3), bio simulation (4),
ODE solvers (2), FFT (1), spectral (3), hydrology (2), infrastructure (4).

Full inventory in V119 handoff (unchanged).

---

Part of [ecoPrimals](https://github.com/syntheticChemistry) · AGPL-3.0-or-later
