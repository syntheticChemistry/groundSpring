# groundSpring — Control Experiment Status

**Last updated**: March 7, 2026 (V98 — 936 tests, 102 delegations, PrecisionRoutingAdvice wired, upstream rewire barraCuda `a898dee`, toadStool S130+, coralReef Iteration 10)

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
| 010 | Bistable Phenotypic Switching | Biological (c-di-GMP) | 10/10 PASS | 10/10 PASS |
| 011 | Multi-Signal QS Integration | Biological (quorum sensing) | 9/9 PASS | 9/9 PASS |
| 012 | Spin Chain Transport | Mathematics (spectral theory) | 18/18 PASS | 18/18 PASS |
| 013 | Resampling Convergence | Statistics (bootstrap) | 10/10 PASS | 8/8 PASS |
| 014 | Drift vs Selection | Biological (population genetics) | 7/7 PASS | 7/7 PASS |
| 015 | Uncertainty Bridge | Cross-domain (sensor→Anderson→QS) | 8/8 PASS | 8/8 PASS |
| 016 | Rare Biosphere Signal Detection | Biological (microbial ecology) | 11/11 PASS | 12/12 PASS |
| 017 | Eco-Evolutionary Noise Threshold | Evolutionary dynamics (quasispecies) | 9/9 PASS | 6/6 PASS |
| 018 | Band Edge Structure | Mathematical physics (spectral theory) | 8/8 PASS | 10/10 PASS |
| 019 | Jackknife Error Estimation | Inverse Problems & Spectral Reconstruction | 9/9 PASS | 9/9 PASS |
| 020 | Freeze-Out Inverse Problem | Inverse Problems & Spectral Reconstruction | 8/8 PASS | 8/8 PASS |
| 021 | Spectral Function Reconstruction | Inverse Problems & Spectral Reconstruction | 8/8 PASS | 8/8 PASS |
| 022 | ET₀ → Anderson Propagation | Cross-spring (FAO-56 + Anderson) | 7/7 PASS | 7/7 PASS |
| 023 | No-Till vs Tilled Sampling | Cross-spring (microbiome + soil) | 7/7 PASS | 7/7 PASS |
| 024 | Aggregate Stability Noise | Cross-spring (soil physics) | 8/8 PASS | 8/8 PASS |
| 025 | f32 vs f64 Precision Drift | WDM MD | 7/7 PASS | 7/7 PASS |
| 026 | System-size Convergence | WDM MD | 7/7 PASS | 7/7 PASS |
| 027 | GPU Vendor Parity | WDM MD | 7/7 PASS | 7/7 PASS |
| 028 | NPU Anderson Regime Classification | Hardware (NPU) | 7/7 PASS | 9/9 PASS |
| 029 | Real GHCND ET₀ Validation | Cross-spring (NOAA) | — | 6/6 PASS |
| 030 | Real NCBI 16S Rare Biosphere | Biological (NCBI) | — | 9/9 PASS |
| 031 | NUCLEUS Stack Validation | Infrastructure | — | 28/28 PASS |
| 032 | IRIS Seismic via NUCLEUS | Geological (IRIS) | — | 12/12 PASS |
| 033 | Cytokine Anderson Lattice | Immunological (tissue geometry) | — | 29/29 PASS |
| 034 | Geometry-Aware Drug Scoring | Immunological (drug repurposing) | — | (combined with 033) |
| 035 | Multi-Method ET₀ Cross-Validation | Hydrology (ET₀ methods) | 15/15 PASS | 19/19 PASS |

**Python Phase 0**: All 29 experiments passing (390 pass + 2 skip)
**Rust Phase 1 (core)**: 340/340 PASS across 30 validation binaries (340 core)
**Rust Phase 1 (NUCLEUS)**: 55/55 PASS across 4 validation binaries (Exp 029–032, `--features biomeos`)
**Total validation**: 395/395 PASS across 34 validation binaries
**Rust tests**: 925/925 PASS (default workspace)
**pytest**: 390/390 PASS + 2 skipped
**Three-tier parity**: 102+ tests — CPU vs barracuda-CPU vs barracuda-GPU proven
**BarraCUDA dispatch**: 102 active (61 CPU + 41 GPU) — V98: barraCuda `a898dee`, toadStool S130+, coralReef Iteration 10. `PrecisionRoutingAdvice` wired into 11 GPU dispatch paths via `get_device_f64_safe()`. tarpc 0.37, ops GPU-gated rewire, domain-esn feature, Rust 2024 unsafe model. coralReef sovereign dispatch path proven on Titan V
**NUCLEUS**: biomeOS Neural API live — Tower, Node, Squirrel validated; NestGate data pipelines (NCBI, NOAA, IRIS); compute.execute + compute.submit validated
**metalForge workloads**: 30 (24 GPU + 2 NPU + 2 CPU-only), 187 checks (130 forge + 57 mixed-hardware)
**metalForge mixed-hardware**: `PCIe` topology, pipeline dispatch, NUCLEUS atomics, fallback chains
**metalForge GPU routing**: f64 workloads → Titan V (Volta, 1:2 native f64), f32/quant → RTX 4070 / AKD1000
**Paper 12**: `tissue_anderson` module — 18 unit tests + 29/29 validation checks + 4D Anderson + Wegner RG (V68)
**Handoff**: V96 (upstream rewire + PrecisionRoutingAdvice — barraCuda `2a6c072`, toadStool S130, coralReef Iteration 7, 11 GPU paths precision-routed)

**Python checks**: ~160 across 28 experiments. **Rust validation checks**: 395 (340 core + 55 NUCLEUS). **metalForge + pipeline checks**: 187.

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

### Exp 016: Rare Biosphere Signal Detection (11/11 PASS)

**Question**: At what sequencing depth can we reliably distinguish rare biological lineages from sequencing artifacts?

**Results**:
- Chao1 richness estimator corrects undersampling at low depth (47.4 vs S_obs 28.7 at D=100)
- Detection threshold for rarest lineages: D* ≈ 998 reads (95% power)
- Dominant species: near-certain detection at D=100
- Very rare (p=0.003): only 26% detected at D=100, near-certain at D=5,000
- Abundance-occupancy correlation: ρ = 0.965

### Exp 017: Eco-Evolutionary Noise Threshold (9/9 PASS)

**Question**: Does Eigen's error threshold predict the critical mutation rate above which genetic information collapses?

**Results**:
- Error threshold μ_c = 0.02276 matches analytical prediction
- Below threshold (μ=0.010): master frequency x_m ≈ 0.42
- Above threshold (μ=0.040): information collapse (x_m ≈ 0)
- Master frequency decays monotonically across mutation rate sweep

### Exp 018: Band Edge Structure (8/8 PASS)

**Question**: Can the transfer matrix method reproduce band-gap structure of 1D tight-binding chains?

