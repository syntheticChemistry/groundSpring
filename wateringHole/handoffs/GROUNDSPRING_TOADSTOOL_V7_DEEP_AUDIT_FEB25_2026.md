# groundSpring → ToadStool Handoff V7: Deep Audit, Property Testing & Python Quality

**Date**: February 25, 2026
**From**: groundSpring (validation spring — measurement noise characterization)
**To**: ToadStool / BarraCUDA team
**Supersedes**: V6 (complete rewiring & cross-spring evolution)
**ToadStool baseline**: Sessions 51–62 + DF64 expansion (Feb 24–25, 2026)

---

## Summary

V7 reports the results of a **comprehensive deep audit** of groundSpring.
All code quality, testing, documentation, and compliance gaps identified in
the audit have been resolved. Key improvements:

- **Property-based testing**: 14 proptest invariants covering mathematical
  identities (mean, std_dev, RMSE, Pearson r, CDF/PPF round-trips)
- **Coverage**: 98.64% workspace line coverage (cargo-llvm-cov)
- **Python quality**: ruff (0 errors) + mypy (0 errors) across 23 source files
- **stats module refactored**: 830-line `stats.rs` → 3 focused submodules
  (metrics 405L, correlation 258L, distributions 235L)
- **Hardcoded fallbacks eliminated**: all validation reads from JSON, no silent defaults
- **Runtime discovery**: hardcoded `Path.home()` replaced with env-var + relative discovery
- **CI workflow**: GitHub Actions with fmt, clippy, doc, test, validation, ruff, mypy, coverage

### What Changed Since V6

| Category | V6 | V7 |
|----------|----|----|
| Rust tests | 122 unit + 1 doc | 131 unit + 14 proptest + 8 integration + 1 doc = **154** |
| Line coverage | unmeasured | **98.64%** (cargo-llvm-cov) |
| Python quality | untested | ruff 0 errors, mypy 0 errors, **34/34 pytest** |
| stats module | monolithic 830L `stats.rs` | `stats/` with 3 submodules (<410L each) |
| Validation binaries | unwrap() in JSON parsing | `.expect()` with descriptive messages everywhere |
| MC range fallbacks | `unwrap_or(3.5)` hardcoded | `.expect("et0_mean_range[0]")` — reads from JSON |
| Inter-primal discovery | hardcoded `Path.home()` | `AIRSPRING_ROOT` / `ECOPRIMALS_ROOT` env vars + relative |
| CI | none | `.github/workflows/ci.yml` (4 jobs) |
| Python version | 3.11 assumed | **3.10+** (tested, pyproject updated) |

### Quality Gates

| Gate | Status |
|------|--------|
| `cargo fmt --check` | Clean |
| `cargo clippy --workspace --all-features -- -D warnings` | 0 warnings |
| `cargo doc --workspace --no-deps` | 0 warnings |
| `cargo test --workspace` | **154/154 PASS** |
| 8 validation binaries (local) | **119/119 PASS** |
| `ruff check control/ tests/` | 0 errors |
| `mypy control/ tests/` | 0 errors |
| `python3 -m pytest tests/ -v` | **34/34 PASS** |
| `cargo llvm-cov --workspace` | **98.64%** line coverage |

---

## Part 1: What BarraCUDA Should Absorb (unchanged from V6)

### Priority 1: `batched_multinomial.wgsl`

112-line production WGSL in `metalForge/shaders/`. No equivalent in barracuda.
Use case: GPU-scale rarefaction (100k+ replicates) for Exp 004 and wetSpring.

### Priority 2: `rawr_weighted_mean_f64` kernel

RAWR (Wang et al. 2021 ISMB): Dirichlet(1,...,1) weights via normalized
Exp(1) variates, then weighted dot product per replicate. Embarrassingly
parallel. CPU reference: `bootstrap::rawr_mean`. No barracuda equivalent.

### Priority 3: `mc_et0_propagate.wgsl` MC wrapper

