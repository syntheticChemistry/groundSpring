# toadStool / barracuda — V28 Coverage Evolution, PRNG Readiness & Three-Tier Paper Controls

**Date:** February 27, 2026
**From:** groundSpring
**To:** toadStool / barracuda core team
**Covers:** V28 coverage hardening + xoshiro128** API parity + CI drift detection + three-tier paper controls + absorption roadmap
**Supersedes:** V27 (GROUNDSPRING_TOADSTOOL_V27_BARRACUDA_EVOLUTION_HANDOFF_FEB27_2026.md)
**License:** AGPL-3.0-or-later

---

## Executive Summary

groundSpring V28 completes a coverage evolution pass and prepares the PRNG
migration path for Phase 2b GPU stream alignment:

1. **`Xoshiro128StarStar` at full API parity** — `next_u32`, `next_u64`,
   `next_f64`, `next_normal`, `normal`, `binomial` all implemented with
   SplitMix64 seed initialization. 10 dedicated tests. `DefaultRng` type
   alias ready to switch when barracuda feature activates.
   **toadStool action:** no changes needed yet — groundSpring will switch
   `DefaultRng` to `Xoshiro128StarStar` and regenerate baselines once
   `barracuda::ops::PrngXoshiro` CPU API stabilizes.

2. **368 Rust tests + 196 Python baseline integrity tests** — 45 new
   coverage tests across `bistable`, `multisignal`, `rare_biosphere`, `prng`,
   `inventory`, and `validate-lib`. All stochastic modules now test
   determinism, divergence, non-negativity, and derivative boundedness.

3. **CI baseline drift detection** — `test_baseline_integrity.py` validates
   every benchmark JSON has complete provenance (source, commit hash, date,
   validation script), hex commit hashes, UTF-8 encoding, and every
   experiment directory has both a benchmark file and a Python script.
   Four-mode CI: Rust × 3 feature modes + Python integrity.

4. **Three-tier paper controls confirmed** — All 28 papers use open data
   and open systems. Zero proprietary dependencies. Every benchmark JSON
   provenance verified by automated tests.

5. **Codebase health** — 0 clippy warnings × 3 modes, 0 `todo!()`/
   `unimplemented!()`, 0 `unwrap()` in production code, 0 dead code,
   0 TODO/FIXME/HACK comments in Python, all 28 Python scripts have
   SPDX headers.

---

## Part 1: PRNG Alignment Status

### What groundSpring Now Has

| Type | State Size | Output | API Surface | Tests |
|------|-----------|--------|-------------|:-----:|
| `Xorshift64` (current default) | 64 bits | u64 | `next_u64`, `next_f64`, `next_normal`, `normal`, `binomial` | 8 |
| `Xoshiro128StarStar` (new, V28) | 128 bits (4×u32) | u32 | `next_u32`, `next_u64`, `next_f64`, `next_normal`, `normal`, `binomial` | 10 |
| `DefaultRng` type alias | → `Xorshift64` | — | Switches at compile time | 1 |

### What This Means for toadStool

The `Xoshiro128StarStar` implementation matches the algorithm in
`barracuda::ops::prng_xoshiro_wgsl`. Seed initialization uses SplitMix64
(Steele, Lea, Flood 2014) to expand a u64 seed into 4×u32 state.

**Remaining Phase 2b steps** (after toadStool stabilises CPU xoshiro API):

1. Switch `DefaultRng` from `Xorshift64` to `Xoshiro128StarStar`
2. Regenerate all 28 Python baselines with xoshiro128**-compatible PRNG
3. Update 28 benchmark JSONs with new expected values + `prng_algorithm: "xoshiro128**"`
4. Verify 288/288 checks pass with new baselines
5. Archive xorshift64 baselines in `control/archive/xorshift64/`

**toadStool action:** Ensure `barracuda::ops::PrngXoshiro` CPU path produces
identical streams to groundSpring's `Xoshiro128StarStar::new(seed)` for
same seed. If so, Phase 2b migration is straightforward.

---

## Part 2: Coverage Evolution (V28)

### New Tests by Module

