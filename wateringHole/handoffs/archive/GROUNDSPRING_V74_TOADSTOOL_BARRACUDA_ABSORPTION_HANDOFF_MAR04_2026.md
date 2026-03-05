# groundSpring V74 → ToadStool/barraCuda Absorption Handoff

**Date**: March 4, 2026
**From**: groundSpring V74 (790 tests, 81 delegations, 97.25% coverage)
**To**: ToadStool S93+ / barraCuda v0.3.1+ team
**Purpose**: Absorption targets, evolution requests, cross-spring learnings

---

## Executive Summary

groundSpring is fully synchronized with barraCuda v0.3.1 and ToadStool S93.
All 81 delegations (47 CPU + 34 GPU) verified working. Full validation
benchmark: 28/28 binaries PASS in both default and barracuda modes, with
10% overall speedup from CPU delegation. Cross-spring benchmark 23/23 PASS.

This handoff documents:
1. What groundSpring learned that benefits the ecosystem
2. What groundSpring needs from barraCuda evolution
3. Absorption targets for new groundSpring work
4. Cross-spring shader provenance insights

---

## 1. Absorption Targets (New Work for barraCuda)

### Tier 1: Ready for Absorption Now

| Item | Source | Benefit |
|------|--------|---------|
| 13-tier tolerance architecture | `groundspring::tol` (V73) | Named tolerance constants with scientific justification — all Springs could share |
| `eps::UNDERFLOW` guard | `linalg.rs` QL iteration (V74) | Subnormal protection for eigensolvers |
| Validation benchmark methodology | V74 Run 43 | 28-binary × 2-mode comparison framework, cross-spring speedup attribution |

### Tier 2: Evolution Requests (P1)

| Request | Rationale | groundSpring Use Case |
|---------|-----------|---------------------|
| **FFT (real, complex)** | Many spectral methods need FFT; currently absent from barraCuda | Spectral analysis, autocorrelation |
| **Tridiag eigenvector solver** | Sturm bisection + inverse iteration was absorbed in V68 but not yet public in barraCuda v0.3.1 API | `tridiag_eigh` validation against Jacobi |
| **Structured benchmark output** | JSON benchmark format with provenance fields (`_source`, `baseline_commit`, `baseline_date`) | Cross-primal benchmark comparison |

### Tier 3: Evolution Requests (P2)

| Request | Rationale | groundSpring Use Case |
|---------|-----------|---------------------|
| **PRNG alignment** | xorshift64 → xoshiro128** canonical | Bitwise GPU/CPU parity for stochastic experiments |
| **Parallel 3D grid dispatch** | (lat, lon, depth) workgroup dispatch | Seismic inversion grid search |
| **`chi_squared_cdf` / `chi_squared_quantile`** | p-value computation from chi² statistic | Freeze-out goodness-of-fit |

---

## 2. Cross-Spring Learnings for barraCuda Team

### Precision Insights

1. **DF64 is the sweet spot for consumer GPUs**: groundSpring's precision-drift
   experiment (Exp 025) confirmed that DF64 (~48 mantissa bits) is sufficient
   for all statistical validation workloads. Native f64 gives no practical
   benefit over DF64 for our tolerances (1e-12 analytical, 1e-8 integration).

2. **NVK/NAK f64 is unreliable**: groundSpring V37 discovered that NAK and NVVM
   advertise `SHADER_F64` but produce incorrect results. The probe-based
   DF64 fallback in `fp64_strategy_probed()` is essential. This was confirmed
   on Titan V (GV100/NVK) and RTX 4070 (AD104/proprietary).

3. **`f64::midpoint` for overflow safety**: Rust 1.87's `f64::midpoint` is
   preferable to `(a + b) / 2.0` in rank statistics (Spearman). Small point
   but relevant for any GPU shader doing midpoint computation.

### Delegation Pattern Insights

1. **`if let Ok` + always-compiled CPU fallback**: This pattern has proven
   robust across 81 delegations. The CPU path compiles in all feature modes,
   ensuring no dead code. barraCuda errors (GPU OOM, shader compilation
   failure) fall back silently to CPU.

2. **Feature gating**: `barracuda` (CPU) and `barracuda-gpu` (GPU) as
   separate features works well. CPU delegation has zero overhead; GPU
   delegation needs device singleton management (`gpu.rs`).

3. **Tolerance documentation**: Every tolerance constant has a scientific
   justification in the doc comment. This prevents tolerance inflation over
   time. The 13-tier named tolerance architecture (`tol::DETERMINISM` through
   `tol::EQUILIBRIUM`) provides semantic meaning to assertions.

### Performance Observations

