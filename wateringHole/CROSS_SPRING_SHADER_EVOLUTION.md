# groundSpring — Cross-Spring Shader & Primitive Evolution

> How the ecoPrimals Springs collectively evolved BarraCUDA into the library
> groundSpring depends on for statistical validation.

**Last Updated**: March 7, 2026 (V95: 102 active delegations (61 CPU + 41 GPU), barraCuda `0bd401f`, toadStool S129, coralReef Phase 11. V95: coralReef push buffer breakthrough — sovereign GPU dispatch on Titan V, mthd_incr field swap fixed. V87: Tier B resolution — `multinomial_sample` CPU-delegated (wetSpring S15 → barraCuda S93), `anderson_potential` CPU-delegated (hotSpring S26 → barraCuda spectral); 5 stale Tier B entries resolved (already wired); `quasispecies_simulation` + `band_structure` coarse scan documented as CPU-by-design. V86: Fp64Strategy DF64 reduce wiring. V85: coralReef sovereign compilation. V84: dual-GPU probe. V82: BootstrapMeanGpu. V73: 13-tier tolerance architecture)

---

## Overview

groundSpring has **102 active delegations** (61 CPU + 41 GPU) with **0 evolution candidates** — Tier B fully resolved (V87).
Those barracuda functions were not built in isolation — they were refined and
battle-tested through absorption from **five Springs**, each bringing domain-specific
requirements that hardened the shared library.

```
hotSpring (nuclear physics)     → f64 precision, DF64 core-streaming, spectral (Lanczos,
                                  Anderson, Hofstadter), Sturm eigensolve, lattice QCD
                                  (SU(3), CG, Wilson, HMC), nuclear HFB, MD forces,
                                  Hermite/Laguerre, ESN multi-head transport
wetSpring (metagenomics)        → bio-stats (Shannon, Simpson, Bray-Curtis, DADA2, HMM,
                                  ANI, dN/dS, Smith-Waterman, Felsenstein, Gillespie),
                                  diversity fusion, ODE generic solver, NMF, kriging,
                                  RTX 4070 f64 pow/exp/log precision discovery
neuralSpring (ML/agents)        → AlphaFold2 Evoformer, HMM forward/backward/Viterbi,
                                  evolutionary (batch fitness, swarm NN, stencil cooperation),
                                  SimpleMLP, spectral density, matmul GPU, RK45 adaptive
airSpring (agriculture)         → FAO-56 ET₀ (8 methods), Hargreaves, Van Genuchten, dual Kc,
                                  seasonal pipeline, Brent root-finding, anderson coupling,
                                  Richards PDE (Crank-Nicolson + cyclic reduction), kriging
groundSpring (noise validation) → jackknife, evolution (Kimura fixation, quasispecies),
                                  diversity (Chao1, detection power), hydrology (fao56_et0),
                                  grid search/fit ops, batched multinomial, MC ET₀ propagation,
                                  L-BFGS refinement, 4D Anderson tissue + Wegner RG
                                  ↓
                          barraCuda v0.3.3 (standalone primal)
                    14,200+ tests, 844 WGSL shaders (f64-canonical, DF64 universal precision, 15 transcendentals)
```

> **barraCuda budding (V70)**: barraCuda has budded from phase1/toadstool into a
> standalone primal at `ecoPrimals/barraCuda/`. groundSpring depends on
> `barraCuda/crates/barracuda` as a sibling primal. ToadStool S-xx session
> references remain as historical session identifiers.

---

## hotSpring → Precision Foundation

hotSpring's nuclear physics work (lattice QCD, nuclear structure) established
the f64 precision infrastructure that ALL statistical operations depend on.

| Contribution | Session | groundSpring Benefit |
|-------------|---------|---------------------|
| `df64_core.wgsl` | S58 | Future GPU bootstrap precision |
| `Fp64Strategy` + `split_workgroups` | S58 | Correct f64 GPU dispatch strategy |
| `spectral/anderson.rs` | S26 | **Direct delegation**: `lyapunov_exponent`, `lyapunov_averaged` |
| `spectral/tridiag.rs` (Sturm bisection) | S26 | **Direct delegation**: `find_all_eigenvalues` → **49.5× Exp 009 speedup** |
| `spectral/stats.rs` | S26 | **Direct delegation**: `level_spacing_ratio` |
| `sum_reduce_f64.wgsl` | S46 | Foundation for RMSE/MBE GPU ops |
| `special/anderson_transport.rs` | S52 | **Direct delegation**: `localization_length` |
| CG solver shaders (6 kernels) | S46-48 | Pattern: iterative GPU solver with convergence |
| DF64 FMA + transcendentals | S60 | Consumer GPU precision for all Springs |
| 8 lattice WGSL (SU(3), PRNG, DF64) | S64 | Nuclear physics shaders in barracuda core |
| 4-layer brain architecture | S79 | NPU cerebellum + dual GPU + CPU cortex pattern |
| 15-head multi-observable ESN | S79 | Multi-head uncertainty for transport + phase detection |
| 14,200+ barracuda tests | — | Validates the precision path we depend on |

**Why it matters**: hotSpring discovered that FP64 operations on consumer GPUs
(RTX 4070) need careful workgroup sizing to avoid precision loss. This
discovery propagated to all barracuda f64 ops, including the `stats::*`
functions groundSpring delegates to. The Sturm tridiag eigenvalue solver
from S26 spectral work enables the 49.5× speedup for Exp 009's Almost-Mathieu
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
| 783 Rust tests + 33 experiments | — | Validates the statistical and bio paths |

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
GPU metrics are wired — `None` for CPU, `Some(device)` for GPU. 3 grid ops remain as evolution candidates (interface mismatch).

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
| 2 WGSL shaders (V62) | `anderson_lyapunov.wgsl` (f64), `anderson_lyapunov_f32.wgsl` — 2 redundant shaders removed V62 (absorbed into ToadStool: `batched_multinomial_f64.wgsl`, `mc_et0_propagate_f64.wgsl`) |
| Architecture-aware GPU dispatch | `GpuArch` detection, `NativeF64` capability, f64→Titan V routing |
| NAK f64 gap discovery | `SHADER_F64` unreliable on both NAK and NVVM — DF64 required everywhere |
| `AdaptiveBatch` memory management | Software-side VRAM batch sizing with architecture defaults |

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
| **optimization** | airSpring + groundSpring (V68) | Brent root-finding + L-BFGS refined from hydrology to physics |
| **4D spectral theory** | hotSpring + groundSpring (V68) | Nuclear Anderson → tissue immunology (dimension promotion) |

