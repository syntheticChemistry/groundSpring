# groundSpring — Paper Review Queue

**Last Updated**: May 10, 2026 (V129)
**Purpose**: Track papers for reproduction/review, ordered by priority

---

## Completed Reproductions

| # | Experiment | Domain | Phase 0 | Phase 1 (Rust) | Key Finding |
|---|-----------|--------|:-------:|:--------------:|-------------|
| 1 | Sensor noise decomposition | Agricultural sensors | 32/32 | 36/36 | EC5 bias-dominated (77%); CS616 mixed |
| 2 | Observation gap (ERA5 vs station) | Meteorology | PASS | 13/13 | Representation noise dominated |
| 3 | Error propagation FAO-56 | ET₀ uncertainty | PASS | 15/15 | Humidity dominates at 66% |
| 4 | Sequencing depth & taxonomic noise | Microbiome | PASS | 15/15 | Genus saturation at 5,000 reads |
| 5 | Seismic source inversion | Geophysics | PASS | 9/9 | ±2km horizontal, ±8.5km depth |
| 9 | Enzymatic signal specificity | Biology (c-di-GMP) | 12/12 | 12/12 | SNR ≈ 2 at 20× activation; 30.9× faster |
| 12 | RAWR resampling | Statistics | 11/11 | 11/11 | Coverage comparable to bootstrap; 7.3× faster |
| 13 | Resampling convergence | Statistics | 10/10 | 8/8 | Meta-statistical optimization (Lee & Liu 2024) |
| 15 | Anderson localization | Mathematics | 8/8 | 8/8 | All states localized; Thouless C ≈ 104; 29.8× faster |
| 16 | Almost-Mathieu quasiperiodic localization | Mathematics | 8/8 | 8/8 | Aubry-André transition at λ=2; Herman's formula verified; level statistics distinguish phases |
| 17 | Spin chain transport | Mathematics | 18/18 | 18/18 | Energy transport through disordered XY chains (Kachkovskiy 2016) |
| 20 | Drift vs selection | Evolutionary biology | 7/7 | 7/7 | Wright-Fisher fixation, Kimura probability (R. Anderson 2022) |
| 10 | Bistable phenotypic switching | Biology | 10/10 | 10/10 | Two stable attractors (0.035 vs 1.634 c-di-GMP); noise-induced transitions; 18.5× faster |
| 11 | Multi-signal QS integration | Biology | 9/9 | 9/9 | Dual signaling sharpens regulation; lower HapR variance; 46.2× faster |
| 21 | Rare biosphere signal detection | Microbial ecology | 11/11 | 12/12 | Sequencing depth determines rare taxa signal boundary (R. Anderson 2015) |
| 14 | Eco-evolutionary noise threshold | Evolutionary dynamics | 9/9 | 6/6 | Eigen's error threshold predicts mutation-driven information collapse (Dolson 2023) |
| 18 | Band edge structure | Mathematical physics | 8/8 | 10/10 | Transfer matrix reproduces tight-binding band-gap structure (Filonov-Kachkovskiy 2018) |
| 19 | Jackknife error estimation | Statistics/Error Estimation | 9/9 | 9/9 | Subpercent precision error bars (Bazavov 2025 Phys Rev D 111, 094508) |
| 20 | Freeze-out inverse problem | Inverse Problems | 8/8 | 8/8 | Inferring freeze-out conditions from heavy ion data (Bazavov 2016 Phys Rev D 93, 014512) |
| 21 | Spectral function reconstruction | Inverse Problems/Spectral Reconstruction | 8/8 | 8/8 | Signal recovery from incomplete/noisy lattice data (Bazavov 2025 arXiv 2501.12259) |
| 22 | ET₀ → Anderson uncertainty propagation | Cross-spring (FAO-56 × Anderson) | 7/7 | 7/7 | Humidity uncertainty propagates through water balance to localization length; ξ_CV/ET₀_CV ≥ 0.5 |
| 23 | No-till vs tilled 16S sampling design | Cross-spring (Rarefaction × Ecology) | 7/7 | 7/7 | No-till saturates later (~1500 reads) vs tilled (~800); higher diversity demands deeper sampling |
| 24 | Aggregate stability measurement noise | Cross-spring (WSA × Anderson) | 8/8 | 8/8 | Bias-variance decomposition distinguishes tilled vs no-till Anderson regimes under measurement noise |
| 25 | f32 vs f64 precision drift | WDM MD | 7/7 | 7/7 | Bias-variance decomposition of f32→f64 Green-Kubo integration error; bias fraction ~28% |
| 26 | System-size convergence for WDM transport | WDM MD | 7/7 | 7/7 | Finite-size extrapolation D(N) = D∞ + α/N^(1/d); R² > 0.999 |
| 27 | GPU vendor parity for WDM observables | WDM MD | 7/7 | 7/7 | Vendor differences at 1e-12 relative level; correlation 1.000000 |
| 28 | NPU Anderson regime classification | Hardware (NPU) | 7/7 | 9/9 | int8 DMA classification on AKD1000 at ~51µs |
| 29 | Real GHCND ET₀ Validation | Cross-spring (NOAA) | — | 6/6 | Live NOAA GHCND data validates FAO-56 ET₀ chain (NUCLEUS Exp 029) |
| 30 | Real NCBI 16S Rare Biosphere | Biological (NCBI) | — | 9/9 | Live NCBI SRA data validates rare biosphere detection (NUCLEUS Exp 030) |
| 31 | NUCLEUS Stack Validation | Infrastructure | — | 28/28 | Tower+Node+Squirrel validated live (NUCLEUS Exp 031) |
| 32 | IRIS Seismic via NUCLEUS | Geological (IRIS) | — | 12/12 | Live IRIS FDSN data validates seismic chain (NUCLEUS Exp 032) |
| 33 | Cytokine Anderson Lattice (Paper 12) | Immunological | — | 29/29 | Tissue 2D/3D + barrier disruption + dimensional duality (Exp 033) |
| 34 | Multi-Method ET₀ Cross-Validation | Hydrology (ET₀) | 15/15 | 19/19 | 5-method comparison: PM, Hargreaves, Makkink, Turc, Hamon (Exp 035) |

