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

**Python Phase 0**: All 5 experiments passing
**Rust Phase 1**: 88/88 PASS across 5 validation binaries (decompose, rarefaction, seismic, weather, fao56)
**pytest**: 31/31 PASS (unit tests, determinism tests, integration tests)

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

## Test Infrastructure

| Suite | Tests | Type |
|-------|------:|------|
| `test_common.py` | 18 | Unit tests for shared statistical primitives |
| `test_determinism.py` | 7 | Rerun-identical verification for stochastic ops |
| `test_experiments.py` | 5 | Integration: each experiment returns exit code 0 |
| Rust `#[test]` | 90 | Unit tests for Rust library modules (+ 1 doc test) |
| **Total** | **120** | |

## Run Log

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

**CPU tier**: 88/88 PASS (complete)
**GPU tier**: 0/88 (pending ToadStool absorption of Tier A ops and Tier C kernels)
**metalForge tier**: 0/88 (after GPU tier)

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

- **Phase 0**: Python/NumPy/SciPy baselines — **COMPLETE** (71/71)
- **Phase 0+**: Real open data pipelines (NOAA CDO, IRIS waveforms) — pending API tokens
- **Phase 1**: Rust CPU validation — **COMPLETE** (88/88, 99.7% coverage)
- **Phase 1b**: metalForge production WGSL — **COMPLETE** (2 shaders, 261 combined lines)
- **Phase 2a**: Tier A rewire — **3 CPU leaned** (`pearson_r`, `spearman_r`, `sample_std_dev`); 6 GPU ops pending adapter
- **Phase 2b**: Tier B adapt — PRNG alignment, grid-search dispatch
- **Phase 2c**: Tier C absorption — FAO-56 **superseded** (absorbed S49); `batched_multinomial` still needed
- **Phase 3**: Faculty extension kernels (FFT, Lanczos, Gillespie)
- **neuralSpring bridge**: Export noise characterizations as labeled training data

## Code Quality

| Gate | Status |
|------|--------|
| `cargo fmt --check` | PASS |
| `cargo clippy --all-targets` | PASS (0 errors, 0 warnings) |
| `cargo clippy --features barracuda` | PASS |
| `cargo doc --no-deps` | PASS |
| `cargo test` | 90/90 PASS (+ 1 doc test) |
| `cargo test --features barracuda` | 90/90 PASS |
| Validation binaries | 88/88 PASS |
| Library line coverage | 99.7% |
| Unsafe code | Forbidden (workspace lint) |
| Max file size | 397 lines (all < 1000) |
| SPDX headers | All `.rs` files |
| License | AGPL-3.0-or-later |

## Handoff Documents

| Handoff | Location | Status |
|---------|----------|--------|
| V1: Initial Barracuda Evolution | `wateringHole/handoffs/archive/GROUNDSPRING_TOADSTOOL_V1_FEB25_2026.md` | Archived |
| V2: Comprehensive Absorption | `wateringHole/handoffs/archive/GROUNDSPRING_TOADSTOOL_V2_FEB25_2026.md` | Archived |
| V3: ToadStool Catch-Up | `wateringHole/handoffs/GROUNDSPRING_TOADSTOOL_V3_FEB25_2026.md` | **Current** |

See `metalForge/ABSORPTION_MANIFEST.md` for detailed absorption inventory.
See `specs/PAPER_REVIEW_QUEUE.md` for per-paper three-tier control plan.
