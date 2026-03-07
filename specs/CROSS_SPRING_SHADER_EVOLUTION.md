# Cross-Spring Shader Evolution Provenance

> How WGSL shaders flow between springs via barraCuda absorption, creating
> a shared compute fabric where each spring benefits from every other's
> domain-specific innovations.

**Last updated**: March 7, 2026 (V95)

## Philosophy

The ecoPrimals shader evolution follows a natural cycle:

1. **Spring creates** — domain experts write the first WGSL shader for their
   specific science (nuclear physics, earth science, marine biology, ML).
2. **barraCuda absorbs** — the shader becomes a universal op accessible to all.
3. **Other springs benefit** — cross-spring reuse without reimplementation.
4. **coralReef compiles** — WGSL → native GPU binary (NVIDIA SASS, AMD GFX)
   via sovereign compiler, eliminating vendor toolchains.

## Provenance Tree

### Precision Foundation (All Springs)

| Shader | Origin | barraCuda Op | Cross-Spring Users |
|--------|--------|--------------|--------------------|
| `df64_core.wgsl` | hotSpring biomeGate | DF64 arithmetic | **All** DF64 shaders across all springs |
| `math_f64.wgsl` | wetSpring + hotSpring fixes | Core f64 math | **All** f64 shaders |
| `complex_f64.wgsl` | hotSpring | Complex arithmetic | lattice QCD, spectral |
| 13-tier tolerances | groundSpring V74 | `tolerances.rs` | **All** springs via validation |

**Note**: hotSpring's DF64 (double-float via f32 pairs, ~48-bit mantissa) gives
f64-class precision on GPUs that lack native f64. This single innovation —
created for nuclear structure calculations — now powers earth science Monte Carlo,
marine bio diversity metrics, and neural network training validation.

### hotSpring → barraCuda → Others (Nuclear Physics, Lattice QCD, MD)

| Shader | Absorbed | Now Used By |
|--------|----------|-------------|
| `nuclear/semf_*_f64.wgsl` (6) | Mar 4-5 2026 | hotSpring nuclear structure |
| `lattice/su3_*_f64.wgsl` (18+) | Feb 2026 | hotSpring lattice QCD |
| `lattice/wilson_plaquette_*.wgsl` | Feb 2026 | hotSpring lattice QCD |
| `lattice/dirac_staggered_f64.wgsl` | Feb 2026 | hotSpring fermion actions |
| `science/hfb_*_f64.wgsl` (12) | Feb 2026 | hotSpring Hartree-Fock-Bogoliubov |
| `md/vacf_dot_f64.wgsl` | Feb 2026 | hotSpring MD, **groundSpring** WDM transport |
| `md/heat_current_f64.wgsl` | Feb 2026 | hotSpring thermal transport |
| `linalg/batched_eigh_*_f64.wgsl` | Feb 2026 | hotSpring, **neuralSpring** eigensolve |
| `linalg/cholesky.wgsl` | Feb 2026 | hotSpring, **groundSpring** Tikhonov |
| `mixing/broyden_f64.wgsl` | Feb 2026 | hotSpring self-consistent field |
| `optimizer/batched_bisection_f64.wgsl` | Feb 16 2026 | hotSpring, **groundSpring** band edges |
| `ml/esn_reservoir_update_f64.wgsl` | Feb 2026 | hotSpring, **wetSpring** reservoir computing |
| `ml/esn_readout_f64.wgsl` | Feb 2026 | hotSpring, **groundSpring** ESN regime classify |

**Cross-spring highlight**: hotSpring's ESN reservoir shaders, originally for nuclear
structure regime detection, are now used by groundSpring for Anderson localization
regime classification and by wetSpring for marine ecosystem state detection.

### groundSpring → barraCuda → Others (Earth Science, Anderson, Statistics)

| Shader | Absorbed | Now Used By |
|--------|----------|-------------|
| `spectral/fft_radix2_f64.wgsl` | Mar 4-5 2026 | **All** springs via FFT ops |
| `spectral/anderson_lyapunov_f64.wgsl` | Mar 4-5 2026 | groundSpring Anderson, **hotSpring** nuclear disorder |
| `special/chi_squared_f64.wgsl` | Mar 4-5 2026 | groundSpring, **hotSpring** SEMF fitting |
| `special/rawr_weighted_mean_f64.wgsl` | V10/V54 | groundSpring bootstrap, **neuralSpring** ensemble |
| `bio/mc_et0_propagate_f64.wgsl` | V10 → S72 | groundSpring FAO-56, **airSpring** ET₀ |
| `grid/grid_fit_2d_f64.wgsl` | V31 | groundSpring freeze-out, **hotSpring** EOS fitting |
| `reduce/mean_variance_f64.wgsl` | Kokkos pattern | **All** springs |
| `reduce/correlation_full_f64.wgsl` | V80 Kokkos | groundSpring, **hotSpring**, **wetSpring** |
| `stats/autocorrelation_f64.wgsl` | V91 wiring | groundSpring WDM, **hotSpring** MD transport |

**Cross-spring highlight**: groundSpring's FFT radix-2 shader, created for spectral
reconstruction of lattice correlators, is now the universal FFT op used by hotSpring
for PPPM electrostatics, wetSpring for signal processing, and neuralSpring for
spectral analysis of training dynamics.

### wetSpring → barraCuda → Others (Marine Bio, Diversity, Genomics)

