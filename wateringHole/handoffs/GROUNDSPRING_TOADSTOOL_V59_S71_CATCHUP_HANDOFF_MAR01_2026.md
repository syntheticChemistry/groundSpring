# groundSpring → ToadStool V59 Handoff: S71+++ Catch-Up

**Date**: March 1, 2026
**From**: groundSpring (V59)
**To**: ToadStool / BarraCUDA team
**ToadStool pin**: S71+++ (`8dc01a37`)
**License**: AGPL-3.0-or-later
**Supersedes**: V58 (Cross-Spring Evolution + Deep-Debt Completion)

---

## Executive Summary

- **ToadStool S71+++ absorbed**: 6 commits, ~9K net lines removed (stale code archived),
  ComputeDispatch builder migration (66 ops), DF64 transcendental suite complete
- **Jackknife GPU promoted**: `jackknife_mean_variance` now dispatches to GPU via
  `JackknifeMeanGpu` + `jackknife_mean_f64.wgsl` (leave-one-out means on GPU)
- **Hargreaves GPU evolved**: batch path now tries `HargreavesBatchGpu` (cleaner S71 API)
  before falling back to S70 `BatchedElementwiseF64`
- **61 active delegations**: 37 CPU + 20 GPU + 4 cross-spring (was 38+19+4)
- **613 workspace tests**, all PASS, all quality gates green

---

## Part 1: What ToadStool S71 Brings

### New GPU Shaders (groundSpring-relevant)

| Shader | Path | groundSpring Use |
|--------|------|-----------------|
| `kimura_fixation_f64.wgsl` | `shaders/bio/` | Available for batch — scalar `kimura_fixation_prob` uses CPU path |
| `hargreaves_batch_f64.wgsl` | `shaders/science/` | **Wired** via `HargreavesBatchGpu::dispatch` |
| `jackknife_mean_f64.wgsl` | `shaders/stats/` | **Wired** via `JackknifeMeanGpu::new().dispatch()` |

### New Dispatch Types

| Type | Module | Status |
|------|--------|--------|
| `KimuraGpu` | `stats::evolution` | Available, not consumed (scalar function doesn't batch) |
| `JackknifeMeanGpu` | `stats::jackknife` | **Consumed** — GPU-parallel leave-one-out |
| `HargreavesBatchGpu` | `stats::hydrology` | **Consumed** — primary GPU path for batch ET₀ |
| `HistogramGpu` | `stats::histogram` | Available, not consumed yet |

### ComputeDispatch Builder

S71 migrates 66 ops to a unified `ComputeDispatch::new(&device, "label").shader().dispatch()` builder.
This pattern replaces per-op boilerplate with a standardized 4-line dispatch:

```rust
ComputeDispatch::new(&device, "op_name")
    .shader(WGSL_SOURCE, "main")
    .storage_read(0, &input_buf)
    .storage_rw(1, &output_buf)
    .uniform(2, &params_buf)
    .dispatch_1d(element_count)
    .submit();
```

### DF64 Transcendental Suite

S71 completes the DF64 transcendental library (15 functions):
- `gamma_df64` (Lanczos g=7), `erf_df64` (Horner-form Taylor)
- Inverse trig: `asin_df64`, `acos_df64`, `atan_df64`, `atan2_df64`
- Hyperbolic: `sinh_df64`, `cosh_df64`, `tanh_df64`
- Plus existing: `exp_df64`, `log_df64`, `sqrt_df64`, `sin_df64`, `cos_df64`, `pow_df64`

### Cleanup

- 671 WGSL shaders (corrected from 700 — stale shaders archived)
- ~9K lines net removed (placeholder examples, stale tests, dead code)
- libc in akida-driver identified for future rustix evolution
- unsafe reduced in GPU device creation and unified memory

---

## Part 2: groundSpring Rewiring

### GPU Promotion: `jackknife_mean_variance`

**Before** (S70+++ pin):
```
barracuda CPU (if let Some) → local CPU fallback
```

**After** (S71+++ pin):
```
barracuda-gpu JackknifeMeanGpu (if Some) → barracuda CPU (if let Some) → local CPU fallback
```

The GPU path uses `jackknife_mean_f64.wgsl` which parallelizes the leave-one-out
means across GPU threads, then computes variance on CPU. For large arrays (N > 256),
GPU should be faster than the sequential CPU loop.

### GPU Evolution: `hargreaves_et0_batch`

**Before** (S70+++ pin):
```
BatchedElementwiseF64(Op::HargreavesEt0) → barracuda CPU batch → local CPU
```

**After** (S71+++ pin):
```
HargreavesBatchGpu (S71 cleaner API) → BatchedElementwiseF64 fallback → barracuda CPU → local CPU
```

### Unchanged

All other 59 delegations unchanged — working correctly at S70+++ level.

---

## Part 3: Quality State

| Gate | Status |
|------|--------|
| `cargo check` (default) | PASS |
| `cargo check --features barracuda` | PASS |
| `cargo check --features barracuda-gpu` | PASS |
| `cargo clippy -- -D warnings` | 0 warnings |
| `cargo fmt --check` | PASS |
| `cargo test --workspace` | 613 PASS |

---

*groundSpring V59 — ToadStool S71+++ Catch-Up — March 1, 2026*
