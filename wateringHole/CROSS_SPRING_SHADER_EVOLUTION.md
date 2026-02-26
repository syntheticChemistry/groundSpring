# groundSpring — Cross-Spring Shader & Primitive Evolution

> How the ecoPrimals Springs collectively evolved BarraCUDA into the library
> groundSpring depends on for statistical validation.

**Last Updated**: February 26, 2026 (complete rewiring — 24 delegations, S58–S65 evolution)

---

## Overview

groundSpring delegates **24 functions** to barracuda. Those barracuda functions
were not built in isolation — they were refined and battle-tested through
absorption from **five Springs**, each bringing domain-specific requirements
that hardened the shared library.

```
hotSpring (nuclear physics)     → f64 precision, spectral theory, DF64, Sturm eigensolve
wetSpring (metagenomics)        → bio-stats, Shannon entropy, log_f64 fix, ODE systems
neuralSpring (ML/agents)        → spectral diagnostics, dispatch, xoshiro PRNG
airSpring (agriculture)         → error metrics (RMSE, MBE, R², IoA, hit rate)
groundSpring (noise validation) → error handling patterns, validation harness
                                  ↓
                          BarraCUDA S65 + DF64
                    2,490+ tests, 694 WGSL shaders
```

---

## hotSpring → Precision Foundation

hotSpring's nuclear physics work (lattice QCD, nuclear structure) established
the f64 precision infrastructure that ALL statistical operations depend on.

| Contribution | Session | groundSpring Benefit |
|-------------|---------|---------------------|
| `df64_core.wgsl` | S58 | Future GPU bootstrap precision |
| `Fp64Strategy` + `split_workgroups` | S58 | Correct f64 GPU dispatch strategy |
| `spectral/anderson.rs` | S26 | **Direct delegation**: `lyapunov_exponent`, `lyapunov_averaged` |
| `spectral/tridiag.rs` (Sturm bisection) | S26 | **Direct delegation**: `find_all_eigenvalues` → **50× Exp 009 speedup** |
| `spectral/stats.rs` | S26 | **Direct delegation**: `level_spacing_ratio` |
| `sum_reduce_f64.wgsl` | S46 | Foundation for RMSE/MBE GPU ops |
| `special/anderson_transport.rs` | S52 | **Direct delegation**: `localization_length` |
| CG solver shaders (6 kernels) | S46-48 | Pattern: iterative GPU solver with convergence |
| DF64 FMA + transcendentals | S60 | Consumer GPU precision for all Springs |
| 8 lattice WGSL (SU(3), PRNG, DF64) | S64 | Nuclear physics shaders in barracuda core |
| 2,490+ barracuda tests | — | Validates the precision path we depend on |

**Why it matters**: hotSpring discovered that FP64 operations on consumer GPUs
(RTX 4070) need careful workgroup sizing to avoid precision loss. This
discovery propagated to all barracuda f64 ops, including the `stats::*`
functions groundSpring delegates to. The Sturm tridiag eigenvalue solver
from S26 spectral work enables the 50× speedup for Exp 009's Almost-Mathieu
level spacing analysis — a direct cross-spring win.

---

## wetSpring → Bio-Statistical Primitives

wetSpring's metagenomics work (16S, metabolomics, diversity) contributed the
statistical and biological primitives groundSpring uses.

| Contribution | Session | groundSpring Benefit |
|-------------|---------|---------------------|
| `FusedMapReduceF64` (Shannon/Simpson) | S15 | GPU target for Shannon diversity |
| `log_f64()` coefficient fix (~1e-3 → 1e-15) | S15 | Accuracy of Shannon entropy calculations |
| `GillespieGpu` | S27 | Future GPU for `birth_death_ssa` |
| `ridge_regression` | S15/S59 | Available for regularized fitting |
| 5 ODE biosystems | S58 | **Delegations #13-14**: BistableOde, MultiSignalOde |
| `stats::diversity` (Shannon, Simpson, Chao1, etc.) | S64 | **Delegation #20**: `shannon_diversity` |
| `anderson_3d_correlated`, `find_w_c` | S59 | Future Anderson extensions |
| `bray_curtis_f64` | S15 | Diversity metric for rarefaction context |
| 918 Rust tests + 95 experiments | — | Validates the statistical and bio paths |

