# groundSpring — Control Experiment Status

**Last updated**: February 26, 2026

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
| 009 | Almost-Mathieu Quasiperiodic Localization | Mathematics (quasiperiodic operators) | 8/8 PASS | 8/8 PASS |
| 010 | Bistable Phenotypic Switching | Biological (c-di-GMP) | 10/10 PASS | 9/9 PASS |
| 011 | Multi-Signal QS Integration | Biological (quorum sensing) | 9/9 PASS | 8/8 PASS |
| 012 | Spin Chain Transport | Mathematics (spectral theory) | 18/18 PASS | 18/18 PASS |
| 013 | Resampling Convergence | Statistics (bootstrap) | 8/8 PASS | 8/8 PASS |
| 014 | Drift vs Selection | Biological (population genetics) | 7/7 PASS | 7/7 PASS |

**Python Phase 0**: All 14 experiments passing
**Rust Phase 1**: 177/177 PASS across 14 validation binaries
**Rust tests**: 205/205 PASS (167 unit + 14 proptest + 9 validate-lib + 14 integration + 1 doc)
**pytest**: 37/37 PASS (unit tests, determinism tests, integration tests)
**BarraCUDA delegations**: 26 active (21 CPU + 5 GPU) — ToadStool S66
**Handoff**: V16 (S66 catch-up + rewiring)

**Python checks**: ~129 across 14 experiments. **Rust validation checks**: 177.

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

### Exp 009: Almost-Mathieu Quasiperiodic Localization (8/8 PASS)

**Question**: When does quasiperiodic (almost-periodic) disorder localize waves at all coupling strengths?

**Results**:
- Almost-Mathieu Hamiltonian; barracuda-gpu delegation for `almost_mathieu_hamiltonian`
- Files: control/quasiperiodic/, crates/groundspring/src/anderson.rs (extended), crates/groundspring-validate/src/validate_quasiperiodic.rs

### Exp 010: Bistable Phenotypic Switching (10/10 PASS)

**Question**: When does noise push a bistable system across a phenotypic threshold?

**Results**:
- BistableOde::cpu_derivative barracuda delegation
- Files: control/bistable_switching/, crates/groundspring/src/bistable.rs, crates/groundspring-validate/src/validate_bistable.rs

### Exp 011: Multi-Signal QS Integration (9/9 PASS)

**Question**: How does multi-input signal fusion behave in a noisy quorum-sensing environment?

**Results**:
- MultiSignalOde::cpu_derivative barracuda delegation
- Files: control/multisignal_qs/, crates/groundspring/src/multisignal.rs, crates/groundspring-validate/src/validate_multisignal.rs

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

### validate-quasiperiodic (8/8 PASS)

Ports Exp 009 Almost-Mathieu quasiperiodic localization to pure safe Rust.  Verifies:
- Almost-Mathieu Hamiltonian; barracuda-gpu delegation for `almost_mathieu_hamiltonian`

### validate-bistable (9/9 PASS)

Ports Exp 010 bistable phenotypic switching to pure safe Rust.  Verifies:
- BistableOde::cpu_derivative barracuda delegation

### validate-multisignal (8/8 PASS)

Ports Exp 011 multi-signal QS integration to pure safe Rust.  Verifies:
- MultiSignalOde::cpu_derivative barracuda delegation

### validate-transport (18/18 PASS)

Ports Exp 012 spin chain transport to pure safe Rust.  Verifies:
- Tridiagonal eigenvector solver (implicit QL), wavepacket MSD, transport exponent

### validate-resampling-convergence (8/8 PASS)

Ports Exp 013 resampling convergence to pure safe Rust.  Verifies:
- Bootstrap convergence (Lee & Liu 2024); uses bootstrap module

### validate-drift (7/7 PASS)

Ports Exp 014 drift vs selection to pure safe Rust.  Verifies:
- Wright-Fisher fixation, Kimura fixation probability, neutral diversity trajectory

## Test Infrastructure

| Suite | Tests | Type |
|-------|------:|------|
| `test_common.py` | 18 | Unit tests for shared statistical primitives |
| `test_determinism.py` | 7 | Rerun-identical verification for stochastic ops |
| `test_experiments.py` | 11 | Integration: each experiment returns exit code 0 |
| Rust `#[test]` (lib) | 153 | Unit tests for Rust library modules |
| Rust `#[test]` (validate-lib) | 9 | Unit tests for shared validation helpers |
| Rust proptest | 14 | Property-based invariant tests |
| Rust integration | 14 | Validation binary integration tests |
| Rust doc test | 1 | Documentation example test |
| **Total** | **228** | (37 Python + 191 Rust) |

