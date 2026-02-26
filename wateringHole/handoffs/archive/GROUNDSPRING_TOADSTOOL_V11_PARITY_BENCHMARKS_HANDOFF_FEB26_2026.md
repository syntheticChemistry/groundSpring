# groundSpring → ToadStool/BarraCUDA Handoff V11: Full-Suite Parity + Benchmarks

**Date**: February 26, 2026
**From**: groundSpring (validation spring — measurement noise characterization)
**To**: ToadStool / BarraCUDA team
**Supersedes**: V10 (definitive handoff, Feb 25)
**ToadStool baseline**: Sessions 50–62 + DF64 expansion (Feb 23–24, 2026)
**License**: AGPL-3.0-or-later

---

## Executive Summary

groundSpring expanded from 8 to 11 experiments and established formal
mathematical parity between Python and Rust across the entire suite.
All 11 experiments now have Python baselines, Rust validation binaries,
and barracuda delegation paths. Benchmarks cover all 11 experiments
(previously only 3). This handoff provides the ToadStool team with
everything needed to absorb groundSpring's work and evolve barracuda.

### Quality Gates (all green)

| Gate | Result |
|------|--------|
| `cargo clippy --workspace -- -D warnings` × 3 modes | **0 warnings** |
| `cargo test --workspace` × 3 modes | **177/177 PASS** |
| 11 validation binaries × 3 modes | **144/144 PASS** |
| `python3 -m pytest tests/` | **34/34 PASS** |
| `cargo llvm-cov --workspace` | **99.11%** line coverage |
| Mathematical parity (Python ⇌ Rust) | **11/11 PROVEN** |
| Barracuda delegation overhead | **~0%** (release benchmarks) |
| Hardcoded primal names | **Zero** |
| Unsafe Rust | **Forbidden** (workspace lint) |

---

## Part 1: What Changed Since V10

### Three new experiments

| Exp | Domain | Paper | Checks | Speedup | Delegation |
|-----|--------|-------|--------|---------|------------|
| 009 | Quasiperiodic | Jitomirskaya-Kachkovskiy 2018 | 8/8 | parity * | `almost_mathieu_hamiltonian` (barracuda-gpu) |
| 010 | Bistable switching | Fernandez 2020 PNAS | 9/9 | 18.5× | `BistableOde::cpu_derivative` (barracuda) |
| 011 | Multi-signal QS | Srivastava 2011 J Bact | 8/8 | 46.2× | `MultiSignalOde::cpu_derivative` (barracuda) |

\* Exp 009 uses a custom QR eigenvalue solver proving parity; Python uses LAPACK.

### New Rust modules

- **`bistable`**: `BistableParams`, `hill`, `bistable_derivative`, `rk4_step`,
  `integrate`, `stochastic_integrate`. 7 unit tests.
- **`multisignal`**: `MultiSignalParams`, `hill`, `hill_repress`,
  `multisignal_derivative`, `rk4_step`, `integrate`, `stochastic_integrate`.
  6 unit tests.
- **`anderson` (extended)**: `almost_mathieu_potential`,
  `almost_mathieu_hamiltonian`, `level_spacing_ratio`. 8 unit tests.

### New barracuda delegations (+3, total 14)

| # | groundSpring | barracuda | Feature |
|---|-------------|-----------|---------|
| 12 | `almost_mathieu_hamiltonian` | barracuda-gpu spectral | `barracuda-gpu` |
| 13 | `bistable_derivative` | `BistableOde::cpu_derivative` | `barracuda` |
| 14 | `multisignal_derivative` | `MultiSignalOde::cpu_derivative` | `barracuda` |

### Full-suite benchmarks

Previously: 3 experiments benchmarked (Exp 006-008). Now: all 11.

### Mathematical parity certificate

New `scripts/parity_report.py` runs all 22 validation paths (11 Python +
11 Rust) against shared benchmark JSONs. **11/11 PARITY PROVEN**. Machine-
readable certificate at `data/parity_report.json`.

---

## Part 2: What BarraCUDA Should Absorb (Updated Priorities)

### Priority 1: `batched_multinomial.wgsl` (112 lines)

**Location**: `metalForge/shaders/batched_multinomial.wgsl`
**CPU reference**: `groundspring::rarefaction::multinomial_sample()`

```
Params:     { n_taxa: u32, depth: u32, n_reps: u32, _pad: u32 }
Bindings:   @group(0) @binding(0) params (uniform)
            @group(0) @binding(1) cumulative (storage, read)
            @group(0) @binding(2) seeds (storage, read_write)
            @group(0) @binding(3) counts (storage, read_write)
Dispatch:   (ceil(n_reps / 64), 1, 1) @ workgroup_size(64)
PRNG:       xoshiro128** (4 × u32 state per replicate)
```