**Results**:
- Free lattice: single band [−2.0, 2.0] matching 2t cos(k)
- Period-2 potential: gap of width 2.0 centered at E=0
- Period-3 potential: exactly 3 bands per zone
- >95% finite-system eigenvalues fall within transfer-matrix band regions

### Exp 019: Jackknife Error Estimation (9/9 PASS)

**Question**: Does jackknife resampling provide reliable variance and bias estimates for lattice QCD observables?

**Results**:
- Bazavov 2025 Phys Rev D 111, 094508
- Jackknife variance matches analytical expectations
- Bias correction validated against known estimators
- Leave-one-out resampling determinism

### Exp 020: Freeze-Out Inverse Problem (8/8 PASS)

**Question**: Can we infer freeze-out temperature from hadron yield ratios?

**Results**:
- Bazavov 2016 Phys Rev D 93, 014512
- Freeze-out temperature inversion from hadron yields
- Hadron yield fitting validated against benchmark

### Exp 021: Spectral Function Reconstruction (8/8 PASS)

**Question**: Can we reconstruct spectral functions from Euclidean correlators?

**Results**:
- Bazavov 2025 arXiv 2501.12259
- Spectral reconstruction from correlators validated
- Inverse problem stability checks

### Exp 022: ET₀ → Anderson Propagation (7/7 PASS)

**Question**: How much does humidity-dominated ET₀ error affect localization length predictions?

**Results**: ET₀ CV 0.043 propagates to ξ CV 0.040 (ratio 0.94); humidity dominates at 51%.

### Exp 023: No-Till vs Tilled 16S Sampling (7/7 PASS)

**Question**: Does saturation depth differ between soil management regimes?

**Results**: No-till H'=3.88, Tilled H'=1.57; both saturate at ~500 reads; distinguishable at 1000.

### Exp 024: Aggregate Stability Measurement Noise (8/8 PASS)

**Question**: How precisely must WSA be measured to distinguish Anderson regimes?

**Results**: Noise floor (0.12-0.14) well below regime gap (1.0); regimes distinguishable.

### Exp 025: f32 vs f64 Precision Drift (7/7 PASS)

**Question**: Does f32 accumulation introduce systematic bias in Green-Kubo transport coefficient calculations?

**Results**: f32 introduces measurable systematic bias (~28% of total error); absolute errors scale with integral magnitude.

### Exp 026: System-size Convergence (7/7 PASS)

**Question**: At what system size N does consumer GPU transport converge to the thermodynamic limit?

**Results**: Finite-size correction fits with R² > 0.999; extrapolation within 1% of true D∞.

### Exp 027: GPU Vendor Parity (7/7 PASS)

**Question**: Do GPU vendor/driver differences affect transport coefficient results?

**Results**: Vendor differences at 1e-12 relative level; correlation 1.000000; chi²/DOF ≈ 0.

### Exp 028: NPU Anderson Regime Classification (9/9 PASS)

**Question**: Can int8 quantized Anderson regime classification run on live BrainChip AKD1000 NPU?

**Results**: int8 DMA round-trip at ~51µs/inference; CPU/NPU parity on 10 disorder values; 9/9 Rust checks, 7/7 Python checks.

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

### validate-bistable (10/10 PASS)

Ports Exp 010 bistable phenotypic switching to pure safe Rust.  Verifies:
- BistableOde::cpu_derivative barracuda delegation

### validate-multisignal (9/9 PASS)

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

### validate-uncertainty-bridge (8/8 PASS)

Ports Exp 015 uncertainty bridge to pure safe Rust.  Verifies:
- Sensor noise → disorder mapping → Lyapunov exponent → localization length ξ
- CV(ξ) ranking preserved (EC5 > CS616); bias correction effect at typical θ

### validate-rare-biosphere (12/12 PASS)

Ports Exp 016 rare biosphere detection to pure safe Rust. Verifies:
- Chao1 accuracy at high and low depth
- Detection power and threshold for rare taxa
- Abundance-occupancy correlation
- Singleton fraction behavior
- Determinism

### validate-quasispecies (6/6 PASS)

Ports Exp 017 quasispecies dynamics to pure safe Rust. Verifies:
- Error threshold in expected analytical range
- Master genotype survival below threshold
- Information collapse above threshold
- Mean fitness drop at threshold
- Monotonic master frequency decay
- Determinism

### validate-band-edge (10/10 PASS)

Ports Exp 018 band edge structure to pure safe Rust. Verifies:
- Free lattice band edges (±2t)
- Period-2 and period-3 band counts and gap widths
- Gap width proportionality with potential contrast
- Finite-system eigenvalue band fraction (≥95%)
- Determinism

### validate-jackknife (9/9 PASS)

Ports Exp 019 jackknife error estimation to pure safe Rust. Verifies:
- Jackknife variance accuracy
- Bias correction
- Leave-one-out resampling determinism

### validate-freeze-out (8/8 PASS)

Ports Exp 020 freeze-out inverse problem to pure safe Rust. Verifies:
- Freeze-out temperature inversion from hadron yields
- Hadron yield fitting against benchmark

### validate-spectral-recon (8/8 PASS)

Ports Exp 021 spectral function reconstruction to pure safe Rust. Verifies:
- Spectral reconstruction from Euclidean correlators
- Inverse problem stability

### validate-et0-anderson (7/7 PASS)

Ports Exp 022 FAO-56→Anderson propagation chain. Verifies ET₀ range, CV propagation, humidity dominance, Anderson propagation ratio.

### validate-notill-sampling (7/7 PASS)

Ports Exp 023 no-till vs tilled rarefaction. Verifies diversity ordering, Chao1, community distinguishability, saturation depths.

### validate-aggregate-stability (8/8 PASS)

Ports Exp 024 aggregate stability noise decomposition. Verifies d_eff ranges, bias-variance decomposition, regime discrimination, noise floor.

### validate-precision-drift (7/7 PASS)

Ports Exp 025 f32 vs f64 precision drift. Verifies f64 analytical match, f32 relative error bounds, bias fraction, error-magnitude correlation.

### validate-size-convergence (7/7 PASS)

Ports Exp 026 system-size convergence. Verifies D∞ extrapolation, fitted α, R², convergence at N_max.

### validate-vendor-parity (7/7 PASS)

Ports Exp 027 GPU vendor parity. Verifies max/mean relative difference, correlation, bias fraction, chi-squared per DOF.

### validate-npu-anderson (9/9 PASS)

Ports Exp 028 NPU Anderson regime classification. Verifies int8 quantized classification on AKD1000, CPU/NPU parity, DMA round-trip.

## Test Infrastructure

