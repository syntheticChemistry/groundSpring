# groundSpring → ToadStool V62 Handoff: S79 Catch-Up, Rewire, Clean

**Date**: March 2, 2026
**From**: groundSpring (V62)
**To**: ToadStool / BarraCUDA team
**ToadStool pin**: S79 (`f97fc2ae`)
**License**: AGPL-3.0-only
**Supersedes**: V61 (Mixed-Hardware Pipeline + NUCLEUS Atomics)

---

## Executive Summary

- **ToadStool pin advanced**: S71+++ (`8dc01a37`) → S79 (`f97fc2ae`) — absorbing
  3 major sessions of evolution (S78 libc→rustix + AFIT, S79 Spring absorption
  + ESN v2 shape fix + MultiHeadEsn + 5 ComputeDispatch ops, FFT buffer fix).
- **pollster eliminated**: All `pollster::block_on` calls replaced with
  `barracuda::device::test_pool::tokio_block_on` — aligning with ToadStool S74
  which removed pollster from barracuda entirely.
- **f64-capable device**: `WgpuDevice::new()` → `WgpuDevice::new_f64_capable()`
  with fallback — uses barracuda's device registry and runtime f64 probe cache
  (groundSpring V35/V37 NVK discovery, wired into barracuda S72).
- **Redundant shaders removed**: `mc_et0_propagate.wgsl` and
  `batched_multinomial.wgsl` deleted — superseded by ToadStool's precision-aware
  `_f64` versions with DF64 fallback. Anderson Lyapunov shaders retained (unique).
- **710 workspace tests, 0 failures**, all quality gates green.
- **61 active delegations** (unchanged from V61).

---

## Part 1: Cross-Spring Shader Evolution Lineage

The ToadStool S72–S79 evolution demonstrates how cross-spring contributions
compound. Each spring's domain expertise strengthens shaders used by all.

### hotSpring → Precision Shaders

hotSpring's condensed matter physics (lattice QCD, HMC, eigensolvers) drove
the f64 precision infrastructure that benefits all springs:

| Shader / Component | hotSpring Origin | Cross-Spring Benefit |
|---|---|---|
| `math_f64.wgsl` (840 lines) | S26: nuclear eigensolvers needed f64 transcendentals | All springs use exp/log/sin/cos/gamma/erf at f64 |
| `df64_transcendentals.wgsl` | S68: NVK/NAK f64 gap required double-float emulation | Consumer GPU support for all springs (GTX 1650, RTX 4060) |
| `Fp64Strategy` (Native/Hybrid/Concurrent) | S68: Dual GPU pipeline needed strategy routing | groundSpring Lyapunov, wetSpring diversity use same probe |
| NAK bypass (Sovereign SPIR-V) | S68: `from_nir.rs:430` crash on Volta/NVK | All springs' f64 shaders run on NVK via Sovereign Compiler |
| `compile_shader_f64()` / `compile_shader_df64()` | S68: universal precision pipeline | Single compile path for all 844 WGSL shaders |
| `poll_safe()` + device-loss resilience | S73: device loss under concurrent GPU tests | All barracuda consumers get graceful error propagation |

### wetSpring → Bio Shaders

wetSpring's microbial ecology and bioinformatics contributed the bio compute
kernels that groundSpring and neuralSpring both consume:

| Shader / Component | wetSpring Origin | Cross-Spring Benefit |
|---|---|---|
| `esn_reservoir_update_f64.wgsl` | V82: microbial community dynamics ESN | groundSpring regime classification, hotSpring plasma ESN |
| `kimura_fixation_f64.wgsl` | V82: population genetics | groundSpring drift.rs, quasispecies.rs |
| `batched_multinomial_f64.wgsl` | V82: rarefaction curves (skbio-compatible) | groundSpring rare_biosphere.rs rarefaction |
| `wright_fisher_step_f64.wgsl` | V82: stochastic population simulation | groundSpring quasispecies.rs |
| `hmm_forward_f64.wgsl` / `hmm_backward_log_f64.wgsl` | V82: hidden Markov models | neuralSpring MSA scoring, genomics pipelines |
| `smith_waterman_f64.wgsl` | V82: pairwise alignment | neuralSpring proteomics |
| `diversity::chao1_classic`, `shannon`, `pielou` | V82: alpha diversity | groundSpring rare_biosphere.rs |

### neuralSpring → Structural Biology Shaders

neuralSpring's protein structure prediction drove the ML and structural
compute kernels:

| Shader / Component | neuralSpring Origin | Cross-Spring Benefit |
|---|---|---|
| AlphaFold2 suite (15 DF64 shaders) | V60/V64: protein folding pipeline | ToadStool sovereign compute showcase |
| `triangle_mul_f64.wgsl` | V64: MSA attention mechanism | Architecture pattern for batched GPU attention |
| `linear_regression_f64.wgsl` | V64: statistical analysis | groundSpring stats, airSpring experiments |
| `matrix_correlation_f64.wgsl` | V64: cross-correlation analysis | groundSpring spectral analysis |
| `SimpleMLP` + `head_split` / `head_concat` | V64: neural network inference | hotSpring ESN MultiHeadEsn, groundSpring classification |

### airSpring → Earth Science Shaders

airSpring's climate and hydrology work contributed the seasonal/agricultural
compute kernels:

| Shader / Component | airSpring Origin | Cross-Spring Benefit |
|---|---|---|
| `seasonal_pipeline.wgsl` | V039: crop season phenology modeling | groundSpring fao56.rs (SeasonalPipelineF64) |
| `hargreaves_batch_f64.wgsl` | V039: ET₀ batch estimation | groundSpring fao56.rs (HargreavesBatchGpu) |
| `van_genuchten_f64.wgsl` | V039: soil moisture retention | groundSpring soil_water_balance |
| `brent_f64.wgsl` | V039: root-finding (bug fix in S72) | General-purpose optimization kernel |
| `batched_elementwise_f64` (Thornthwaite, GDD, VG) | V039: climate indices | groundSpring crop coefficient modeling |

### groundSpring → Measurement Noise Shaders

groundSpring's contributions to the ecosystem:

| Shader / Component | groundSpring Origin | Cross-Spring Benefit |
|---|---|---|
| `mc_et0_propagate_f64.wgsl` | V10: Monte Carlo uncertainty propagation | airSpring ET₀ confidence intervals |
| `rawr_weighted_mean_f64.wgsl` | V10/V54: RAWR resampling mean | wetSpring weighted diversity |
| `jackknife_mean_f64.wgsl` | V10: leave-one-out variance estimation | General-purpose cross-validation |
| `anderson_lyapunov.wgsl` (unique) | V35: transfer matrix Lyapunov exponent | Only GPU implementation; hotSpring Anderson studies |
| `boltzmann_sampling_f64.wgsl` | V54: temperature-scaled softmax sampling | neuralSpring inference sampling |
| NAK f64 probe (V35/V37 discovery) | V37: NVK advertises SHADER_F64 but fails | Core of barracuda's `probe_f64_builtins` cache |

---

## Part 2: Rewiring Changes (V62)

### 2.1 pollster → `tokio_block_on`

ToadStool S74 removed pollster from barracuda, replacing all sync→async bridges
with `barracuda::device::test_pool::tokio_block_on` which handles both sync
and tokio runtime contexts via `Handle::try_current()`.

| File | Before | After |
|---|---|---|
| `gpu.rs` | `pollster::block_on(WgpuDevice::new())` | `tokio_block_on(WgpuDevice::new_f64_capable().or(new()))` |
| `esn.rs` (3 sites) | `pollster::block_on(ESN::new/train/predict)` | `tokio_block_on(ESN::new/train/predict)` |
| `validate_metalforge_titan_v.rs` (2 sites) | `pollster::block_on(adapter.request_device)` | `tokio_block_on(adapter.request_device)` |

`pollster` dependency removed from `crates/groundspring/Cargo.toml` and
`metalForge/forge/Cargo.toml`.

### 2.2 Device Evolution: `new()` → `new_f64_capable()`

`gpu.rs::get_device()` now:
1. Tries `WgpuDevice::new_f64_capable()` — selects first GPU with working
   `SHADER_F64` from barracuda's device registry
2. Falls back to `WgpuDevice::new()` if no f64 GPU found
3. Both paths benefit from barracuda's runtime f64 probe cache (S72)

This matters because all groundSpring GPU paths (FAO-56 batch, Hargreaves
batch, grid search, ESN, correlation, fused map-reduce) require f64 precision.

### 2.3 Shader Cleanup

| Shader | Action | Reason |
|---|---|---|
| `mc_et0_propagate.wgsl` | **Removed** | Superseded by `barracuda::shaders::bio::mc_et0_propagate_f64.wgsl` (S72 `McEt0PropagateGpu`) |
| `batched_multinomial.wgsl` | **Removed** | Superseded by `barracuda::shaders::bio::batched_multinomial_f64.wgsl` (S76) |
| `anderson_lyapunov.wgsl` | **Retained** | Unique — ToadStool has no Lyapunov GPU shader |
| `anderson_lyapunov_f32.wgsl` | **Retained** | f32 fallback for NAK/NVVM; used by `validate-metalforge-titan-v` |

### 2.4 Cargo.toml Changes

