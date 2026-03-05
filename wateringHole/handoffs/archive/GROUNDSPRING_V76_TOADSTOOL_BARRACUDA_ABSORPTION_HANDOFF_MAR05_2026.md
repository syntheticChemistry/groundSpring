# SPDX-License-Identifier: AGPL-3.0-only

# groundSpring V76 → ToadStool/barraCuda: Absorption + Evolution Intelligence

**Date:** 2026-03-05
**From:** groundSpring V76 (33 experiments, 376/376 checks, 81 delegations, deep debt zero)
**To:** barraCuda team (math primitives), toadStool team (hardware dispatch)
**License:** AGPL-3.0-only

---

## Executive Summary

groundSpring is fully synchronized with barraCuda v0.3.1 and toadStool S93.
All 81 delegations verified. Deep debt zero achieved: zero TODOs, zero
`unwrap()` in production, zero bare tolerance literals, zero files >1000
lines. This handoff captures everything learned from V72–V76 that benefits
the toadstool/barraCuda evolution, including absorption targets, precision
insights, and architectural patterns worth adopting.

---

## 1. Absorption Targets for barraCuda

### 1.1 Ready for Absorption Now

| Item | Source | Benefit to Ecosystem |
|------|--------|---------------------|
| **13-tier `tol::` module** | `groundspring::tol` (V73) | Named tolerance constants with scientific justification. wetSpring adopted 164 tiers. All springs should share the pattern. |
| **`eps::` production guards** | `groundspring::eps` (V73) | `SAFE_DIV` (1e-10), `SSA_FLOOR` (1e-15), `UNDERFLOW` (1e-300). Prevent NaN in GPU kernels. |
| **`NucleusHarness` pattern** | `groundspring_forge::nucleus` (V76) | Reusable pass/fail harness with `finish() -> bool` for NUCLEUS validation binaries. Canonical pattern for all springs. |
| **Anderson Lyapunov shaders** | `metalForge/shaders/` | 2 production WGSL shaders (`anderson_lyapunov.wgsl`, `anderson_lyapunov_f32.wgsl`). Candidates for `barracuda::ops::anderson_lyapunov_{f64,f32}`. |

### 1.2 Evolution Requests (P1)

| Request | Rationale | groundSpring Use Case |
|---------|-----------|----------------------|
| **FFT (real, complex)** | Many spectral methods need FFT; absent from barraCuda | Spectral analysis, autocorrelation, potential Exp 021 optimization |
| **Tridiag eigenvector solver** | Sturm bisection + inverse iteration absorbed V68 but not public in barraCuda v0.3.1 | `tridiag_eigh` validation against Jacobi; promotes transport from Tier B to A |
| **Structured benchmark JSON** | Provenance fields (`_source`, `baseline_commit`, `baseline_date`) | Cross-primal benchmark comparison, automated regression tracking |

### 1.3 Evolution Requests (P2)

| Request | Rationale | groundSpring Use Case |
|---------|-----------|----------------------|
| **PRNG alignment** (xorshift64 → xoshiro128**) | Bitwise GPU/CPU parity for stochastic experiments | Requires full baseline regeneration of 28 benchmark JSONs |
| **Parallel 3D grid dispatch** | `(lat, lon, depth)` workgroup dispatch | Seismic inversion grid search (Exp 005) |
| **`chi_squared_cdf` / `chi_squared_quantile`** | p-value computation from chi² statistic | Freeze-out goodness-of-fit (Exp 020) |

---

## 2. Precision Insights from 33 Experiments

### 2.1 Tolerance Tier Discovery

Through 33 experiments across 10 domains, groundSpring discovered that
numerical tolerances cluster into 13 natural tiers:

| Tier | Value | What Introduces This Error |
|------|-------|---------------------------|
| `DETERMINISM` | 1e-15 | IEEE 754 rounding only (same path, same seed) |
| `STRICT` | 1e-14 | Compensated arithmetic |
| `EXACT` | 1e-12 | Summation-only f64 paths |
| `ANALYTICAL` | 1e-10 | One transcendental (sqrt, ln) |
| `INTEGRATION` | 1e-8 | ODE RK4 O(dt⁴) accumulation |
| `CDF_APPROX` | 1e-6 | CDF/erf approximation |
| `ROUNDTRIP` | 1e-5 | CDF↔PPF round-trip |
| `RECONSTRUCTION` | 1e-4 | Spectral Tikhonov roundtrip RMSE |
| `LITERATURE` | 0.001 | Published 3-4 significant figures |
| `DECOMPOSITION` | 0.005 | Pythagorean RMSE² = MBE² + σ² |
| `STOCHASTIC` | 0.01 | O(1/√N) sampling noise |
| `NORM_2PCT` | 0.02 | ~2% normalization tolerance |
| `EQUILIBRIUM` | 0.1 | ODE steady-state / measurement precision |

**Key insight for barraCuda**: GPU introduces ~1 extra ULP per
transcendental vs CPU. If CPU achieves `tol::EXACT`, GPU typically achieves
`tol::ANALYTICAL` — one tier looser. Batch dispatch does **not** degrade
precision for independent operations. Reduce operations lose ~1 tier due to
non-deterministic summation order.

### 2.2 Division-by-Zero is the #1 GPU NaN Source

Every GPU kernel that divides should guard with `eps::SAFE_DIV` or
equivalent. groundSpring found 3 production sites: Wright-Fisher mean
fitness, Gillespie SSA rate, Anderson condition number.

### 2.3 f32 Accumulation is Not Acceptable

Exp 025 proves f32 accumulation in Green-Kubo transport introduces ~28%
systematic bias vs f64. This is direction-dependent drift, not random noise.
All scientific compute must use f64 or DF64.