**Phase 0**: ~276 checks (Python, 29 experiments). **Phase 1**: 395/395 PASS (Rust, 35 experiments / 34 binaries). **Speedup**: 11.5× median (excl. LAPACK-bound), 47.7× peak (Sturm tridiag).
**Mathematical Parity**: 29/29 PROVEN — Python and Rust both pass against shared benchmark JSONs (Exp 029–033 have no Python baseline).
**Current (V129)**: 395/395 checks, 1,101 Rust workspace tests, 287 Python tests, 140 metalForge checks.
**Tier 4 IPC-first (V128)**: `barracuda` removed from default features; IPC via `CompositionContext` is the default. `local` feature for opt-in library linkage.
**GPU dispatch**: 16 modules wired for `barracuda-gpu` — 110 delegations (67 CPU + 43 GPU), barraCuda v0.3.13, toadStool S158+. 30 metalForge workloads (24 GPU + 2 NPU + 2 CPU-only + 2 mixed).
**Three-tier parity**: 30/30 PROVEN (default = barracuda-CPU = barracuda-GPU). metalForge: 140 checks.
**Exp 015** bridges Papers 22-24 (Sub-thesis 06): sensor noise → Anderson ξ → QS regime uncertainty.
**Cross-spring shader evolution**: `hotSpring` precision shaders and `wetSpring` bio shaders feed `groundSpring` GPU tier via `ToadStool` unidirectional streaming.

---

## Review Queue

### Inverse Problems & Spectral Reconstruction (Bazavov)

| # | Paper | Journal | Year | Faculty | Why | Status |
|---|-------|---------|------|---------|-----|--------|
| 6 | "Spectral reconstruction inverse problem in lattice QCD" | arXiv 2501.12259 | 2025 | Bazavov et al. | Signal recovery from incomplete/noisy data — direct generalization of Exp 005 seismic inversion, but at subpercent precision | **Active** (Exp 021: 8/8 Py, 8/8 Rust) |
| 7 | "Hadronic vacuum polarization for the muon g-2" | Phys Rev D 111, 094508 | 2025 | Bazavov et al. | Jackknife/bootstrap error estimation at subpercent precision. Exp 003 MC propagation is a simplified version of this | **Active** (Exp 019: 9/9 Py, 9/9 Rust) |
| 8 | "Curvature of the freeze-out line in heavy ion collisions" | Phys Rev D 93, 014512 | 2016 | Bazavov et al. | Inverse problem — inferring freeze-out conditions. Same math as seismic inversion, different physics | **Active** (Exp 020: 8/8 Py, 8/8 Rust) |

### Biological Signal vs Noise (Waters)

| # | Paper | Journal | Year | Faculty | Why | Status |
|---|-------|---------|------|---------|-----|--------|
| 9 | Massie et al. "Quantification of High Specificity Cyclic di-GMP Signaling" | PNAS 109:12746-51 | 2012 | Waters | How cells resolve signal from noise with 60+ competing enzymes. Biological Exp 001 | **Active** (Exp 006: 12/12 Py, 12/12 Rust) |
| 10 | Fernandez et al. "V. cholerae adapts by c-di-GMP regulation of cell shape" | PNAS 117:29046-29054 | 2020 | Waters | Bistable switching — when does noise push a system across a threshold? Bifurcation analysis | **Active** (Exp 010: 10/10 Py, 10/10 Rust) |
| 11 | Srivastava et al. "Integration of Cyclic di-GMP and Quorum Sensing" | J Bacteriology 193:6331-41 | 2011 | Waters | Multi-input signal fusion in noisy environment. Biological analog of sensor fusion | **Active** (Exp 011: 9/9 Py, 9/9 Rust) |

### Statistical Confidence & Resampling (Liu)

| # | Paper | Journal | Year | Faculty | Why | Status |
|---|-------|---------|------|---------|-----|--------|
| 12 | Wang et al. "Build a better bootstrap and the RAWR shall beat a random path" | Bioinformatics (ISMB) 37:i111-i119 | 2021 | Liu | RAWR: modern weighted resampling that outperforms naive bootstrap for structured data. Upgrade for Exp 003 | **Active** (Exp 007: 11/11 Py, 11/11 Rust) |
| 13 | Lee & Liu "A Statistical Optimization Technique to Inform Statistical Resampling" | IEEE BIBM 2024 | 2024 | Liu | Meta-statistical optimization — improving the resampling strategy itself | **Active** (Exp 013: 10/10 Py, 8/8 Rust) |

### Anderson Localization & Spectral Theory (Kachkovskiy)

Ilya Kachkovskiy (Math, MSU — previously IAS, UC Irvine; co-author with
Fields Medalist Jean Bourgain) studies when waves propagate vs. when disorder
traps them. This is the rigorous mathematical formalization of groundSpring's
central question: **when does signal propagate through a noisy system, and when
does noise win?**

