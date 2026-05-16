# Paper Extension Roadmap — groundSpring V143

**Date:** May 8, 2026
**Version:** V143
**Prerequisite:** Local NUCLEUS on eastGate, NestGate data pipeline validated

---

## Overview

This document consolidates the extension roadmap for baseCamp papers that
groundSpring contributes to. Each paper section specifies: what datasets
extend the work, what compute is needed, what new experiments result, and
what primal wiring is required.

All extensions build on the NestGate data pipeline validated in Exp 029-032
and the NUCLEUS deploy graphs updated to V117.

---

## Paper 01 — Anderson QS (Highest Priority)

### Current State

- 28 synthetic biomes, 56 NCBI Protein queries, EMP 30K synthetic atlas
- groundSpring: Exp 008 (Anderson 1D/2D/3D), Exp 009 (Almost-Mathieu),
  Exp 012 (spin chain transport), Exp 015 (uncertainty bridge),
  Exp 018 (band edge structure)
- 52 Rust checks across 4 Kachkovskiy papers, all benchmark-grade

### Dataset Extensions

| Dataset | Accession | Size | NestGate Route | New Data |
|---------|-----------|------|----------------|----------|
| Cold seep 170 metagenomes | PRJNA315684 | ~5GB meta | `data.ncbi_search` (sra) | 5,000× more than 56 queries |
| LTEE frozen fossil | PRJNA294072 | ~2GB | `data.ncbi_search` (sra) | 60K generation W time series |
| wetSpring QS profiling | Exp140-153 | Cross-spring | Internal | 34 QS types, 299K genes |
| lsrB + luxS pairing | NCBI Protein | Metadata | `data.ncbi_search` (protein) | AI-2 interspecies bridge |

### New Experiments

| Exp | Description | Inputs | Outputs | Compute |
|-----|-------------|--------|---------|---------|
| 036 | Cold seep metadata → Anderson regime | PRJNA315684 metadata | W per community, regime map | ~30min |
| 037 | Real diversity vs synthetic proxy | Cold seep + synthetic | Correlation, bias quantification | ~1h |
| 038 | LTEE W(t) time series | PRJNA294072 | W evolution, drift dominance test | ~30min |

### Primal Wiring

- **NestGate**: `data.ncbi_search` → SRA metadata → Anderson classification
- **ToadStool**: GPU rarefaction + spectral analysis at scale
- **Squirrel**: ESN regime classification on real community time series
- **groundSpring**: Uncertainty budget (Exp 019 jackknife) on real W estimates

---

## Paper 04 — Sentinel Microbes

### Current State

- PFAS (Cai/Guo), HAB (Cahill/Smallwood) sentinel frameworks
- NPU validated on AKD1000 (Exp 028, 193-195)
- ESN regime classifier (96.5% accuracy from neuralSpring nW-05)

### Dataset Extensions

| Dataset | Source | Size | NestGate Route | Purpose |
|---------|--------|------|----------------|---------|
| Baseline 16S monitoring | Field collection | TBD | `storage.put` | ESN training baseline |
| Paired PFAS 16S + LC-MS/MS | Jones Lab MSU | TBD | Direct collaboration | Detection threshold validation |
| HAB lake weekly 16S | Field deployment | TBD | `storage.put` | Real-time regime classification |
| NCBI 16S reference | NCBI SRA | ~1GB | `data.ncbi_search` | Baseline community characterization |

### New Experiments

| Exp | Description | Inputs | Outputs | Compute |
|-----|-------------|--------|---------|---------|
| 036 | Sentinel detection threshold | Exp 019 (jackknife) + Exp 015 (bridge) | Minimum detectable perturbation | Minutes |
| 037 | ESN training on simulated perturbation | Synthetic baseline + perturbation | ROC curve, detection sensitivity | ~1h |
| 038 | Real weather covariate integration | GHCND (Exp 029) + community shift | Environmental correlation | Minutes |

