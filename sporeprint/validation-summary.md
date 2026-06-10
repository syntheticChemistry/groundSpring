+++
title = "groundSpring Validation Summary"
description = "Measurement noise and uncertainty — 1,123 tests, 39 experiments (34 core + 5 LTEE B1–B4, B6), guideStone Level 5, Eukaryotic UniBin, 5 notebooks, 11 validation scenarios"
date = 2026-06-10

[taxonomies]
primals = ["barracuda", "toadstool", "beardog", "songbird", "nestgate"]
springs = ["groundspring", "hotspring", "wetspring", "neuralspring", "airspring"]
+++

## Status

- **1,123 tests** passing, 0 failed, 0 clippy warnings on all targets
- **39 experiments** across 12 scientific domains
- **11 validation scenarios** in ScenarioRegistry (9 Tier 1, 2 Tier 2)
- **461/461 validation checks** (340 core + 55 NUCLEUS + 66 LTEE)
- **29/29 Python baselines** with math parity proven
- **110 barraCuda delegations** (67 CPU + 43 GPU)
- **guideStone Level 5** (eukaryotic UniBin, bonding model)
- **Tier 4 IPC-first** — `barracuda` removed from default features; IPC via `CompositionContext` default
- **biomeOS v3.75** — `composition.status` + `method.register` + cross-gate `capability.call` routing
- **skunkBat** — `security.audit_log` wired in all 7 deploy graphs
- **certification/ organelle** — Properties 1-5 (bare) + Layers 2-4 (NUCLEUS)
- **groundspring_unibin** — single binary: certify / validate / status / version
- **src/ipc/ tree** — per-primal modules (barraCuda, ToadStool, NestGate, BearDog, Songbird)
- **primalSpring v0.9.27 pinned** — CompositionContext, ScenarioMeta, ScenarioRegistry
- **fossilRecord/** — consolidated to dedicated repo (breadcrumb in-tree)
- **Zero** unsafe, bare `#[allow]`/`#[expect]` without reason, TODO/FIXME

## Key Validation Binaries

- `groundspring_unibin certify` — L0-L4 certification (supersedes groundspring_guidestone)
- `groundspring_unibin validate --tier rust` — 9 Tier 1 scenarios (CI-safe, no IPC)
- `groundspring_unibin validate --tier live` — Tier 2 NUCLEUS composition parity
- `validate_all` — meta-runner for all 39 validation binaries (exit-code protocol)
- `bench_gpu_vs_kokkos` — three-mode GPU benchmark (default → barraCuda CPU → GPU)

## sporePrint Notebooks (5)

| # | Notebook | Focus |
|---|----------|-------|
| 01 | Composition Validation | Deploy graphs, capabilities, guideStone, verb reconciliation |
| 02 | Benchmark Comparison | Rust vs Python timing, three-mode GPU, delegation inventory |
| 03 | Ecosystem Evidence | 39 experiments, domain distribution, gap resolution, security |
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
| `gs-validate-all` | Run all 39 Rust validators |
| `gs-guidestone` | Run guideStone Level 5 check |
| `gs-bench-gpu` | Three-mode GPU benchmark |
| `gs-python-baselines` | Execute 29 Python baselines for provenance |

## See Also

- [Spring Catalog](https://primals.eco/architecture/spring-catalog/) on primals.eco
- [Lab Notebooks](https://primals.eco/lab/notebooks/) for rendered notebook views
- All baseCamp papers (groundSpring contributes uncertainty to all)
