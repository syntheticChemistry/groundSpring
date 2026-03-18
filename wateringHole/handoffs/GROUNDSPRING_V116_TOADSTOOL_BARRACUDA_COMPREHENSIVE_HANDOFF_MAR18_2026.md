# groundSpring V116 → toadStool / barraCuda Comprehensive Absorption Handoff

**Date**: March 18, 2026
**From**: groundSpring V116
**To**: toadStool, barraCuda, coralReef
**Covers**: Full delegation inventory, evolution opportunities, absorption guidance, V116 learnings
**Pins**: barraCuda v0.3.5, toadStool S158+, coralReef Iteration 55+
**License**: AGPL-3.0-or-later

---

## Executive Summary

- groundSpring V116 has **102 active barraCuda delegations** (61 CPU + 41 GPU) across 46 modules with ~250+ `barracuda::` references, all with graceful CPU fallback
- Three typed error enums (`DispatchError`, `EsnError`, `ResilienceError<E>`) replace all opaque `Result<_, String>` in the dispatch and IPC layers — recommended pattern for toadStool
- `OnceLock` GPU probe cache prevents `SIGSEGV` from concurrent `wgpu::Instance` creation in parallel tests — critical fix for any crate creating GPU instances in `#[test]`
- Two pending evolution items: PRNG alignment (Xorshift64 → xoshiro128\*\*) and `tridiag_eigh` eigenvectors
- Capability advertisement now supports 4 formats (A–D) + `"methods"` wrapper — toadStool/barraCuda should advertise in Format C/D for richer metadata

---

## Part 1: Delegation Inventory (102 Active)

### CPU Delegations (61) — `#[cfg(feature = "barracuda")]`

| # | groundSpring Function | barraCuda Target | Module |
|---|----------------------|-----------------|--------|
| 1 | `stats::pearson_r` | `stats::pearson_correlation` | stats/correlation |
| 2 | `stats::spearman_r` | `stats::correlation::spearman_correlation` | stats/correlation |
| 3 | `stats::sample_std_dev` | `stats::correlation::std_dev` | stats/correlation |
| 4 | `stats::covariance` | `stats::correlation::covariance` | stats/correlation |
| 5 | `stats::norm_cdf` | `stats::norm_cdf` | stats/distributions |
| 6 | `stats::norm_ppf` | `stats::norm_ppf` | stats/distributions |
| 7 | `stats::chi2_statistic` | `stats::chi2_decomposed` | stats/distributions |
| 8 | `stats::rmse` | `stats::metrics::rmse` | stats/agreement |
| 9 | `stats::mbe` | `stats::metrics::mbe` | stats/agreement |
| 10 | `stats::r_squared` | `stats::metrics::r_squared` | stats/agreement |
| 11 | `stats::index_of_agreement` | `stats::metrics::index_of_agreement` | stats/agreement |
| 12 | `stats::hit_rate` | `stats::metrics::hit_rate` | stats/agreement |
| 13 | `stats::mae` | `stats::metrics::mae` | stats/agreement |
| 14 | `stats::nash_sutcliffe` | `stats::metrics::nash_sutcliffe` | stats/agreement |
| 15 | `stats::mean` | `stats::metrics::mean` | stats/metrics |
| 16 | `stats::percentile` | `stats::metrics::percentile` | stats/metrics |
| 17 | `stats::fit_linear` | `stats::regression::fit_linear` | stats/regression |
| 18 | `stats::fit_quadratic` | `stats::regression::fit_quadratic` | stats/regression |
| 19 | `stats::fit_exponential` | `stats::regression::fit_exponential` | stats/regression |
| 20 | `stats::fit_logarithmic` | `stats::regression::fit_logarithmic` | stats/regression |
| 21 | `bootstrap::bootstrap_mean` | `stats::bootstrap_mean` | bootstrap |
| 22 | `rawr::rawr_mean` | `stats::rawr_mean` | rawr |
| 23 | `rarefaction::shannon_diversity` | `stats::diversity::shannon` | rarefaction |
| 24 | `rarefaction::evenness` | `stats::pielou_evenness` | rarefaction |
| 25 | `anderson::analytical_localization_length` | `special::anderson_transport::localization_length` | anderson |
| 26 | `bistable::bistable_derivative` | `numerical::ode_bio::BistableOde::cpu_derivative` | bistable |
| 27 | `multisignal::multisignal_derivative` | `numerical::ode_bio::MultiSignalOde::cpu_derivative` | multisignal |
| 28 | `kinetics::hill` | `stats::hill` | kinetics |
| 29 | `drift::kimura_fixation_prob` | `stats::evolution::kimura_fixation_prob` | drift |
| 30 | `jackknife::jackknife_mean_variance` | `stats::jackknife::jackknife_mean_variance` | jackknife |
| 31 | `fao56::daily_et0` | `stats::hydrology::fao56_et0` | fao56 |
| 32 | `fao56::hargreaves_et0` | `stats::hydrology::hargreaves_et0` | fao56 |
| 33 | `fao56::crop_coefficient` | `stats::hydrology::crop_coefficient` | fao56 |
| 34 | `fao56::soil_water_balance` | `stats::hydrology::soil_water_balance` | fao56 |
| 35 | `freeze_out::chi2_decomposed_weighted` | `stats::chi2::chi2_decomposed_weighted` | freeze_out |
| 36 | `rare_biosphere::chao1` | `stats::diversity::chao1_classic` | rare_biosphere |
| 37 | `rare_biosphere::detection_power` | `stats::evolution::detection_power` | rare_biosphere |
| 38 | `rare_biosphere::detection_threshold` | `stats::evolution::detection_threshold` | rare_biosphere |
| 39 | `quasispecies::error_threshold` | `stats::evolution::error_threshold` | quasispecies |
| 40–61 | *(remaining: moving_window, anderson spectral, etc.)* | *(see BARRACUDA_EVOLUTION.md for full table)* | various |

