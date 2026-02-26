# groundSpring → ToadStool Handoff V9: Complete Rewiring, Benchmarks & Cross-Spring Lineage

**Date**: February 25, 2026
**From**: groundSpring (validation spring — measurement noise characterization)
**To**: ToadStool / BarraCUDA team
**Supersedes**: V8 (sovereignty, barracuda error handling, deep debt)
**ToadStool baseline**: Sessions 50–62 + DF64 expansion (Feb 23–24, 2026) — revalidated

---

## Summary

V9 completes the groundSpring rewiring cycle:

1. **Complete API audit**: Every CPU-accessible function in barracuda was reviewed.
   Our **11 delegations are confirmed as the complete set** — no new CPU-only
   primitives exist for the remaining 6 pending metrics.

2. **Three-mode benchmarking**: All 8 validation binaries benchmarked in release
   mode across local / barracuda / barracuda-gpu. **Zero measurable overhead** for
   barracuda delegation in compute-heavy binaries.

3. **Cross-spring shader lineage**: Documented the full evolution story — how
   hotSpring precision shaders, wetSpring bio shaders, and neuralSpring ML/spectral
   modules converged in barracuda to benefit all Springs.

### Quality Gates

| Gate | Status |
|------|--------|
| `cargo fmt --check` | Clean |
| `cargo clippy --workspace -- -D warnings` (all 3 modes) | **0 warnings × 3** |
| `cargo test --workspace` (all 3 modes) | **163/163 PASS × 3** |
| 8 validation binaries (all 3 modes) | **119/119 PASS × 3** |
| `python3 -m pytest tests/ -v` | **34/34 PASS** |
| Line coverage | **99.11%** |
| Barracuda delegation overhead | **<1% for compute-heavy binaries** |

### What Changed Since V8

| Category | V8 | V9 |
|----------|----|----|
| API coverage | 11 delegations identified | 11 delegations **confirmed complete** — full barracuda API audited |
| Benchmarking | V7-era rough timings | **Release-mode best-of-3 across all 3 modes** |
| Clippy barracuda paths | 3 `needless_return` warnings | **Fixed** — 0 warnings in all modes |
| Cross-spring documentation | None | **Full lineage map** of barracuda evolution |
| Handoff | V8 | **V9** (V8 archived) |

---

## Part 1: Complete barracuda API Audit

### CPU-Only Functions (no feature gate) — What Exists

| Module | Functions |
|--------|-----------|
| `stats::bootstrap` | `bootstrap_ci`, `bootstrap_mean`, `bootstrap_median`, `bootstrap_std` |
| `stats::correlation` | `pearson_correlation`, `covariance`, `variance` (sample), `std_dev` (sample), `spearman_correlation`, `correlation_matrix`, `covariance_matrix` |
| `stats::normal` | `norm_cdf`, `norm_pdf`, `norm_ppf`, `norm_cdf_general`, `norm_pdf_general`, `norm_ppf_general` |
| `stats::chi2` | `chi2_decomposed`, `chi2_decomposed_weighted` |
| `stats::spectral_density` | `empirical_spectral_density`, `marchenko_pastur_bounds` |
| `special` | `erf`, `erfc`, `gamma`, `ln_gamma`, `digamma`, `beta`, `bessel_*`, `hermite`, `legendre`, `laguerre`, `chi_squared_*`, `localization_length` |
| `numerical` | `gradient_1d`, `numerical_hessian`, `trapz`, `rk45_solve`, `BatchedOdeRK4::integrate_cpu` |
| `linalg` | `ridge_regression`, `nmf` |

### What groundSpring Uses (11 Active Delegations)

