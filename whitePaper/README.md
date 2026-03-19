# groundSpring White Paper

## The Dirty Differences: Characterizing Measurement Noise Across Scientific Domains

### Purpose

This white paper documents groundSpring's systematic approach to quantifying the gap between what models predict and what instruments actually measure. Where airSpring validates clean FAO-56 equations and wetSpring validates taxonomy pipelines, groundSpring asks: **"how confident are we in these numbers?"**

### Status

- Phase 0 baselines: **~375 quantitative checks passed** across 29 experiments, 9 domains.
- Phase 1 Rust validation: **395/395 checks passed** across 34 validation binaries (340 core + 55 NUCLEUS).
- Mathematical parity: **29/29 PROVEN** (Python ⇌ Rust against shared benchmark JSONs).
- Performance: **11.6× faster** (Rust vs Python, excl. LAPACK-bound); 5.1× overall. Exp 009: **47.4× from Sturm tridiag**.
- V118: 110 delegations (67 CPU + 43 GPU), March 19, 2026.
- V117: All-features compilation fixed (tarpc-ipc), `cargo deny` modernised and passing, PRNG `DefaultRng` feature-gated for GPU alignment, `validate_all` meta-binary (29/29 PASS), clippy `--all-features` clean, bare literal cleanup. 960+ tests.
- V116: Typed error evolution — `DispatchError`, `EsnError`, `ResilienceError<E>`; `ValidationSink` trait; Format C/D capability parsing; `OnceLock` GPU probe cache; RAWR extracted to `rawr.rs`; dispatch defaults named with provenance.
- V115: API evolution (`assert!` → `Result`), CI hardening, ecoBin compliance. V113: GemmF64 transpose, RetryPolicy + CircuitBreaker.
- V110: Cross-ecosystem absorption — `#[expect(reason)]` migration (95 files, zero `#[allow()]`), `control/tolerances.py` (full Python tolerance mirror, 28 constants), structured tracing in primal binary, toadStool `compute.dispatch.*` direct dispatch (3 methods), dual-format capability parsing (neuralSpring S156 compat), `deny.toml`, aarch64 cross-compile CI. 912+ tests, 0 clippy, 0 fmt diff.
- V110: Zero-panic validation binaries (28 converted to Result-based), smart module refactoring (regression/fao56/pipeline/validate-lib), named physical constants, Python deps pinned.
- V108: License corrected to AGPL-3.0-or-later (SCYBORG trio), barracuda WelfordState CPU delegation, tolerance centralization, typed capability-based discovery, Python provenance enrichment.
- V107: Release profile, enriched niche.rs, tolerance provenance citations, bare literal elimination, feature-gated spectral constants.
- V106: primal_names module (wetSpring V119 pattern), typed BiomeOsError enum, zero hardcoded primal strings.
- V105: Panic-free production (`#![deny(expect_used, unwrap_used)]`), freeze_out 4-module refactor, typed IPC client.
- V103: Deep debt audit — named constants with provenance, `biomeos/interaction` extraction, `eps::LOG_FLOOR` and `TOL_ET0` centralized, tissue-anderson thresholds documented (Paper 12), zero clippy (pedantic + nursery). 936 tests (all feature gates).
- V102: Niche deployment via biomeOS graph composition.
- V101: DRY evolution + capability-based discovery.
- V100: Deep debt audit — build-breaking path fix, silent fallback elimination, tolerance provenance, capability-based health. barraCuda v0.3.5, toadStool S130+, coralReef Iteration 10.
- V99: First live NUCLEUS connection — adaptive health probing, direct primal discovery, biomeOS protocol version handling.
- V98: Upstream rewire — barraCuda `a898dee`, toadStool S130+, coralReef Iteration 10. Three-tier parity intact.
- V97: GPU smoke test + three-tier parity proven: 29/29 validation binaries PASS at default CPU, barracuda-CPU, and barracuda-GPU tiers. 102 delegations (61 CPU + 41 GPU). 936 Rust tests. All quality gates pass.
- V95: coralReef push buffer breakthrough — sovereign GPU dispatch on Titan V.
- V94: Ecosystem sync + Shannon delegation. 3 large modules refactored (rarefaction, drift, tissue_anderson). FFT wired via `Fft1DF64`.
- V91: Complete ecosystem rewire. 100 delegations (59 CPU + 41 GPU). 807 Rust tests. 91.55% coverage. Zero TODO/FIXME/unsafe/unwrap in production. All files < 1000 lines. 21 benchmark workloads. Cross-spring shader evolution documented.

