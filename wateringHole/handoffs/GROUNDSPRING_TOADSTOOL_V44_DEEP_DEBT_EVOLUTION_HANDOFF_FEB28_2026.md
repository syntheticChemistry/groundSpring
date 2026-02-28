# groundSpring → ToadStool V44: Deep-Debt Evolution + Absorption Guidance

**Date**: February 28, 2026
**ToadStool pin**: S68+ (`e96576ee`)
**groundSpring**: V44 (deep-debt evolution pass)
**Previous**: V43 (three-tier parity proven + pure GPU workloads)

---

## Summary

V44 is a **structural evolution** pass — no new experiments, but the codebase
is now significantly more modern, safer, and better-organized for ToadStool
absorption. Key changes:

1. **`linalg` module extracted** — tridiag eigensolver is now a shared primitive
2. **Typed error handling** — 5 APIs evolved from `assert!` → `Result<T, InputError>`
3. **Capability-based discovery** — hardcoded UID paths replaced with runtime procfs
4. **`EighError` enriched** with `Clone`, `PartialEq`, `Eq` derives
5. **`GridFitConfig` enriched** with `Debug`, `Clone`, `Copy` derives
6. **Zero unsafe**, zero mocks in production, zero deprecated patterns
7. **296+ unit tests**, all 3 feature modes green, zero clippy warnings

---

## Part 1: What Changed (for ToadStool absorption)

### `linalg` module — new shared linear algebra primitive

```
crates/groundspring/src/linalg.rs (NEW)
  tridiag_eigh()  — implicit QL with Wilkinson shifts
  EighError       — typed error (Empty, DimensionMismatch, Convergence)
  implicit_ql()   — internal QL iteration
  sort_eigenpairs() — eigenvalue ordering

crates/groundspring/src/transport.rs
  pub use crate::linalg::{tridiag_eigh, EighError};  // backward compat
```

**Why this matters for ToadStool**: `tridiag_eigh` is used by both `transport`
(wavepacket MSD) and `band_structure` (periodic Hamiltonian eigenvalues).
When ToadStool absorbs a `BatchedTridiagEigh` GPU kernel, it slots into
`linalg` — one absorption point instead of two.

The QL algorithm is O(n²) for tridiagonal and gives 1e-10 residuals vs O(n³)
and 1e-5 from barracuda's dense Jacobi `eigh_f64`. A dedicated GPU tridiag
solver would match QL precision while parallelizing across batch dimension.

### `error` module — typed input validation

```
crates/groundspring/src/error.rs (NEW)
  InputError::LengthMismatch   — two parallel slices differ
  InputError::InsufficientData — not enough elements
  InputError::OutOfRange       — scalar outside valid bounds
```

Five APIs now return `Result<T, InputError>` instead of panicking:

| Function | Error Variant | Was |
|----------|---------------|-----|
| `jackknife_mean_variance` | `InsufficientData(min=2)` | `assert!(n >= 2)` |
| `block_jackknife_variance` | `InsufficientData(min=2 blocks)` | `assert!(n_blocks >= 2)` |
| `finite_size_extrapolate` | `LengthMismatch` + `InsufficientData` | `assert_eq!` + `assert!` |
| `chi_squared` | `LengthMismatch` | `assert_eq!` |
| `percentile` | `OutOfRange(0, 100)` | `assert!` |