| Suite | Tests | Type |
|-------|------:|------|
| `test_common.py` | 18 | Unit tests for shared statistical primitives |
| `test_determinism.py` | 7 | Rerun-identical verification for stochastic ops |
| `test_experiments.py` | 27 | Integration: each experiment returns exit code 0 |
| `test_baseline_integrity.py` | 196 | Provenance metadata validation |
| `test_three_tier_parity.py` | 72 | Three-tier parity verification |
| Rust `#[test]` (lib) | 272 | Unit tests for Rust library modules |
| Rust `#[test]` (validate-lib) | 12 | Unit tests for shared validation helpers |
| Rust proptest | 14 | Property-based invariant tests |
| Rust determinism | 13 | Bitwise-identical rerun verification |
| Rust three-tier parity | 100 | CPU/GPU/barracuda parity + pure GPU validation + CPU/GPU stats parity (100% delegation coverage) |
| Rust integration | 33 | Validation binary integration tests |
| Rust doc test | 2 | Documentation example test |
| Rust forge | 49 | groundspring-forge crate tests (incl. 14 V35 arch-aware routing) |
| Rust biomeos | 32 | biomeOS client + integration tests (feature-gated) |
| **Total Rust (default)** | **925** | |
| **Total Rust (barracuda-gpu)** | **814** | |
| **Total Python** | **375** | (+3 skipped) |
| **Grand Total** | **1,182** | |


## Run Log

See [CONTROL_RUN_LOG.md](CONTROL_RUN_LOG.md) for the complete historical run log.

**Latest**: Run 39 (V69 Experiment Buildout + GPU Parity + Mixed-Hardware Pipeline, Mar 2, 2026) — 783 Rust tests, 375 Python tests, 376/376 validation checks + 57/57 mixed-hardware checks, all PASS. 76 barracuda delegations (44 CPU + 32 GPU). GPU→NPU→CPU pipeline dispatch validated. NUCLEUS atomic coordination validated.

### Run 37 (V51 GPU Stats Dispatch + CPU/GPU Parity Proof, Feb 28, 2026)

GPU tier buildout: wired 5 core stats functions for GPU dispatch via barracuda reduce ops, added 9 explicit CPU vs GPU parity tests, validated metalForge routing, and upgraded benchmark provenance.

**GPU stats dispatch**: `stats::mean` → `SumReduceF64::mean`, `stats::std_dev` → `VarianceReduceF64::population_std`, `stats::rmse` → `FusedMapReduceF64::sum_of_squares`, `stats::mbe` → `SumReduceF64::mean` on residuals, `stats::pearson_r` → `CorrelationF64::correlation`. All use Pattern B (GPU optional with CPU fallback).

**CPU vs GPU parity proof** (9 new tests): Verify GPU-dispatched statistics match known analytical values — mean(5.0), std_dev(2.0), RMSE(0.1), MBE(0.5), Pearson(1.0/-/0.0), R²(1.0), Pythagorean decomposition identity, and determinism across tiers.

**metalForge validation**: All 19 workloads route correctly — 17 GPU + 2 NPU. No routing gaps.

**Provenance upgrade**: Seismic and observation gap benchmarks updated with NestGate capability routing for Phase 0+ real data download.

**Dispatch targets**: 61 active (37 CPU + 20 GPU + 4 cross-spring), 1 evolution candidate. V53: GPU grid adapters + 3 new CPU delegations.

- **Rust tests**: 569 (all 3 modes) — PASS
- **Python tests**: 375 (+3 skip) — PASS
- **Quality gates**: `cargo fmt`, `clippy` (pedantic+nursery), `rustdoc` (-D warnings) — all PASS
- **GPU stats parity**: 9/9 PASS (mean, std, rmse, mbe, pearson, R², decompose identity, determinism)

### Run 36 (V50 GPU Dispatch Wiring + Pure GPU Validation, Feb 28, 2026)

GPU dispatch buildout: wired 3 new batch GPU APIs, added pure GPU workload validation tests, and built CPU vs GPU timing benchmark infrastructure.

**New GPU batch dispatch APIs**:
- `gillespie::birth_death_ssa_batch` — dispatches N independent SSA trajectories to `GillespieGpu` (barracuda `ops::bio`), mapping birth-death to a 2-reaction 1-species network. Pattern B (GPU optional, CPU fallback).
- `drift::wright_fisher_fixation_batch` — dispatches N independent fixation trials to `WrightFisherGpu` (barracuda `ops::bio`), managing ping-pong frequency buffers, PRNG state, and readback for fixation classification. Full wgpu buffer API integration.
- `fao56::daily_et0_batch` — dispatches N station-day ET₀ computations to `BatchedElementwiseF64::fao56_et0_batch` (barracuda `ops`), converting sunshine hours to solar radiation on host before GPU dispatch.

**Dependencies**: Added `wgpu` v22 and `bytemuck` v1 as optional deps behind `barracuda-gpu` feature for buffer management in WrightFisherGpu integration. Added `barracuda-gpu` feature forwarding to `groundspring-validate`.

**Public API**: Added `groundspring::gpu_available()` for runtime GPU status without exposing the internal `gpu` module.

**CPU vs GPU benchmark**: `bench-cpu-vs-gpu` binary times 6 workloads (Gillespie, Wright-Fisher, FAO-56, rare biosphere, Anderson, neutral diversity) in both sequential-CPU and batch/GPU modes. GPU produces correct results matching CPU baselines; small-batch overhead expected — GPU shines at production scale via ToadStool streaming.

**Pure GPU validation tests** (6 new): Verify GPU-dispatched results match known scientific values directly — steady-state convergence (Gillespie), Kimura agreement (WF), FAO-56 Example 18 (ET₀), Anderson localization (positive γ), rare biosphere dominance, and batch determinism.

**Dispatch targets**: 43 active (31 CPU + 12 GPU), 4 pending ToadStool. Three new GPU targets wired this run.

- **Rust tests**: 560 (all 3 modes) — PASS
- **Python tests**: 375 (+3 skip) — PASS
- **Quality gates**: `cargo fmt`, `clippy` (pedantic+nursery), `rustdoc` (-D warnings) — all PASS
- **GPU workload validation**: 6/6 PASS (math portable to GPU)

### Run 35 (V49 Deep Debt: Capability Routing + Magic Number Evolution, Feb 28, 2026)

Comprehensive deep-debt pass addressing hardcoding, primal discovery, idiomatic patterns, and typed error handling.

**Capability-based primal routing**: `biomeos.rs` and `nestgate.rs` evolved from hardcoded `"nestgate"` target names to capability-based routing via `capability_call()`. groundSpring now declares *what capability it needs* (`storage.store`, `data.ncbi_search`, `data.noaa_ghcnd`) and lets biomeOS discover which primal provides it at runtime. Zero hardcoded primal references in production code.

**Named constants (magic number evolution)**:
- FAO-56: 11 named constants (`ANGSTROM_A/B`, `CLEAR_SKY_BASE/ALT_COEFF`, `LW_HUMIDITY_*`, `LW_CLOUD_*`, `TETENS_A/B/C`) replace inline literals with FAO-56 equation references.
- Drift: `NEUTRAL_SELECTION_THRESHOLD` and `KIMURA_DENOM_EPSILON` replace bare `1e-10`/`1e-15`.