### Key Results

| Experiment | Domain | Phase 0 | Phase 1 (Rust) | Key Finding |
|------------|--------|---------|----------------|-------------|
| 001: Sensor Noise | Agricultural | 32/32 | 36/36 PASS | EC5 bias-dominated (77%); CS616 mixed noise structure |
| 002: Observation Gap | Meteorology | 5/5 | 13/13 PASS | Stats + hit rate validated on weather-domain data |
| 003: Error Propagation | ET₀ uncertainty | 8/8 | 15/15 PASS | Humidity dominates ET₀ variance (65%); MC/analytical agree |
| 004: Sequencing Noise | Microbiome | 16/16 | 15/15 PASS | Genus saturation at 5000 reads; Shannon converges by 500 |
| 005: Seismic Inversion | Geophysics | 10/10 | 9/9 PASS | Grid-search recovers source exactly; Vec alloc hoisted |
| 006: Signal Specificity | Biology (c-di-GMP) | 12/12 | 12/12 PASS | SNR ≈ 2 at 20× activation; Poisson variance confirmed |
| 007: RAWR Resampling | Statistics | 11/11 | 11/11 PASS | RAWR competitive or better than bootstrap across all test cases |
| 008: Anderson Localization | Math (spectral) | 8/8 | 8/8 PASS | Thouless scaling ξ ≈ 104/W²; all states localized for W > 0 |
| 009: Quasiperiodic | Math (spectral) | 8/8 | 8/8 PASS | Aubry-André transition at λ=2; Herman's formula confirmed |
| 010: Bistable Switching | Biology (c-di-GMP) | 10/10 | 10/10 PASS | Two stable attractors; noise-induced transitions |
| 011: Multi-Signal QS | Biology (QS) | 9/9 | 9/9 PASS | Dual signaling sharpens regulation; lower variance |
| 012: Spin Chain Transport | Math (spectral) | 18/18 | 18/18 PASS | Ballistic→localized transport transition (Kachkovskiy 2016) |
| 013: Resampling Convergence | Statistics | 10/10 | 8/8 PASS | Bootstrap/RAWR converge by ~2000 replicates (Lee & Liu 2024) |
| 014: Drift vs Selection | Evolutionary Bio | 7/7 | 7/7 PASS | N×s threshold: drift dominates at small N (R. Anderson 2022) |
| 015: Uncertainty Bridge | Cross-domain | 8/8 | 8/8 PASS | Sensor noise → Anderson ξ; CV(ξ) ranking preserved; bias correction minimal at saturated disorder |
| 016: Rare Biosphere | Microbial ecology | 11/11 | 12/12 PASS | Sequencing depth determines rare taxa signal boundary; Chao1 corrects undersampling; D*≈998 for rarest taxa |
| 017: Quasispecies Threshold | Evolutionary dynamics | 9/9 | 6/6 PASS | Eigen's error threshold μ_c≈0.023 predicts mutation-driven information collapse; sharp phase transition |
| 018: Band Edge Structure | Mathematical physics | 8/8 | 10/10 PASS | Transfer matrix reproduces tight-binding band-gap structure; period-p potential → p bands |
| 019: Jackknife Estimation | Statistics | 9/9 | 9/9 PASS | Delete-one jackknife variance, bias correction, block jackknife; extends Exp 007 RAWR |
| 020: Freeze-Out Inverse | Inverse problems | 8/8 | 8/8 PASS | Chi-squared grid-search recovers T0, κ₂ from noisy observables; extends Exp 005 seismic |
| 021: Spectral Recon | Inverse problems | 8/8 | 8/8 PASS | Tikhonov-regularized spectral reconstruction from noisy correlator; most advanced inverse |
| 022: ET₀ → Anderson | Cross-spring | 7/7 | 7/7 PASS | Humidity-dominated ET₀ error → localization length CV; ξ_CV/ET₀_CV ≥ 0.5 |
| 023: No-Till Sampling | Cross-spring | 7/7 | 7/7 PASS | No-till saturates later (~1500 reads) vs tilled (~800); higher diversity demands deeper sampling |
| 024: Aggregate Stability | Cross-spring | 8/8 | 8/8 PASS | WSA bias-variance decomposition distinguishes tilled vs no-till Anderson regimes |
| 025: f32/f64 Drift | WDM | 7/7 | 7/7 PASS | f32→f64 Green-Kubo error: bias fraction ~28%, systematic not random |
| 026: Size Convergence | WDM | 7/7 | 7/7 PASS | D(N) = D∞ + α/N^(1/d); R² > 0.999; consumer GPU converges by N=10k |
| 027: Vendor Parity | WDM | 7/7 | 7/7 PASS | Vendor differences at 1e-12 relative level; correlation 1.000000 |
| 028: NPU Anderson | Hardware | 7/7 | 9/9 PASS | int8 DMA classification on AKD1000 at ~51 µs/inference |
| 029: Real GHCND ET₀ | NOAA | — | 6/6 PASS | Hargreaves vs Penman-Monteith on real weather via NestGate |
| 030: Real NCBI 16S | NCBI | — | 9/9 PASS | Rare biosphere detection on real 16S metagenomes |
| 031: NUCLEUS Stack | Infrastructure | — | 28/28 PASS | Full NUCLEUS primal validation: Tower + Node + Squirrel + Nest |
| 032: IRIS Seismic | IRIS FDSN | — | 12/12 PASS | IRIS station geometry + travel times via NestGate |
| 033: Tissue Anderson | Immunological | — | 29/29 PASS | Cytokine Anderson lattice + geometry-aware drug scoring (Paper 12) |