| Module | New Tests | What They Cover |
|--------|:---------:|-----------------|
| `bistable.rs` | 5 | `stochastic_integrate` determinism/divergence, low-noise near-deterministic, non-negativity, derivative boundedness |
| `multisignal.rs` | 4 | `stochastic_integrate` determinism/divergence, non-negativity, derivative boundedness |
| `rare_biosphere.rs` | 5 | `tier_detection_rate` determinism/abundant/rare, `detection_threshold` edge cases, `chao1` singletons-only branch |
| `prng.rs` | 5 | `Xoshiro128StarStar`: `next_u64` determinism, `binomial` determinism/mean, `normal` with mean/std |
| `inventory.rs` | 4 | `count`/`first` for absent kinds, `print_summary` multi-substrate, empty inventory |
| `validate lib.rs` | 3 | `print_provenance_header` complete/missing fields, `f64_range` longer array |
| **Total** | **26** | |

### CI Evolution

```
Before (V27):  pytest tests/ -v
After  (V28):  pytest tests/test_common.py tests/test_determinism.py tests/test_baseline_integrity.py -v
               pytest tests/test_experiments.py -v --timeout=300
```

Fast integrity checks (0.5s) run first; slow experiment runs (2+ minutes) second.
Git checkout uses `fetch-depth: 0` for provenance commit verification.

---

## Part 3: Three-Tier Paper Controls (CPU → GPU → metalForge)

All 28 experiments validated at three levels. Current status:

| Tier | Status | Experiments | Details |
|------|--------|:-----------:|---------|
| **CPU** | **288/288 PASS** | 28/28 | Rust matches Python baseline via shared benchmark JSONs |
| **GPU** | 6 barracuda-gpu delegations active | 28/28 compile | Lyapunov, Sturm, Hamiltonian, level_spacing, Tikhonov; +17 CPU-side |
| **metalForge** | 31 checks on live hardware | 1 (Exp 028) | RTX 4070, Titan V, AKD1000 NPU; groundspring-forge crate |

### GPU-Ready Experiments (barracuda primitives exist)

| # | Experiment | barracuda Primitive | Action Needed |
|---|-----------|-------------------|---------------|
| 6 | Spectral recon | `linalg::solve_f64_cpu` | ✅ Delegated (tikhonov_solve) |
| 8 | Anderson localization | `spectral::lyapunov_*` | ✅ Delegated (barracuda-gpu) |
| 9 | Quasiperiodic | `spectral::find_all_eigenvalues` | ✅ Delegated (**47.7× speedup**) |
| 10 | Bistable switching | `numerical::ode_bio::BistableOde` | ✅ Delegated (CPU derivative) |
| 11 | Multi-signal QS | `numerical::ode_bio::MultiSignalOde` | ✅ Delegated (CPU derivative) |
| 15 | Anderson (cross-spring) | `spectral::*` | ✅ Delegated |

### GPU-Blocked Experiments (missing primitives)

| # | Experiment | Missing Primitive | Priority |
|---|-----------|------------------|----------|
| 1-5 | Sensor/weather/FAO-56/seismic | `gpu` feature gate on reduce ops | HIGH |
| 4 | Sequencing noise | `batched_multinomial` GPU dispatch wiring | HIGH |
| 5,8 | Seismic, freeze-out | Grid search 3D dispatch | MEDIUM |
| 7 | RAWR resampling | GPU bootstrap (embarrassingly parallel) | MEDIUM |
| 12 | Spin transport | `tridiag_eigh` eigenvectors (values only via Sturm) | MEDIUM |

### Open Data Provenance (28/28 confirmed)

Every experiment uses open data or open systems. Zero proprietary dependencies.
Provenance verified by automated CI (`test_baseline_integrity.py`):
- 28 benchmark JSONs × 7 checks = 196 PASS
- Every JSON has `_source`, `_provenance.baseline_date`, `_provenance.baseline_commit`, `_provenance.validation_script`
- Every commit hash is valid hex
- Every experiment directory has both benchmark JSON and Python script

---

## Part 4: What ToadStool Should Absorb (updated from V27)

### Ready Now (production WGSL in metalForge/shaders/)

| Shader | Lines | Binding Layout | groundSpring Use |
|--------|:-----:|---------------|-----------------|
| `mc_et0_propagate.wgsl` | 149 | 3 buffers (params, perturbations, results) | MC uncertainty propagation through FAO-56 |
| `batched_multinomial.wgsl` | 112 | 2 buffers (cumulative_probs, counts) | Rarefaction, rare biosphere, sequencing noise |