**Why it matters**: wetSpring's bio-ODE systems were absorbed into barracuda
(S58), enabling groundSpring to delegate Exp 010/011 derivatives. The diversity
module (S64) brought Shannon entropy, completing the chain from wetSpring's
metagenomics to groundSpring's sequencing noise analysis.

---

## airSpring → Error Metrics

airSpring's agricultural validation work (ET₀, soil moisture) contributed the
error metrics that groundSpring shares.

| Contribution | Session | groundSpring Benefit |
|-------------|---------|---------------------|
| `stats::metrics` (RMSE, MBE, NSE, R², IoA, hit_rate, mean, percentile) | S64 | **Delegations #15-19, #21-22** |
| FAO-56 ET₀ validation | S49 | Independent validation of shared metrics |
| 468 Rust tests | — | Validates the same metric functions we use |

**Why it matters**: airSpring and groundSpring independently needed RMSE, MBE,
R², and IoA for different domains (agriculture vs noise). ToadStool absorbed
both into `barracuda::stats::metrics` (S64), creating a single validated
implementation that both Springs delegate to — a textbook cross-spring win.

---

## neuralSpring → ML/Spectral Infrastructure

neuralSpring's neural network and agent work contributed the dispatch
infrastructure and spectral diagnostics.

| Contribution | Session | groundSpring Benefit |
|-------------|---------|---------------------|
| `empirical_spectral_density` | S54 | Future Anderson spectral diagnostics |
| `marchenko_pastur_bounds` | S54 | Random matrix theory bounds |
| `dispatch/domain_ops.rs` (device: Option) | S52 | Blueprint for GPU dispatch |
| `boltzmann_sampling` (Metropolis MCMC) | S56 | Future MC uncertainty propagation |
| `prng_xoshiro` GPU PRNG | S43 | PRNG alignment target for Phase 2b |
| `TensorSession` (matmul, relu, softmax) | S20 | ML pipeline infrastructure |
| 1,560+ validation checks | — | Validates dispatch and spectral infra |

**Why it matters**: neuralSpring's `domain_ops.rs` dispatch pattern
(`device: Option<&Arc<WgpuDevice>>`) is the blueprint for how groundSpring's
6 pending GPU metrics should be wired — `None` for CPU, `Some(device)` for GPU.

---

## groundSpring → Validation Patterns

groundSpring contributes back to the ecosystem primarily through **patterns
and learnings** rather than GPU shaders:

| Contribution | Benefit to Ecosystem |
|-------------|---------------------|
| `if let Ok` + always-compiled CPU fallback | Adopted as wateringHole standard for barracuda delegation |
| `ValidationHarness` pattern | ToadStool absorbed as `barracuda::validation::ValidationHarness` |
| Capability-based primal discovery | wateringHole standard: scan for capability, not primal name |
| Three-mode validation (local / barracuda / barracuda-gpu) | Proves correctness across feature configurations |
| Zero-overhead benchmark methodology | Proves barracuda delegation is free for compute-heavy code |
| Tolerance documentation standard | Every tolerance justified with mathematical basis |
| 2 production WGSL shaders | `batched_multinomial.wgsl`, `mc_et0_propagate.wgsl` (pending absorption) |

---

## Multi-Spring Convergence

Several barracuda modules benefited from **multiple Springs discovering the
same need independently**:

| Module | Springs | Evolution |
|--------|---------|-----------|
| **f64 precision** | hotSpring + wetSpring + neuralSpring | Three Springs found precision issues; all fixes merged |
| **error metrics** | airSpring + groundSpring | Both needed RMSE/MBE/R² independently; unified in S64 |
| **bio ops** | wetSpring + neuralSpring | Complementary biological simulation primitives |
| **spectral analysis** | hotSpring + neuralSpring + groundSpring | Physics + ML + localization perspectives |
| **PRNG** | neuralSpring + wetSpring + groundSpring | GPU xoshiro128** shared across stochastic workloads |
| **validation patterns** | All five Springs | `ValidationHarness`, tolerance docs, struct extraction |

---

## groundSpring Delegation Lineage

Each of groundSpring's 24 delegations has a traceable cross-spring history:

| # | groundSpring fn | barracuda fn | Primary Origin | Validated By |
|---|----------------|--------------|---------------|-------------|
| 1 | `pearson_r` | `stats::pearson_correlation` | ToadStool core | wetSpring + neuralSpring |
| 2 | `spearman_r` | `stats::correlation::spearman_correlation` | S54 (neuralSpring baseCamp) | neuralSpring spectral diagnostics |
| 3 | `sample_std_dev` | `stats::correlation::std_dev` | ToadStool core | wetSpring diversity metrics |
| 4 | `covariance` | `stats::correlation::covariance` | ToadStool core | neuralSpring correlation matrices |
| 5 | `norm_cdf` | `stats::norm_cdf` | ToadStool core | All Springs (significance testing) |
| 6 | `norm_ppf` | `stats::norm_ppf` | ToadStool core | groundSpring bootstrap CI |
| 7 | `chi2_statistic` | `stats::chi2_decomposed` | ToadStool core | wetSpring goodness-of-fit |
| 8 | `bootstrap_mean` | `stats::bootstrap_mean` | ToadStool core | groundSpring RAWR validation |
| 9 | `lyapunov_exponent` | `spectral::lyapunov_exponent` | hotSpring S26 (Kachkovskiy) | hotSpring spectral checks |
| 10 | `lyapunov_averaged` | `spectral::lyapunov_averaged` | hotSpring S26 (Kachkovskiy) | hotSpring + groundSpring |
| 11 | `analytical_localization_length` | `special::localization_length` | wetSpring S52 (transport) | groundSpring Anderson checks |
| 12 | `almost_mathieu_hamiltonian` | `spectral::almost_mathieu_hamiltonian` | hotSpring S26 (Kachkovskiy) | groundSpring Exp 009 |
| 13 | `bistable_derivative` | `BistableOde::cpu_derivative` | S58 (bio ODE) | groundSpring Exp 010 |
| 14 | `multisignal_derivative` | `MultiSignalOde::cpu_derivative` | S58 (bio ODE) | groundSpring Exp 011 |
| 15 | `rmse` | `stats::rmse` | S64 (airSpring/groundSpring absorption) | groundSpring Exp 002 |
| 16 | `mbe` | `stats::mbe` | S64 (airSpring/groundSpring absorption) | groundSpring Exp 002 |
| 17 | `r_squared` | `stats::r_squared` | S64 (airSpring/groundSpring absorption) | groundSpring Exp 002 |
| 18 | `index_of_agreement` | `stats::index_of_agreement` | S64 (airSpring/groundSpring absorption) | groundSpring Exp 002 |
| 19 | `hit_rate` | `stats::hit_rate` | S64 (airSpring/groundSpring absorption) | groundSpring Exp 002 |
| 20 | `shannon_diversity` | `stats::shannon` | S64 (wetSpring absorption) | groundSpring Exp 004 |
| 21 | `mean` | `stats::mean` | S64 (airSpring/groundSpring) | All experiments |
| 22 | `percentile` | `stats::percentile` | S64 (airSpring/groundSpring) | groundSpring bootstrap CI |
| 23 | `level_spacing_ratio` | `spectral::level_spacing_ratio` | hotSpring S26 (spectral stats) | groundSpring Exp 009 |
| 24 | `almost_mathieu_eigenvalues` | `spectral::find_all_eigenvalues` | hotSpring S26 (Sturm tridiag) | groundSpring Exp 009 (**50× speedup**) |