| # | Paper | Journal | Year | Faculty | Why | Status |
|---|-------|---------|------|---------|-----|--------|
| 15 | Bourgain & Kachkovskiy "Anderson localization for two interacting quasiperiodic particles" | GAFA 29:3-43 | 2018 | Kachkovskiy | Anderson localization = signal trapped by disorder. Two-particle case models how coupled noisy sensors affect each other — directly extends Exp 001's correlated sensor noise decomposition | **Active** (Exp 008: 8/8 Py, 8/8 Rust) |
| 16 | Jitomirskaya & Kachkovskiy "All couplings localization for quasiperiodic operators with Lipschitz monotone potentials" | JEMS 21:777-795 | 2018 | Kachkovskiy | Localization at ALL coupling strengths for monotone potentials. Quasiperiodic = "almost periodic" = structured noise (seasonal drift, tidal cycles, orbital harmonics). Math of Exp 002's ERA5 vs station gap | **Active** (Exp 009: 8/8 Py, 8/8 Rust) |
| 17 | Kachkovskiy "On transport properties of isotropic quasiperiodic XY spin chains" | CMP 345:659-673 | 2016 | Kachkovskiy | Energy transport through disordered chains — when does a signal reach the other end? Mathematical framework for Exp 005's seismic wave propagation through heterogeneous crust | **Active** (Exp 012: 18/18 Py, 18/18 Rust) |
| 18 | Filonov & Kachkovskiy "On the structure of band edges of 2d periodic elliptic operators" | Acta Math 221:59-80 | 2018 | Kachkovskiy | Band edges = frequencies where waves transition from propagating to evanescent. The mathematical boundary between "signal gets through" and "noise kills it" | **Active** (Exp 018: 8/8 Py, 10/10 Rust) |

**Why this is groundSpring's mathematical foundation**: groundSpring's 5 pillars —
Signal vs Noise, Inverse Problems, Sensing Systems, Temporal Dynamics, Spatial
Propagation — are all application domains of spectral theory. Kachkovskiy proves
the theorems; groundSpring runs the experiments.

| groundSpring Pillar | Kachkovskiy Paper | Connection |
|--------------------|--------------------|------------|
| Signal vs Noise | Bourgain-Kachkovskiy 2018 | Anderson localization = noise trapping signal |
| Inverse Problems | Filonov-Kachkovskiy 2018 | Band edge structure constrains inverse solutions |
| Sensing Systems | Bourgain-Kachkovskiy 2018 | Two interacting particles = two coupled sensors |
| Temporal Dynamics | Jitomirskaya-Kachkovskiy 2018 | Quasiperiodic potentials = structured temporal noise |
| Spatial Propagation | Kachkovskiy 2016 | Transport in disordered chains = wave propagation through heterogeneous media |

### Stochastic vs Deterministic Evolution in Extreme Environments (R. Anderson)

Rika Anderson (Carleton College) studies when evolutionary forces in extreme
environments are governed by natural selection (deterministic signal) vs. genetic
drift (stochastic noise). Her 2021 mSystems paper explicitly frames this as the
central question of subsurface microbiology — and it maps directly to groundSpring's
founding question: **when does signal propagate and when does noise win?**

| # | Paper | Journal | Year | Faculty | Why | Status |
|---|-------|---------|------|---------|-----|--------|
| 19 | Anderson (2021) "Tracking Microbial Evolution in the Subseafloor Biosphere" | mSystems 6:e00731-21 | 2021 | R. Anderson | Formalizes when stochastic forces dominate over deterministic selection in low-biomass environments. Cites Lenski LTEE (§1.2 of CONSTRAINED_EVOLUTION_FORMAL.md). Introduces Muller's ratchet as consequence of extreme energy limitation. Directly maps to groundSpring's signal vs noise framework | **Reference** (review paper — theoretical framework, not a numbered reproduction; empirical validation via Paper 20 mBio Exp 014) |
| 20 | Anderson et al. (2022) "Microbial population dynamics are dominated by stochastic forces in a low biomass subseafloor habitat" | mBio 13:e00354-22 | 2022 | R. Anderson | **Empirical proof** that drift dominates selection in energy-limited subsurface. Quantitative genomic evidence for stochastic > deterministic evolution. The biological equivalent of groundSpring Exp 001's finding that noise dominates signal in some sensor configurations | **Active** (Exp 014: 7/7 Py, 7/7 Rust) |
| 21 | Anderson, Sogin, Baross (2015) "Biogeography and ecology of the rare and abundant microbial lineages" | FEMS Microbiol Ecol 91:fiu016 | 2015 | R. Anderson | Rare biosphere problem — when does a detected microbial lineage represent real biological signal vs. sequencing noise? Directly extends groundSpring Exp 004's genus saturation analysis | **Active** (Exp 016: 11/11 Py, 12/12 Rust) |

**Why this is groundSpring's evolutionary validation**: groundSpring decomposes
measurement error into bias and noise across physics, agriculture, meteorology,
biology, and geophysics. Anderson does the same decomposition in evolutionary
biology: selection = signal, drift = noise, and the environment determines which
dominates. Her 2022 mBio paper proves drift dominance empirically — exactly as
groundSpring Exp 001 proves bias dominance in EC5 sensors and Exp 005 shows
depth is poorly constrained by surface noise.

| groundSpring Pillar | R. Anderson Paper | Connection |
|--------------------|--------------------|------------|
| Signal vs Noise | Anderson (2022) mBio | Selection (signal) vs drift (noise) in low-biomass systems |
| Inverse Problems | Anderson et al. (2017) Nat Comm | Inferring evolutionary history from metagenomic snapshots |
| Sensing Systems | Anderson (2015) FEMS | Rare biosphere — is a detected lineage signal or sampling artifact? |
| Temporal Dynamics | Anderson (2021) mSystems | How quickly does evolution proceed under extreme energy limitation? |
| Spatial Propagation | Anderson (2021) mSystems | Gene flow via fluid highways — dispersal vs isolation in the deep sea |

### No-Till Sampling Design & Soil Measurement Uncertainty (baseCamp Sub-thesis 06)

baseCamp Sub-thesis 06 applies Anderson localization to no-till soil health.
groundSpring's contribution is the **uncertainty budget** for the cross-spring
pipeline: how does measurement noise in soil moisture, 16S sequencing depth,
and aggregate stability propagate into QS regime predictions?

