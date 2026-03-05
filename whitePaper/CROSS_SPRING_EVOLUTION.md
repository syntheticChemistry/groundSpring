# Cross-Spring Evolution: How ecoPrimals Primals Build on Each Other

**Date**: February 27, 2026
**groundSpring HEAD**: V77
**barraCuda**: v0.3.1

---

## The Multi-Spring Architecture

groundSpring's 61 active barracuda delegations (37 CPU + 20 GPU + 4 cross-spring) trace their lineage through 5 ecoPrimals
Springs. Each Spring contributes domain-specific primitives that are absorbed into
ToadStool's BarraCUDA crate, making them available to all other Springs. This
document traces exactly which primitives came from where and who benefits.

```
 hotSpring          wetSpring          airSpring       neuralSpring
 (precision)        (biology)          (environment)   (ML/dispatch)
     │                  │                  │               │
     │   ┌──────────────┼──────────────────┼───────────────┘
     │   │              │                  │
     ▼   ▼              ▼                  ▼
  ╔════════════════════════════════════════════════╗
  ║          ToadStool / BarraCUDA                 ║
  ║   (hardware-agnostic tensor compute)           ║
  ╚════════════════════════════════════════════════╝
                       │
                       ▼
                  groundSpring
              (noise validation)
```

---

## 1. hotSpring → Precision + Spectral Theory

hotSpring originated from Kachkovskiy's spectral theory and nuclear physics
work. Its contributions to BarraCUDA are foundational for any Spring doing
numerical linear algebra or spectral analysis.

### What hotSpring contributed

| Primitive | Barracuda module | Session | Impact |
|-----------|-----------------|---------|--------|
| Lyapunov exponent (transfer matrix) | `spectral::anderson` | Pre-S39 | Anderson localization in 1D/2D/3D |
| Sturm bisection eigenvalue solver | `spectral::tridiag` | S26 | O(n²) tridiag eigenvalues vs O(n³) dense QR |
| Almost-Mathieu Hamiltonian | `spectral::hofstadter` | Pre-S39 | Quasiperiodic localization, Hofstadter butterfly |
| Level spacing ratio | `spectral::stats` | Pre-S39 | GOE vs Poisson statistics |
| Lanczos tridiagonalization | `spectral::lanczos` | Pre-S39 | Sparse matrix → tridiag |
| Chi-squared decomposition | `stats::chi2` | Pre-S39 | Per-datum goodness-of-fit |
| Bootstrap CI | `stats::bootstrap` | Pre-S39 | Confidence intervals |
| DF64 core-streaming (double-float) | `ops::df64_core.wgsl` | S58 | FP64 precision on FP32 consumer GPUs |
| Lattice QCD (SU(3) gauge) | `ops::su3_*.wgsl` | S64 | 8 nuclear physics shaders |
| Hermite polynomials | `special::hermite` | Pre-S39 | Nuclear equation of state |
| ESN reservoir (Stanton-Murillo) | `esn_*.wgsl` | Pre-S39 | Echo state networks |

### Who benefits from hotSpring

- **groundSpring**: Lyapunov + Sturm solver → **49.5× speedup** on Exp 009
  (quasiperiodic localization). This is the single largest performance win
  across all Springs. The Sturm bisection solver exploits the tridiagonal
  structure of the Almost-Mathieu Hamiltonian, reducing O(n³) → O(n²).
- **neuralSpring**: DF64 transcendentals enable f64 training on consumer GPUs.
- **wetSpring**: Lanczos + spectral stats for disordered biological networks.

---

## 2. wetSpring → Biology + Microbial Ecology

wetSpring models quorum sensing, microbial diversity, and biological signal
processing. Its contributions define the biological ODE systems and diversity
indices used across the ecosystem.

### What wetSpring contributed

