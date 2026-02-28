# groundSpring → ToadStool V37 Handoff: BarraCUDA Evolution & Absorption Summary

**Date**: February 27, 2026
**From**: groundSpring V35 (V37 docs)
**To**: ToadStool S68+ / BarraCUDA team
**License**: AGPL-3.0-or-later
**Previous**: V35 (Titan V / NAK adaptive GPU dispatch)

---

## Part 1: Executive Summary

groundSpring has completed its full validation arc — 28 experiments across 9
scientific domains, 288/288 Rust checks, 762 total tests, 32 active barracuda
delegations (25 CPU + 7 GPU), 9 pending ToadStool absorption, and live GPU
compute validated on both Titan V and RTX 4070. This handoff synthesizes
everything groundSpring has learned about BarraCUDA's evolution, what ToadStool
should absorb next, and the critical NAK/driver findings that affect all springs.

**Key numbers**:
- 28 experiments, 9 domains, 288/288 Phase 1 checks
- 32 active delegations (25 CPU + 7 GPU) + 9 pending
- 49 metalForge tests, 19 workloads, 5 substrates
- 11.5× faster than Python (excl. LAPACK), 47.7× peak (Sturm tridiag)
- 2.2× GPU speedup overall, 47.4× peak
- f32 GPU compute: Titan V 797 µs, RTX 4070 274 µs (Anderson Lyapunov)
- NAK f64 gap: both NAK (Volta) and NVVM (Ada) fail f64 shader compilation

---

## Part 2: BarraCUDA Delegation Inventory

### 25 Active CPU Delegations

| # | groundSpring | BarraCUDA Target | Session |
|---|---|---|---|
| 1 | `stats::pearson_r` | `stats::pearson_correlation` | Pre-S39 |
| 2 | `stats::spearman_r` | `stats::correlation::spearman_correlation` | Pre-S39 |
| 3 | `stats::sample_std_dev` | `stats::correlation::std_dev` | Pre-S39 |
| 4 | `stats::covariance` | `stats::correlation::covariance` | Pre-S39 |
| 5 | `stats::norm_cdf` | `stats::norm_cdf` | Pre-S39 |
| 6 | `stats::norm_ppf` | `stats::norm_ppf` | Pre-S39 |
| 7 | `stats::chi2_statistic` | `stats::chi2_decomposed` | Pre-S39 |
| 8 | `stats::rmse` | `stats::metrics::rmse` | S64 |
| 9 | `stats::mbe` | `stats::metrics::mbe` | S64 |
| 10 | `stats::r_squared` | `stats::metrics::r_squared` | S64 |
| 11 | `stats::index_of_agreement` | `stats::metrics::index_of_agreement` | S64 |
| 12 | `stats::hit_rate` | `stats::metrics::hit_rate` | S64 |
| 13 | `stats::mean` | `stats::metrics::mean` | S64 |
| 14 | `stats::percentile` | `stats::metrics::percentile` | S64 |
| 15 | `bootstrap::bootstrap_mean` | `stats::bootstrap_mean` | Pre-S39 |
| 16 | `bootstrap::rawr_mean` | `stats::rawr_mean` | S66 |
| 17 | `rarefaction::shannon_diversity` | `stats::diversity::shannon` | S64 |
| 18 | `rarefaction::evenness` | `stats::pielou_evenness` | S64 |
| 19 | `anderson::analytical_localization_length` | `special::anderson_transport::localization_length` | S52 |
| 20 | `bistable::bistable_derivative` | `numerical::ode_bio::BistableOde::cpu_derivative` | S58 |
| 21 | `multisignal::multisignal_derivative` | `numerical::ode_bio::MultiSignalOde::cpu_derivative` | S58 |
| 22 | `kinetics::hill` | `stats::hill` | S68 |
| 23 | `wdm::finite_size_extrapolate` | `stats::regression::fit_linear` | S66 |
| 24 | `drift::kimura_fixation_prob` | `stats::kimura_fixation` | S66 |
| 25 | `fao56::daily_et0` | `stats::hydrology::fao56_et0` | S66 |

### 7 Active GPU Delegations