---

## V68 Cross-Spring Evolution Highlights

V68 demonstrates the deepest cross-spring convergence yet — shaders and
algorithms that evolved in one domain directly accelerating a different one.

### hotSpring Precision Shaders → groundSpring Tissue 4D

hotSpring's condensed matter work (Anderson localization, Wegner RG) evolved
the spectral module through increasingly sophisticated lattice constructions:
`anderson_1d` (S26) → `anderson_2d` / `anderson_3d` (S26) →
`anderson_3d_correlated` (S59) → **`anderson_4d` + `wegner_block_4d` (S84)**.

groundSpring absorbs the 4D variants for tissue immunology (Paper 12): the
fourth dimension represents an immune response gradient (cytokine concentration
over time). The Wegner block RG coarsening reveals how disorder flows under
cell-cluster coarse-graining — whether cytokine signaling localizes or
propagates depends on the effective dimensionality at the cluster scale.

**The same physics**: hotSpring asks "does an electron propagate through a
disordered 4D lattice?" and groundSpring asks "does a cytokine signal
propagate through a 4D tissue structure?" — identical mathematics, different
domains, shared barracuda implementation.

### airSpring Optimizer → groundSpring Freeze-Out Refinement

airSpring's parameter fitting for FAO-56 ET₀ models evolved L-BFGS into
barracuda (S84). groundSpring absorbs `lbfgs_numerical` to refine the coarse
grid search in QCD freeze-out curve fitting (Bazavov et al.): the grid search
finds the basin, L-BFGS converges to sub-grid precision.

**Cross-domain transfer**: agricultural sensor calibration → nuclear physics
parameter estimation. The optimizer doesn't care about the domain; it only sees
an objective function and its numerical gradient.

### wetSpring Bio Shaders ↔ neuralSpring

wetSpring's biodiversity primitives (Shannon, Simpson, Bray-Curtis) and
stochastic shaders (`BatchedMultinomialGpu`, `GillespieGpu`) were hardened by
neuralSpring's metalForge evolutionary computation. The hardened versions serve
groundSpring's rare biosphere and rarefaction experiments, and also flow back
to neuralSpring for fitness landscape analysis.

neuralSpring's ML shaders (matmul, softmax, ESN) flow into wetSpring for
functional annotation and metabolomics classification, and into groundSpring
via the `nautilus` feature gate for concept edge detection.

**Bidirectional flow**: unlike the one-way hotSpring → all pattern, bio and ML
shaders evolve in a cycle where wetSpring (domain biology) and neuralSpring
(domain ML) each improve the other's foundations.

---

## V87 Bidirectional Cross-Spring Provenance

V87 resolves all Tier B evolution candidates, completing groundSpring's
delegation story. Each resolution traces through cross-spring provenance:

### hotSpring Precision Foundation → ALL Springs

`df64_core.wgsl` (S58) is the single most impactful cross-spring contribution:
**every** Spring's f64 GPU operations work on consumer GPUs because hotSpring's
nuclear physics precision requirements forced DF64 double-float emulation.

Flow: hotSpring S58 `df64_core.wgsl` → toadStool absorption → barraCuda universal
precision tier → groundSpring `SumReduceF64`, `VarianceReduceF64` (V86 Hybrid
strategy) → wetSpring bio-stats diversity → neuralSpring spectral density → airSpring
error metrics.

Welford mean+variance (hotSpring S26) flows to wetSpring (diversity stats) and
groundSpring (bootstrap, sensor noise). The lattice QCD SU(3) shaders adopted by
neuralSpring for spectral density analysis — nuclear physics informing neural
network theory via shared matrix algebra.

### wetSpring Bio Primitives ↔ groundSpring Ecology

Shannon/Simpson/Bray-Curtis diversity (wetSpring S15 metagenomics) adopted by
groundSpring (rarefaction Exp 004, rare biosphere Exp 016) and neuralSpring
(fitness landscape analysis). Gillespie SSA (wetSpring S27) adopted by groundSpring
birth-death models. Wright-Fisher drift (wetSpring S66) adopted by groundSpring
quasispecies (CPU-by-design — single-locus mutation overhead exceeds GPU dispatch).

**Bidirectional**: `multinomial_sample` originated in groundSpring (V62), was absorbed
by toadStool, transferred to barraCuda S93, and now groundSpring delegates back to
`barracuda::ops::bio::multinomial_sample_cpu` — a complete round-trip.

wetSpring's `log_f64()` precision fix (~1e-3 → 1e-15 coefficient error) flowed back
to hotSpring, improving nuclear force calculations. Cross-domain precision hardening.

### neuralSpring ML Infrastructure → ALL Springs

`pow_f64` polyfill (neuralSpring S-17) unblocked Ada Lovelace (RTX 40xx) for ALL
Springs — a single bug fix enabling an entire GPU architecture family.

Pairwise distance ops (neuralSpring) adopted by wetSpring for bio pipelines.
`domain_ops` dispatch pattern (neuralSpring S52 `device: Option<&Arc<WgpuDevice>>`)
adopted by groundSpring for all GPU wiring — the blueprint for `if let Some(device)`.

HMM forward/backward (neuralSpring) adopted by wetSpring (sequence alignment) and
hotSpring (nuclear level transition models). ESN reservoir networks (wetSpring →
hotSpring → neuralSpring S59) power groundSpring's Anderson regime classifier.

### airSpring Hydrology → groundSpring Physics

RMSE/MBE/NSE/R² error metrics (airSpring S64) unified for ALL Springs — both
airSpring (agricultural) and groundSpring (noise validation) independently needed
the same error metrics for different domains.

Brent root-finding (airSpring V035 Richards PDE) adopted by groundSpring band edge
refinement — agricultural soil physics → condensed-matter band structure. L-BFGS
optimizer (airSpring) adopted by groundSpring freeze-out QCD curve fitting —
sensor calibration → nuclear parameter estimation.

### groundSpring Validation Patterns → ALL Springs

The 13-tier tolerance architecture (V73) adopted by wetSpring (expanded to 164
tiers for metagenomics). The `if let Ok` + CPU fallback delegation pattern adopted
as wateringHole standard. Three-mode validation (local / barracuda / barracuda-gpu)
proves correctness across feature configurations — adopted by all Springs.