**Typed JSON accessors**: Added `str_field`, `array_field`, `f64_vec`, `bool_field` to groundspring-validate lib. Migrated `.as_array().expect()` across 6 validate binaries to use typed helpers.

**Idiomatic Rust**: Spearman rank loop in `stats/correlation.rs` modernized from manual index tracking to position-based tie detection.

- **Rust tests**: 547 (all 3 modes) — PASS
- **Python tests**: 375 (+3 skip) — PASS
- **Quality gates**: `cargo fmt`, `clippy` (pedantic+nursery), `rustdoc` (-D warnings) — all PASS
- **Unsafe code**: 0 (entire codebase)
- **Production mocks**: 0
- **Hardcoded primal refs**: 0 (down from 5)

### Run 34 (V48 Three-Tier Parity Completion, Feb 28, 2026)

Added 29 three-tier parity tests covering every active barracuda delegation
(bootstrap, rawr, moving_window, rarefaction indices, kinetics, metrics,
agreement, correlation, bistable/multisignal ODE derivatives, linear regression).
Total parity tests: 73 (up from 44). All 73 pass identically across default,
`--features barracuda`, and `--features barracuda-gpu`.

- **Rust workspace tests**: 547 (default) / 547 (barracuda) / 547 (barracuda-gpu)
- **Python tests**: 375 passed, 3 skipped
- **Validation checks**: 292/292 PASS
- **Three-tier parity**: 73/73 PASS (100% delegation coverage)
- **Active delegations**: 40 (31 CPU + 9 GPU)
- **Pending ToadStool**: 6 (kimura, jackknife, fao56 scalar, grid_fit_2d, grid_search_3d, band_edges_parallel)
- `cargo fmt --check`: PASS
- `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic -W clippy::nursery`: PASS
- `cargo doc --workspace --no-deps` (RUSTDOCFLAGS="-D warnings"): PASS

## Three-Tier Control Matrix

Each experiment is validated at three hardware tiers:

| Tier | Substrate | Description |
|------|-----------|-------------|
| **CPU** | `cargo test` + validation binary | Rust matches Python baseline |
| **GPU** | `barracuda` feature + GPU adapter | GPU matches CPU within tolerance |
| **metalForge** | Mixed hardware dispatch | Cross-substrate agreement |

### Current Status (V98)

| # | Experiment | CPU | GPU | metalForge | GPU Status |
|---|-----------|:---:|:---:|:----------:|------------|
| 1 | Sensor noise decomposition | **36/36 PASS** | **Wired** | Workload | MAE/NSE/R² via `FusedMapReduceF64` (V66) |
| 2 | Observation gap | **13/13 PASS** | **Wired** | Workload | MAE/NSE/R² via `FusedMapReduceF64` (V66) |
| 3 | Error propagation FAO-56 | **15/15 PASS** | **Wired** | Workload | `McEt0PropagateGpu` + `SeasonalPipelineF64` (V67) |
| 4 | Sequencing noise | **15/15 PASS** | **Wired** | Workload | `BatchedMultinomialGpu` (V67, API fix) |
| 5 | Seismic inversion | **9/9 PASS** | **Wired** | Workload | Grid dispatch + stats GPU (V55) |
| 6 | Signal specificity | **12/12 PASS** | **Wired** | Workload | `GillespieGpu` + `BatchedOdeRK4F64` (V63) |
| 7 | RAWR resampling | **11/11 PASS** | **Wired** | Workload | `JackknifeMeanGpu` (V59) |
| 8 | Anderson localization | **8/8 PASS** | **Wired** | **Parity** | `spectral::lyapunov_*` (barracuda-gpu) |
| 9 | Almost-Mathieu quasiperiodic | **8/8 PASS** | **Wired** | **Parity** | `find_all_eigenvalues` (**47.4×** speedup) |
| 10 | Bistable phenotypic switching | **10/10 PASS** | **Wired** | Workload | `BatchedOdeRK4F64` bistable batch (V66) |
| 11 | Multi-signal QS integration | **9/9 PASS** | **Wired** | Workload | `MultiSignalOde` batch path (V66) |
| 12 | Spin chain transport | **18/18 PASS** | CPU | — | QL stays local (beats dense Jacobi) |
| 13 | Resampling convergence | **8/8 PASS** | CPU | — | `rawr_mean` + `bootstrap` CPU delegation |
| 14 | Drift vs selection | **7/7 PASS** | **Wired** | — | `WrightFisherGpu` fixation (V63) |
| 15 | Uncertainty bridge | **8/8 PASS** | **Wired** | — | Inherits Anderson GPU (Exp 008) |
| 16 | Rare biosphere signal detection | **12/12 PASS** | **Wired** | Workload | `BatchedMultinomialGpu` occupancy (V31) |
| 17 | Eco-evolutionary noise threshold | **6/6 PASS** | **Partial** | — | Eigenvalues GPU, eigenvectors CPU-only |
| 18 | Band edge structure | **10/10 PASS** | **Wired** | Workload | `optimize::brent` band edge (V55) |
| 19 | Jackknife error estimation | **9/9 PASS** | **Wired** | Workload | `JackknifeMeanGpu` (V59) |
| 20 | Freeze-out inverse problem | **8/8 PASS** | **Wired** | Workload | Grid + `lbfgs_numerical` (V53+V68) |
| 21 | Spectral function reconstruction | **8/8 PASS** | **Wired** | Workload | `cholesky_f64` + `tikhonov_solve_gpu` (V67) |
| 22 | ET₀ → Anderson propagation | **7/7 PASS** | **Wired** | — | `McEt0PropagateGpu` + Anderson spectral (V67) |
| 23 | No-till vs tilled sampling | **7/7 PASS** | **Wired** | — | `BatchedMultinomialGpu` + Shannon GPU (V95) |
| 24 | Aggregate stability noise | **8/8 PASS** | **Wired** | — | `rmse`/`mbe`/`mean_and_std_dev` GPU (V95) |
| 25 | f32 vs f64 precision drift | **7/7 PASS** | CPU | — | Analytical math, no GPU path needed |
| 26 | System-size convergence | **7/7 PASS** | CPU | — | Analytical math, no GPU path needed |
| 27 | GPU vendor parity | **7/7 PASS** | CPU | — | Analytical math, no GPU path needed |
| 28 | NPU Anderson regime classification | **9/9 PASS** | — | **9/9 PASS** | NPU (AKD1000 DMA) |
| 29-32 | NUCLEUS sovereign experiments | **55/55 PASS** | — | Sovereign | Real data (NOAA/NCBI/IRIS) |
| 33 | Tissue Anderson 4D + Wegner RG | **29/29 PASS** | **Wired** | Workload | `anderson_4d` + `wegner_block_4d` (V68) |

