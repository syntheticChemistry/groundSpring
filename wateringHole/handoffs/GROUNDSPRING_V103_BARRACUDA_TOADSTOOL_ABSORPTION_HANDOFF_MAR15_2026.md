# groundSpring V103 → BarraCUDA/ToadStool Absorption Handoff

**Date**: March 15, 2026
**From**: groundSpring V103 (35 experiments, 395/395 checks, 936 tests, 102 delegations)
**To**: BarraCUDA/ToadStool team
**Authority**: wateringHole (ecoPrimals Core Standards)
**Supersedes**: V102 BarraCUDA/ToadStool Niche Handoff (Mar 14, 2026)

---

## Executive Summary

groundSpring V103 is a deep debt audit focused on code quality and constant provenance.
No new delegations or API changes. All 102 delegations (61 CPU + 41 GPU) verified
green against barraCuda v0.3.5, toadStool S130+, coralReef Iteration 10.

Key evolution: every hardcoded magic number in dispatch, validation, and ESN paths
has been replaced with a named constant carrying physical provenance. This improves
auditability for any future upstream absorption of these constants into barraCuda.

---

## 1. What Changed (V102 → V103)

### Named Constants with Physical Provenance

These constants document the physics behind groundSpring's default parameter values.
If barraCuda absorbs any of these domains (batch uncertainty, regime classification),
the provenance travels with the constant.

| Module | Constant | Value | Provenance |
|--------|----------|-------|------------|
| `dispatch.rs` | `DEFAULT_REGULARIZATION` | `1e-4` | Tikhonov α for spectral reconstruction (Bazavov 2025, arXiv 2501.12259) |
| `dispatch.rs` | `DEFAULT_TAU_STEP` | `0.1` | Euclidean-time grid spacing for lattice correlator inversion |
| `dispatch.rs` | `DEFAULT_OMEGA_STEP` | `0.2` | Spectral-function frequency resolution (MeV scale) |
| `dispatch.rs` | `DEFAULT_SIGMA` | `1.0` | Default kernel width for spectral reconstruction |
| `dispatch.rs` | `DEFAULT_T0_LO/HI/STEP` | `100.0/200.0/1.0` | Freeze-out temperature scan range (MeV, Bazavov 2016 PRD 93) |
| `dispatch.rs` | `DEFAULT_K2_LO/HI/STEP` | `0.001/0.05/0.001` | Freeze-out curvature scan range (Bazavov 2016) |
| `esn/classifier.rs` | `ESN_READOUT_REGULARIZATION` | `1e-6` (f32) | Ridge regularization for ESN readout layer (hotSpring validation) |
| `lib.rs` | `eps::LOG_FLOOR` | `1e-15` | Near-zero guard for log/entropy calculations |
| `validate_tissue_anderson.rs` | `MIN_INFLAMED_EVENNESS` | `0.8` | Paper 12 (Gonzales): inflamed dermis evenness threshold |
| `validate_tissue_anderson.rs` | `MAX_HEALTHY_D_EFF` | `2.5` | Paper 12: effective diffusion barrier limit |
| `validate_tissue_anderson.rs` | `ANDERSON_3D_W_C` | `16.5` | Anderson 3D critical disorder (Slevin & Ohtsuki 1999) |
| `validate_tissue_anderson.rs` | `MIN/MAX_BARRIER_TRANSITION` | `0.4/0.8` | Paper 12: barrier regime transition bounds |
| `validate_tissue_anderson.rs` | `MAX_TOPICAL_MAB_PENETRATION` | `0.15` | Paper 12: topical mAb penetration ceiling |
| `validate_tissue_anderson.rs` | `MIN_SYSTEMIC_PENETRATION` | `0.5` | Paper 12: systemic delivery minimum |
| `validate_tissue_anderson.rs` | `MIN_GOOD_COMPOSITE_SCORE` | `0.5` | Paper 12: minimum acceptable composite treatment score |
| `groundspring-validate/lib.rs` | `TOL_ET0` | `0.005` | ET₀ method comparison tolerance — 0.5% relative, justified by FAO-56 precision |

### Module Extraction

`biomeos/interaction.rs` extracted from `biomeos/mod.rs` (683 → 531 LOC):
- `DiscoveredPrimal` struct
- `discover_primals()`, `biomeos_socket_dir()`
- `primal_health()`, `direct_primal_rpc()`
- `proprioception()`, `topology()`

This extraction is internal to groundSpring. No API surface change for barraCuda.

### Quality Fixes

- `log::error!` replaces `eprintln!` in `biomeos/server.rs`
- `clippy::doc_markdown` fixes in validation binaries
- `clippy::let_unit_value` fix in server test

---

