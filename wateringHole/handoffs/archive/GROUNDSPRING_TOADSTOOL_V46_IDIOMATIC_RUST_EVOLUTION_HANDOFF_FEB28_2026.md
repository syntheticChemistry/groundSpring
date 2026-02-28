# groundSpring → ToadStool V46: Idiomatic Rust Evolution + Absorption Guidance

**Date**: February 28, 2026
**ToadStool pin**: S68+ (`e96576ee`)
**groundSpring**: V46 (idiomatic Rust evolution + validation gap closure)
**Previous**: V45 (validation gap closure: 292/292), V44 (deep-debt evolution)

---

## Summary

V45–V46 is a **quality deepening** pass:

- **V45**: Closed all Python↔Rust validation gaps (+4 checks → 292/292 PASS)
- **V46**: Idiomatic Rust evolution — domain-driven module split, R²/NSE
  deduplication, iterator modernization, doc/clippy zero warnings

No new experiments. No new delegations. The codebase is structurally mature
and ready for the next phase: BarraCUDA CPU purity proofs, then GPU portability.

---

## Part 1: What Changed (for ToadStool absorption)

### `stats::agreement` module — domain-driven split from `metrics`

```
crates/groundspring/src/stats/agreement.rs (NEW — 310 lines)
  rmse, mae, mbe, nash_sutcliffe, r_squared, index_of_agreement, hit_rate
  coefficient_of_efficiency  — shared helper (private)

crates/groundspring/src/stats/metrics.rs (REDUCED — 170 lines)
  mean, std_dev, sample_std_dev, percentile

crates/groundspring/src/stats/mod.rs (UPDATED)
  pub use agreement::{...};  // re-exports preserved, zero API change
  pub use metrics::{...};
```

**Key insight**: `r_squared_cpu` and `nash_sutcliffe_cpu` were identical
implementations (`1 - SS_res / SS_tot`). They now share a private
`coefficient_of_efficiency` helper. NSE and R² remain separate public APIs
because hydrology and statistics use different names for the same formula.

**Why this matters for ToadStool**: When barracuda exposes `stats::r_squared`
and `stats::nash_sutcliffe`, they can share the same kernel. The groundSpring
split documents this equivalence.

### V45: Validation gap closure (+4 checks)

| Experiment | New Check | Significance |
|------------|-----------|-------------|
| Exp 010 | Low-noise c-di-GMP agrees with deterministic | Stochastic→deterministic convergence |
| Exp 011 | Dual-signal has lower c-di-GMP variance | Noise suppression via dual signaling |
| Exp 016 | Spearman ρ(abundance, occupancy) > 0.2 | Positive correlation despite rank ties |
| Exp 016 | Multinomial deterministic (same seed) | Bitwise PRNG reproducibility |

All Python baseline checks now have Rust counterparts: **292/292 PASS**.

### Iterator modernization

```rust
// Before (almost_mathieu::level_spacing_ratio_cpu)
for i in 0..n - 2 {
    let d1 = eigenvalues[i + 1] - eigenvalues[i];
    ...
}

// After — idiomatic .windows(3).fold()
eigenvalues.windows(3).fold((0.0, 0usize), |(s, c), w| {
    let (d1, d2) = (w[1] - w[0], w[2] - w[1]);
    ...
})
```

### Hardcode evolution

- `NESTGATE_DEFAULT_PORT` constant (was magic `8090` in 3 places)
- NestGate URL already uses `NESTGATE_URL` env var with capability fallback

---

## Part 2: Absorption Requests (unchanged from V44)

### Priority 1: `jackknife_mean_variance` (CPU, embarrassingly parallel)

```rust
// crates/groundspring/src/jackknife.rs:44
// TODO(toadstool): wire when barracuda adds stats::jackknife_mean_variance
```

Returns `Result<JackknifeResult, InputError>` — barracuda should match.

### Priority 2: `grid_fit_2d_f64` (GPU, 2D grid search)

```rust
// crates/groundspring/src/freeze_out.rs:99
// TODO(toadstool): wire when barracuda adds ops::grid::grid_fit_2d_f64
```

### Priority 3: `kimura_fixation` (CPU, scalar)

```rust
// crates/groundspring/src/drift.rs:92
```

### Priority 4: `grid_search_3d_f64` (GPU, 3D grid search)

```rust
// crates/groundspring/src/seismic.rs:125
```

### Priority 5: `band_edges_parallel` (GPU, per-energy parallel)

```rust
// crates/groundspring/src/band_structure.rs:66
```

### Priority 6: `fao56_et0` (CPU, scalar)

```rust
// crates/groundspring/src/fao56.rs:247
```

### Priority 7: `BatchedTridiagEigh` (new — GPU eigensolver)

Not yet in any TODO. `linalg::tridiag_eigh` (V44) serves transport,
band_structure, and almost_mathieu. Batching across realizations would
multiply throughput for Anderson, WDM, and spectral workloads.

---

## Part 3: Delegation Inventory (39 active + 7 pending)

### barracuda modules exercised by groundSpring