149-line production WGSL. FAO-56 equation chain is absorbed upstream
(`BatchedElementwiseF64::fao56_et0_batch`), but the MC uncertainty
propagation wrapper is the absorption target.

---

## Part 2: Lessons for BarraCUDA Evolution (expanded from V6)

### Carried forward from V6

1. **Private submodule re-exports**: `spectral::anderson` is private.
   Document re-export at `spectral::*` prominently.
2. **Seed convention divergence**: Local `base_seed + i` vs barracuda
   `base_seed + r * 1000`. Document per-function.
3. **RAWR is a valuable primitive**: Add to `barracuda::stats` alongside
   `bootstrap_mean`.
4. **`f64::total_cmp`**: NaN-safe float sorting. All springs should use it.
5. **`cpu-math` feature gate**: Works perfectly for CPU-only delegation.
6. **Centralized cast helpers**: `cast` module prevents `#[allow]` proliferation.
7. **Analytical vs numerical tolerance**: Use ratio (0.2..5.0), not absolute.

### New from V7 audit

8. **Property-based testing with proptest**: Mathematical invariants that manual
   test vectors cannot cover. groundSpring now has 14 proptest tests covering:
   - Mean/std_dev algebraic identities
   - RMSE/MBE symmetry (antisymmetry, zero-on-identity)
   - Pearson r bounded [-1, 1], perfect fit → 1.0
   - CDF/PPF round-trip within 1e-6
   - CDF monotonicity and [0,1] bounds

   **Recommendation**: BarraCUDA should add proptest to its validation arsenal.
   Key invariants: GPU output ≡ CPU output for all inputs; reduce operations
   are commutative/associative within tolerance; PRNG period never enters
   short cycle.

9. **stats module decomposition pattern**: Splitting a large module (830 lines)
   into domain-focused submodules (`metrics`, `correlation`, `distributions`)
   while maintaining a single public API via `mod.rs` re-exports. Zero
   downstream breakage. Pattern applicable to barracuda's own growing stats module.

10. **Python baseline quality gates**: ruff + mypy on Python control experiments
    catches type errors, unused variables, and import hygiene before they
    corrupt baselines. groundSpring's `pyproject.toml` config suppresses
    intentional scientific conventions (Greek letters, uppercase variables).

11. **Edge-case coverage matters**: The 98.64% coverage was achieved by adding
    tests for all early-return paths (empty input, zero-rate termination,
    burn-in past trajectory end, single-element data). These edge cases
    are exactly where CPU/GPU divergence is most likely.

12. **`unwrap_or()` with hardcoded fallback is a silent data source**:
    groundSpring had `unwrap_or(3.5)` on MC range bounds — if JSON parsing
    failed, the validation would silently pass with the wrong expected value.
    All such patterns replaced with `.expect()` to fail loud.

---

## Part 3: Three-Tier Control Matrix

### Current Status

| Tier | Papers | Status |
|------|--------|--------|
| CPU (Rust matches Python) | 8 done, 19 queued | **119/119 PASS** |
| GPU (BarraCUDA GPU matches CPU) | 6 ops GPU-pending | Blocked on GPU adapter |
| metalForge (cross-substrate) | Future | After GPU tier |

### Open Data Audit

All 27 papers in the queue use open data or open systems. Zero proprietary
dependencies. Full audit in `specs/PAPER_REVIEW_QUEUE.md`.

### GPU-Ready Papers (barracuda primitives exist)

Papers 9, 10, 12, 14, 15, 16 — can proceed to GPU tier once CPU tier completes.

### GPU-Blocked Papers

- Papers 6, 7: FFT gap in barracuda
- Papers 1-5: ops exist but need GPU adapter
- Papers 4, 20-21: batched multinomial Tier C absorption needed

---

## Part 4: Complete Delegation Inventory (unchanged)

11 active CPU delegations. See V6 Part 1 for the full table.

