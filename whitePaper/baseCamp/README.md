# groundSpring baseCamp — Faculty Research Briefings

**Purpose**: Per-faculty research briefings for groundSpring noise characterization
reproductions and extensions.

**Last Updated**: March 7, 2026

**Validation Summary**: V93 — 395/395 validation checks (340 core + 55 NUCLEUS) + 187 metalForge checks, **101 active delegations (60 CPU + 41 GPU) — barraCuda v0.3.3, toadStool S128, coralReef Phase 9**. 903 workspace tests + 261 Python provenance tests. All 35 experiments PASS. **29/29 mathematical parity proven**. 3 large modules smart-refactored (rarefaction, drift, tissue_anderson). FFT wired into spectral_recon via `Fft1DF64`. Zero clippy warnings (pedantic + nursery), zero unsafe, zero TODO, zero `.unwrap()` in production, all files < 1000 lines. 21 benchmark workloads. Cross-spring shader evolution documented. 10 domains, 34 modules.

---

## Faculty Network

| Faculty | Institution | Domain | groundSpring Connection |
|---------|------------|--------|------------------------|
| Alexei Bazavov | CMSE + Physics, MSU | Lattice QCD, inverse problems | **Exp 019** (jackknife 9/9), **Exp 020** (freeze-out 8/8), **Exp 021** (spectral recon 8/8) |
| Christopher Waters | MMG, MSU | Quorum sensing, c-di-GMP | **Exp 006** (30.5×), **Exp 010** (18.5×), **Exp 011** (46.2×) |
| Kevin Liu | CMSE, MSU | Phylogenetics, statistical resampling | **Exp 007**: RAWR bootstrap (11/11 PASS, 7.3× faster) |
| Ilya Kachkovskiy | Math, MSU | Anderson localization, spectral theory | **Exp 008** (29.9×), **Exp 009** (49.5× Sturm), **Exp 012** (spin chain), **Exp 018** (band edge 10/10) |
| Rika Anderson | Biology, Carleton College | Deep subsurface microbiology | **Exp 014** (drift/selection), **Exp 016** (rare biosphere 10/10) |
| Emily Dolson | CSE, MSU | Eco-evolutionary dynamics | **Exp 017**: Quasispecies threshold (6/6 Rust) |
| Andrea J. Gonzales | Pharmacology & Toxicology, MSU | Immunopharmacology, JAK/cytokine signaling | **Paper 12**: Exp 008 (2D/3D Anderson), Exp 012 (transport), Exp 015 (uncertainty), Exp 018 (band edge) → cytokine propagation |

## Validation Chain

Following the Write → Absorb → Lean cycle:

```
Python baseline → BarraCUDA CPU → BarraCUDA GPU → metalForge cross-substrate
```

Each faculty extension paper is validated at three tiers:

| Tier | Substrate | Validation |
|------|-----------|-----------|
| 1: CPU | `cargo test` + validation binary | Rust matches Python baseline |
| 2: GPU | `barracuda` feature + GPU adapter | GPU matches CPU within tolerance |
| 3: metalForge | Mixed hardware dispatch | Cross-substrate agreement |

## Faculty Briefings

- [bazavov.md](bazavov.md) — Inverse Problems & Spectral Reconstruction
- [waters.md](waters.md) — Biological Signal Specificity
- [liu.md](liu.md) — Statistical Resampling & Confidence
- [kachkovskiy.md](kachkovskiy.md) — Anderson Localization & Spectral Theory
- [anderson.md](anderson.md) — Eco-Evolutionary Noise (R. Anderson, Carleton)
- [dolson.md](dolson.md) — Eco-Evolutionary Noise Threshold
- [gonzales.md](gonzales.md) — Immunological Anderson & Drug Geometry (Paper 12)

## Cross-Spring Impact

| groundSpring Experiment | Faculty Extension | Cross-Spring |
|------------------------|-------------------|-------------|
| Exp 001 (sensor noise) | Waters → **Exp 006** (signal specificity) | wetSpring (bio sensing) |
| Exp 003 (error propagation) | Liu → **Exp 007** (RAWR bootstrap) | neuralSpring (confidence) |
| Exp 005 (seismic inversion) | Bazavov → **Exp 019** (jackknife), **Exp 020** (freeze-out), **Exp 021** (spectral recon) | hotSpring (lattice QCD) |
| All 34 experiments | Kachkovskiy → **Exp 008** + **Exp 009** + **Exp 015** (uncertainty bridge) | hotSpring (spectral theory) |
| Exp 009 (quasiperiodic) | Kachkovskiy → **Almost-Mathieu** (Aubry-André) | hotSpring (spectral theory) |
| Exp 001 + 006 | Waters → **Exp 010** (bistable switching) | wetSpring (QS bifurcation) |
| Exp 006 + 010 | Waters → **Exp 011** (multi-signal QS) | wetSpring (dual-signal integration) |
| Exp 014 (drift selection) | R. Anderson → **Exp 016** (rare biosphere) | wetSpring (rare taxa) |
| Exp 001 + 004 | R. Anderson (drift vs selection) | wetSpring (rare biosphere) |
| Exp 008 + 009 | Kachkovskiy → **Exp 018** (band edge) | hotSpring (spectral) |
| Exp 002 + 008 + 015 | Cross-spring → **Exp 022** (ET₀→Anderson propagation) | airSpring (ET₀) → groundSpring (ξ) |
| Exp 004 + 016 | Cross-spring → **Exp 023** (no-till vs tilled 16S) | wetSpring (16S pipeline) |
| Exp 008 + 015 | Cross-spring → **Exp 024** (aggregate stability noise) | airSpring (soil structure) |
| Exp 019-021 | WDM → **Exp 025** (f32/f64 drift), **Exp 026** (size convergence), **Exp 027** (vendor parity) | hotSpring (WDM simulation) |
| Exp 008 + 015 | metalForge → **Exp 028** (NPU Anderson classification) | hotSpring (Akida driver), airSpring (edge IoT) |
| Exp 008 + 012 + 015 + 018 | Gonzales → **Paper 12** (immunological Anderson) | wetSpring (Anderson spectral Exp270-274), neuralSpring (ESN/LSTM) |

## Sub-Theses

groundSpring contributes to several Gen3 cross-spring sub-theses:

| Sub-thesis | groundSpring Role |
|-----------|-------------------|
| 01: Anderson-QS | Error propagation through Anderson localization geometry |
| 05: Cross-Species Signal | Noise floor for detecting cross-species QS signal |
| 06: No-Till Soil Health | Uncertainty budget: sensor noise → QS regime prediction confidence. Exp 022-024 cross-spring validation |
| 07: Sovereign WDM | Inverse problem math (Exp 019-021), WDM uncertainty budget (Exp 025-027: f32/f64 drift, size convergence, vendor parity) |
| 12: Immunological Anderson | 2D/3D spectral diagnostics (Exp 008), cytokine transport modeling (Exp 012), uncertainty bridge for cytokine measurement (Exp 015), band edge for epidermal periodicity (Exp 018). `ConceptEdge` for AD flare detection, `DriftAction` for treatment response steering. Dimensional promotion–collapse duality with Paper 06 |
