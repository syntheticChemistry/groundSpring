# groundSpring baseCamp — Faculty Research Briefings

**Purpose**: Per-faculty research briefings for groundSpring noise characterization
reproductions and extensions.

**Last Updated**: February 27, 2026

**Validation Summary**: 288/288 checks (+ 49 metalForge), 32 active delegations + 9 pending ToadStool (25 CPU + 7 GPU), 442 Rust tests (biomeos) / 410 default + 320 Python tests = 762 total. All 28 experiments DONE. **11.5× faster** than Python (excl. LAPACK-bound). **28/28 mathematical parity proven**. V35: Titan V / NAK adaptive GPU dispatch, 19 metalForge workloads, 5 substrates, architecture-aware routing (f64→Titan V, f32→RTX 4070). V31: GPU dispatch wiring. V30: biomeOS Neural API. V26: metalForge live hardware (NPU DMA on AKD1000, RTX 4070, Titan V). 9 domains, 26 modules.

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

## Cross-Spring Impact

| groundSpring Experiment | Faculty Extension | Cross-Spring |
|------------------------|-------------------|-------------|
| Exp 001 (sensor noise) | Waters → **Exp 006** (signal specificity) | wetSpring (bio sensing) |
| Exp 003 (error propagation) | Liu → **Exp 007** (RAWR bootstrap) | neuralSpring (confidence) |
| Exp 005 (seismic inversion) | Bazavov → **Exp 019** (jackknife), **Exp 020** (freeze-out), **Exp 021** (spectral recon) | hotSpring (lattice QCD) |
| All 28 experiments | Kachkovskiy → **Exp 008** + **Exp 009** + **Exp 015** (uncertainty bridge) | hotSpring (spectral theory) |
| Exp 009 (quasiperiodic) | Kachkovskiy → **Almost-Mathieu** (Aubry-André) | hotSpring (spectral theory) |
| Exp 001 + 006 | Waters → **Exp 010** (bistable switching) | wetSpring (QS bifurcation) |
| Exp 006 + 010 | Waters → **Exp 011** (multi-signal QS) | wetSpring (dual-signal integration) |
| Exp 014 (drift selection) | R. Anderson → **Exp 016** (rare biosphere) | wetSpring (rare taxa) |
| Exp 001 + 004 | R. Anderson (drift vs selection) | wetSpring (rare biosphere) |
| Exp 008 + 009 | Kachkovskiy → **Exp 018** (band edge) | hotSpring (spectral) |

## Sub-Theses

groundSpring contributes to several Gen3 cross-spring sub-theses:

| Sub-thesis | groundSpring Role |
|-----------|-------------------|
| 06: No-Till Soil Health | Uncertainty budget: sensor noise → QS regime prediction confidence |
| 01: Anderson-QS | Error propagation through Anderson localization geometry |
| 05: Cross-Species Signal | Noise floor for detecting cross-species QS signal |