### Key Research Questions Answered

1. **How much sensor error is correctable?** 50-80% of total soil moisture sensor error is systematic bias that can be removed with site-specific calibration (Exp 001).

2. **Which measurement matters most for ET0?** Humidity sensor accuracy dominates ET0 uncertainty (66% of variance), followed by radiation (20%) and temperature (10%) (Exp 003).

3. **When does more sequencing stop helping?** Above 5000 reads, genus discovery yields diminishing returns. Shannon diversity stabilizes by 500 reads (Exp 004).

4. **How does noise propagate through an inverse problem?** ±0.5s arrival time noise produces ~2km horizontal location uncertainty but ~8.5km depth uncertainty — the classic tradeoff between well-constrained and poorly-constrained parameters (Exp 005).

### Documents

- [STUDY.md](STUDY.md) — Detailed results and analysis
- [METHODOLOGY.md](METHODOLOGY.md) — Experimental design and validation approach
- [experiments/](experiments/) — Per-experiment summaries (34 experiments, 10 domains)
- [baseCamp/](baseCamp/) — Per-faculty research briefings (Bazavov, Waters, Liu, Kachkovskiy, R. Anderson, Dolson, Gonzales)
- [../wateringHole/CROSS_SPRING_SHADER_EVOLUTION.md](../wateringHole/CROSS_SPRING_SHADER_EVOLUTION.md) — Cross-spring shader provenance (S58–S93)
- [../specs/BARRACUDA_EVOLUTION.md](../specs/BARRACUDA_EVOLUTION.md) — Module → GPU promotion mapping

