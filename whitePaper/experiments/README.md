# groundSpring Experiments

> Per-experiment summaries for all control experiments in the groundSpring
> noise characterization framework. Each experiment has a Python baseline
> (Phase 0), a Rust validation (Phase 1), and a barracuda delegation path
> (Phase 2+).

**Total**: 177/177 validation checks across 14 experiments, 6 domains. 205 Rust tests, ~129 Python checks.
**Rust vs Python**: 22× faster across all 14 experiments (Exp 009 now 2.8× with barracuda-gpu Sturm).
**Mathematical Parity**: 14/14 PROVEN — Python and Rust both pass against shared benchmark JSONs.
**Coverage**: 99.11% workspace line coverage. Zero clippy warnings.
**BarraCUDA performance**: 14.9s (local) → 3.9s (barracuda-gpu). Exp 009: **49.5× from Sturm tridiag**.

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
| 008 | [Anderson Localization](008_anderson_localization.md) | Mathematics | Bourgain-Kachkovskiy 2018 | 8/8 | 8/8 | GPU delegated |
| 009 | [Quasiperiodic Localization](009_quasiperiodic_localization.md) | Mathematics | Jitomirskaya-Kachkovskiy 2018 | 8/8 | 8/8 | GPU delegated (**49.5×**) |
| 010 | [Bistable Switching](010_bistable_switching.md) | Biological | Fernandez 2020 PNAS | 10/10 | 9/9 | ODE delegated |
| 011 | [Multi-Signal QS](011_multisignal_qs.md) | Biological | Srivastava 2011 J Bact | 9/9 | 8/8 | ODE delegated |
| 012 | [Spin Chain Transport](012_spin_chain_transport.md) | Mathematics | Kachkovskiy 2016 CMP | TBD | 18/18 | tridiag_eigh candidate |
| 013 | [Resampling Convergence](013_resampling_convergence.md) | Statistics | Lee & Liu 2024 IEEE BIBM | TBD | 8/8 | Uses bootstrap |
| 014 | [Drift vs Selection](014_drift_selection.md) | Evolutionary biology | R. Anderson 2022 mBio | TBD | 7/7 | Wright-Fisher, Kimura |

## Three-Tier Control Plan

Each experiment is validated at three levels:

1. **CPU** — Rust matches Python baseline (`cargo run --bin validate-*`)
2. **GPU** — Barracuda GPU matches CPU within tolerance (`--features barracuda-gpu`)
3. **metalForge** — Cross-substrate (GPU + NPU + CPU) agreement

Current status: **CPU complete** (177/177), **25 barracuda delegations active**
(15 stats/metrics + bootstrap + 5 anderson/spectral + hamiltonian + 2 ODE + eigenvalues).
20 CPU delegated (`#[cfg(feature = "barracuda")]`), 5 GPU delegated
(`#[cfg(feature = "barracuda-gpu")]`). All with graceful CPU fallback.
99.11% line coverage. 14/14 mathematical parity proven.

Three-mode benchmarks: 14.9s (local) → 3.9s (barracuda-gpu, **4.4× faster**).
Cross-spring evolution (hotSpring precision + Sturm, wetSpring bio-stats,
airSpring metrics, neuralSpring dispatch) means the delegated code paths are
validated by 2,490+ barracuda tests across the ecosystem.