---

## S87–S93 + barraCuda Budding Evolution (V70–V74)

The largest structural evolution in the ecosystem: barraCuda budded from
ToadStool into a standalone math primal, and groundSpring caught up to the
full S93 state.

### barraCuda Budding (S89, March 3, 2026)

ToadStool's embedded `crates/barracuda/` extracted to `ecoPrimals/barraCuda/`
as a standalone primal. groundSpring rewired with a zero-code-change path swap
(V70), confirmed by all 5 Springs:

```
groundSpring ──→ barraCuda v0.3.1   (WHAT to compute — math primitives)
toadStool    ──→ barraCuda v0.3.1   (WHERE/HOW — dispatch, scheduling)
akida-driver remains in ToadStool   (hardware, not math)
```

### Universal Precision Architecture

barraCuda v0.3.1 compiles every f64-canonical shader to 4 precision tiers:

| Tier | Hardware | Mantissa Bits | Source |
|------|----------|--------------|--------|
| F16 | Mobile/NPU | 10 | Downcast from f64 |
| F32 | Consumer default | 23 | Downcast from f64 |
| F64 | Compute GPUs (Titan V) | 52 | Native |
| DF64 | Consumer GPUs (RTX 4070) | ~48 | **hotSpring S58** double-float emulation |

`compile_shader_universal(source, precision)` auto-selects based on
`Fp64Strategy::probe()` runtime discovery. groundSpring benefits transparently —
delegated functions get optimal precision per hardware without code changes.

### Cross-Spring Evolution in S87–S93

| Session | Cross-Spring Impact | groundSpring Benefit |
|---------|--------------------|--------------------|
| S87 | FHE shader fix (`u64_mod_simple`) from internal audit | No direct impact |
| S88 | groundSpring V68 absorption: `anderson_4d`, `wegner_block_4d`, `LbfgsGpu`, `tridiag_eigenvectors` | Our tissue immunology shaders now upstream |
| S89 | barraCuda budding: 767 shaders, standalone primal | Zero-code-change path swap (V70) |
| S90 | REST→JSON-RPC: all Springs migrate to capability-based discovery | Already JSON-RPC (V30+) |
| S91-S92 | BearDog sovereignty neutralization, middleware removal | No impact |
| S93 | D-DF64 transfer to barraCuda team; 12 stale docs removed | DF64 precision now barraCuda-owned |

### Where Cross-Spring Evolution Helped Most

**hotSpring precision → all**: The DF64 core-streaming architecture (S58)
means groundSpring's statistical computations get f64-class precision
(~48 mantissa bits) on consumer RTX 4070 hardware — without any
groundSpring-specific effort. This is the single largest cross-spring win.

**wetSpring bio → groundSpring diversity**: Shannon, Simpson, Bray-Curtis,
Chao1, rarefaction_curve all came from wetSpring metagenomics (S64). These
power groundSpring's rare biosphere and sequencing noise experiments.

**airSpring hydrology → groundSpring physics**: FAO-56 ET₀ methods (S66) and
L-BFGS optimizer (S84) from agricultural sensor calibration now serve QCD
freeze-out parameter estimation — the optimizer sees an objective function,
not a domain.

**neuralSpring ML → groundSpring classification**: ESN reservoir networks
(wetSpring → hotSpring → S59) power groundSpring's Anderson regime classifier
(Exp 028). The `pow_f64` polyfill fix (neuralSpring S-17) unblocked Ada
Lovelace GPUs for all Springs.

**groundSpring patterns → all**: The `if let Ok` + CPU fallback delegation
pattern, `ValidationHarness`, tolerance documentation standard, and
capability-based discovery are now wateringHole standards adopted by all Springs.

---

## groundSpring Delegation Lineage

Each of groundSpring's 91 active delegations has a traceable cross-spring history:

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
| 24 | `almost_mathieu_eigenvalues` | `spectral::find_all_eigenvalues` | hotSpring S26 (Sturm tridiag) | groundSpring Exp 009 (**49.5× speedup**) |
| 25 | `evenness` | `stats::pielou_evenness` | S64 (wetSpring absorption) | groundSpring Exp 004 |
| 26 | `rawr_mean` | `stats::rawr_mean` | S66 (groundSpring V15 request) | groundSpring Exp 007, Exp 013 |
| 27 | `hill` | `stats::hill` | S68 (V20 catch-up) | groundSpring Exp 010, Exp 011 (bistable, multisignal) |
| 28 | `tikhonov_solve` | `linalg::solve_f64_cpu` | hotSpring linalg (Gauss–Jordan) | groundSpring Exp 021 (spectral recon, Bazavov 2025) |
| 29 | `finite_size_extrapolate` | `stats::regression::fit_linear` | S66 (regression absorption) | groundSpring Exp 026 (system-size convergence, WDM) |
| 30 | `mae` | `stats::mae` | S64 (airSpring/groundSpring absorption) | V33 — Mean Absolute Error, cross-validated with airSpring ET₀ metrics |
| 31 | `nash_sutcliffe` | `stats::nash_sutcliffe` | S64 (airSpring/groundSpring absorption) | V33 — Nash-Sutcliffe Efficiency, hydrology standard from airSpring |
| 32 | `detect_band_ranges` | `spectral::detect_bands` | hotSpring v0.6 (spectral theory) | V33 — GPU band detection from eigenvalue spectrum (barracuda-gpu tier) |
| 33 | `hargreaves_et0` | `stats::hydrology::hargreaves_et0` | airSpring V035 → `ToadStool` S70+ | V55 — Temperature-only ET₀ when radiation data unavailable |
| 34 | `hargreaves_et0_batch` (CPU) | `stats::hydrology::hargreaves_et0_batch` | airSpring V035 → `ToadStool` S70+ | V55 — Batch CPU delegation for multi-day Hargreaves |
| 35 | `crop_coefficient` | `stats::hydrology::crop_coefficient` | airSpring FAO-56 → `ToadStool` S70+ | V55 — Kc interpolation between growth stages |
| 36 | `soil_water_balance` | `stats::hydrology::soil_water_balance` | airSpring precision agriculture → `ToadStool` S70+ | V55 — Daily θ update with P+I−ET_c clamped to FC |
| 37 | `hargreaves_et0_batch` (GPU) | `BatchedElementwiseF64::execute(Op::HargreavesEt0)` | airSpring V035 → `ToadStool` S70+ | V55 — GPU batch Hargreaves via barracuda-gpu |
| 38 | `find_band_edges` (Brent refine) | `optimize::brent` | airSpring V035 (Richards PDE) → `ToadStool` S70+ | V55 — Brent root-finder refines coarse band edges to 1e-12 precision |
| 39 | `monte_carlo_et0` (GPU) | `McEt0PropagateGpu` | airSpring V010 (ET₀ uncertainty) → `ToadStool` S72 | V67 — MC ET₀ uncertainty propagation on GPU with CPU fallback |
| 40 | `seasonal_step` (GPU) | `SeasonalPipelineF64` | airSpring fused pipeline → `ToadStool` S80 | V67 — Fused ET₀ → Kc → water balance → stress on GPU |
| 41 | `grid_fit_2d` (L-BFGS refine) | `optimize::lbfgs_numerical` | airSpring V035 param fit → `ToadStool` S84 | V68 — Post-grid-search gradient refinement (sub-grid precision) |
| 42 | `tissue_4d_simulation` | `spectral::anderson::anderson_4d` | hotSpring S26 spectral → `ToadStool` S84 | V68 — 4D Anderson lattice for spatio-temporal tissue disorder |
| 43 | `tissue_4d_rg_coarsen` | `spectral::anderson::wegner_block_4d` | hotSpring condensed matter → `ToadStool` S84 | V68 — 4D Wegner RG coarsening reveals disorder flow at tissue cluster scale |
| 44 | `multinomial_sample` | `ops::bio::multinomial_sample_cpu` | wetSpring S15 → groundSpring V62 → barraCuda S93 | V87 — CPU delegation via cumulative prob adapter; batch path via `BatchedMultinomialGpu` |
| 45 | `anderson_potential` | `spectral::anderson_potential` | hotSpring S26 (Anderson localization) → barraCuda spectral | V87 — CPU delegation with documented PRNG divergence (Xorshift64 vs LcgRng) |