| # | groundSpring | BarraCUDA Target | Speedup |
|---|---|---|---|
| 26 | `anderson::lyapunov_exponent` | `spectral::lyapunov_exponent` | — |
| 27 | `anderson::lyapunov_averaged` | `spectral::lyapunov_averaged` | — |
| 28 | `almost_mathieu::hamiltonian` | `spectral::almost_mathieu_hamiltonian` | — |
| 29 | `almost_mathieu::level_spacing_ratio` | `spectral::level_spacing_ratio` | — |
| 30 | `almost_mathieu::eigenvalues` | `spectral::find_all_eigenvalues` | **47.7×** |
| 31 | `spectral_recon::tikhonov_solve` | `linalg::solve_f64_cpu` | 1.7× |
| 32 | `jackknife::jackknife_mean_variance` | `stats::jackknife_mean_variance` | — |

### 9 Pending ToadStool Absorption

These are commented out in groundSpring code with `TODO(toadstool)`:

| groundSpring | Expected BarraCUDA Target | Blocker |
|---|---|---|
| `drift::kimura_fixation_prob` (GPU) | `stats::kimura_fixation` (GPU path) | CPU-only in barracuda |
| `jackknife::jackknife_mean_variance` (GPU) | `stats::jackknife_mean_variance` (GPU path) | Embarrassingly parallel |
| `fao56::daily_et0` (batch GPU) | `stats::hydrology::fao56_et0_batch` | Batch dispatch adapter |
| `freeze_out::grid_fit_2d` | `ops::grid::grid_fit_2d_f64` | Not yet in barracuda |
| `band_structure::find_band_edges` | `spectral::band_edges_parallel` | Not yet in barracuda |
| `seismic::grid_search_inversion` | `ops::grid::grid_search_3d_f64` | Not yet in barracuda |
| `quasispecies::quasispecies_simulation` | `ops::bio::wright_fisher_simulate` | Batch replicate dispatch |
| `rare_biosphere::abundance_occupancy` | `ops::bio::batched_multinomial_occupancy` | Signature mismatch |
| `rare_biosphere::tier_detection_rate` | `ops::bio::batched_multinomial_tier_rate` | Signature mismatch |

---

## Part 3: Architecture-Aware GPU Dispatch (V35)

### What We Built

metalForge now detects GPU architecture at runtime via `GpuArch::from_name()`
and routes f64 workloads to the best available GPU:

```
Workload requires F64Compute?
  → Yes: prefer NativeF64 GPU (Titan V, 1:2) → fallback to any GPU → NPU → CPU
  → No:  prefer any GPU → NPU → CPU
```

### `AdaptiveBatch` for GPU Memory

Software-side batch sizing that accounts for VRAM and architecture:

| Architecture | f64 Ratio | Workgroup | VRAM Default | Resident Memory |
|---|---|---|---|---|
| Volta (GV100) | 1:2 | 64 | 12 GB | Yes (HBM2) |
| Ada (AD104) | 1:64 | 256 | 12 GB | No |
| Ampere | 1:64 | 256 | 10 GB | No |
| Turing | 1:32 | 128 | 8 GB | No |

VRAM defaults used when `wgpu` reports API-maximum (`max_buffer_size` ~2^57)
instead of actual device memory (common on NVK/NAK and some Mesa drivers).

### Live GPU Compute Results

Anderson Lyapunov (L=200, W=2.0, 1024 realizations):

| Substrate | γ | ξ | Time | Precision |
|---|---|---|---|---|
| Titan V (NVK/NAK) | 0.0386 | 25.90 | 797 µs | f32 |
| RTX 4070 (NVVM) | 0.0386 | 25.90 | 274 µs | f32 |
| CPU reference | 0.0406 | 24.61 | 6341 µs | f64 |

f32-vs-f64 precision delta: **5.0%** — validates the need for DF64.

---

## Part 4: Critical NAK/Driver f64 Findings

### The Problem

Both NAK (Volta/NVK) and NVVM (Ada/proprietary) advertise `SHADER_F64` but
**cannot compile f64 WGSL compute shaders**:

- **NAK**: `from_nir.rs:1092: assertion failed: alu.def.bit_size() == 32`
  — NIR→NAK conversion assumes 32-bit ALU ops