**CPU tier**: 395/395 PASS (34 binaries, complete)
**GPU tier**: 27 of 34 papers have GPU wiring (79%). 102 delegations (61 CPU + 41 GPU). 30/30 metalForge parity.
**V98**: Exp 023/024 GPU wired (V95). PrecisionRoutingAdvice wired into 11 f64 reduction GPU paths (V96). barraCuda `a898dee`, toadStool S130+, coralReef Iteration 10.
**metalForge tier**: 30 workloads, 187 checks (57 mixed-hardware). Exp 028 NPU 9/9 PASS (AKD1000 DMA). GPU→NPU→CPU pipeline dispatch validated.

### BarraCUDA Integration Status (V98 — toadStool S130+, coralReef Iteration 10)

**102 active delegations** (61 CPU + 41 GPU). V82: `BootstrapMeanGpu` GPU dispatch, `freeze_out` gate fix. V81: fused `correlation_full` GPU. V68: `lbfgs_numerical`, `anderson_4d`, `wegner_block_4d`. All active delegations use `if let Ok` / `#[cfg]` with always-compiled CPU fallback.

| # | Module | BarraCUDA Target | Feature Gate | Status |
|---|--------|-----------------|:------------:|--------|
| 1 | `stats::pearson_r` | `stats::pearson_correlation` | barracuda | **DONE** |
| 2 | `stats::spearman_r` | `stats::correlation::spearman_correlation` | barracuda | **DONE** |
| 3 | `stats::sample_std_dev` | `stats::correlation::std_dev` | barracuda | **DONE** |
| 4 | `stats::covariance` | `stats::correlation::covariance` | barracuda | **DONE** |
| 5 | `stats::norm_cdf` | `stats::norm_cdf` | barracuda | **DONE** |
| 6 | `stats::norm_ppf` | `stats::norm_ppf` | barracuda | **DONE** |
| 7 | `stats::chi2_statistic` | `stats::chi2_decomposed` | barracuda | **DONE** |
| 8 | `stats::rmse` | `stats::metrics::rmse` | barracuda | **DONE** |
| 9 | `stats::mbe` | `stats::metrics::mbe` | barracuda | **DONE** |
| 10 | `stats::r_squared` | `stats::metrics::r_squared` | barracuda | **DONE** |
| 11 | `stats::index_of_agreement` | `stats::metrics::index_of_agreement` | barracuda | **DONE** |
| 12 | `stats::hit_rate` | `stats::metrics::hit_rate` | barracuda | **DONE** |
| 13 | `stats::mean` | `stats::metrics::mean` | barracuda | **DONE** |
| 14 | `stats::percentile` | `stats::metrics::percentile` | barracuda | **DONE** |
| 15 | `bootstrap::bootstrap_mean` | `stats::bootstrap_mean` | barracuda | **DONE** |
| 16 | `bootstrap::rawr_mean` | `stats::rawr_mean` | barracuda | **DONE** (S66) |
| 17 | `rarefaction::shannon_diversity` | `stats::diversity::shannon` | barracuda | **DONE** |
| 18 | `rarefaction::evenness` | `stats::pielou_evenness` | barracuda | **DONE** |
| 19 | `anderson::analytical_localization_length` | `special::anderson_transport::localization_length` | barracuda | **DONE** |
| 20 | `bistable::bistable_derivative` | `numerical::ode_bio::BistableOde::cpu_derivative` | barracuda | **DONE** |
| 21 | `multisignal::multisignal_derivative` | `numerical::ode_bio::MultiSignalOde::cpu_derivative` | barracuda | **DONE** |
| 22 | `kinetics::hill` | `stats::hill` | barracuda | **DONE** (S68) |
| 23 | `anderson::lyapunov_exponent` | `spectral::lyapunov_exponent` | barracuda-gpu | **DONE** |
| 24 | `anderson::lyapunov_averaged` | `spectral::lyapunov_averaged` | barracuda-gpu | **DONE** |
| 25 | `almost_mathieu::hamiltonian` | `spectral::almost_mathieu_hamiltonian` | barracuda-gpu | **DONE** |
| 26 | `almost_mathieu::level_spacing_ratio` | `spectral::level_spacing_ratio` | barracuda-gpu | **DONE** |
| 27 | `almost_mathieu::eigenvalues` | `spectral::find_all_eigenvalues` | barracuda-gpu | **DONE** (49.5×) |
| 28 | `spectral_recon::tikhonov_solve` | `linalg::solve_f64_cpu` | barracuda-gpu | **DONE** |
| 29 | `wdm::finite_size_extrapolate` | `stats::regression::fit_linear` | barracuda | **DONE** |

**Remaining gaps** (2 items):

| Module | BarraCUDA Target | Blocker |
|--------|-----------------|---------|
| `transport::tridiag_eigh` | `linalg::eigh_f64` | GPU eigenvectors not yet in barracuda (eigenvalues only via Sturm) |
| `prng::Xorshift64` | `PrngXoshiro` | Different PRNG, baseline regeneration needed (Tier B alignment) |

**Previously blocked — now resolved**: `GillespieGpu` (V63), `WrightFisherGpu` (V63), `CholeskyF64` (V67), `BatchedMultinomialGpu` (V67 API fix).

## Evolution Roadmap

- **Phase 0**: Python/NumPy/SciPy baselines — **COMPLETE** (375 pytest checks across 28 experiments)
- **Phase 0+**: Real open data pipelines (NOAA CDO, IRIS waveforms) — **COMPLETE** (NUCLEUS Exp 029-032)
- **Phase 1**: Rust CPU validation — **COMPLETE** (395/395 across 34 binaries)
- **Phase 1b**: metalForge production WGSL — **COMPLETE** (2 shaders, 261 combined lines)
- **Phase 1c**: Paper queue experiments — **COMPLETE** (Exp 001-033: all domains)
- **Phase 2a**: Tier A rewire — **COMPLETE** — 102 active delegations (61 CPU + 41 GPU), toadStool S130+. PrecisionRoutingAdvice wired (V98)
- **Phase 2b**: BarraCUDA CPU parity — **PROVEN** — 11.6× faster than Python (excl. LAPACK-bound), 28/28 math parity
- **Phase 2c**: BarraCUDA GPU tier — **PROVEN** — 30/30 three-tier parity, 2.2× total GPU speedup, 47.4× peak (Exp 009)
  - GPU-wired: 25 of 34 papers (76%) — stats, hydrology, spectral, bio ODE, multinomial, optimization, 4D Anderson
  - V66: Stats Tier A (MAE/NSE/R²), bistable batch ODE
  - V67: `McEt0PropagateGpu`, `SeasonalPipelineF64`, `BatchedMultinomialGpu` API fix, Cholesky GPU
  - V68: `lbfgs_numerical`, `anderson_4d` + `wegner_block_4d` (tissue immunology)
  - Remaining gap: eigenvector solver (eigenvalues only via Sturm); PRNG alignment (Tier B)
