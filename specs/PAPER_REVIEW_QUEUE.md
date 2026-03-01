# groundSpring — Paper Review Queue

**Last Updated**: March 1, 2026
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

**Phase 0**: ~261 checks (Python). **Phase 1**: 292/292 PASS (Rust). **Speedup**: 11.6× median (excl. LAPACK-bound), 51.2× peak (seismic).
**Mathematical Parity**: 28/28 PROVEN — Python and Rust both pass against shared benchmark JSONs.
**V54 fresh validation**: 283/283 checks (27 binaries), 95/95 three-tier parity, `bench_rust_vs_python.json` saved.
**GPU dispatch (V31–V51)**: 13 modules wired for `barracuda-gpu` — freeze_out, band_structure, seismic, quasispecies, rare_biosphere, stats::metrics, stats::agreement, stats::correlation, gillespie, drift, fao56, almost_mathieu, anderson. 19 metalForge workloads (17 GPU + 2 NPU). 57 active delegations (38 CPU + 19 GPU), 1 evolution candidate — ToadStool S70+++.
**Three-tier parity (V43)**: 27/27 PROVEN (default = barracuda-CPU = barracuda-GPU). GPU tier: 39/39 checks. Pure GPU: 26/26 checks. metalForge dispatch: 17/19 → Titan V.
**Exp 015** bridges Papers 22-24 (Sub-thesis 06): sensor noise → Anderson ξ → QS regime uncertainty.

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
| 19 | Anderson (2021) "Tracking Microbial Evolution in the Subseafloor Biosphere" | mSystems 6:e00731-21 | 2021 | R. Anderson | Formalizes when stochastic forces dominate over deterministic selection in low-biomass environments. Cites Lenski LTEE (§1.2 of CONSTRAINED_EVOLUTION_FORMAL.md). Introduces Muller's ratchet as consequence of extreme energy limitation. Directly maps to groundSpring's signal vs noise framework | Reference |
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

**Status**: All 28 papers use open data or open systems. Zero proprietary dependencies.
**Verified V28**: `test_baseline_integrity.py` confirms all 28 benchmark JSONs have complete provenance (196/196 PASS).

---

## Three-Tier Control Matrix

Each paper's experiments are validated at three hardware tiers, following the
Write → Absorb → Lean cycle:

| Tier | Substrate | Description | How |
|------|-----------|-------------|-----|
| **CPU** | `cargo test` + validation binary | Rust matches Python baseline | BarraCUDA CPU stats where available |
| **GPU** | `barracuda` feature + GPU adapter | GPU matches CPU within tolerance | BarraCUDA GPU ops (reduce, map, fused) |
| **metalForge** | Mixed hardware dispatch | Cross-substrate agreement | metalForge forge crate routes to best substrate |

### Completed Experiments (Papers 1-5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 20, 21, 22, 23, 24, 25, 26, 27)

| # | Experiment | CPU | GPU | metalForge | Barracuda delegation |
|---|-----------|:---:|:---:|:----------:|---------------------|
| 1 | Sensor noise decomposition | **36/36** | Tier A pending (reduce ops) | After GPU | 3 stats (CPU) |
| 2 | Observation gap (ERA5 vs station) | **13/13** | Tier A pending (reduce ops) | After GPU | 3 stats (CPU) |
| 3 | Error propagation FAO-56 | **15/15** | Tier C (`mc_et0_propagate.wgsl`) | After GPU | fao56 absorbed |
| 4 | Sequencing noise | **15/15** | Tier C (`batched_multinomial.wgsl`) | After GPU | — |
| 5 | Seismic source inversion | **9/9** | Tier B (grid dispatch) | After GPU | — |
| 6 | Spectral function reconstruction | **8/8** | Dense linear algebra (Cholesky, mat-vec) | After GPU | Highest GPU potential of Bazavov trio |
| 7 | Jackknife error estimation | **9/9** | Embarrassingly parallel (N leave-one-out subsets) | After GPU | Jackknife GPU kernel candidate |
| 8 | Freeze-out inverse problem | **8/8** | Grid search embarrassingly parallel | After GPU | Grid dispatch candidate |
| 9 | Enzymatic signal specificity | **12/12** | `GillespieGpu` (ready) | After GPU | GPU-only (no CPU) |
| 10 | Bistable phenotypic switching | **10/10** | `BistableOde` (ready) | After GPU | `BistableOde::cpu_derivative` |
| 11 | Multi-signal QS integration | **9/9** | `MultiSignalOde` (ready) | After GPU | `MultiSignalOde::cpu_derivative` |
| 12 | RAWR resampling | **11/11** | Embarrassingly parallel | After GPU | `bootstrap_mean` (CPU) |
| 13 | Resampling convergence | **8/8** | Embarrassingly parallel | After GPU | Uses `bootstrap` module |
| 15 | Anderson localization | **8/8** | `spectral::*` (ready) | After GPU | 2 lyapunov (barracuda-gpu) |
| 16 | Almost-Mathieu quasiperiodic | **8/8** | `almost_mathieu_hamiltonian` (ready) | After GPU | barracuda-gpu delegation |
| 17 | Spin chain transport | **18/18** | `transport::*` (ready) | After GPU | tridiag_eigh candidate |
| 20 | Drift vs selection | **7/7** | Embarrassingly parallel | After GPU | wright_fisher_fixation, kimura_fixation_prob candidates |
| 14 | Eco-evolutionary noise threshold | **6/6** | Embarrassingly parallel | After GPU | Simulation-only (multinomial+mutation) |
| 18 | Band edge structure | **10/10** | Transfer matrix per-energy parallel | After GPU | tridiag_eigh candidate |
| 21 | Rare biosphere signal detection | **12/12** | Embarrassingly parallel | After GPU | Chao1, multinomial sampling |
| 28 | NPU Anderson regime classification | **9/9** | — | **Live** (AKD1000 DMA) | int8 centroid classifier on NPU |