### Primal Wiring

- **NestGate**: `data.noaa_ghcnd` for weather covariates, `storage.put` for field data
- **ToadStool**: GPU ESN inference at scale (18.8K Hz from NPU)
- **Squirrel**: ConceptEdge for regime boundary detection
- **groundSpring**: Uncertainty budget determines ESN detection threshold

### Blockers

- 12-24mo baseline field sampling (calendar time, not compute)
- Jones Lab PFAS data (collaboration, not infrastructure)

---

## Paper 05 — Cross-Species Signaling

### Current State

- Cold seep 299K QS genes, 34 types (Exp144-145)
- luxR phylogeny × geometry (Exp146)
- Eavesdropper R:P ratios across 6 habitats (Exp142)

### Dataset Extensions

| Dataset | Source | Size | NestGate Route | Purpose |
|---------|--------|------|----------------|---------|
| Lichen metagenomes | NCBI SRA | ~5GB | `data.ncbi_search` (isolation_source) | 2D vs 3D QS gene density |
| Root nodule metagenomes | NCBI SRA | ~5GB | `data.ncbi_search` (isolation_source) | Stage-specific QS expression |
| Coral metagenomes | NCBI SRA | ~10GB | `data.ncbi_search` (isolation_source) | Skeleton vs mucus QS |
| luxS + lsrB co-occurrence | NCBI Protein | Metadata | `data.ncbi_search` (protein) | AI-2 bridge validation |

### New Experiments

| Exp | Description | Inputs | Outputs | Compute |
|-----|-------------|--------|---------|---------|
| 036 | Symbiotic QS gene density comparison | Lichen + nodule + coral 16S | Anderson regime per system | ~1h per system |
| 037 | AI-2 bridge validation | luxS + lsrB NCBI queries | Co-occurrence matrix, geometry correlation | ~10min |
| 038 | Mycorrhizal relay transport | Exp 012 extension | Transport distance through AM network | ~5min |

### Primal Wiring

- **NestGate**: `data.ncbi_search` by isolation_source for symbiotic metagenomes
- **ToadStool**: GPU spectral analysis for Anderson classification
- **groundSpring**: Exp 012 (spin chain transport) for mycorrhizal relay,
  Exp 018 (band edge) for coral skeleton signal windows

---

## Paper 06 — No-Till Anderson

### Current State

- Track 4 complete: 9 papers, 321 checks, full three-tier
- Dynamic W(t) models validated (Exp186 tillage/antibiotic/seasonal)
- airSpring coupling: θ → S_e → d_eff → QS regime (55+95 checks)

### Dataset Extensions

| Dataset | Source | Size | NestGate Route | Status |
|---------|--------|------|----------------|--------|
| Real GHCND Ohio weather | NOAA | ~1MB/yr | `data.noaa_ghcnd` | Wired, Exp 029 |
| NCBI SRA no-till 16S | NCBI | ~105K entries | `data.ncbi_search` (sra) | Ready |
| Open-Meteo ERA5 Ohio | Open-Meteo | API | Future `data.open_meteo` | Provider exists |
| KBS LTER 30yr soil | MSU KBS | ~200GB | Manual / patient download | Tier 2 |

### New Experiments

| Exp | Description | Inputs | Outputs | Compute |
|-----|-------------|--------|---------|---------|
| 036 | Real weather → ET₀ → Anderson (Ohio 2023) | GHCND via NestGate | Seasonal QS regime dynamics | Minutes |
| 037 | NCBI no-till 16S diversity | SRA no-till studies | Real H′ comparison, W distribution | ~2h |
| 038 | Dynamic W(t) on real weather | GHCND + Exp186 framework | Annual QS cycle for Ohio soils | ~30min |

### Primal Wiring