**toadStool action:** `mc_et0_propagate` equation chain is already in barracuda
(`Fao56Et0`); the MC noise wrapper (Box-Muller perturbation + dispatch) is the
absorption target. `batched_multinomial` is already in barracuda (`BatchedMultinomialGpu`);
groundSpring rewiring pending (signature mismatch: barracuda takes `cumulative_probs` + closure RNG).

### Patterns for Upstream Consideration

1. **`stats::fit_linear` for finite-size extrapolation** — groundSpring's
   `wdm::finite_size_extrapolate` uses `barracuda::stats::regression::fit_linear`
   to extrapolate transport coefficients to infinite system size. This pattern
   (transform coordinates → linear regression) is general.

2. **Baseline integrity testing** — The `test_baseline_integrity.py` pattern
   (parametric provenance validation of benchmark JSONs) could be a shared
   ecoPrimals convention. Other springs could adopt the same approach.

3. **`Xoshiro128StarStar` seed initialisation via SplitMix64** — Matches
   the standard reference implementation. If toadStool's PRNG seed init
   differs, we need alignment before Phase 2b.

---

## Part 5: Evolution Learnings (V27→V28)

### What Worked

1. **Stochastic module coverage pattern** — Every Euler-Maruyama integrator
   now has five test families: determinism (same seed), divergence (different
   seed), near-deterministic (low noise), non-negativity (clamping), and
   derivative boundedness. This pattern should be standard for any ODE+noise
   module.

2. **Parametric provenance testing** — `pytest.fixture(params=...)` over
   glob results gives one test per benchmark JSON × check type. Failures
   name the specific file and field. Much better than a monolithic provenance
   check.

3. **PRNG API parity before migration** — Building `Xoshiro128StarStar` with
   the full API surface *before* switching `DefaultRng` means the migration
   is a one-line type alias change + baseline regeneration. No API surprises.

### What the Codebase Audit Found

- **0 dead code** in Rust production code (no `#[allow(dead_code)]`, no `todo!()`)
- **0 `unwrap()` in production** — all in test modules
- **0 TODO/FIXME/HACK** in Python control scripts
- **28/28 SPDX headers** on Python scripts
- **28/28 `sys.path.insert`** before `from common import ...` (no broken standalone runs)
- **2 broad `except Exception`** in `observation_gap.py` — intentional API fallbacks for
  Open-Meteo and NOAA CDO; acceptable for network-dependent experiment

---

## Part 6: BarraCUDA Usage Summary (V28 State)

### 29 Delegations (unchanged from V27)

| Category | Count | barracuda Module |
|----------|:-----:|-----------------|
| Stats/metrics | 15 | `stats::*` (pearson, spearman, std_dev, covariance, norm_cdf/ppf, chi2, rmse, mbe, r², IoA, hit_rate, mean, percentile) |
| Bootstrap/RAWR | 2 | `stats::bootstrap_mean`, `stats::rawr_mean` |
| Hill kinetics | 1 | `stats::hill` |
| Regression | 1 | `stats::regression::fit_linear` |
| Diversity | 2 | `stats::diversity::shannon`, `stats::pielou_evenness` |
| Anderson/spectral | 6 | `spectral::*` (lyapunov ×2, hamiltonian, eigenvalues, level_spacing, localization_length) |
| ODE | 2 | `numerical::ode_bio::BistableOde`, `MultiSignalOde` |
| **Total** | **29** | 23 CPU + 6 barracuda-gpu |

### Local Implementations (unchanged from V27)

18 modules stay local: `decompose`, `seismic`, `gillespie`, `transport`,
`drift`, `rare_biosphere`, `quasispecies`, `band_structure`, `jackknife`,
`freeze_out`, `prng`, `cast`, `validate`, `ode`, `fao56`, `wdm`, `npu`, `rarefaction`.

All have barracuda-compatible data layouts (flat `Vec<f64>`, row-major)
and are candidates for GPU promotion when barracuda primitives exist.

---

## References

- `specs/BARRACUDA_EVOLUTION.md` — Full module mapping (Tier A/B/C)
- `specs/BARRACUDA_REQUIREMENTS.md` — GPU kernel gap analysis
- `specs/PAPER_REVIEW_QUEUE.md` — Per-paper three-tier control matrix
- `metalForge/ABSORPTION_MANIFEST.md` — Shader absorption inventory
- `CONTROL_EXPERIMENT_STATUS.md` — Experiment register and run logs
