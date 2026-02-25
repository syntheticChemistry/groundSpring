# groundSpring — Control Experiment Status

**Last updated**: February 25, 2026

## Experiment Register

| ID | Title | Domain | Phase 0 (Python) | Phase 1 (Rust) |
|----|-------|--------|:-----------------:|:--------------:|
| 001 | Sensor Noise Characterization | Agricultural (soil sensors) | 32/32 PASS | 36/36 PASS |
| 002 | Weather Model vs Observation | Meteorological | PASS (synthetic NOAA) | 13/13 PASS |
| 003 | Error Propagation Through FAO-56 | Agricultural (ET₀) | PASS | 15/15 PASS |
| 004 | Sequencing Depth & Taxonomic Noise | Biological (microbiome) | PASS | 15/15 PASS |
| 005 | Seismic Wave Propagation | Geological (seismology) | PASS | 9/9 PASS |
| 006 | Enzymatic Signal Specificity | Biological (c-di-GMP) | 12/12 PASS | 12/12 PASS |
| 007 | RAWR Resampling | Statistics (bootstrap) | 11/11 PASS | 11/11 PASS |
| 008 | Anderson Localization | Mathematics (spectral theory) | 8/8 PASS | 8/8 PASS |

**Python Phase 0**: All 8 experiments passing
**Rust Phase 1**: 119/119 PASS across 8 validation binaries
**pytest**: 34/34 PASS (unit tests, determinism tests, integration tests)

## Phase 0 — Python/NumPy/SciPy Baselines

### Exp 001: Sensor Noise Characterization (32/32 PASS)

**Question**: How much of the RMSE between factory-calibrated and field-measured soil moisture is systematic bias (correctable) vs random noise (irreducible)?

**Results**:
- CS616 in sand: NOISE-dominated (34.6% bias), noise floor 0.006 m³/m³
- CS616 in loamy sand: BIAS-dominated (59.2% bias), noise floor 0.021 m³/m³
- CS616 in sandy clay loam: NOISE-dominated (26.3% bias), noise floor 0.012 m³/m³
- EC5 across all soils: BIAS-dominated (62-77%), noise floor 0.004-0.020 m³/m³
- Site-specific calibration removes 50-80% of total sensor error

### Exp 002: Weather Model vs Observation (PASS)

**Question**: How does ERA5 reanalysis (Open-Meteo) differ from GHCND station observations (NOAA CDO)?

**Results** (synthetic NOAA mode — accuracy checks SKIPPED):
- Metrics computed for tmax, tmin, precipitation
- Seasonal decomposition shows winter has largest gaps
- **Pending**: Real NOAA CDO data for full accuracy validation

### Exp 003: Error Propagation Through FAO-56 (PASS)

**Question**: Given known sensor uncertainties, how does measurement noise propagate through the FAO-56 ET₀ chain?

**Results**:
- ET₀ uncertainty: 3.879 ± 0.142 mm/day (CV = 3.7%)
- 90% CI: [3.647, 4.118] mm/day
- Sensitivity ranking: humidity > radiation > temperature > wind
- Monte Carlo / analytical agreement: ratio ≈ 1.0

### Exp 004: Sequencing Depth & Taxonomic Noise (PASS)

**Question**: At what sequencing depth does 16S taxonomic assignment become stable?

**Results**:
- All phyla detected: ~100 reads
- Shannon convergence (5%): ~500 reads
- Genus saturation: ~5,000 reads
- Noise floor at 100k reads: ±0.004 Shannon, ±0.4 genera

### Exp 005: Seismic Wave Propagation (PASS)

**Question**: Can we locate an earthquake source from noisy P-wave arrivals?

**Results**:
- Clean inversion: 0.00 km error (perfect recovery)
- Noisy inversion (±0.5s): <30 km error
- Monte Carlo uncertainty: ±2 km horizontal (90th: ~4 km)
- Depth poorly constrained with surface-only stations

### Exp 006: Enzymatic Signal Specificity (12/12 PASS)

**Question**: How does a cell resolve signal from noise when 40+ enzymes compete for c-di-GMP?

**Results**:
- Steady-state mean matches Gillespie SSA (18.2 molecules, Poisson variance)
- 10× activation SNR ≈ 0.97; 20× activation SNR ≈ 2.03
- SNR monotonically increases with activation fold-change
- Specificity requires α >> N_dgc for SNR >> 1

### Exp 007: RAWR Resampling (11/11 PASS)

