# Tier 1 Dataset Acquisition Plan

**Date:** May 8, 2026
**Version:** V125
**Prerequisite:** Local NUCLEUS on eastGate (groundspring_nucleus_local.toml)

---

## Overview

Tier 1 datasets are downloadable now via NestGate and processable on a single
GPU in hours. These represent the first real-data extensions to baseCamp papers,
replacing synthetic/analytical baselines with empirical data.

## Dataset Inventory

### 1. Cold Seep 170 Metagenomes (PRJNA315684)

**Paper:** 01 (Anderson QS), 05 (Cross-Species Signaling), 06 (No-Till Anderson)
**Source:** Dong et al. (2025) Microbiome — 299,355 QS genes, 34 QS types
**Size:** ~5GB metadata, ~170GB raw FASTQ (metadata first, raw later)

**NestGate Route:**
```
nestgate::ncbi_search(socket, "sra", "PRJNA315684[BioProject]")
  → Returns SRA run accessions for 170 metagenomes
nestgate::ncbi_fetch(socket, "sra", "{accession}")
  → Returns metadata for each run (sample attributes, organism, platform)
```

**Phase 1 — Metadata Only (immediate):**
- Fetch SRA metadata for all 170 runs via `data.ncbi_search`
- Extract: sample attributes (depth, temperature, chemistry), organism lists
- Compute: Pielou evenness J → Anderson disorder W for each community
- Classify: Anderson regime (extended/localized/marginal) per sample
- Cache via `storage.put` with key `groundspring:data:ncbi:PRJNA315684:{accession}`

**Phase 2 — FASTQ Download (needs NestGate SRA evolution):**
- Bulk FASTQ download via SRA Toolkit (not yet wired to NestGate)
- Process: DADA2 → ASV table → diversity metrics → Anderson classification
- Compute budget: ~2h diversity + W calculation on single GPU

**groundSpring Experiments:**
- New Exp 036: Cold seep metadata → Anderson regime classification
- New Exp 037: Real diversity vs synthetic proxy comparison

### 2. LTEE Frozen Fossil (PRJNA294072)

**Paper:** 01 (Anderson QS)
**Source:** Lenski Long-Term Evolution Experiment — 60,000+ generations
**Size:** ~2GB

**NestGate Route:**
```
nestgate::ncbi_search(socket, "sra", "PRJNA294072[BioProject]")
nestgate::ncbi_fetch(socket, "sra", "{accession}")
```

**Analysis:**
- Time-series W evolution across 60,000 generations
- Drift vs selection decomposition (Exp 014 extension)
- Anderson regime dynamics during citrate innovation (Ara-3)
- Compute budget: ~30min on single GPU

### 3. Real GHCND Weather (Lansing MI 2023-2024)

**Paper:** 02 (ET₀), 03 (Bioag), 06 (No-Till), 22 (FAO-56)
**Source:** NOAA GHCND — station USW00014836 (Capital Region Airport)
**Size:** ~1MB per year

**NestGate Route (already wired, Exp 029 validated):**
```
nestgate::noaa_ghcnd(socket, "USW00014836", "2023-01-01", "2024-12-31",
    &["TMAX", "TMIN", "AWND", "RHAV"])
```

**Analysis:**
- Real ET₀ calculation via Penman-Monteith (Exp 002 extension)
- θ(t) → d_eff(t) → Anderson QS regime dynamics
- Seasonal oscillation of QS regime in Michigan soil
- Compute budget: minutes

### 4. IRIS Seismic Events (New Madrid Seismic Zone)

**Paper:** 05 (Cross-Species), 32 (Seismic Validation)
**Source:** IRIS FDSN — NMSZ events 2023-2024
**Size:** ~1MB

**NestGate Route (already wired, Exp 032 validated):**
```
nestgate::iris_events(socket, &IrisEventQuery {
    min_lat: 34.0, max_lat: 40.0,
    min_lon: -92.0, max_lon: -86.0,
    start_date: "2023-01-01", end_date: "2024-12-31",
    min_magnitude: 2.0,
})
```

**Analysis:**
- Real P-wave data for seismic inverse problem (Exp 005 extension)
- Anderson transport validation with real geological disorder
- Compute budget: minutes

### 5. Symbiotic Metagenomes (Lichen, Root Nodule, Coral)