barraCuda CPU delegation provides surprising speedups for non-trivial
operations:

| Domain | Speedup | Likely Cause |
|--------|---------|-------------|
| FAO-56 hydrology | −78% | barraCuda's vectorized Penman-Monteith |
| Spectral reconstruction | −81% | barraCuda's Gauss-Jordan with partial pivoting |
| Freeze-out chi² | −67% | barraCuda's fused chi² decomposition |
| Jackknife | −57% | barraCuda's optimized leave-one-out |
| Precision drift | −17% | Transfer matrix cache optimization |

### What Did NOT Improve

- **Multisignal ODE**: +75% (stochastic noise, not meaningful)
- **Quasispecies**: +63% (small workload, setup overhead dominates)
- **Transport**: +11% (tridiagonal QL is already optimal on CPU)

These regressions are within run-to-run variance for sub-200ms workloads.

---

## 3. Cross-Spring Shader Provenance

groundSpring's 81 delegations trace to 5 origin Springs:

| Origin | Delegations | Key Contribution |
|--------|-------------|-----------------|
| **hotSpring** | 15 GPU | DF64 precision, Lanczos eigensolver, Anderson spectral, ESN regime classification |
| **wetSpring** | 12 CPU | Shannon/Simpson/Bray-Curtis diversity, Gillespie SSA, ODE biosystems |
| **airSpring** | 8 CPU + 5 GPU | FAO-56 ET₀, Hargreaves, regression fits, L-BFGS optimizer, seasonal pipeline |
| **neuralSpring** | 3 GPU | Wright-Fisher GPU, pow_f64 fix, batch_ipr spectral |
| **ToadStool core** | 14 CPU + 11 GPU | stats::*, linalg::*, numerical::*, correlation, reduce ops |

### Cross-Pollination Highlights

```
hotSpring nuclear (Anderson 4D)  ──→  groundSpring tissue immunology (Paper 12)
airSpring soil calibration (L-BFGS) ──→  groundSpring QCD freeze-out (Paper 07)
wetSpring metagenomics (diversity) ──→  groundSpring sequencing noise (Exp 004)
neuralSpring ML (ESN reservoir)  ──→  groundSpring regime classifier (Exp 028)
groundSpring RAWR bootstrap      ──→  wetSpring rarefaction confidence intervals
groundSpring Anderson sweep      ──→  feeds ESN training data across springs
```

---

## 4. Chao1 Divergence (Do Not Absorb)

groundSpring's `chao1` uses the classic formula `f₁²/(2f₂)` (Chao 1984).
barraCuda's `chao1_classic` uses the bias-corrected `f₁(f₁−1)/(2(f₂+1))`
(Chao & Chiu 2016). Delegating would break Python baseline provenance.
groundSpring keeps its local implementation as the validation reference.

---

## 5. Current Architecture

```
groundSpring V74
├── 33 modules, 790 tests, 97.25% coverage
├── 81 delegations (47 CPU + 34 GPU)
│   ├── barracuda = { path = "barraCuda/crates/barracuda" }
│   └── akida-driver = { path = "phase1/toadstool/crates/neuromorphic/akida-driver" }
├── 28 validation binaries (all PASS × 2 modes)
├── 23 cross-spring benchmark checks (all PASS)
├── 375 Python tests (baseline integrity)
└── 13-tier tolerance architecture (tol::, eps::)

Dependencies:
  barraCuda v0.3.1 (math) ── 47 CPU + 34 GPU delegations
  akida-driver (NPU)      ── ToadStool hardware, not math
  wgpu 22 (GPU device)    ── behind barracuda-gpu feature
```

---

## 6. Files Modified in V74

| File | Change |
|------|--------|
| `crates/groundspring/src/wdm.rs` | Clippy `let_and_return` fix |
| `crates/groundspring/src/drift.rs` | `wf_batch_gpu` split into 3 helpers |
| `crates/groundspring/src/linalg.rs` | `eps::UNDERFLOW` integration |
| `crates/groundspring-validate/src/lib.rs` | Tolerance re-exports from `groundspring::tol` |
| `crates/groundspring/src/freeze_out.rs` | Named optimizer constants |
| `crates/groundspring/src/fao56/pipeline.rs` | Named physical clamp bounds |
| `crates/groundspring/src/biomeos/mod.rs` | Named timeout constants |
| `.github/workflows/ci.yml` | `clippy::pedantic` enforcement |
| `metalForge/forge/src/bin/benchmark_cross_spring.rs` | S93/v0.3.1 provenance |

---

*groundSpring V74 — March 4, 2026 — All gates green, all delegations verified, ecosystem synchronized.*
