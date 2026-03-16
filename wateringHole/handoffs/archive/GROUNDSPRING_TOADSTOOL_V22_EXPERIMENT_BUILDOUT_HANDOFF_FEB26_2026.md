# groundSpring → ToadStool V22: Experiment Buildout + Absorption Candidates

**Date**: February 26, 2026
**From**: groundSpring
**To**: ToadStool / BarraCUDA team
**License**: AGPL-3.0-only
**Covers**: V22 (Exp 016-018 buildout, linting cleanup, barracuda evolution review)

---

## Executive Summary

- **3 new experiments built**: Exp 016 Rare Biosphere (R. Anderson 2015), Exp 017
  Quasispecies Threshold (Dolson 2023), Exp 018 Band Edge Structure
  (Filonov-Kachkovskiy 2018). All green: 211/211 Rust checks, 18/18 pytest, 262
  Rust tests, zero clippy warnings (all-targets, all-features), zero ruff warnings.
- **3 new library modules** with GPU-parallel structure: `rare_biosphere` (Chao1,
  detection power, multinomial sampling), `quasispecies` (Wright-Fisher + mutation,
  error threshold scan), `band_structure` (transfer matrix, tridiagonal eigenvalues).
- **Pre-existing clippy debt resolved**: 14 warnings across `almost_mathieu`,
  `anderson`, `transport`, `bistable`, `multisignal`, `ode`, `determinism.rs` —
  cfg gates for barracuda-gpu dead code, float_cmp in determinism tests, mul_add.
- **18/18 mathematical parity proven** (Python ⇌ Rust against shared benchmark JSONs).
- **Barracuda absorption candidates**: 3 new modules with embarrassingly parallel
  structure suitable for GPU dispatch.

---

## Part 1: New Experiments

### Exp 016: Rare Biosphere Signal Detection

**Paper**: Anderson, Sogin, Baross (2015) FEMS Microbiol Ecol 91:fiu016
**Domain**: Microbial ecology — when does a detected lineage represent real
biological signal vs. sequencing noise?

| Module | Function | GPU Opportunity |
|--------|----------|----------------|
| `rare_biosphere::chao1` | Richness estimation from frequency spectrum | Scalar — no GPU value |
| `rare_biosphere::detection_power` | P(detect) = 1 − (1−p)^D | Scalar arithmetic |
| `rare_biosphere::detection_threshold` | D* = ⌈ln(α)/ln(1−p)⌉ | Scalar arithmetic |
| `rare_biosphere::abundance_occupancy` | Multinomial → detection rates across replicates | **Parallel**: independent replicates × species |
| `rare_biosphere::singleton_fraction` | f₁/S_obs from multinomial samples | Uses `rarefaction::multinomial_sample` |
| `rare_biosphere::tier_detection_rate` | Detection rate per abundance tier | **Parallel**: independent trials |

**toadStool action**: `abundance_occupancy` and `tier_detection_rate` are
embarrassingly parallel across replicates. The existing `batched_multinomial`
shader (metalForge Tier C, already production-ready in `metalForge/shaders/`)
provides the core sampling primitive. Wrapping it with a detection-counting
reduction would complete the GPU path.

### Exp 017: Eco-Evolutionary Noise Threshold

**Paper**: Dolson, Banzhaf, Ofria (2023) J R Soc Interface 20(208)
**Domain**: Quasispecies theory — Eigen's error threshold predicts when
mutation rate exceeds the information carrying capacity of a genome.

| Module | Function | GPU Opportunity |
|--------|----------|----------------|
| `quasispecies::error_threshold` | μ_c = 1 − σ^(−1/L) | Scalar arithmetic |
| `quasispecies::master_frequency_analytical` | Closed-form master genotype fraction | Scalar arithmetic |
| `quasispecies::quasispecies_simulation` | Wright-Fisher + per-locus mutation | **Parallel**: population × loci |
| `quasispecies::mean_fitness` | Population-weighted fitness average | Reduction (sum) |

**toadStool action**: The simulation kernel is Wright-Fisher resampling
(multinomial) followed by independent per-locus Bernoulli mutation. This is
the same multinomial primitive already in barracuda (`batched_multinomial`),
plus a per-element Bernoulli step. The mutation scan across 12 rates is
trivially parallel. A `quasispecies_sweep` op would batch the entire
experiment into a single GPU dispatch.

### Exp 018: Band Edge Structure

**Paper**: Filonov & Kachkovskiy (2018) Acta Math 221:59-80
**Domain**: Spectral theory — band-gap structure of 1D periodic operators.

