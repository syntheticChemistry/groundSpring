# groundSpring Experiments

> Per-experiment summaries for all control experiments in the groundSpring
> noise characterization framework. Each experiment has a Python baseline
> (Phase 0), a Rust validation (Phase 1), and a barracuda delegation path
> (Phase 2+).

**Total**: 292/292 validation checks across 28 experiments, 9 domains. 569 Rust workspace tests + 375 Python tests = 944 total.
**Rust vs Python**: 11.5× faster (excl. LAPACK-bound), 5.1× overall across all 28 experiments.
**Mathematical Parity**: 28/28 PROVEN — Python and Rust both pass against shared benchmark JSONs.
**Coverage**: Zero clippy warnings (pedantic + nursery). 95 three-tier parity tests + 9 CPU vs GPU parity.
**BarraCUDA**: 52 active delegations (35 CPU + 17 GPU), 0 pending — ToadStool S70+. GPU stats dispatch (mean, std_dev, rmse, mbe, pearson_r). Batch GPU APIs (GillespieGpu, WrightFisherGpu, BatchedElementwiseF64). Exp 009: **47.7× from Sturm tridiag**.
**Modules**: 30 (including `linalg`, `error`, `jackknife`, `freeze_out`, `spectral_recon`, `wdm`, `npu`, `biomeos`, `nestgate`).
**metalForge**: 19 workloads (17 GPU + 2 NPU), 49+ metalForge checks, 5+ substrates, architecture-aware routing. Exp 028 NPU DMA at ~51µs.
**Baseline integrity**: All 28 benchmark JSONs verified — provenance fields, hex commit hashes, UTF-8.

## Experiment Index