- **NVVM**: `NVVM compilation failed: 1`
  — proprietary compiler rejects f64 compute on consumer Ada

### The Implication

**DF64 (double-float f32-pair) is required on ALL current GPUs**, not just
consumer Ada. Even the Titan V with native 1:2 f64 hardware cannot use f64
WGSL shaders through NVK/NAK. ToadStool's `df64_rewrite.rs` is the only
viable path for production f64 precision on GPU.

### Recommended Fallback Chain

```
WGSL f64 → try compile → if fails → DF64 rewrite → f32 pairs → identical precision
```

groundSpring's `probe_f64_pipeline()` in `validate_metalforge_titan_v.rs`
demonstrates this pattern with `std::panic::catch_unwind`.

### NAK Evolution Opportunities

1. **f64 ALU lowering**: NAK's `from_nir.rs` needs 64-bit ALU operation
   support. Volta hardware (SM 7.0) has native f64 at 1:2 throughput — this
   is a compiler gap, not a hardware limitation.
2. **Runtime f64 probe**: Don't trust `SHADER_F64` feature flag. Try
   compiling a minimal f64 shader and fall back to DF64 on failure.
3. **`max_buffer_size` reporting**: NVK reports API maximum (~2^57 bytes)
   instead of actual VRAM. Use `vkGetPhysicalDeviceMemoryBudgetPropertiesEXT`
   or conservative architecture defaults.

---

## Part 5: What ToadStool Should Absorb

### Priority 1: Grid Search Ops (3 new GPU kernels)

groundSpring has 3 embarrassingly-parallel grid searches that need GPU dispatch:

1. **`grid_fit_2d_f64`** — freeze-out temperature (T₀, κ₂) grid. Each point
   is independent. CPU reference: `freeze_out::grid_fit_2d()`.
2. **`grid_search_3d_f64`** — seismic source localization (lat, lon, depth).
   Each grid point is independent. CPU reference: `seismic::grid_search_inversion()`.
3. **`band_edges_parallel`** — energy scan (10,001 points). One thread per
   energy, L sequential 2×2 matrix multiplies per thread. CPU reference:
   `band_structure::find_band_edges()`.

All three use flat `Vec<f64>` layouts and are ready for GPU dispatch.

### Priority 2: Batched Bio Ops (2 signatures)

1. **`batched_multinomial_occupancy`** — abundance-occupancy via batched
   multinomial sampling. barracuda has `BatchedMultinomialGpu` but the
   signature uses cumulative_probs + closure RNG.
2. **`wright_fisher_simulate`** — batched Wright-Fisher across replicates.
   barracuda has `WrightFisherGpu` from S66.

### Priority 3: DF64 Fallback as Default

Based on our NAK findings, `df64_rewrite.rs` should be the **default** path
for f64 precision, not an optional fallback. The `SHADER_F64` feature flag
is unreliable on both open-source (NAK) and proprietary (NVVM) drivers.

### Priority 4: PRNG Alignment

groundSpring still uses `prng::Xorshift64` as the default PRNG.
`Xoshiro128StarStar` is implemented and API-complete (next_u32, next_u64,
next_f64, next_normal, binomial) but not yet the default. Switching requires
regenerating all 28 benchmark baselines.

---

## Part 6: Production WGSL Shaders for Absorption

### `metalForge/shaders/batched_multinomial.wgsl` (148 lines)

Batched multinomial sampling for rarefaction and rare biosphere experiments.
Binding layout, dispatch geometry, and CPU reference in header comment.
Uses xoshiro128** PRNG matching `barracuda::ops::prng_xoshiro_wgsl`.

### `metalForge/shaders/mc_et0_propagate.wgsl` (113 lines)

Monte Carlo noise propagation through FAO-56 ET₀. Box-Muller perturbation
on each input parameter. Complements the already-absorbed `Op::Fao56Et0`.

### `metalForge/shaders/anderson_lyapunov.wgsl` (f64) and `anderson_lyapunov_f32.wgsl`

Anderson Lyapunov exponent via transfer matrix. f64 version requires DF64
rewrite (NAK/NVVM f64 gap). f32 version validated on both Titan V and RTX 4070.

---

