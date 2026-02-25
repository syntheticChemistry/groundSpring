# groundSpring Study: The Dirty Differences

## Abstract

groundSpring systematically characterizes the gap between model predictions and real-world measurements across four scientific domains: agricultural sensing, meteorology, microbiome biology, and seismology. Through five Phase 0 experiments (71/71 checks passed), we demonstrate a unified framework for decomposing measurement error into correctable bias and irreducible noise. Key findings include: (1) soil moisture sensor bias accounts for 26-77% of total error depending on sensor/soil combination; (2) humidity sensor accuracy dominates FAO-56 ET0 uncertainty at 66% of total variance; (3) 16S taxonomic assignments stabilize above 5000 reads; and (4) seismic source localization shows ±2km horizontal but ±8.5km depth uncertainty from ±0.5s arrival time noise. These results inform minimum sensor requirements for Penny Irrigation and establish the noise characterization primitives needed for neuralSpring's transfer learning.

## 1. Introduction

Every measurement is a lie — or more precisely, an approximation. The question groundSpring asks is not "what is the true value?" but rather "how much should we trust this number?"

This matters because:
- airSpring's FAO-56 ET0 computation is only as good as the weather sensors feeding it
- wetSpring's microbiome taxonomy is only as reliable as the sequencing depth allows
- Any model applied to a new location (New Mexico pistachios, California almonds) must know how much to distrust its inputs

groundSpring establishes the measurement uncertainty budget that neuralSpring will later use for transfer learning and adaptation.

## 2. Experiment 001: Sensor Noise Characterization

### 2.1 Results

Using Dong et al. (2020) factory calibration data for CS616 and EC5 soil moisture sensors across three Michigan soil types:

| Sensor | Soil Type | Bias (MBE) | Random σ | Bias Fraction | Noise Floor (corrected RMSE) |
|--------|-----------|-----------|----------|---------------|------------------------------|
| CS616 | Sand | -0.010 | 0.014 | 34.6% | 0.006 m³/m³ |
| CS616 | Loamy sand | -0.030 | 0.025 | 59.2% | 0.021 m³/m³ |
| CS616 | Sandy clay loam | -0.020 | 0.034 | 26.3% | 0.012 m³/m³ |
| EC5 | Sand | +0.030 | 0.023 | 62.3% | 0.004 m³/m³ |
| EC5 | Loamy sand | -0.030 | 0.018 | 73.5% | 0.006 m³/m³ |
| EC5 | Sandy clay loam | -0.050 | 0.027 | 77.0% | 0.020 m³/m³ |

### 2.2 Key Insight

**EC5 sensors are bias-dominated** (62-77% of error is systematic), meaning site-specific calibration removes most of the error. **CS616 sensors have mixed noise structure** — sand and clay soils are noise-dominated, while loamy sand is bias-dominated.

**Implication**: For Penny Irrigation, EC5 sensors benefit most from calibration; CS616 sensors in sandy soil need averaging or filtering instead.

## 3. Experiment 002: Weather Model vs Observation

### 3.1 Results

Compared Open-Meteo ERA5 reanalysis against NOAA CDO station data for Lansing, MI (2023). Note: NOAA data was synthetic in this run; real comparison pending CDO token integration.

Methodology validated:
- Bias-variance decomposition of model-observation gap
- Seasonal analysis (DJF/MAM/JJA/SON) showing winter has largest gaps
- Precipitation hit-rate analysis

### 3.2 Key Insight

The model-observation gap is **representation noise dominated** — most of the difference between a 10km grid cell and a point station is not systematic bias but irreducible spatial variability. This means bias correction alone cannot close the gap; the remaining error is the cost of working with gridded reanalysis instead of local stations.

## 4. Experiment 003: Error Propagation Through FAO-56

### 4.1 Results

Monte Carlo propagation (N=10,000) through the FAO-56 Penman-Monteith equation chain:

| Input Variable | Sensor Uncertainty | Variance Fraction | Sensitivity |
|---------------|-------------------|-------------------|-------------|
| Humidity (RH) | ±5% absolute | **65.6%** | Dominant |
| Radiation (Rs) | ±5% relative | 20.1% | Secondary |
| Temperature | ±0.5°C | 10.0% | Moderate |
| Wind speed | ±10% relative | 4.3% | Low |

Overall ET0 uncertainty: **3.879 ± 0.142 mm/day (CV = 3.7%)**

Monte Carlo vs analytical Taylor expansion: ratio = 1.009 (first-order approximation is adequate for this equation chain).

### 4.2 Key Insight

**Humidity is the bottleneck.** Despite temperature being the "headline" weather variable, it's the humidity sensor that controls ET0 accuracy. This is because the vapour pressure deficit (VPD) enters the equation both through the aerodynamic term and through the net longwave radiation term.

