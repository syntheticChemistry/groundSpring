# groundSpring V119 → barraCuda / toadStool Absorption Handoff

**Date:** March 22, 2026
**From:** groundSpring V119
**To:** barraCuda team, toadStool team
**License:** AGPL-3.0-or-later

---

## Executive Summary

groundSpring V119 consumes 110 barraCuda delegations (67 CPU + 43 GPU) across 40 modules
covering statistics, spectral theory, inverse problems, biological dynamics, and hydrology.
This handoff documents: (1) current consumption patterns, (2) upstream compile issues found,
(3) patterns worth absorbing, and (4) evolution priorities for next cycle.

---

## 1. Current Consumption (barraCuda v0.3.5)

### CPU Delegated (67 operations)

| Domain | Operations | Key Primitives |
|--------|-----------|----------------|
| Statistics | 20+ | `barracuda::stats::{mean, std_dev, variance, percentile, sample_std_dev}` |
| Regression | 8 | `barracuda::stats::regression::{linear, quadratic, exponential, logarithmic}` |
| Correlation | 6 | `barracuda::stats::{pearson_r, spearman_r, covariance, correlation_full}` |
| Distributions | 3 | `barracuda::stats::{norm_cdf, norm_ppf, chi_squared}` |
| Special | 4+ | `barracuda::special::{erf, erfc, log_gamma, incomplete_gamma}` |
| Linear algebra | 6 | `barracuda::linalg::{gemm_f64, cholesky, tridiag_eigh, qr_decomp}` |
| Optimization | 5 | `barracuda::optimize::{brent_root, nelder_mead_batch, lbfgs_numerical}` |
| Biology | 8 | `barracuda::stats::{hill, monod, chao1, shannon, simpson, bray_curtis}` |
| Hydrology | 7 | `barracuda::stats::{fao56_et0, hargreaves_et0, thornthwaite_*}` |

### GPU Dispatched (43 operations)

| Domain | Operations | Key Primitives |
|--------|-----------|----------------|
| Reduce ops | 8 | `SumReduceF64`, `FusedMapReduceF64`, `VarianceReduceF64`, `CorrelationF64` |
| Matrix ops | 3 | `GemmF64`, `BatchedEighGpu`, `CholGpu` |
| Bio simulation | 4 | `GillespieGpu`, `WrightFisherGpu`, `BatchedMultinomialGpu`, `BootstrapMeanGpu` |
| ODE solvers | 2 | `BatchedOdeRK4F64` (bistable, multisignal) |
| FFT | 1 | `Fft1DF64` (spectral reconstruction) |
| Spectral | 3 | `BatchedEighGpu` (Anderson), `SturmTridiag`, `SparseEigensolverGpu` (Lanczos) |
| Hydrology | 2 | `BatchedElementwiseF64` (ET₀), `McEt0PropagateGpu` |
| Infrastructure | 4 | `PrecisionRoutingAdvice`, `GpuDriverProfile`, f64 smoke test, `OnceLock` probe |

---

## 2. Upstream Issues Found

### 2a. Compile failure without `gpu` feature

When compiling `barracuda` crate **without** the `gpu` feature (which is the common case
for CPU-only consumers like `cargo test -p groundspring`), two errors occur:

```
error[E0433]: crate::ops::lattice::cpu_complex::Complex64
  --> barracuda/src/special/plasma_dispersion.rs:23
  note: `pub mod ops` is gated behind `gpu` feature

error[E0433]: crate::linalg::eigh::eigh_f64
  --> barracuda/src/spectral/stats.rs:142
  note: `pub mod eigh` is gated behind `gpu` feature
```

**Fix needed**: Either gate these imports behind `#[cfg(feature = "gpu")]` or extract
the CPU-accessible parts of `ops` and `linalg::eigh` from behind the `gpu` gate.

### 2b. `cast` module parity