---

## Phase 1 Rust Library

The `groundspring` crate provides 40 modules of pure safe Rust:

| Module | Experiment | GPU Tier | Notes |
|--------|-----------|----------|-------|
| `stats` | All | 22 CPU delegated | Pearson, Spearman, std_dev, covariance, norm_cdf, norm_ppf, chi2, rmse, mbe, r², IoA, hit_rate, mean, percentile → barracuda |
| `decompose` | Exp 001 | CPU-only | Bias-variance decomposition, scalar math |
| `fao56` | Exp 003 | **Absorbed** upstream | Equation chain → barracuda `Op::Fao56Et0`; MC wrapper pending |
| `prng` | Exp 003, 004 | B (adapt) | Xorshift64 + Box-Muller, aligning to barracuda xoshiro |
| `rarefaction` | Exp 004 | C (WGSL ready) | Batched multinomial shader production-quality |
| `seismic` | Exp 005 | **GPU-ready** (V31) | Haversine, travel time, grid-search inversion |
| `gillespie` | Exp 006 | Pending (batch API) | Gillespie SSA birth-death process (serial per-trajectory) |
| `bootstrap` | Exp 007 | A Lean | Bootstrap + RAWR CIs → `barracuda::stats::bootstrap_mean` |
| `anderson` | Exp 008-009 | A Lean | Lyapunov, level_spacing, eigenvalues → `barracuda::spectral`, analytical ξ → `barracuda::special`. **49.5× Exp 009.** |
| `almost_mathieu` | Exp 009, 012 | A Lean | Almost-Mathieu Hamiltonian, flat QR eigenvalues, level spacing |
| `transport` | Exp 012 | B (adapt) | Tridiag eigenvector solver (implicit QL), wavepacket MSD — flat buffers |
| `drift` | Exp 014 | B (adapt) | Wright-Fisher fixation, Kimura probability, neutral diversity |
| `bistable` | Exp 010 | A Lean | Bistable ODE derivative → `barracuda::numerical::ode_bio` |
| `multisignal` | Exp 011 | A Lean | Multi-signal ODE derivative → `barracuda::numerical::ode_bio` |
| `kinetics` | Exp 010-011 | A Lean | Hill functions — shared by bistable + multisignal, barracuda delegation |
| `cast` | All | N/A | Centralized numeric casts with documented safety |
| `validate` | All | N/A | Generic `Write` harness (hotSpring pattern) |
| `rare_biosphere` | Exp 016 | **GPU-ready** (V31) | Chao1, detection power/threshold, abundance-occupancy |
| `quasispecies` | Exp 017 | **GPU-ready** (V31) | Error threshold, master frequency, Wright-Fisher mutation sim |
| `band_structure` | Exp 018 | **GPU-ready** (V31) | Transfer matrix, band edges, periodic Hamiltonian, eigenvalue fraction |
| `jackknife` | Exp 019 | CPU delegated | Delete-one jackknife, block jackknife, bias correction |
| `freeze_out` | Exp 020 | **GPU-ready** (V31) | Freeze-out curve, chi-squared, 2D grid-search inverse |
| `spectral_recon` | Exp 021 | GPU delegated | Tikhonov regularization (tikhonov_solve → barracuda linalg) |
| `primal_names` | Infrastructure | N/A | Centralized primal name constants (wetSpring V119 pattern); zero hardcoded strings |

### GPU Evolution (metalForge)

Two local WGSL shaders in `metalForge/shaders/` (unique to groundSpring, no ToadStool equivalent):

1. **`anderson_lyapunov.wgsl`** (~80 lines) — f64 Lyapunov exponent computation for
   Anderson localization experiments. Unique to groundSpring spectral theory.
2. **`anderson_lyapunov_f32.wgsl`** (~80 lines) — f32 fallback for NAK/NVVM pipelines.

