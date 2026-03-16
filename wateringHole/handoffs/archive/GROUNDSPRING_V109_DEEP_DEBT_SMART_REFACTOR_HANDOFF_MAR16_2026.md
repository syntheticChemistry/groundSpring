# groundSpring V109 — Deep Debt Resolution + Smart Refactoring Handoff

**Date**: March 16, 2026
**From**: groundSpring V109 (39 modules, 878+ tests, 102 delegations)
**To**: barraCuda / toadStool / coralReef teams, All Springs
**Authority**: wateringHole (ecoPrimals Core Standards)
**Supersedes**: GROUNDSPRING_V108_DEEP_DEBT_ABSORPTION_HANDOFF_MAR16_2026.md
**Pins**: barraCuda v0.3.5, toadStool S155b, coralReef Iteration 49+
**License**: AGPL-3.0-or-later (SCYBORG Provenance Trio)

---

## Executive Summary

V109 is a deep debt resolution and structural evolution sprint. No new science
or new delegations — this version eliminates all remaining `panic!()`/`expect()`
paths in validation binaries, refactors four large modules into coherent
submodule trees, centralizes the last hardcoded string, and pins Python
dependencies for baseline reproducibility.

Key changes:
1. **Zero-panic validation binaries** — all 28 `serde_json::from_str().expect()`
   calls converted to `let Ok(...) else { eprintln!("FATAL: ..."); return 1; }`
2. **Smart module refactoring** — 4 modules split by algorithmic domain, not
   arbitrary line counts
3. **Named physical constants** — `ET0_PLAUSIBLE_MIN_MM` / `ET0_PLAUSIBLE_MAX_MM`
   with FAO-56 provenance
4. **Last hardcoded socket** — `"biomeos-neural-api.sock"` → `primal_names::LEGACY_NEURAL_API_SOCK`
5. **Python dependency pinning** — upper bounds on numpy/scipy/pandas prevent
   silent PRNG drift from major-version upgrades
6. **Non-delegation rationale documented** — `mat_transpose_mul`/`mat_transpose_vec`
   kept local (small matrices; Cholesky delegated); barraCuda `GemmF64` lacks
   transpose flags

---

## Quality Gates (V109)

| Gate | Status |
|------|--------|
| `cargo test --workspace --no-default-features` | **878 passed, 0 failed** |
| `cargo clippy --workspace --no-default-features -- -D warnings -W pedantic` | **0 warnings** |
| `cargo fmt --all -- --check` | **0 diff** |
| `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-default-features --no-deps` | **0 warnings** |
| License | **AGPL-3.0-or-later** (SCYBORG trio) |
| `unsafe` in application code | **forbidden** (`#![forbid(unsafe_code)]` on all 3 crate roots) |
| `#![deny(clippy::expect_used, clippy::unwrap_used)]` | **enforced** (all 3 crate roots) |
| `panic!()` in validation binaries | **0** |
| `expect()` in validation binaries | **0** (outside `#[expect()]`-annotated lib helpers) |
| Bare numeric literals in production | **0** |
| Hardcoded primal name strings | **0** |
| TODO/FIXME/HACK in .rs | **0** |
| Files > 1000 LOC | **0** (largest: 705 LOC) |
| Mocks in production | **0** |

---

## Part 1: Zero-Panic Validation Binaries

### What changed

Every validation binary in `crates/groundspring-validate/src/validate_*.rs`
previously used `serde_json::from_str(BENCHMARK).expect("valid benchmark JSON")`
at the top level. V109 converts all 28 such call sites to:

```rust
let Ok(bench) = serde_json::from_str::<serde_json::Value>(BENCHMARK) else {
    eprintln!("FATAL: benchmark JSON failed to parse");
    return 1;
};
```

Additionally:
- `validate_notill_sampling.rs`: `panic!("benchmark depths[{i}]: expected u64")`
  replaced with `let Ok(...) else { eprintln!(...); return 1; }`
- `validate_et0_methods.rs`: `.as_array().expect("seasonal array")` replaced
  with `let Some(seasonal) = ... else { eprintln!(...); return; }`
- `validate_nucleus_stack.rs`: JSON parse `expect()` converted to same pattern

### Why this matters

