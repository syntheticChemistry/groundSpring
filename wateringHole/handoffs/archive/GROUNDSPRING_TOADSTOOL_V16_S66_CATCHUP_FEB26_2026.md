# groundSpring → ToadStool/BarraCUDA V16 — S66 Catch-up + Rewiring

**Date**: February 26, 2026
**From**: groundSpring (noise validation + environmental modeling)
**To**: ToadStool/BarraCUDA team
**Previous**: V15 (absorption request)
**ToadStool HEAD**: `045103a7` (S66 Wave 5)
**groundSpring HEAD**: current (V16 rewiring)

---

## Purpose

This handoff acknowledges ToadStool's S66 evolution (from S65 in V15), wires
the newly absorbed `rawr_mean` into groundSpring as delegation #26, and
provides an updated inventory of what V13–V15 items have been absorbed vs
what still awaits ToadStool consumption.

---

## Part 1: What ToadStool Absorbed (S64–S66)

### 1.1 Items from groundSpring V15 Request — Status

| V15 Request | ToadStool Status | groundSpring Action |
|-------------|------------------|---------------------|
| `rawr_mean` (Dirichlet bootstrap) | **ABSORBED** in S66 as `stats::rawr_mean` | **Wired** as delegation #26 (CPU) |
| `batched_multinomial.wgsl` shader | Absorbed S64 as `BatchedMultinomialGpu` | Rewiring deferred (seed API mismatch) |
| `mc_et0_propagate.wgsl` shader | Not yet absorbed | Still in metalForge |
| `pielou_evenness` S≤1 → 1.0 | **NOT FIXED** — still returns 0.0 | Adapter remains in groundSpring |
| Spectral CPU promotion (5 functions) | **NOT DONE** — still behind `gpu` feature | groundSpring uses `barracuda-gpu` feature |
| Seed-based `batched_multinomial` API | **NOT DONE** — closure-based API persists | Rewiring deferred |

### 1.2 New S66 Capabilities Available to groundSpring

| New S66 barracuda API | Module | groundSpring Use Case |
|-----------------------|--------|----------------------|
| `stats::regression` (fit_linear, etc.) | `stats::regression` | Could replace manual `transport_exponent` log-log fit |
| `stats::hydrology` (hargreaves_et0) | `stats::hydrology` | FAO-56 CPU reference (complementing ET₀ experiments) |
| `stats::moving_window_f64` | `stats::moving_window_f64` | Sliding-window statistics for time series |
| `stats::mae` | `stats::metrics` | Mean Absolute Error (new metric) |
| `shannon_from_frequencies` | `stats::diversity` | Direct Shannon from precomputed frequency vectors |
| `WrightFisherGpu` | `ops::bio` | Future Exp 014 GPU delegation (batched drift+selection) |
| `eigh_f64` / `BatchedEighGpu` | `linalg` | Future Exp 012 GPU delegation (dense eigenvectors) |
| `hill()` / `monod()` | `stats::metrics` | Dose-response kinetics (now public Rust API) |

### 1.3 V13–V15 Handoff Consumption Status

| Handoff | Key Items | ToadStool Consumed? |
|---------|-----------|---------------------|
| V7 | Deep audit, proptest patterns | **Yes** — groundSpring provenance in `stats/metrics.rs`, `stats/bootstrap.rs` |
| V13 | 24 delegations, Sturm tridiag 50× | **Not consumed** — ToadStool doesn't reference V13 |
| V14 | 25 delegations, evenness, S65 revalidation | **Not consumed** — ToadStool doesn't reference V14 |
| V15 | Absorption request: 2 shaders, 3 semantic fixes | **Partially** — `rawr_mean` absorbed, rest pending |

---

## Part 2: Current Delegation Inventory (26 active)

### CPU delegations (`barracuda` feature) — 21

| # | groundSpring | barracuda | Notes |
|---|-------------|-----------|-------|
| 1 | `pearson_r` | `stats::pearson_correlation` | NaN-safe |
| 2 | `spearman_r` | `stats::correlation::spearman_correlation` | NaN-safe |
| 3 | `sample_std_dev` | `stats::correlation::std_dev` | Bessel-corrected |
| 4 | `covariance` | `stats::correlation::covariance` | Sample covariance |
| 5 | `norm_cdf` | `stats::norm_cdf` | Standard normal Φ(x) |
| 6 | `norm_ppf` | `stats::norm_ppf` | Inverse Φ⁻¹(p) |
| 7 | `chi2_statistic` | `stats::chi2_decomposed` | Goodness-of-fit |
| 8 | `bootstrap_mean` | `stats::bootstrap_mean` | Result struct mapping |
| 9 | `rawr_mean` | `stats::rawr_mean` | **NEW (S66)** — Dirichlet-weighted mean |
| 10 | `analytical_localization_length` | `special::localization_length` | Perturbative ξ(W,E) |
| 11 | `bistable_derivative` | `BistableOde::cpu_derivative` | OdeSystem trait |
| 12 | `multisignal_derivative` | `MultiSignalOde::cpu_derivative` | OdeSystem trait |
| 13 | `rmse` | `stats::rmse` | S64 absorption |
| 14 | `mbe` | `stats::mbe` | S64 absorption |
| 15 | `r_squared` | `stats::r_squared` | S64 absorption |
| 16 | `index_of_agreement` | `stats::index_of_agreement` | S64 absorption |
| 17 | `hit_rate` | `stats::hit_rate` | S64 absorption |
| 18 | `shannon_diversity` | `stats::shannon` | u64→f64 conversion |
| 19 | `mean` | `stats::mean` | Direct |
| 20 | `percentile` | `stats::percentile` | Direct |
| 21 | `evenness` | `stats::pielou_evenness` | u64→f64 + S≤1 adapter |