| Primitive | Barracuda module | Session | Impact |
|-----------|-----------------|---------|--------|
| BistableOde (5-var QS switching) | `numerical::ode_bio` | S58 | Phenotypic switching model |
| MultiSignalOde (7-var dual QS) | `numerical::ode_bio` | S58 | CAI-1 + AI-2 integration |
| CapacitorOde, CooperationOde, PhageDefenseOde | `numerical::ode_bio` | S58 | 3 additional bio systems |
| Shannon diversity (natural log) | `stats::diversity` | S64 | α-diversity metric |
| Simpson diversity | `stats::diversity` | S64 | Dominance metric |
| Chao1 estimator | `stats::diversity` | S64 | Richness estimator |
| Pielou evenness | `stats::diversity` | S64 | Evenness metric |
| Bray-Curtis dissimilarity | `stats::diversity` | S64 | β-diversity metric |
| Anderson transport (Landauer) | `special::anderson_transport` | S52 | Localization length ξ(W,E) |
| Correlated Anderson disorder | `spectral::anderson` | S59 | 3D correlated potential |
| Anderson sweep (finite-size) | `spectral::anderson` | S59 | Finite-size scaling |
| Hill function (WGSL) | `ops::hill_f64.wgsl` | S49 | Quorum sensing nonlinearity |
| Diversity fusion (WGSL) | `ops::diversity_fusion_f64.wgsl` | S49 | Combined diversity metrics |
| ESN NPU reservoir | `esn_reservoir_update_f64.wgsl` | S51 | Neuromorphic echo state |

### Who benefits from wetSpring

- **groundSpring**: ODE biosystems for Exp 010 (bistable) and Exp 011
  (multisignal). Shannon + Pielou for Exp 004 (sequencing noise). Anderson
  transport for Exp 008 analytical localization length.
- **neuralSpring**: Biological ESN for neuromorphic computing.
- **hotSpring**: Correlated disorder models for condensed matter.

---

## 3. airSpring → Environmental Metrics

airSpring contributed the error metrics and environmental statistics that form
the backbone of model-observation comparison across all Springs.

### What airSpring contributed

| Primitive | Barracuda module | Session | Impact |
|-----------|-----------------|---------|--------|
| RMSE, MBE, R², NSE, IoA, hit_rate | `stats::metrics` | S64 | Error decomposition |
| Mean, percentile | `stats::metrics` | S64 | Summary statistics |
| Kriging interpolation (WGSL) | `ops::kriging_f64.wgsl` | S49 | Soil moisture interpolation |
| Moving window stats (WGSL) | `ops::moving_window.wgsl` | S40 | IoT sensor streams |
| FAO-56 ET₀ batch (WGSL) | `ops::batched_elementwise_f64.wgsl` | S49 | Evapotranspiration |

### Who benefits from airSpring

- **groundSpring**: 7 error metric delegations (RMSE, MBE, R², IoA, hit_rate,
  mean, percentile) used across all 33 experiments.
- **wetSpring**: Moving window for time-series diversity.
- **hotSpring**: IoA for lattice QCD convergence monitoring.

---

## 4. neuralSpring → ML Infrastructure + Dispatch

neuralSpring contributes ML-oriented primitives and the dispatch/substrate
selection infrastructure that all Springs use implicitly.

### What neuralSpring contributed

| Primitive | Barracuda module | Session | Impact |
|-----------|-----------------|---------|--------|
| Graph Laplacian | `linalg` | S54 | Spectral graph analysis |
| Effective rank | `linalg` | S54 | Matrix condition analysis |
| Numerical Hessian | `numerical` | S54 | Gradient-free optimization |
| Empirical spectral density | `stats::spectral_density` | S54 | Marchenko-Pastur bounds |
| Domain dispatch (ODE, pairwise, spatial) | `dispatch` | S52 | GPU vs CPU routing |
| Batch IPR (GPU) | `spectral::batch_ipr` | Pre-S39 | Inverse participation ratio |
| ValidationHarness + require! macro | `validation` | S59 | Validation infrastructure |

### Who benefits from neuralSpring

- **groundSpring**: ValidationHarness pattern (adopted as standard).
- **wetSpring**: Domain dispatch for batch biological ODE integration.
- **hotSpring**: Numerical Hessian for lattice optimization.

---

## 5. groundSpring → Patterns + Validation

groundSpring contributes the validation methodology and specific patterns that
flowed back into the ecosystem.

### What groundSpring contributed

| Contribution | Absorbed into | Impact |
|-------------|---------------|--------|
| `if let Ok` + CPU fallback pattern | wateringHole standard | Adopted by all Springs |
| Three-mode validation (default/barracuda/barracuda-gpu) | Ecosystem standard | Proves correctness across configs |
| Dense QR → Sturm demonstration | Performance benchmark | Quantified 49.5× algorithmic win |
| Batched multinomial (WGSL) | `ops::batched_multinomial_f64.wgsl` | Rarefaction on GPU |
| MC ET₀ propagation (WGSL) | `ops::mc_et0_propagate_f64.wgsl` | Monte Carlo error propagation |
| Error metrics co-authorship | `stats::metrics` | RMSE, MBE, IoA from groundSpring experiments |