- **Phase 3**: metalForge cross-substrate dispatch — **VALIDATED** — 30 workloads (24 GPU + 2 NPU + 2 CPU-only + 2 mixed), 187 checks (57 mixed-hardware)
  - Live: Exp 028 NPU Anderson (AKD1000 DMA at ~51µs)
  - `PCIe` topology (6 bandwidth tiers), multi-stage pipeline dispatch (fallback/degrade/skip)
  - NUCLEUS atomic types (Tower/Node/Nest/Full), fallback chains, sovereign degradation
  - Architecture-aware routing: Titan V for f64, RTX 4070 for f32, AKD1000 for int8
- **neuralSpring bridge**: Export noise characterizations as labeled training data

## Code Quality

| Gate | Status |
|------|--------|
| `cargo fmt --check` | PASS |
| `cargo clippy --all-targets` | PASS (0 errors, 0 warnings — pedantic + nursery) |
| `cargo clippy --features barracuda` | PASS (0 warnings) |
| `cargo clippy --features barracuda-gpu` | PASS (0 warnings) |
| `cargo doc --no-deps` | PASS (0 warnings) |
| `cargo test` | 925/925 PASS (default) |
| `cargo test --features biomeos` | ~830 PASS |
| `cargo test --features barracuda` | 925/925 PASS |
| `cargo test --features barracuda-gpu` | 814/814 PASS |
| Validation binaries (local) | 395/395 PASS (34 binaries) |
| Validation binaries (barracuda-gpu) | 395/395 PASS |
| `python3 -m pytest tests/` | 375/375 PASS + 2 skipped (28 experiments + unit/determinism) |
| Library line coverage | 99.37% (cargo-llvm-cov, 100% function coverage) |
| Unsafe code | Forbidden (workspace lint) |
| Max file size | 405 lines (all < 1000) |
| `#[allow]` → `#[expect]` | **Zero `#[allow]` remaining** — all lint suppressions use `#[expect]` with `reason` (warns if suppression becomes unnecessary); migration caught 1 stale suppression in seismic.rs |
| Magic numbers | Extracted to named constants (npu.rs, probe.rs, regression.rs `SINGULARITY_THRESHOLD`) |
| Validation thresholds | All hardcoded thresholds evolved to benchmark JSON with rationale strings |
| SPDX headers | All `.rs` and `.py` files (consistent shebang order in Python) |
| License | AGPL-3.0-only |

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
| `gillespie::birth_death_ssa` | `ops::bio::GillespieGpu` | **DONE** (V63) |
| `bootstrap::rawr_mean` | `barracuda::stats::rawr_mean` | **DONE** (S66) |
| `kinetics::hill` | `barracuda::stats::hill` | **DONE** (S68) |
| `freeze_out::grid_fit_2d` | `ops::grid::grid_fit_2d_f64` + `lbfgs_numerical` | **DONE** (V53+V68) |
| `band_structure::find_band_edges` | `spectral::band_edges_parallel` | **DONE** |
| `seismic::grid_search_inversion` | `ops::grid::grid_search_3d_f64` | **DONE** |
| `quasispecies::quasispecies_simulation` | `ops::bio::wright_fisher_simulate` | **DONE** (V63) |
| `rare_biosphere::abundance_occupancy` | `ops::bio::BatchedMultinomialGpu` | **DONE** (V67 API fix) |
| `rare_biosphere::tier_detection_rate` | `ops::bio::BatchedMultinomialGpu` | **DONE** (V67 API fix) |
| `fao56::mc_et0_propagate` | `McEt0PropagateGpu` | **DONE** (V67) |
| `fao56::seasonal_pipeline` | `SeasonalPipelineF64` | **DONE** (V67) |
| `tissue_anderson::tissue_4d_simulation` | `spectral::anderson::anderson_4d` | **DONE** (V68) |
| `tissue_anderson::tissue_4d_rg_coarsen` | `spectral::anderson::wegner_block_4d` | **DONE** (V68) |

## Four-Stage Validation Progression

The complete validation chain: Python (interpreted) → Rust (compiled) → barracuda-CPU (delegated) → barracuda-GPU (portable) → metalForge (cross-system).
Each stage produces **identical mathematical results** — proving the math is correct, portable, and increasingly fast.
`ToadStool` unidirectional streaming massively reduces dispatch round-trips: CPU uploads once, GPU computes full pipeline, result downloads once.

### Stage 1: Python → Rust (pure math, 5.2× faster)

All 28 Python-paired experiments, median of 3 trials (Feb 27, 2026). See `data/bench_rust_vs_python.json`.

| Metric | Value |
|--------|-------|
| Total Python | 107.14s |
| Total Rust | 20.49s |
| Overall speedup | **5.2×** |
| Speedup excl. LAPACK-bound | **11.7×** |
| Best: Exp 005 Seismic | **53.0×** |
| Best: Exp 011 Multi-Signal QS | **44.7×** |

**Mathematical Parity**: 28/28 experiments PROVEN. See `data/parity_report.json`.

### Stage 2: Rust → barracuda-CPU (pure math delegation, parity proven)

barracuda CPU delegation adds ~3.6% overhead from call indirection.
The key insight: **delegated code produces bitwise-identical results**.

| Metric | Default | Barracuda CPU | Delta |
|--------|---------|--------------|-------|
| Total (27 exps) | 22,030ms | 22,828ms | +3.6% |
| Checks | 279/279 | 279/279 | 0 mismatches |

### Stage 3: barracuda-CPU → barracuda-GPU (portable math, 2.2× faster)

toadStool S94b universal precision (DF64 on FP32 cores via `naga`-guided
`df64_rewrite.rs`) with **complete DF64 transcendental suite** (15 functions:
gamma, erf, inverse trig, hyperbolics) allows GPU dispatch with f64-equivalent
precision across all mathematical domains.
Unidirectional streaming reduces dispatch round-trips.

| Metric | Default | Barracuda GPU | Speedup |
|--------|---------|--------------|---------|
| Total (27 exps) | 22,030ms | 9,798ms | **2.2×** |
| Exp 009 Quasiperiodic | 11,376ms | 240ms | **47.4×** |
| Exp 019 Jackknife | 410ms | 100ms | **4.1×** |
| Exp 020 Freeze-Out | 219ms | 127ms | **1.7×** |
| Exp 026 Size Convergence | 176ms | 111ms | **1.6×** |
| Checks | 279/279 | 279/279 | 0 mismatches |

**Three-Tier Parity**: 30/30 experiments PROVEN. See `data/three_tier_parity_report.json`.

### Stage 4: metalForge Cross-System (CPU ↔ GPU ↔ NPU) — V84 VALIDATED

30 metalForge workloads route to optimal substrate per operation.
187 total checks (130 forge + 57 mixed-hardware).

