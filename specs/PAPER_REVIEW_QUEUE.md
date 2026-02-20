# groundSpring — Paper Review Queue

**Last Updated**: February 12, 2026
**Purpose**: Track papers for reproduction/review, ordered by priority

---

## Completed Reproductions

| # | Experiment | Domain | Checks | Key Finding |
|---|-----------|--------|--------|-------------|
| 1 | Sensor noise decomposition | Agricultural sensors | 32/32 | EC5 bias-dominated (77%); CS616 mixed |
| 2 | Observation gap (ERA5 vs station) | Meteorology | 5/5 | Representation noise dominated |
| 3 | Error propagation FAO-56 | ET₀ uncertainty | 8/8 | Humidity dominates at 66% |
| 4 | Sequencing depth & taxonomic noise | Microbiome | 16/16 | Genus saturation at 5,000 reads |
| 5 | Seismic source inversion | Geophysics | 10/10 | ±2km horizontal, ±8.5km depth |

---

## Review Queue

### Inverse Problems & Spectral Reconstruction (Bazavov)

| # | Paper | Journal | Year | Faculty | Why | Status |
|---|-------|---------|------|---------|-----|--------|
| 6 | "Spectral reconstruction inverse problem in lattice QCD" | arXiv 2501.12259 | 2025 | Bazavov et al. | Signal recovery from incomplete/noisy data — direct generalization of Exp 005 seismic inversion, but at subpercent precision | Queued |
| 7 | "Hadronic vacuum polarization for the muon g-2" | Phys Rev D 111, 094508 | 2025 | Bazavov et al. | Jackknife/bootstrap error estimation at subpercent precision. Exp 003 MC propagation is a simplified version of this | Queued |
| 8 | "Curvature of the freeze-out line in heavy ion collisions" | Phys Rev D 93, 014512 | 2016 | Bazavov et al. | Inverse problem — inferring freeze-out conditions. Same math as seismic inversion, different physics | Queued |

### Biological Signal vs Noise (Waters)

| # | Paper | Journal | Year | Faculty | Why | Status |
|---|-------|---------|------|---------|-----|--------|
| 9 | Massie et al. "Quantification of High Specificity Cyclic di-GMP Signaling" | PNAS 109:12746-51 | 2012 | Waters | How cells resolve signal from noise with 60+ competing enzymes. Biological Exp 001 | Queued |
| 10 | Fernandez et al. "V. cholerae adapts by c-di-GMP regulation of cell shape" | PNAS 117:29046-29054 | 2020 | Waters | Bistable switching — when does noise push a system across a threshold? Bifurcation analysis | Queued |
| 11 | Srivastava et al. "Integration of Cyclic di-GMP and Quorum Sensing" | J Bacteriology 193:6331-41 | 2011 | Waters | Multi-input signal fusion in noisy environment. Biological analog of sensor fusion | Queued |

### Statistical Confidence & Resampling (Liu)

| # | Paper | Journal | Year | Faculty | Why | Status |
|---|-------|---------|------|---------|-----|--------|
| 12 | Wang et al. "Build a better bootstrap and the RAWR shall beat a random path" | Bioinformatics (ISMB) 37:i111-i119 | 2021 | Liu | RAWR: modern weighted resampling that outperforms naive bootstrap for structured data. Upgrade for Exp 003 | Queued |
| 13 | Lee & Liu "A Statistical Optimization Technique to Inform Statistical Resampling" | IEEE BIBM 2024 | 2024 | Liu | Meta-statistical optimization — improving the resampling strategy itself | Queued |

### Anderson Localization & Spectral Theory (Kachkovskiy)

Ilya Kachkovskiy (Math, MSU — previously IAS, UC Irvine; co-author with
Fields Medalist Jean Bourgain) studies when waves propagate vs. when disorder
traps them. This is the rigorous mathematical formalization of groundSpring's
central question: **when does signal propagate through a noisy system, and when
does noise win?**

| # | Paper | Journal | Year | Faculty | Why | Status |
|---|-------|---------|------|---------|-----|--------|
| 15 | Bourgain & Kachkovskiy "Anderson localization for two interacting quasiperiodic particles" | GAFA 29:3-43 | 2018 | Kachkovskiy | Anderson localization = signal trapped by disorder. Two-particle case models how coupled noisy sensors affect each other — directly extends Exp 001's correlated sensor noise decomposition | Queued |
| 16 | Jitomirskaya & Kachkovskiy "All couplings localization for quasiperiodic operators with Lipschitz monotone potentials" | JEMS 21:777-795 | 2018 | Kachkovskiy | Localization at ALL coupling strengths for monotone potentials. Quasiperiodic = "almost periodic" = structured noise (seasonal drift, tidal cycles, orbital harmonics). Math of Exp 002's ERA5 vs station gap | Queued |
| 17 | Kachkovskiy "On transport properties of isotropic quasiperiodic XY spin chains" | CMP 345:659-673 | 2016 | Kachkovskiy | Energy transport through disordered chains — when does a signal reach the other end? Mathematical framework for Exp 005's seismic wave propagation through heterogeneous crust | Queued |
| 18 | Filonov & Kachkovskiy "On the structure of band edges of 2d periodic elliptic operators" | Acta Math 221:59-80 | 2018 | Kachkovskiy | Band edges = frequencies where waves transition from propagating to evanescent. The mathematical boundary between "signal gets through" and "noise kills it" | Queued |

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

### Eco-Evolutionary Noise (Dolson)

| # | Paper | Journal | Year | Faculty | Why | Status |
|---|-------|---------|------|---------|-----|--------|
| 14 | Dolson et al. "The ecology-evolution continuum and the origin of life" | J R Soc Interface 20(208) | 2023 | Dolson | Where does signal begin in a system that starts as pure noise? Origin-of-life context | Queued |

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

## Notes

- Papers 6-8 (Bazavov) connect groundSpring to hotSpring via inverse problem methodology
- Papers 9-11 (Waters) connect groundSpring to wetSpring via biological noise/signal
- Papers 12-13 (Liu) upgrade groundSpring's statistical methodology
- Paper 14 (Dolson) is philosophical/theoretical — frames noise as the starting condition for life
- The common thread: **extracting reliable conclusions from noisy measurements**
