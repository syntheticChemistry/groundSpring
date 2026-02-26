# Emily Dolson — Eco-Evolutionary Noise Threshold

**Faculty**: Emily Dolson (Computer Science & Engineering, Michigan State University)
**Domain**: Eco-evolutionary dynamics, digital evolution, origin of life
**groundSpring Experiment**: Exp 017 (Quasispecies Threshold)
**Status**: Phase 0 (9/9 Py) + Phase 1 (6/6 Rust) PASS

---

## Papers

| Paper | Journal | Year | groundSpring Exp |
|-------|---------|------|:----------------:|
| Dolson, Banzhaf, Ofria "The ecology-evolution continuum and the origin of life" | J R Soc Interface 20(208) | 2023 | **Exp 017** |

## Connection to groundSpring

Dolson's work asks where **signal begins in a system that starts as pure noise**.
This is the philosophical complement to groundSpring's operational question
(how do we distinguish signal from noise in measurements). Eigen's quasispecies
model provides a quantitative answer: below the error threshold μ_c, genetic
information persists (signal); above it, information collapses (pure noise).

**Exp 017 validates the baseline Eigen model** before layering in the
eco-evolutionary complexity that Dolson et al. (2023) explore.

## Key Results

| Check | Value | Status |
|-------|-------|--------|
| Error threshold μ_c | 0.02276 (analytical: 1 − σ^(−1/L)) | PASS |
| Master frequency below threshold | x_m ≈ 0.42 at μ=0.010 | PASS |
| Information collapse above threshold | x_m ≈ 0 at μ=0.040 | PASS |
| Phase transition sharpness | Monotonic decay across sweep | PASS |
| Simulation-analytical agreement | Within 10% below threshold | PASS |

## BarraCUDA Needs

- **Wright-Fisher resampling**: Multinomial sampling (already in barracuda)
- **Per-locus mutation**: Embarrassingly parallel across population — ideal GPU
- **Error threshold scan**: Independent per mutation rate — trivially parallel

## Cross-Spring Links

| Spring | Connection |
|--------|-----------|
| wetSpring | Mutation-selection balance in microbial populations (Track 1c Anderson) |
| hotSpring | Phase transitions in lattice systems (deconfinement ↔ error threshold) |
| neuralSpring | Evolutionary algorithms use mutation — error threshold bounds exploration |

## Evolution Path

```
Python baseline (Exp 017)
  → Rust CPU validation (6/6 PASS)
    → BarraCUDA CPU (Wright-Fisher + mutation → barracuda multinomial)
      → BarraCUDA GPU (population-parallel mutation scan)
        → metalForge (mixed substrate for large populations)
```