**Question**: Does weighted resampling outperform naive bootstrap for confidence estimation?

**Results**:
- Gaussian: both methods achieve ~95% coverage
- Skewed (log-normal): RAWR slightly better coverage than naive bootstrap
- Correlated (AR(1)): RAWR/Bootstrap RMSE ratio ≈ 1.0 (competitive)
- Both methods fully deterministic with same seed

### Exp 008: Anderson Localization (8/8 PASS)

**Question**: When does a wave propagate through a disordered medium vs when does noise trap it?

**Results**:
- Clean system (W=0): γ = 0 (extended states)
- ANY disorder localizes (γ > 0 for all W > 0)
- Thouless scaling: ξ ≈ 104/W² at band center
- Strong disorder (W=8): ξ ≈ 1.9 sites (strongly localized)

## Phase 1 — Rust Validation (hotSpring Pattern)

### validate-decompose (36/36 PASS)

Ports Exp 001 core algorithm to pure safe Rust.  Verifies:
- All 6 sensor-soil decompositions match analytically derived expected values
- Pythagorean identity (RMSE² = MBE² + σ²) holds to machine epsilon
- Noise floor reduction pythagorean holds

### validate-rarefaction (15/15 PASS)

Ports Exp 004 core algorithm to pure safe Rust.  Verifies:
- Shannon diversity analytical known values (uniform, single-species)
- Multinomial sampling determinism, total conservation, proportion accuracy
- Rarefaction convergence properties (monotonicity, high-depth completeness)

### validate-seismic (9/9 PASS)

Ports Exp 005 core algorithm to pure safe Rust.  Verifies:
- Haversine distance known values (zero, NY-London)
- Travel-time proportionality and known values
- Grid-search inversion recovers clean source exactly

### validate-weather (13/13 PASS)

Ports Exp 002 core algorithm to pure safe Rust.  Verifies:
- Precipitation hit-rate analytical known values
- Temperature stats (RMSE, MBE, R², IA) on constant-bias and noisy cases
- Bias-variance decomposition on weather-domain data
- Edge cases (empty hit_rate)

### validate-fao56 (15/15 PASS)

Ports Exp 003 core algorithm to pure safe Rust.  Verifies:
- Penman-Monteith ET₀ against FAO-56 Example 18 (Uccle, Belgium)
- Monte Carlo error propagation (mean, std, CV, 90% CI)
- Determinism of MC runs
- Sensitivity ranking (humidity > radiation > temperature > wind)

### validate-signal-specificity (12/12 PASS)

Ports Exp 006 Gillespie SSA to pure safe Rust.  Verifies:
- Analytical steady-state mean and Poisson variance
- Ensemble mean from 200 stochastic trajectories
- Response ratios at α=10, α=20
- SNR scaling (monotonicity, range bounds)
- Determinism with same/different seeds

### validate-rawr (11/11 PASS)

Ports Exp 007 bootstrap/RAWR to pure safe Rust.  Verifies:
- Bootstrap and RAWR CIs on Gaussian data
- Coverage rates across 200 trials (Gaussian, skewed, correlated)
- RAWR/Bootstrap RMSE ratio on correlated data
- Determinism of both methods

### validate-anderson (8/8 PASS)

Ports Exp 008 transfer-matrix Lyapunov to pure safe Rust.  Verifies:
- Clean system γ ≈ 0
- All disordered states have γ > 0
- γ monotonically increases with W
- Thouless scaling (C = ξ·W² in [60, 140])
- Localization length decreases with disorder
- Determinism of potential generation and Lyapunov computation

## Test Infrastructure

| Suite | Tests | Type |
|-------|------:|------|
| `test_common.py` | 18 | Unit tests for shared statistical primitives |
| `test_determinism.py` | 7 | Rerun-identical verification for stochastic ops |
| `test_experiments.py` | 8 | Integration: each experiment returns exit code 0 |
| Rust `#[test]` | 108 | Unit tests for Rust library modules (+ 1 doc test) |
| **Total** | **141** | |

## Run Log

### Run 3 — February 25, 2026 (Paper queue experiment buildout)