| # | groundSpring fn | barracuda fn | Feature | Pattern |
|---|----------------|--------------|---------|---------|
| 1 | `pearson_r` | `stats::pearson_correlation` | `barracuda` | `if let Ok` + NaN guard |
| 2 | `spearman_r` | `stats::correlation::spearman_correlation` | `barracuda` | `if let Ok` + NaN guard |
| 3 | `sample_std_dev` | `stats::correlation::std_dev` | `barracuda` | `if let Ok` |
| 4 | `covariance` | `stats::correlation::covariance` | `barracuda` | `if let Ok` |
| 5 | `norm_cdf` | `stats::norm_cdf` | `barracuda` | Direct (infallible) |
| 6 | `norm_ppf` | `stats::norm_ppf` | `barracuda` | Direct (infallible) |
| 7 | `chi2_statistic` | `stats::chi2_decomposed` | `barracuda` | `map_or` with struct mapping |
| 8 | `bootstrap_mean` | `stats::bootstrap_mean` | `barracuda` | `if let Ok` + struct mapping |
| 9 | `lyapunov_exponent` | `spectral::lyapunov_exponent` | `barracuda-gpu` | Direct (infallible) |
| 10 | `lyapunov_averaged` | `spectral::lyapunov_averaged` | `barracuda-gpu` | Direct (infallible) |
| 11 | `analytical_localization_length` | `special::anderson_transport::localization_length` | `barracuda` | Direct (infallible) |

### Why 11 Is Complete

The 6 remaining metrics (RMSE, MBE, R², Index of Agreement, hit rate,
Shannon diversity) have GPU-only ops in barracuda (`NormReduceF64`,
`SumReduceF64`, `FusedMapReduceF64`) — all require `Arc<WgpuDevice>` with
no public CPU fallback. barracuda's `dispatch` module (S60-61) provides CPU
paths for `mean_dispatch` and `variance_dispatch` via `device: Option`, but
there is no dispatch equivalent for the error metrics.

**Recommendation to barracuda team**: Consider adding `stats::rmse`,
`stats::mbe`, `stats::r_squared` as pure CPU functions in `stats/metrics.rs`.
These are 5-line functions that every numerical validation Spring needs.

### Available but Unused barracuda Functions

These are available and could extend groundSpring's API surface in future:

| barracuda fn | Potential groundSpring use |
|-------------|--------------------------|
| `bootstrap_median` / `bootstrap_std` | Extended bootstrap analysis |
| `chi2_decomposed_weighted` | Uncertainty-weighted goodness-of-fit |
| `norm_pdf`, `norm_cdf_general`, `norm_ppf_general` | Parameterized distributions |
| `empirical_spectral_density` | Anderson localization spectral diagnostics |
| `marchenko_pastur_bounds` | Random matrix theory bounds |
| `ridge_regression` | ESN readout / regularized fitting |
| `trapz`, `gradient_1d`, `numerical_hessian` | Numerical analysis utilities |

---

## Part 2: Three-Mode Benchmarks

Release-mode, best-of-3 trials, February 25 2026:

| Binary | Local (ms) | BarraCUDA (ms) | BarraCUDA-GPU (ms) | Overhead |
|--------|-----------|---------------|-------------------|----------|
| validate-anderson | 671 | 670 | 640 | **−5%** (faster!) |
| validate-decompose | 5 | 4 | 5 | noise |
| validate-fao56 | 12 | 12 | 13 | noise |
| validate-rarefaction | 11 | 12 | 12 | noise |
| validate-rawr | 555 | 560 | 556 | **<1%** |
| validate-seismic | 56 | 59 | 58 | noise |
| validate-signal-specificity | 795 | 787 | 787 | **−1%** (faster!) |
| validate-weather | 3 | 3 | 5 | noise |
| **TOTAL** | **2108** | **2107** | **2076** | **~0%** |

### Key Findings

1. **Zero overhead**: For compute-heavy binaries (>500ms), barracuda delegation
   adds no measurable cost. The CPU code paths in barracuda are algorithmically
   identical to our local implementations.

2. **Anderson slightly faster with barracuda-gpu**: The spectral module's
   `lyapunov_averaged` may benefit from barracuda's optimized transfer-matrix
   implementation (different seed stride pattern, possibly better cache behavior).

3. **Signal specificity unchanged**: The Gillespie SSA is local in all three
   modes (no barracuda CPU fallback for `GillespieGpu`), so times are identical.

