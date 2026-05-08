+++
title = "groundSpring Validation Summary"
description = "Measurement noise and uncertainty — 965+ tests, 35 experiments, guideStone Level 4, 5 notebooks, 2 composition crates"
date = 2026-05-08

[taxonomies]
primals = ["barracuda", "toadstool", "beardog", "songbird", "nestgate"]
springs = ["groundspring", "hotspring", "wetspring", "neuralspring", "airspring"]
+++

## Status

- **965+ tests** passing, 0 failed (146s full suite)
- **35 experiments** across 10 scientific domains
- **395/395 validation checks** (340 core + 55 NUCLEUS)
- **29/29 Python baselines** with math parity proven
- **110 barraCuda delegations** (67 CPU + 43 GPU)
- **guideStone Level 4** — bare + NUCLEUS composition parity (Tower + Node + Nest + cross-atomic)
- **2 composition experiment crates** — exp094 (NUCLEUS parity replication) + exp095 (measurement niche parity)
- **6 registry sync tests** — capability_registry.toml cross-validated against primalSpring canonical (389 methods)
- **16 measurement capabilities** synced (niche, YAML, deploy graphs)
- **Zero** unsafe blocks, production mocks, hardcoded addresses, `#[allow]` attrs

## Key Validation Binaries

- `groundspring_guidestone` — 5 bare properties + Layer 2-4 NUCLEUS composition (Tower/Node/Nest/cross-atomic)
- `validate_all` — meta-runner for all 29 Python-parity validators (exit-code protocol)
- `bench_gpu_vs_kokkos` — three-mode GPU benchmark (default → barraCuda CPU → GPU)

## sporePrint Notebooks (5)

| # | Notebook | Focus |
|---|----------|-------|
| 01 | Composition Validation | Deploy graphs, capabilities, guideStone, verb reconciliation |
| 02 | Benchmark Comparison | Rust vs Python timing, three-mode GPU, delegation inventory |
| 03 | Ecosystem Evidence | 35 experiments, domain distribution, gap resolution, security |
| 04 | Cross-Spring Connections | Primal consumption matrix, ecosystem flows, patterns pioneered |
| 05 | Measurement Science Deep Dive | Five pillars, tolerance architecture, Anderson localization thread |

## Baseline Notebooks (29)

Publication-grade Python baselines — each experiment as a live, executable notebook.

| # | Notebook | Domain | Faculty |
|---|----------|--------|---------|
| 001 | Sensor Noise Characterization | Measurement | Dong et al. |
| 002 | Observation Gap Analysis | Measurement | — |
| 003 | Error Propagation FAO-56 | Hydrology | Allen et al. |
| 004 | Sequencing Noise | Genomics | — |
| 005 | Seismic Wave Propagation | Geophysics | — |
| 006 | Signal Specificity (QS) | Biochemistry | Waters (MSU) |
| 007 | RAWR Bootstrap | Statistics | Liu (MSU) |
| 008 | Anderson Localization | Condensed Matter | Kachkovskiy (MSU) |
| 009 | Almost-Mathieu | Condensed Matter | Kachkovskiy (MSU) |
| 010 | Bistable Switching | Biochemistry | Waters (MSU) |
| 011 | Multi-Signal QS | Biochemistry | Waters (MSU) |
| 012 | Spin Chain Transport | Condensed Matter | Kachkovskiy / Gonzales |
| 013 | Resampling Convergence | Statistics | — |
| 014 | Drift vs Selection | Population Genetics | R. Anderson (Carleton) |
| 015 | Uncertainty Bridge | Cross-Domain | — |
| 016 | Rare Biosphere | Genomics | R. Anderson (Carleton) |
| 017 | Quasispecies Threshold | Evolutionary Biology | Dolson (MSU) |
| 018 | Band Edge Structure | Condensed Matter | Kachkovskiy (MSU) |
| 019 | Jackknife Estimation | Statistics | Bazavov (MSU) |
| 020 | Freeze-Out Inverse | Lattice QCD | Bazavov (MSU) |
| 021 | Spectral Reconstruction | Lattice QCD | Bazavov (MSU) |
| 022 | ET₀-Anderson Propagation | Hydrology | airSpring cross |
| 023 | No-Till 16S Sampling | Soil Science | wetSpring cross |
| 024 | Aggregate Stability | Soil Science | airSpring cross |
| 025 | f32/f64 Precision Drift | Numerical Methods | WDM |
| 026 | System-Size Convergence | Numerical Methods | WDM |
| 027 | GPU Vendor Parity | GPU Validation | WDM |
| 028 | NPU Anderson | Neuromorphic | metalForge |
| 029 | Multi-Method ET₀ | Hydrology | airSpring cross |

## Workload TOMLs (foundation)

4 workloads registered in `gardens/foundation/workloads/groundspring/`:

| Workload | Purpose |
|----------|---------|
| `gs-validate-all` | Run all 29 Rust validators |
| `gs-guidestone` | Run guideStone Level 4 check |
| `gs-bench-gpu` | Three-mode GPU benchmark |
| `gs-python-baselines` | Execute 29 Python baselines for provenance |

## See Also

- [Spring Catalog](https://primals.eco/architecture/spring-catalog/) on primals.eco
- [Lab Notebooks](https://primals.eco/lab/notebooks/) for rendered notebook views
- All baseCamp papers (groundSpring contributes uncertainty to all)
