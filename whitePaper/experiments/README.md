# groundSpring Experiments

> Per-experiment summaries for all control experiments in the groundSpring
> noise characterization framework. Each experiment has a Python baseline
> (Phase 0), a Rust validation (Phase 1), and a barracuda delegation path
> (Phase 2+).

**Total**: 119/119 validation checks across 8 experiments, 6 domains.
**Rust vs Python**: 24× faster (Exp 006-008 benchmark).

## Experiment Index

| ID | Title | Domain | Paper | Phase 0 | Phase 1 | Barracuda |
|----|-------|--------|-------|:-------:|:-------:|:---------:|
| 001 | [Sensor Noise Characterization](001_sensor_noise.md) | Agricultural | Dong et al. 2020 | 32/32 | 36/36 | GPU pending |
| 002 | [Observation Gap](002_observation_gap.md) | Meteorological | ERA5/NOAA | PASS | 13/13 | GPU pending |
| 003 | [Error Propagation FAO-56](003_error_propagation.md) | Agricultural | FAO-56 Paper 56 | PASS | 15/15 | Absorbed |
| 004 | [Sequencing Noise](004_sequencing_noise.md) | Biological | Synthetic community | PASS | 15/15 | WGSL ready |
| 005 | [Seismic Waves](005_seismic_waves.md) | Geological | NMSZ synthetic | PASS | 9/9 | GPU pending |
| 006 | [Signal Specificity](006_signal_specificity.md) | Biological | Massie 2012 PNAS | 12/12 | 12/12 | GillespieGpu |
| 007 | [RAWR Resampling](007_rawr_resampling.md) | Statistics | Wang 2021 ISMB | 11/11 | 11/11 | Gap (RAWR) |
| 008 | [Anderson Localization](008_anderson_localization.md) | Mathematics | Bourgain-Kachkovskiy 2018 | 8/8 | 8/8 | CPU delegated |

## Three-Tier Control Plan

Each experiment is validated at three levels:

1. **CPU** — Rust matches Python baseline (`cargo run --bin validate-*`)
2. **GPU** — Barracuda GPU matches CPU within tolerance (`--features barracuda-gpu`)
3. **metalForge** — Cross-substrate (GPU + NPU + CPU) agreement

Current status: **CPU complete** (119/119), GPU and metalForge pending.