| # | Paper | Journal | Year | Faculty | Why | Status |
|---|-------|---------|------|---------|-----|--------|
| 22 | Soil moisture → Anderson geometry uncertainty propagation | — | Cross-spring | Extend Exp 003 (FAO-56 uncertainty): humidity uncertainty → θ(t) uncertainty → d_eff(t) uncertainty → r(t) uncertainty. How much does 66% humidity-dominated ET₀ error affect QS regime prediction? | **Active** (Exp 022: 7/7 Py, 7/7 Rust) |
| 23 | No-till vs tilled 16S sampling design | — | Cross-spring | Extend Exp 004 (genus saturation at 5,000 reads): is the saturation depth different in no-till (higher diversity) vs tilled (lower diversity) soil? Does aggregate stability affect DNA extraction and therefore effective sampling depth? | **Active** (Exp 023: 7/7 Py, 7/7 Rust) |
| 24 | Aggregate stability measurement noise | — | Cross-spring | How precisely must aggregate stability be measured to distinguish Anderson regimes (d_eff = 2 vs d_eff = 3)? Error decomposition similar to Exp 001 (sensor noise) | **Active** (Exp 024: 8/8 Py, 8/8 Rust) |

**Cross-spring impact**: groundSpring provides error bars for the entire
cross-spring pipeline. Exp 003 → airSpring θ(t) uncertainty. Exp 004 →
wetSpring 16S sampling design. New Exp 022-024 → baseCamp Sub-thesis 06
QS regime prediction confidence intervals.

### WDM Simulation Uncertainty & Consumer GPU Error Budget (baseCamp Sub-thesis 07)

baseCamp Sub-thesis 07 claims WDM transport coefficients can be reproduced
on consumer GPU hardware. groundSpring's contribution is the **uncertainty
budget**: how does finite precision (f32 vs f64), finite system size, and
GPU arithmetic affect the accuracy of WDM observables?

| # | Target | Domain | Connection | Status |
|---|--------|--------|-----------|--------|
| 25 | f32 vs f64 transport coefficient drift | WDM MD | Extend Exp 001 (sensor noise) methodology: decompose f32→f64 error into systematic bias vs stochastic noise. Does reduced precision introduce directional bias in D*, η*, λ*? | **Active** (Exp 025: 7/7 Py, 7/7 Rust) |
| 26 | System-size convergence for WDM transport | WDM MD | At what N does consumer GPU (N≤10k) transport converge vs institutional HPC (N≥100k)? Map the N→∞ extrapolation uncertainty | **Active** (Exp 026: 7/7 Py, 7/7 Rust) |
| 27 | GPU vendor parity for WDM observables | WDM MD | Extend hotSpring's RTX 4070 vs Titan V (NVK) parity tests to WDM conditions. Does vendor/driver affect physics? (should be zero, but prove it) | **Active** (Exp 027: 7/7 Py, 7/7 Rust) |

**Cross-spring impact**: These experiments provide the error bars for
Sub-thesis 07's central claim. If f32→f64 bias is <1% for transport
coefficients, the entire distributed consumer GPU argument holds. If
system-size convergence requires N>50k, the RTX 4070 (12 GB) approach
needs qualification.

### Eco-Evolutionary Noise (Dolson)

| # | Paper | Journal | Year | Faculty | Why | Status |
|---|-------|---------|------|---------|-----|--------|
| 14 | Dolson et al. "The ecology-evolution continuum and the origin of life" | J R Soc Interface 20(208) | 2023 | Dolson | Where does signal begin in a system that starts as pure noise? Origin-of-life context | **Active** (Exp 017: 9/9 Py, 6/6 Rust) |

### Anderson Localization in Immunological Signaling (baseCamp Paper 12)

| # | Target | Domain | Connection | Status |
|---|--------|--------|-----------|--------|
| 29 | Cytokine Anderson lattice (Exp 033) | Immunological (tissue geometry) | 2D epidermis + 3D dermis as Anderson lattice; barrier disruption → dimensional promotion; Pielou evenness → effective disorder W | **Active** (29/29 Rust) |
| 30 | Geometry-aware drug scoring (Exp 034) | Immunological (drug repurposing) | Anderson-augmented Fajgenbaum MATRIX score: pathway_score × penetration_factor × anderson_factor; AD panel (6 drugs) | **Active** (combined with Exp 033) |

**Cross-spring impact**: Paper 12 bridges groundSpring's Anderson localization (Papers 15-18)
with immunological signaling. The dimensional promotion–collapse duality connects
Paper 06 (tillage COLLAPSES 3D→2D soil structure, bad) with Paper 12 (scratching
PROMOTES 2D→3D cytokine propagation, bad). Same physics, opposite directions,
context-dependent outcomes. Drug repurposing scoring adds spatial geometry to
pathway-based approaches — a drug must both target the right pathway AND physically
reach its target through the tissue Anderson lattice.

---

## Open Data Provenance Audit

Every paper in the queue must use **open data and open systems** — no paywalled
datasets, no proprietary software dependencies.

