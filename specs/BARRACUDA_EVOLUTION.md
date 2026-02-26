# BarraCUDA Evolution Mapping

> groundSpring Rust module → BarraCUDA primitive → WGSL shader → pipeline stage

**Last updated**: February 26, 2026 (V21 complete barracuda rewiring + dual-mode CI)

## Philosophy

groundSpring follows the **Write → Absorb → Lean** cycle established by hotSpring:

1. **Write** — Pure-Rust CPU implementations in `crates/groundspring/`.
   Production WGSL shaders in `metalForge/shaders/`.
2. **Absorb** — ToadStool/BarraCUDA absorbs shaders as upstream ops.
   Handoff via `wateringHole/handoffs/`.
3. **Lean** — groundSpring rewires to `barracuda::ops::*` behind `#[cfg(feature = "barracuda")]`.

The CPU implementations are **validation references** — they must produce
identical results within documented tolerances.  The GPU implementations
are for throughput (100k+ MC samples, batch rarefaction).

## Module Mapping

> **ToadStool catch-up (S50–S62 + DF64, Feb 23–24 2026)**: Major absorption wave.
> FAO-56 ET₀ is now a GPU op (`BatchedElementwiseF64::fao56_et0_batch`).
> Shannon entropy has a GPU convenience method. Population variance resolved.
> 5 biological ODE systems absorbed. Spearman correlation added to CPU stats.
> S59: `anderson_3d_correlated`, `anderson_sweep_averaged`, `find_w_c`,
> `ridge_regression`, `ValidationHarness` absorbed.
> S60-61: `cpu-math` feature gate (wgpu optional under `gpu` feature).
> S62: `BandwidthTier`, `PeakDetectF64`.
> Post-S62: DF64 core-streaming, `ComputeDispatch` builder.
> barracuda also has `bootstrap_mean_f64.wgsl` GPU shader (65 lines).
>
> **Complete rewiring (Feb 25 2026)**: 4 new CPU delegations added:
> `covariance`, `norm_cdf`, `norm_ppf`, `chi2_statistic`,
> `analytical_localization_length`. Total: **11 active delegations**.
> 225 Rust tests, 185/185 validation checks PASS in all three modes.
> All 11 delegations use `if let Ok` with always-compiled CPU fallback.
> Benchmarks confirm <2% overhead for compute-heavy binaries.
>
> **ToadStool S62 catch-up (Feb 25 2026)**: Reviewed full S50–S62 evolution
> (14,200+ tests, 650+ shaders). No new CPU stats primitives to wire.
> Our 11 delegations remain current. Fixed 3 `needless_return` clippy
> warnings in barracuda feature paths. Revalidated all three modes:
> 225/225 PASS (dual-mode: CPU-only + barracuda), 0 clippy warnings × 3 modes.
>
> **ToadStool S64 catch-up (Feb 26 2026)**: Session 64 absorbed
> `stats::metrics` (rmse, mbe, nash_sutcliffe, r_squared, index_of_agreement,
> hit_rate, mean, percentile, dot, l2_norm — 18 tests) and `stats::diversity`
> (shannon, simpson, chao1, bray_curtis, rarefaction_curve — 16 tests) from
> airSpring/groundSpring. Also absorbed `batched_multinomial` (GPU + CPU).
> groundSpring wired **6 new CPU delegations**: rmse, mbe, r_squared,
> index_of_agreement, hit_rate, shannon_diversity.
> Also fixed pre-existing barracuda-mode bugs: OdeSystem trait import for
> BistableOde/MultiSignalOde, hofstadter module path (now re-exported at
> spectral level), dead-code gates for local helpers.
> Total: **20 active delegations**. 0 clippy warnings × 3 modes.
>
> **Complete rewiring (Feb 26 2026)**: 4 more delegations: `mean`,
> `percentile`, `level_spacing_ratio`, `almost_mathieu_eigenvalues`
> (via `find_all_eigenvalues` Sturm tridiag solver from hotSpring S26).
> Exp 009 quasiperiodic: 11.7s → 0.23s (**50× speedup**).
> Dense Givens QR moved from validation binary to library, gated behind
> `#[cfg(not(feature = "barracuda-gpu"))]`.
> Total: **24 active delegations**. 0 clippy warnings × 3 modes.
> Three-mode benchmark: 14.5s (local) → 3.3s (barracuda-gpu).
> RAWR kernel, CPU xoshiro128**, Gillespie CPU fallback still pending.
>
> **ToadStool S66 catch-up (Feb 26 2026)**: S66 absorbed `rawr_mean` into
> `barracuda::stats::rawr_mean` (from groundSpring V15 request). Also added
> `stats::regression` (fit_linear, fit_quadratic, etc.), `stats::hydrology`
> (FAO-56), `stats::moving_window_f64`, `stats::mae`, `shannon_from_frequencies`,
> `spearman_correlation` re-export, and `hill()/monod()` public Rust APIs.
> GPU: `WrightFisherGpu` (batched drift+selection), `eigh_f64` (dense eigenvectors),
> `BatchedEighGpu`, sovereign compiler fixes for `BatchedElementwiseF64`.
> groundSpring wired **1 new CPU delegation**: `rawr_mean` (#26).
> Fixed `bootstrap ≠ RAWR` comparison test for barracuda parity (both methods
> converge to sample mean on small symmetric data).
> Total: **27 active delegations** (22 CPU + 5 GPU). 0 clippy warnings × 3 modes.
> 225 tests, 185/185 validation checks.
>
> **Deep debt evolution (Feb 26 2026)**: Eliminated all 20 `#[allow(unreachable_code)]`
> via proper `#[cfg]`/`#[cfg(not)]` mutual exclusion. Fixed covariance/pearson_r/
> spearman_r bug — now fall through to CPU on barracuda error instead of returning
> 0.0. `BistableParams`/`MultiSignalParams` derive `Copy` (no more `.clone()`).
> 7 magic numbers extracted as named constants. Misleading delegation docs fixed
> in transport.rs, drift.rs, gillespie.rs.
>
> **Idiomatic Rust evolution (Feb 26 2026)**: New `kinetics` module extracts
> `hill()` / `hill_repress()` from bistable + multisignal with barracuda
> barracuda delegation (hill is live) (`barracuda::stats::hill`). All `Vec<Vec<f64>>` eliminated —
> `almost_mathieu.rs` QR and `transport.rs` eigenvectors refactored to flat
> row-major `Vec<f64>` (GPU-promotable layout). 13 bitwise determinism tests
> added. All 15 benchmark JSONs have DOIs and stamped `baseline_commit`.
> CI now runs all 15 validation binaries. 225 tests, 98.93% llvm-cov.
> **V20 (Feb 26 2026)**: Hill delegation #27 LIVE. ToadStool S68 (f0feb226). 700 shaders (zero f32-only), 2,546+ tests, 21,599 workspace tests. `hill_repress` → `1.0 - hill()`.
>
> **V21 (Feb 26 2026)**: Complete barracuda rewiring. `--features barracuda` compiles cleanly (zero warnings both modes). Dual-mode CI: `cargo clippy` and `cargo test` run with and without barracuda feature. 225/225 tests pass in both CPU-only and barracuda-delegated modes. Domain guard fix for hill (biological convention before delegation). 17 `_cpu` functions properly gated behind `#[cfg(not(feature = "barracuda"))]`. CPU delegation overhead: +1.7% total.

### Tier A — Lean (rewire to existing barracuda ops)

| groundSpring Module | BarraCUDA Op | Status | How |
|---|---|---|---|
| `stats::pearson_r` | `stats::pearson_correlation` | **DONE** (CPU delegated) | NaN-safe wrapper |
| `stats::spearman_r` | `stats::correlation::spearman_correlation` | **DONE** (CPU delegated) | NaN-safe wrapper |
| `stats::sample_std_dev` | `stats::correlation::std_dev` | **DONE** (CPU delegated) | Bessel-corrected |
| `stats::covariance` | `stats::correlation::covariance` | **DONE** (CPU delegated) | Sample covariance |
| `stats::norm_cdf` | `stats::norm_cdf` | **DONE** (CPU delegated) | Standard normal Φ(x) |
| `stats::norm_ppf` | `stats::norm_ppf` | **DONE** (CPU delegated) | Inverse Φ⁻¹(p) (Acklam) |
| `stats::chi2_statistic` | `stats::chi2_decomposed` | **DONE** (CPU delegated) | Goodness-of-fit |
| `bootstrap::bootstrap_mean` | `stats::bootstrap_mean` | **DONE** (CPU delegated) | Result struct mapping |
| `anderson::lyapunov_exponent` | `spectral::lyapunov_exponent` | **DONE** (barracuda-gpu) | Transfer matrix method |
| `anderson::lyapunov_averaged` | `spectral::lyapunov_averaged` | **DONE** (barracuda-gpu) | Multi-realization average |
| `anderson::analytical_localization_length` | `special::anderson_transport::localization_length` | **DONE** (CPU delegated) | Perturbative ξ(W,E) |
| `anderson::almost_mathieu_hamiltonian` | `spectral::almost_mathieu_hamiltonian` | **DONE** (barracuda-gpu) | λ/2 coupling convention |
| `bistable::bistable_derivative` | `numerical::ode_bio::BistableOde::cpu_derivative` | **DONE** (CPU delegated) | OdeSystem trait |
| `multisignal::multisignal_derivative` | `numerical::ode_bio::MultiSignalOde::cpu_derivative` | **DONE** (CPU delegated) | OdeSystem trait |
| `stats::rmse` | `stats::metrics::rmse` | **DONE** (CPU delegated) | S64 absorption |
| `stats::mbe` | `stats::metrics::mbe` | **DONE** (CPU delegated) | S64 absorption |
| `stats::r_squared` | `stats::metrics::r_squared` | **DONE** (CPU delegated) | S64 absorption |
| `stats::index_of_agreement` | `stats::metrics::index_of_agreement` | **DONE** (CPU delegated) | S64 absorption |
| `stats::hit_rate` | `stats::metrics::hit_rate` | **DONE** (CPU delegated) | S64 absorption |
| `rarefaction::shannon_diversity` | `stats::diversity::shannon` | **DONE** (CPU delegated) | u64→f64 conversion |
| `stats::mean` | `stats::metrics::mean` | **DONE** (CPU delegated) | S64 absorption |
| `stats::percentile` | `stats::metrics::percentile` | **DONE** (CPU delegated) | S64 absorption |
| `anderson::level_spacing_ratio` | `spectral::level_spacing_ratio` | **DONE** (barracuda-gpu) | Sort adapter |
| `anderson::almost_mathieu_eigenvalues` | `spectral::find_all_eigenvalues` | **DONE** (barracuda-gpu) | **49.5× Exp 009 speedup** — Sturm tridiag |
| `rarefaction::evenness` | `stats::pielou_evenness` | **DONE** (CPU delegated) | S≤1 semantic adapter (ecology convention) |
| `bootstrap::rawr_mean` | `stats::rawr_mean` | **DONE** (CPU delegated) | S66 absorption — Dirichlet-weighted mean |
| `kinetics::hill` | `stats::hill` | **DONE** (CPU delegated) | S68 absorption — infallible `#[cfg]`/`#[cfg(not)]` |
| `kinetics::hill_repress` | `stats::hill` (1 − hill) | **DONE** (CPU delegated) | Composes `1.0 - hill(x, k, n)` — gets barracuda delegation for free |

### Tier B — Adapt (needs alignment or wrapper)

| groundSpring Module | BarraCUDA Target | Blocker | Action |
|---|---|---|---|
| `prng::Xorshift64` | `ops::PrngXoshiro` (f64) | Different PRNG algorithm | Align to xoshiro; retain xorshift as CPU reference |
| `seismic::grid_search_inversion` | Parallel grid dispatch | No existing grid-search op | Dispatch as 3D workgroup; reduce min RMS |
| `rarefaction::multinomial_sample` | `ops::PrngXoshiro` + binary search | No batched multinomial | Production WGSL in metalForge |
| `gillespie::birth_death_ssa` | `ops::bio::GillespieGpu` | GPU-only (no CPU fallback) | Write CPU → GPU dispatch when adapter ready |
| ~~`bootstrap::rawr_mean`~~ | ~~New: `ops::rawr_weighted_mean_f64`~~ | **RESOLVED** — absorbed as `stats::rawr_mean` in S66 | Moved to Tier A (#26) |
| `anderson::anderson_potential` | `spectral::anderson_potential` | Requires `barracuda-gpu` feature | Align PRNG seeds |

### Tier C — ~~New Kernel Required~~ → Partially Absorbed

| Proposed Kernel | Status | Notes |
|---|---|---|
| `ops::mc_et0_propagate_f64` | **SUPERSEDED** — `BatchedElementwiseF64::fao56_et0_batch()` already in barracuda | ToadStool absorbed FAO-56 as `Op::Fao56Et0` with 9-input batch (tmax, tmin, rh_max, rh_min, wind, Rs, elev, lat, doy). GPU + CPU reference. groundSpring's `mc_et0_propagate.wgsl` MC wrapper remains valuable for uncertainty propagation. |
| `ops::batched_multinomial_f64` | **ABSORBED** — `BatchedMultinomialGpu` + `multinomial_sample_cpu` in barracuda S64 | groundSpring rewiring deferred (signature mismatch: barracuda takes cumulative_probs + closure RNG) |

### Stays Local (no GPU benefit)

| Module | Reason |
|---|---|
| `decompose::decompose_error` | Two scalar ops: bias² = MBE², var = RMSE² - MBE² |
| `decompose::noise_floor_reduction` | Three scalar ops |
| `validate::ValidationHarness` | Harness, not compute. Equivalent to `barracuda::validation::ValidationHarness` but with groundSpring-specific method names |
| `seismic::haversine_km` | Single scalar trig |
| `seismic::travel_time_1d` | One sqrt + division |

### New Modules (Exp 012, 013, 014) — Future BarraCUDA Candidates

| Module | Key Functions | BarraCUDA Potential |
|---|---|---|
| `transport` | `tridiag_eigh` | New eigenvector primitive for spin chain transport — future barracuda candidate |
| `drift` | `wright_fisher_fixation`, `kimura_fixation_prob` | Wright-Fisher and Kimura fixation probabilities — future barracuda candidates |
| `prng` | `binomial()` | Added for drift/selection experiments; complements existing normal sampling |

### New barracuda ops relevant to groundSpring (discovered S51–S62+)

| Op | Module | groundSpring Use |
|---|---|---|
| `BatchedElementwiseF64::fao56_et0_batch` | `ops::batched_elementwise_f64` | FAO-56 ET₀ batch compute — **supersedes our Tier C shader** |
| `FusedMapReduceF64::shannon_entropy` | `ops::fused_map_reduce_f64` | Shannon diversity — **convenience method, GPU-ready** |
| `FusedMapReduceF64::simpson_index` | `ops::fused_map_reduce_f64` | Simpson diversity — bonus |
| `VarianceReduceF64::population_variance` | `ops::variance_reduce_f64` | Population variance — **resolves semantics mismatch** |
| `VarianceReduceF64::population_std` | `ops::variance_reduce_f64` | Population std dev |
| `spearman_correlation` | `stats::correlation` | **DONE** — groundSpring delegates |
| `CapacitorOde`, `CooperationOde`, etc. | `numerical::ode_bio` | Waters paper-ready ODE systems |
| `NmfResult` | `linalg::nmf` | NMF for R. Anderson metagenomics |
| `anderson_3d_correlated` | `spectral` | S59 — correlated disorder (Méndez-Bermúdez) |
| `anderson_sweep_averaged` | `spectral` | S59 — disorder sweep ⟨r⟩(W) with stderr |
| `find_w_c` | `spectral` | S59 — critical disorder interpolation |
| `ridge_regression` | `linalg` | S59 — Tikhonov regression from ESN readout |
| `PeakDetectF64` | `ops` | S62 — GPU local-maxima with prominence |
| `BandwidthTier` | `dispatch` | S62 — PCIe/NvLink bandwidth-aware routing |
| `stats::rawr_mean` | `stats::bootstrap` | S66 — RAWR Dirichlet-weighted bootstrap (**DONE**, delegation #26) |
| `stats::regression` | `stats::regression` | S66 — `fit_linear`, `fit_quadratic`, `fit_exponential`, `fit_logarithmic` |
| `stats::hydrology` | `stats::hydrology` | S66 — FAO-56 `hargreaves_et0`, `crop_coefficient`, `soil_water_balance` |
| `stats::moving_window_f64` | `stats::moving_window_f64` | S66 — CPU sliding-window mean/var/min/max |
| `stats::mae` | `stats::metrics` | S66 — Mean Absolute Error |
| `WrightFisherGpu` | `ops::bio` | S66 — Batched drift+selection GPU (future Exp 014 GPU delegation) |
| `eigh_f64` / `BatchedEighGpu` | `linalg` | S66 — Dense symmetric eigendecomposition (future Exp 012 GPU delegation) |

## Feature Gate

```toml
# Cargo.toml
[features]
default = []
barracuda = ["dep:barracuda"]
barracuda-gpu = ["barracuda", "barracuda/gpu"]

[dependencies]
barracuda = { path = "../../../phase1/toadstool/crates/barracuda", optional = true, default-features = false }
```

Two feature gates:
- `barracuda` — enables CPU delegation (`stats::bootstrap_mean`, `pearson_r`, etc.)
- `barracuda-gpu` — enables GPU + spectral delegation (`anderson::lyapunov_*`, `GillespieGpu`)

The CPU implementation remains the default and the validation reference.

## What NOT to Duplicate

BarraCUDA primitives that already exist and MUST NOT be reimplemented:

- Variance computation (`ops::variance_f64_wgsl`)
- Covariance (`ops::covariance_f64_wgsl`)
- Correlation (`ops::correlation_f64_wgsl`)
- Fused map-reduce (`ops::fused_map_reduce_f64`)
- Nelder-Mead optimization (`optimize::nelder_mead_gpu`)
- PRNG (`ops::prng_xoshiro_wgsl`)
- Shannon/Bray-Curtis diversity (`ops::bray_curtis_f64`)
- Norm reduce (`ops::norm_reduce_f64`)
- Sum reduce (`ops::sum_reduce_f64`)
- Bootstrap mean/std (`stats::bootstrap_mean`, `stats::bootstrap_std`)
- RAWR mean (`stats::rawr_mean`)

groundSpring's CPU implementations serve as **validation references** for
these GPU kernels.

## GPU Promotion Blockers

1. **f64 precision** — groundSpring requires f64 for statistical validity.
   All WGSL shaders must use f64 or df64 (double-float f32-pair per
   hotSpring's `df64_core.wgsl` pattern).
2. **Multinomial sampling kernel** — does not exist in BarraCUDA.
   Production WGSL in `metalForge/shaders/batched_multinomial.wgsl`.
3. **FAO-56 MC wrapper kernel** — equation chain absorbed upstream as
   `Op::Fao56Et0`; MC noise wrapper (Box-Muller perturbation + dispatch) in
   `metalForge/shaders/mc_et0_propagate.wgsl`.
4. **PRNG alignment** — xorshift64 ↔ xoshiro128** produces different
   streams. Need to regenerate baselines after alignment.

## PRNG Alignment Roadmap

groundSpring currently uses `prng::Xorshift64` (Marsaglia 2003) — simple
and deterministic, but not the same generator as BarraCUDA's `PrngXoshiro`
(xoshiro128**).  Alignment is required before Tier B GPU promotion to ensure
GPU and CPU paths produce bitwise-identical streams.

### Semantic Mismatch

| Property | groundSpring `Xorshift64` | BarraCUDA `PrngXoshiro` |
|----------|--------------------------|------------------------|
| State size | 64 bits | 256 bits |
| Period | 2⁶⁴ − 1 | 2²⁵⁶ − 1 |
| Output bits | 64 | 64 |
| Quality | Fails BigCrush | Passes BigCrush |
| GPU kernel | None | `prng_xoshiro_wgsl` |

### Migration Steps (Phase 2b)

1. **Feature-gate the PRNG** — add `#[cfg(feature = "barracuda")]` path
   that delegates to `barracuda::ops::PrngXoshiro` for stream generation.
   Keep `Xorshift64` as the default (CPU reference).
2. **Create `prng::Xoshiro128` wrapper** — thin CPU-side wrapper around
   barracuda's generator with `next_u64()` and `next_normal()` matching
   the existing API surface.
3. **Regenerate all baselines** — rerun Python baselines with a compatible
   xoshiro128** implementation (e.g., a pure-Python xoshiro128** port).
4. **Update benchmark JSONs** — new expected values, new `baseline_commit`,
   new `baseline_date`, xoshiro128** noted in `prng_algorithm` field.
5. **Verify 177/177 checks** — run full validation suite.
6. **Remove old baselines** — archive xorshift64 baselines in
   `control/archive/xorshift64/` for fossil record.

### Variance Semantics

groundSpring `std_dev` uses **population variance** (÷ N); BarraCUDA
`stats::std_dev` uses **sample variance** (÷ N−1).  groundSpring now
provides both:

- `stats::std_dev` — population (canonical for RMSE decomposition)
- `stats::sample_std_dev` — Bessel-corrected (compatible with BarraCUDA)

The `stats::pearson_r` function is already wired to
`barracuda::stats::pearson_correlation` under `#[cfg(feature = "barracuda")]`.

## Local vs BarraCUDA CPU Delegation Performance

Best-of-3 benchmark (release mode, Feb 25 2026, post-S62 barracuda):

| Binary | Local (ms) | BarraCUDA (ms) | BarraCUDA-GPU (ms) | Overhead |
|--------|-----------|---------------|-------------------|----------|
| validate-anderson | 671 | 670 | 640 | **−5%** |
| validate-decompose | 5 | 4 | 5 | noise |
| validate-fao56 | 12 | 12 | 13 | noise |
| validate-rarefaction | 11 | 12 | 12 | noise |
| validate-rawr | 555 | 560 | 556 | **<1%** |
| validate-seismic | 56 | 59 | 58 | noise |
| validate-signal-specificity | 795 | 787 | 787 | **−1%** |
| validate-weather | 3 | 3 | 5 | noise |
| **TOTAL** | **2108** | **2107** | **2076** | **~0%** |

Zero measurable overhead. The S62 barracuda (with `cpu-math` modularization
and dead-code elimination) links with essentially zero runtime cost. Anderson
is slightly faster with barracuda-gpu due to optimized transfer-matrix cache
behavior in `spectral::lyapunov_averaged`.

## Rust vs Python Performance (Phase 1c — Full Suite)

Pure Rust CPU math vs interpreted Python (NumPy/SciPy), median of 3 trials
across all 15 experiments:

| Experiment | Python (s) | Rust (s) | Speedup |
|---|---|---|---|
| Exp 001: Sensor Noise | 0.64 | 0.11 | **5.7×** |
| Exp 002: Observation Gap | 0.28 | 0.07 | **4.4×** |
| Exp 003: Error Propagation | 0.36 | 0.10 | **3.8×** |
| Exp 004: Sequencing Noise | 0.14 | 0.08 | **1.8×** |
| Exp 005: Seismic Inversion | 7.63 | 0.12 | **63.6×** |
| Exp 006: Signal Specificity (Gillespie SSA) | 26.78 | 0.88 | **30.5×** |
| Exp 007: RAWR Resampling (bootstrap) | 4.64 | 0.63 | **7.3×** |
| Exp 008: Anderson Localization (transfer matrix) | 21.98 | 0.73 | **29.9×** |
| Exp 009: Quasiperiodic (Almost-Mathieu) | 0.65 | 0.23 * | **2.8×** |
| Exp 010: Bistable Switching (ODE) | 3.58 | 0.19 | **18.5×** |
| Exp 011: Multi-Signal QS (ODE) | 4.30 | 0.09 | **46.2×** |
| **Total** | **70.98** | **3.23** | **22.0×** |

\* With barracuda-gpu (Sturm tridiag). Without: 11.7s (dense QR). **49.5× speedup.**

**Note on Exp 009**: With barracuda-gpu, the Sturm tridiag eigenvalue solver
(from hotSpring S26 spectral module) exploits the tridiagonal structure of
the Almost-Mathieu Hamiltonian. Without barracuda, the custom dense Givens QR
still works but is O(n³) vs O(n²). Python delegates to numpy/LAPACK for
this workload.  Barracuda GPU kernels will close this gap.

Speedup varies with algorithm type:
- **Branching loops** (Gillespie, Anderson, seismic grid search): 30–64×
- **ODE integration** (bistable, multisignal): 18–46×
- **Vectorized ops** (RAWR, sensor noise): 4–7×
- **Lightweight checks** (sequencing noise, error propagation): 2–4×

### Mathematical Parity Certificate

All 15 experiments: **PARITY PROVEN**.  Both Python baselines and Rust
validation binaries check against the same shared benchmark JSON files.
If both pass, the math is identical within stated tolerances.

See `data/parity_report.json` for the machine-readable certificate.

## Timeline

| Phase | Milestone | Status |
|---|---|---|
| Phase 0 | Python baselines | **Done** (~137 checks across 15 experiments) |
| Phase 1a | Rust CPU validation | **Done** (185/185 PASS across 15 binaries) |
| Phase 1b | metalForge production WGSL | **Done** (2 production shaders, 261 combined lines) |
| Phase 1c | Paper queue buildout (Exp 006-014) | **Done** (33 new checks for Exp 012-014, 23.4× faster than Python) |
| Phase 1d | Full-suite parity + benchmarks | **Done** (15/15 parity proven, timing data for all experiments) |
| Phase 2a | Tier A rewire (stats + bootstrap + anderson → barracuda) | **27 delegated** (15 stats + bootstrap_mean + rawr_mean + hill + 5 anderson + analytical ξ + hamiltonian + 2 ODE + shannon + eigenvalues) |
| Phase 2b | Tier B adapt (PRNG alignment, grid dispatch, gillespie GPU) | After 2a |
| Phase 2c | Tier C absorption (multinomial, RAWR kernels) | After 2b |
| Phase 3 | Full GPU pipeline, metalForge cross-substrate | After Phase 2 |

## Cross-Reference

- `metalForge/ABSORPTION_MANIFEST.md` — Detailed absorption inventory
- `metalForge/shaders/` — Production WGSL shaders
- `specs/BARRACUDA_REQUIREMENTS.md` — GPU kernel gap analysis