**CPU tier**: 283/283 PASS across 27 validation binaries (Exp 028 NPU hardware-only = +9 checks).
**Barracuda**: 57 active delegations (38 CPU + 19 GPU), 1 evolution candidate — ToadStool S70+++. **Performance**: 11.6× faster than Python (excl. LAPACK-bound); 5.2× overall; 51.2× peak (seismic). **Tests**: 569 Rust workspace + 375 Python = 944. 95 three-tier parity tests (100% delegation coverage).
**Mathematical parity**: 28/28 PROVEN. See `data/parity_report.json` and `data/bench_rust_vs_python.json`.
**Three-tier parity**: 95 parity tests validate CPU ↔ barracuda-CPU equivalence (100% delegation coverage).
**GPU tier**: 13 modules wired with `#[cfg(feature = "barracuda-gpu")]` — including GPU grid adapters (seismic, freeze-out). 316/322 tests pass (6 require f64-capable GPU: Titan V / A100).
**metalForge tier**: partially validated (groundspring-forge crate, Exp 028 NPU DMA on AKD1000).

### GPU / metalForge Progression (updated V54 — ToadStool S70+++)

| # | Paper (short) | CPU | GPU | metalForge | Blocker |
|---|--------------|:---:|:---:|:----------:|---------|
| 6 | Bazavov spectral | **8/8 PASS** | After CPU | — | Dense linear algebra (Cholesky, mat-vec) — highest GPU potential |
| 7 | Bazavov g-2 | **9/9 PASS** | After CPU | — | Jackknife GPU kernel candidate (embarrassingly parallel) |
| 8 | Bazavov freeze-out | **8/8 PASS** | **Wired** (V53) | — | `grid_search_3d` GPU adapter (pre-eval + argmin) |
| 9 | Massie c-di-GMP | **12/12 PASS** | **Ready** | — | `GillespieGpu` + `BatchedOdeRK4` + 5 bio ODEs (S58) |
| 10 | Fernandez cell shape | **9/9 PASS** | **Ready** | — | `BatchedEighGpu` + `BistableOde` (S58) |
| 11 | Srivastava QS | **8/8 PASS** | **Ready** | — | `CooperationOde` + `MultiSignalOde` (S58) |
| 12 | Wang RAWR | **11/11 PASS** | Ready | — | Embarrassingly parallel |
| 13 | Lee resampling | **8/8 PASS** | After 12 | — | Builds on #12 |
| 14 | Dolson eco-evo | **6/6 PASS** | Ready | — | Simulation only |
| 15 | Bourgain-Kachkovskiy | **8/8 PASS** | **Ready** | — | `spectral` + Anderson (S56) |
| 16 | Jitomirskaya-Kachkovskiy | **8/8 PASS** | **Ready** | — | Almost-Mathieu + `disordered_laplacian` (S56) |
| 17 | Kachkovskiy transport | **18/18 PASS** | After 15 | — | Builds on #15 |
| 18 | Filonov-Kachkovskiy | **10/10 PASS** | After 15 | — | Builds on #15 |
| 19 | R. Anderson (review) | Reference | — | — | Not a reproduction |
| 20 | R. Anderson mBio | **7/7 PASS** | Partial | — | `SmithWatermanGpu` + `BrayCurtisF64` + NMF (S58); rarefaction GPU still Tier C |
| 21 | R. Anderson FEMS | **10/10 PASS** | Partial | — | Same as #20 |
| 22-24 | Sub-thesis 06 | Queued | After 1-4 GPU | — | Depends on Exp 001-004 GPU tier |

### BarraCUDA Kernel Requirements Summary (post ToadStool S70+++)