| Substrate | Workloads | Routing | Status |
|-----------|-----------|---------|--------|
| GPU (F64 + Shader) | Anderson, Mathieu, Green-Kubo, freeze-out, seismic, band-edge, quasispecies, rare biosphere, Gillespie, spectral recon, jackknife, MC ET₀, Wright-Fisher, bootstrap, bistable ODE, seasonal pipeline, L-BFGS, tissue 4D Anderson, Wegner RG, Cholesky | 24/30 → Titan V (NVK GV100) | **VALIDATED** |
| CPU (F64 only) | Bias-variance decompose, finite-size extrapolation | Fallback available | **VALIDATED** |
| NPU (int8 quantized) | Anderson regime classify, diversity saturation predict | AKD1000 DMA | **VALIDATED** (Exp 028) |
| Mixed | Pipeline dispatch, PCIe topology | Cross-substrate routing | **VALIDATED** |

**V68 tier checks**: 30/30 workload parity, 187 total checks (57 mixed-hardware: GPU→NPU bypass, NUCLEUS coordination).

### Complete Progression (34 experiments)

```
Python (interpreted)     107.1s   ─── math correctness (open data + open systems)
  │  5.2× faster
Rust (compiled)           20.5s   ─── pure safe Rust, same math (28/28 parity)
  │  ~0% overhead                     395/395 validation checks (34 experiments)
barracuda-CPU             22.8s   ─── delegation proves portability (30/30 parity)
  │  2.2× faster                     102 delegations (61 CPU + 41 GPU)
barracuda-GPU              9.8s   ─── GPU proves the math is truly portable
  │                                    47.4× peak (hotSpring Sturm eigensolver)
  │                                    ToadStool unidirectional streaming
metalForge                        ─── cross-system: GPU → NPU → CPU per-workload
                                       30 workloads, 187 checks, sovereign fallback
                                       PCIe GPU→NPU bypass (no CPU round-trip)
```

## Handoff Documents