## Run Log

### Run 16 — February 26, 2026 (V16 ToadStool S66 catch-up + rewiring)

```
ToadStool S66 review: 2,541 tests, 707 WGSL shaders, sovereign compiler.
  V7 was last groundSpring handoff consumed. V13–V15 await ToadStool pickup.
  S66 absorbed rawr_mean from V15 request.

New delegation #26: rawr_mean → barracuda::stats::rawr_mean (CPU)
  Total: 26 active delegations (21 CPU + 5 GPU)

Test fix: bootstrap_different_from_rawr and validate-rawr RAWR comparison
  updated for barracuda parity (compare CI widths instead of exact estimates).

Three-mode revalidation:
  default:       205/205 tests PASS, 177/177 checks PASS
  barracuda:     205/205 tests PASS, 177/177 checks PASS
  barracuda-gpu: 205/205 tests PASS, 177/177 checks PASS
  clippy:        0 warnings × 3 modes

New S66 capabilities documented (not yet wired):
  WrightFisherGpu, eigh_f64, stats::regression, stats::hydrology,
  stats::moving_window_f64, stats::mae
```

### Run 15 — February 26, 2026 (V15 Experiment Buildout: Exp 012–014)

```
3 new experiments built:
  Exp 012: Spin Chain Transport (Kachkovskiy 2016)    18/18 PASS  transport.rs
  Exp 013: Resampling Convergence (Lee & Liu 2024)    8/8  PASS  bootstrap
  Exp 014: Drift vs Selection (R. Anderson 2022)       7/7  PASS  drift.rs

New modules:
  transport   tridiag_eigh, wavepacket_msd, transport_exponent
  drift       wright_fisher_fixation, kimura_fixation_prob, neutral_diversity_trajectory

prng::binomial   Added for Wright-Fisher sampling

Totals:
  14 experiments, 177/177 validation checks
  205 Rust tests (167 unit + 14 proptest + 9 validate-lib + 14 integration + 1 doc)
  14 validation binaries
  Mathematical parity: 14/14 PROVEN (Python ⇌ Rust)

Paper queue: Papers #13, #17, #20 moved Queued → Active
```

### Run 14 — February 26, 2026 (V14 S65 revalidation + cross-spring documentation)

```
New delegation #25: evenness → barracuda::stats::pielou_evenness
  S≤1 semantic adapter (groundSpring returns 1.0, barracuda returns 0.0)
  Total: 25 active delegations (20 CPU + 5 GPU)

Code quality:
  anderson.rs → almost_mathieu.rs split (594 → 264 + 329 lines)
  stats/correlation.rs modernized (CPU always compiled)
  Python: 14 ruff errors fixed (zip(strict=True), unused vars)
  Python linting: zero-warning

Three-mode benchmark (release, single pass):
  Binary                   Local(ms)  Barracuda(ms)  Barra-GPU(ms)
  validate-decompose             82           71            560
  validate-rarefaction            70           99            102
  validate-seismic              141          128            171
  validate-weather                65           71             97
  validate-fao56                  79           80            106
  validate-signal-specificity    854          858            898
  validate-rawr                  619          625            651
  validate-anderson              745          745            774
  validate-quasiperiodic      11986        11867            242
  validate-bistable              167          222            207
  validate-multisignal            85          118            118
  TOTAL                       14893        14884           3926

Three-mode validation:
  190/190 Rust tests PASS × 3 modes
  144/144 validation checks × 3 modes
  0 clippy warnings × 3 modes
  37/37 Python tests PASS

New artifacts:
  whitePaper/CROSS_SPRING_EVOLUTION.md   Cross-spring lineage for all 25 delegations
  scripts/regenerate_benchmarks.sh       Benchmark drift guard
  scripts/three_mode_benchmark.sh        Automated three-mode timing

Handoff V14 posted (V13 archived)
```

### Run 11 — February 26, 2026 (Full-suite parity + benchmarks)

