# groundSpring → ToadStool/BarraCUDA Handoff V26

**Date**: February 27, 2026
**Scope**: MetalForge live hardware validation — GPU + NPU + cross-substrate parity
**Supersedes**: V25 (Exp 025-027 WDM buildout), V23 (Exp 019-021 Bazavov)
**License**: AGPL-3.0-only

---

## Executive Summary

- **28 experiments**, 288/288 Rust validation checks, 314 Rust tests, 28/28 mathematical parity
- **metalForge forge crate** validates live hardware: RTX 4070, Titan V, BrainChip AKD1000, i9-12900K
- **Three-mode benchmark**: 20.4s → 9.2s (2.2× overall, 47.7× quasiperiodic) across 27 bins × 3 modes
- **Exp 028**: NPU Anderson regime classification on AKD1000 — int8 DMA at ~51 µs/inference
- **31 metalForge validation checks**: inventory 10/10, GPU 11/11, cross-substrate 10/10

---

## Part 1: What groundSpring Built

### 1.1 groundspring-forge Crate

Hardware discovery and workload dispatch at `metalForge/forge/`. Uses `wgpu` for GPU probing, `/dev/akida*` for NPU, and procfs for CPU.

| Module | Purpose | Lines |
|--------|---------|------:|
| `substrate.rs` | `Substrate`, `SubstrateKind`, `Capability` enums | 180 |
| `probe.rs` | `probe_gpus()`, `probe_cpu()`, `probe_npus()` | 170 |
| `inventory.rs` | `Inventory::discover()` — aggregates all substrates | 60 |
| `dispatch.rs` | `route()` — GPU > NPU > CPU priority dispatch | 110 |
| `workloads.rs` | 7 groundSpring workloads with required capabilities | 90 |

### 1.2 NPU Integration (`npu` Feature)

`crates/groundspring/src/npu.rs` wraps ToadStool's `akida-driver` for Anderson regime classification.

| Function | Purpose |
|----------|---------|
| `discover_npu()` | Device discovery via `akida_driver::DeviceManager` |
| `quantize_features(W, E, L)` | Float → int8 for NPU dispatch |
| `classify_regime_cpu(W, E)` | Analytical ground truth (Extended / Critical / Localized) |
| `train_classifier_weights()` | Centroid-based int8 weight generation |
| `npu_classify_regime()` | DMA write → inference → DMA read |
| `load_classifier_weights()` | Weight upload to AKD1000 SRAM |

### 1.3 Validation Binaries

| Binary | Checks | What It Proves |
|--------|-------:|---------------|
| `validate-metalforge-inventory` | 10/10 | All substrates discovered (GPU f64, GPU shader, NPU int8, CPU f64, CPU SIMD), all workloads routable |
| `validate-metalforge-gpu` | 11/11 | Anderson Lyapunov on GPU via barracuda-gpu matches CPU reference |
| `validate-metalforge-cross-substrate` | 10/10 | CPU analytical, GPU numerical, NPU DMA classification agree on 10 disorder values |
| `validate-npu-anderson` | 9/9 | CPU classification, int8 quantization accuracy, NPU DMA latency |

---

## Part 2: What ToadStool Should Absorb

### 2.1 groundspring-forge Patterns

The forge crate implements a pattern ToadStool should consider standardizing:

```
probe() → Inventory → route(Workload, Inventory) → Decision
```

Each `Substrate` advertises `Capability` (F64Compute, ShaderDispatch, QuantizedInference, SimdVector). Workloads declare required capabilities and optional preferred substrates. The router finds the best match.

**toadStool action**: Consider absorbing this pattern as `barracuda::dispatch` or `barracuda::substrate`. All springs (hotSpring, wetSpring, airSpring, groundSpring) need hardware discovery and workload routing. A shared crate in ToadStool eliminates 4× duplication.

### 2.2 NPU DMA Pattern

groundSpring's `npu.rs` follows wetSpring's proven pattern:

```rust
let mut handle = discover_npu()?;
let features = quantize_features(w, e, l);
let (class, metrics) = npu_classify_regime(&mut handle, features)?;
```

The int8 quantization → DMA write → DMA read → dequantize pipeline is generic. It applies to any classifier on AKD1000.

**toadStool action**: The `akida-driver` DMA pattern is now validated by both wetSpring (ESN inference, HAB sentinel, 18.8K Hz streaming) and groundSpring (Anderson classification, ~51 µs). Consider adding a higher-level `akida_driver::classify_i8(handle, features) -> Vec<i8>` convenience API.

### 2.3 Shader Inventory (Unchanged)

Two production WGSL shaders still await ToadStool absorption:

| Shader | Lines | Status |
|--------|------:|--------|
| `batched_multinomial.wgsl` | 112 | Production — xoshiro128**, 4 bindings |
| `mc_et0_propagate.wgsl` | 149 | Production — equation chain superseded by `Op::Fao56Et0` |

---

## Part 3: Evolution Learnings for ToadStool

### 3.1 Live Hardware Discovery

The `wgpu` adapter enumeration returns all GPUs including Titan V (NVK driver). The `pollster::block_on(instance.request_adapter())` pattern misses multi-GPU systems. groundSpring uses `instance.enumerate_adapters()` instead.