**Paper:** 05 (Cross-Species Signaling)
**Source:** NCBI SRA — filtered by isolation_source
**Size:** ~20GB combined

**NestGate Route:**
```
nestgate::ncbi_search(socket, "sra",
    "lichen[isolation_source] AND 16S[All Fields]")
nestgate::ncbi_search(socket, "sra",
    "root nodule[isolation_source] AND 16S[All Fields]")
nestgate::ncbi_search(socket, "sra",
    "coral[isolation_source] AND 16S[All Fields]")
```

**Analysis:**
- QS gene density comparison: symbiotic vs free-living
- Anderson geometry prediction validation (2D lichen vs 3D nodule)
- AI-2 bridge: luxS + lsrB co-occurrence search
- Compute budget: ~1h per system on single GPU

### 6. NCBI Protein QS Gene Queries

**Paper:** 01 (Anderson QS), 05 (Cross-Species), 12 (Immunological Anderson)
**Source:** NCBI Protein database
**Size:** Metadata only

**NestGate Route:**
```
nestgate::ncbi_search(socket, "protein", "luxI[Gene] AND bacteria[Organism]")
nestgate::ncbi_search(socket, "protein", "luxR[Gene] AND bacteria[Organism]")
nestgate::ncbi_search(socket, "protein", "luxS[Gene] AND bacteria[Organism]")
nestgate::ncbi_search(socket, "protein", "lsrB[Gene] AND bacteria[Organism]")
nestgate::ncbi_search(socket, "protein", "agrA[Gene] AND bacteria[Organism]")
nestgate::ncbi_search(socket, "protein", "sdiA[Gene] AND bacteria[Organism]")
nestgate::ncbi_search(socket, "protein",
    "IL-31RA[Gene] AND skin[All Fields]")
nestgate::ncbi_search(socket, "protein",
    "IL-4R[Gene] AND skin[All Fields]")
```

**Analysis:**
- QS gene prevalence by habitat geometry (3D-dense vs 2D-mat vs planktonic)
- Cytokine receptor distribution for Paper 12
- Compute budget: seconds per query

---

## Acquisition Order

| Priority | Dataset | NestGate Status | Blocking On |
|:--------:|---------|-----------------|-------------|
| 1 | Real GHCND weather | Wired, validated (Exp 029) | Nothing |
| 2 | IRIS seismic events | Wired, validated (Exp 032) | Nothing |
| 3 | NCBI Protein QS queries | Wired (ncbi_search) | Nothing |
| 4 | Cold seep metadata | Wired (ncbi_search) | Nothing |
| 5 | LTEE metadata | Wired (ncbi_search) | Nothing |
| 6 | Symbiotic metagenomes | Wired (ncbi_search) | Nothing |
| 7 | Cold seep FASTQ | Not wired | NestGate SRA evolution |

Items 1-6 can proceed immediately when local NUCLEUS is running.
Item 7 requires NestGate evolution for bulk SRA Toolkit download.

---

## Cache Strategy

All fetched data is cached via NestGate `storage.put`:

```
groundspring:data:noaa_cdo:USW00014836_2023       — GHCND weather
groundspring:data:iris:nmsz_events_2023_2024       — Seismic events
groundspring:data:ncbi:PRJNA315684:{accession}     — Cold seep metadata
groundspring:data:ncbi:PRJNA294072:{accession}     — LTEE metadata
groundspring:data:ncbi:lichen_16s_sra              — Lichen metagenomes
groundspring:data:ncbi:protein_luxI_bacteria       — QS gene counts
```

Cache-through pattern via `nestgate::fetch_cached()`:
1. Check `storage.get` for cached data
2. On miss, call live provider
3. Cache result via `storage.put`
4. Return data

---

## Compute Budget Summary

| Dataset | Download | GPU Processing | Total |
|---------|----------|---------------|-------|
| GHCND weather | Seconds | Minutes | Minutes |
| IRIS seismic | Seconds | Minutes | Minutes |
| NCBI Protein queries | Seconds | Seconds | Seconds |
| Cold seep metadata | ~5min | ~30min | ~35min |
| LTEE metadata | ~2min | ~10min | ~12min |
| Symbiotic metagenomes | ~30min | ~1h | ~1.5h |
| **Total Tier 1** | **~40min** | **~2h** | **~3h** |

All processable on eastGate (RTX 4070, 12GB VRAM) in a single session.