### 2.4 NVK/NAK f64 is Unreliable

V37 discovered NAK and NVVM advertise `SHADER_F64` but produce incorrect
results on Titan V (GV100/NVK) and RTX 4070 (AD104/proprietary). The
probe-based DF64 fallback in `fp64_strategy_probed()` is essential.

---

## 3. Delegation Patterns Worth Adopting

### 3.1 Graceful Fallback

```rust
#[cfg(feature = "barracuda")]
{
    if let Ok(result) = barracuda::stats::rmse(a, b) {
        result
    } else {
        local_rmse(a, b)
    }
}
#[cfg(not(feature = "barracuda"))]
{
    local_rmse(a, b)
}
```

CPU path **always compiles**. barraCuda errors (GPU OOM, shader failure)
fall back silently. 81 delegations prove this scales.

### 3.2 Feature Gating

```toml
barracuda = ["dep:barracuda"]
barracuda-gpu = ["barracuda", "barracuda/gpu"]
```

Two tiers: CPU-only and GPU. GPU implies CPU. Zero overhead for CPU
delegation.

### 3.3 Domain-Split Validation Binaries

V76's GPU tier split demonstrates that validation binaries should be
organized by scientific domain, not by chronology or feature. Each module
owns a domain (stats, spectral, bio) with a single `validate_all` entry
point. The `main.rs` orchestrates and manages the harness.

---

## 4. Cross-Spring Shader Provenance

groundSpring's 81 delegations trace to 5 origin springs:

| Origin | Count | Key Contribution |
|--------|-------|-----------------|
| **hotSpring** | 15 GPU | DF64, Lanczos eigensolver, Anderson spectral, ESN |
| **wetSpring** | 12 CPU | Shannon/Simpson/Bray-Curtis, Gillespie SSA, ODE bio |
| **airSpring** | 8 CPU + 5 GPU | FAO-56 ET₀, regression fits, L-BFGS, seasonal pipeline |
| **neuralSpring** | 3 GPU | Wright-Fisher GPU, pow_f64, batch_ipr |
| **ToadStool core** | 14 CPU + 11 GPU | stats::*, linalg::*, reduce ops |

### Cross-Pollination Map

```
hotSpring nuclear (Anderson 4D)  ──→  groundSpring tissue immunology (Paper 12)
airSpring soil calibration (L-BFGS) ──→  groundSpring QCD freeze-out (Paper 07)
wetSpring metagenomics (diversity) ──→  groundSpring sequencing noise (Exp 004)
neuralSpring ML (ESN reservoir)  ──→  groundSpring regime classifier (Exp 028)
groundSpring RAWR bootstrap      ──→  wetSpring rarefaction confidence intervals
groundSpring Anderson sweep      ──→  feeds ESN training data across springs
groundSpring 13-tier tol::       ──→  adopted by all springs as named tolerance pattern
```

---

## 5. What groundSpring Does NOT Need

| Item | Reason |
|------|--------|
| **Chao1 estimator** | Local uses classic Chao 1984; barraCuda uses bias-corrected Chao & Chiu 2016. Delegation would break Python baseline provenance. |
| **PRNG core** | Local `xorshift64` matches Python baselines. GPU PRNG uses `xoshiro128**` independently. |
| **akida-driver** | NPU hardware stays in toadStool. groundSpring's `npu` module calls toadStool's driver. |

---

## 6. Architecture (V76)

```
groundSpring V76
├── 33 modules, 790 tests, 97.25% coverage
├── 81 delegations (47 CPU + 34 GPU)
│   ├── barracuda = { path = "barraCuda/crates/barracuda" }
│   └── akida-driver = { path = "phase1/toadstool/.../akida-driver" }
├── 28 validation binaries (all PASS × 2 modes)
├── 23 cross-spring benchmark checks (all PASS)
├── 375 Python tests (baseline integrity)
├── 13-tier tolerance architecture (tol::, eps::)
└── Deep debt zero (0 TODO, 0 unwrap, 0 unsafe, 0 mocks)

Dependencies:
  barraCuda v0.3.1 (math) — 47 CPU + 34 GPU delegations
  akida-driver (NPU)      — ToadStool hardware, not math
  wgpu 22 (GPU device)    — behind barracuda-gpu feature
```

---

## 7. Recommendations for barraCuda/toadStool Evolution

1. **Adopt `tol::` pattern** — Named tolerance constants prevent tolerance
   inflation. The 13-tier taxonomy works across all scientific domains.

2. **Adopt `eps::` guards** — GPU kernels face division-by-zero at scale.
   Named epsilon constants (`SAFE_DIV`, `SSA_FLOOR`, `UNDERFLOW`) are
   self-documenting and auditable.

3. **Use `f64::midpoint`** — Overflow-safe midpoint in rank statistics,
   reduce ops, or any code computing average of two values.

4. **Capability-based discovery** — Socket discovery should scan for
   capabilities, not hardcode primal names. groundSpring's `biomeos_socket_dir`
   and `discover_primal_sockets` patterns are proven at scale.

5. **Domain-split binaries** — Large validation binaries should split by
   scientific domain, not arbitrary line count. Each domain module owns
   its validation logic.

6. **`BTreeMap` for deterministic iteration** — Any map whose iteration
   order affects output must use `BTreeMap`, not `HashMap`.

7. **Silent defaults are bugs** — `unwrap_or(0.0)` on scientific data is
   data corruption. Use `let Some(...) else { continue }` or `expect()`.

---

*groundSpring V76 → barraCuda/toadStool. 81 delegations, 13 tolerance
tiers, 33 experiments, 10 domains, deep debt zero. The uncertainty budget
for the entire ecoPrimals ecosystem.*