| # | Paper | Data Source | Access | Open? |
|---|-------|-----------|--------|:-----:|
| 1-5 | Completed experiments | Dong et al. 2020 (Ag 10:598), FAO-56 Ex 18, synthetic | DOI / analytical | **Yes** |
| 6 | Bazavov spectral reconstruction | MILC lattice ensembles (ILDG/USQCD) | Public gauge configs | **Yes** |
| 7 | Bazavov muon g-2 | HotQCD/MILC lattice configs | ILDG public | **Yes** |
| 8 | Bazavov freeze-out | STAR/PHENIX beam energy scan | BNL open data | **Yes** |
| 9 | Massie c-di-GMP | PNAS supplementary (SI Tables) | Open access | **Yes** |
| 10 | Fernandez cell shape | PNAS supplementary (flow cytometry) | Open access | **Yes** |
| 11 | Srivastava QS integration | J Bacteriology SI (qRT-PCR) | Open access | **Yes** |
| 12 | Wang RAWR | Simulation + TreeBASE/Dryad | Public repos | **Yes** |
| 13 | Lee & Liu resampling | Simulation (reproducible) | Params in paper | **Yes** |
| 14 | Dolson eco-evolution | Theoretical/simulation | Open source (MABE) | **Yes** |
| 15-18 | Kachkovskiy spectral theory | Analytical (theorems) | Fully specified | **Yes** |
| 19 | R. Anderson mSystems | Review paper | Open access | **Yes** |
| 20 | R. Anderson mBio | NCBI SRA metagenomes | SRA accession | **Yes** |
| 21 | R. Anderson FEMS | NCBI SRA 16S amplicons | SRA accession | **Yes** |
| 22-24 | Cross-spring sub-thesis 06 | Derived from Exp 001-004 | Internal | **Yes** |
| 25-27 | Sub-thesis 07 (WDM GPU) | Simulation + analytical | Reproducible | **Yes** |
| 28 | NPU Anderson (AKD1000) | ToadStool akida-driver | Pure Rust, zero mocks | **Yes** |
| 29 | Cytokine Anderson lattice | Synthetic + GWAS catalog (NHGRI-EBI) | Open access | **Yes** |
| 30 | Geometry-aware drug scoring | DrugBank open + ChEMBL 34 | CC-BY-SA / Open access | **Yes** |
| 31-32 | NUCLEUS sovereign experiments | NOAA ISD, NCBI SRA, IRIS FDSN | US gov open data / public repos | **Yes** |
| 33 | Tissue Anderson 4D + Wegner RG | Synthetic lattice (analytical) | Reproducible params | **Yes** |

| 34 | Multi-Method ET₀ (Exp 035) | FAO-56 Example 18 (analytical) | Reproducible params | **Yes** |

**Status**: All 34 papers use open data or open systems. Zero proprietary dependencies.
**Verified V69**: `test_baseline_integrity.py` confirms all baseline JSONs have complete provenance.

---

## Three-Tier Control Matrix

Each paper's experiments are validated at three hardware tiers, following the
Write → Absorb → Lean cycle:

| Tier | Substrate | Description | How |
|------|-----------|-------------|-----|
| **CPU** | `cargo test` + validation binary | Rust matches Python baseline | BarraCUDA CPU stats where available |
| **GPU** | `barracuda` feature + GPU adapter | GPU matches CPU within tolerance | BarraCUDA GPU ops (reduce, map, fused) |
| **metalForge** | Mixed hardware dispatch | Cross-substrate agreement | metalForge forge crate routes to best substrate |

### Completed Experiments (Papers 1-33)

| # | Experiment | CPU | GPU | metalForge | Barracuda delegation |
|---|-----------|:---:|:---:|:----------:|---------------------|
| 1 | Sensor noise decomposition | **36/36** | **Wired** (MAE/NSE/R² via `FusedMapReduceF64`) | Workload | 3 stats (CPU+GPU) |
| 2 | Observation gap (ERA5 vs station) | **13/13** | **Wired** (MAE/NSE/R² via `FusedMapReduceF64`) | Workload | 3 stats (CPU+GPU) |
| 3 | Error propagation FAO-56 | **15/15** | **Wired** (V67 `McEt0PropagateGpu` + `SeasonalPipelineF64`) | Workload | fao56 + MC GPU |
| 4 | Sequencing noise | **15/15** | **Wired** (V67 `BatchedMultinomialGpu`) | Workload | multinomial GPU |
| 5 | Seismic source inversion | **9/9** | **Wired** (V55 grid dispatch + stats GPU) | Workload | grid search GPU adapter |
| 6 | Spectral function reconstruction | **8/8** | **Wired** (V67 Cholesky GPU + `tikhonov_solve_gpu`) | Workload | dense linear algebra GPU |
| 7 | Jackknife error estimation | **9/9** | **Wired** (V59 `JackknifeMeanGpu`) | Workload | `jackknife_mean_f64.wgsl` |
| 8 | Freeze-out inverse problem | **8/8** | **Wired** (V53 grid + V68 `lbfgs_numerical`) | Workload | grid+L-BFGS GPU |
| 9 | Enzymatic signal specificity | **12/12** | **Wired** (V63 `GillespieGpu` + `BatchedOdeRK4F64`) | Workload | Gillespie+ODE GPU |
| 10 | Bistable phenotypic switching | **10/10** | **Wired** (V66 `BatchedOdeRK4F64` bistable) | Workload | ODE batch GPU |
| 11 | Multi-signal QS integration | **9/9** | **Wired** (V66 `MultiSignalOde` batch) | Workload | ODE batch GPU |
| 12 | RAWR resampling | **11/11** | CPU delegation (`rawr_mean`) | — | CPU 11.6× vs Python |
| 13 | Resampling convergence | **8/8** | CPU delegation (`bootstrap`) | — | Builds on #12 |
| 14 | Eco-evolutionary noise threshold | **6/6** | **Wired** (`WrightFisherGpu` fixation) | — | multinomial+mutation GPU |
| 15 | Anderson localization | **8/8** | **Wired** (V62 `anderson_sweep` + `lyapunov_averaged`) | Workload | spectral GPU |
| 16 | Almost-Mathieu quasiperiodic | **8/8** | **Wired** (V33 Sturm GPU, **47.4× speedup**) | Workload | `find_all_eigenvalues` GPU |
| 17 | Spin chain transport | **18/18** | **Partial** (eigenvalues GPU, eigenvectors CPU) | — | `tridiag_eigh` candidate |
| 18 | Band edge structure | **10/10** | **Wired** (V55 Brent band edge) | Workload | `optimize::brent` GPU |
| 20 | Drift vs selection | **7/7** | **Wired** (V63 `WrightFisherGpu` + multinomial) | — | fixation sim GPU |
| 21 | Rare biosphere signal detection | **12/12** | **Wired** (V31 `BatchedMultinomialGpu`) | Workload | occupancy GPU |
| 22 | ET₀ → Anderson uncertainty | **7/7** | **Wired** (V67 `McEt0PropagateGpu`) | — | MC→spectral GPU chain |
| 23 | No-till sampling design | **7/7** | **Wired** (`BatchedMultinomialGpu` + Shannon GPU) | — | rarefaction GPU (V95) |
| 24 | Aggregate stability noise | **8/8** | **Wired** (`rmse`/`mbe`/`mean_and_std_dev` GPU) | — | stats GPU (V95) |
| 25-27 | WDM sub-thesis 07 | **21/21** | CPU delegation (analytical math) | — | No GPU path needed |
| 28 | NPU Anderson classification | **9/9** | — | **Live** (AKD1000 DMA) | int8 centroid on NPU |
| 29-32 | NUCLEUS sovereign experiments | **55/55** | — | Sovereign fallback | Real data (NOAA/NCBI/IRIS) |
| 33 | Tissue Anderson 4D + Wegner RG | **29/29** | **Wired** (V68 `anderson_4d` + `wegner_block_4d`) | Workload | 4D Anderson + RG GPU |

