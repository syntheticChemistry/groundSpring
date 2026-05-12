# groundSpring Methodology

## Experimental Design

groundSpring follows the same multi-phase validation approach as airSpring, applied to the domain of measurement uncertainty and noise characterization.

### Phase 0: Python/NumPy/SciPy Baselines

Every experiment begins with a pure Python implementation that:

1. **Defines the question** — What is being measured, and what noise is expected?
2. **Implements the model** — Forward model, statistical decomposition, or simulation
3. **Digitizes benchmark data** — From published papers or known analytical results
4. **Validates against benchmarks** — With explicit numerical tolerances
5. **Reports key findings** — Quantitative answers to the groundSpring question

### Validation Framework

Each experiment produces:

- **Benchmark JSON file** — Expected values with sources and tolerances
- **Python script** — Self-contained implementation with validation harness
- **PASS/FAIL checks** — Explicit numerical comparison against benchmarks
- **Key findings** — Summary of what was learned about noise in that domain

### Shared Statistical Language

All experiments use a common statistical framework:

| Metric | Definition | Use |
|--------|-----------|-----|
| RMSE | Root Mean Square Error | Total error magnitude |
| MBE | Mean Bias Error | Systematic bias |
| R² | Coefficient of Determination | Correlation strength |
| IA | Index of Agreement (Willmott) | Pattern agreement |
| Bias fraction | MBE²/RMSE² | What fraction of error is correctable |
| CV | Coefficient of Variation | Relative uncertainty |

### Error Decomposition

The core groundSpring operation is decomposing total error into components:

```
RMSE² = MBE² + σ²(random)

where:
  MBE = systematic bias (correctable with calibration)
  σ(random) = irreducible noise (the noise floor)
  bias_fraction = MBE² / RMSE² (how much is correctable)
```

This decomposition is applied across all domains:
- Sensor calibration (Exp 001)
- Model-observation gaps (Exp 002)
- Input-to-output propagation (Exp 003)
- Sampling noise (Exp 004)
- Inverse problem uncertainty (Exp 005)
- Stochastic biochemical noise (Exp 006)
- Resampling confidence estimation (Exp 007)
- Disorder-induced localization (Exp 008)
- Quasiperiodic localization (Exp 009)
- Bistable phenotypic switching (Exp 010)
- Multi-signal QS integration (Exp 011)

## Experiment Details

### Exp 001: Sensor Noise Characterization

**Input**: Factory calibration statistics from Dong et al. (2020)
**Method**: Bias-variance decomposition of RMSE into MBE + random components
**Validation**: 18 bias/std/fraction checks + 6 noise floor + 2 cross-soil + 6 normality = 32 checks

### Exp 002: Weather Model vs Observation

**Input**: Open-Meteo ERA5 reanalysis + NOAA CDO station data (Lansing, MI 2023)
**Method**: Side-by-side comparison of temperature and precipitation
**Validation**: 3 metric computation + 1 hit rate + 1 seasonal = 5 checks (synthetic mode)
**Note**: Full validation requires real NOAA CDO token

### Exp 003: Error Propagation Through FAO-56

**Input**: FAO-56 Example 18 (Uccle, Belgium) + WMO sensor uncertainties
**Method**: Monte Carlo (N=10,000) + analytical Taylor expansion + sensitivity analysis
**Validation**: 1 baseline + 4 MC + 2 sensitivity + 1 analytical = 8 checks

### Exp 004: Sequencing Depth & Taxonomic Noise

**Input**: Synthetic 150-genus, 8-phylum soil microbiome community
**Method**: Multinomial rarefaction at 9 depths × 50 replicates
**Validation**: 2 community + 8 pattern + 3 convergence + 1 noise floor + 2 monotonicity = 16 checks

### Exp 005: Seismic Wave Propagation

**Input**: Synthetic New Madrid earthquake + 7 NMSZ stations
**Method**: 1D travel-time forward model + grid-search + Nelder-Mead inversion + MC uncertainty
**Validation**: 2 forward + 3 grid + 1 refinement + 1 noisy + 2 MC + 1 subset = 10 checks

### Exp 006: Enzymatic Signal Specificity

**Input**: Massie et al. (2012, PNAS) c-di-GMP birth-death kinetics
**Method**: Gillespie SSA with analytical steady-state comparison + SNR sweep
**Validation**: 2 analytical + 2 SSA + 4 SNR + 2 determinism = 12 checks (Python and Rust)

### Exp 007: RAWR Resampling

**Input**: Synthetic Gaussian, log-normal, and AR(1) correlated data
**Method**: Standard percentile bootstrap vs RAWR weighted resampling (Wang et al. 2021)
**Validation**: 2 estimates + 1 CI width + 4 coverage + 1 RMSE ratio + 2 determinism = 11 checks (Python and Rust)

### Exp 008: Anderson Localization

**Input**: 1D tight-binding Anderson model with varying disorder (Bourgain-Kachkovskiy 2018)
**Method**: Transfer-matrix Lyapunov exponent computation + Thouless scaling
**Validation**: 1 clean + 1 positivity + 1 monotonicity + 1 strong disorder + 1 Thouless + 1 ξ ordering + 2 determinism = 8 checks (Python and Rust)

### Exp 009: Almost-Mathieu Quasiperiodic Localization

