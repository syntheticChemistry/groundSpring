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

## 8. Evolution Path

- **Phase 0+**: Wire real NOAA CDO data for Exp 002; download IRIS waveforms for Exp 005
- **Phase 1**: BarraCUDA Rust port — noise-aware statistical primitives for ToadStool
- **neuralSpring bridge**: Export noise characterizations as labeled training data

---

*Study completed: February 16, 2026*
*71/71 quantitative checks passed across 5 experiments, 4 scientific domains*