| Module | Function | GPU Opportunity |
|--------|----------|----------------|
| `band_structure::transfer_matrix_half_trace` | 2×2 matrix product chain → half-trace | Sequential per energy, **parallel across energies** |
| `band_structure::find_band_edges` | Sign-change scan of |τ|−1 | Sequential scan |
| `band_structure::count_bands` | Count connected |τ|≤1 intervals | Sequential scan |
| `band_structure::periodic_hamiltonian` | Build N×N tridiagonal Hamiltonian | Allocation only |
| `band_structure::eigenvalue_band_fraction` | Fraction of eigenvalues in band regions | Uses tridiag eigenvalues |

**toadStool action**: `transfer_matrix_half_trace` is a sequential matrix
product chain for a single energy, but the energy scan (10,001 points) is
embarrassingly parallel. Each energy point computes an independent 2×2 chain.
This maps directly to a `transfer_matrix_trace_batch` GPU kernel — one thread
per energy, each doing L sequential 2×2 multiplies. The tridiagonal
eigenvalue computation uses `barracuda::spectral::find_all_eigenvalues`
(already in barracuda via Sturm bisection from hotSpring S26).

---

## Part 2: Barracuda Evolution Review

### Current Delegation Inventory (27 active)

| Tier | Count | Modules |
|------|------:|---------|
| CPU (`barracuda` feature) | 22 | stats (15), bootstrap (2), anderson (1), bistable (1), multisignal (1), kinetics (1), rarefaction (1) |
| GPU (`barracuda-gpu` feature) | 5 | anderson (2: lyapunov_exponent, lyapunov_averaged), almost_mathieu (3: hamiltonian, eigenvalues, level_spacing_ratio) |
| **Total** | **27** | |

### New Modules — No Barracuda Delegation Yet

| Module | Functions | Delegation Candidates |
|--------|-----------|----------------------|
| `rare_biosphere` | 6 public functions | `abundance_occupancy` (multinomial batch), `tier_detection_rate` (parallel trials) |
| `quasispecies` | 4 public functions | `quasispecies_simulation` (multinomial + Bernoulli), mutation sweep (batch) |
| `band_structure` | 5 public functions | `transfer_matrix_half_trace` (energy-parallel batch), eigenvalue fraction (via Sturm) |

### Absorption Priority for New Modules

| Priority | Op | Module | Why |
|----------|----|--------|-----|
| **HIGH** | `batched_multinomial` absorption | `rare_biosphere`, `quasispecies` | Production WGSL already in metalForge/shaders/; used by Exp 004, 016, 017 |
| **MEDIUM** | `transfer_matrix_trace_batch` | `band_structure` | New kernel: one thread per energy, L sequential 2×2 multiplies |
| **LOW** | `quasispecies_sweep` | `quasispecies` | Batch entire mutation-rate sweep into single dispatch |

### Existing Gaps (unchanged from V21)

| Gap | Papers | Notes |
|-----|--------|-------|
| FFT (real, complex) | 6, 7 (Bazavov) | Not in barracuda; blocks spectral reconstruction |
| Grid search 3D dispatch | 5, 8 | New kernel needed |
| PRNG alignment (xoshiro) | All | Xorshift64 → xoshiro128** for bitwise GPU parity |
| Tridiag eigenvector solver | 12, 18 | Eigenvalues via Sturm; eigenvectors CPU-only |

---

## Part 3: Learnings for ToadStool Evolution

### 1. cfg-gate Dead Code Properly

When barracuda-gpu delegates a public function, the CPU fallback helper
becomes dead code under `--all-features`. We fixed 11 functions across
`almost_mathieu.rs` and `anderson.rs` by adding
`#[cfg(not(feature = "barracuda-gpu"))]` to the `_cpu` helpers. Without
this, `cargo clippy --all-features -D warnings` fails.

**Pattern to follow:**

```rust
pub fn foo() -> T {
    #[cfg(feature = "barracuda-gpu")]
    { barracuda::foo() }  // expression, not `return`
    #[cfg(not(feature = "barracuda-gpu"))]
    foo_cpu()
}

#[cfg(not(feature = "barracuda-gpu"))]
fn foo_cpu() -> T { /* ... */ }
```

### 2. `#[expect]` vs `#[allow]` for Test Lints

`#[expect(clippy::float_cmp)]` on a test that compares `Vec<f64>` (not bare
`f64`) produces `unfulfilled_lint_expectations` because the lint doesn't fire
on Vec comparisons. Use `#[allow]` for module-level test files, or apply
`#[expect]` only where the lint actually fires (bare f64 comparisons).

### 3. Numerical Patterns Worth Absorbing

| Pattern | Module | Description |
|---------|--------|-------------|
| `f64::log(base)` | `rare_biosphere` | Clearer than `ln(x)/ln(base)` for detection threshold |
| `f64::midpoint` | `band_structure` | Overflow-safe `(a+b)/2` for transfer matrix half-trace |
| `mul_add` chains | `band_structure`, `quasispecies` | FMA for transfer matrix and fitness computation |
| Log-exp form | `rare_biosphere::detection_power` | `1 - exp(D * ln(1-p))` avoids `(1-p)^D` overflow |