**CPU tier**: 395/395 PASS across 34 validation binaries.
**Barracuda**: 110 active delegations (67 CPU + 43 GPU) — barraCuda v0.3.7, toadStool S158+, coralReef Iteration 55+. **Performance**: 11.6× faster than Python (excl. LAPACK-bound); 5.1× overall; 53.5× peak (seismic). **Tests**: 1020+ default-feature Rust tests + 287 Python provenance. 100+ three-tier parity tests (100% delegation coverage). `PrecisionRoutingAdvice` wired.
**Mathematical parity**: 29/29 PROVEN. Generate reports: `python3 scripts/parity_report.py` and `python3 scripts/bench_rust_vs_python.py`.
**Three-tier parity**: 100+ parity tests validate CPU ↔ barracuda-CPU equivalence (100% delegation coverage).
**GPU tier**: 15 modules wired with `#[cfg(feature = "barracuda-gpu")]` — stats Tier A complete (MAE, NSE, R²), bistable batch ODE, McEt0PropagateGpu, SeasonalPipelineF64, 4D Anderson + Wegner RG, L-BFGS refinement. 30 metalForge workloads (24 GPU + 2 NPU + 2 CPU-only). GPU grid adapters (seismic, freeze-out). 936 tests pass. 11 GPU dispatch paths runtime smoke test + three-tier parity (V97).
**metalForge tier**: 30 workloads, 187 checks (groundspring-forge crate, Exp 028 NPU DMA on AKD1000, pipeline dispatch, PCIe topology, GPU→NPU bypass).

### GPU / metalForge Progression (updated V74 — toadStool S93)

| # | Paper (short) | CPU | GPU | metalForge | Status |
|---|--------------|:---:|:---:|:----------:|--------|
| 6 | Bazavov spectral | **8/8 PASS** | **Wired** (V67 Cholesky GPU) | Workload defined | `cholesky_f64` + `tikhonov_solve_gpu` |
| 7 | Bazavov g-2 | **9/9 PASS** | **Wired** (V59 JackknifeMeanGpu) | Workload defined | `jackknife_mean_f64.wgsl` |
| 8 | Bazavov freeze-out | **8/8 PASS** | **Wired** (V53 grid + V68 L-BFGS) | Workload defined | `grid_search_3d` + `lbfgs_numerical` |
| 9 | Massie c-di-GMP | **12/12 PASS** | **Wired** (V63 GillespieGpu) | Workload defined | `GillespieGpu` + `BatchedOdeRK4F64` |
| 10 | Fernandez cell shape | **10/10 PASS** | **Wired** (V66 BatchedOdeRK4) | Workload defined | `BatchedOdeRK4F64` (bistable batch) |
| 11 | Srivastava QS | **9/9 PASS** | **Wired** (V66 ODE batch) | Workload defined | `MultiSignalOde` batch path |
| 12 | Wang RAWR | **11/11 PASS** | CPU delegation (rawr_mean) | — | CPU already faster than Python |
| 13 | Lee resampling | **8/8 PASS** | CPU delegation (bootstrap) | — | Builds on #12 |
| 14 | Dolson eco-evo | **6/6 PASS** | **Wired** (quasispecies GPU) | — | `WrightFisherGpu` for fixation sim |
| 15 | Bourgain-Kachkovskiy | **8/8 PASS** | **Wired** (V62+ spectral GPU) | Workload defined | `anderson_sweep`, `lyapunov_averaged` |
| 16 | Jitomirskaya-Kachkovskiy | **8/8 PASS** | **Wired** (V33 Sturm GPU) | Workload defined | `find_all_eigenvalues` (**47.4× speedup**) |
| 17 | Kachkovskiy transport | **18/18 PASS** | **Partial** (spectral GPU) | — | Eigenvalues GPU, eigenvectors CPU-only |
| 18 | Filonov-Kachkovskiy | **10/10 PASS** | **Wired** (V55 Brent) | Workload defined | `optimize::brent` band edge refinement |
| 20 | R. Anderson mBio | **7/7 PASS** | **Wired** (V63 WF + multinomial) | — | `WrightFisherGpu` + `BatchedMultinomialGpu` |
| 21 | R. Anderson FEMS | **12/12 PASS** | **Wired** (V31 rare biosphere GPU) | Workload defined | `BatchedMultinomialGpu` occupancy |
| 22 | ET₀ → Anderson | **7/7 PASS** | **Wired** (V67 McEt0 GPU) | — | `McEt0PropagateGpu` + Anderson spectral |
| 23 | No-till sampling | **7/7 PASS** | **Wired** (`BatchedMultinomialGpu` + Shannon) | — | Rarefaction GPU chain (V95) |
| 24 | Aggregate stability | **8/8 PASS** | **Wired** (`rmse`/`mbe`/`mean_and_std_dev` GPU) | — | All stats GPU (V95) |
| 25-27 | WDM sub-thesis 07 | **21/21 PASS** | CPU delegation | — | CPU math proven, no GPU path needed |
| 28 | NPU Anderson | **9/9 PASS** | — | **Live** (AKD1000) | int8 DMA on NPU hardware |
| 29-32 | NUCLEUS experiments | **55/55 PASS** | — | sovereign fallback | Real data (NOAA, NCBI, IRIS) |
| 33 | Paper 12 tissue Anderson | **29/29 PASS** | **Wired** (V68 4D Anderson + RG) | Workload defined | `anderson_4d` + `wegner_block_4d` |