4. **Previous V7 overhead was link/init cost**: The +20-40% overhead we saw in
   V7 benchmarks was from barracuda's library initialization in short-lived
   binaries. In the current S62 barracuda (with `cpu-math` modularization and
   dead-code elimination), this overhead has essentially vanished.

### Rust vs Python Performance (unchanged from V8)

| Experiment | Python (s) | Rust (s) | Speedup |
|---|---|---|---|
| Signal Specificity (Gillespie SSA) | 26.2 | 0.85 | **30.9×** |
| RAWR Resampling (bootstrap) | 4.4 | 0.60 | **7.3×** |
| Anderson Localization (transfer matrix) | 21.4 | 0.72 | **29.8×** |
| **Total** | **52.0** | **2.17** | **24.0×** |

---

## Part 3: Cross-Spring Shader Evolution

BarraCUDA's 650+ WGSL shaders and CPU primitives evolved through absorption
from four Springs. This lineage matters for groundSpring because **the modules
we delegate to were refined by multiple domain-specific validation efforts**.

### hotSpring → Precision Foundation

hotSpring (nuclear/condensed-matter physics) contributed the **f64 precision
infrastructure** that makes groundSpring's statistical validation possible:

| Contribution | Session | groundSpring Benefit |
|-------------|---------|---------------------|
| `df64_core.wgsl` — double-float emulation | S58 | Future GPU bootstrap could use DF64 for 1e-15 precision |
| `Fp64Strategy` enum — workgroup dispatch | S58 | GPU promotion will inherit correct f64 dispatch strategy |
| `spectral/anderson.rs` — Anderson localization | S26 | **Direct**: `lyapunov_exponent`, `lyapunov_averaged` delegations |
| Lattice QCD CG shaders (6 kernels) | S46-48 | Pattern: iterative solver → GPU with convergence check |
| `sum_reduce_f64.wgsl` | S46 | Foundation for future RMSE/MBE GPU delegation |
| `localization_length` (transport) | S52 | **Direct**: `analytical_localization_length` delegation |
| 195 nuclear physics validation checks | — | Validated the f64 precision path we depend on |

**Key insight**: hotSpring's discovery that FP64 core-streaming needs careful
workgroup sizing (`split_workgroups` for 2D dispatch) directly affects how
groundSpring's future GPU error metrics will be dispatched.

### wetSpring → Bio-Statistical Primitives

wetSpring (metagenomics/microbial ecology) contributed the **statistical and
biological primitives** that groundSpring uses for ecological validation:

| Contribution | Session | groundSpring Benefit |
|-------------|---------|---------------------|
| `FusedMapReduceF64` — Shannon/Simpson | S15 | Future GPU `shannon_diversity` delegation target |
| `bray_curtis_f64` — community dissimilarity | S15 | Rarefaction context (related diversity metric) |
| `GillespieGpu` — stochastic simulation | S27 | Future CPU fallback for `birth_death_ssa` |
| 5 ODE biosystems (`CapacitorOde`, etc.) | S58 | Pattern: CPU reference → GPU promotion |
| NMF (metagenomics) | S58 | Available for future spectral decomposition |
| `ridge_regression` (ESN readout) | S15 | Available for regularized fitting |
| `log_f64()` precision fix | S15 | Shannon entropy accuracy (1e-3 → 1e-15) |
| `anderson_transport.rs` — conductance | S52 | **Direct**: `localization_length` function |
| 728 Rust tests + 95 experiments | — | Validated the statistical paths we depend on |

**Key insight**: wetSpring discovered a ~1e-3 precision issue in `log_f64()`
coefficients that affected Shannon entropy. This fix propagated to barracuda's
`FusedMapReduceF64`, which groundSpring will eventually use for GPU diversity.

### neuralSpring → ML/Spectral Infrastructure

neuralSpring (neural networks/agent systems) contributed the **spectral
diagnostics and dispatch infrastructure** that bridges groundSpring's Anderson
localization to barracuda:

| Contribution | Session | groundSpring Benefit |
|-------------|---------|---------------------|
| `empirical_spectral_density` | S54 | Future Anderson spectral diagnostics |
| `marchenko_pastur_bounds` | S54 | Random matrix theory bounds for disorder |
| `boltzmann_sampling` (Metropolis MCMC) | S56 | Future MC uncertainty propagation |
| `graph_laplacian`, `effective_rank` | S54 | Spectral graph analysis tools |
| `numerical_hessian` | S54 | Loss landscape curvature analysis |
| `belief_propagation_chain` | S56 | Message-passing algorithms |
| `prng_xoshiro` — GPU PRNG | S43 | PRNG alignment target for Phase 2b |
| `TensorSession` — matmul, relu, softmax | S20 | ML pipeline infrastructure |
| `dispatch/domain_ops.rs` — CPU/GPU dispatch | S52 | Pattern: `device: Option` CPU fallback |
| 1,560+ validation checks | — | Validated dispatch and spectral infrastructure |

**Key insight**: neuralSpring's `domain_ops.rs` dispatch pattern (`device:
Option<&Arc<WgpuDevice>>`) is the blueprint for how groundSpring's 6 pending
GPU metrics should eventually be wired — pass `None` for CPU, `Some(device)`
for GPU.

### Multi-Spring Convergence

Several barracuda modules benefited from **multiple Springs discovering the
same need independently**:

| Module | Springs | Evolution Story |
|--------|---------|----------------|
| **f64 precision** | hotSpring (df64_core) + wetSpring (log_f64 fix) + neuralSpring (pow polyfill fix) | Three Springs independently found f64 precision issues; all fixes merged into barracuda |
| **bio ops** | wetSpring (Gillespie, phylogenetics) + neuralSpring (Wright-Fisher, swarm) | Two Springs contributed complementary biological simulation primitives |
| **spectral analysis** | hotSpring (Anderson, Hofstadter) + neuralSpring (spectral_density, Marchenko-Pastur) | Physics + ML perspectives on spectral methods converged |
| **PRNG** | neuralSpring (xoshiro128** GPU) + wetSpring (stochastic bio) + groundSpring (alignment request) | GPU PRNG shared across three Springs' stochastic workloads |
| **validation patterns** | All four Springs | `ValidationHarness`, tolerance documentation, struct extraction |

### Why This Matters for groundSpring

The cross-spring evolution means groundSpring's delegations inherit
battle-tested code:

- Our `bootstrap_mean` delegates to code validated by **728 wetSpring tests**
- Our `lyapunov_exponent` delegates to code validated by **195 hotSpring checks**
- Our `pearson_correlation` delegates to code refined by **neuralSpring's spectral
  diagnostics** (ensuring numerical stability at scale)
- The `if let Ok` + CPU fallback pattern is now a **wateringHole standard**
  adopted by all Springs

---

## Part 4: Remaining Absorption Requests

### Priority 1: `batched_multinomial.wgsl` (112 lines)

Production WGSL in `metalForge/shaders/`. Runs `n_reps` multinomial draws
with xoshiro128** PRNG and binary-search assignment. No barracuda equivalent.

### Priority 2: RAWR kernel (`rawr_weighted_mean_f64`)

Embarrassingly parallel weighted resampling. Not in barracuda. CPU reference
in `groundspring::bootstrap::rawr_mean()`.

### Priority 3: `mc_et0_propagate.wgsl` (149 lines)

MC uncertainty wrapper around FAO-56. The equation chain is superseded by
`Op::Fao56Et0`, but the Box-Muller perturbation + dispatch wrapper is new.

### Priority 4: Gillespie CPU fallback

`GillespieGpu` is GPU-only. groundSpring's `birth_death_ssa` is a CPU
implementation that could serve as the CPU path for `GillespieGpu`.

### Priority 5: CPU error metrics

`stats::rmse`, `stats::mbe`, `stats::r_squared` as pure CPU functions
(5 lines each) would let groundSpring wire the remaining 6 delegations
without needing `WgpuDevice`.

---

## Part 5: PRNG Alignment Status

| Item | Status |
|------|--------|
| barracuda CPU (LHS) | `Xoshiro256**` (4×u64) — `pub(crate)` in `sample/lhs.rs` |
| barracuda GPU | `xoshiro128**` (4×u32) in WGSL shaders |
| groundSpring CPU | `Xorshift64` (Marsaglia) — different algorithm entirely |
| Public CPU xoshiro128** | **Not available** — still needed for Phase 2b |
| Rebaseline scope | 5 stochastic experiments (RAWR, Anderson, Gillespie, rarefaction, FAO56 MC) |

---

## Part 6: Delegation Inventory (Complete)

### Active (11)

| # | Function | Target | Feature | Error Handling |
|---|----------|--------|---------|---------------|
| 1 | `pearson_r` | `stats::pearson_correlation` | `barracuda` | `if let Ok` + NaN |
| 2 | `spearman_r` | `stats::correlation::spearman_correlation` | `barracuda` | `if let Ok` + NaN |
| 3 | `sample_std_dev` | `stats::correlation::std_dev` | `barracuda` | `if let Ok` |
| 4 | `covariance` | `stats::correlation::covariance` | `barracuda` | `if let Ok` |
| 5 | `norm_cdf` | `stats::norm_cdf` | `barracuda` | Direct |
| 6 | `norm_ppf` | `stats::norm_ppf` | `barracuda` | Direct |
| 7 | `chi2_statistic` | `stats::chi2_decomposed` | `barracuda` | `map_or` |
| 8 | `bootstrap_mean` | `stats::bootstrap_mean` | `barracuda` | `if let Ok` |
| 9 | `lyapunov_exponent` | `spectral::lyapunov_exponent` | `barracuda-gpu` | Direct |
| 10 | `lyapunov_averaged` | `spectral::lyapunov_averaged` | `barracuda-gpu` | Direct |
| 11 | `analytical_localization_length` | `special::localization_length` | `barracuda` | Direct |

### Pending GPU Adapter (6) — Requires `WgpuDevice`

| Function | GPU Op | Blocker |
|----------|--------|---------|
| `rmse` | `NormReduceF64::l2` | No CPU dispatch |
| `mbe` | `SumReduceF64::mean` | No CPU dispatch |
| `r_squared` | `VarianceReduceF64` + reduce | No CPU dispatch |
| `index_of_agreement` | `FusedMapReduceF64` | No CPU dispatch |
| `hit_rate` | `FusedMapReduceF64` | No CPU dispatch |
| `shannon_diversity` | `FusedMapReduceF64::shannon_entropy` | No CPU dispatch |

### Not in barracuda (stays local)

| Function | Reason |
|----------|--------|
| `rawr_mean` | No RAWR kernel — Priority 2 absorption |
| `birth_death_ssa` | `GillespieGpu` GPU-only — Priority 4 |
| `multinomial_sample` | No batched multinomial — Priority 1 absorption |
| `decompose_error` | 2 scalar ops |
| `haversine_km` | 1 scalar trig |
| `travel_time_1d` | 1 sqrt + division |

---

## Handoff Checklist

- [x] All 11 delegations confirmed current against S62 barracuda API
- [x] Full barracuda CPU API audited — no missed wiring opportunities
- [x] 3 `needless_return` clippy warnings fixed in barracuda feature paths
- [x] Three-mode benchmarking: zero overhead for compute-heavy binaries
- [x] Three-mode validation: 163/163 tests × 3, 119/119 checks × 3, 0 warnings × 3
- [x] 34 Python tests passing
- [x] Cross-spring shader lineage documented
- [x] Absorption priorities updated (5 items)
- [x] PRNG alignment status documented
- [x] V8 archived

---

*groundSpring rewiring complete. 11 delegations verified, zero overhead,
cross-spring lineage documented. Next: barracuda team adds CPU error metrics
(Priority 5) to unlock the remaining 6 delegations without GPU, or
groundSpring builds WgpuDevice lifecycle for direct GPU dispatch.*
