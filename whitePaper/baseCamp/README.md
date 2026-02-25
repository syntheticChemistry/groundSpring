# groundSpring baseCamp — Faculty Research Briefings

**Purpose**: Per-faculty research briefings for groundSpring noise characterization
reproductions and extensions.

**Last Updated**: February 25, 2026

---

## Faculty Network

| Faculty | Institution | Domain | groundSpring Connection |
|---------|------------|--------|------------------------|
| Alexei Bazavov | CMSE + Physics, MSU | Lattice QCD, inverse problems | Spectral reconstruction (Exp 005 generalization) |
| Christopher Waters | MMG, MSU | Quorum sensing, c-di-GMP | Biological signal specificity (Exp 001 analog) |
| Kevin Liu | CMSE, MSU | Phylogenetics, statistical resampling | RAWR bootstrap (Exp 003 upgrade) |
| Ilya Kachkovskiy | Math, MSU | Anderson localization, spectral theory | Mathematical foundation for all 5 pillars |
| Rika Anderson | Biology, Carleton College | Deep subsurface microbiology | Stochastic vs deterministic evolution (Exp 001 biological analog) |
| Emily Dolson | CSE, MSU | Eco-evolutionary dynamics | Origin-of-life noise (philosophical) |

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

## Cross-Spring Impact

| groundSpring Experiment | Faculty Extension | Cross-Spring |
|------------------------|-------------------|-------------|
| Exp 001 (sensor noise) | Waters (signal specificity) | wetSpring (bio sensing) |
| Exp 003 (error propagation) | Liu (RAWR bootstrap) | neuralSpring (confidence) |
| Exp 005 (seismic inversion) | Bazavov (spectral reconstruction) | hotSpring (lattice QCD) |
| All 5 pillars | Kachkovskiy (Anderson localization) | hotSpring (spectral theory) |
| Exp 001 + 004 | R. Anderson (drift vs selection) | wetSpring (rare biosphere) |

## Sub-Theses

groundSpring contributes to several Gen3 cross-spring sub-theses:

| Sub-thesis | groundSpring Role |
|-----------|-------------------|
| 06: No-Till Soil Health | Uncertainty budget: sensor noise → QS regime prediction confidence |
| 01: Anderson-QS | Error propagation through Anderson localization geometry |
| 05: Cross-Species Signal | Noise floor for detecting cross-species QS signal |