### GPU Delegations (41) — `#[cfg(feature = "barracuda-gpu")]`

| # | groundSpring Function | barraCuda GPU Dispatch | Speedup |
|---|----------------------|-----------------------|---------|
| 1 | `anderson::lyapunov_exponent` | `spectral::lyapunov_exponent` | native |
| 2 | `anderson::lyapunov_averaged` | `spectral::lyapunov_averaged` | native |
| 3 | `anderson::anderson_2d/3d/4d` | `spectral::anderson_*` | native |
| 4 | `anderson::wegner_block_4d` | `spectral::anderson::wegner_block_4d` | native |
| 5 | `almost_mathieu::hamiltonian` | `spectral::almost_mathieu_hamiltonian` | **47.4×** |
| 6 | `almost_mathieu::eigenvalues` | `spectral::find_all_eigenvalues` | **49.5×** |
| 7 | `almost_mathieu::level_spacing_ratio` | `spectral::level_spacing_ratio` | native |
| 8 | `gillespie::birth_death_ssa_batch` | `ops::bio::GillespieGpu` | batch |
| 9 | `drift::wright_fisher_fixation_batch` | `ops::bio::WrightFisherGpu` | batch |
| 10 | `rarefaction::multinomial_sample_batch` | `ops::bio::BatchedMultinomialGpu` | batch |
| 11 | `fao56::mc_et0_propagate` | `McEt0PropagateGpu` | batch |
| 12 | `fao56::seasonal_pipeline` | `SeasonalPipelineF64` | batch |
| 13 | `spectral_recon::tikhonov_solve` | `linalg::solve_f64_cpu` | native |
| 14 | `freeze_out::grid_search_3d` | `ops::grid::grid_search_3d` | parallel |
| 15 | `esn::EsnClassifier` | `esn_v2::ESN` | native |
| 16 | `lanczos::sparse_eigenvalues` | `spectral::lanczos` | native |
| 17–41 | *(GPU stats: SumReduce, VarianceReduce, FusedMapReduce, Correlation, Covariance, Autocorrelation, PeakDetect, etc.)* | *(ops reduce/correlation shaders)* | kernel |

### Delegation Pattern

All delegations follow the same safe pattern:

```rust
#[cfg(feature = "barracuda")]
{
    if let Ok(result) = barracuda::stats::pearson_correlation(x, y) {
        return result;
    }
}
pearson_r_cpu(x, y) // always-compiled fallback
```

Zero `.expect()` or `.unwrap()` on barracuda calls in production code.

---

## Part 2: V116 Typed Error Evolution (toadStool Action Items)

### What Changed