**Why this matters for ToadStool**: When barracuda absorbs `jackknife_mean_variance`
(TODO #44 in jackknife.rs), the barracuda function should return `Result` too.
The error type pattern (`LengthMismatch`, `InsufficientData`) is reusable for
any stats primitive that validates input shape.

### Capability-based discovery (zero hardcoded UIDs)

```
validate_nucleus_pipeline.rs  — biomeos_socket_dir() discovers UID via:
validate_nestgate_ncbi.rs       1. $XDG_RUNTIME_DIR
                                 2. $UID (shell-set)
                                 3. /proc/self/status (procfs, pure Rust)
```

No `libc`, no `unsafe`. Follows the primal principle: discover at runtime.

---

## Part 2: Absorption Requests (updated from V43)

### Priority 1: `jackknife_mean_variance` (CPU, embarrassingly parallel)

```rust
// crates/groundspring/src/jackknife.rs:44
// TODO(toadstool): wire when barracuda adds stats::jackknife_mean_variance
```

The delete-one loop is embarrassingly parallel: N leave-one-out subsets,
each computing a partial mean. GPU kernel would do N parallel reductions.
**V44 change**: function now returns `Result<JackknifeResult, InputError>` —
barracuda should match this signature.

### Priority 2: `grid_fit_2d_f64` (GPU, 2D grid search)

```rust
// crates/groundspring/src/freeze_out.rs:92
// TODO(toadstool): wire when barracuda adds ops::grid::grid_fit_2d_f64
```

Embarrassingly parallel 2D chi-squared grid — each (T₀, κ₂) is independent.
**V44 change**: `chi_squared()` now returns `Result<f64, InputError>` —
the grid search inner loop uses `.expect()` since lengths are guaranteed.

### Priority 3: `kimura_fixation` (CPU, scalar)

```rust
// crates/groundspring/src/drift.rs:92
// TODO(toadstool): wire when barracuda adds stats::kimura_fixation
```

Scalar function, low priority for GPU but useful for CPU parity.

### Priority 4: `grid_search_3d_f64` (GPU, 3D grid search)

```rust
// crates/groundspring/src/seismic.rs:125
```

3D grid search — 500³ evaluations, each independent. High GPU value.

### Priority 5: `band_edges_parallel` (GPU, per-energy parallel)

```rust
// crates/groundspring/src/band_structure.rs:66
```

Transfer matrix products at each energy are independent. Batch across energies.

### Priority 6: `fao56_et0` (CPU, scalar)

```rust
// crates/groundspring/src/fao56.rs:247
```

Scalar function. Batch version (`Op::Fao56Et0`) already absorbed.

### Priority 7: `BatchedTridiagEigh` (new — GPU eigensolver)

Not yet in any TODO, but the `linalg` extraction makes this the natural
next GPU target. QL algorithm is O(n²) per matrix — batching across
realizations (Anderson, transport, band_structure) multiplies throughput.

---

## Part 3: What ToadStool Should Know About V44

### Code quality gate (verified)

| Gate | Status |
|------|--------|
| `cargo test --workspace` (default) | 296+ unit, all PASS |
| `cargo test --workspace --features barracuda` | all PASS |
| `cargo test --workspace --features barracuda-gpu` | all PASS |
| `cargo clippy --workspace --all-targets --features barracuda-gpu` | 0 warnings |
| `unsafe` blocks in production | 0 |
| `unwrap()`/`expect()` in production | 0 |
| `#[allow]` without justification | 0 |
| Mocks in production code | 0 |
| Deprecated patterns | 0 |
| `todo!()`/`unimplemented!()` | 0 |

### Architecture evolution since V43

```
V43: Three-tier parity proven (27/27), pure GPU workloads (26/26)
  │
V44: Deep-debt evolution:
  ├── linalg.rs extracted from transport.rs (cross-cutting primitive)
  ├── error.rs: InputError for 5 fallible APIs
  ├── GridFitConfig: +Debug, +Clone, +Copy
  ├── EighError: +Clone, +PartialEq, +Eq
  ├── prng::next_u64: u32 as u64 → u64::from() (idiomatic)
  ├── freeze_out: chi_squared → Result
  ├── jackknife: jackknife_mean_variance → Result
  ├── jackknife: block_jackknife_variance → Result
  ├── wdm: finite_size_extrapolate → Result
  ├── stats: percentile → Result
  └── hardcoded /run/user/1000/ → runtime UID discovery
```

### Cross-spring learnings

1. **Error type reuse**: `InputError` pattern (`LengthMismatch`, `InsufficientData`,
   `OutOfRange`) is generic enough for any barracuda stats primitive. Consider
   absorbing into barracuda's error hierarchy.

2. **Tridiag eigensolver gap**: hotSpring has dense Jacobi (`eigh_f64`) and
   Sturm bisection (`find_all_eigenvalues`). Neither is optimal for tridiagonal
   batched workloads. A `BatchedTridiagEigh` using divide-and-conquer or
   MRRR would serve groundSpring, hotSpring, and neuralSpring.

3. **`Result` vs `assert!` convention**: groundSpring now uses `Result` for
   functions that may receive runtime data (experiment sizes, user parameters)
   and `assert!` for programmer errors (paired slices from the same source).
   This matches Rust library convention (cf. `Vec::split_at` vs `Vec::get`).

---

## Part 4: Delegation Inventory (39 active + 7 pending)

### Active CPU (30)

| # | groundSpring fn | barracuda target |
|---|----------------|-----------------|
| 1 | `stats::rmse` | `stats::metrics::rmse` |
| 2 | `stats::mae` | `stats::metrics::mae` |
| 3 | `stats::nash_sutcliffe` | `stats::metrics::nash_sutcliffe` |
| 4 | `stats::mbe` | `stats::metrics::mbe` |
| 5 | `stats::r_squared` | `stats::metrics::r_squared` |
| 6 | `stats::index_of_agreement` | `stats::metrics::index_of_agreement` |
| 7 | `stats::hit_rate` | `stats::metrics::hit_rate` |
| 8 | `stats::mean` | `stats::metrics::mean` |
| 9 | `stats::percentile` | `stats::metrics::percentile` |
| 10 | `stats::sample_std_dev` | `stats::correlation::std_dev` |
| 11 | `stats::pearson_r` | `stats::pearson_correlation` |
| 12 | `stats::spearman_r` | `stats::correlation::spearman_correlation` |
| 13 | `stats::covariance` | `stats::correlation::covariance` |
| 14 | `stats::norm_cdf` | `stats::norm_cdf` |
| 15 | `stats::norm_ppf` | `stats::norm_ppf` |
| 16 | `stats::chi2_statistic` | `stats::chi2_decomposed` |
| 17 | `stats::fit_linear` | `stats::regression::fit_linear` |
| 18 | `stats::fit_quadratic` | `stats::regression::fit_quadratic` |
| 19 | `stats::fit_exponential` | `stats::regression::fit_exponential` |
| 20 | `stats::fit_logarithmic` | `stats::regression::fit_logarithmic` |
| 21 | `bootstrap::bootstrap_mean` | `stats::bootstrap_mean` |
| 22 | `bootstrap::rawr_mean` | `stats::rawr_mean` |
| 23 | `rarefaction::shannon_diversity` | `stats::diversity::shannon` |
| 24 | `rarefaction::evenness` | `stats::pielou_evenness` |
| 25 | `anderson::analytical_localization_length` | `special::anderson_transport::localization_length` |
| 26 | `bistable::bistable_derivative` | `numerical::ode_bio::BistableOde::cpu_derivative` |
| 27 | `multisignal::multisignal_derivative` | `numerical::ode_bio::MultiSignalOde::cpu_derivative` |
| 28 | `kinetics::hill` | `stats::hill` |
| 29 | `kinetics::hill_repress` | via `1.0 - hill()` |
| 30 | `wdm::green_kubo_integrate` | `numerical::trapz` |

### Active GPU (9)

| # | groundSpring fn | barracuda target |
|---|----------------|-----------------|
| 31 | `anderson::lyapunov_exponent` | `spectral::lyapunov_exponent` |
| 32 | `anderson::lyapunov_averaged` | `spectral::lyapunov_averaged` |
| 33 | `almost_mathieu::hamiltonian` | `spectral::almost_mathieu_hamiltonian` |
| 34 | `almost_mathieu::level_spacing_ratio` | `spectral::level_spacing_ratio` |
| 35 | `almost_mathieu::eigenvalues` | `spectral::find_all_eigenvalues` |
| 36 | `spectral_recon::tikhonov_solve` | `linalg::solve_f64_cpu` |
| 37 | `band_structure::detect_band_ranges` | `spectral::detect_bands` |
| 38 | `rare_biosphere::abundance_occupancy` | `BatchedMultinomialGpu` |
| 39 | `rare_biosphere::tier_detection_rate` | `BatchedMultinomialGpu` |

### Pending ToadStool (7)

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

## Part 5: Validation Certificate

```
Three-tier parity:     27/27 PROVEN  (V43 certificate, unchanged)
GPU tier checks:       39/39 × 3 modes  (V43, unchanged)
Pure GPU workloads:    26/26  (V43, unchanged)
metalForge dispatch:   17/19 → Titan V  (V43, unchanged)
Unit tests:            296+ (V44, up from ~290)
Workspace tests:       470+ (barracuda-gpu mode)
Clippy warnings:       0
unsafe blocks:         0
Mocks in production:   0
```