`crates/groundspring/Cargo.toml`:
- Removed: `pollster = { version = "0.4", optional = true }`
- Removed: `"dep:pollster"` from `barracuda-gpu` feature

`metalForge/forge/Cargo.toml`:
- Removed: `pollster = "0.4"`
- Added: `barracuda = { path = "...", features = ["gpu"] }` (direct dep for
  `tokio_block_on` and future precision-aware dispatch)

---

## Part 3: What ToadStool S72–S79 Absorbed

| Session | Key Items Absorbed |
|---|---|
| S72 | SeasonalPipelineF64, brent_f64 bug fix, SymmetrizeGpu, LaplacianGpu, McEt0PropagateGpu, ChiSquaredBatchGpu |
| S73 | Device-loss root cause fix (`poll_safe`), compile streamlining, NAK bypass audit |
| S74 | `serde_yaml` → `serde_yaml_ng`, `async-trait` → native AFIT, capability-based evolution, God file refactoring (precision/mod.rs, workload.rs, unified.rs) |
| S75 | `primal_integration.rs` smart split, `capability_provider.rs` smart split, build streamlining |
| S76 | RAWR weighted resampling, pedotransfer, 15 DF64 folding shaders, VG/Thornthwaite/GDD, boltzmann sampling |
| S78 | libc → rustix, AFIT migration, wildcard narrowing, archive cleanup |
| S79 | ESN v2 shape fix, MultiHeadEsn (6 HeadGroups), spectral extensions, 5 ComputeDispatch ops (76 total), asin_df64 iterative fix |

---

## Part 4: Delegation Inventory (61 active, 6 pending)

### CPU Delegations (25 — barracuda feature)

`pearson_r`, `spearman_r`, `covariance`, `norm_cdf`, `norm_ppf`,
`chi2_statistic`, `rmse`, `mae`, `nash_sutcliffe`, `mbe`, `r_squared`,
`index_of_agreement`, `hit_rate`, `mean`, `sample_std_dev`, `percentile`,
`fit_linear`, `bootstrap_mean`, `rawr_mean`, `analytical_localization_length`,
`shannon_diversity`, `evenness`, `hill`, `bistable_derivative`,
`multisignal_derivative`

### GPU Delegations (7 — barracuda-gpu feature)

`lyapunov_exponent`, `lyapunov_averaged`, `level_spacing_ratio`,
`almost_mathieu_hamiltonian`, `almost_mathieu_eigenvalues`,
`detect_band_ranges`, `tikhonov_solve`

### Extended CPU Delegations (22 — S70+ absorption)

`fao56_et0`, `hargreaves_et0`, `hargreaves_et0_batch`, `crop_coefficient`,
`soil_water_balance`, `chao1_classic`, `detection_power`,
`detection_threshold`, `error_threshold`, `kimura_fixation_prob`,
`jackknife_mean_variance`, `solve_f64_cpu`, `monod`, `moving_window_stats_f64`,
`MultiSignalOde::cpu_derivative`

### GPU Ops (7 — barracuda-gpu feature, S70+ absorption)

`FusedMapReduceF64`, `SumReduceF64`, `CorrelationF64`,
`BatchedElementwiseF64`, `HargreavesBatchGpu`, `BatchedMultinomialGpu`,
`grid_search_3d`

### Not Yet Delegated (6 — unchanged from V61)

| Module | Target | Blocker |
|---|---|---|
| `gillespie::birth_death_ssa` | `GillespieGpu` | GPU-only, no CPU fallback |
| `drift::wright_fisher_fixation` | `WrightFisherGpu` | GPU dispatch, needs device |
| `spectral_recon::cholesky_solve` | `linalg::CholeskyF64` | GPU linalg, needs device |
| `transport::tridiag_eigh` | `linalg::eigh_f64` | GPU eigenvectors, needs device |
| `rarefaction::multinomial_sample` | `BatchedMultinomialGpu` | Signature mismatch |
| `prng::Xorshift64` | `PrngXoshiro` | Baseline regeneration needed |

---

## Part 5: Validation Summary

```
Rust workspace tests:  710/710 PASS (default features)
Quality gates:         fmt, clippy (pedantic+nursery), doc — all PASS
barracuda feature:     cargo check --features barracuda PASS
barracuda-gpu feature: cargo check --features barracuda-gpu PASS
All-features clippy:   cargo clippy --all-features -- -D warnings PASS
```

---

## Quality Certification

| Gate | Status |
|---|---|
| `cargo fmt --check` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | PASS |
| `cargo doc --workspace --no-deps -D warnings` | PASS |
| `cargo test --workspace` | 710/710 PASS |
| Unsafe code | 0 (workspace lint) |
| TODO/FIXME | 0 |
| Production mocks | 0 |
| pollster usage | 0 (eliminated) |
