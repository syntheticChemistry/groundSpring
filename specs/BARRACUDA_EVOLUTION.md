# BarraCUDA Evolution Mapping

> groundSpring Rust module → BarraCUDA primitive → WGSL shader → pipeline stage

## Philosophy

groundSpring's Rust crate provides **pure-Rust CPU implementations** of all
algorithms.  When the `barracuda` feature is enabled, operations delegate to
BarraCUDA's hardware-agnostic tensor compute layer, which runs the same WGSL
shaders on GPU, CPU, NPU, or TPU via wgpu backend selection.

The integration follows the **capability discovery** pattern: groundSpring
does not hardcode knowledge of BarraCUDA's location.  It exposes an optional
Cargo feature that, when enabled, brings in the dependency.

## Module Mapping

### Tier A — Direct Rewire (ready now)

These modules map 1:1 to existing BarraCUDA operations.

| groundSpring Module | BarraCUDA Primitive | WGSL Shader | Status |
|---|---|---|---|
| `stats::rmse` | `ops::norm_reduce_f64` | `norm_reduce_f64.wgsl` | Ready |
| `stats::mbe` | `ops::mean` | `mean.wgsl` | Ready |
| `stats::r_squared` | `ops::variance_f64_wgsl` + reduce | `variance_f64.wgsl` | Ready |
| `stats::index_of_agreement` | `ops::fused_map_reduce_f64` | `fused_map_reduce_f64.wgsl` | Ready |
| `decompose::decompose_error` | Scalar math, no GPU needed | N/A | CPU-only |
| `decompose::noise_floor_reduction` | Scalar math, no GPU needed | N/A | CPU-only |

### Tier B — Adapt (needs wrapper)

| groundSpring Module | BarraCUDA Target | Notes |
|---|---|---|
| `rarefaction::multinomial_sample` | `ops::prng_xoshiro_wgsl` + `sample::*` | GPU PRNG + cumulative sum + binary search. Needs custom kernel for batched multinomial. |
| `rarefaction::shannon_diversity` | `ops::fused_map_reduce_f64` | Map p→ -p·ln(p), then reduce-sum. |
| `seismic::haversine_km` | `ops::spherical_harmonics_f64_wgsl` (partial) | Could use trig WGSL, but scalar haversine is fast enough on CPU. |
| `seismic::grid_search_inversion` | `optimize::nelder_mead_gpu` | After grid search, Nelder-Mead refinement maps directly. Grid search itself is embarrassingly parallel → GPU. |

### Tier C — New Kernel Required

| groundSpring Module | Proposed BarraCUDA Kernel | Notes |
|---|---|---|
| Monte Carlo ET₀ propagation | `ops::mc_propagate_f64` | Batched: generate N perturbed input sets, run equation chain, collect statistics. Classic GPU workload. |
| Batch rarefaction (many depths × many replicates) | `ops::batched_multinomial_f64` | Each workgroup handles one replicate at one depth. |

## Feature Gate

In `Cargo.toml`:
```toml
[features]
default = []
barracuda = ["dep:barracuda"]

[dependencies]
barracuda = { path = "../../phase1/toadstool/crates/barracuda", optional = true }
```

In code:
```rust
#[cfg(feature = "barracuda")]
pub mod gpu;

// GPU-accelerated stats when available
#[cfg(feature = "barracuda")]
pub use gpu::stats_gpu;
```

## What NOT to Duplicate

The following BarraCUDA primitives already exist and MUST NOT be reimplemented
in groundSpring:

- Variance computation (`ops::variance_f64_wgsl`)
- Covariance (`ops::covariance_f64_wgsl`)
- Correlation (`ops::correlation_f64_wgsl`)
- Fused map-reduce (`ops::fused_map_reduce_f64`)
- Nelder-Mead optimization (`optimize::nelder_mead_gpu`)
- PRNG (`ops::prng_xoshiro_wgsl`)
- Shannon/Bray-Curtis diversity (`ops::bray_curtis_f64`)

groundSpring's CPU implementations serve as **validation references** for these
GPU kernels — they must produce identical results within documented tolerances.

## GPU Promotion Blockers

1. **No wgpu in groundSpring yet** — needs `barracuda` feature gate
2. **Multinomial sampling kernel** — does not exist in BarraCUDA, needs Tier C development
3. **Monte Carlo equation chain** — FAO-56 is complex; need to verify numerical stability in f32 on GPU
4. **f64 precision** — some WGSL shaders use f32; groundSpring requires f64 for statistical validity

## Timeline

| Phase | Milestone | Target |
|---|---|---|
| Phase 0 | Python baselines (current) | Done |
| Phase 1a | Rust CPU validation (current) | In progress |
| Phase 1b | Feature-gate barracuda, Tier A rewire | Next |
| Phase 2 | Tier B adapters, Tier C kernels | After Phase 1b |
| Phase 3 | Full GPU pipeline, performance benchmarks | After Phase 2 |
