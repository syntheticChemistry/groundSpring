# groundSpring metalForge

**Write → Absorb → Lean** artifacts for GPU evolution.

## Purpose

metalForge is where groundSpring writes GPU-ready implementations for
absorption into ToadStool/BarraCUDA, following the pattern established
by hotSpring. We write production-quality WGSL shaders locally, validate
them against CPU references, then hand off via `wateringHole/handoffs/`.

groundSpring's metalForge is focused on **statistical compute kernels** —
the measurement noise primitives that underpin all five experiments.
Hardware discovery and substrate dispatch are handled by hotSpring's
metalForge (shared infrastructure).

## Structure

```
metalForge/
├── README.md                    # This file
├── ABSORPTION_MANIFEST.md       # Module-by-module absorption inventory
└── shaders/                     # Production WGSL shaders for absorption
    ├── mc_et0_propagate.wgsl    # Monte Carlo FAO-56 propagation (149 lines)
    └── batched_multinomial.wgsl # Batched multinomial rarefaction (112 lines)
```

## Current Status (Phase 2a — barracuda CPU delegation)

| Phase | Count | Examples |
|-------|------:|---------|
| **Lean** (delegated to barracuda) | 6 | `pearson_r`, `spearman_r`, `sample_std_dev`, `bootstrap_mean`, `lyapunov_exponent`, `lyapunov_averaged` |
| **Ready** (GPU op exists, needs adapter) | 6 | `rmse`, `mbe`, `r_squared`, `ia`, `hit_rate`, `shannon_diversity` |
| **Absorbed upstream** | 1 | `fao56_et0_batch` (ToadStool S49) |
| **Write** (WGSL ready for absorption) | 2 | `batched_multinomial`, `mc_et0_propagate` |
| **Write** (local CPU, needs kernel) | 2 | `rawr_mean`, `birth_death_ssa` |
| **Adapt** (needs alignment) | 2 | PRNG xoshiro, grid search |
| **Stays local** | 5 | Scalar ops, harness |

## Write → Absorb → Lean Cycle

### 1. Write (current)

Every module in `crates/groundspring/` has a pure safe Rust CPU
implementation that serves as the validation reference. WGSL shaders
in `metalForge/shaders/` are production-quality with:

- `struct Params` for uniforms (u32-aligned with padding)
- `@group(0) @binding(N)` sequential bindings
- `@compute @workgroup_size(64, 1, 1)` standard workgroup
- xoshiro128** PRNG matching `barracuda::ops::prng_xoshiro_wgsl`
- f64 precision throughout
- Documented binding layouts and dispatch geometry

### 2. Absorb

ToadStool reviews handoff and absorbs:
1. WGSL shader → `barracuda::ops::{module}`
2. Rust op struct → `barracuda::ops::{module}.rs`
3. Tests → `barracuda::tests/`

### 3. Lean

After absorption, groundSpring rewires:
1. Add `#[cfg(feature = "barracuda")]` delegation path
2. Delete local shader
3. Run validation binaries to confirm

## WGSL Shader Conventions

Following hotSpring's pattern:

- **Naming**: `{operation}_{domain}.wgsl` (e.g. `batched_multinomial.wgsl`)
- **License**: `// SPDX-License-Identifier: AGPL-3.0-or-later`
- **Bindings**: group 0 only, sequential, documented in header
- **PRNG**: xoshiro128** with `vec4<u32>` state per invocation
- **Precision**: f64 for all scientific compute
- **CPU reference**: documented path to Rust implementation

## BarraCUDA Primitives We Lean On

| groundSpring | barracuda op | Status |
|---|---|---|
| `stats::pearson_r` | `stats::pearson_correlation` | **Done** (CPU) |
| `stats::spearman_r` | `stats::correlation::spearman_correlation` | **Done** (CPU) |
| `stats::sample_std_dev` | `stats::correlation::std_dev` | **Done** (CPU) |
| `bootstrap::bootstrap_mean` | `stats::bootstrap_mean` | **Done** (CPU) |
| `anderson::lyapunov_exponent` | `spectral::lyapunov_exponent` | **Done** (barracuda-gpu) |
| `anderson::lyapunov_averaged` | `spectral::lyapunov_averaged` | **Done** (barracuda-gpu) |
| `stats::rmse` | `ops::NormReduceF64::l2` | Pending GPU adapter |
| `stats::mbe` | `ops::SumReduceF64::mean` | Pending GPU adapter |
| `rarefaction::shannon_diversity` | `ops::FusedMapReduceF64::shannon_entropy` | Pending GPU adapter |
| `fao56::daily_et0` | `ops::BatchedElementwiseF64::fao56_et0_batch` | **Absorbed** upstream |

## New Kernels for Absorption (Tier C)

| Shader | Status | Key Detail |
|---|---|---|
| `batched_multinomial.wgsl` | **Production** | xoshiro PRNG + binary search over cumulative probs |
| `mc_et0_propagate.wgsl` | **Production** | Equation chain superseded by `Op::Fao56Et0`; MC noise wrapper still needed |

See `ABSORPTION_MANIFEST.md` for binding layouts, dispatch geometry, and
the full module-by-module absorption inventory.
