# R. Anderson — Eco-Evolutionary Noise

**Faculty**: Rika Anderson (Biology, Carleton College)
**Domain**: Deep subsurface microbiology, microbial evolution, rare biosphere
**groundSpring Connection**: Stochastic vs deterministic evolution (Exp 001 biological analog). *Note: The groundSpring `anderson` module (Anderson localization, Exp 008) is DONE (8/8 PASS, 29.8× faster) — that module is for Kachkovskiy's spectral theory work, not this briefing.*

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
| CPU | Python metagenomic baseline matches Rust | **DONE** — Exp 014 (drift/selection 8/8 Py, 8/8 Rust), Exp 016 (rare biosphere 10/10 Py, 10/10 Rust) |
| GPU | barracuda alignment + diversity GPU | Partial (SW, BC exist; 43 GPU delegations active) |
| metalForge | Cross-substrate for full pipeline | In progress (19 workloads defined, `forge::tolerance` module with 4 tiers) |

## Cross-Spring

- **wetSpring**: Metagenomic pipelines (DADA2, taxonomy) are wetSpring's domain.
  groundSpring adds drift-vs-selection decomposition.
- **Shared with wetSpring**: Rarefaction (Exp 004), Bray-Curtis, Shannon diversity

## V114 Extension Roadmap

### V115 Capabilities

- `wright_fisher_fixation` returns `Result<bool, InputError>` — zero panicking public API
- `neutral_diversity_trajectory` returns `Result<Vec<f64>, InputError>`
- New error-path tests: zero population, out-of-range frequency, zero species
- `validate-drift` binary updated to `.or_exit()` on Result
- CI: nursery lint enforcement, `--all-features` test coverage, aarch64 cross-compile

### V114 Capabilities

- `.expect()` → `OrExit` in all validation binaries (Exp 014, 016)
- `cast::usize_u64()` in rarefaction sampling — checked numeric conversions
- `resilient_call()` for NestGate IPC data pipeline calls
- `health.liveness`/`health.readiness` probes for NUCLEUS deployment
- NUCLEUS composition: groundSpring measurement.* capabilities in Tower + Node + Nest

### Dataset Extensions

| Dataset | Accession | Size | NestGate Route | Papers |
|---------|-----------|------|----------------|--------|
| Cold seep 170 metagenomes | PRJNA315684 | ~5GB metadata, ~170GB raw | `data.ncbi_search` (sra) | 01, 05, 06 |
| LTEE frozen fossil | PRJNA294072 | ~2GB | `data.ncbi_search` (sra) | 01 |
| Deep-sea vent 16S | PRJNA283159 | ~3GB | `data.ncbi_search` (sra) | 01 |
| Symbiotic metagenomes | NCBI isolation_source | ~20GB | `data.ncbi_search` | 05 |

### Compute Budget

| Workload | Single GPU (RTX 4070) | LAN (176GB VRAM) |
|----------|-----------------------|------------------|
| Cold seep diversity + W | ~2h | ~15min |
| LTEE drift/selection | ~30min | N/A |
| Rare biosphere D* on real data | ~1h | ~10min |

### New Experiments (Planned)

- **Exp 036+**: Real SRA drift vs selection on Anderson 2022 mBio sequences (NestGate → NCBI)
- **Exp 037+**: Rare biosphere D* calibration on real deep-sea vent communities
- Integration with wetSpring QS gene profiling (Exp140/141/144/146) for biological context

### Primal Wiring

- NestGate: `data.ncbi_search` with `database: "sra"` for real 16S datasets (Exp 029/030 pattern)
- ToadStool: `compute.execute` for GPU rarefaction at scale
- Squirrel: ESN regime classification on real community diversity time series
