# groundSpring V118 Handoff — Deep Audit Execution: RPC Expansion + Proptest + PRNG Alignment + GPU Delegation

**Date**: March 19, 2026
**From**: groundSpring V118
**To**: barraCuda team, toadStool team, coralReef team, ecosystem
**License**: AGPL-3.0-or-later
**Supersedes**: GROUNDSPRING_V117_ALLFEATURES_DENY_PRNG_HANDOFF_MAR18_2026.md (archived)
**Pins**: barraCuda v0.3.5, toadStool S158+, coralReef Iteration 55+

---

## Executive Summary

V118 is the result of a comprehensive deep audit execution. The dispatch surface
doubled from 8 to 16 JSON-RPC measurement methods. Property-based testing covers
5 mathematical domains (16 new proptest invariants, 30 total). Production PRNG
calls migrated to `DefaultRng` for zero-code-change xoshiro alignment. The
`spectral_recon::forward_correlator` now delegates to barraCuda GEMM for GPU
acceleration. Delegation count rises from 102 (61+41) to 110 (67+43).

**Key metrics:** 960+ tests, 30 proptest invariants, 16 JSON-RPC capabilities,
110 delegations (67 CPU + 43 GPU), zero clippy (pedantic+nursery), zero unsafe,
zero mocks in production, zero TODO/FIXME, all files < 1000 LOC.

---

## Part 1: What V118 Changed

### 1a. RPC Method Expansion (8 → 16 capabilities)

| New Method | Module | Parameters | Output |
|------------|--------|-----------|--------|
| `measurement.bootstrap` | `bootstrap` | `data`, `n_bootstrap`, `seed` | mean CI (lower, upper, width) |
| `measurement.rarefaction` | `rarefaction` | `counts`, `depth`, `seed` | Shannon, Simpson, Bray-Curtis, evenness |
| `measurement.drift` | `drift` | `n`, `p`, `generations`, `seed` | fixation probability, Kimura analytical |
| `measurement.band_edge` | `band_structure` | `n_sites`, `potential_period`, `contrast` | edges array, band count, gap width |
| `measurement.rare_biosphere` | `rare_biosphere` | `abundances`, `depth`, `seed` | Chao1, detection threshold, power |
| `measurement.gillespie` | `gillespie` | `species`, `reactions`, `stoichiometry`, `rates`, `t_max`, `n_runs`, `seed` | batch mean, variance, individual trajectories |
| `measurement.bistable` | `bistable` | `k_f`, `k_r`, `alpha`, `dt`, `t_max` | trajectory, steady state |
| `measurement.quasispecies` | `quasispecies` | `genome_length`, `mutation_rate`, `pop_size`, `seed` | master frequency, error threshold |

`niche.rs` expanded: `CAPABILITIES` (16), `SEMANTIC_MAPPINGS` (16),
`COST_ESTIMATES` (16), `OPERATION_DEPENDENCIES` (16). Refactored from
`const fn` to `pub static` arrays for `clippy::too_many_lines` compliance,
with backward-compatible `const fn` wrappers.

### 1b. Proptest Coverage (16 new, 30 total)

| Domain | Property | Invariant |
|--------|----------|-----------|
| `drift` | Kimura fixation | p_fix ∈ [0, 1] for all valid (N, p) |
| `drift` | Neutral fixation | p_fix ≈ 1/N when s = 0 |
| `fao56` | ET₀ positivity | ET₀ > 0 for valid meteorological inputs |
| `fao56` | ET₀ bounded | ET₀ < 15 mm/day (FAO-56 plausibility) |
| `fao56` | ET₀ monotonic in radiation | ∂ET₀/∂Rₛ > 0 |
| `spectral_recon` | Tikhonov round-trip | ‖Kρ − G‖ < tol::RECONSTRUCTION |
| `quasispecies` | Error threshold convergence | µ_c → 1/L for large genome |
| `band_structure` | Edge count parity | edges.len().is_multiple_of(2) |

### 1c. PRNG Alignment — Xorshift64 → DefaultRng

14 production files migrated from `Xorshift64::new(seed)` to
`DefaultRng::new(seed)`. `Xorshift64` retained in `#[cfg(test)]` modules
for deterministic baseline preservation. When `prng-xoshiro-default` feature
is enabled, all production code silently switches to `Xoshiro128StarStar`.

**Migration path:**
1. Enable `prng-xoshiro-default` feature
2. Regenerate Python baselines with xoshiro128**
3. Update benchmark JSONs
4. Archive xorshift64 baselines

### 1d. GPU Delegation — spectral_recon forward_correlator

`spectral_recon::forward_correlator` now dispatches to
`barracuda::ops::linalg::GemmF64::execute_gemm_ex` when `barracuda-gpu` is
enabled. CPU fallback via `forward_correlator_cpu` retained. This eliminates
the last manual matrix multiplication in the spectral reconstruction pipeline.

### 1e. Hardcoded → Named + Provenance

| File | Before | After |
|------|--------|-------|
| `bootstrap.rs` tests | `1e-12` | `crate::tol::EXACT` |
| `rawr.rs` tests | `1e-12` | `crate::tol::EXACT` |
| `validate_seismic.rs` | bare distance/depth/vp | inline provenance comments |
| `validate_fao56.rs` | bare wind height/daylight | inline provenance comments |
| `tolerances.py` | missing SSA floor | `EPS_SSA_FLOOR = 1e-15` |
| CI Python coverage | 80% | 90% |

---

## Part 2: Absorption Candidates for barraCuda / toadStool

### P1 — Immediate Absorption

