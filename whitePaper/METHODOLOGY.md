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

### Phase 1: Rust / BarraCUDA Validation

Phase 1 ports each experiment's core algorithm to idiomatic Rust in the
`groundspring` crate, validated by binaries that follow the hotSpring pattern
(exit 0 = all pass, exit 1 = any fail).

| Binary | Experiment | Checks | Modules Exercised |
|--------|-----------|--------|-------------------|
| `validate-decompose` | Exp 001 | 36 | `stats`, `decompose` |
| `validate-rarefaction` | Exp 004 | 15 | `rarefaction`, `prng` |
| `validate-seismic` | Exp 005 | 9 | `seismic`, `stats` |
| `validate-weather` | Exp 002 | 13 | `stats`, `decompose`, `prng` |
| `validate-fao56` | Exp 003 | 15 | `fao56`, `prng`, `stats` |

### Rust Quality Gates

| Gate | Requirement |
|------|-------------|
| `cargo test` | 90 unit + 1 doc test, all pass |
| `cargo clippy` | Zero warnings (pedantic + nursery) |
| `cargo fmt` | Clean |
| `cargo doc` | Clean, `missing_docs = "deny"` |
| `cargo llvm-cov` | 99.7% line, 100% function |
| No `unsafe` | Enforced at workspace lint level |
| Max file size | 1000 lines per file |
| Provenance | Benchmark JSONs have real commit SHA |

### GPU Evolution Methodology

groundSpring follows the **Write → Absorb → Lean** cycle (hotSpring pattern):

1. Write CPU implementations + production WGSL shaders (`metalForge/shaders/`)
2. Validate CPU against Python baselines (88/88 checks)
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

- **Phase 0 (Python)**: 71/71 quantitative checks passed across 5 experiments, 4 domains.
- **Phase 1 (Rust)**: 88/88 checks passed across 5 validation binaries, 99.7% coverage.