```
Benchmark expansion:
  bench_rust_vs_python.py     3 → 11 experiments (full suite)
  bench_barracuda_modes.sh    8 → 11 binaries (full suite)
  run_all_baselines.sh        8+8 → 11+11 experiments (Python + Rust)

New scripts:
  parity_report.py            Formal Python⇌Rust parity certificate
  data/parity_report.json     Machine-readable parity certificate
  data/bench_rust_vs_python.json  Updated with all 11 experiments

Parity certificate:
  11/11 experiments: PARITY PROVEN
  Python baselines + Rust validation both pass against same benchmark JSONs
  Python checks: ~129    Rust checks: 144/144

Performance (median of 3 trials):
  10/11 experiments: Rust 1.8×–63.6× faster than Python
  1/11 (Exp 009):   custom QR vs LAPACK — parity proven, LAPACK faster
  Total (excl. LAPACK-bound): 23.4× Rust speedup
```

### Run 10 — February 25, 2026 (Exp 009–011: quasiperiodic, bistable, multisignal)

```
Phase 0 (Python):
  Exp 009: Almost-Mathieu Quasiperiodic    8/8  PASS
  Exp 010: Bistable Phenotypic Switching  10/10 PASS
  Exp 011: Multi-Signal QS Integration   9/9  PASS

Phase 1 (Rust):
  validate-quasiperiodic                  8/8  PASS
  validate-bistable                      9/9  PASS
  validate-multisignal                   8/8  PASS

New experiments:
  control/quasiperiodic/                  Almost-Mathieu Hamiltonian
  control/bistable_switching/             BistableOde phenotypic switching
  control/multisignal_qs/                MultiSignalOde QS integration

Barracuda delegations (+3):
  almost_mathieu_hamiltonian              barracuda-gpu (Exp 009)
  BistableOde::cpu_derivative           barracuda (Exp 010)
  MultiSignalOde::cpu_derivative        barracuda (Exp 011)

Totals:
  11 experiments, 144/144 validation checks
  Rust tests: 177 (153 lib + 9 validate-lib + 14 proptest + 11 integration + 1 doc)
  Python checks: ~129
  Barracuda delegations: 14
```

### Run 7 — February 25, 2026 (Deep debt resolution & sovereignty evolution)

```
Phase 1 (Rust) — local mode:
  8/8 binaries, 119/119 PASS

Sovereignty:
  error_propagation_fao56.py    capability-based discovery (no hardcoded primal names)
  test_experiments.py           capability scan for FAO-56 skip check

BarraCUDA error handling:
  All 11 delegations             .expect() / .unwrap_or() → if let Ok + CPU fallback
  CPU fallbacks                  always compiled (no #[cfg(not(feature))] guard)

Shared validation helpers (DRY):
  groundspring-validate lib.rs   f64_field, usize_field, u64_field, f64_range, print_provenance_header
  9 unit tests for validate-lib  (was 0% coverage)

Validation refactoring:
  validate_seismic               SourceTruth + AcceptanceCriteria structs
  validate_fao56                 Uncertainties struct, split run()
  validate_rawr                  validate_gaussian/skewed/correlated/determinism
  validate_signal_specificity    EnzymeNetwork + SimConfig structs, split run()

Dead code removal:
  control/common.py              write_benchmark(), provenance_metadata() removed (unused)

Clippy: 0 warnings
Rust tests: 163/163 PASS (131 unit + 9 validate-lib + 14 proptest + 8 integration + 1 doc)
Python tests: 34/34 PASS
Coverage: 99.11% (cargo-llvm-cov)
```

### Run 9 — February 25, 2026 (Complete rewiring + benchmarks + cross-spring lineage)

```
Complete barracuda API audit:
  All CPU-accessible functions reviewed
  11 delegations confirmed as the complete set
  6 remaining metrics (rmse, mbe, r², IoA, hit_rate, shannon) require WgpuDevice
  No new CPU-only primitives available to wire

Three-mode benchmarks (release, best-of-3):
  Binary                   Local(ms)  BarraCUDA(ms)  BarraCUDA-GPU(ms)
  validate-anderson            671         670             640
  validate-decompose             5           4               5
  validate-fao56                12          12              13
  validate-rarefaction          11          12              12
  validate-rawr                555         560             556
  validate-seismic              56          59              58
  validate-signal-specificity  795         787             787
  validate-weather               3           3               5
  TOTAL                       2108        2107            2076
  Overhead: ~0% (compute-heavy <1%, signal-spec -1%, anderson -5%)

Cross-spring lineage documented:
  hotSpring → precision (df64_core, spectral/anderson, sum_reduce_f64)
  wetSpring → bio-stats (FusedMapReduce, Gillespie, log_f64 fix, ridge)
  neuralSpring → ML/dispatch (spectral_density, domain_ops, xoshiro)

Validation (all three modes):
  163/163 Rust tests PASS × 3 modes
  119/119 validation checks × 3 modes
  0 clippy warnings × 3 modes
  34/34 Python tests PASS

Handoff V9 posted (V8 archived)
```

