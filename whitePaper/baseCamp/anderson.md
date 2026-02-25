# R. Anderson — Eco-Evolutionary Noise

**Faculty**: Rika Anderson (Biology, Carleton College)
**Domain**: Deep subsurface microbiology, microbial evolution, rare biosphere
**groundSpring Connection**: Stochastic vs deterministic evolution (Exp 001 biological analog)

---

## Why This Matters for groundSpring

groundSpring Exp 001 decomposes measurement error into bias (signal) and noise
(random). Rika Anderson's work asks the identical question in evolutionary
biology: in energy-limited subsurface environments, is microbial evolution
driven by natural selection (signal) or genetic drift (noise)?

Her 2022 mBio paper provides **empirical proof** that drift dominates selection
in low-biomass systems — exactly as groundSpring Exp 001 finds that noise
dominates signal in certain sensor configurations. This is the biological
validation of groundSpring's decomposition framework applied to evolution.

## Papers for Reproduction

### Tier 1 (Priority)

**Paper #20**: Anderson et al. (2022) "Microbial population dynamics are
dominated by stochastic forces in a low biomass subseafloor habitat."
mBio 13:e00354-22. DOI: 10.1128/mbio.00354-22

- **Open Data**: Metagenomic sequences deposited in NCBI SRA (accession in paper)
- **Open Code**: Bioinformatics pipeline described in Methods
- **groundSpring Modules**: `decompose` (drift vs selection decomposition),
  `stats` (diversity metrics), `rarefaction` (depth-normalized comparison)
- **BarraCUDA Needs**: Smith-Waterman alignment (`SmithWatermanGpu` exists),
  diversity metrics (`BrayCurtisF64` exists), rarefaction (Tier C kernel)
- **Control Plan**: Python metagenomic pipeline → Rust CPU → barracuda GPU

### Tier 2

**Paper #21**: Anderson, Sogin, Baross (2015) "Biogeography and ecology of
the rare and abundant microbial lineages in deep-sea hydrothermal vent fluids."
FEMS Microbiol Ecol 91:fiv016.

- **Open Data**: SRA accession in paper
- **Method**: Rare biosphere detection — when does a detected lineage represent
  real biological signal vs sequencing artifact?
- **groundSpring Modules**: `rarefaction` (detection limits), `stats` (hit rate
  for rare taxa detection)

### Reference

**Paper #19**: Anderson (2021) "Tracking Microbial Evolution in the Subseafloor
Biosphere." mSystems 6:e00731-21.

- **Type**: Review / framework paper
- **Value**: Formalizes selection-vs-drift question; cites Lenski LTEE;
  introduces Muller's ratchet in energy-limited environments

## BarraCUDA Kernel Requirements

| Primitive | Status | Notes |
|-----------|--------|-------|
| Smith-Waterman | Exists (`SmithWatermanGpu`) | Sequence alignment |
| Bray-Curtis | Exists (`BrayCurtisF64`) | Community dissimilarity |
| Shannon diversity | `fused_map_reduce_f64` | Tier A rewire |
| Rarefaction | **Gap** — Tier C kernel | `batched_multinomial.wgsl` |
| Taxonomy classifier | Exists (`PangenomeClassifyGpu`) | Read classification |

## Three-Tier Control Plan

| Tier | Validation | Status |
|------|-----------|--------|
| CPU | Python metagenomic baseline matches Rust | Queued |
| GPU | barracuda alignment + diversity GPU | Partial (SW, BC exist) |
| metalForge | Cross-substrate for full pipeline | After GPU tier |

## Cross-Spring

- **wetSpring**: Metagenomic pipelines (DADA2, taxonomy) are wetSpring's domain.
  groundSpring adds drift-vs-selection decomposition.
- **Shared with wetSpring**: Rarefaction (Exp 004), Bray-Curtis, Shannon diversity