### BarraCUDA Kernel Requirements Summary (V74 — toadStool S93)

| Kernel | Papers | Status | Priority |
|--------|--------|--------|----------|
| `fused_map_reduce_f64` (GPU) | 1-5 (stats Tier A) | **WIRED** — MAE/NSE/R² GPU adapters live (V66) | ~~HIGH~~ Done |
| `norm_reduce_f64` (GPU) | 1-5 (RMSE) | **WIRED** — GPU adapter live (V66) | ~~HIGH~~ Done |
| `batched_multinomial` | 4, 20-21, 22-24 | **WIRED** — `BatchedMultinomialGpu` (V67, API fix) | ~~HIGH~~ Done |
| `BatchedElementwiseF64::fao56_et0_batch` | 3, 22 | **ABSORBED** — exists in barracuda (S49) | ~~HIGH~~ Done |
| `FusedMapReduceF64::shannon_entropy` | 4, 20-21 | **ABSORBED** — convenience method exists | ~~HIGH~~ Done |
| `McEt0PropagateGpu` | 3, 22 | **WIRED** (V67) — GPU Monte Carlo ET₀ propagation | ~~HIGH~~ Done |
| `SeasonalPipelineF64` | 3, 22 | **WIRED** (V67) — fused seasonal water balance GPU | ~~HIGH~~ Done |
| `lbfgs_numerical` | 8 | **WIRED** (V68) — post-grid-search L-BFGS refinement | ~~HIGH~~ Done |
| `anderson_4d` + `wegner_block_4d` | 33 | **WIRED** (V68) — 4D tissue Anderson + Wegner RG | ~~HIGH~~ Done |
| FFT (real, complex) | 6 (optional) | **WIRED** (V93) — `spectral_recon::fft_power_spectrum()` delegates to `Fft1DF64`. CPU DFT fallback. | ~~MEDIUM~~ Done |
| `JackknifeMeanGpu` | 7 | **WIRED** (V59) — `jackknife_mean_f64.wgsl` | ~~MEDIUM~~ Done |
| RAWR weighted resampling | 12, 13 | **ABSORBED** — `stats::rawr_mean` (S66) | ~~MEDIUM~~ Done |
| Grid search 3D dispatch | 5, 8 | **WIRED** (V53) — GPU grid adapter + argmin | ~~MEDIUM~~ Done |
| Spectral recon (Cholesky, mat-vec) | 6 | **WIRED** (V67) — `cholesky_f64` + `tikhonov_solve_gpu` | ~~MEDIUM~~ Done |
| Gillespie SSA (GPU) | 9, 10, 11 | **WIRED** (V63) — `GillespieGpu` + `BatchedOdeRK4F64` | Done |
| Bio ODEs (Bistable, Cooperation, etc.) | 9, 10, 11 | **WIRED** (V66) — 5 ODE systems batch GPU | Done |
| NMF (Euclidean + KL) | 20, 21 | **ABSORBED** (S58) — `linalg::nmf` | Done |
| Anderson 1D/2D/3D/4D + disordered_laplacian | 15-18, 33 | **WIRED** — expanded to 4D (V68) | Done |
| Lanczos eigensolve (GPU) | 15-18 | **WIRED** — `spectral` GPU | Done |
| Smith-Waterman (GPU) | 20, 21 | **WIRED** — `SmithWatermanGpu` | Done |
| Bray-Curtis (GPU) | 20, 21 | **WIRED** — `BrayCurtisF64` | Done |
| Hill kinetics (`hill`, `hill_repress`) | 10, 11 | **WIRED** — `barracuda::stats::hill` (S68) | Done |
| Brent root-finding (GPU) | 18 | **WIRED** (V55) — band edge refinement | Done |
| Eigenvector solver (tridiag QL) | 17 | **Gap** — eigenvalues only (Sturm); eigenvectors CPU-only | MEDIUM |

### GPU-Ready vs GPU-Blocked

**GPU-Ready** (27 of 33 papers have GPU wiring — 82%):
Papers 1-11, 14, 15, 16, 18, 20, 21, 22, 23, 24, 33 — **fully wired** with active GPU delegation.

**GPU-Blocked** (remaining gaps):
- Paper 17 — eigenvector solver (Sturm finds eigenvalues on GPU, eigenvectors still CPU)
- Papers 12-13, 25-27 — CPU delegation sufficient (no GPU path needed)
- Papers 29-32 — NUCLEUS sovereign (metalForge fallback, not GPU-targeted)

---

## Cross-Spring Paper Connections

| groundSpring Pillar | Current Exp | Faculty Extension | Shared With |
|--------------------|-------------|-------------------|-------------|
| Signal vs Noise | Exp 001 (sensors) | Waters: biological signal specificity | wetSpring |
| Inverse Problems | Exp 005 (seismic) | Bazavov: spectral reconstruction in lattice QCD | hotSpring |
| Sensing Systems | Exp 002 (ERA5 vs station) | Waters: quorum sensing as bio sensor network | wetSpring |
| Temporal Dynamics | Exp 004 (sequencing depth) | Liu: phylogenetic confidence over evolutionary time | wetSpring |
| Error Propagation | Exp 003 (FAO-56 MC) | Liu: RAWR bootstrap — better resampling | neuralSpring |