## 2. Delegation Inventory (Unchanged)

| Category | CPU | GPU | Total |
|----------|-----|-----|-------|
| Stats (mean, std, correlation, regression, chi²) | 15 | 12 | 27 |
| Bio (Gillespie, multinomial, Wright-Fisher, Hill, Monod) | 8 | 6 | 14 |
| Spectral (Anderson, Almost-Mathieu, Lanczos, ESN, ESD) | 12 | 9 | 21 |
| Hydrology (FAO-56, Hargreaves, Makkink, Turc, Hamon) | 8 | 3 | 11 |
| Bootstrap/Jackknife/RAWR | 4 | 3 | 7 |
| FFT, Autocorrelation, Peak detection | 3 | 3 | 6 |
| Other (seismic, drift, precision, covariance) | 11 | 5 | 16 |
| **Total** | **61** | **41** | **102** |

---

## 3. Absorption Opportunities (Updated)

### Priority 1 — Batch Primitives

These are the same P2 opportunities from V102, elevated to P1 because the
named constants and provenance documentation now make absorption cleaner:

| Opportunity | Current Path | Proposed barraCuda Primitive | Constants Included |
|-------------|-------------|-----------------------------|--------------------|
| Batch uncertainty budget | `bootstrap_mean` + `jackknife_mean_variance` (2 dispatches) | `barracuda::stats::uncertainty_budget` (1 dispatch) | — |
| Batch regime classification | `spectral_features` + `classify_by_spacing_ratio` (2 calls) | `barracuda::stats::regime_classification` (1 dispatch) | `ESN_READOUT_REGULARIZATION` |
| Spectral reconstruction defaults | `tikhonov_solve` with manual params | `barracuda::spectral::tikhonov_solve` with default config | `DEFAULT_REGULARIZATION`, `DEFAULT_TAU_STEP`, `DEFAULT_OMEGA_STEP`, `DEFAULT_SIGMA` |
| Freeze-out grid scan defaults | `grid_fit_2d` with manual ranges | `barracuda::inverse::freeze_out_scan` with config | `DEFAULT_T0_*`, `DEFAULT_K2_*` |

### Priority 2 — Streaming for Continuous Niches

When biomeOS runs groundSpring in `Continuous` mode at fixed Hz, per-call GPU
buffer allocation becomes overhead. A streaming primitive holding persistent
buffers across ticks would reduce this.

### Priority 3 — PRNG Alignment

`prng::Xorshift64` remains local. Roadmap: align to `xoshiro256++` when barraCuda
absorbs it. Documented in `specs/BARRACUDA_EVOLUTION.md` Tier B.

---

## 4. Learnings Relevant to barraCuda/ToadStool Evolution

### Constant Provenance Pattern

groundSpring V103 proves a pattern: every numeric constant carries:
1. A `const` name describing its physical role
2. A doc comment citing the source paper/standard
3. A validation binary that exercises it with pass/fail

This pattern could benefit barraCuda's own internal constants (tolerances,
precision thresholds, epsilon guards). If barraCuda adopts named-constant
provenance for its `eps::*` module, cross-spring auditability improves.

### `eps::LOG_FLOOR` vs `eps::SSA_FLOOR`

groundSpring now has `eps::LOG_FLOOR = 1e-15` (for log/entropy paths) alongside
the existing `eps::SSA_FLOOR = 1e-15` (for SSA Gillespie paths, gated behind
`barracuda-gpu`). Both are `1e-15` but serve different semantic purposes. If
barraCuda centralizes epsilon guards, these should remain distinct constants
with documented domains.

### Structured Logging

groundSpring V103 migrated the last `eprintln!` to `log::error!`. All Springs
should use structured logging for biomeOS observability integration.

---

## 5. coralReef Notes

No change from V102. Sovereign f64 compilation pipeline independent of
groundSpring evolution. groundSpring consumes via `get_device_f64_safe()`
which routes to DF64 or CPU fallback when native f64 shared memory is broken.

---

## Verification

```
cargo fmt --all -- --check: PASS
cargo clippy --workspace --all-targets -- -D warnings: PASS
cargo clippy --workspace --all-targets --all-features -- -D warnings: PASS
cargo doc --workspace --no-deps: PASS
cargo test --workspace: 936 PASS (all feature gates)
```

---

## Next Steps

1. **barraCuda**: Review P1 batch primitive opportunities. Named constants with provenance are ready for upstream adoption.
2. **ToadStool**: No blocking action. `compute.execute` routing unchanged.
3. **coralReef**: No change. Sovereign f64 pipeline independent.
4. **All primals**: Consider adopting the named-constant-with-provenance pattern for internal tolerances and thresholds.