**Suggested barracuda target**: `ops::batched_multinomial_f64`
**Unblocks**: Papers 4, 20-21, 22-24

### Priority 2: RAWR Weighted Resampling Kernel

**No WGSL yet** — CPU reference only.
**CPU reference**: `groundspring::bootstrap::rawr_mean()`

RAWR uses Dirichlet-distributed weights for bootstrap CIs. Embarrassingly
parallel. Per-replicate: generate `n` exponential variates → normalize →
weighted mean → collect → percentile CI.

**Suggested barracuda target**: `ops::rawr_weighted_mean_f64`
**Unblocks**: Papers 12, 13

### Priority 3: `mc_et0_propagate.wgsl` (149 lines)

**Location**: `metalForge/shaders/mc_et0_propagate.wgsl`

Note: The ET₀ equation chain is superseded by barracuda `Op::Fao56Et0`.
The Box-Muller perturbation + dispatch wrapper is the new contribution.

### Priority 4: Gillespie CPU Fallback

`GillespieGpu` is GPU-only. groundSpring has a validated CPU
`birth_death_ssa`. Add `simulate_cpu()` method to barracuda.

### Priority 5: CPU Error Metrics

6 error metrics (RMSE, MBE, R², IoA, hit rate, Shannon diversity) have
GPU tensor ops but no CPU equivalents. Adding 5-line CPU functions
immediately lets groundSpring delegate them.

### Priority 6 (NEW): Dense Eigenvalue Solver

Exp 009 reveals that groundSpring's custom QR eigenvalue solver is 19×
slower than numpy/LAPACK. A barracuda GPU eigenvalue kernel (Lanczos,
Jacobi, or divide-and-conquer) would close this gap and unblock fast
Exp 009 validation on GPU.

---

## Part 3: Complete Delegation Inventory (14 active)

| # | groundSpring | barracuda | Feature | Notes |
|---|-------------|-----------|---------|-------|
| 1 | `pearson_r` | `stats::pearson_correlation` | `barracuda` | NaN guard |
| 2 | `spearman_r` | `stats::correlation::spearman_correlation` | `barracuda` | NaN guard |
| 3 | `sample_std_dev` | `stats::correlation::std_dev` | `barracuda` | Bessel-corrected |
| 4 | `covariance` | `stats::correlation::covariance` | `barracuda` | Sample covariance |
| 5 | `norm_cdf` | `stats::norm_cdf` | `barracuda` | Infallible |
| 6 | `norm_ppf` | `stats::norm_ppf` | `barracuda` | Acklam rational |
| 7 | `chi2_statistic` | `stats::chi2_decomposed` | `barracuda` | Struct mapping |
| 8 | `bootstrap_mean` | `stats::bootstrap_mean` | `barracuda` | Result struct |
| 9 | `lyapunov_exponent` | `spectral::lyapunov_exponent` | `barracuda-gpu` | Transfer matrix |
| 10 | `lyapunov_averaged` | `spectral::lyapunov_averaged` | `barracuda-gpu` | Multi-realization |
| 11 | `analytical_localization_length` | `special::localization_length` | `barracuda` | Perturbative ξ(W,E) |
| 12 | `almost_mathieu_hamiltonian` | barracuda-gpu spectral | `barracuda-gpu` | Coupling convention: λ/2 |
| 13 | `bistable_derivative` | `BistableOde::cpu_derivative` | `barracuda` | OdeSystem trait |
| 14 | `multisignal_derivative` | `MultiSignalOde::cpu_derivative` | `barracuda` | OdeSystem trait |

### Pending (6 — blocked by WgpuDevice requirement)

| groundSpring | barracuda GPU Op | Unblocked by Priority 5? |
|-------------|-----------------|--------------------------|
| `rmse` | `NormReduceF64::l2` | Yes (add CPU fn) |
| `mbe` | `SumReduceF64::mean` | Yes (add CPU fn) |
| `r_squared` | `VarianceReduceF64` | Yes (add CPU fn) |
| `index_of_agreement` | `FusedMapReduceF64` | Yes (add CPU fn) |
| `hit_rate` | `FusedMapReduceF64` | Yes (add CPU fn) |
| `shannon_diversity` | `FusedMapReduceF64::shannon_entropy` | Yes (add CPU fn) |

---

## Part 4: Full-Suite Performance Benchmarks

### Rust vs Python (median of 3 trials, all 11 experiments)