### 4. Test Determinism

All 3 new experiments include explicit determinism checks (same seed →
identical output). The Xorshift64 PRNG ensures bitwise reproducibility
within Rust. Python uses PCG64 — results differ between languages but
statistical properties agree.

---

## Part 4: Paper Queue Status + BarraCUDA Tier Plan

### Completed Papers (18 experiments)

| Paper # | Experiment | CPU | GPU Tier | metalForge | Barracuda Status |
|---------|-----------|:---:|:--------:|:----------:|-----------------|
| 1-5 | Sensor, Weather, FAO-56, Sequencing, Seismic | ✓ | A/B/C | After GPU | 22 CPU delegated |
| 9-11 | Signal Specificity, Bistable, Multi-Signal | ✓ | GPU-ready | After GPU | ODE + Gillespie |
| 12-13 | RAWR, Resampling Convergence | ✓ | Parallel | After GPU | bootstrap delegated |
| 14 | **NEW** Dolson Quasispecies | ✓ | Parallel | After GPU | Multinomial candidate |
| 15-17 | Anderson, Quasiperiodic, Spin Chain | ✓ | GPU delegated | After GPU | spectral + Sturm |
| 18 | **NEW** Filonov-Kachkovskiy Band Edge | ✓ | Energy-parallel | After GPU | tridiag candidate |
| 20 | Drift vs Selection | ✓ | Parallel | After GPU | Wright-Fisher candidate |
| 21 | **NEW** R. Anderson Rare Biosphere | ✓ | Parallel | After GPU | Multinomial candidate |

### Remaining Queued Papers

| Paper # | Paper | Blocker |
|---------|-------|---------|
| 6-8 | Bazavov (spectral reconstruction, g-2, freeze-out) | FFT gap |
| 19 | R. Anderson (2021) mSystems review | Reference only |
| 22-24 | Cross-spring sub-thesis 06 (soil-Anderson) | Depends on Exp 001-004 GPU |
| 25-27 | Sub-thesis 07 (WDM GPU precision) | Depends on hotSpring WDM |

### BarraCUDA Evolution Tiers

```
Tier 1: BarraCUDA CPU (COMPLETE — 211/211 PASS, 27 delegations)
  Pure safe Rust with optional barracuda feature gate.
  21 library modules, 262 tests, 18/18 parity.

Tier 2: BarraCUDA GPU (NEXT)
  Priority A: Wire existing barracuda GPU ops (stats, spectral)
  Priority B: Absorb batched_multinomial (Exp 004, 016, 017)
  Priority C: New kernels (transfer_matrix_trace_batch for Exp 018)

Tier 3: metalForge Cross-Substrate (AFTER GPU)
  CPU ↔ GPU parity validation for all 18 experiments.
  Mixed dispatch: metalForge routes to best substrate per operation.
```

---

## Action Items

1. **toadStool action**: Absorb `batched_multinomial.wgsl` from
   `metalForge/shaders/` — used by Exp 004, 016, 017 (3 experiments depend
   on this primitive for GPU tier). Production WGSL, 112 lines, documented
   bindings and dispatch geometry.

2. **toadStool action**: Consider `transfer_matrix_trace_batch` kernel for
   Exp 018 — one thread per energy point, L sequential 2×2 multiplies.
   10,001 energy points × L=2-3 period → ~20-30k FMAs per thread, highly
   occupancy-friendly.

3. **toadStool action**: Review `quasispecies_sweep` opportunity —
   batching 12 mutation rates × 500 generations × 10k population into a
   single GPU dispatch. This is the same multinomial + Bernoulli pattern
   as Exp 004 rarefaction.

4. **toadStool action**: `tridiag_eigh` eigenvector solver (not just
   eigenvalues) would benefit Exp 012 and 018. Currently CPU-only; Sturm
   bisection gives eigenvalues but not eigenvectors.

5. **groundSpring action**: Wire `rare_biosphere::chao1` to
   `barracuda::stats::diversity::chao1` once V22 handoff is consumed.

6. **groundSpring action**: Wire `quasispecies::quasispecies_simulation`
   multinomial step to barracuda once `batched_multinomial` is absorbed.

---

## Pin History

| Version | ToadStool Pin | Session | Changes |
|---------|:------------:|---------|---------|
| V22 | `f0feb226` (S68) | V22 | 3 new experiments, 3 new modules, linting cleanup, barracuda evolution review |
| V21 | `f0feb226` (S68) | V21 | Complete barracuda rewiring, dual-mode CI, 27 delegations |
| V20 | `f0feb226` (S68) | V20 | Hill delegation #27, S68 catch-up |