**Input**: Almost-Mathieu operator with golden-ratio frequency (Paper #16)
**Method**: Aubry-André metal-insulator transition, Herman's formula, level spacing statistics
**Validation**: 1 clean + 3 coupling sweep + 1 critical + 1 monotonicity + 2 level spacing = 8 checks

### Exp 010: Bistable Phenotypic Switching

**Input**: Fernandez et al. (2020 PNAS) 5-variable ODE parameters
**Method**: Deterministic integration with two initial conditions; monostable control; stochastic switching
**Validation**: 2 cell capacity + 4 attractor + 1 monostable + 1 determinism + 1 stochastic = 9 checks

### Exp 011: Multi-Signal QS Integration

**Input**: Srivastava et al. (2011 J Bacteriology) 7-variable dual-signal ODE
**Method**: Dual-signal vs single-signal comparison; deterministic and low-noise stochastic
**Validation**: 3 steady state + 3 single-signal + 1 determinism + 1 low-noise = 8 checks

### Phase 1: Rust / BarraCUDA Validation

Phase 1 ports each experiment's core algorithm to idiomatic Rust in the
`groundspring` crate, validated by binaries that follow the hotSpring pattern
(exit 0 = all pass, exit 1 = any fail).

| Binary | Experiment | Checks | Modules Exercised |
|--------|-----------|--------|-------------------|
| `validate_decompose` | Exp 001 | 36 | `stats`, `decompose` |
| `validate_rarefaction` | Exp 004 | 15 | `rarefaction`, `prng` |
| `validate_seismic` | Exp 005 | 9 | `seismic`, `stats` |
| `validate_weather` | Exp 002 | 13 | `stats`, `decompose`, `prng` |
| `validate_fao56` | Exp 003 | 15 | `fao56`, `prng`, `stats` |
| `validate_signal_specificity` | Exp 006 | 12 | `gillespie`, `prng` |
| `validate_rawr` | Exp 007 | 11 | `bootstrap`, `prng`, `stats` |
| `validate_anderson` | Exp 008 | 8 | `anderson`, `prng` |
| `validate_quasiperiodic` | Exp 009 | 8 | `anderson` |
| `validate_bistable` | Exp 010 | 9 | `bistable` |
| `validate_multisignal` | Exp 011 | 8 | `multisignal` |

### Rust Quality Gates

| Gate | Requirement |
|------|-------------|
| `cargo test` | 965+ Rust tests (includes three-tier parity integration tests), all pass |
| `cargo clippy` | Zero warnings (pedantic + nursery) × 3 feature modes |
| `cargo fmt` | Clean |
| `cargo doc` | Clean, `missing_docs = "deny"` |
| Python baseline integrity | 322 tests (provenance, completeness, UTF-8, three-tier parity) via `test_baseline_integrity.py` and `test_three_tier_parity.py` |
| Three-tier parity | CPU → GPU → metalForge validation chain exercised by `three_tier_parity.rs` and `test_three_tier_parity.py` |
| No `unsafe` | Enforced at workspace lint level |
| Max file size | 1000 lines per file |
| Provenance | Benchmark JSONs have real commit SHA |

### GPU Evolution Methodology

groundSpring follows the **Write → Absorb → Lean** cycle (hotSpring pattern):

1. Write CPU implementations + production WGSL shaders (`metalForge/shaders/`)
2. Validate CPU against Python baselines (395/395 checks)
3. Hand off WGSL to ToadStool/BarraCUDA with binding layout documentation
4. BarraCUDA absorbs as upstream op
5. groundSpring rewires behind `#[cfg(feature = "barracuda")]`
6. Re-run validation binaries to confirm GPU-CPU agreement within tolerance

WGSL conventions (matching hotSpring):
- `struct Params` for uniforms (u32-aligned with padding)
- `@group(0) @binding(N)` sequential bindings
- `@compute @workgroup_size(64, 1, 1)` standard workgroup
- xoshiro128** PRNG matching `barracuda::ops::prng_xoshiro_wgsl`
- f64 precision throughout
- Documented binding layouts and dispatch geometry in shader headers

## Hardware Gate

Same as all ecoPrimals springs:

| Component | Specification |
|-----------|--------------|
| CPU | Intel i9-12900K (16C/24T) |
| RAM | 64 GB DDR5-4800 |
| GPU | NVIDIA RTX 4070 (12 GB) |
| OS | Pop!_OS 22.04 |

## Grand Total

- **Phase 0 (Python)**: ~288 quantitative checks passed across 28 experiments, 9 domains.
- **Phase 1 (Rust)**: 395/395 checks passed across 35 validation binaries (340 core + 55 NUCLEUS). 965+ Rust tests.
- **Phase 2a (Barracuda)**: 110 delegations (67 CPU + 43 GPU) — barraCuda v0.4.0. 11.6× faster than Python (excl. LAPACK-bound). 29/29 parity proven. 965+ tests, ≥92% library coverage. runtime f64 smoke test + three-tier parity (V121).
- **Phase 4 (NUCLEUS)**: biomeOS Neural API live — Tower + Node + Squirrel validated. NestGate data pipelines (NCBI, NOAA, IRIS). 4 NUCLEUS experiments (Exp 029–032).
- **metalForge**: 5 live hardware binaries (RTX 4070, Titan V, AKD1000 NPU). 138 metalForge checks, 30 workloads, 5 substrates, architecture-aware routing, `PCIe` topology, GPU→NPU bypass, pipeline dispatch, NUCLEUS atomics (V113).
