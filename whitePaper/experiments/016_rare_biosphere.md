# Exp 016: Rare Biosphere Signal Detection

**Domain**: Biological (microbial ecology, rare biosphere)
**Paper**: Anderson, Sogin, Baross (2015) FEMS Microbiol Ecol 91:fiv016
**Faculty**: Rika Anderson (Biology, Carleton College)
**Question**: At what sequencing depth can we reliably distinguish rare
biological lineages from sequencing artifacts?

## Data Source

Synthetic community: 50 species across 5 abundance tiers (dominant 0.06–0.15,
common 0.03, moderate 0.008, rare 0.004, very rare 0.003). Multinomial
sampling simulates sequencing at depths 100–50,000 reads. Chao1 richness
estimator (Chao 1984) corrects for unobserved species.

## Method

1. **Chao1 accuracy**: Compare estimated vs true richness at each depth.
   Chao1 = S_obs + f₁²/(2f₂) exceeds S_obs at low depth.
2. **Detection power**: P(detect) = 1 − (1−p)^D verified by simulation.
3. **Detection threshold**: D* = ⌈ln(0.05)/ln(1−p)⌉ for 95% power.
4. **Abundance-occupancy**: Rare taxa detected in fewer replicate samples.
5. **Singleton discrimination**: Singleton fraction decreases with depth
   as undersampled common species are fully resolved.

## Key Result

**Sequencing depth determines the signal/noise boundary for rare taxa.**
- Dominant species: near-certain detection at D=100
- Very rare (p=0.003): only 26% detected at D=100, near-certain at D=5,000
- Detection threshold for rarest lineages: D* ≈ 998 reads (95% power)
- Chao1 corrects undersampling at D=100 (47.4 vs S_obs 28.7) but
  converges to true richness (50.0) at D=50,000
- Abundance-occupancy correlation: ρ = 0.965

**Anderson et al. (2015) showed** that rare microbial lineages in deep-sea
hydrothermal vents are real biological signal, not sequencing noise. This
experiment quantifies the depth required to distinguish the two.

## Validation

| Phase | Checks | Binary |
|-------|--------|--------|
| Phase 0 (Python) | 11/11 | `control/rare_biosphere/rare_biosphere.py` |
| Phase 1 (Rust) | 10/10 | `validate-rare-biosphere` |

## Barracuda Path

`chao1` exists in barracuda (`barracuda::stats::diversity::chao1`).
Detection power and threshold are pure arithmetic (no delegation needed).
Multinomial sampling for occupancy analysis is embarrassingly parallel.

## Modules

`rare_biosphere` (`chao1`, `detection_power`, `detection_threshold`,
`abundance_occupancy`, `singleton_fraction`, `tier_detection_rate`),
`rarefaction` (reused `multinomial_sample`), `prng`