---

## Hardware Evolution: CPU → GPU → metalForge

### Tier 1: BarraCUDA CPU (current — 376/376 PASS)

Pure safe Rust with optional `barracuda` feature gate delegation.
110 active delegations (67 CPU + 43 GPU) — barraCuda v0.3.7, toadStool S158+. 11.5× faster than Python (excl. LAPACK-bound).
990+ Rust workspace tests + 287 Python provenance tests. 29/29 mathematical parity proven. 100+ three-tier parity tests (100% delegation coverage).
395/395 validation checks across 34 experiments (V113, zero-debt audit certified).
All 34 experiments validated. GPU stats dispatch (mean, std_dev, rmse, mbe, mae, nse, r², pearson_r). L-BFGS post-grid refinement (V68). 14 CPU vs GPU parity tests. CPU vs GPU benchmark binary.

### Tier 2: BarraCUDA GPU (36 GPU dispatch targets — V83)

GPU wiring covers 25 of 33 papers (76%). Unidirectional streaming via
`toadStool` dispatch reduces round trips — CPU uploads once, GPU computes
full pipeline, result downloads once. Cross-spring shader evolution means
`hotSpring` precision shaders (f64 fused-reduce, Cholesky) and `wetSpring`
bio shaders (Gillespie, ODE, multinomial) both feed `groundSpring` GPU tier.

| Category | Papers | Barracuda Op | Status |
|----------|--------|-------------|--------|
| Stats Tier A | 1-5 | `FusedMapReduceF64`, `NormReduceF64` | **DONE** (V66) — MAE/NSE/R²/RMSE GPU |
| Hydrology GPU | 3, 22 | `McEt0PropagateGpu`, `SeasonalPipelineF64` | **DONE** (V67) — full MC→seasonal chain |
| Multinomial GPU | 4, 20-21 | `BatchedMultinomialGpu` | **DONE** (V67, API fix) — occupancy + rare biosphere |
| Spectral recon | 6 | `cholesky_f64`, `tikhonov_solve_gpu` | **DONE** (V67) — dense linear algebra GPU |
| Jackknife GPU | 7 | `JackknifeMeanGpu` | **DONE** (V59) — `jackknife_mean_f64.wgsl` |
| Grid+L-BFGS | 5, 8 | Grid dispatch + `lbfgs_numerical` | **DONE** (V53+V68) — grid→L-BFGS refinement |
| Bio ODE batch | 9-11 | `GillespieGpu`, `BatchedOdeRK4F64` | **DONE** (V63+V66) — 5 ODE systems |
| Eco-evo GPU | 14, 20 | `WrightFisherGpu` | **DONE** (V63) — fixation batch |
| Spectral GPU | 15, 16 | `anderson_sweep`, `find_all_eigenvalues` | **DONE** (V33+V62) — **47.4× speedup** |
| Band edge | 18 | `optimize::brent` | **DONE** (V55) — Brent GPU refinement |
| Rare biosphere | 21 | `BatchedMultinomialGpu` | **DONE** (V31) — occupancy simulation |
| 4D Anderson+RG | 33 | `anderson_4d`, `wegner_block_4d` | **DONE** (V68) — tissue immunology |
| RAWR/bootstrap | 12, 13 | `rawr_mean`, `bootstrap` | **ABSORBED** (S66) — CPU delegation |
| FAO-56 batch | 3, 22 | `BatchedElementwiseF64::fao56_et0_batch` | **ABSORBED** (S49) |
| Shannon entropy | 4, 20-21 | `FusedMapReduceF64::shannon_entropy` | **ABSORBED** |
| Hill kinetics | 10, 11 | `barracuda::stats::hill` | **ABSORBED** (S68) |

### Tier 3: metalForge Cross-Substrate (30 workloads — V69)

Mixed hardware dispatch using metalForge forge crate. 30 workloads
(24 GPU + 2 NPU + 2 CPU-only + 2 mixed), 187 checks. Exp 028 validates
live NPU DMA on AKD1000 hardware. Pipeline dispatch and PCIe topology
tests ensure GPU→NPU→CPU routing without CPU round-trips.

| Validation | Description | Status |
|-----------|-------------|--------|
| CPU ↔ GPU parity | GPU output matches CPU within documented tolerance | **30/30 PASS** |
| Cross-vendor parity | RTX 4070 vs other GPUs produce identical physics | Sub-thesis 07 (25-27) |
| Mixed dispatch | metalForge routes to best substrate per operation | **187 checks PASS** |
| f32 ↔ f64 drift | Sub-thesis 07: quantify precision loss on consumer GPU | **21/21 PASS** |
| NPU DMA | AKD1000 int8 centroid classification via PCIe DMA | **9/9 PASS** (Exp 028) |
| Pipeline topology | GPU→NPU bypass (no CPU round-trip) | Validated in forge crate |

metalForge tier depends on GPU tier completing first. groundSpring's
metalForge focus is statistical kernels; hardware discovery and substrate
dispatch use hotSpring's shared metalForge infrastructure.

---

## Notes

- Papers 6-8 (Bazavov) connect groundSpring to hotSpring via inverse problem methodology
- Papers 9-11 (Waters) connect groundSpring to wetSpring via biological noise/signal
- Papers 12-13 (Liu) upgrade groundSpring's statistical methodology
- Paper 14 (Dolson) is philosophical/theoretical — frames noise as the starting condition for life
- Papers 15-18 (Kachkovskiy) are the mathematical foundation — all use barracuda `spectral`
- Papers 20-21 (R. Anderson) share barracuda bio ops with wetSpring
- The common thread: **extracting reliable conclusions from noisy measurements**
- **All 28 papers use open data and open systems. Zero proprietary dependencies.**