Validation binaries are the ecosystem's trust layer — they prove mathematical
correctness via exit codes (0 = pass, 1 = fail). A `panic!()` produces
confusing output (backtrace noise) and unreliable exit codes on some platforms.
The `let Ok(...) else { return 1; }` pattern ensures clean diagnostic output
and a deterministic exit code 1 on any malformed input.

### Pattern for all springs

This is the recommended pattern for all validation binaries ecosystem-wide:

```rust
fn main() -> i32 {
    let Ok(bench) = serde_json::from_str::<serde_json::Value>(BENCHMARK) else {
        eprintln!("FATAL: benchmark JSON failed to parse");
        return 1;
    };
    // ... validation logic ...
}
```

---

## Part 2: Smart Module Refactoring

Four modules were refactored by algorithmic domain (not arbitrary line counts):

### `groundspring-validate/src/lib.rs` (647 → 506 LOC)

Extracted two coherent submodules:
- **`tolerances.rs`** (106 LOC): All `TOL_*`, `THRESHOLD_*`, `EPS_*`,
  `ET0_PLAUSIBLE_*` constants with mathematical provenance comments
- **`provenance.rs`** (71 LOC): `print_provenance_header` and
  `try_print_provenance_header`

The parent `lib.rs` retains `BenchFieldError`, `BenchResult`, `get_*` JSON
accessors, and panicking test helpers. All constants re-exported for backward
compatibility.

### `stats/regression.rs` (624 → 4 files)

Split by algorithm family:
- **`linear.rs`** (160 LOC): `fit_linear`, `fit_linear_cpu`, `r_squared_from_residuals`
- **`quadratic.rs`** (153 LOC): `fit_quadratic`, `fit_quadratic_cpu`, `det3`, `cramer3`
- **`nonlinear.rs`** (214 LOC): `fit_exponential`, `fit_logarithmic` + CPU variants
- **`mod.rs`** (148 LOC): `LinearFit`, `NonlinearFit`, `fit_all`, re-exports

### `fao56/mod.rs` (642 → 47 LOC)

Split by ET₀ method domain:
- **`daily.rs`** (299 LOC): `DailyWeatherInputs`, `daily_et0`, batch variants
- **`hargreaves.rs`** (135 LOC): `hargreaves_et0`, batch variants
- **`crop_soil.rs`** (120 LOC): `crop_coefficient`, `soil_water_balance`

### `fao56/pipeline.rs` (623 → 3 files)

Split by pipeline concern:
- **`monte_carlo.rs`** (261 LOC): `Et0Uncertainties`, `McEt0Result`, GPU/CPU MC
- **`seasonal.rs`** (348 LOC): `SeasonalCellInputs`, `SeasonalParams`, multi-day loop
- **`mod.rs`** (33 LOC): Physical constants, re-exports

### Pattern worth absorbing

The "smart refactoring" principle: split by **algorithmic responsibility**, not
line count. Each submodule should be independently testable and have a single
coherent purpose. Tests stay with their implementation.

---

## Part 3: barraCuda Primitive Consumption (V109 — 102 Active)

No new delegations in V109. Current inventory unchanged from V108:

| Category | Count | Examples |
|----------|------:|---------|
| CPU delegated | 61 | stats (pearson, spearman, std_dev, welford, norm_cdf/ppf, chi2, rmse, mbe, r², IoA, hit_rate, mean, percentile), bootstrap/rawr, diversity, kinetics, bistable/multisignal ODE, anderson localization_length, drift kimura, jackknife, fao56, seismic grid, band_structure, quasispecies, rare_biosphere, wdm regression |
| GPU dispatched | 41 | lyapunov (spectral), almost_mathieu (spectral), gillespie (bio), wright_fisher (bio), batched_multinomial (bio), bootstrap_mean (GPU), mc_et0 (GPU), seasonal_pipeline (GPU), cholesky (linalg), tikhonov (linalg), anderson_4d (spectral), wegner_block_4d (spectral), esn (domain-esn), lanczos (spectral), stats GPU (mean/std/rmse/mbe/pearson via reduce ops) |

### Remaining gaps (2 items — unchanged)

| Module | barraCuda Target | Blocker |
|--------|-----------------|---------|
| `transport::tridiag_eigh` | `linalg::eigh_f64` | GPU eigenvectors not yet in barraCuda (eigenvalues only via Sturm) |
| `prng::Xorshift64` | `PrngXoshiro` | Different PRNG family; baseline regeneration needed |