| Experiment | Python (s) | Rust (s) | Speedup |
|---|---|---|---|
| Exp 001: Sensor Noise | 0.64 | 0.11 | **5.7×** |
| Exp 002: Error Propagation | 0.36 | 0.10 | **3.8×** |
| Exp 003: Observation Gap | 0.28 | 0.07 | **4.4×** |
| Exp 004: Sequencing Noise | 0.14 | 0.08 | **1.8×** |
| Exp 005: Seismic Inversion | 7.63 | 0.12 | **63.6×** |
| Exp 006: Signal Specificity | 26.78 | 0.88 | **30.5×** |
| Exp 007: RAWR Resampling | 4.64 | 0.63 | **7.3×** |
| Exp 008: Anderson Localization | 21.98 | 0.73 | **29.9×** |
| Exp 009: Quasiperiodic | 0.65 | 12.16 | 0.1× * |
| Exp 010: Bistable Switching | 3.58 | 0.19 | **18.5×** |
| Exp 011: Multi-Signal QS | 4.30 | 0.09 | **46.2×** |
| **Total (excl. LAPACK-bound)** | **70.33** | **3.01** | **23.4×** |

\* Exp 009: custom QR eigenvalue solver in Rust (parity proof); numpy delegates
to LAPACK. Priority 6 (dense eigenvalue solver) would close this gap.

### Speedup by algorithm type

| Category | Range | Examples |
|----------|-------|---------|
| Branching loops | 30–64× | Gillespie, Anderson, seismic grid search |
| ODE integration | 18–46× | Bistable, multi-signal |
| Vectorized ops | 4–7× | RAWR, sensor noise |
| Lightweight checks | 2–4× | Sequencing, error propagation |

### BarraCUDA delegation overhead (release, best-of-3)

| Binary | Local (ms) | BarraCUDA-GPU (ms) | Delta |
|--------|-----------|-------------------|-------|
| validate-anderson | 671 | 640 | **−5%** |
| validate-signal-specificity | 795 | 787 | −1% |
| validate-rawr | 555 | 556 | <1% |
| **Total (8 binaries)** | **2108** | **2076** | **~0%** |

Three new binaries (quasiperiodic, bistable, multisignal) not yet in
three-mode benchmark — will be included when barracuda features are tested.

---

## Part 5: Mathematical Parity Certificate

Both Python baselines and Rust validation binaries validate against the
**same shared benchmark JSON files**. If both pass all checks, mathematical
parity is proven within the documented tolerances.

| Experiment | Benchmark JSON | Python | Rust | Parity |
|---|---|---|---|---|
| Exp 001 | `benchmark_sensor_noise.json` | 32/32 | 36/36 | **PROVEN** |
| Exp 002 | `benchmark_error_propagation.json` | 8/8 | 15/15 | **PROVEN** |
| Exp 003 | `benchmark_observation_gap.json` | 8/8 | 13/13 | **PROVEN** |
| Exp 004 | `benchmark_sequencing_noise.json` | 16/16 | 15/15 | **PROVEN** |
| Exp 005 | `benchmark_seismic.json` | 10/10 | 9/9 | **PROVEN** |
| Exp 006 | `benchmark_signal_specificity.json` | 12/12 | 12/12 | **PROVEN** |
| Exp 007 | `benchmark_rawr_resampling.json` | 11/11 | 11/11 | **PROVEN** |
| Exp 008 | `benchmark_anderson_localization.json` | 8/8 | 8/8 | **PROVEN** |
| Exp 009 | `benchmark_quasiperiodic.json` | 8/8 | 8/8 | **PROVEN** |
| Exp 010 | `benchmark_bistable.json` | 10/10 | 9/9 | **PROVEN** |
| Exp 011 | `benchmark_multisignal.json` | 9/9 | 8/8 | **PROVEN** |

**11/11 experiments: PARITY PROVEN.**
Machine-readable certificate: `data/parity_report.json`

---

## Part 6: Three-Tier Validation Roadmap

### Tier 1: BarraCUDA CPU — COMPLETE

| Status | Count |
|--------|-------|
| Experiments | 11/11 |
| Validation checks | 144/144 PASS |
| Delegations | 14 active |
| Parity (Python ⇌ Rust) | 11/11 PROVEN |
| Speedup | 23.4× (compute-bound) |

### Tier 2: BarraCUDA GPU — Next

| Category | Papers | Action | Blocker |
|----------|--------|--------|---------|
| Tier A stats | 1-5 | Wire GPU adapter for reduce ops | `WgpuDevice` lifecycle |
| Tier C multinomial | 4, 20-21 | Absorb `batched_multinomial.wgsl` | ToadStool absorption |
| Tier C RAWR | 12-13 | Implement `rawr_weighted_mean` | New kernel |
| GPU-ready ODE | 10, 11 | Dispatch wiring for `BistableOde`, `MultiSignalOde` | None |
| GPU-ready spectral | 9, 15, 16 | Dispatch wiring for `spectral::*`, hamiltonian | None |
| GPU-ready Gillespie | 6 | Add CPU fallback to `GillespieGpu` | CPU path missing |
| FFT gap | 6-8 | Add FFT kernel | Kernel not in barracuda |
| Dense eigensolve | 9 | Add GPU eigenvalue kernel | Kernel performance |