### Run 8 — February 25, 2026 (ToadStool catch-up revalidation)

```
ToadStool baseline: S50–S62 + DF64 expansion (Feb 23-24, 2026)
  14,200+ tests, 650+ WGSL shaders, shader-first architecture

Review findings:
  No new CPU stats primitives added since our S62 baseline
  Our 11 delegations remain current and complete
  ToadStool has NOT absorbed our shaders (batched_multinomial, mc_et0_propagate)

Code fix:
  correlation.rs  3× needless_return in barracuda cfg blocks → removed

Three-mode validation:
  Local:          163/163 PASS, 0 clippy warnings
  Barracuda:      163/163 PASS, 0 clippy warnings
  Barracuda-GPU:  163/163 PASS, 0 clippy warnings
```

### Run 6 — February 25, 2026 (Complete rewiring + benchmarks)

```
Phase 1 (Rust) — local mode:
  8/8 binaries, 119/119 PASS

Phase 1 (Rust) — barracuda-gpu mode (11 delegated):
  8/8 binaries, 119/119 PASS

New delegations wired (5 new):
  stats::covariance          → barracuda::stats::correlation::covariance
  stats::norm_cdf            → barracuda::stats::norm_cdf
  stats::norm_ppf            → barracuda::stats::norm_ppf
  stats::chi2_statistic      → barracuda::stats::chi2_decomposed
  anderson::analytical_ξ     → barracuda::special::anderson_transport::localization_length

Rust tests: 154/154 PASS (131 unit + 14 proptest + 8 integration + 1 doc)
Clippy: 0 warnings (pedantic + nursery)

Benchmarks (best-of-3, release mode):
  Local total:          2573 ms
  Barracuda-GPU total:  2721 ms (+6%)
  Compute-heavy delta:  <2% overhead (signal-specificity, RAWR, anderson)
```

### Run 5 — February 25, 2026 (ToadStool catch-up revalidation)

```
Phase 1 (Rust) — local mode:
  8/8 binaries, 119/119 PASS

Phase 1 (Rust) — barracuda-gpu mode:
  8/8 binaries, 119/119 PASS (11 delegated functions, all correct)

ToadStool baseline: S62 + DF64 expansion (Feb 24-25, 2026)
  S59: anderson_3d_correlated, sweep_averaged, find_w_c, ridge_regression
  S60-61: cpu-math feature gate, SpMM, TransE
  S62: BandwidthTier, PeakDetectF64
  Post-S62: DF64 core-streaming, ComputeDispatch builder

Verified:
  cargo test --features barracuda-gpu     154/154 PASS (131 unit + 14 proptest + 8 integration + 1 doc)
  cargo clippy --features barracuda-gpu   0 warnings (pedantic + nursery)
  barracuda has bootstrap_mean_f64.wgsl   GPU path available
```

### Run 4 — February 25, 2026 (Code audit & deep debt resolution)