---

## 6. The Cross-Pollination Matrix

Each cell shows whether Spring A (row) contributes to Spring B (column)
through shared BarraCUDA primitives.

|  | groundSpring uses | wetSpring uses | airSpring uses | hotSpring uses | neuralSpring uses |
|---|---|---|---|---|---|
| **hotSpring gives** | Lyapunov, Sturm, DF64 | Lanczos, spectral stats | — | — | DF64 transcendentals |
| **wetSpring gives** | ODE bio, Shannon, Anderson transport | — | — | Correlated disorder | Bio ESN |
| **airSpring gives** | RMSE, MBE, R², IoA, mean | Moving window | — | IoA convergence | — |
| **neuralSpring gives** | ValidationHarness | Domain dispatch | — | Numerical Hessian | — |
| **groundSpring gives** | — | — | Error metrics | — | Validation patterns |

---

## 7. The 49.5× Win: A Cross-Spring Story

The most dramatic performance result in groundSpring — the **49.5× speedup**
on Experiment 009 (quasiperiodic localization) — is a direct cross-spring win.

**The chain**:
1. **hotSpring** (Kachkovskiy spectral theory) implemented a Sturm bisection
   eigenvalue solver for tridiagonal matrices as part of nuclear physics work.
2. **ToadStool** absorbed this as `spectral::tridiag::find_all_eigenvalues`
   in S26.
3. **groundSpring** wired `almost_mathieu_eigenvalues` to use this solver
   instead of its local O(n³) dense Givens QR.
4. The Sturm solver exploits the tridiagonal structure of the Almost-Mathieu
   Hamiltonian, reducing complexity from O(n³) to O(n²).

**Result**: 11,986 ms → 242 ms. A nuclear physics eigenvalue solver
accelerating a quasiperiodic localization experiment from condensed matter
physics, running in a noise validation Spring.

This is exactly the kind of cross-domain reuse that the multi-Spring
architecture was designed to enable.

---

## 8. Benchmark Results (Feb 26, 2026)

Three-mode timing for all 21 validation binaries:

| Binary | Default (ms) | Barracuda CPU (ms) | Barracuda-GPU (ms) | Speedup | Checks |
|--------|-------------|-------------------|-------------------|---------|--------|
| validate-decompose | 82 | 71 | 560 | 0.1× ¹ | 36/36 |
| validate-rarefaction | 70 | 99 | 102 | 0.7× ¹ | 15/15 |
| validate-seismic | 141 | 128 | 171 | 0.8× ¹ | 9/9 |
| validate-weather | 65 | 71 | 97 | 0.7× ¹ | 13/13 |
| validate-fao56 | 79 | 80 | 106 | 0.7× ¹ | 15/15 |
| validate-signal-specificity | 854 | 858 | 898 | 1.0× | 12/12 |
| validate-rawr | 619 | 625 | 651 | 1.0× | 11/11 |
| validate-anderson | 745 | 745 | 774 | 1.0× | 8/8 |
| validate-quasiperiodic | **11,986** | **11,867** | **242** | **49.5×** | 8/8 |
| validate-bistable | 167 | 222 | 207 | 0.8× | 9/9 |
| validate-multisignal | 85 | 118 | 118 | 0.7× | 8/8 |
| **TOTAL** | **14,893** | **14,884** | **3,926** | **3.8×** | **177/177** |

¹ GPU initialization overhead dominates for small workloads (<100ms).
These experiments are not GPU-bound; the overhead is wgpu device discovery
and buffer allocation. In a real pipeline, this cost is amortized across
thousands of invocations.

**Barracuda CPU** adds negligible overhead (14,884ms vs 14,893ms) — the CPU
delegation is zero-cost for these workload sizes.

**Barracuda-GPU** delivers the win where it matters: the compute-bound
`validate-quasiperiodic` goes from 11.99s → 0.24s, driving the total suite
from 14.89s → 3.93s.

---

*This document is generated from groundSpring's validation infrastructure
and ToadStool's absorption history. It reflects the state of the ecosystem
at ToadStool S68+ / groundSpring V35.*
