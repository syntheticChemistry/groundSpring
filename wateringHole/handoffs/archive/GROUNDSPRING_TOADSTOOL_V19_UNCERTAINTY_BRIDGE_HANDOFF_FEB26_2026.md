# groundSpring → ToadStool/BarraCUDA V19 — Uncertainty Bridge + Extension Roadmap

**Date**: February 26, 2026
**From**: groundSpring (noise validation + environmental modeling)
**To**: ToadStool/BarraCUDA team
**Previous**: V18 (idiomatic Rust + full provenance)
**groundSpring HEAD**: `459e5d0` (V19 uncertainty bridge)
**License**: AGPL-3.0-only

---

## Executive Summary

- **226 Rust tests** (was 225), **185/185 validation checks** (was 177), **98.93% coverage**
- **New Experiment 015: Uncertainty Bridge** — first cross-domain experiment, propagating sensor noise (Exp 001) through Anderson localization (Exp 008) to predict localization length uncertainty. 8/8 PASS both Python and Rust.
- **Zero `#[allow]` remaining** — the last `#[allow(clippy::many_single_char_names)]` in `transport.rs` converted to `#[expect(..., reason = "...")]`. All suppressions now have documented rationale.
- **Seismic refactor**: `grid_search_inversion` refactored — `origin_time_and_rms()` extracted for clarity and testability. Clippy `too_many_arguments` resolved via `BridgeParams` struct pattern.
- **Gen3 bridge validated**: Exp 015 confirms the precursor for Sub-thesis 01+06 (Anderson-QS bridge). Sensor noise quantification in θ propagates to ~3-4% CV in localization length ξ.

---

## Part 1: Experiment 015 — Uncertainty Bridge

### What It Does

Cross-domain uncertainty propagation pipeline:

```
θ_measured = θ_true + bias + N(0,σ)     Exp 001: Dong et al. 2020 sensor noise
W_eff = α(1 − θ) + β                   moisture → Anderson disorder mapping
γ = lyapunov_averaged(W_eff, E=0)       Exp 008: 1D transfer matrix
ξ = 1/γ                                localization length
```

Monte Carlo: 200 samples × 10 disorder realizations per sample.

### Key Findings

| Sensor | CV(ξ) raw | CV(ξ) corrected | Improvement |
|--------|-----------|-----------------|-------------|
| CS616 Sand | 0.027-0.032 | 0.026-0.031 | ~5% |
| EC5 Sandy Clay Loam | 0.041-0.043 | 0.042-0.043 | ~0% |

**Physical insight**: At θ ≈ 0.30 (typical soil moisture), the Anderson disorder W ≈ 11 is in the saturated Lyapunov regime where γ is insensitive to small perturbations. Bias correction improves θ accuracy but has minimal effect on ξ uncertainty because the physics has saturated.

**Sensor ranking preserved**: Higher-noise sensors (EC5) produce higher CV(ξ), confirming that sensor quality matters even in the saturated regime.

### Modules Used

Exp 015 validates by composition — no new library modules were needed:
- `groundspring::anderson` — `lyapunov_averaged`, `localization_length`
- `groundspring::prng` — `Xorshift64` for Monte Carlo noise generation
- `groundspring::validate` — `ValidationHarness`

### BarraCUDA Relevance

The Anderson model (`lyapunov_averaged`) is already delegated to barracuda:
- CPU path: `barracuda::spectral::lyapunov_exponent`
- GPU path: could parallelize the 200 MC samples trivially

**ToadStool opportunity**: The MC loop in `validate_uncertainty_bridge.rs::propagate_sensor_noise()` is embarrassingly parallel. A single barracuda dispatch could run all 200 noise perturbations simultaneously, with the outer loop over realizations handled by the existing `lyapunov_averaged`.

---

## Part 2: Code Quality Evolution

### Last `#[allow]` Eliminated

```rust
// Before (V18):
#[allow(clippy::many_single_char_names)]
fn implicit_ql(d: &mut [f64], e: &mut [f64], z: &mut [f64], n: usize) {

// After (V19):
#[expect(clippy::many_single_char_names,
    reason = "standard QL algorithm notation (LAPACK dsteqr convention)")]
fn implicit_ql(d: &mut [f64], e: &mut [f64], z: &mut [f64], n: usize) {
```