```
Phase 0 (Python):
  Exp 001: Sensor Noise              32/32 PASS
  Exp 002: Observation Gap            PASS (synthetic SKIP)
  Exp 003: Error Propagation          PASS
  Exp 004: Sequencing Noise           PASS
  Exp 005: Seismic Inversion          PASS
  Exp 006: Signal Specificity        12/12 PASS
  Exp 007: RAWR Resampling           11/11 PASS
  Exp 008: Anderson Localization      8/8  PASS

Phase 1 (Rust):
  validate-decompose                 36/36 PASS
  validate-rarefaction               15/15 PASS
  validate-seismic                    9/9  PASS
  validate-weather                   13/13 PASS
  validate-fao56                     15/15 PASS
  validate-signal-specificity        12/12 PASS
  validate-rawr                      11/11 PASS
  validate-anderson                   8/8  PASS

pytest:
  test_common                        18/18 PASS
  test_determinism                    7/7  PASS
  test_experiments                    8/8  PASS (3 new)
```

### Run 2 — February 25, 2026 (Phase 1 port)

```
Phase 0 (Python):
  Exp 001: Sensor Noise              32/32 PASS
  Exp 002: Observation Gap            PASS (synthetic SKIP)
  Exp 003: Error Propagation          PASS
  Exp 004: Sequencing Noise           PASS
  Exp 005: Seismic Inversion          PASS

Phase 1 (Rust):
  validate-decompose                 36/36 PASS
  validate-rarefaction               15/15 PASS
  validate-seismic                    9/9  PASS
  validate-weather                   13/13 PASS
  validate-fao56                     15/15 PASS

pytest:
  test_common                        18/18 PASS
  test_determinism                    7/7  PASS
  test_experiments                    5/5  PASS
```

### Run 1 — February 16, 2026 (initial baselines)

```
Exp 001–005: 71/71 PASS (Python only)
```

## Three-Tier Control Matrix

Each experiment is validated at three hardware tiers:

| Tier | Substrate | Description |
|------|-----------|-------------|
| **CPU** | `cargo test` + validation binary | Rust matches Python baseline |
| **GPU** | `barracuda` feature + GPU adapter | GPU matches CPU within tolerance |
| **metalForge** | Mixed hardware dispatch | Cross-substrate agreement |

### Current Status

| # | Experiment | CPU | GPU | metalForge | GPU Blocker |
|---|-----------|:---:|:---:|:----------:|-------------|
| 1 | Sensor noise decomposition | **36/36 PASS** | Pending | — | `fused_map_reduce_f64` needs `gpu` feature |
| 2 | Observation gap | **13/13 PASS** | Pending | — | `fused_map_reduce_f64` needs `gpu` feature |
| 3 | Error propagation FAO-56 | **15/15 PASS** | Pending | — | `fao56_et0_batch` **absorbed** — GPU adapter needed |
| 4 | Sequencing noise | **15/15 PASS** | Pending | — | `batched_multinomial` Tier C absorption |
| 5 | Seismic inversion | **9/9 PASS** | Pending | — | Grid search dispatch kernel |
| 6 | Signal specificity | **12/12 PASS** | Pending | — | `GillespieGpu` (exists) |
| 7 | RAWR resampling | **11/11 PASS** | Pending | — | Embarrassingly parallel |
| 8 | Anderson localization | **8/8 PASS** | Pending | — | `spectral::anderson` (exists) |

**CPU tier**: 119/119 PASS (complete)
**GPU tier**: 0/119 (pending ToadStool absorption of Tier A ops and Tier C kernels)
**metalForge tier**: 0/119 (after GPU tier)

### BarraCUDA Integration Status (post ToadStool S62)

| Module | Barracuda CPU | Barracuda GPU | metalForge |
|--------|:------------:|:------------:|:----------:|
| `stats::pearson_r` | **Wired** (`pearson_correlation`) | Pending GPU adapter | — |
| `stats::spearman_r` | **Wired** (`spearman_correlation`) | Pending GPU adapter | — |
| `stats::sample_std_dev` | **Wired** (`correlation::std_dev`) | Pending GPU adapter | — |
| `stats::rmse` | Local (CPU reference) | `NormReduceF64::l2` (exists) | — |
| `stats::mbe` | Local | `SumReduceF64::mean` (exists) | — |
| `stats::r_squared` | Via `pearson_r²` | Via GPU `pearson_r` | — |
| `stats::index_of_agreement` | Local | `FusedMapReduceF64` (exists) | — |
| `stats::hit_rate` | Local | `FusedMapReduceF64` (exists) | — |
| `rarefaction::shannon_diversity` | Local | `FusedMapReduceF64::shannon_entropy` (**ready**) | — |
| `fao56::daily_et0` | Local | `BatchedElementwiseF64::fao56_et0_batch` (**absorbed**) | — |
| `rarefaction::multinomial_sample` | Local | `batched_multinomial` (**Tier C**, production WGSL) | — |
| `seismic::grid_search` | Local | Grid dispatch (**Tier B**) | — |
| `prng::Xorshift64` | Local | `PrngXoshiro` (**Tier B**, needs alignment) | — |