```
Phase 1 (Rust):
  validate-decompose                 36/36 PASS
  validate-rarefaction               15/15 PASS
  validate-seismic                    9/9  PASS
  validate-weather                   13/13 PASS
  validate-fao56                     15/15 PASS
  validate-signal-specificity        12/12 PASS
  validate-rawr                      11/11 PASS
  validate-anderson                   8/8  PASS

Fixes:
  barracuda::spectral::anderson::*  → barracuda::spectral::* (E0603 fix)
  cargo fmt                          6 files reformatted
  cargo clippy (pedantic + nursery)  0 warnings (was 3: too_many_lines × 3)
  bootstrap sort                     partial_cmp().unwrap_or() → f64::total_cmp
  validate_rawr generate_normal      duplicate Box-Muller → library Xorshift64::normal()
  bootstrap percentile_ci            extracted shared helper (DRY)
  validate_anderson main()           extracted disorder_sweep() + thouless_and_localization()
  validate_fao56 main()              extracted validate_monte_carlo() + validate_sensitivity()
  validate_seismic main()            extracted validate_forward_model() + validate_inversion()
  control/common.py                  added provenance_metadata() + write_benchmark()
  phantom bootstrap_mean_f64.wgsl    removed from README + whitePaper
```

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
| 8 | Anderson localization | **8/8 PASS** | Pending | — | `spectral::*` (lyapunov re-exported) |
| 9 | Almost-Mathieu quasiperiodic | **8/8 PASS** | Pending | — | `almost_mathieu_hamiltonian` (barracuda-gpu) |
| 10 | Bistable phenotypic switching | **9/9 PASS** | Pending | — | `BistableOde::cpu_derivative` |
| 11 | Multi-signal QS integration | **8/8 PASS** | Pending | — | `MultiSignalOde::cpu_derivative` |
| 12 | Spin chain transport | **18/18 PASS** | Pending | — | transport module |
| 13 | Resampling convergence | **8/8 PASS** | Pending | — | bootstrap module |
| 14 | Drift vs selection | **7/7 PASS** | Pending | — | drift module |

**CPU tier**: 177/177 PASS (complete)
**GPU tier**: 0/177 (pending ToadStool absorption of Tier A ops and Tier C kernels)
**metalForge tier**: 0/177 (after GPU tier)

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

- **Phase 0**: Python/NumPy/SciPy baselines — **COMPLETE** (129/129 across 14 experiments)
- **Phase 0+**: Real open data pipelines (NOAA CDO, IRIS waveforms) — pending API tokens
- **Phase 1**: Rust CPU validation — **COMPLETE** (177/177 across 14 binaries)
- **Phase 1b**: metalForge production WGSL — **COMPLETE** (2 shaders, 261 combined lines)
- **Phase 1c**: Paper queue experiments — **COMPLETE** (Exp 006-014: biology, statistics, math, quasiperiodic, bistable, multisignal, transport, resampling, drift)
- **Phase 2a**: Tier A rewire — **24 delegated** (19 CPU + 5 GPU; stats, metrics, diversity, bootstrap, anderson, ODE, eigenvalues)
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
| `cargo test` | 205/205 PASS (167 unit + 9 validate-lib + 14 proptest + 14 integration + 1 doc) |
| `cargo test --features barracuda` | 205/205 PASS |
| `cargo test --features barracuda-gpu` | 205/205 PASS |
| Validation binaries (local) | 177/177 PASS |
| Validation binaries (barracuda-gpu) | 177/177 PASS |
| `ruff check control/ tests/` | 0 errors |
| `mypy control/ tests/` | 0 errors |
| `python3 -m pytest tests/` | 34/34 PASS |
| Workspace line coverage | 99.11% (cargo-llvm-cov) |
| Unsafe code | Forbidden (workspace lint) |
| Max file size | 405 lines (all < 1000) |
| SPDX headers | All `.rs` files |
| License | AGPL-3.0-or-later |

## Barracuda CPU Delegation (Phase 2a)

| Module | BarraCUDA Target | Status |
|--------|-----------------|--------|
| `stats::sample_std_dev` | `stats::correlation::std_dev` | **DONE** — `#[cfg(feature = "barracuda")]` |
| `stats::pearson_r` | `stats::pearson_correlation` | **DONE** — `#[cfg(feature = "barracuda")]` |
| `stats::spearman_r` | `stats::correlation::spearman_correlation` | **DONE** — `#[cfg(feature = "barracuda")]` |
| `stats::covariance` | `stats::correlation::covariance` | **DONE** — `#[cfg(feature = "barracuda")]` |
| `stats::norm_cdf` | `stats::norm_cdf` | **DONE** — `#[cfg(feature = "barracuda")]` |
| `stats::norm_ppf` | `stats::norm_ppf` | **DONE** — `#[cfg(feature = "barracuda")]` |
| `stats::chi2_statistic` | `stats::chi2_decomposed` | **DONE** — `#[cfg(feature = "barracuda")]` |
| `bootstrap::bootstrap_mean` | `stats::bootstrap_mean` | **DONE** — `#[cfg(feature = "barracuda")]` |
| `anderson::lyapunov_exponent` | `spectral::lyapunov_exponent` | **DONE** — `#[cfg(feature = "barracuda-gpu")]` |
| `anderson::lyapunov_averaged` | `spectral::lyapunov_averaged` | **DONE** — `#[cfg(feature = "barracuda-gpu")]` |
| `anderson::analytical_localization_length` | `special::anderson_transport::localization_length` | **DONE** — `#[cfg(feature = "barracuda")]` |
| `quasiperiodic::almost_mathieu_hamiltonian` | barracuda-gpu | **DONE** — Exp 009 |
| `bistable::BistableOde::cpu_derivative` | barracuda | **DONE** — Exp 010 |
| `multisignal::MultiSignalOde::cpu_derivative` | barracuda | **DONE** — Exp 011 |
| `gillespie::birth_death_ssa` | `ops::bio::GillespieGpu` | Pending — GPU-only, no CPU fallback |
| `bootstrap::rawr_mean` | New kernel needed | Pending — no RAWR in barracuda |