| ID | Title | Domain | Paper | Phase 0 | Phase 1 | Barracuda |
|----|-------|--------|-------|:-------:|:-------:|:---------:|
| 001 | [Sensor Noise Characterization](001_sensor_noise.md) | Agricultural | Dong et al. 2020 | 32/32 | 36/36 | GPU pending |
| 002 | [Observation Gap](002_observation_gap.md) | Meteorological | ERA5/NOAA | PASS | 13/13 | GPU pending |
| 003 | [Error Propagation FAO-56](003_error_propagation.md) | Agricultural | FAO-56 Paper 56 | PASS | 15/15 | Absorbed |
| 004 | [Sequencing Noise](004_sequencing_noise.md) | Biological | Synthetic community | PASS | 15/15 | WGSL ready |
| 005 | [Seismic Waves](005_seismic_waves.md) | Geological | NMSZ synthetic | PASS | 9/9 | **GPU-ready** (V31) |
| 006 | [Signal Specificity](006_signal_specificity.md) | Biological | Massie 2012 PNAS | 12/12 | 12/12 | GillespieGpu |
| 007 | [RAWR Resampling](007_rawr_resampling.md) | Statistics | Wang 2021 ISMB | 11/11 | 11/11 | Gap (RAWR) |
| 008 | [Anderson Localization](008_anderson_localization.md) | Mathematics | Bourgain-Kachkovskiy 2018 | 8/8 | 8/8 | GPU delegated |
| 009 | [Quasiperiodic Localization](009_quasiperiodic_localization.md) | Mathematics | Jitomirskaya-Kachkovskiy 2018 | 8/8 | 8/8 | GPU delegated (**49.5×**) |
| 010 | [Bistable Switching](010_bistable_switching.md) | Biological | Fernandez 2020 PNAS | 10/10 | 9/9 | ODE delegated |
| 011 | [Multi-Signal QS](011_multisignal_qs.md) | Biological | Srivastava 2011 J Bact | 9/9 | 8/8 | ODE delegated |
| 012 | [Spin Chain Transport](012_spin_chain_transport.md) | Mathematics | Kachkovskiy 2016 CMP | 18/18 | 18/18 | tridiag_eigh candidate |
| 013 | [Resampling Convergence](013_resampling_convergence.md) | Statistics | Lee & Liu 2024 IEEE BIBM | 10/10 | 8/8 | Uses bootstrap |
| 014 | [Drift vs Selection](014_drift_selection.md) | Evolutionary biology | R. Anderson 2022 mBio | 7/7 | 7/7 | Wright-Fisher, Kimura |
| 015 | [Uncertainty Bridge](015_uncertainty_bridge.md) | Cross-domain | Dong 2020 + Bourgain-Kachkovskiy 2018 | 8/8 | 8/8 | Sensor noise → Anderson ξ |
| 016 | [Rare Biosphere Signal Detection](016_rare_biosphere.md) | Biological | R. Anderson 2015 FEMS | 11/11 | 10/10 | **GPU-ready** (V31) |
| 017 | [Quasispecies Threshold](017_quasispecies_threshold.md) | Evolutionary | Dolson 2023 J R Soc | 9/9 | 6/6 | **GPU-ready** (V31) |
| 018 | [Band Edge Structure](018_band_edge_structure.md) | Mathematical | Filonov-Kachkovskiy 2018 | 8/8 | 10/10 | **GPU-ready** (V31) |
| 019 | [Jackknife Error Estimation](019_jackknife_estimation.md) | Statistics | Bazavov 2025 Phys Rev D 111 | 9/9 | 9/9 | CPU delegated |
| 020 | [Freeze-Out Inverse](020_freeze_out_inverse.md) | Inverse problems | Bazavov 2016 Phys Rev D 93 | 8/8 | 8/8 | **GPU-ready** (V31) |
| 021 | [Spectral Function Reconstruction](021_spectral_recon.md) | Inverse problems | Bazavov 2025 arXiv 2501.12259 | 8/8 | 8/8 | GPU delegated (tikhonov) |
| 022 | [ET₀ → Anderson Propagation](022_et0_anderson_propagation.md) | Cross-spring | FAO-56 + Bourgain-Kachkovskiy 2018 | 7/7 | 7/7 | Uses fao56+anderson |
| 023 | [No-Till vs Tilled Sampling](023_notill_sampling.md) | Cross-spring | R. Anderson 2015 FEMS | 7/7 | 7/7 | Uses rarefaction+rare_biosphere |
| 024 | [Aggregate Stability Noise](024_aggregate_stability.md) | Cross-spring | Nimmo & Perkins 2002 | 8/8 | 8/8 | Uses decompose+stats |
| 025 | [f32 vs f64 Precision Drift](025_f32_f64_precision.md) | WDM MD | IEEE 754-2019, Higham 2002 | 7/7 | 7/7 | Bias-variance decomposition of f32→f64 Green-Kubo integration error; bias fraction ~28% |
| 026 | [System-size Convergence](026_system_size_convergence.md) | WDM MD | Yeh & Hummer 2004 | 7/7 | 7/7 | Finite-size extrapolation D(N) = D∞ + α/N^(1/d); R² > 0.999 |
| 027 | [GPU Vendor Parity](027_vendor_parity.md) | WDM MD | hotSpring parity framework | 7/7 | 7/7 | Vendor differences at 1e-12 relative level; correlation 1.000000 |
| 028 | [NPU Anderson Classification](028_npu_anderson.md) | Hardware (NPU) | Anderson 1958; BrainChip | 7/7 | 9/9 | NPU DMA validated |

## Three-Tier Control Plan

Each experiment is validated at three levels:

1. **CPU** — Rust matches Python baseline (`cargo run --bin validate-*`)
2. **GPU** — Barracuda GPU matches CPU within tolerance (`--features barracuda-gpu`)
3. **metalForge** — Cross-substrate (GPU + NPU + CPU) agreement

Current status: **CPU complete** (292/292), **52 active delegations (35 CPU + 17 GPU),
0 pending — ToadStool S70+**. V52: 4 new CPU delegations (kimura, jackknife, fao56_et0, chao1).
19 metalForge workloads (17 GPU + 2 NPU), architecture-aware routing (V35), remote NUCLEUS discovery (V39). All delegations use sovereign fallback.
28/28 mathematical parity proven. 569 Rust workspace tests + 375 Python tests = 944 total.
**bench-cpu-vs-gpu**: Dedicated binary for CPU vs GPU performance comparison across 6 workloads.
**metalForge tier**: groundspring-forge crate with live hardware validation
(RTX 4070, Titan V, AKD1000 NPU). 4 validation binaries, 49+ metalForge checks, 5+ substrates.

Three-mode benchmarks: 20.4s → 9.2s (**2.2× speedup**); quasiperiodic 47.7×.
Cross-spring evolution (hotSpring precision + Sturm, wetSpring bio-stats,
airSpring metrics, neuralSpring dispatch) means the delegated code paths are
validated by 2,546+ barracuda tests across the ecosystem.