## Evolution Roadmap

- **Phase 0**: Python/NumPy/SciPy baselines — **COMPLETE** (102/102 across 8 experiments)
- **Phase 0+**: Real open data pipelines (NOAA CDO, IRIS waveforms) — pending API tokens
- **Phase 1**: Rust CPU validation — **COMPLETE** (119/119 across 8 binaries)
- **Phase 1b**: metalForge production WGSL — **COMPLETE** (2 shaders, 261 combined lines)
- **Phase 1c**: Paper queue experiments — **COMPLETE** (Exp 006-008: biology, statistics, math)
- **Phase 2a**: Tier A rewire — **3 CPU leaned** (`pearson_r`, `spearman_r`, `sample_std_dev`); 6 GPU ops pending adapter
- **Phase 2b**: Tier B adapt — PRNG alignment, grid-search dispatch
- **Phase 2c**: Tier C absorption — FAO-56 **superseded** (absorbed S49); `batched_multinomial` still needed
- **Phase 3**: Faculty extension kernels (FFT, Lanczos, Gillespie GPU)
- **neuralSpring bridge**: Export noise characterizations as labeled training data

## Code Quality

| Gate | Status |
|------|--------|
| `cargo fmt --check` | PASS |
| `cargo clippy --all-targets` | PASS (0 errors, 0 warnings) |
| `cargo clippy --features barracuda` | PASS |
| `cargo doc --no-deps` | PASS |
| `cargo test` | 108/108 PASS (+ 1 doc test) |
| `cargo test --features barracuda` | 108/108 PASS |
| Validation binaries | 119/119 PASS |
| Library line coverage | 99.7% |
| Unsafe code | Forbidden (workspace lint) |
| Max file size | 397 lines (all < 1000) |
| SPDX headers | All `.rs` files |
| License | AGPL-3.0-or-later |

## Barracuda CPU Delegation (Phase 2a)

| Module | BarraCUDA Target | Status |
|--------|-----------------|--------|
| `bootstrap::bootstrap_mean` | `stats::bootstrap_mean` | **DONE** — `#[cfg(feature = "barracuda")]` |
| `anderson::lyapunov_exponent` | `spectral::anderson::lyapunov_exponent` | **DONE** — `#[cfg(feature = "barracuda-gpu")]` |
| `anderson::lyapunov_averaged` | `spectral::anderson::lyapunov_averaged` | **DONE** — `#[cfg(feature = "barracuda-gpu")]` |
| `gillespie::birth_death_ssa` | `ops::bio::GillespieGpu` | Pending — GPU-only, no CPU fallback |
| `bootstrap::rawr_mean` | New kernel needed | Pending — no RAWR in barracuda |

## Rust vs Python Performance

| Experiment | Python (s) | Rust (s) | Speedup |
|---|---|---|---|
| Exp 006: Signal Specificity | 26.2 | 0.85 | **30.9×** |
| Exp 007: RAWR Resampling | 4.4 | 0.60 | **7.3×** |
| Exp 008: Anderson Localization | 21.4 | 0.72 | **29.8×** |
| **Total** | **52.0** | **2.17** | **24.0×** |

## Handoff Documents

| Handoff | Location | Status |
|---------|----------|--------|
| V1: Initial Barracuda Evolution | `wateringHole/handoffs/archive/GROUNDSPRING_TOADSTOOL_V1_FEB25_2026.md` | Archived |
| V2: Comprehensive Absorption | `wateringHole/handoffs/archive/GROUNDSPRING_TOADSTOOL_V2_FEB25_2026.md` | Archived |
| V3: ToadStool Catch-Up | `wateringHole/handoffs/archive/GROUNDSPRING_TOADSTOOL_V3_FEB25_2026.md` | Archived |
| V4: Phase 2a + Benchmarks | `wateringHole/handoffs/GROUNDSPRING_TOADSTOOL_V4_FEB25_2026.md` | **Current** |

See `metalForge/ABSORPTION_MANIFEST.md` for detailed absorption inventory.
See `specs/PAPER_REVIEW_QUEUE.md` for per-paper three-tier control plan.