**Implication**: A $5 humidity sensor upgrade has more impact on irrigation accuracy than a $50 pyranometer upgrade.

## 5. Experiment 004: Sequencing Depth & Taxonomic Noise

### 5.1 Results

Rarefaction from a 150-genus, 8-phylum synthetic soil community:

| Depth (reads) | Genera Detected | Shannon H' | % of True Shannon |
|---------------|----------------|------------|-------------------|
| 100 | 56 ± 4 | 3.81 ± 0.09 | 86% |
| 500 | 108 ± 4 | 4.25 ± 0.04 | 96% |
| 1,000 | 126 ± 4 | 4.33 ± 0.04 | 98% |
| 5,000 | 146 ± 1 | 4.39 ± 0.01 | 99.7% |
| 10,000 | 148 ± 1 | 4.40 ± 0.01 | 99.8% |
| 100,000 | 150 ± 0 | 4.41 ± 0.00 | 100% |

**Convergence thresholds**:
- All 8 phyla detected: 100 reads
- Shannon within 5% of true: 500 reads
- Genus discovery saturation (<5% new per doubling): 5,000 reads

### 5.2 Key Insight

**Phylum-level taxonomy is robust** — even 100 reads detect all 8 phyla in a 150-genus community. **Genus-level taxonomy** is the bottleneck: below 5,000 reads, you're still missing ~3% of genera per sample. For wetSpring's pond crash detection, the signal (phylum-level shifts) is well above the noise floor, but genus-level changes require careful depth-normalization.

## 6. Experiment 005: Seismic Wave Propagation

### 6.1 Results

Source localization for a synthetic New Madrid Seismic Zone earthquake (M~3, 10km depth), using 7 regional stations:

| Scenario | Horizontal Error | Depth Error | RMS Residual |
|----------|-----------------|-------------|--------------|
| Clean arrivals | 0.00 km | 0.00 km | 0.000 s |
| Noisy (±0.5s) | 0.9 km | 7.7 km | 0.455 s |
| MC mean (50 trials) | 0.4 km | — | 0.303 s |
| MC uncertainty | ±2.1 km (90th: 3.9 km) | ±8.5 km | ±0.126 s |

### 6.2 Key Insight

**Horizontal location is well-constrained; depth is not.** With only surface stations, the depth-origin time tradeoff makes depth poorly determined. This is the seismological analog of sensor noise: the information content of the data constrains some parameters much better than others.

**More stations help, but with diminishing returns**: going from 3 to 5 stations dramatically improves accuracy; going from 5 to 7 adds little.

## 7. Cross-Domain Synthesis

The five experiments share a common structure:

| Concept | Exp 001 | Exp 003 | Exp 004 | Exp 005 |
|---------|---------|---------|---------|---------|
| **Input noise** | Sensor calibration error | Weather measurement error | Sequencing sampling | Arrival time picks |
| **Model** | Topp equation | FAO-56 PM chain | Multinomial sampling | 1D travel times |
| **Output** | VWC estimate | ET0 (mm/day) | Genus assignments | Source location |
| **Bias fraction** | 26-77% | 66% (humidity) | N/A (sampling) | N/A (random picks) |
| **Noise floor** | 0.004-0.021 m³/m³ | ±0.14 mm/day (3.7% CV) | ±0.004 H' at 100k reads | ±2.1 km horizontal |

The framework — decompose error, identify the dominant source, quantify the noise floor — is universal.

## 8. Phase 1: Rust Validation

All five experiments have been ported to idiomatic Rust in the `groundspring` crate.

### 8.1 Coverage

| Metric | Value |
|--------|-------|
| Validation binaries | 5 (decompose, rarefaction, seismic, weather, fao56) |
| Total checks | 88/88 PASS |
| Unit tests | 90 + 1 doc test |
| Line coverage | 99.7% |
| Function coverage | 100% |
| Clippy warnings | 0 |

### 8.2 New Modules

- **`fao56`** — Pure-Rust port of the FAO-56 Penman-Monteith equation chain
  (originally from airSpring Python). 17 sub-functions + `daily_et0` wrapper.
  Validates against FAO Irrigation & Drainage Paper 56 Example 18
  (ET₀ = 3.88 mm/day for Uccle, Belgium).

- **`prng`** — Xorshift64 PRNG with Box-Muller normal sampling. Extracted
  from rarefaction for reuse in Monte Carlo simulations. Will align to
  barracuda's xoshiro128** when GPU feature is enabled.

- **`stats::pearson_r`** — Pearson correlation coefficient. Delegates to
  `barracuda::stats::pearson_correlation` under `#[cfg(feature = "barracuda")]`.