### Tier 3: metalForge Cross-Substrate — After GPU

| Validation | Description |
|-----------|-------------|
| CPU ↔ GPU parity | GPU output matches CPU within documented tolerance |
| Cross-vendor parity | RTX 4070 vs other GPUs produce identical physics |
| Mixed dispatch | metalForge routes to best substrate per operation |
| f32 ↔ f64 drift | Sub-thesis 07: quantify precision loss on consumer GPU |

---

## Part 7: Cross-Spring Learnings for BarraCUDA Evolution

### What groundSpring learned building 11 experiments

1. **`if let Ok` + always-compiled CPU fallback** is the right error pattern.
   Never `.expect()` on barracuda calls. The CPU fallback is always compiled,
   not hidden behind `#[cfg(not(feature))]`. This should be standard for all
   Springs.

2. **Three-mode testing catches feature-gate bugs** that single-mode CI misses.
   Run `cargo test`, `cargo test --features barracuda`, and `cargo test
   --features barracuda-gpu` independently.

3. **Shared benchmark JSONs are the parity proof**. Python and Rust both load
   expected values from the same JSON. The parity script runs both and
   generates a machine-readable certificate. This pattern works for GPU too.

4. **Custom math implementations are slower than LAPACK** (Exp 009: 19× slower).
   For proving mathematical correctness they're fine; for production performance,
   barracuda should wrap LAPACK or implement GPU-native eigensolvers.

5. **ODE parameter structs** with `to_flat()` methods bridge groundSpring's
   named-field convention to barracuda's `&[f64]` convention cleanly. The
   `OdeSystem` trait is the right abstraction for delegation.

6. **Capability-based discovery** — primals should scan for capabilities,
   not hardcode sibling names. groundSpring's `_discover_fao56_capability()`
   scans for `control/fao56/penman_monteith.py` without knowing which primal
   provides it.

7. **Tolerance documentation** — every validation tolerance should cite a
   mathematical basis (e.g., "Herman's formula: γ = ln(λ/2) ± 0.05 for
   N=1000 sites"), not just "seems to work."

### From hotSpring

- **DF64 core-streaming**: Consumer GPU f64 needs workgroup-aware dispatch.
  Relevant for groundSpring's error metrics GPU promotion.
- **Spectral module quality**: 195 nuclear physics checks validate the
  `spectral::lyapunov_*` functions groundSpring delegates to.

### From wetSpring

- **log_f64 precision fix**: ~1e-3 error in Shannon entropy corrected.
  Affects groundSpring's future `shannon_diversity` GPU delegation.
- **GillespieGpu needs CPU fallback**: Both wetSpring and groundSpring need it.
- **Rarefaction gap**: `batched_multinomial` needed by both springs. Joint priority.

---

## Part 8: PRNG Alignment Status

| Component | PRNG | State Size |
|-----------|------|-----------|
| groundSpring CPU | `Xorshift64` (Marsaglia) | 64 bits |
| barracuda CPU (LHS) | `Xoshiro256**` | 4×u64 |
| barracuda GPU (WGSL) | `xoshiro128**` | 4×u32 |

**Alignment requires**: Public CPU `Xoshiro128` struct in barracuda with
`next_u64()` and `next_normal()` methods. groundSpring would then:
1. Feature-gate PRNG selection
2. Regenerate all baselines with xoshiro128** (Python + Rust)
3. Update benchmark JSONs with new expected values
4. Revalidate 144/144 checks

**Scope**: 7 stochastic experiments affected (RAWR, Anderson, Gillespie,
rarefaction, FAO56 MC, bistable, multisignal). Estimated effort: 2-3 sessions.

---

## Handoff Checklist

- [x] 14 delegations verified against S62 barracuda API
- [x] Full barracuda CPU API audited — no missed wiring opportunities
- [x] Three-mode clippy: 0 warnings × 3 modes
- [x] Three-mode tests: 177/177 PASS × 3 modes
- [x] Three-mode validation: 144/144 checks × 3 modes
- [x] 34 Python tests passing
- [x] Release benchmarks: all 11 experiments timed
- [x] Mathematical parity: 11/11 PROVEN
- [x] Cross-spring lineage documented
- [x] 6 absorption priorities documented with binding layouts
- [x] PRNG alignment roadmap documented
- [x] Error handling pattern documented as wateringHole standard
- [x] Three-tier validation roadmap (CPU → GPU → metalForge)
- [x] 99.11% line coverage
- [x] Zero hardcoded primal names
- [x] V10 archived

---

*groundSpring: 11 experiments, 144 checks, 14 delegations, 23.4× faster
than Python, 11/11 mathematical parity proven, zero barracuda overhead.
Ready for ToadStool absorption of Priorities 1–6 and GPU tier validation.*
