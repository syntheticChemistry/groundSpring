# groundSpring Baseline Notebooks

Publication-grade Python baselines for groundSpring experiments.
Each notebook is executable, self-contained, and produces charts for the website.

**Coverage**: 29 core experiment baselines (Exp 001–029). Experiments 030–035 (NUCLEUS/live-data/hydrology) and 036–039, 040 (LTEE B1–B4, B6) have Python baselines in `control/` scripts rather than notebooks. Two NUCLEUS parity harnesses (`exp094`, `exp095`) are Rust-only validation crates.

| # | Notebook | Domain | Cells |
|---|----------|--------|-------|
| 001 | [Sensor Noise Characterization](exp-001-sensor-noise.ipynb) | Measurement | 27 |
| 002 | [Observation Gap Analysis](exp-002-observation-gap.ipynb) | Measurement | 26 |
| 003 | [Error Propagation FAO-56](exp-003-error-propagation.ipynb) | Hydrology | 30 |
| 004 | [Sequencing Noise Characterization](exp-004-sequencing-noise.ipynb) | Genomics | 28 |
| 005 | [Seismic Wave Propagation](exp-005-seismic.ipynb) | Geophysics | 32 |
| 006 | [Signal Specificity in Quorum Sensing](exp-006-signal-specificity.ipynb) | Biochemistry | 25 |
| 007 | [RAWR Bootstrap Resampling](exp-007-rawr-resampling.ipynb) | Statistics | 24 |
| 008 | [Anderson Localization](exp-008-anderson-localization.ipynb) | Condensed Matter | 26 |
| 009 | [Almost-Mathieu Quasiperiodic Localization](exp-009-quasiperiodic.ipynb) | Condensed Matter | 26 |
| 010 | [Bistable Phenotypic Switching](exp-010-bistable-switching.ipynb) | Biochemistry | 28 |
| 011 | [Multi-Signal Quorum Sensing Integration](exp-011-multisignal-qs.ipynb) | Biochemistry | 25 |
| 012 | [Spin Chain Transport](exp-012-spin-transport.ipynb) | Condensed Matter | 22 |
| 013 | [Resampling Convergence Analysis](exp-013-resampling-convergence.ipynb) | Statistics | 22 |
| 014 | [Drift vs Selection in Microbial Populations](exp-014-drift-selection.ipynb) | Population Genetics | 20 |
| 015 | [Uncertainty Bridge: Sensor Noise → Localization](exp-015-uncertainty-bridge.ipynb) | Cross Domain | 20 |
| 016 | [Rare Biosphere Signal Detection](exp-016-rare-biosphere.ipynb) | Genomics | 22 |
| 017 | [Quasispecies Error Threshold](exp-017-quasispecies-threshold.ipynb) | Evolutionary Biology | 24 |
| 018 | [Band Edge Structure](exp-018-band-edge.ipynb) | Condensed Matter | 26 |
| 019 | [Jackknife Error Estimation](exp-019-jackknife-estimation.ipynb) | Statistics | 20 |
| 020 | [Freeze-Out Inverse Problem](exp-020-freeze-out-inverse.ipynb) | Lattice Qcd | 20 |
| 021 | [Spectral Reconstruction](exp-021-spectral-recon.ipynb) | Lattice Qcd | 18 |
| 022 | [ET₀-Anderson Error Propagation](exp-022-et0-anderson-propagation.ipynb) | Hydrology | 24 |
| 023 | [No-Till vs Tilled 16S Sampling](exp-023-notill-sampling.ipynb) | Soil Science | 25 |
| 024 | [Aggregate Stability Noise Analysis](exp-024-aggregate-stability.ipynb) | Soil Science | 26 |
| 025 | [f32 vs f64 Precision Drift](exp-025-precision-drift.ipynb) | Numerical Methods | 16 |
| 026 | [System-Size Convergence](exp-026-size-convergence.ipynb) | Numerical Methods | 16 |
| 027 | [GPU Vendor Parity](exp-027-vendor-parity.ipynb) | Gpu Validation | 16 |
| 028 | [NPU Anderson Classification](exp-028-npu-anderson.ipynb) | Neuromorphic | 8 |
| 029 | [Multi-Method ET₀ Comparison](exp-029-et0-methods.ipynb) | Hydrology | 23 |

## Conventions

- All notebooks load frozen benchmark data from `control/<experiment>/benchmark_*.json`
- Charts use ecosystem palette: `#2ecc71` (pass), `#e74c3c` (fail), `#3498db` (info)
- Each notebook ends with a provenance summary
- Notebooks are executable in CI via `jupyter nbconvert --execute`
