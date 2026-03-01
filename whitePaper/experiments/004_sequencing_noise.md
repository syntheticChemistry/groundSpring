# Exp 004: Sequencing Depth & Taxonomic Noise

**Domain**: Biological (microbiome)
**Source**: Synthetic 150-genus, 8-phylum soil community
**Question**: How does sequencing depth affect taxonomic reliability?

## Data Source

Synthetic community with known relative abundances.
Open system — reproducible from species abundance vector + PRNG seed.

## Method

Multinomial rarefaction at 9 depths × 50 replicates.
Shannon diversity convergence analysis.
Genus detection saturation curves.

## Key Result

**Phylum-level taxonomy is robust** — even 100 reads detect all 8 phyla.
**Genus-level** is the bottleneck: below 5,000 reads, ~3% of genera are
missed per sample. For wetSpring's pond crash detection, phylum-level
signals are well above the noise floor.

## Validation

| Phase | Checks | Binary |
|-------|--------|--------|
| Phase 0 (Python) | PASS | `control/sequencing_noise/sequencing_noise.py` |
| Phase 1 (Rust) | 15/15 | `validate-rarefaction` |

## Barracuda Path

Production WGSL shader (`batched_multinomial.wgsl`) in metalForge.
Tier C — BatchedMultinomialGpu absorbed in ToadStool; rare_biosphere wired.

## Modules

`rarefaction`, `prng`