### GPU delegations (`barracuda-gpu` feature) — 5

| # | groundSpring | barracuda | Notes |
|---|-------------|-----------|-------|
| 22 | `lyapunov_exponent` | `spectral::lyapunov_exponent` | Transfer matrix |
| 23 | `lyapunov_averaged` | `spectral::lyapunov_averaged` | Seed convention differs |
| 24 | `almost_mathieu_hamiltonian` | `spectral::almost_mathieu_hamiltonian` | λ/2 coupling convention |
| 25 | `level_spacing_ratio` | `spectral::level_spacing_ratio` | Sort adapter |
| 26 | `almost_mathieu_eigenvalues` | `spectral::find_all_eigenvalues` | Sturm tridiag (49.5×) |

---

## Part 3: Remaining V15 Requests (Still Open)

These items from V15 have NOT been addressed by ToadStool and remain active requests:

### 3.1 `pielou_evenness` S≤1 → 1.0

Barracuda returns 0.0; ecology convention (vegan R, scipy, scikit-bio) returns
1.0 for single-species communities. groundSpring has a pre-delegation adapter.
Request remains: update `barracuda::stats::pielou_evenness` to return 1.0 for S≤1.

### 3.2 Spectral CPU promotion

Five barracuda spectral functions are pure CPU but gated behind `#[cfg(feature = "gpu")]`:
`lyapunov_exponent`, `lyapunov_averaged`, `level_spacing_ratio`,
`find_all_eigenvalues`, `almost_mathieu_hamiltonian`.

Request remains: promote to CPU-only module so Springs can use `barracuda`
instead of `barracuda-gpu` for these 5 delegations, reducing build times.

### 3.3 Seed-based `batched_multinomial` API

Current closure-based API makes delegation awkward. Request remains: add
`(abundances: &[u64], depth: u64, seed: u64) -> Vec<u64>` alongside existing API.

### 3.4 `mc_et0_propagate.wgsl` absorption

149-line MC uncertainty propagation shader still in metalForge. Pairs with
already-absorbed `Op::Fao56Et0` for end-to-end GPU uncertainty quantification.

---

## Part 4: Convention Differences (Active Adapters)

| Adapter | groundSpring | barracuda | Resolution |
|---------|-------------|-----------|------------|
| `pielou_evenness` S≤1 | Returns 1.0 | Returns 0.0 | Pre-delegation adapter (awaiting barracuda fix) |
| `almost_mathieu_hamiltonian` 2λ factor | V = λ cos(2παn+θ) | V = 2λ cos(2παn+θ) | Pass coupling/2 to barracuda |
| `bootstrap ≠ RAWR` comparison | Estimates may differ with Xorshift64 | Both converge to sample mean for small symmetric data | Compare CI widths OR estimates |

---

## Part 5: Validation State

| Metric | Value |
|--------|-------|
| Experiments | 14 |
| Validation checks | 177/177 PASS × 3 modes |
| Rust tests | 205/205 PASS × 3 modes |
| Clippy warnings | 0 × 3 modes |
| Python tests | 37/37 PASS |
| Delegations | 26 (21 CPU + 5 GPU) |
| Mathematical parity | 14/14 PROVEN |

---

## Part 6: Future GPU Delegation Candidates (Exp 012–014)

| Experiment | Function | barracuda Target | Notes |
|------------|----------|-----------------|-------|
| 012 Spin Chain Transport | `tridiag_eigh` (eigenvectors) | `linalg::eigh_f64` | Dense → tridiagonal conversion needed |
| 012 Spin Chain Transport | `transport_exponent` (log-log fit) | `stats::regression::fit_linear` | Direct delegation possible |
| 014 Drift vs Selection | `wright_fisher_fixation` (batched sims) | `ops::bio::WrightFisherGpu` | GPU does 1 generation; CPU loops to fixation |
| 014 Drift vs Selection | `kimura_fixation_prob` (analytical) | None — no barracuda Kimura | Pure math, low priority |

---

*groundSpring → ToadStool V16. 1 new delegation (#26 rawr_mean), S66 catch-up,
V13–V15 consumption status documented. 26 delegations validated. 4 V15 requests
remain open (pielou S≤1, spectral CPU promotion, batched_multinomial seed API,
mc_et0 shader).*