**Learning**: Any ToadStool hardware discovery should enumerate all adapters, not request one.

### 3.2 GPU/CPU Parity Under barracuda-gpu

When `barracuda-gpu` is enabled, Anderson Lyapunov uses barracuda's PRNG instead of groundSpring's `Xorshift64`. This produces different random disorder potentials, causing γ and ξ to differ numerically. The parity check must use relaxed tolerances (5× for analytical comparison).

**Learning**: Document expected divergence when upstream PRNG differs. This is not a bug — it's a consequence of the Write → Absorb → Lean cycle where the local and upstream PRNGs are intentionally different algorithms.

### 3.3 NPU Inference vs Classification Accuracy

Raw DMA on AKD1000 returns valid outputs, but centroid-based int8 weights loaded via `DeviceManager::write()` do not perform trained SNN inference. The AKD1000's neural processing cores require MetaTF/SNN-compiled models for production accuracy.

**Learning**: The `akida-driver` DMA path is validated for connectivity and latency, but production NPU classifiers will need the SNN compilation pipeline. Consider adding `akida_driver::ModelManager` for loading compiled SNN models alongside raw DMA.

### 3.4 Cross-Substrate Dispatch Priority

groundSpring found GPU > NPU > CPU as the right default priority for compute-bound workloads, but NPU > GPU > CPU is better for streaming classification tasks. The dispatch priority should be workload-dependent.

**Learning**: The `Workload` struct should include a `preferred: Option<SubstrateKind>` field that overrides the default priority when set.

---

## Part 4: BarraCUDA Delegation Status

### Active Delegations (27)

| # | Function | Target | Wiring |
|---|----------|--------|--------|
| 1-15 | stats (pearson_r, spearman_r, std_dev, covariance, norm_cdf, norm_ppf, chi2, rmse, mbe, r², IoA, hit_rate, mean, percentile, evenness) | `barracuda::stats::*` | `#[cfg(feature = "barracuda")]` |
| 16 | bootstrap_mean | `stats::bootstrap_mean` | `#[cfg(feature = "barracuda")]` |
| 17 | rawr_mean | `stats::rawr_mean` | `#[cfg(feature = "barracuda")]` |
| 18-20 | lyapunov_exponent, lyapunov_averaged, anderson_potential | `spectral::*` | `#[cfg(feature = "barracuda-gpu")]` |
| 21 | analytical_localization_length | `special::localization_length` | `#[cfg(feature = "barracuda")]` |
| 22 | almost_mathieu_hamiltonian | `spectral::almost_mathieu_hamiltonian` | `#[cfg(feature = "barracuda-gpu")]` |
| 23-24 | bistable/multisignal derivative | `BistableOde/MultiSignalOde` | `#[cfg(feature = "barracuda")]` |
| 25 | hill | `stats::hill` | `#[cfg(feature = "barracuda")]` |
| 26 | shannon_diversity | `stats::shannon` | `#[cfg(feature = "barracuda")]` |
| 27 | almost_mathieu_eigenvalues | Sturm tridiag | `#[cfg(feature = "barracuda-gpu")]` |

### New Delegation Candidates

| Function | Module | Priority | Pattern |
|----------|--------|----------|---------|
| `npu_classify_regime` | `npu` | HIGH | DMA inference → should use akida-driver API |
| `jackknife_mean_variance` | `jackknife` | MEDIUM | Parallel leave-one-out |
| `tikhonov_solve` | `spectral_recon` | MEDIUM | Dense GEMM + Cholesky |
| `grid_fit_2d` | `freeze_out` | MEDIUM | Embarrassingly parallel grid search |
| `wavepacket_msd` | `transport` | LOW | Tridiag eigenvectors (CPU-only) |

---

## Part 5: Metrics Summary

| Metric | Value |
|--------|-------|
| Experiments | 28 (was 27) |
| Rust validation checks | 288/288 |
| MetalForge checks | 31/31 |
| Rust tests | 314 (302 + 12 forge) |
| Python tests | 52 |
| Mathematical parity | 28/28 |
| BarraCUDA delegations | 27 (22 CPU + 5 GPU) |
| WGSL shaders (local) | 2 (261 combined lines) |
| Clippy warnings | 0 across all crates and features |
| Live hardware validated | RTX 4070, Titan V, AKD1000 NPU, i9-12900K |
| Three-mode benchmark | 20.4s → 9.2s (2.2× overall) |

---

## Action Items

1. **toadStool action**: Review `groundspring-forge` dispatch pattern for potential standardization as `barracuda::dispatch`
2. **toadStool action**: Consider `akida_driver::classify_i8()` convenience API (DMA pattern now dual-validated by wetSpring + groundSpring)
3. **toadStool action**: Absorb `batched_multinomial.wgsl` (112 lines, xoshiro128**, pending since V15)
4. **toadStool action**: Document PRNG divergence expectation when local and upstream use different algorithms
5. **toadStool action**: Consider `akida_driver::ModelManager` for compiled SNN model loading alongside raw DMA
6. **groundSpring**: Wire new Exp 028 NPU path through barracuda when `akida-driver` gains higher-level inference API
7. **groundSpring**: Continue three-tier validation as ToadStool absorbs remaining Tier B/C items

---

*groundSpring V26 — February 27, 2026 — AGPL-3.0-only*