---

## ToadStool Session Evolution (S58–S70+)

The complete cross-spring evolution that led to groundSpring's 63 delegations:

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
| groundSpring | `batched_multinomial` (GPU + CPU) | **WIRED** (V87) — `multinomial_sample` CPU-delegated via cumulative prob adapter; batch path via `BatchedMultinomialGpu` |

### S65 — Smart Refactoring

compute_graph, esn_v2, tensor, gamma, rk45 all slimmed. Quality refinement.

---

## Benchmark: Cross-Spring Evolution Impact

### V33 Three-Mode Benchmark (Feb 27, 2026)

All 27 experiments timed in three feature modes, ToadStool S68+ (e96576ee):

| Experiment | Default | Barracuda CPU | Barracuda GPU | GPU Speedup | Cross-Spring Origin |
|-----------|---------|--------------|--------------|-------------|-------------------|
| Exp 001 decompose | 86ms | 447ms | 134ms | 0.6× | airSpring metrics |
| Exp 002 weather | 69ms | 142ms | 89ms | 0.8× | airSpring metrics |
| Exp 003 fao56 | 84ms | 201ms | 103ms | 0.8× | airSpring hydrology |
| Exp 004 rarefaction | 83ms | 95ms | 155ms | 0.5× | wetSpring diversity |
| Exp 005 seismic | 130ms | 328ms | 196ms | 0.7× | groundSpring grid search |
| Exp 006 signal | 863ms | 949ms | 1012ms | 0.9× | wetSpring bio-ODE |
| Exp 007 rawr | 634ms | 597ms | 624ms | 1.0× | groundSpring V15 |
| Exp 008 anderson | 805ms | 776ms | 741ms | 1.1× | hotSpring S26 spectral |
| **Exp 009 quasiperiodic** | **11376ms** | 12071ms | **240ms** | **47.4×** | **hotSpring Sturm tridiag** |
| Exp 010 bistable | 179ms | 289ms | 199ms | 0.9× | wetSpring bio-ODE S58 |
| Exp 011 multisignal | 146ms | 122ms | 113ms | 1.3× | wetSpring bio-ODE S58 |
| Exp 012 transport | 333ms | 332ms | 301ms | 1.1× | hotSpring spectral |
| Exp 013 resampling | 121ms | 227ms | 132ms | 0.9× | groundSpring bootstrap |
| Exp 014 drift | 1454ms | 1190ms | 1169ms | 1.2× | neuralSpring WF |
| Exp 015 uncertainty | 144ms | 131ms | 122ms | 1.2× | hotSpring + groundSpring |
| Exp 016 rare biosphere | 201ms | 245ms | 205ms | 1.0× | wetSpring diversity |
| Exp 017 quasispecies | 110ms | 117ms | 120ms | 0.9× | wetSpring Eigen model |
| Exp 018 band edge | 111ms | 189ms | 117ms | 0.9× | hotSpring spectral |
| **Exp 019 jackknife** | **410ms** | **96ms** | **100ms** | **4.1×** | groundSpring V15 |
| Exp 020 freeze-out | 219ms | 87ms | 127ms | 1.7× | groundSpring chi² |
| Exp 021 spectral recon | 96ms | 80ms | 116ms | 0.8× | hotSpring linalg |
| Exp 022 ET0 anderson | 105ms | 129ms | 125ms | 0.8× | airSpring + hotSpring |
| Exp 023 no-till sampling | 131ms | 393ms | 118ms | 1.1× | airSpring agro |
| Exp 024 aggregate stab | 95ms | 100ms | 88ms | 1.1× | airSpring agro |
| Exp 025 precision drift | 3726ms | 3174ms | 3096ms | 1.2× | neuralSpring spectral |
| Exp 026 size convergence | 176ms | 136ms | 111ms | 1.6× | hotSpring WDM |
| Exp 027 vendor parity | 143ms | 185ms | 145ms | 1.0× | all Springs |
| **TOTAL** | **22030ms** | **22828ms** | **9798ms** | **2.2×** | **cross-spring evolution** |

**279/279 checks pass in all three modes. 28/28 Python↔Rust parity proven.**