**Absorbed into ToadStool (removed V62)**: `mc_et0_propagate.wgsl` (→ S72
`McEt0PropagateGpu`), `batched_multinomial.wgsl` (→ S76 `BatchedMultinomialGpu`).
See `metalForge/ABSORPTION_MANIFEST.md` for the full inventory.

## Next Phase: Paper Review Candidates

groundSpring asks "how confident are we in these numbers?" The faculty network reveals four professors whose work directly extends this question into new domains: **Alexei Bazavov** (inverse problems in lattice QCD), **Christopher Waters** (signal specificity in biological systems), **Kevin Liu** (statistical resampling for phylogenetic confidence), and **Ilya Kachkovskiy** (Anderson localization — the mathematical theory of when signal propagates vs. when noise wins).

### Inverse Problems & Spectral Reconstruction (Bazavov)

groundSpring Exp 005 (seismic inversion) is an inverse problem — inferring source location from noisy arrival times. Bazavov's lattice QCD work contains a rich set of related inverse problems at much higher precision requirements.

| Priority | Paper | Why |
|----------|-------|-----|
| **Tier 1** | Bazavov et al. (2025) "Spectral reconstruction inverse problem in lattice QCD." arXiv 2501.12259 | Spectral reconstruction = signal recovery from incomplete/noisy data. Direct generalization of Exp 005's seismic inversion. The lattice QCD version demands subpercent precision |
| **Tier 2** | Bazavov et al. (2025) "Hadronic vacuum polarization for the muon g-2." Phys Rev D 111, 094508 | Subpercent precision from noisy lattice data via jackknife/bootstrap error estimation. groundSpring's Monte Carlo error propagation (Exp 003) is a simplified version of the same methodology |
| **Tier 2** | Bazavov et al. (2016) "Curvature of the freeze-out line in heavy ion collisions." Phys Rev D 93, 014512 | Inverse problem — inferring freeze-out conditions from experimental observables. Different physics, same mathematical structure as seismic inversion |

### Signal vs Noise in Biological Systems (Waters)

groundSpring Exp 001 decomposes sensor noise into bias + random. Waters' c-di-GMP work poses the same question inside a living cell: how does a bacterium resolve signal from noise when 60+ enzymes control a single diffusible molecule?

| Priority | Paper | Why |
|----------|-------|-----|
| **Tier 1** | Massie et al. (2012) "Quantification of High Specificity Cyclic di-GMP Signaling." PNAS 109:12746-51 | Quantitative signal specificity in a noisy intracellular environment. How do cells achieve high-specificity signaling with a shared, diffusible molecule? This is the biological version of Exp 001's signal decomposition |
| **Tier 2** | Fernandez et al. (2020) "V. cholerae adapts to sessile and motile lifestyles by c-di-GMP regulation of cell shape." PNAS 117:29046-29054 | Phenotypic switching as a bistable dynamical system. Bifurcation analysis — when does noise push a system across a threshold? groundSpring quantifies *how much* noise; this paper shows *what happens* when noise exceeds a critical level |
| **Tier 2** | Srivastava et al. (2011) "Integration of Cyclic di-GMP and Quorum Sensing in the Control of vpsT and aphA." J Bacteriology 193:6331-41 | Multi-input regulatory network integration. How do cells combine multiple noisy signals? The biological analog of sensor fusion |

### Statistical Confidence & Resampling (Liu)

groundSpring's Monte Carlo error propagation (Exp 003) uses N=10,000 random draws. Liu's phylogenetic work develops much more sophisticated resampling methods for confidence estimation on noisy data.