| # | groundSpring | BarraCUDA | Gate |
|---|-------------|-----------|------|
| 1 | `stats::pearson_r` | `stats::pearson_correlation` | barracuda |
| 2 | `stats::spearman_r` | `stats::correlation::spearman_correlation` | barracuda |
| 3 | `stats::sample_std_dev` | `stats::correlation::std_dev` | barracuda |
| 4 | `stats::covariance` | `stats::correlation::covariance` | barracuda |
| 5 | `stats::norm_cdf` | `stats::norm_cdf` | barracuda |
| 6 | `stats::norm_ppf` | `stats::norm_ppf` | barracuda |
| 7 | `stats::chi2_statistic` | `stats::chi2_decomposed` | barracuda |
| 8 | `bootstrap::bootstrap_mean` | `stats::bootstrap_mean` | barracuda |
| 9 | `anderson::lyapunov_exponent` | `spectral::lyapunov_exponent` | barracuda-gpu |
| 10 | `anderson::lyapunov_averaged` | `spectral::lyapunov_averaged` | barracuda-gpu |
| 11 | `anderson::analytical_localization_length` | `special::anderson_transport::localization_length` | barracuda |

---

## Part 5: Files Changed Since V6

### New files
- `crates/groundspring/src/stats/mod.rs` — module root with re-exports
- `crates/groundspring/src/stats/metrics.rs` — error metrics, descriptive stats
- `crates/groundspring/src/stats/correlation.rs` — Pearson/Spearman, covariance
- `crates/groundspring/src/stats/distributions.rs` — CDF, PPF, chi-squared
- `crates/groundspring/tests/proptest_invariants.rs` — 14 property tests
- `.github/workflows/ci.yml` — GitHub Actions CI (4 jobs)
- `wateringHole/` — local handoff directory
- This handoff (V7)

### Deleted files
- `crates/groundspring/src/stats.rs` — replaced by `stats/` module directory

### Modified (code)
- `crates/groundspring/Cargo.toml` — added `proptest = "1"` dev-dependency
- `crates/groundspring/src/gillespie.rs` — 7 new edge-case tests
- `crates/groundspring/src/validate.rs` — 3 new tests (Debug, check_range, check_min)
- `crates/groundspring-validate/src/validate_fao56.rs` — `.unwrap_or()` → `.expect()`
- `pyproject.toml` — Python 3.10+, ruff ignores for scientific conventions, mypy config
- 6 Python files — ruff fixes (imports, zip strict, ternary, unused var, subprocess check)
- `control/error_propagation/error_propagation_fao56.py` — runtime discovery via env vars
- `control/observation_gap/observation_gap.py` — `np.asarray()` for type safety

### Modified (docs)
- README.md, CHANGELOG.md, CONTRIBUTING.md — harmonized to 154 tests, 98.64% coverage
- whitePaper/STUDY.md, whitePaper/METHODOLOGY.md — same
- whitePaper/baseCamp/README.md, whitePaper/experiments/README.md — same
- CONTROL_EXPERIMENT_STATUS.md — same
- specs/BARRACUDA_REQUIREMENTS.md, metalForge/README.md, metalForge/ABSORPTION_MANIFEST.md

---

## Handoff Checklist

- [x] Deep audit: all recommendations executed
- [x] stats module refactored (3 submodules, all <410 lines)
- [x] 14 proptest invariant tests (mathematical identities)
- [x] 98.64% workspace line coverage (measured, not estimated)
- [x] ruff 0 errors, mypy 0 errors across 23 Python source files
- [x] All `.unwrap_or(literal)` fallbacks replaced with `.expect()`
- [x] Runtime inter-primal discovery via env vars (no hardcoded paths)
- [x] CI workflow: fmt, clippy, doc, test, validation, ruff, mypy, coverage
- [x] All docs harmonized to 154 tests, 98.64% coverage, 11 delegations
- [x] AGPL-3.0 headers on all 48 source files
- [x] All files under 1000 lines (wateringHole compliance)
- [x] Zero TODOs, FIXMEs, mocks in production code
- [x] V6 superseded

---

*groundSpring Phase 2a: complete. Ready for Phase 2b (PRNG alignment) and Phase 3 (full GPU pipeline).*
