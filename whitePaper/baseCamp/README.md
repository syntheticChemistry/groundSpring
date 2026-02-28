# groundSpring baseCamp — Faculty Research Briefings

**Purpose**: Per-faculty research briefings for groundSpring noise characterization
reproductions and extensions.

**Last Updated**: February 28, 2026

**Validation Summary**: 292/292 checks (+ 49 metalForge + 13 metalForge validation + 9 NestGate NCBI + 15 NUCLEUS pipeline), **46 active delegations + 7 pending ToadStool (37 CPU + 9 GPU)**, 490+ workspace Rust tests (barracuda-gpu) + 320 Python tests. All 28 experiments DONE. **11.5× faster** than Python (excl. LAPACK-bound). **28/28 mathematical parity proven**. 44 three-tier parity tests, 6 live NUCLEUS integration tests, zero clippy/doc warnings. V46: idiomatic Rust evolution — `stats::agreement` domain split (R²/NSE deduplicated), `.windows(3).fold()` iterator modernization, `NESTGATE_DEFAULT_PORT` constant, full codebase audit (0 unsafe, 0 mocks, 0 unwrap in library). V45: validation gap closure (+4 checks → 292/292). V44: deep-debt evolution — `linalg` module, `InputError` typed errors, 5 APIs `assert!` → `Result`, capability-based discovery. V43: three-tier parity proven (27/27), pure GPU workloads (26/26). ToadStool S68+: 700 WGSL, zero f32-only, dual-layer DF64. 9 domains, 30 modules.

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
| Exp 002 + 008 + 015 | Cross-spring → **Exp 022** (ET₀→Anderson propagation) | airSpring (ET₀) → groundSpring (ξ) |
| Exp 004 + 016 | Cross-spring → **Exp 023** (no-till vs tilled 16S) | wetSpring (16S pipeline) |
| Exp 008 + 015 | Cross-spring → **Exp 024** (aggregate stability noise) | airSpring (soil structure) |
| Exp 019-021 | WDM → **Exp 025** (f32/f64 drift), **Exp 026** (size convergence), **Exp 027** (vendor parity) | hotSpring (WDM simulation) |
| Exp 008 + 015 | metalForge → **Exp 028** (NPU Anderson classification) | hotSpring (Akida driver), airSpring (edge IoT) |

## Sub-Theses

groundSpring contributes to several Gen3 cross-spring sub-theses:

| Sub-thesis | groundSpring Role |
|-----------|-------------------|
| 06: No-Till Soil Health | Uncertainty budget: sensor noise → QS regime prediction confidence. Exp 022-024 cross-spring validation |
| 07: Sovereign WDM | Inverse problem math (Exp 019-021), WDM uncertainty budget (Exp 025-027: f32/f64 drift, size convergence, vendor parity) |
| 01: Anderson-QS | Error propagation through Anderson localization geometry |
| 05: Cross-Species Signal | Noise floor for detecting cross-species QS signal |