## Part 7: Unidirectional Streaming (Barracuda GPU Path)

The adaptive batch infrastructure enables "data goes to GPU once, stays in
HBM2/GDDR6, results come back":

1. `AdaptiveBatch::for_gpu()` computes max batch elements from VRAM
2. `use_resident_memory = true` on Volta (HBM2) — buffers persist
3. `native_f64 = true` on Volta — route f64 workloads here
4. `workgroup_size` tuned per architecture (64 for Volta, 256 for Ada)

This reduces dispatch round-trips for batch workloads (the 19 metalForge
workloads are all batched). The key win: intermediate results stay on-device
between kernel dispatches instead of bouncing through PCIe.

---

## Part 8: Cross-Spring Learnings for ToadStool

### 1. `if let Ok` + CPU Fallback

Every barracuda delegation in groundSpring uses:
```rust
#[cfg(feature = "barracuda")]
if let Ok(result) = barracuda::stats::some_op(args) {
    return result;
}
// CPU fallback always compiled
local_cpu_implementation(args)
```

This pattern is now wateringHole standard. CPU fallback is never behind
`#[cfg(not(feature = "barracuda"))]` — it's always compiled and always
available. Zero-cost when barracuda succeeds; graceful degradation when it
fails.

### 2. Feature Flag Reliability

`SHADER_F64` is advertised by both NAK and NVVM but neither can actually
compile f64 shaders. ToadStool should:
- Never trust feature flags alone for precision decisions
- Use runtime probe (try compile → catch failure → fall back)
- Default to DF64 for all f64 workloads

### 3. `max_buffer_size` Inflation

NVK reports `max_buffer_size` as the Vulkan API maximum (~2^57 bytes) instead
of actual VRAM. groundSpring's `AdaptiveBatch` falls back to conservative
architecture defaults when the reported value exceeds 64 GB. ToadStool should
apply similar sanity checking.

### 4. Workgroup Sizing by Architecture

Volta (64-wide warps, 2 f32 FMA units per SM) benefits from 64-wide
workgroups. Ada (128-wide, 4 FMA units) benefits from 256-wide. NAK may
have different optimal sizing than NVVM.

### 5. Three-Mode Validation Pattern

groundSpring validates all 28 experiments in three modes:
- `cargo test` (local CPU)
- `cargo test --features barracuda` (CPU delegation)
- `cargo test --features barracuda-gpu` (GPU delegation)

All produce identical mathematical results. This three-mode pattern should
be adopted by all springs for barracuda integration testing.

---

## Part 9: Validation Summary

```
cargo clippy --workspace --all-features -- -D warnings  → 0 warnings
cargo test --workspace                                    → 410/410 PASS
cargo test --workspace --features biomeos                 → 442/442 PASS
cargo test -p groundspring-forge                          → 49/49 PASS
python3 -m pytest tests/                                  → 320/320 + 2 skip
validate-metalforge-inventory                             → 14/14 PASS
validate-metalforge-gpu                                   → 11/11 PASS
validate-metalforge-titan-v                               → 13/13 PASS
validate-metalforge-cross-substrate                       → 10/10 PASS
28 validation binaries                                    → 288/288 PASS
```

---

## Part 10: Evolution Roadmap

```
Phase 0 (DONE)     Phase 1 (DONE)       Phase 2a (DONE)         Phase 2b (DONE)
Python baselines → Rust validation    → barracuda CPU          → barracuda GPU
28 experiments     288/288 checks       32 active delegations    7 GPU, 47.7× peak
320 Python tests   762 total tests      25 CPU + 7 GPU           three-tier parity

Phase 3 (IN PROGRESS)                Phase 4 (NEXT)
metalForge cross-substrate         → ToadStool absorption
19 workloads, 5 substrates            9 pending delegations
arch-aware routing (Titan V)          3 grid search ops
49 metalForge tests                   DF64 as default
NAK f64 gap confirmed                 PRNG alignment
```

**Next milestone**: ToadStool absorbs the 3 grid search ops + 2 bio batch
ops. groundSpring rewires with `TODO(toadstool)` stubs → live GPU dispatch.
NAK f64 ALU patch would unlock Titan V's native 1:2 throughput for all springs.