| Module | Functions | Count |
|--------|-----------|-------|
| `barracuda::stats` | rmse, mae, mbe, nse, r², ia, hit_rate, mean, percentile, hill, bootstrap_mean, rawr_mean, shannon, pielou_evenness, norm_cdf, norm_ppf, chi2_decomposed | 17 |
| `barracuda::stats::correlation` | std_dev, spearman_correlation, covariance, pearson_correlation | 4 |
| `barracuda::stats::regression` | fit_linear, fit_quadratic, fit_exponential, fit_logarithmic | 4 |
| `barracuda::spectral` | level_spacing_ratio, almost_mathieu_hamiltonian, find_all_eigenvalues, lyapunov_exponent, lyapunov_averaged, detect_bands | 6 |
| `barracuda::numerical` | trapz, OdeSystem, ode_bio::BistableOde, ode_bio::MultiSignalOde | 4 |
| `barracuda::special` | anderson_transport::localization_length | 1 |
| `barracuda::linalg` | solve_f64_cpu | 1 |
| `barracuda::device` | WgpuDevice | 1 |
| `barracuda::ops::bio` | BatchedMultinomialGpu | 1 |
| **Total** | | **39** |

### Pending ToadStool absorption (7)

| # | groundSpring fn | barracuda target | Type |
|---|----------------|-----------------|------|
| P1 | `jackknife::jackknife_mean_variance` | `stats::jackknife_mean_variance` | CPU |
| P2 | `drift::kimura_fixation_prob` | `stats::kimura_fixation` | CPU |
| P3 | `fao56::daily_et0` | `stats::hydrology::fao56_et0` | CPU |
| P4 | `freeze_out::grid_fit_2d` | `ops::grid::grid_fit_2d_f64` | GPU |
| P5 | `seismic::grid_search_inversion` | `ops::grid::grid_search_3d_f64` | GPU |
| P6 | `band_structure::find_band_edges` | `spectral::band_edges_parallel` | GPU |
| P7 | `linalg::tridiag_eigh` (batch) | `linalg::BatchedTridiagEigh` | GPU |

---

## Part 4: Cross-Spring Learnings for ToadStool

### 1. R²/NSE equivalence

Nash-Sutcliffe Efficiency and R² are the same formula (`1 - SS_res/SS_tot`)
when applied to the same (observed, modeled) pairs. groundSpring documents
this in `stats::agreement::coefficient_of_efficiency`. barracuda can share
a single kernel for both.

### 2. Error type pattern

groundSpring's `InputError` (`LengthMismatch`, `InsufficientData`,
`OutOfRange`) is generic enough for any barracuda stats primitive.
Consider absorbing into barracuda's error hierarchy.

### 3. FAO-56 constants stay inline

FAO-56 formula coefficients (0.6108, 17.27, 237.3, etc.) are intentionally
kept inline with equation citations rather than extracted to named constants.
For standards-based formula sets, inline-with-citation is the correct pattern —
it allows direct verification against the reference paper.

### 4. Tridiag eigensolver gap

hotSpring has dense Jacobi (`eigh_f64`) and Sturm bisection
(`find_all_eigenvalues`). Neither is optimal for tridiagonal batched
workloads. A `BatchedTridiagEigh` using divide-and-conquer or MRRR
would serve groundSpring, hotSpring, and neuralSpring.

### 5. PRNG alignment still pending

groundSpring uses `Xorshift64` for reproducibility. barracuda uses
`xoshiro128**`. Phase 2b baseline regeneration is blocked until PRNG
alignment. The alignment path is documented in `specs/BARRACUDA_EVOLUTION.md`.

---

## Part 5: Code Quality Certificate

| Gate | Status |
|------|--------|
| `cargo fmt --check` | clean |
| `cargo clippy --workspace --all-targets` | 0 warnings |
| `cargo doc --workspace --no-deps` | 0 warnings |
| `cargo test --workspace` | 296 unit + 292/292 validation, all PASS |
| `unsafe` blocks | 0 (workspace lint: `forbid`) |
| `.unwrap()`/`.expect()` in library | 0 |
| Mocks in production | 0 |
| External dependencies | 6 (serde_json, wgpu, pollster, bytemuck, proptest, tempfile) |

### Architecture evolution V44 → V46

```
V44: Deep-debt evolution (linalg, typed errors, capability discovery)
  │
V45: Validation gap closure (+4 checks → 292/292 PASS)
  │  Exp 010: low-noise stochastic→deterministic agreement
  │  Exp 011: dual-signal variance suppression
  │  Exp 016: Spearman occupancy correlation + multinomial determinism
  │  validate_bistable.rs refactored: SimCtx struct, validate_stochastic helper
  │
V46: Idiomatic Rust evolution
  ├── stats/agreement.rs: domain split from metrics.rs
  ├── coefficient_of_efficiency: R²/NSE deduplication
  ├── almost_mathieu: .windows(3).fold() iterator idiom
  ├── NestGate: NESTGATE_DEFAULT_PORT constant
  └── All doc/clippy warnings resolved
```

---

## Part 6: Evolution Roadmap

```
groundSpring V46 (current — all green)
  │
  ├─→ BarraCUDA CPU purity proofs
  │     Validate pure Rust math is faster than interpreted language
  │     (already 11.5× average, 53.5× peak for seismic)
  │
  ├─→ BarraCUDA GPU portability
  │     Show the math is truly portable via barracuda-gpu
  │     (already 2.2× overall, 47.7× for quasiperiodic Sturm)
  │
  ├─→ ToadStool unidirectional streaming
  │     Massively reduce dispatch and round trips
  │
  └─→ BarraCUDA PURE GPU final workload validation
        metalForge cross-substrate (GPU → NPU → CPU)
```