| Shader | Absorbed | Now Used By |
|--------|----------|-------------|
| `math/bray_curtis_f64.wgsl` | Feb 16 2026 | wetSpring, **airSpring** ecological diversity |
| `reduce/fused_map_reduce_f64.wgsl` | S66 | wetSpring, **airSpring**, **hotSpring** |
| `math/cosine_similarity_f64.wgsl` | Priority 2 | wetSpring, **neuralSpring** embedding similarity |
| `math/hill_f64.wgsl` | Exp019 | wetSpring dose-response |
| `ml/rf_batch_inference.wgsl` | v5, Feb 20 | wetSpring random forest |
| `bio/hmm_forward_f64.wgsl` | Pattern | wetSpring, **neuralSpring** sequence analysis |

**Cross-spring highlight**: wetSpring's Bray-Curtis and Shannon/Simpson diversity
shaders, created for marine metagenomics, are now used by airSpring for
agricultural biodiversity monitoring and by groundSpring for rarefaction analysis.

### neuralSpring → barraCuda → Others (ML, Spectral, Bio)

| Shader | Absorbed | Now Used By |
|--------|----------|-------------|
| `special/fused_chi_squared_f64.wgsl` | V24 | neuralSpring, **hotSpring** nuclear |
| `special/fused_kl_divergence_f64.wgsl` | V24 | neuralSpring training |
| `spectral/batch_ipr_f64.wgsl` | metalForge Feb 21 | neuralSpring, **groundSpring** Anderson IPR |
| `numerical/hessian_column_f64.wgsl` | baseCamp V18 | neuralSpring optimization |
| `stats/histogram_f64.wgsl` | baseCamp V18 | neuralSpring, **groundSpring** distributions |
| `sample/metropolis_f64.wgsl` | baseCamp V18 | neuralSpring, **hotSpring** HMC |
| `linalg/laplacian_f64.wgsl` | baseCamp V18 | neuralSpring graph learning |
| `math/matmul_*_f64.wgsl` | handoff #11 | neuralSpring, **hotSpring** linalg |
| `bio/batch_fitness_eval.wgsl` | metalForge Feb 21 | neuralSpring evolutionary |

**Cross-spring highlight**: neuralSpring's Metropolis sampling shader, originally
for Bayesian neural network training, is now used by hotSpring for Hybrid Monte
Carlo in lattice QCD — different physics, same algorithm.

### airSpring → barraCuda → Others (Agriculture, Hydrology)

| Shader | Absorbed | Now Used By |
|--------|----------|-------------|
| `spectral/anderson_coupling_f64.wgsl` | S69 | **groundSpring** Anderson coupling |
| `science/seasonal_pipeline.wgsl` | V035 → S70 | airSpring, **groundSpring** FAO-56 seasonal |
| `optimize/brent_f64.wgsl` | V035 | airSpring, **groundSpring** band structure |
| `grid/van_genuchten_f64.wgsl` | Agriculture | airSpring soil physics |
| `science/batched_elementwise_f64.wgsl` ops 17-19 | Mar 4-5 2026 | airSpring SCS-CN, Stewart, Blaney-Criddle |
| `stats/moving_window_f64.wgsl` | IoT | airSpring, **wetSpring** sensor streams |

**Cross-spring highlight**: airSpring's Brent root-finding shader, created for
agricultural optimization, is now used by groundSpring for Anderson band edge
detection — utterly different domains, identical numerical method.

## Cross-Spring Benefit Matrix

Shows which springs benefit from each other's shader contributions:

| Shader Origin → | hotSpring | groundSpring | wetSpring | neuralSpring | airSpring |
|-----------------|-----------|--------------|-----------|--------------|-----------|
| **hotSpring uses** | ■ | FFT, chi², tolerances | Bray-Curtis, HMM | Metropolis, matmul | Brent |
| **groundSpring uses** | DF64, Cholesky, bisection, ESN | ■ | fused_map_reduce | batch_ipr, histogram | coupling, seasonal, Brent |
| **wetSpring uses** | ESN, DF64 | mean_variance, RAWR | ■ | KL, cosine_sim | moving_window, Bray-Curtis |
| **neuralSpring uses** | DF64, batched_eigh | correlation, chi² | HMM, diversity | ■ | — |
| **airSpring uses** | DF64 | MC ET₀, FFT | Bray-Curtis, diversity | — | ■ |

## Evolution Timeline

```
Feb 2026    hotSpring: lattice QCD, HFB, MD, DF64 core absorbed
            wetSpring: Bray-Curtis, RF inference absorbed
            neuralSpring: baseCamp V18 shaders absorbed

Feb 21      neuralSpring metalForge: batch_ipr, fitness, pairwise absorbed

Mar 3-5     Major absorption wave:
            - hotSpring: nuclear shaders, VACF
            - groundSpring: FFT radix-2, Anderson Lyapunov, chi², tolerances
            - airSpring: ops 17-19 (SCS-CN, Stewart, Blaney-Criddle)
            - neuralSpring: HMM forward/backward

Mar 6-7     groundSpring V90-V91: deep debt, rewire to modern ecosystem
            - CovarianceF64, AutocorrelationF64, PeakDetectF64 wired
            - Marchenko-Pastur, empirical_spectral_density wired
            - Cross-spring provenance documented
```

## Sovereign Compilation Path (coralReef)

All WGSL shaders in barraCuda (708) can now be compiled to native GPU binary
via coralReef Phase 11:

| Target | Backend | Status |
|--------|---------|--------|
| NVIDIA SM70-SM89 | NvidiaBackend (SASS) | Complete, 801 tests |
| AMD RDNA2/GFX1030 | AmdBackend (GFX) | Complete |
| Intel (future) | IntelBackend | Planned |

The f64 lowering pipeline (sqrt, rcp, exp2, log2, sin, cos via Newton-Raphson
on MUFU seeds) means every spring gets native f64 performance even on GPUs
that don't natively support it — a benefit that started with hotSpring's
nuclear precision requirements and now powers all science across the ecosystem.
