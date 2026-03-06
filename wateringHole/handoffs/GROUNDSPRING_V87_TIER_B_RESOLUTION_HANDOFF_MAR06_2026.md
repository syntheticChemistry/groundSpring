# groundSpring V87 — Tier B Resolution & Cross-Spring Delegation Completion Handoff

**Date**: March 6, 2026
**From**: groundSpring V87
**To**: toadStool team, barraCuda team, coralReef team
**Pins**: barraCuda `e1184f3`, toadStool S96c (`d77fc546`), coralReef `849fedd`

---

## Executive Summary

groundSpring V87 resolves all remaining Tier B evolution candidates. Two new CPU
delegations were wired (`multinomial_sample`, `anderson_potential`), five stale
Tier B entries were corrected (already wired in earlier versions), and two modules
were formally documented as CPU-by-design (`quasispecies_simulation`,
`band_structure` coarse scan).

**Result**: 93 active delegations (56 CPU + 37 GPU), 0 evolution candidates remaining.
All 804+ tests pass in both default and barracuda feature modes. Zero clippy warnings
(pedantic) in both modes.

---

## 2. What Changed in groundSpring

### 2.1 New CPU Delegations

| # | Function | barraCuda Target | Cross-Spring Origin |
|---|----------|-----------------|---------------------|
| 55 | `rarefaction::multinomial_sample` | `ops::bio::multinomial_sample_cpu` | wetSpring S15 → groundSpring V62 → barraCuda S93 |
| 56 | `anderson::anderson_potential` | `spectral::anderson_potential` | hotSpring S26 → barraCuda spectral |

**`multinomial_sample` wiring**:
- barraCuda's `multinomial_sample_cpu` expects cumulative probabilities + closure RNG
- Adapter: groundSpring's existing `abundances_to_cumulative()` normalizes raw abundances
- RNG: passes `Xorshift64::next_f64` via closure — PRNG stream preserved
- The batch path (`multinomial_sample_batch`) was already wired to `BatchedMultinomialGpu`

**`anderson_potential` wiring**:
- barraCuda's implementation uses `LcgRng`; groundSpring fallback uses `Xorshift64`
- PRNG values differ across feature gates — documented as expected
- Statistical properties are identical (both uniform distributions on [-W/2, W/2])
- Downstream tests verify statistical properties (γ > 0, ξ decreases with W), not exact values

### 2.2 Stale Tier B Entries Resolved

Five modules previously listed as "Tier B not delegated" were found to be already wired:

| Module | Actual Status | When Wired |
|--------|--------------|------------|
| `freeze_out::grid_fit_2d` | WIRED → `barracuda::ops::grid::grid_search_3d` + L-BFGS | V68 |
| `seismic::grid_search_inversion` | WIRED → `barracuda::ops::grid::grid_search_3d` | V42+ |
| `rare_biosphere::abundance_occupancy` | WIRED → `BatchedMultinomialGpu` | V42 |
| `rare_biosphere::tier_detection_rate` | WIRED → `BatchedMultinomialGpu` | V42 |
| `gillespie::birth_death_ssa` | batch path WIRED → `GillespieGpu` | V63 |

### 2.3 CPU-by-Design Decisions

Two modules were formally documented as CPU-by-design (not delegation failures):

**`quasispecies::quasispecies_simulation`**: Single-locus model with per-generation
mutation thinning. WrightFisherGpu handles selection+drift, but the mutation step
(binomial thinning by Q = (1-μ)^L) requires a GPU→CPU round-trip per generation.
For O(N) scalar binomial draws, GPU dispatch overhead (~0.1ms) dominates actual
compute (~0.001ms). This mirrors `birth_death_ssa` (single trajectory stays CPU).
A `quasispecies_simulation_batch` function was added for multiple independent
replicates.

**`band_structure::find_band_edges` coarse scan**: Evaluates `|Tr(T(E))/2|` at
n_points energies. Each point performs L sequential 2×2 matrix multiplications —
data-dependent chains not expressible in current barraCuda ops. For typical
periods (L=2-10) at 2000 points, this is ~20K multiplications — well below GPU
dispatch threshold. The Brent refinement (airSpring V035 → barraCuda S71+++) IS
delegated and dominates accuracy improvement.

---

## 3. Validation Results