**Star performers** (GPU speedup from cross-spring absorption):
- **Exp 009 quasiperiodic: 47.4×** — hotSpring's Sturm tridiag eigenvalue solver (S26)
- **Exp 019 jackknife: 4.1×** — barracuda optimized jackknife (S64)
- **Exp 020 freeze-out: 1.7×** — barracuda chi² grid fit (S64)
- **Exp 026 size-convergence: 1.6×** — barracuda regression (S66)

### Historical Evolution

| Period | Total Runtime | Quasiperiodic | Overhead vs Local |
|--------|-------------|---------------|-------------------|
| V7 (pre-S50) | 2,721ms | (not benchmarked) | **+6%** |
| V9 (post-S62) | 2,076ms | (not benchmarked) | **~0%** |
| V12 (S64) | 14,434ms | 11,355ms (dense QR) | **~0%** |
| V13 (S64+Sturm) | 3,274ms (barracuda-gpu) | **234ms** (Sturm tridiag) | **−77%** (faster!) |
| **V33 (S68+)** | **9,798ms total (GPU)** | **240ms** | **−55% overall** |

The Sturm bisection eigenvalue solver (from hotSpring's S26 spectral module,
absorbed into `barracuda::spectral::tridiag`) exploits the tridiagonal structure
of the Almost-Mathieu Hamiltonian. Combined with `find_all_eigenvalues`, this
replaces the O(n³) dense Givens QR with an O(n²) tridiag solver — the single
largest cross-spring speedup in the ecoPrimals ecosystem.

---

## V18 Evolution: Flat Buffers + Kinetics Module

V18 completed the GPU-promotability groundwork:

| Change | Impact |
|--------|--------|
| `Vec<Vec<f64>>` → flat `Vec<f64>` (almost_mathieu, transport) | Data layout matches GPU dispatch (row-major, no indirection) |
| `kinetics::hill()` with barracuda delegation | Delegation #27 ready — `barracuda::stats::hill` exists in S66 |
| 13 determinism tests | Guards against silent PRNG/FP regressions across platforms |
| Full provenance (14 DOIs, 14 baseline_commits) | Machine-auditable chain: paper → Python → JSON → Rust → pass/fail |

### Learnings for ToadStool

1. **Flat buffers from day one**: Designing APIs with `&[f64]` + explicit `n` dimension
   avoids a refactor step when GPU dispatch arrives. The almost_mathieu and transport
   refactors were low-risk only because the public API was already slice-based.

2. **Determinism tests are cheap insurance**: 13 tests with `#[expect(clippy::float_cmp)]`
   for bitwise equality. Any PRNG stream change, reduction reorder, or platform FP
   difference will fail loudly. barracuda should adopt this pattern for all stateful
   computations.

3. **`if let Ok` is the right delegation pattern**: 26 delegations use it. The V17 bug
   fix (covariance/pearson/spearman silently returning 0.0) proved that the alternative
   (match + default) masks errors. Always fall through to CPU on delegation failure.

---

## V19 Evolution: Uncertainty Bridge (Exp 015)

V19 adds the first cross-domain experiment that chains validated modules:

| Change | Impact |
|--------|--------|
| Exp 015: Uncertainty Bridge | Sensor noise (Exp 001) → disorder mapping → Anderson Lyapunov (Exp 008) → localization length ξ |
| `validate-uncertainty-bridge` | 8/8 PASS; uses existing `anderson` module (no new barracuda delegation) |
| 21 experiments, 236/236 checks | Bridges Papers 22-24 (Sub-thesis 06: soil moisture → Anderson geometry → QS regime uncertainty) |
| Zero `#[allow]` | transport.rs fix removes last remaining allow |

**Key finding**: At typical soil moisture (θ≈0.30), Lyapunov exponent is in the saturated regime where bias correction has minimal effect on ξ uncertainty. CV(ξ) ranking preserved (EC5 > CS616). This validates the uncertainty propagation chain for the Anderson-QS bridge (Gen3 Sub-thesis 01+06).

---

## V20 Evolution: ToadStool S68 Catch-up + Hill Delegation

V20 pins ToadStool at `f0feb226` (S68 universal precision). Hill kinetics delegation #27 is now LIVE:

| Change | Impact |
|--------|--------|
| `kinetics::hill` | Stub → active delegation to `barracuda::stats::hill` (`#[cfg]`/`#[cfg(not)]` infallible pattern) |
| `kinetics::hill_repress` | Simplified to `1.0 - hill(x, k, n)` — gets barracuda delegation for free |
| ToadStool S68 | 700 shaders (zero f32-only), 2,546+ barracuda tests, 21,599 workspace tests (superseded by S79: 844 shaders, 14,200+ tests) |
| Delegation count | 27 active (22 CPU + 5 GPU), was 26 (21 + 5) |

**S68 universal precision**: ToadStool S68 completed the f32-only shader migration. All 700 shaders use f64 or df64. Feature gate bug resolved in latest ToadStool HEAD.

---

## V21 Evolution: Complete Barracuda Rewiring + Dual-Mode CI

V21 completes the barracuda integration by making `--features barracuda` compile cleanly (zero warnings) and validating all 280 tests in both CPU-only and barracuda-delegated modes.

### What Changed

| Change | Impact |
|--------|--------|
| Domain guard fix | `kinetics::hill` preserves biological convention (x ≤ 0 → 0) before delegating |
| 17 `_cpu` functions gated | `#[cfg(not(feature = "barracuda"))]` on all fallback functions — zero dead-code warnings |
| `needless_return` cleanup | `#[cfg]` blocks use expression position instead of `return` keyword |
| Import gating | `bistable`, `multisignal`, `metrics` imports gated per feature flag |
| Dual-mode CI | `cargo clippy --features barracuda` + `cargo test --features barracuda` added to CI |

### Cross-Spring Benchmark: CPU vs Barracuda CPU Delegation

All 21 validation binaries timed in `--release` mode, CPU-only vs barracuda-delegated (CPU math, no GPU):