- **`stats::spearman_r`** — Spearman rank correlation coefficient. Delegates to
  `barracuda::stats::correlation::spearman_correlation` under `#[cfg(feature = "barracuda")]`.

- **`stats::sample_std_dev`** — Bessel-corrected (÷ N−1) sample standard
  deviation. Delegates to `barracuda::stats::correlation::std_dev` under
  `#[cfg(feature = "barracuda")]`.

- **`validate`** — Replaced global `AtomicU32` counters with a struct-based
  `ValidationHarness`. Enables independent, concurrent, and nested validation
  scopes without shared state.

### 8.3 Key Improvements

- **Hot-loop optimization**: `seismic::grid_search_inversion` Vec allocations
  hoisted outside triple loop (lat × lon × depth).
- **Modern Rust idioms**: `f64::total_cmp` replaces `partial_cmp().unwrap_or(Equal)`
  (5 sites); `f64::midpoint`, `.hypot()`, `.mul_add()`, `f64::from()` used throughout.
- **Data-driven validation**: All 5 binaries load expected values from benchmark
  JSONs via `include_str!` + `serde_json` — single source of truth.
- **Barracuda feature gate**: `pearson_r` delegates to `barracuda::stats::pearson_correlation`
  under `#[cfg(feature = "barracuda")]`. Builds and tests clean with and without the feature.
- **Provenance**: All benchmark JSONs include DOI/references, `data_origin`,
  `prng_algorithm`, and `real_data_accession` fields.
- **`missing_docs`** promoted from `warn` to `deny`.

### 8.4 GPU Evolution

Two production WGSL shaders in `metalForge/shaders/` following hotSpring
conventions (documented bindings, xoshiro128** PRNG, f64, workgroup_size(64)):

1. **`mc_et0_propagate.wgsl`** (149 lines) — Monte Carlo FAO-56 propagation.
   Equation chain superseded by barracuda `Op::Fao56Et0`; the MC noise wrapper
   (Box-Muller perturbation + dispatch) is the absorption target.

2. **`batched_multinomial.wgsl`** (112 lines) — Batched multinomial sampling.
   Binary search over cumulative probabilities, per-replicate xoshiro state.

See `metalForge/ABSORPTION_MANIFEST.md` for binding layouts, dispatch geometry,
and the full module-by-module absorption inventory.

## 9. Evolution Path

- **Phase 0+**: Wire real NOAA CDO data for Exp 002; download IRIS waveforms for Exp 005
- **Phase 2a**: Tier A rewire — **3 CPU leaned** (`pearson_r`, `spearman_r`, `sample_std_dev`); 6 GPU ops pending adapter. FAO-56 ET₀ absorbed upstream (ToadStool S49).
- **Phase 2b**: Tier B adapt — PRNG alignment, grid-search dispatch
- **Phase 2c**: Tier C absorption — MC and multinomial kernels → barracuda
- **neuralSpring bridge**: Export noise characterizations as labeled training data

## 10. Next Phase: Faculty-Driven Paper Candidates

The faculty network identifies three directions that extend groundSpring's noise-characterization framework into new domains:

1. **Inverse problems at high precision** (Bazavov, CMSE/Physics MSU): Lattice QCD spectral reconstruction demands subpercent precision from noisy data — the same mathematical structure as seismic inversion (Exp 005) but at much higher fidelity. Reproducing Bazavov et al. (arXiv 2501.12259) would extend groundSpring's inverse problem toolkit to include regularized spectral methods and jackknife error estimation.

2. **Biological signal specificity** (Waters, MMG MSU): Massie et al. (2012, PNAS) asks how cells resolve signal from noise when 60+ enzymes control a single diffusible molecule. This is the biological analog of Exp 001's sensor noise decomposition — but inside a living cell. Fernandez et al. (2020, PNAS) extends this to bifurcation analysis: at what noise level does a cell switch phenotype?

3. **Resampling confidence methods** (Liu, CMSE MSU): Wang et al. (2021, ISMB/Bioinformatics) develops RAWR — modern weighted resampling that outperforms naive bootstrap for structured data. groundSpring's Monte Carlo (Exp 003) uses simple random draws; RAWR could improve both efficiency and accuracy of our error propagation framework.

These extensions share the common theme: **how do you extract reliable conclusions from noisy measurements?** Whether the measurements are soil moisture readings, lattice QCD correlators, intracellular c-di-GMP concentrations, or phylogenetic tree topologies, the mathematical framework for confidence estimation is the same.

---

*Phase 0 completed: February 16, 2026 — 71/71 PASS (Python)*
*Phase 1 completed: February 25, 2026 — 88/88 PASS (Rust, 99.7% coverage)*
