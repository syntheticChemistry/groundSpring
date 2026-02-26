# Exp 014: Drift vs Selection

**Domain**: Population genetics / evolutionary biology
**Paper**: Anderson (2022) mBio 13:e00354-22 — drift dominates in low-biomass habitats
**Question**: When does stochastic drift (noise) overwhelm deterministic selection (signal) in evolving populations?

## Data Source

Analytical: Wright-Fisher model with Kimura (1968) fixation probabilities.
Open theory — fully specified by population size N, selection coefficient s,
and initial allele frequency p₀.

## Method

Wright-Fisher simulation: N diploid individuals, binomial sampling each
generation. Allele A has fitness 1+s relative to allele a. Track fixation
probability over 500 trials across population sizes N ∈ {20, 50, 100, 500, 1000}.
Validate against Kimura's analytical formula P_fix = (1-e^{-4Nsp₀})/(1-e^{-4Ns}).

## Key Result

**N×s threshold correctly predicts drift vs selection dominance.**
- N=20 (N×s=0.2, DRIFT): P_fix = 0.580 ≈ neutral (Kimura 0.599)
- N=100 (N×s=1.0, transition): P_fix = 0.872 (Kimura 0.881)
- N=1000 (N×s=10.0, SELECTION): P_fix = 1.000 (selection wins completely)

Kimura formula accurate within 2% across all population sizes.
Shannon diversity decays 2.30 → 0.23 under drift at N=50 (200 generations)
vs 2.30 → 0.92 at N=500 — small populations lose diversity 4× faster.

**R. Anderson's insight confirmed**: in low-biomass (small N) habitats,
drift overwhelms selection. This is the biological N×s analog of
groundSpring's signal-to-noise ratio framework.

## Performance

| Metric | Python | Rust | Speedup |
|--------|--------|------|---------|
| Time | TBD | 2.1s | TBD |

## Validation

| Phase | Checks | Binary |
|-------|--------|--------|
| Phase 0 (Python) | TBD | `control/drift_selection/drift_selection.py` |
| Phase 1 (Rust) | 7/7 | `validate-drift` |

## Barracuda Path

`kimura_fixation_prob` — pure math, suitable for `barracuda::stats` CPU delegation.
`wright_fisher_fixation` — Monte Carlo, embarrassingly parallel on GPU.
`neutral_diversity_trajectory` — uses multinomial sampling (connects to
`batched_multinomial` Tier C kernel).

## Modules

`drift`, `prng`
