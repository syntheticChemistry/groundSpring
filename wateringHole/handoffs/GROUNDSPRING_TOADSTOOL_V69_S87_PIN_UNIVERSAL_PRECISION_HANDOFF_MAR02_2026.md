# groundSpring → ToadStool Handoff: V69 / S87 Pin + Universal Precision Audit

**Date**: March 2, 2026
**From**: groundSpring V69
**To**: ToadStool / BarraCUDA team
**ToadStool pin**: S87 (`2dc26792`)
**License**: AGPL-3.0-only
**Covers**: V68 → V69 (S86 → S87 pin update, universal precision documentation)

---

## Executive Summary

- **Pin bump**: S86 (`7e01ac7e`) → S87 (`2dc26792`) — FHE shader fix, async-trait reclassification, unsafe audit
- **76 active delegations** (44 CPU + 32 GPU), 0 breaking changes
- **780 tests** passed, 0 failed; zero unsafe / zero TODO / zero `.unwrap()` / zero `#[allow]` without reason
- **No rewiring needed** — S87 is a debt evolution commit with no public API changes
- **Universal precision architecture documented** — groundSpring acknowledges and benefits from S67-S68 evolution

---

## Part 1: S87 Changes Affecting groundSpring

| Change | Impact on groundSpring |
|--------|----------------------|
| FHE shader fix (fhe_ntt/fhe_intt/fhe_pointwise_mul) | None — groundSpring does not use FHE |
| async-trait 75 TODO(afit) → NOTE(async-dyn) | None — reclassification only |
| MatMul inner-dimension shape validation | None — groundSpring uses `barracuda::linalg` ops |
| BarracudaError::is_device_lost() + with_device_retry | Available for future GPU resilience |
| gpu_helpers.rs → 3 submodules (buffers, bind_group_layouts, pipelines) | Internal — no public API change |
| All ~60+ unsafe sites documented with SAFETY comments | Audit confidence — zero new unsafe |

---

## Part 2: Universal Precision Architecture (S67-S68)

groundSpring has audited and documented ToadStool's S67-S68 universal precision
evolution. This is the most significant architectural advancement in the barracuda
shader ecosystem.

### Design Principle

**"Math is universal, precision is silicon."**

One f64-canonical shader compiles to any precision via `compile_shader_universal()`.

### Compilation Pipeline

```
Shader source (f64 — true math)
       │
compile_shader_universal(source, precision)
       │
  ┌────┼────────────────┐
  F32  F64               Df64
  │    │                 │
  downcast             polyfill        df64_core +
  f64→f32              + ILP +         df64_transcendentals
                       sovereign
```

### Precision Tiers

| Tier | Pipeline | Hardware | Throughput |
|------|----------|----------|------------|
| F16 | `downcast_f64_to_f16` + clamp ±65504 | Mobile, edge | Highest |
| F32 | `downcast_f64_to_f32` (sentinel-protected) | Consumer GPUs | High |
| F64 | `compile_shader_f64` (driver patching, ILP) | Compute GPUs (Titan V, A100) | Native |
| Df64 | `compile_shader_df64` (double-float f32-pair) | Consumer GPUs needing f64-class | ~9.9× vs native f64 |

### Dual-Layer DF64

- **Layer 1 — Op Preamble**: `op_add`/`op_mul`/`Scalar` type alias → routes to `df64_add`/`df64_mul`
- **Layer 2 — Naga IR Rewrite**: `rewrite_f64_infix_full()` walks typed IR, replaces f64 `Binary{+,-,*,/}` with df64 bridge functions

### `Fp64Strategy` Auto-Selection

| Strategy | Use Case |
|----------|----------|
| Native | Compute-class GPUs (Titan V, A100, MI250) — 1:2 FP64:FP32 |
| Hybrid | Consumer GPUs (RTX 3090, 4070) — 1:64; DF64 for bulk math |
| Concurrent | Validation — run DF64 and native f64 side-by-side |

### Impact on groundSpring

**Transparent**: All barracuda ops (`eigh_f64`, `BatchedMultinomialGpu`, `lbfgs_numerical`,
`anderson_4d`, etc.) internally query `GpuDriverProfile::fp64_strategy()` and select the
optimal precision path. groundSpring's 76 delegations automatically benefit from the best
available precision on any hardware without code changes.

**Future**: If groundSpring ever compiles custom shaders, use `compile_shader_universal(src, precision)`.

---

## Part 3: Doc Comment Fix

- `almost_mathieu.rs` doc comments updated: `barracuda::spectral::hofstadter` → `barracuda::spectral` (hofstadter module is now private; functions re-exported at `spectral::` level)

---

## Part 4: Stale Reference Cleanup

| Category | Old | New | Files |
|----------|-----|-----|-------|
| ToadStool pin | S86 (`7e01ac7e`) | S87 (`2dc26792`) | 15+ markdown files |
| Graph TOMLs | S79 (`f97fc2ae`), V62 | S87 (`2dc26792`), V68 | 5 TOML files |
| Cross-spring shader categories | S79 | S87 | CROSS_SPRING_SHADER_EVOLUTION |
| CROSS_SPRING_EVOLUTION header | S79 (`f97fc2ae`) | S87 (`2dc26792`) | whitePaper/ |
| Benchmark binary header | S79 | S87 | benchmark_cross_spring.rs |
| BARRACUDA_EVOLUTION metrics | 743 tests / S79 | 780 tests / S87 | specs/ |

---

## Part 5: toadStool Actions

### P0 — Immediate

None. S87 is clean from groundSpring's perspective.

### P1 — Next S-Release

- **Device-retry API**: groundSpring could benefit from `BarracudaError::is_device_lost()` +
  `with_device_retry` for GPU resilience in long-running metalForge pipelines. Consider
  exposing a retry-aware dispatch wrapper.
- **`Fp64Strategy::Concurrent` for validation**: groundSpring's `three_tier_parity_gpu`
  tests could use Concurrent mode to run DF64 and native f64 side-by-side for
  precision delta measurement.

### P2 — Evolution

- **`compile_shader_universal` for metalForge custom workloads**: groundSpring's
  `metalForge/shaders/anderson_lyapunov.wgsl` could be compiled via
  `compile_shader_universal(src, Fp64Strategy)` instead of raw `compile_shader()`,
  enabling auto-precision selection per hardware.

---

## Provenance

| Metric | Value |
|--------|-------|
| ToadStool pin | S87 (`2dc26792`) |
| Active delegations | 76 (44 CPU + 32 GPU) |
| groundSpring tests | 780 |
| groundSpring validation checks | 376/376 |
| metalForge checks | 187 (130 forge + 57 mixed-hardware) |
| Python parity | 28/28 |
| Debt | Zero |

---

## Quality Gates

| Gate | Status |
|------|--------|
| `cargo fmt --check` | PASS |
| `cargo clippy --all-targets --all-features -- -W clippy::pedantic -W clippy::nursery` | PASS (zero warnings) |
| `cargo test --workspace` | 780 passed, 0 failed |
| `cargo doc --no-deps` | PASS |
| Zero unsafe | PASS |
| Zero TODO | PASS |
| Zero .unwrap() | PASS |
| Zero #[allow] without reason | PASS |