| Source | Target | Benefit |
|--------|--------|---------|
| `spectral_recon::forward_correlator` | Already delegates to `GemmF64` | Validates GEMM dispatch for lattice correlator workloads |
| `dispatch.rs` 8 new method bodies | Reference implementation | Each method body is a clean delegation example for any spring |

### P1 — Open Requests (Unchanged from V117)

| Priority | Request | Context |
|----------|---------|---------|
| P1 | Tridiag eigenvectors | `transport.rs` — local QL retained |
| P1 | GPU FFT (real + complex) | `spectral_recon` — CPU DFT retained |
| P2 | PRNG alignment (xoshiro128** CPU) | Feature-gated; needs Python baselines |
| P2 | Parallel 3D grid dispatch | seismic.rs, freeze_out.rs |
| P3 | Unified ComputeScheduler | metalForge routes manually |
| P3 | `erfc` large-x stability | hotSpring Exp 046 |

### P2 — New Observations for barraCuda Team

| Observation | Detail |
|-------------|--------|
| GEMM dispatch overhead | `forward_correlator_gpu` adds ~2ms for small (16×16) matrices — batch path would amortize |
| Proptest reveals edge cases | `band_structure` edge count assertion caught a subtle off-by-one in period-1 chains — barraCuda's `band_edges_parallel` should add the same invariant test |
| DefaultRng migration smooth | 14 files, zero behavioral change — validates the `DefaultRng` abstraction design |

---

## Part 3: Cross-Spring Impact

### Delegations by Origin (110 total)

| Origin Spring | CPU | GPU | Total | Examples |
|--------------|-----|-----|-------|----------|
| hotSpring | 8 | 12 | 20 | DF64 core, Sturm tridiag, stress_virial |
| wetSpring | 6 | 8 | 14 | Gillespie, diversity fusion, Bray-Curtis |
| neuralSpring | 5 | 3 | 8 | chi², KL divergence, matrix correlation |
| airSpring | 7 | 2 | 9 | seasonal pipeline, Hargreaves, Makkink |
| groundSpring | 12 | 6 | 18 | Anderson Lyapunov, uncertainty prop |
| Shared/core | 29 | 12 | 41 | stats, linalg, bootstrap, fao56 |
| **Total** | **67** | **43** | **110** | |

### New Capability Surface (V118)

Any primal or biomeOS graph can now call 16 groundSpring measurement methods
via JSON-RPC. This enables:
- wetSpring: `measurement.rarefaction` + `measurement.rare_biosphere` for 16S validation
- healthSpring: `measurement.gillespie` + `measurement.bistable` for stochastic models
- airSpring: `measurement.bootstrap` for sensor CI estimation
- neuralSpring: `measurement.drift` + `measurement.quasispecies` for evolutionary dynamics

---

## Part 4: Ecosystem Learnings

### 4a. RPC Method Expansion Pattern

Adding a new JSON-RPC method to a spring requires 4 touchpoints:
1. `dispatch.rs` — match arm + function body
2. `niche.rs` — `CAPABILITIES`, `SEMANTIC_MAPPINGS`, `COST_ESTIMATES`, `OPERATION_DEPENDENCIES`
3. `dispatch.rs` — unit test
4. Handoff — document for ecosystem

The `pub static` array pattern (V118 refactor) scales better than `const fn`
for large niche definitions — avoids clippy `too_many_lines` while maintaining
compile-time initialization.

### 4b. Proptest for Mathematical Invariants

Property-based testing complements known-value unit tests. Unit tests verify
"does this input produce this output?" Proptests verify "does this function
satisfy this invariant for ALL valid inputs?" V118 demonstrates the pattern
for 5 mathematical domains. Every spring should add proptest coverage for:
- Range constraints (probabilities ∈ [0, 1])
- Monotonicity (ET₀ increases with radiation)
- Symmetry (even-count band edges)
- Convergence (error threshold → 1/L)

### 4c. DefaultRng Migration is Non-Breaking

The `DefaultRng` abstraction (V28 design, V117 feature gate, V118 production
migration) proves that PRNG swaps can be zero-behavioral-change in Rust. The
pattern: type alias → feature gate → migrate call sites → enable feature →
regenerate baselines. Other springs can follow this exact path.

### 4d. dispatch.rs Parameter Name Alignment

V118 discovered that `niche.rs` `OperationDeps.required_inputs` and
`OperationDeps.optional_inputs` field names had drifted from the actual
JSON-RPC parameter names in `dispatch.rs`. The fix was systematic alignment.
**All springs should audit niche ↔ dispatch parity.**

---

## Part 5: Metrics

| Metric | V117 | V118 | Delta |
|--------|------|------|-------|
| Tests | 960+ | 960+ | +24 (proptest + dispatch) |
| Proptest invariants | 14 | 30 | +16 |
| JSON-RPC capabilities | 8 | 16 | +8 |
| Delegations (CPU) | 61 | 67 | +6 |
| Delegations (GPU) | 41 | 43 | +2 |
| Delegations (total) | 102 | 110 | +8 |
| Local WGSL | 0 | 0 | — |
| Unsafe blocks | 0 | 0 | — |
| Clippy warnings | 0 | 0 | — |
| `#[allow()]` | 0 | 0 | — |
| Files > 1000 LOC | 0 | 0 | — |
| Largest file | dispatch.rs 530 | dispatch.rs 823 | +293 (8 methods) |
| Production Xorshift64 | 14 files | 0 files | −14 (migrated to DefaultRng) |

---

**groundSpring V118 | 40 modules | 35 experiments | 960+ tests | 110 delegations | 16 capabilities | 30 proptests | AGPL-3.0-or-later**