- **NestGate**: `data.noaa_ghcnd` (weather), `data.ncbi_search` (16S)
- **airSpring**: θ(t) from real weather via FAO-56 water balance
- **ToadStool**: GPU diversity + Anderson spectral at scale
- **groundSpring**: Exp 022 (ET₀→Anderson propagation), Exp 023 (no-till sampling),
  Exp 024 (aggregate stability noise)

---

## Paper 12 — Immunological Anderson

### Current State

- Gonzales G1-G6 reproduced, McCandless IL-31 targets
- Tissue Anderson Exp 033: 29/29 Rust checks
- Fajgenbaum MATRIX: 6 drug candidates scored (nS-605)
- Dimensional promotion–collapse duality documented

### Dataset Extensions

| Dataset | Source | Size | NestGate Route | Tier |
|---------|--------|------|----------------|------|
| NCBI Protein (IL-31RA, IL-4Rα, OSMR) | NCBI | Metadata | `data.ncbi_search` (protein) | 1 |
| ADDRC 8,000+ compound library | MSU ADDRC | Metadata | Manual / future | 3 |
| Single-cell skin transcriptomics | GEO/SRA | ~50GB | `data.ncbi_search` (sra) | 3 |
| Gonzales iPSC validation data | Lab data | TBD | Collaboration | Future |

### New Experiments

| Exp | Description | Inputs | Outputs | Compute |
|-----|-------------|--------|---------|---------|
| 036 | Real cytokine receptor density | NCBI Protein queries | W estimation for skin compartments | Minutes |
| 037 | Anderson-augmented MATRIX expansion | ADDRC compound metadata | Expanded drug scoring (>6 candidates) | ~1h |
| 038 | Cross-species skin comparison | Canine vs human NCBI data | d_eff barrier differences | ~30min |

### Primal Wiring

- **NestGate**: `data.ncbi_search` (protein) for receptor counts
- **ToadStool**: GPU 3D tissue lattice simulation
- **Squirrel**: ConceptEdge/DriftAction for AD flare detection
- **Full NUCLEUS**: Tower (patient crypto) + Node (GPU) + Nest (data) + Squirrel (AI)

### Gonzales Lab Integration

1. NCBI receptor density → groundSpring uncertainty → tissue W
2. MATRIX scoring (nS-605) → ADDRC compound selection
3. iPSC validation → Exp 033 real-data extension
4. Patient monitoring → ESN classification on NPU

---

## Cross-Paper Integration Map

```
                 NestGate (data)
                     │
    ┌────────────────┼────────────────┐
    │                │                │
  Paper 01      Paper 06         Paper 05
  (NCBI SRA)   (GHCND weather)  (NCBI metagenomes)
    │                │                │
    └───────┬────────┘                │
            │                         │
      groundSpring                    │
      (uncertainty)                   │
            │                         │
    ┌───────┴────────┐                │
    │                │                │
  Paper 04       Paper 12          Paper 05
  (sentinels)   (immunological)   (symbiotic)
    │                │                │
    └───────┬────────┴────────────────┘
            │
      neuralSpring
      (ESN/LSTM classification)
```

---

## Implementation Priority

| Priority | Action | Papers | Blocked By |
|:--------:|--------|--------|------------|
| 1 | Real GHCND weather (already wired) | 06 | Nothing |
| 2 | NCBI Protein QS queries | 01, 05, 12 | Nothing |
| 3 | Cold seep metadata fetch | 01, 05 | Nothing |
| 4 | Symbiotic metagenome search | 05 | Nothing |
| 5 | NCBI SRA no-till 16S | 06 | Nothing |
| 6 | LTEE metadata | 01 | Nothing |
| 7 | Cold seep FASTQ download | 01 | NestGate SRA evolution |
| 8 | KBS LTER 30yr | 06 | 10G cables or patience |
| 9 | Single-cell skin transcriptomics | 12 | LAN infrastructure |
| 10 | ADDRC compound library | 12 | Lab collaboration |

Items 1-6 can execute immediately with local NUCLEUS on eastGate.