### Non-delegation rationale (documented in V109)

`spectral_recon.rs` retains local `mat_transpose_mul` and `mat_transpose_vec`
because:
1. barraCuda `GemmF64` lacks transpose flags — delegation would require
   materializing a transposed copy, negating zero-copy benefits
2. Matrix sizes are small (n_omega × n_tau, typically < 100×100)
3. GPU dispatch overhead exceeds computation time for these sizes
4. The heavy operation (`cholesky_f64` → Tikhonov solve) IS delegated

**Action for barraCuda team**: When `GemmF64` gains transpose flags
(`TransA`/`TransB`), groundSpring can delegate these too. Low priority —
the savings are minimal.

---

## Part 4: Patterns Worth Absorbing Ecosystem-Wide

### 1. Zero-panic validation binaries

All springs should convert `serde_json::from_str(...).expect(...)` in
validation binary `main()` to the `let Ok(...) else { return 1; }` pattern.
Benefits: clean error messages, reliable exit codes, no backtrace noise.

### 2. Smart refactoring by responsibility

When a module exceeds 500 LOC, split by **algorithmic family** not arbitrary
line count. Each submodule gets its own tests. Parent becomes a thin re-export
module. This preserves public API while improving navigability.

### 3. Tolerance module pattern

`groundspring-validate/src/tolerances.rs` is a self-contained module of named
tolerance constants with mathematical provenance comments. Every constant
explains its origin (e.g., "FAO-56 Table 2 plausible ET₀ range"). This pattern
(also adopted by wetSpring V121 with 214 constants) should be standard across
all springs.

### 4. Python dependency pinning

`pyproject.toml` now pins upper bounds (`numpy>=1.24,<2.0`, etc.) to prevent
PRNG drift from silent major-version upgrades that change internal random
number generators. All springs with Python baselines should adopt this.

---

## Part 5: Learnings Relevant to barraCuda/toadStool Evolution

### GemmF64 transpose flags

groundSpring's `spectral_recon` module needs A^T·A and A^T·b operations on
small matrices. The current `GemmF64` API requires pre-materialized transposed
matrices. Adding `TransA`/`TransB` flags (standard BLAS convention) would
enable zero-copy delegation for these patterns. This affects any spring doing
least-squares or normal equations.

### Eigenvector gap

The `transport::tridiag_eigh` module uses implicit QL with Wilkinson shifts to
compute both eigenvalues AND eigenvectors of tridiagonal matrices. barraCuda's
Sturm method computes eigenvalues only. For groundSpring's spin chain transport
(Exp 012), eigenvectors are essential (wavepacket decomposition). Promoting
the QL algorithm to barraCuda would benefit any spring needing full spectral
decomposition.

### PRNG alignment path

groundSpring uses Xorshift64 + Box-Muller for all stochastic experiments.
barraCuda uses xoshiro128**. Aligning requires regenerating all 29 benchmark
JSONs with the new PRNG, which means rerunning all Python baselines. This is a
coordinated effort across Python and Rust codebases. The payoff is GPU PRNG
dispatch via barraCuda's `PrngXoshiro` shader.

---

## Part 6: What Springs Should Know

### For wetSpring
- The tolerance module pattern (`tolerances.rs` with provenance comments) is
  now proven in both wetSpring (214 constants) and groundSpring (20+ constants).
  Consider it a de facto ecosystem standard.

### For hotSpring
- The zero-panic validation pattern is especially important for hotSpring's
  GPU validation binaries where backtrace noise can obscure real failures.

### For airSpring
- `ET0_PLAUSIBLE_MIN_MM` (0.01) and `ET0_PLAUSIBLE_MAX_MM` (15.0) are now
  named constants with FAO-56 provenance. airSpring may want to reference
  or re-export these for consistent physical bounds checking.

### For all springs
- Python `pyproject.toml` should pin numpy/scipy/pandas upper bounds to
  prevent baseline drift. groundSpring's `numpy>=1.24,<2.0` pattern works.

---

*groundSpring V109 — March 16, 2026*
*878 tests, 0 clippy, 0 fmt diff, 0 doc warnings*
*102 delegations (61 CPU + 41 GPU) — barraCuda v0.3.5*
*Zero panic, zero expect, zero hardcode, zero unsafe*