| Experiment | CPU-only | Barracuda | Notes |
|-----------|----------|-----------|-------|
| Exp 001 decompose | 69ms | 84ms | Small data; call overhead dominates |
| Exp 002 weather | 67ms | 78ms | Small data |
| Exp 003 seismic | 119ms | 128ms | No barracuda delegation in hot path |
| Exp 004 rarefaction | 70ms | 93ms | Shannon/evenness via barracuda stats |
| Exp 005 fao56 | 82ms | 95ms | No barracuda delegation in hot path |
| Exp 006 signal | 855ms | 919ms | Bootstrap via barracuda |
| Exp 007 rawr | 640ms | 604ms | **Faster** — barracuda rawr_mean optimized |
| Exp 008 anderson | 831ms | 742ms | **Faster** — barracuda lyapunov hot path |
| Exp 009 quasiperiodic | 11,750ms | 11,836ms | Dominated by eigensolver (both use Sturm) |
| Exp 010 bistable | 173ms | 198ms | ODE derivative via barracuda |
| Exp 011 multisignal | 101ms | 128ms | ODE derivative via barracuda |
| Exp 012 transport | 313ms | 365ms | No barracuda eigenvector solver yet |
| Exp 013 resampling | 123ms | 166ms | Bootstrap CI via barracuda |
| Exp 014 drift | 1,146ms | 1,155ms | Wright-Fisher CPU-only (no delegation) |
| Exp 015 uncertainty | 108ms | 131ms | MC loop CPU-only, lyapunov via barracuda |
| **Total** | **16,447ms** | **16,722ms** | **+1.7%** — negligible CPU delegation overhead |

**Key insight**: CPU delegation adds ~1.7% total overhead from function indirection — functionally free. Heavy experiments (Anderson, RAWR) are actually slightly *faster* with barracuda's optimized implementations. The real speedup opportunity is GPU delegation for Exp 009 (eigensolver) and Exp 014 (Wright-Fisher batching).

### Cross-Spring Shader Categories (barraCuda v0.3.1)

The 844 barracuda WGSL shaders that groundSpring's delegations ultimately depend on:

| Category | Count | Primary Origin Springs | groundSpring Usage |
|----------|-------|----------------------|-------------------|
| math/ | 108 | All springs | Foundation for all numeric ops |
| reduce/ | 31 | All springs | Sum, mean, variance reductions |
| linalg/ | 32 | hotSpring, neuralSpring | CG solver, eigenvalues |
| special/ | 36 | hotSpring | Transcendentals, DF64 |
| loss/ | 34 | wetSpring, neuralSpring | Loss functions |
| activation/ | 37 | neuralSpring | Neural network activations |
| bio/ | 38 | wetSpring, neuralSpring | Genomics, ODE, Gillespie |
| lattice/ | 36 | hotSpring | QCD, SU(3) |
| tensor/ | 43 | neuralSpring | Shape operations |
| stats/ | ~10 | airSpring, groundSpring | Histogram, bootstrap |
| spectral/ | ~8 | hotSpring, neuralSpring | IPR, Lanczos |
| science/ | 13 | hotSpring | HFB nuclear structure |
| numerical/ | ~6 | neuralSpring, wetSpring | RK4, Hessian |
| ml/ | ~12 | hotSpring, wetSpring, neuralSpring | ESN, RF, HMM |
| interpolation/ | ~4 | airSpring, wetSpring | Kriging |
| Other | ~52 | Various | Norm, pooling, conv, RNN, optimizer |

### Cross-Pollination Timeline Highlights