| Priority | Paper | Why |
|----------|-------|-----|
| **Tier 1** | Wang et al. (2021) "Build a better bootstrap and the RAWR shall beat a random path to your door." Bioinformatics (ISMB) 37:i111-i119 | RAWR resampling: modern bootstrap methods for confidence estimation on structured data. groundSpring's MC is naive bootstrap; RAWR is weighted resampling that's faster and more accurate |
| **Tier 2** | Lee & Liu (2024) "A Statistical Optimization Technique to Inform Statistical Resampling Assessments." IEEE BIBM 2024 | Optimizing resampling strategy itself — meta-statistical methods. Could improve groundSpring's error propagation efficiency |

### Eco-Evolutionary Noise (Dolson)

| Priority | Paper | Why |
|----------|-------|-----|
| **Tier 2** | Dolson et al. (2023) "The ecology-evolution continuum and the origin of life." J R Soc Interface 20(208) | Emergence of organization from chemical noise — where does signal begin in a system that starts as pure noise? Philosophical extension of groundSpring's noise decomposition to the origin-of-life context |

### Cross-Domain Connection: groundSpring's Five Pillars Extended

| Pillar | Current Exp | Faculty Extension |
|--------|-------------|-------------------|
| Signal vs Noise | Exp 001 (sensors) | Waters: biological signal specificity in noisy cells |
| Inverse Problems | Exp 005 (seismic) | Bazavov: spectral reconstruction in lattice QCD |
| Sensing Systems | Exp 002 (ERA5 vs station) | Waters: quorum sensing = biological sensor network |
| Temporal Dynamics | Exp 004 (sequencing depth) | Liu: phylogenetic confidence over evolutionary time |
| Spatial Propagation | Exp 005 (wave propagation) | Bazavov: gauge field propagation on lattices |
| ALL PILLARS | Exps 001-005 | Kachkovskiy: Anderson localization = mathematical theory of signal vs noise |

### Anderson Localization & Spectral Theory (Kachkovskiy)

Ilya Kachkovskiy (Math, MSU — previously IAS; co-author with Fields Medalist Jean Bourgain) provides the rigorous mathematical formalization of groundSpring's central question. His work on Anderson localization proves when waves propagate through disordered media vs. when noise traps them — the theorem behind all five pillars.

| Priority | Paper | Why |
|----------|-------|-----|
| **Tier 1** | Bourgain & Kachkovskiy (2018) "Anderson localization for two interacting quasiperiodic particles." GAFA 29:3-43 | Two coupled noisy sensors: when does interaction help vs. hurt signal recovery? Direct extension of Exp 001 |
| **Tier 1** | Jitomirskaya & Kachkovskiy (2018) "All couplings localization for quasiperiodic operators." JEMS 21:777-795 | Quasiperiodic disorder = structured noise (seasonal patterns, tidal drift). Localization at all coupling strengths |
| **Tier 2** | Kachkovskiy (2016) "On transport properties of isotropic quasiperiodic XY spin chains." CMP 345:659-673 | When does a signal propagate through a disordered chain? Framework for Exp 005's seismic wave propagation |
| **Tier 2** | Filonov & Kachkovskiy (2018) "On the structure of band edges of 2d periodic elliptic operators." Acta Math 221:59-80 | Band edges = the frequency boundary between signal propagation and noise domination |

### BarraCUDA Kernel Needs for Extensions

| Domain | Required Primitives | Status |
|--------|-------------------|--------|
| Spectral reconstruction | FFT, regularization, matrix inverse | **Gap**: FFT not yet in BarraCUDA |
| Stochastic simulation | Gillespie algorithm, PRNG | **CPU complete** (Exp 006); GPU via `GillespieGpu` |
| Bifurcation analysis | Eigenvalue computation, continuation | `BatchedEighGpu` handles eigenvalues |
| Bootstrap/resampling | Parallel resampling, weighted draws | **CPU complete** (Exp 007); GPU embarrassingly parallel |
| Monte Carlo | Already validated in Exp 003 | Extend to jackknife, bootstrap-t |
| Anderson localization | Transfer matrix, Lyapunov exponents | **CPU complete** (Exp 008); GPU via `barracuda::spectral::lyapunov_*` |