groundSpring expanded its internal cast module to match airSpring's barracuda `cast.rs`
(7 new helpers: `i32_f64`, `u32_f64`, `f64_u32`, `u32_usize`, `u64_usize`, `f64_i32`,
`usize_i32`). These are duplicated across springs. Consider promoting the full set to
`barracuda::cast` so springs can `use barracuda::cast::*` instead of maintaining local copies.

---

## 3. Patterns Worth Absorbing

### 3a. Provenance Registry Completeness Test

groundSpring V119 added a compile-time test that `include_str!`s all 29 benchmark JSONs
and asserts the expected count. This catches silent benchmark drift when files are
added/removed. Pattern applicable to any crate with `include_str!`-ed data.

```rust
const EXPECTED_BENCHMARKS: usize = 29;
let benchmarks: &[(&str, &str)] = &[
    ("sensor_noise", include_str!("../../../control/.../benchmark.json")),
    // ... all 29
];
assert_eq!(benchmarks.len(), EXPECTED_BENCHMARKS);
for (name, json_str) in benchmarks {
    assert!(serde_json::from_str::<Value>(json_str).is_ok());
}
```

### 3b. IPC Test Isolation via Atomic Counter

All biomeOS tests now use unique socket paths via an atomic counter, preventing
parallel test collisions in CI. Pattern: `static TEST_SOCKET_ID: AtomicU64`.

### 3c. `publish = false` Hygiene

All workspace crates set `publish = false` to prevent accidental crates.io publishing.
Should be ecosystem standard for all springs/primals.

### 3d. `cfg_attr(not(test), expect(dead_code))` for Absorbed API Surface

New cast helpers are tested but not yet called from production code. Using
`#[cfg_attr(not(test), expect(dead_code, reason = "..."))]` documents intent while
keeping tests green. Preferable to blanket `#[expect(dead_code)]`.

---

## 4. Evolution Priorities

### For barraCuda

| Priority | Item | Effort |
|----------|------|--------|
| **P0** | Fix non-`gpu` compile (plasma_dispersion + spectral/stats imports) | Small |
| **P1** | Promote full cast module to `barracuda::cast` (deduplicate across springs) | Small |
| **P1** | RAWR GPU kernel (Bayesian weighted resampling — no barraCuda equivalent) | Medium |
| **P2** | Sparse matrix-vector product (CSR SpMV) for Lanczos inner loop at scale | Medium |
| **P2** | Matrix exponentiation (general case, not just Cayley/SU(3)) | Medium |

### For toadStool

| Priority | Item | Effort |
|----------|------|--------|
| **P1** | Absorb groundSpring's 2 remaining `anderson_lyapunov*.wgsl` reference shaders | Small |
| **P2** | Fused spectral pipeline (Anderson Lyapunov + level statistics in one dispatch) | Medium |

### Cross-Spring GPU Sharing Opportunities

| Kernel | groundSpring Use | Also Benefits |
|--------|-----------------|---------------|
| Jackknife GPU | Leave-one-out resampling (Bazavov precision) | neuralSpring (model uncertainty) |
| Anderson 4D | 4D localization for lattice QCD proxy | hotSpring (Wegner flow) |
| Batched Nelder-Mead | Multi-start optimization | airSpring (isotherm fitting) |
| Rarefaction GPU | Multinomial sampling + diversity in one dispatch | wetSpring (16S pipelines) |

---

## 5. Quality Metrics

| Metric | Value |
|--------|-------|
| barraCuda delegations | 110 (67 CPU + 43 GPU) |
| Validation checks | 395/395 PASS |
| Library coverage | ≥92% |
| Workspace tests | 990+ |
| Clippy (pedantic+nursery) | 0 warnings |
| Unsafe code | 0 (`#![forbid(unsafe_code)]` on all 3 crate roots) |
| `.unwrap()` in library | 0 |
| `publish = false` | 3/3 crates |
| MSRV | 1.85 (Rust 2024) |

---

Part of [ecoPrimals](https://github.com/syntheticChemistry) · AGPL-3.0-or-later