| Date | Flow | What Evolved |
|------|------|-------------|
| Feb 12 | hotSpring → ToadStool | Complex64 WGSL, SU(3) matrix algebra |
| Feb 14 | hotSpring → ToadStool | Native f64 builtins, MD observables |
| Feb 16 | wetSpring → ToadStool | First 3 bio shaders (Bray-Curtis, diversity) |
| Feb 18 | hotSpring → ToadStool | DF64 core streaming (FP64 on FP32 cores) |
| Feb 19-20 | wetSpring → ToadStool | Gillespie SSA, Smith-Waterman, Felsenstein, tree inference |
| Feb 20 | neuralSpring → ToadStool | TensorSession ML ops, pairwise metrics |
| Feb 21 | hotSpring → ToadStool | Spectral module (Lanczos, Anderson, Sturm) |
| Feb 22 | airSpring → ToadStool | Richards PDE, kriging, moving window |
| Feb 24 | neuralSpring → ToadStool | Graph Laplacian, spectral density, Hessian |
| Feb 24 | wetSpring + hotSpring → ToadStool | NMF, 5 bio ODEs, Fp64Strategy, df64_core |
| Feb 25 | hotSpring → ToadStool | 8 lattice QCD shaders, DF64 GEMM |
| Feb 26 | airSpring + groundSpring → ToadStool | stats::regression, hydrology, rawr_mean |
| Feb 26 | **ToadStool S68** | **291 f32→f64 canonical, zero f32-only, universal precision** |
| Feb 26 | **groundSpring V21** | **Complete barracuda rewiring, dual-mode CI** |
| Feb 27 | **groundSpring V26** | **metalForge live hardware**: NPU DMA on AKD1000, Exp 028, groundspring-forge crate |
| Feb 27 | **groundSpring V27** | **Barracuda evolution review**: 29 delegations, paper controls audit, three-tier validation |
| Feb 27 | **groundSpring V28** | **Coverage evolution + PRNG readiness**: xoshiro128** at API parity, 368 tests + 196 Python integrity, CI baseline drift detection |
| Feb 27 | **groundSpring V29** | **Three-tier validation buildout**: 391 Rust + 322 Python = 713 total, 32 delegations (26 CPU + 6 GPU), 23 three-tier parity integration tests, 3 new CPU delegations (kimura_fixation_prob, jackknife_mean_variance, daily_et0), 8 GPU-annotated modules with barracuda documentation |
| Feb 27 | **groundSpring V30** | **biomeOS Neural API integration**: JSON-RPC 2.0 Unix socket client (`biomeos.rs`), `validate-anderson` routed through `capability.call`, pipeline graph, capability surface docs |
| Feb 27 | **groundSpring V31** | **GPU dispatch wiring + metalForge expansion**: 5 modules wired for `barracuda-gpu` (freeze_out, band_structure, seismic, quasispecies, rare_biosphere), 12 metalForge workloads, 37 dispatch targets, 442 Rust (biomeos) / 410 default + 320 Python = 762 total |
| Feb 27 | **groundSpring V32** | **ToadStool S68+ catch-up**: 9 forward declarations cleaned (3 CPU + 6 GPU, pending ToadStool absorption), 29 active delegations (23 CPU + 6 GPU), `--features barracuda` and `barracuda-gpu` compile clean, universal precision architecture (DF64, f32/f64/df64 per hardware) documented |
| Feb 27 | **groundSpring V33** | **Complete rewiring + three-mode benchmark**: 3 new delegations (#30 MAE from airSpring, #31 NSE from airSpring, #32 detect_bands from hotSpring), 32 active (25 CPU + 7 GPU), 279/279 checks ×3 modes, 28/28 parity proven, **47.4× GPU speedup** (Exp 009 quasiperiodic via hotSpring Sturm), **2.2× total GPU speedup** |
| Feb 28 | **groundSpring V55** | **Modern ToadStool S70+ rewiring**: 6 new delegations (#33-38), 57 active (38 CPU + 19 GPU), airSpring hydrology chain (Hargreaves ET₀ + crop Kc + soil water balance), Brent root-finder for band edge precision, **17.9× GPU speedup** on lib tests |
| Mar 1 | **groundSpring V56** | **NUCLEUS integration**: biomeOS Neural API live (Tower + Node + Squirrel validated), NestGate data pipelines (NCBI, NOAA, IRIS), 4 NUCLEUS experiments (Exp 029–032), 347/347 checks (292 core + 55 NUCLEUS), sovereign fallback on all paths |
| Mar 1 | **groundSpring V58** | **Cross-spring evolution + deep-debt completion**: 4 new cross-spring S59+ delegations (disorder_sweep, anderson_2d, anderson_3d, chi2_analysis), ESN regime classification module (wetSpring lineage), Lanczos sparse eigensolver module (hotSpring lineage), 61 active delegations (38 CPU + 19 GPU + 4 xspring), FAMILY_ID evolution, DRY refactoring, comprehensive deep-debt audit clean |
| Mar 1 | **groundSpring V59** | **ToadStool S71+++ catch-up**: jackknife promoted to GPU (`JackknifeMeanGpu` + `jackknife_mean_f64.wgsl`), Hargreaves batch GPU evolved (`HargreavesBatchGpu` + `hargreaves_batch_f64.wgsl`), ToadStool pin advanced 6 commits (S70+++→S71+++), 671 WGSL shaders, ComputeDispatch builder (66 ops), DF64 transcendental suite complete (15 functions), ~9K lines stale code archived upstream, 61 delegations (37 CPU + 20 GPU + 4 xspring) |
| Mar 1 | **groundSpring V60** | **hotSpring cross-spring absorption**: `DriftMonitor` (`N_e`·`s` tracking from Nautilus Shell), `ClassificationUncertainty` (multi-head ESN disagreement from hotSpring), `detect_concept_edges` (LOO cross-validation from Nautilus Brain), `nautilus` feature gate (`bingocube-nautilus` optional dep), 620 tests (+7), 4 new native functions, 10 new tests |
| Mar 2 | **groundSpring V62** | **ToadStool S79 catch-up**: pollster eliminated (`tokio_block_on`), f64-capable device selection (`WgpuDevice::new_f64_capable` with fallback), DF64 precision strategy wired, 2 redundant shaders removed (absorbed into ToadStool), SPDX harmonized (AGPL-3.0-only), cross-spring shader lineage documented. 710 tests, 23/23 cross-spring benchmark, 39/39 GPU tier, 13/13 Titan V + RTX 4070 validation |
| Mar 2 | **groundSpring V67** | **ToadStool S87 catch-up**: `McEt0PropagateGpu` + `SeasonalPipelineF64` GPU wirings, `BatchedMultinomialGpu::sample` API break fix (3 sites), 73 delegations (43 CPU + 30 GPU), 28 metalForge workloads |
| Mar 2 | **groundSpring V68** | **Complete rewiring + cross-spring benchmark**: L-BFGS refinement (airSpring V035 → S84 → freeze_out), 4D Anderson + Wegner RG (hotSpring precision → S84 → tissue_anderson), 76 delegations (44 CPU + 32 GPU), 30 metalForge workloads. Cross-spring lineage: hotSpring precision shaders enable tissue 4D; airSpring optimizer enables freeze-out sub-grid refinement; wetSpring bio + neuralSpring infra form foundation for all stochastic GPU dispatch |
| Mar 2 | **groundSpring V69** | **S87 pin + cross-spring evolution parity**: 5 new parity tests validating cross-spring shader evolution — Shannon diversity (wetSpring S64), Simpson diversity (wetSpring S64), Seismic grid search (groundSpring S71+++), Anderson 2D (hotSpring S59), Anderson 3D (hotSpring S59). Cross-spring evolution timeline (4-phase) + provenance table (+6 S72-S87 ops). Universal precision audit: "Math is universal, precision is silicon." 783 tests, 187 checks |
| Mar 6 | **groundSpring V87** | **Tier B resolution + cross-spring delegation completion**: `multinomial_sample` CPU-delegated (wetSpring bio → barraCuda S93 cumulative prob adapter), `anderson_potential` CPU-delegated (hotSpring spectral → barraCuda LcgRng), 5 stale Tier B entries resolved (freeze_out/seismic/rare_biosphere already wired, gillespie batch wired), `quasispecies_simulation` + band_structure coarse scan confirmed CPU-by-design. 93 active delegations (56 CPU + 37 GPU), 0 evolution candidates remaining |

---

## V55 Evolution: Modern ToadStool S70+ Rewiring (March 1, 2026)

V55 wires the latest cross-spring capabilities absorbed into barracuda during
ToadStool S70+ (sessions 70 through 70+++).

### New Delegations

| # | Function | barracuda Target | Cross-Spring Origin | When Evolved |
|---|----------|-----------------|--------------------|----|
| 33 | `hargreaves_et0` | `stats::hydrology::hargreaves_et0` | airSpring V035 → ToadStool S70+ | Feb 28, 2026 |
| 34 | `hargreaves_et0_batch` (CPU) | `stats::hydrology::hargreaves_et0_batch` | airSpring V035 → ToadStool S70+ | Feb 28, 2026 |
| 35 | `crop_coefficient` | `stats::hydrology::crop_coefficient` | airSpring FAO-56 → ToadStool S70+ | Feb 28, 2026 |
| 36 | `soil_water_balance` | `stats::hydrology::soil_water_balance` | airSpring precision ag → ToadStool S70+ | Feb 28, 2026 |
| 37 | `hargreaves_et0_batch` (GPU) | `BatchedElementwiseF64(Op::HargreavesEt0)` | airSpring V035 → ToadStool S70+ | Feb 28, 2026 |
| 38 | `find_band_edges` (Brent) | `optimize::brent` | airSpring V035 (Richards PDE root-finding) → ToadStool S70+ | Feb 28, 2026 |

### Cross-Spring Evolution Highlights

**airSpring → ToadStool → groundSpring** (hydrology chain):

- **Hargreaves ET₀**: airSpring needed a temperature-only ET₀ estimate when radiation
  sensors are unavailable or during gap-filling. The Hargreaves equation
  (`ET₀ = 0.0023 · (T_mean + 17.8) · ΔT^0.5 · Ra`) was implemented in airSpring V035,
  absorbed into ToadStool S70+ as `stats::hydrology::hargreaves_et0`, and now
  delegated by groundSpring — giving 28 experiments access to a fallback ET₀ method.
  The GPU batch path uses `BatchedElementwiseF64` with `Op::HargreavesEt0` for
  multi-station, multi-day workloads.

- **Crop coefficient interpolation**: airSpring's FAO-56 implementation included
  crop coefficient stage interpolation (`Kc` progression through initial, development,
  mid-season, and late-season stages). Absorbed into ToadStool S70+ as
  `stats::hydrology::crop_coefficient`, now delegated by groundSpring. This
  completes the ET₀ → ET_c chain for baseCamp Paper 06 (no-till soil health).

- **Soil water balance**: airSpring precision agriculture needed daily soil moisture
  tracking (`θ_{t+1} = min(θ_t + P + I − ET_c, FC)`). Absorbed into ToadStool S70+
  as `stats::hydrology::soil_water_balance`. Combined with Kc and ET₀, groundSpring
  now has the full soil-plant-atmosphere continuum for uncertainty propagation.

**airSpring → ToadStool → groundSpring** (numerical methods):

- **Brent root-finding for band edges**: airSpring's Richards PDE solver needed
  robust root-finding for nonlinear infiltration equations. The Brent solver was
  absorbed into ToadStool S70+ as `barracuda::optimize::brent`. groundSpring now
  uses it to refine coarse band edge detections from the energy scan to machine
  precision (1e-12 tolerance, 100 max iterations). This is a cross-domain transfer:
  an agricultural soil physics method now improves condensed-matter band structure
  calculations — a textbook example of how the ecoPrimals "Springs don't import,
  they learn" model creates unexpected cross-pollination.

### V80 Full-Suite Benchmark (March 5, 2026)

28 validation binaries, release mode, barraCuda v0.3.3, i9-12900K:

| Mode | Binaries | Total Time | Ratio |
|------|----------|------------|-------|
| Default (no barracuda) | 28/28 PASS | 21.7s | 1.0× |
| barraCuda (CPU delegation) | 28/28 PASS | 19.6s | **1.11×** (−10%) |

**Notable cross-spring speedups in barraCuda mode:**
- FAO-56 (airSpring hydrology): −78% (74ms → 16ms)
- Spectral reconstruction (hotSpring Lanczos): −81% (62ms → 12ms)
- Freeze-out (airSpring L-BFGS): −67% (30ms → 10ms)
- Jackknife (wetSpring stats): −57% (14ms → 6ms)
- Precision drift (hotSpring DF64): −17% (4162ms → 3453ms)

Cross-spring benchmark: **23/23 PASS** (4.5s total).

### V55 Three-Mode Benchmark (March 1, 2026, historical)

333 lib tests timed in three feature modes, ToadStool S70+:

| Mode | Tests | Time | Ratio |
|------|-------|------|-------|
| Default (no barracuda) | 333/333 pass | 4.30s | 1.0× |
| barracuda (CPU delegation) | 333/333 pass | 4.26s | 1.0× (−1%) |
| barracuda-gpu | 327/333 pass* | 0.24s | **17.9×** |

*6 GPU failures are pre-existing `enable f64` WGSL parser compatibility on this
hardware — shader compilation fails before dispatch. All 6 fall back to CPU
correctly in production.

### Delegation Summary (V93 Current)

| Tier | Count | Notes |
|------|-------|-------|
| CPU active | 60 | barraCuda v0.3.3 canonical — V87: +multinomial_sample, +anderson_potential |
| GPU active | 41 | includes 4D Anderson, Wegner RG, McEt0, seasonal pipeline, JackknifeMeanGpu, HargreavesBatchGpu |
| CPU by design | 2 | `quasispecies_simulation` (per-gen mutation thinning), `band_structure` coarse scan (data-dependent matrix chains) |
| Evolution candidates | 0 | Tier B fully resolved |
| **Total active** | **102** | 907 tests, clippy pedantic clean |

### NUCLEUS Integration (V63)

| Capability | Status | Provider |
|-----------|--------|----------|
| Socket discovery | ✅ active | biomeOS |
| Tower health + beacon | ✅ live | crypto capability |
| Compute health/caps/version | ✅ live | compute capability |
| AI health | ✅ live | AI capability |
| Storage put/get | ○ requires Nest mode | storage capability |
| Data pipelines (NCBI, NOAA, IRIS) | ○ requires Nest mode | data capability |
| science.* registration (7 caps) | ✅ V63 | groundSpring self-registration |
| Configurable timeouts | ✅ V63 | env vars |
| Capability-based discovery | ✅ V63 | zero hardcoded primal names |

### baseCamp Paper Coverage (V63)

| Paper | groundSpring Experiments | Key Feature |
|-------|------------------------|-------------|
| 01 (Anderson-QS) | Exp 008, 009, 015, 018 | Spectral theory validation |
| 02 (LTEE Extensions) | Exp 014, 016, 017 | Drift vs selection, quasispecies |
| 03 (BioAg) | Exp 004, 016, 019 | Rare biosphere, jackknife |
| 04 (Sentinels) | Exp 001, 015, 028 | NPU Anderson classification |
| 05 (Cross-Species) | Exp 012, 018 | Transport, band edge |
| 06 (No-Till) | Exp 022-024 | Uncertainty bridge |
| 07 (WDM) | Exp 019-021, 025-027 | Freeze-out, convergence |
| 08 (NPU Ag IoT) | Exp 028 | Cross-substrate parity |
| 09 (Field Genomics) | Exp 016, 028 | Rare biosphere, metalForge |
| **12 (Immuno-Anderson)** | **Exp 008, 012, 015, 018** | **Cytokine transport, 2D/3D spectral, ConceptEdge, DriftAction** |