| Component | Before (V115) | After (V116) |
|-----------|--------------|--------------|
| `dispatch::dispatch()` | `Result<Value, String>` | `Result<Value, DispatchError>` |
| `serve_one()` | `F: Fn -> Result<Value, String>` | `F: Fn -> Result<Value, E: Display>` |
| `EsnClassifier` | `Result<_, String>` | `Result<_, EsnError>` with `#[source] BarracudaError` |
| `resilient_call()` | `Result<T, String>` | `Result<T, ResilienceError<E>>` |

### toadStool action: Adopt typed dispatch errors

If toadStool has a dispatch layer returning `Result<Value, String>`, the same
`thiserror` enum pattern eliminates opaque error strings:

```rust
#[derive(Debug, thiserror::Error)]
pub enum DispatchError {
    #[error("method not found: {0}")]
    MethodNotFound(String),
    #[error("missing parameter: {0}")]
    MissingParam(String),
    #[error("invalid parameter: {0}")]
    InvalidParam(String),
    #[error(transparent)]
    Input(#[from] InputError),
}
```

### toadStool action: Adopt OnceLock for GPU singletons

The `SIGSEGV` from parallel `wgpu::Instance` creation in tests is fixed with:

```rust
static GPU_PROBE_CACHE: OnceLock<Vec<Substrate>> = OnceLock::new();

pub fn probe_gpus() -> Vec<Substrate> {
    GPU_PROBE_CACHE.get_or_init(probe_gpus_inner).clone()
}
```

Any code creating `wgpu::Instance` in `#[test]` should apply this pattern.

### toadStool action: Preserve error source chains

`EsnError` preserves the `BarracudaError` source via `#[source]` instead of
flattening to `format!("{e}")`. This enables callers to inspect the root cause:

```rust
#[derive(Debug, thiserror::Error)]
pub enum EsnError {
    #[error("ESN init failed: {0}")]
    Init(#[source] barracuda::error::BarracudaError),
    // ...
}
```

---

## Part 3: Pending Evolution Items

### P0: PRNG Alignment (Xorshift64 → xoshiro128**)

groundSpring uses `prng::Xorshift64` for all stochastic operations.
barraCuda uses `PrngXoshiro` (xoshiro128**). Alignment requires:

1. barraCuda exposes `prng::Xoshiro128StarStar` with a stable Rust API
2. groundSpring regenerates all benchmark JSON baselines with the new PRNG
3. All 13 bitwise determinism tests must pass with the new stream

**Impact**: Every stochastic experiment (007, 010, 011, 014, 015, 016, 017) needs
baseline regeneration. This is a coordinated cross-spring effort.

**Recommendation**: Schedule this as a joint sprint. groundSpring will provide
the list of affected benchmarks and expected test count impact.

### P1: Eigenvector Support for `tridiag_eigh`

barraCuda has eigenvalues via Sturm bisection but eigenvectors are not yet exposed.
groundSpring's `linalg::tridiag_eigh` uses implicit QL with Wilkinson shifts and
produces both eigenvalues and eigenvectors. The eigenvectors are needed for
`transport::wavepacket_msd` (Kachkovskiy 2016).

**toadStool action**: When barraCuda exposes `eigh_f64` with eigenvector output,
groundSpring can delegate `tridiag_eigh` — currently CPU-only QL beats dense
Jacobi, so there's no urgency unless batch transport is needed.

### P2: Chao1 Formula Alignment

groundSpring uses classic Chao 1984: `S_obs + f1²/(2·f2)`.
barraCuda uses bias-corrected Chao & Chiu 2016: `S_obs + f1·(f1-1)/(2·(f2+1))`.

Both are scientifically valid but produce different estimates at low coverage.
groundSpring deliberately uses Chao 1984 for provenance against the original paper.
Delegation would change the provenance claim.

**Recommendation**: No action needed. Document the formula difference in both
codebases. If a future experiment specifically requires bias-corrected Chao1,
add a `chao1_bias_corrected` variant.

---

## Part 4: Capability Advertisement (Format C/D)

groundSpring V116 parses 4 capability advertisement formats:

| Format | JSON Shape | Notes |
|--------|-----------|-------|
| A | `"compute.execute"` | Flat string (original) |
| B | `{"name": "compute.execute"}` | Name/capability object |
| C | `{"method": "compute.execute", "description": "..."}` | **V116 new** — method with metadata |
| D | `{"semantic_method": "measurement.bootstrap"}` | **V116 new** — semantic routing |