### 3.1 Test Suite

| Mode | Tests | Status |
|------|-------|--------|
| Default (no barracuda) | 804+ | ALL PASS |
| barracuda (CPU delegation) | 804+ | ALL PASS |
| clippy --pedantic (default) | — | 0 warnings |
| clippy --pedantic (barracuda) | — | 0 warnings |
| cargo fmt --check | — | Clean |

### 3.2 Delegation Count

| Tier | V86 | V87 | Delta |
|------|-----|-----|-------|
| CPU active | 54 | 56 | +2 (multinomial_sample, anderson_potential) |
| GPU active | 37 | 37 | — |
| CPU by design | — | 2 | +2 (quasispecies, band_structure coarse) |
| Evolution candidates | 1 | 0 | −1 (resolved) |
| **Total active** | **91** | **93** | **+2** |

---

## 4. Cross-Spring Evolution Narratives

### 4.1 `multinomial_sample` Round-Trip

The most complete cross-spring journey:
1. **groundSpring V62**: `batched_multinomial_f64.wgsl` written for rare biosphere
2. **toadStool absorption**: Shader absorbed into compute primal
3. **barraCuda S93**: `BatchedMultinomialGpu` + `multinomial_sample_cpu` live
4. **groundSpring V87**: Delegates BACK to `barracuda::ops::bio::multinomial_sample_cpu`

This demonstrates the Write → Absorb → Lean cycle completing a full loop.

### 4.2 Anderson Potential (hotSpring Precision Lineage)

Anderson localization originated in hotSpring nuclear physics (S26). The potential
generation function (`anderson_potential`) is now delegated to barraCuda's spectral
module, which uses `LcgRng` (vs groundSpring's `Xorshift64`). The PRNG difference is
intentional — barraCuda aligns with Kokkos baselines while groundSpring aligns with
Python baselines. Both produce identical statistical distributions.

### 4.3 Bidirectional Provenance Summary

| Direction | Example | Impact |
|-----------|---------|--------|
| hotSpring → ALL | DF64 core-streaming (S58) | f64 precision on consumer GPUs |
| wetSpring ↔ groundSpring | Shannon diversity, multinomial sampling | Ecology ↔ metagenomics cross-validation |
| airSpring → groundSpring | Brent root-finding, L-BFGS | Hydrology methods → physics band edges |
| neuralSpring → ALL | `pow_f64` polyfill, dispatch pattern | Ada Lovelace unblock, GPU wiring blueprint |
| groundSpring → ALL | `if let Ok` fallback, tolerance tiers | Delegation standard for all Springs |

---

## 5. Remaining Work

### 5.1 No Delegation Gaps

Tier B is fully resolved. No modules require wiring.

### 5.2 GPU Pipeline (Pre-existing)

GPU reduce operations (sum, variance, bootstrap) return 0 on consumer hardware
(RTX 4070) due to a deeper issue in the `compile_shader_f64` → sovereign/SPIR-V
pipeline. This is documented in V86 and remains a barraCuda/coralReef investigation
item — not a groundSpring delegation issue.

### 5.3 PRNG Alignment (Phase 2b)

groundSpring uses `Xorshift64` for CPU reference, `Xoshiro128StarStar` for GPU
alignment. barraCuda uses `LcgRng` for `anderson_potential`. Full PRNG alignment
(Phase 2b) remains future work — requires regenerating Python baselines with
compatible PRNG.

---

## 6. Files Changed

| File | Change |
|------|--------|
| `crates/groundspring/src/rarefaction.rs` | `multinomial_sample` CPU delegation via `barracuda::ops::bio::multinomial_sample_cpu` |
| `crates/groundspring/src/anderson.rs` | `anderson_potential` CPU delegation via `barracuda::spectral::anderson_potential` |
| `crates/groundspring/src/quasispecies.rs` | CPU-by-design documented; `quasispecies_simulation_batch` added |
| `crates/groundspring/src/band_structure.rs` | Delegation rationale updated (coarse scan CPU-by-design) |
| `specs/BARRACUDA_EVOLUTION.md` | Tier B table resolved; 93 delegations; V87 entries |
| `wateringHole/CROSS_SPRING_SHADER_EVOLUTION.md` | Bidirectional provenance narratives; V87 timeline; delegation lineage #44-45 |