**Impact**: Every lint suppression in the codebase now has a documented `reason`. This is modern Rust best practice — `#[expect]` will warn if the lint is no longer triggered, preventing stale suppressions.

### Seismic Refactor

`grid_search_inversion` (84 lines) refactored:
- Extracted `origin_time_and_rms()` — pure function computing origin time estimate and RMS residual from paired observed/predicted travel times
- Main function is now 55 lines with clear separation between grid traversal and evaluation
- `BridgeParams` struct pattern applied to validation binary to satisfy `clippy::too_many_arguments`

---

## Part 3: Extension Roadmap for ToadStool

### Immediate (V20+)

1. **Hill kinetics delegation**: `kinetics::hill()` and `kinetics::hill_repress()` are stubbed for barracuda (V18). ToadStool can wire delegation #27-28.

2. **MC parallelization**: Exp 015's uncertainty bridge MC loop is a natural barracuda batch dispatch target. The pattern (perturb → compute γ → collect ξ) maps cleanly to a GPU kernel.

3. **Eigenvector solver**: `transport.rs::tridiag_eigh` uses flat buffers (V18). Ready for barracuda absorption — the Sturm solver from hotSpring S26 already handles eigenvalues; eigenvectors are the gap.

### Near-Term (Gen3 Papers 22-24)

4. **Anderson-QS bridge papers**: Exp 015 validates the precursor. Next experiments will test the full bridge: does Anderson ξ predict observed QS communication range in real soil?
   - Paper 22: Rodriguez-Verdugo adaptive laboratory evolution
   - Paper 23: Blackburn-Meselson mutation accumulation
   - Paper 24: Wiser LTEE fitness dynamics
   These papers provide the biological constraint data. ToadStool's ODE integration performance (barracuda `bistable_derivative`, `multisignal_derivative`) will be critical.

5. **Real data integration**: Exp 002 already has ERA5 + NOAA CDO infrastructure. Future experiments can reuse this pattern for other open datasets.

### Long-Term (metalForge)

6. **Cross-substrate dispatch**: The uncertainty bridge MC loop is a perfect candidate for NPU offload — it's embarrassingly parallel with no data dependencies between samples. metalForge could dispatch the 200 samples across CPU/GPU/NPU automatically.

---

## Part 4: Three-Tier Control Matrix

| Tier | Experiments | Status |
|------|-------------|--------|
| **CPU (Rust)** | 001-015 | 185/185 PASS, 15 binaries |
| **Barracuda CPU** | 001-009, 012-015 | 26 delegations, graceful fallback |
| **Barracuda GPU** | 009 (eigenvalues) | 5 GPU delegations, Sturm solver |
| **metalForge** | Exp 015 MC (candidate) | Embarrassingly parallel pipeline |

---

## Verification Commands

```bash
cargo test --workspace                        # 226 tests, all pass
cargo clippy --workspace -- -D warnings       # zero warnings
cargo run --bin validate-uncertainty-bridge    # 8/8 PASS
cargo llvm-cov --workspace --summary-only     # 98.93% line coverage
python3 control/uncertainty_bridge/uncertainty_bridge.py  # 8/8 PASS
python3 -m pytest tests/test_experiments.py::TestExperimentExitCodes::test_exp015_uncertainty_bridge -v
```

---

## Action Items for ToadStool/BarraCUDA

1. **Wire Hill kinetics delegation** (#27-28): `barracuda::stats::hill(x, k, n)` — verify signature matches, enable in V20
2. **Evaluate MC batch kernel**: Can barracuda parallelize the uncertainty bridge MC loop? The pattern is: (θ perturbation → W mapping → lyapunov_averaged → ξ collection)
3. **Eigenvector gap**: `tridiag_eigh` flat buffers are ready. When hotSpring's Sturm solver adds eigenvector support, groundSpring can delegate immediately
4. **Review Gen3 Paper 22-24 ODE requirements**: The Anderson-QS bridge will stress bistable/multisignal ODE integration at scale — barracuda's GPU ODE solvers need to handle parameter sweeps

---

*groundSpring V19 | February 26, 2026 | 15 experiments, 226 tests, 185/185 checks, 98.93% coverage*