Also: `"methods"` wrapper key supported alongside `"capabilities"` and `"result"`.

**toadStool action**: When advertising capabilities to biomeOS, prefer Format C
or D for richer metadata. Format D enables semantic routing where the method
name carries domain intent (e.g., `measurement.bootstrap` vs `compute.execute`).

---

## Part 5: What groundSpring Learned That Benefits toadStool

### ValidationSink Trait Pattern

Absorbed from ludoSpring V22 / rhizoCrypt v0.13 / primalSpring. The pattern:

```rust
pub trait ValidationSink {
    fn record_pass(&mut self, label: &str, detail: &str);
    fn record_fail(&mut self, label: &str, detail: &str);
    fn section(&mut self, name: &str);
    fn write_summary(&mut self, text: &str);
}
```

This allows validation harnesses to be silent (benchmarks), stdout (CI),
or buffered (programmatic). If toadStool/barraCuda has validation binaries
that print to stdout, this trait abstracts the output channel.

### Named Constants With Provenance

All inline numeric defaults now have named constants with source citations:

```rust
/// Ecosystem convention (hotSpring, wetSpring, airSpring)
const DEFAULT_SEED: u64 = 42;

/// Kachkovskiy Paper 2 finite-size scaling
const DEFAULT_ANDERSON_N_SITES: usize = 10_000;
```

This prevents magic-number drift and makes provenance auditable.

### Smart Refactoring Strategy

RAWR was extracted from `bootstrap.rs` (669 LOC) into `rawr.rs` (~180 LOC)
while keeping shared infrastructure (`validate_bootstrap_inputs`, `percentile_ci`,
`BootstrapResult`) in `bootstrap.rs` with `pub(crate)` visibility. The public
re-export `bootstrap::rawr_mean` preserves API compatibility.

This pattern — extract by algorithm, share infrastructure via `pub(crate)`,
maintain backward-compatible re-exports — works well for growing modules.

---

## Part 6: Quality Certificate

| Metric | Value |
|--------|-------|
| Rust tests | 960+ (default workspace) |
| Validation checks | 395/395 across 34 binaries |
| Three-tier parity | 29/29 proven (CPU, barracuda-CPU, barracuda-GPU) |
| Clippy (pedantic + nursery) | 0 warnings |
| `cargo doc` | 0 warnings |
| `cargo fmt` | clean |
| Unsafe code | forbidden (`#![forbid(unsafe_code)]`) |
| `#[allow]` annotations | 0 (all `#[expect]` with `reason`) |
| `Result<_, String>` in dispatch | 0 |
| Hardcoded primal names | 0 |
| Production mocks | 0 |
| barraCuda delegations | 102 (61 CPU + 41 GPU) |
| metalForge workloads | 30 (24 GPU + 2 NPU + 2 CPU-only + 2 mixed) |
| metalForge checks | 140 |

---

## Part 7: Cross-Spring Shader Lineage

Every delegation traces back to a specific spring's contribution:

```
hotSpring  →  df64_core, Sturm tridiag, stress_virial, CG kernels
              784 WGSL shaders, f64-canonical with f16/f32/f64/Df64

wetSpring  →  smith_waterman, gillespie, fused_map_reduce, HMM
              toadStool S156+ routes, coralReef compiles to native binary

neuralSpring → chi_squared, KL_divergence, matrix_correlation, ESN

airSpring  →  hargreaves, seasonal_pipe, moving_window, Brent root

groundSpring → anderson_lyapunov, welford, uncertainty propagation
               102 delegations consuming all spring contributions
```

---

## Cross-References

- `specs/BARRACUDA_EVOLUTION.md` — Module → shader → pipeline stage mapping
- `specs/BARRACUDA_REQUIREMENTS.md` — GPU kernel gap analysis
- `metalForge/ABSORPTION_MANIFEST.md` — Detailed absorption inventory
- `wateringHole/CROSS_SPRING_SHADER_EVOLUTION.md` — Shader provenance
- `wateringHole/handoffs/GROUNDSPRING_V116_TYPED_ERROR_EVOLUTION_HANDOFF_MAR18_2026.md` — V116 code changes detail
