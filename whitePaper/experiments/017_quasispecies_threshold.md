# Exp 017: Eco-Evolutionary Noise Threshold

**Domain**: Evolutionary dynamics (quasispecies theory)
**Paper**: Dolson, Banzhaf, Ofria (2023) ALife
**Faculty**: Emily Dolson (Computer Science & Engineering, MSU)
**Question**: Does Eigen's error threshold predict the critical mutation rate
above which genetic information collapses in a finite-population simulation?

## Data Source

Eigen's quasispecies model (1971): L=100 genome length, σ=10 master
fitness, background fitness 1. Population of 10,000 discrete organisms.
Wright-Fisher selection + per-locus mutation. 500 generations per trial.

## Method

1. **Error threshold**: μ_c = 1 − σ^(−1/L). For σ=10, L=100:
   μ_c ≈ 0.02276.
2. **Master frequency**: x_m = max(0, (σ(1−μ)^L − 1)/(σ − 1)).
   Below threshold, master genotype dominates; above, population
   uniformly distributes across genotypes.
3. **Simulation**: Wright-Fisher resampling with multinomial fitness
   weighting, followed by independent per-locus mutation. Track master
   frequency across 500 generations.
4. **Sweep**: Scan 12 mutation rates across the phase transition
   (μ = 0.005 to 0.060) and verify monotonic decay of master frequency.

## Key Result

**The error threshold is a sharp phase transition in evolutionary dynamics.**
- Below threshold (μ=0.010): master frequency x_m ≈ 0.42 (analytical: 0.44)
- Near threshold (μ=0.023): x_m drops to ≈ 0.002
- Above threshold (μ=0.040): x_m ≈ 0 (information collapse)
- Analytical μ_c = 0.02276 matches simulation transition zone
- Master frequency decays monotonically across the sweep

**Dolson et al. (2023) showed** that ecological interactions can modulate
this threshold — understanding the baseline Eigen model is essential before
layering in eco-evolutionary complexity.

## Validation

| Phase | Checks | Binary |
|-------|--------|--------|
| Phase 0 (Python) | 9/9 | `control/quasispecies_threshold/quasispecies_threshold.py` |
| Phase 1 (Rust) | 6/6 | `validate-quasispecies` |

## Barracuda Path

Wright-Fisher resampling is multinomial sampling (already in barracuda
statistics module). Per-locus mutation is embarrassingly parallel across
the population — ideal for GPU dispatch via metalForge. The analytical
formulas are pure arithmetic.

## Modules

`quasispecies` (`error_threshold`, `master_frequency_analytical`,
`quasispecies_simulation`, `mean_fitness`), `prng`