## Rust vs Python Performance

All 11 experiments, median of 3 trials (Feb 26, 2026):

| Experiment | Python (s) | Rust (s) | Speedup |
|---|---|---|---|
| Exp 001: Sensor Noise | 0.64 | 0.11 | **5.7×** |
| Exp 002: Observation Gap | 0.28 | 0.07 | **4.4×** |
| Exp 003: Error Propagation | 0.36 | 0.10 | **3.8×** |
| Exp 004: Sequencing Noise | 0.14 | 0.08 | **1.8×** |
| Exp 005: Seismic Inversion | 7.63 | 0.12 | **63.6×** |
| Exp 006: Signal Specificity | 26.78 | 0.88 | **30.5×** |
| Exp 007: RAWR Resampling | 4.64 | 0.63 | **7.3×** |
| Exp 008: Anderson Localization | 21.98 | 0.73 | **29.9×** |
| Exp 009: Quasiperiodic | 0.65 | 0.23 * | **2.8×** |
| Exp 010: Bistable Switching | 3.58 | 0.19 | **18.5×** |
| Exp 011: Multi-Signal QS | 4.30 | 0.09 | **46.2×** |
| **Total** | **70.98** | **3.23** | **22.0×** |

\* Exp 009 with barracuda-gpu (Sturm tridiag solver). Without barracuda: 11.7s.

**Mathematical Parity**: 14/14 experiments PROVEN. See `data/parity_report.json`.

## Handoff Documents

| Handoff | Scope | Status |
|---------|-------|--------|
| V16: S66 Catch-Up + Rewiring | rawr_mean delegation #26, V13–V15 consumption audit, 26 delegations (21 CPU + 5 GPU) | **Current** |
| V15: Absorption Request | 2 shaders, 3 semantic fixes, 25 delegations, cross-spring learnings | **Current** |
| V14: S65 Revalidation | 25 delegations, evenness added, 49.5× Exp 009, three-mode benchmark | Archived |
| V13: Complete Rewiring | 24 delegations, Sturm tridiag (50×), cross-spring S58-S65 | Archived |
| V12: S64 Catch-Up | ToadStool S64 absorption, 6 new delegations (20 total), 3 bug fixes | Archived |
| V11: Parity + Benchmarks | Full-suite parity, 11 experiments, 14 delegations, three-tier roadmap | Archived |
| V10: Definitive Handoff | 5 absorption priorities, benchmarks, cross-spring lineage, PRNG roadmap | Archived |
| V9: Complete Rewire + Benchmarks | API audit, zero-overhead benchmarks, cross-spring lineage | Archived |
| V8: Sovereignty + BarraCUDA | Sovereignty evolution, error handling, PRNG/GPU assessment | Archived |
| V7: Deep Audit + Proptest | Deep debt, proptest, Python quality, coverage | Archived |
| V1–V6 | Initial evolution through complete rewiring | Archived (shared wateringHole) |

Active: `wateringHole/handoffs/GROUNDSPRING_TOADSTOOL_V16_S66_CATCHUP_FEB26_2026.md`
Archive: `wateringHole/handoffs/archive/`

See `metalForge/ABSORPTION_MANIFEST.md` for detailed absorption inventory.
See `specs/PAPER_REVIEW_QUEUE.md` for per-paper three-tier control plan.