| Kernel | Papers | Status | Priority |
|--------|--------|--------|----------|
| `fused_map_reduce_f64` (GPU) | 1-5 (stats Tier A) | Exists — needs GPU adapter | **HIGH** |
| `norm_reduce_f64` (GPU) | 1-5 (RMSE) | Exists — needs GPU adapter | **HIGH** |
| `batched_multinomial` | 4, 20-21, 22-24 | **Tier C**: production WGSL in metalForge | HIGH |
| `BatchedElementwiseF64::fao56_et0_batch` | 3, 22 | **ABSORBED** — exists in barracuda (S49) | ~~HIGH~~ Done |
| `FusedMapReduceF64::shannon_entropy` | 4, 20-21 | **ABSORBED** — convenience method exists | ~~HIGH~~ Done |
| FFT (real, complex) | 6 (optional) | **Gap** — not in barracuda | MEDIUM |
| Jackknife leave-one-out | 7 | CPU complete — embarrassingly parallel, GPU candidate for large N | MEDIUM |
| RAWR weighted resampling | 12, 13 | **ABSORBED** — `stats::rawr_mean` (S66) | ~~MEDIUM~~ Done |
| Grid search 3D dispatch | 5, 8 | CPU complete (8/8) — grid search embarrassingly parallel | MEDIUM |
| Spectral recon (Cholesky, mat-vec) | 6 | CPU complete (8/8) — dense linear algebra, highest GPU potential | MEDIUM |
| Gillespie SSA (GPU) | 9, 10, 11 | Exists (`GillespieGpu`) | Done |
| Bio ODEs (Bistable, Cooperation, etc.) | 9, 10, 11 | **NEW (S58)** — 5 ODE systems absorbed | Done |
| NMF (Euclidean + KL) | 20, 21 | **NEW (S58)** — `linalg::nmf` | Done |
| Anderson 1D/2D/3D + disordered_laplacian | 15-18 | Exists + expanded (S56) | Done |
| Lanczos eigensolve (GPU) | 15-18 | Exists (`spectral`) | Done |
| Smith-Waterman (GPU) | 20, 21 | Exists (`SmithWatermanGpu`) | Done |
| Bray-Curtis (GPU) | 20, 21 | Exists (`BrayCurtisF64`) | Done |
| Hill kinetics (`hill`, `hill_repress`) | 10, 11 | **ACTIVE** — `barracuda::stats::hill` delegated (S68); `hill_repress` = `1.0 - hill()` | Done |
| Eigenvector solver (tridiag QL) | 17 | **Gap** — eigenvalues only (Sturm); eigenvectors CPU-only | MEDIUM |

### GPU-Ready vs GPU-Blocked

**GPU-Ready** (barracuda primitives already exist):
Papers 6, 7, 8, 9, 10, 11, 12, 14, 15, 16, 18, 21 — can proceed to GPU tier once CPU tier completes.

**GPU-Blocked** (missing barracuda primitives):
- Papers 1-5 — blocked by `gpu` feature gate (ops exist but need GPU adapter)
- Papers 4, 20-21 — blocked by **batched multinomial** Tier C absorption

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

### Tier 1: BarraCUDA CPU (current — 292/292 PASS)

Pure safe Rust with optional `barracuda` feature gate delegation.
57 active delegations (38 CPU + 19 GPU), 1 evolution candidate — ToadStool S70+++. 11.5× faster than Python (excl. LAPACK-bound).
569 Rust workspace tests + 375 Python. 28/28 mathematical parity proven. 95+ three-tier parity tests (100% delegation coverage).
All 28 experiments validated. GPU stats dispatch (mean, std_dev, rmse, mbe, pearson_r). 9 CPU vs GPU parity tests. CPU vs GPU benchmark binary.

### Tier 2: BarraCUDA GPU (in progress — 19 GPU dispatch targets)

GPU adapter wiring for existing barracuda ops + Tier C shader absorption.
New batch APIs: `birth_death_ssa_batch` (GillespieGpu), `wright_fisher_fixation_batch` (WrightFisherGpu),
`daily_et0_batch` (BatchedElementwiseF64). All produce correct results matching CPU baselines.

| Category | Papers | Barracuda Op | Action |
|----------|--------|-------------|--------|
| Tier A adapt | 1-5 | `FusedMapReduceF64`, `NormReduceF64` | Wire `gpu` feature + adapter |
| Tier B align | 5, all | `PrngXoshiro`, grid dispatch | Regenerate baselines with xoshiro |
| Tier C absorb | 4, 20-21 | `batched_multinomial` (new) | ToadStool absorbs metalForge WGSL |
| Tier C absorb | 12-13 | `rawr_mean` | **ABSORBED** — `stats::rawr_mean` (S66) |
| **GPU-wired** | 6 | `GillespieGpu` | **DONE** — `birth_death_ssa_batch` batch API |
| **GPU-wired** | 14 | `WrightFisherGpu` | **DONE** — `wright_fisher_fixation_batch` buffer dispatch |
| **GPU-wired** | 3, 22 | `BatchedElementwiseF64` | **DONE** — `daily_et0_batch` GPU shader |
| GPU-ready | 15 | `spectral::*` | Previously wired (anderson, almost_mathieu, band_structure) |

### Tier 3: metalForge Cross-Substrate (future)

Mixed hardware dispatch using metalForge forge crate. Each experiment
validated across CPU, GPU, and potentially NPU substrates.

| Validation | Description |
|-----------|-------------|
| CPU ↔ GPU parity | GPU output matches CPU within documented tolerance |
| Cross-vendor parity | RTX 4070 vs other GPUs produce identical physics |
| Mixed dispatch | metalForge routes to best substrate per operation |
| f32 ↔ f64 drift | Sub-thesis 07: quantify precision loss on consumer GPU |

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