---

## ToadStool Session Evolution (S58–S65)

The complete cross-spring evolution that led to groundSpring's 24 delegations:

### S58 — Cross-Spring Absorption Wave

| Source | Absorbed | groundSpring Impact |
|--------|----------|-------------------|
| hotSpring | `df64_core.wgsl`, `Fp64Strategy`, `split_workgroups` | Future GPU bootstrap precision on consumer GPUs |
| neuralSpring | `pow_f64` polyfill fix (NAK/Ada Lovelace) | All f64 transcendentals work on NVVM |
| wetSpring | 5 ODE systems (`BistableOde`, `MultiSignalOde`, etc.), NMF | **Delegations #13-14**: ODE derivatives |

### S59 — Anderson + Validation Harness

| Source | Absorbed | groundSpring Impact |
|--------|----------|-------------------|
| wetSpring | `anderson_3d_correlated`, `anderson_sweep_averaged`, `find_w_c` | Future Anderson extension |
| neuralSpring | `ValidationHarness`, `require!` macro | Validation infrastructure |
| hotSpring | `bench_fp64_ratio` binary | FP64 benchmarking methodology |

### S60 — DF64 FMA + Transcendentals (hotSpring precision → all Springs)

| Evolution | Detail |
|-----------|--------|
| `df64_core.wgsl` FMA | Dekker `split()` (17 ops) → `fma(a,b,-p)` (2 ops) |
| `df64_transcendentals.wgsl` | sqrt, exp, log, sin, cos, pow, tanh at FP32-core speed |
| 4 force shaders | Born-Mayer, Morse, Yukawa, Lennard-Jones → all-DF64 |

### S61 — Sovereign Compiler

FMA fusion: `Mul(a,b) + c` → `fma(a,b,c)` benefits all 694 WGSL shaders.

### S62 — Infrastructure

`BandwidthTier` (PCIe/NvLink aware), `PeakDetectF64`, pool padding.

### S64 — Statistics Absorption (the big one for groundSpring)

| Source | Absorbed | groundSpring Delegations |
|--------|----------|------------------------|
| airSpring + groundSpring | `stats::metrics`: rmse, mbe, r², IoA, hit_rate, mean, percentile | **#15-19, #21-22** |
| wetSpring | `stats::diversity`: shannon, simpson, chao1, bray_curtis | **#20** (shannon) |
| hotSpring | 8 lattice WGSL: su3_math, prng_pcg, gauge_force, kinetic_energy | Nuclear physics shaders |
| groundSpring | `batched_multinomial` (GPU + CPU) | Future rewiring (signature adapter needed) |

### S65 — Smart Refactoring

compute_graph, esn_v2, tensor, gamma, rk45 all slimmed. Quality refinement.

---

## Benchmark: Cross-Spring Evolution Impact

The cross-spring evolution (S50–S65) eliminated overhead and added the
Sturm tridiag solver that transforms Exp 009:

| Period | Total Runtime | Quasiperiodic | Overhead vs Local |
|--------|-------------|---------------|-------------------|
| V7 (pre-S50) | 2,721ms | (not benchmarked) | **+6%** |
| V9 (post-S62) | 2,076ms | (not benchmarked) | **~0%** |
| V12 (S64) | 14,434ms | 11,355ms (dense QR) | **~0%** |
| V13 (S64+Sturm) | 3,274ms (barracuda-gpu) | **234ms** (Sturm tridiag) | **−77%** (faster!) |

The Sturm bisection eigenvalue solver (from hotSpring's S26 spectral module,
absorbed into `barracuda::spectral::tridiag`) exploits the tridiagonal structure
of the Almost-Mathieu Hamiltonian. Combined with `find_all_eigenvalues`, this
replaces the O(n³) dense Givens QR with an O(n²) tridiag solver — closing
the LAPACK gap that was Exp 009's only performance outlier.