| Handoff | Scope | Status |
|---------|-------|--------|
| V74: Deep Debt + ToadStool Catch-Up | 81 delegations (47 CPU + 34 GPU), clippy pedantic CI, 790 tests, barraCuda v0.3.1, toadStool S93 | Archived |
| V79: Exp 035 + seismic delegation | 85 delegations (51 CPU + 34 GPU), 807 tests, barraCuda v0.3.3, toadStool S94b | Archived |
| V80: Fused Ops + BarraCuda Catch-Up | 87 delegations (51 CPU + 36 GPU), 812 tests, barraCuda v0.3.3+, toadStool S94b | Archived |
| V81: Modern Rewire + coralReef | 88 delegations (51 CPU + 37 GPU), 812+ tests, barraCuda `0bd401f`, toadStool S94b, coralReef (390 tests), 27/27 cross-spring | Archived |
| V98: Upstream Rewire (barraCuda a898dee, toadStool S130+, coralReef Iter 10) | 102 delegations (61 CPU + 41 GPU), 936 tests, zero API breakage, three-tier parity intact, coralReef f64 sovereign path unlocked | **Current** |
| V97: GPU Smoke Test + Three-Tier Parity | 102 delegations (61 CPU + 41 GPU), 936 tests, runtime f64 reduction smoke test, three-tier parity 29/29 at all tiers, 382 Python provenance tests | Archived |
| V96: Upstream Rewire + Precision Routing | 102 delegations (61 CPU + 41 GPU), 925 tests, barraCuda `2a6c072`, toadStool S130, coralReef Iteration 7, PrecisionRoutingAdvice wired, shader.compile.* IPC alignment, doc sync | Archived |
| V95: coralReef Breakthrough | coralReef Phase 11, sovereign GPU dispatch on Titan V, push buffer encoding fixed, V95 handoff, doc sync | Superseded |
| V94: Ecosystem Sync + Shannon Delegation | 102 delegations (61 CPU + 41 GPU), 907 tests, tarpc 0.37, ops GPU-gated, domain-esn, Rust 2024 unsafe model, pins updated | Archived |
| V87: Tier B Resolution + Cross-Spring Completion | 93 delegations (56 CPU + 37 GPU), 804+ tests, Tier B fully resolved, multinomial_sample + anderson_potential CPU-delegated, quasispecies + band_structure CPU-by-design, bidirectional provenance documented | Archived |
| V86: DF64 Reduce Wiring + Full Stats Benchmark | 91 delegations, Fp64Strategy wired into SumReduceF64/VarianceReduceF64, 4-tier benchmark (Python/Kokkos/Rust CPU/GPU), GPU reduce returns 0 (pipeline issue) | Archived |
| V85: coralReef Sovereign + Evolution Handoff | 91 delegations, 824 CPU tests + 672 coralReef, coralReef CFG/RA fixes, f64 reduction compiles to native SM70/SM89, toadStool/barraCuda evolution handoff | Archived |
| V84: GPU Validation | 91 delegations, 824 CPU tests, 17/32 GPU tests, RTX 4070 + Titan V probed, DF64 green, f64 reduction broken (naga), CoralCompiler fix | Archived |
| V83: Pin Refresh + Catch-Up | 91 delegations (54 CPU + 37 GPU), 824 tests, barraCuda `0bd401f`, toadStool S96c (`d77fc546`), coralReef `1e048be` (672 tests), all deps verified | Archived |
| V82: Delegation Expansion | 91 delegations (54 CPU + 37 GPU), 824 tests, Thornthwaite ET₀, fit_all, smart refactoring, deep debt audit | Archived |
| V67: S86 Catch-Up | `McEt0PropagateGpu`, `SeasonalPipelineF64`, `BatchedMultinomialGpu` API fix, Cholesky GPU | Archived |
| V55: barracuda Evolution Review + Docs Cleanup | Complete 57-delegation inventory, cross-spring lineage, recommended evolutions, stale refs cleaned. | Archived |
| V54: Full Control Validation + CPU Parity Proof | 283/283 checks, 95/95 parity, Rust 11.6× faster than Python. | Archived |
| V53: Complete Rewiring + GPU Grid Adapters | GPU grid adapters (seismic, freeze-out), 3 new CPU delegations, 57 active (38 CPU + 19 GPU). | Superseded by V54 |
| V52: ToadStool S70+ Catch-Up | 4 new CPU delegations (kimura, jackknife, fao56_et0, chao1), 52 active. | Superseded by V53 |
| V61: Mixed-Hardware Pipeline + NUCLEUS Atomics | PCIe topology, pipeline dispatch, fallback chains, NUCLEUS atomics (Tower/Node/Nest/Full), deep idiomatic debt pass, 668 tests, 42/42 mixed-hardware checks | Archived |
| V60: hotSpring Cross-Spring Absorption | DriftMonitor, ClassificationUncertainty, concept edges, Nautilus optional dep, 620 tests | Archived |
| V59: ToadStool S71+++ Catch-Up | ESN regime classification, Lanczos sparse eigensolver, 2D/3D Anderson, hotSpring S68+ absorption | Archived |
| V58: Cross-Spring Evolution + Deep Debt | Cross-spring WGSL evolution, 13 clippy fixes, provenance headers, 4 NUCLEUS binaries | Archived |
| V56–V57: Gap closure | NestGate, NUCLEUS validation, real data pipelines | Archived |
| V52–V55: GPU evolution + metalForge expansion | GPU parity proof, metalForge remote, tolerance module, 85 metalForge tests | Archived |
| V51: GPU Stats Dispatch + CPU/GPU Parity Proof | GPU stats dispatch, batch GPU APIs, 9 parity tests, bench-cpu-vs-gpu. 48 active. | Archived |
| V47: Library Buildout + BarraCUDA CPU Expansion | 7 new barracuda CPU delegations, 46 active (37 CPU + 9 GPU), 322 lib tests | Superseded by V51 |
| V46: Idiomatic Rust Evolution | `stats::agreement` domain split, `#[allow]` → `#[expect]`, hardcoded thresholds → benchmark JSONs, named constants | Superseded by V47 |
| V45: Validation Gap Closure | +4 checks (292/292): Exp 010 low-noise agreement, Exp 011 dual-signal variance, Exp 016 Spearman occupancy + multinomial determinism. All Python checks now covered in Rust. | Superseded by V46 |
| V44: Deep-Debt Evolution | `linalg` module extraction, `InputError` typed errors, 5 `assert!` → `Result` APIs, capability-based UID discovery, idiomatic casts, enriched derives, absorption guidance for ToadStool | Superseded by V45 |
| V43: Three-Tier Parity + Pure GPU Workloads | 27/27 three-tier parity PROVEN, 39/39 GPU tier checks, 26/26 pure-GPU workload checks, 17/19 metalForge dispatch to Titan V, 462 Rust tests, full certificate in `data/three_tier_parity_report.json` | Superseded by V44 |
| V39: NUCLEUS Integration + NestGate + metalForge Remote | NestGate data pipeline (NCBI/NOAA), metalForge remote substrate discovery, Tower/Node/Nest pipeline graphs, baseCamp sync, 498+ tests | Archived |
| V37: BarraCUDA Evolution | 39 active delegations (30 CPU + 9 GPU, V42 GPU rewiring), 7 pending, NAK f64 gap, absorption priorities, cross-spring learnings | Superseded by V47 |
| V35: Titan V / NAK Adaptive GPU Dispatch | `GpuArch` detection, `NativeF64`, `AdaptiveBatch`, 19 workloads, 49 metalForge tests, 5 substrates, arch-aware routing, NAK f64 gap confirmed, live GPU compute | Superseded by V37/V39 |
| V33: Delegation Count Expansion | 39 active (30 CPU + 9 GPU, V42 GPU rewiring), 7 pending ToadStool | Superseded by V42 |
| V31: GPU Dispatch Wiring + metalForge Expansion | 5 GPU dispatch blocks, 5 metalForge workloads (12 total), 10 GPU parity tests | Superseded by V32 |
| V28: Coverage Evolution + PRNG Readiness | 368 tests + 196 Python integrity, xoshiro128** API parity, CI baseline drift detection, 45 new coverage tests | Superseded by V31 |
| V27: Docs + Handoff Audit | 29 delegations (23 CPU + 6 GPU), paper controls confirmed, three-tier validation, 323 tests, 99.37% coverage | Superseded by V28 |
| V26: MetalForge Live Hardware + Exp 028 | groundspring-forge crate, npu module, validate-metalforge-*; 28 experiments, 288/288 checks, 314 tests, 31 metalForge checks | Superseded by V27 |
| V25: Experiment Buildout Exp 025-027 | precision-drift, size-convergence, vendor-parity; 27 experiments, 279/279 checks, 302 tests | Superseded by V26 |
| V24: Experiment Buildout Exp 022-024 | ET₀→Anderson, notill-sampling, aggregate-stability; 24 experiments, 258/258 checks, 290 tests | Superseded by V25 |
| V23: Experiment Buildout Exp 019-021 | Jackknife, freeze-out, spectral recon; 21 experiments, 236/236 checks, 280 tests | Superseded by V24 |
| V22: Experiment Buildout Exp 016-018 | Rare biosphere, quasispecies threshold, band edge structure; 18 experiments, 211/211 checks, 262 tests | Superseded by V23 |
| V21: Complete Rewiring + Dual-Mode CI | Complete barracuda rewiring, dual-mode CI (clippy/test with and without barracuda), 27 delegations, domain guard fix | Superseded by V22 |
| V20: S68 Catch-Up + Hill | Hill delegation #27 (22 CPU + 5 GPU), ToadStool f0feb226 (S68), 700 shaders, 2,546+ tests | Superseded by V21 |
| V19: Uncertainty Bridge | Exp 015 (8/8 PASS), validate-uncertainty-bridge, 225 tests, 185/185 checks, zero #[allow] | Superseded |
| V17: Deep Debt Evolution | Bug fix, delegation patterns, 9 action items, 3 absorption candidates | Superseded |
| V16: S66 Catch-Up + Rewiring | rawr_mean delegation #26, V13–V15 consumption audit, 26 delegations (21 CPU + 5 GPU) | Superseded |
| V15: Absorption Request | 2 shaders, 3 semantic fixes, 25 delegations, cross-spring learnings | Archived |
| V14: S65 Revalidation | 25 delegations, evenness added, 49.5× Exp 009, three-mode benchmark | Archived |
| V13: Complete Rewiring | 24 delegations, Sturm tridiag (50×), cross-spring S58-S65 | Archived |
| V12: S64 Catch-Up | ToadStool S64 absorption, 6 new delegations (20 total), 3 bug fixes | Archived |
| V11: Parity + Benchmarks | Full-suite parity, 11 experiments, 14 delegations, three-tier roadmap | Archived |
| V10: Definitive Handoff | 5 absorption priorities, benchmarks, cross-spring lineage, PRNG roadmap | Archived |
| V9: Complete Rewire + Benchmarks | API audit, zero-overhead benchmarks, cross-spring lineage | Archived |
| V8: Sovereignty + BarraCUDA | Sovereignty evolution, error handling, PRNG/GPU assessment | Archived |
| V7: Deep Audit + Proptest | Deep debt, proptest, Python quality, coverage | Archived |
| V1–V6 | Initial evolution through complete rewiring | Archived (shared wateringHole) |

Active: `wateringHole/handoffs/GROUNDSPRING_V96_UPSTREAM_REWIRE_HANDOFF_MAR07_2026.md`
Archive: `wateringHole/handoffs/archive/`

See `metalForge/ABSORPTION_MANIFEST.md` for detailed absorption inventory.
See `specs/PAPER_REVIEW_QUEUE.md` for per-paper three-tier control plan.
