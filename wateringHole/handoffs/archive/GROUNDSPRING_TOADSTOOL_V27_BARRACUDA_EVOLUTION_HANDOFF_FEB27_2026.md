# toadStool / barracuda — V27 Barracuda Evolution & Absorption Handoff

**Date:** February 27, 2026
**From:** groundSpring
**To:** toadStool / barracuda core team
**Covers:** V27 barracuda usage review + evolution learnings + paper controls (open data) + three-tier validation + absorption requests
**License:** AGPL-3.0-only

---

## Executive Summary

groundSpring V27 completes a full documentation and barracuda audit. The
spring is at its cleanest state: 0 clippy warnings (three modes), 0 TODOs
in production, 99.37% line coverage, 323 Rust tests, 288/288 validation
checks, 28/28 mathematical parity proven. This handoff documents:

1. **How groundSpring uses barracuda** — what we lean on, what's local, and why
2. **What to absorb next** — 2 WGSL shaders + 3 patterns ready for upstream
3. **Evolution discoveries** — delegation patterns, flat buffers, tolerance docs
4. **Paper validation controls** — confirming open data/systems for all 28 papers
5. **Hardware validation matrix** — CPU, GPU, and metalForge mixed-substrate results

---

## Part 1: How groundSpring Uses BarraCUDA (V27 State)

### Upstream Dependencies (lean — we use barracuda for this)

| barracuda Module | groundSpring Usage | Delegation Count |
|------------------|-------------------|:----------------:|
| `stats::pearson_correlation` | Correlation in sensor noise, obs gap | 1 |
| `stats::correlation::spearman_correlation` | Rank correlation (Exp 002) | 1 |
| `stats::correlation::std_dev` | Sample standard deviation | 1 |
| `stats::correlation::covariance` | Covariance for pearson_r denominator | 1 |
| `stats::norm_cdf` / `stats::norm_ppf` | Normal distribution Φ(x) and Φ⁻¹(p) | 2 |
| `stats::chi2_decomposed` | Chi-squared goodness-of-fit | 1 |
| `stats::bootstrap_mean` | Bootstrap confidence intervals | 1 |
| `stats::rawr_mean` | RAWR Dirichlet-weighted resampling | 1 |
| `stats::hill` | Hill kinetics for bistable/multisignal ODE | 1 |
| `stats::metrics::*` | rmse, mbe, r², IoA, hit_rate, mean, percentile | 7 |
| `stats::diversity::shannon` | Shannon diversity (rarefaction) | 1 |
| `stats::pielou_evenness` | Pielou evenness J' | 1 |
| `stats::regression::fit_linear` | Linear regression for finite-size extrapolation | 1 |
| `spectral::lyapunov_exponent` | Anderson transfer matrix method | 1 |
| `spectral::lyapunov_averaged` | Multi-realization Lyapunov average | 1 |
| `spectral::almost_mathieu_hamiltonian` | Quasiperiodic Hamiltonian construction | 1 |
| `spectral::find_all_eigenvalues` | Sturm tridiag eigenvalue solver (**47.7× Exp 009**) | 1 |
| `spectral::level_spacing_ratio` | Level spacing statistics for Anderson phases | 1 |
| `special::anderson_transport::localization_length` | Analytical ξ(W,E) perturbative formula | 1 |
| `numerical::ode_bio::BistableOde` | Bistable ODE derivative | 1 |
| `numerical::ode_bio::MultiSignalOde` | Multi-signal QS ODE derivative | 1 |
| `linalg::solve_f64_cpu` | Gauss–Jordan solve for Tikhonov regularization | 1 |
| **Total** | | **29** |

**Split**: 23 CPU delegated (`#[cfg(feature = "barracuda")]`), 6 GPU delegated
(`#[cfg(feature = "barracuda-gpu")]`).

### Local Implementation (groundSpring owns the science, barracuda owns the compute)

| Module | Why Local |
|--------|-----------|
| `decompose` | Two scalar ops (bias² = MBE², var = RMSE² − MBE²) — no GPU benefit |
| `seismic` | Haversine, travel time — scalar trig; grid search is Tier B candidate |
| `gillespie` | SSA loop inherently serial; `GillespieGpu` exists for batched dispatch |
| `transport` | Eigenvector solver (implicit QL) — only eigenvalues delegated |
| `drift` | Wright-Fisher fixation, Kimura probability — future batched candidates |
| `rare_biosphere` | Chao1, detection power — CPU-only, parallel candidates |
| `quasispecies` | Eigen error threshold — simulation-only |
| `band_structure` | Transfer matrix, band edge detection — energy scan parallelizable |
| `jackknife` | Leave-one-out — embarrassingly parallel, GPU candidate for large N |
| `freeze_out` | Grid search — embarrassingly parallel, GPU candidate |
| `prng` | Xorshift64 — awaiting PRNG alignment to xoshiro128** (Phase 2b) |
| `cast`, `validate` | Infrastructure — no GPU benefit |

---

## Part 2: What ToadStool Should Absorb

### Ready Now (production WGSL in metalForge/shaders/)

| Shader | Lines | Purpose | groundSpring Experiments |
|--------|:-----:|---------|------------------------|
| `batched_multinomial.wgsl` | 126 | GPU multinomial sampling (alias method) | Exp 004 (sequencing), Exp 016 (rare biosphere), Exp 023 (no-till) |
| `mc_et0_propagate.wgsl` | 135 | MC uncertainty propagation through FAO-56 | Exp 003 (error propagation), Exp 022 (ET₀→Anderson) |

The FAO-56 equation chain itself is already absorbed (`Op::Fao56Et0`). The
MC wrapper adds Box-Muller perturbation + workgroup dispatch — the missing
piece for GPU uncertainty propagation.

### Patterns Worth Absorbing

| Pattern | Source | Benefit |
|---------|--------|---------|
| `if let Ok` delegation with always-compiled CPU fallback | All 29 groundSpring delegations | Standard delegation pattern — prevents silent failures |
| Tolerance documentation (justification per check) | All 28 validation binaries | Machine-auditable tolerance chain |
| Three-mode benchmark harness (`three_mode_benchmark.sh`) | groundSpring CI | Proves feature-flag correctness across CPU-only, barracuda-CPU, barracuda-GPU |

### Future Absorption Candidates (when barracuda evolves)

| Module | barracuda Target | Why |
|--------|-----------------|-----|
| `jackknife::leave_one_out` | `stats::jackknife` (new) | N independent subsets — embarrassingly parallel |
| `freeze_out::grid_search` | `optimize::grid_search_2d` (new) | (T₀, κ₂) grid — embarrassingly parallel |
| `band_structure::energy_scan` | `spectral::transfer_matrix_scan` (new) | 10,001 energy points — one thread per energy |
| `transport::tridiag_eigh` (eigenvectors) | `linalg::eigh_tridiag_f64` (new) | Extends Sturm (eigenvalues-only) to full eigenvector recovery |

---

## Part 3: Evolution Discoveries & Learnings

### 3.1 Delegation Patterns

- **`if let Ok` is the universal delegation pattern.** 29 delegations all use
  `if let Ok(result) = barracuda::some_fn(args) { result } else { local_fn(args) }`.
  The V17 bug (covariance/pearson/spearman silently returning 0.0 on error)
  proved that `match + default(0.0)` masks failures. Always fall through to CPU.

- **`#[cfg]` / `#[cfg(not)]` mutual exclusion eliminates `unreachable_code`.**
  V20 replaced all 20 `#[allow(unreachable_code)]` with proper feature-flag
  gating. Both code paths compile, but only one is active per feature set.

- **Domain guards before delegation.** `kinetics::hill` preserves biological
  convention (x ≤ 0 → 0.0) before calling `barracuda::stats::hill`, which
  uses the mathematical convention (undefined for x < 0). The guard is cheap
  insurance against semantic drift.

### 3.2 Flat Buffers from Day One

The almost_mathieu and transport modules started with `Vec<Vec<f64>>` and
were refactored to flat `Vec<f64>` row-major layout for GPU promotability.
**Lesson**: Design all new modules with `&[f64]` + explicit dimensions from
the start to avoid a refactor step when GPU dispatch arrives.

### 3.3 Determinism Tests

13 bitwise determinism tests use `#[expect(clippy::float_cmp)]` for exact
equality. Any PRNG stream change, reduction reorder, or platform FP
difference fails loudly. barracuda should adopt this for stateful computations.

### 3.4 Provenance Chain

Every benchmark JSON has `_provenance` with `paper_doi`, `baseline_commit`,
`baseline_date`, `prng_algorithm`, `tolerance_basis`. This makes the full
chain machine-auditable: paper → Python → JSON → Rust → pass/fail.

### 3.5 metalForge Integration Learnings

- **Zero-mock NPU**: ToadStool's `akida-driver` (pure Rust) + groundspring-forge
  dispatch work end-to-end on real AKD1000 hardware. DMA round-trip ~51µs.
- **Cross-substrate dispatch**: `metalForge/forge/` routes to CPU/GPU/NPU based
  on capability discovery — no substrate hardcoding.
- **Validation binary pattern**: `validate-metalforge-*` binaries follow the
  same exit-0-pass/exit-1-fail pattern as all other validation binaries.

---

## Part 4: Paper Validation Controls — Open Data Audit

### Requirement

Every paper must use **open data and open systems** — no paywalled datasets,
no proprietary software dependencies.

### Audit Result: 28/28 PASS

| Papers | Data Source | Access | Status |
|--------|-----------|--------|:------:|
| 1–5 | Dong et al. 2020, FAO-56 Ex 18, synthetic | DOI / analytical | **Open** |
| 6–8 (Bazavov) | MILC/HotQCD lattice configs (ILDG), BNL open data | Public | **Open** |
| 9–11 (Waters) | PNAS supplementary (SI Tables, flow cytometry, qRT-PCR) | Open access | **Open** |
| 12–13 (Liu) | Simulation + TreeBASE/Dryad | Public repos | **Open** |
| 14 (Dolson) | Theoretical/simulation (MABE) | Open source | **Open** |
| 15–18 (Kachkovskiy) | Analytical (theorems — fully specified) | N/A | **Open** |
| 19 (R. Anderson review) | Review paper | Open access | **Open** |
| 20–21 (R. Anderson empirical) | NCBI SRA metagenomes/16S amplicons | SRA accession | **Open** |
| 22–24 (Sub-thesis 06) | Derived from Exp 001–004 | Internal/reproducible | **Open** |
| 25–27 (Sub-thesis 07 WDM) | Simulation + analytical | Reproducible | **Open** |
| 28 (NPU Anderson) | ToadStool akida-driver | Pure Rust, zero mocks | **Open** |

**Zero proprietary dependencies across all 28 papers.**

### Control Tiers

| Tier | What | How | Status |
|------|------|-----|:------:|
| Python baseline | Reproduce paper result | `python3 -m pytest tests/ -v` (28 experiments) | **PASS** |
| Rust validation | Match Python within tolerance | `cargo run --bin validate-*` (288/288) | **PASS** |
| barracuda CPU | Delegated result matches local | `cargo test --features barracuda` (323 tests) | **PASS** |
| barracuda GPU | GPU result matches CPU | `cargo test --features barracuda-gpu` (323 tests) | **PASS** |
| metalForge | Cross-substrate agreement | `cargo run --bin validate-metalforge-*` (31 checks) | **PASS** |

---

## Part 5: Three-Tier Hardware Validation Matrix

### Tier 1: BarraCUDA CPU (23 delegations)

Pure safe Rust with `#[cfg(feature = "barracuda")]` delegation. All 29
functions have always-compiled CPU fallbacks. Zero measurable overhead
(+1.7% from function indirection — functionally free).

| Metric | Value |
|--------|-------|
| Validation checks | 288/288 PASS |
| Rust tests | 323 |
| Mathematical parity | 28/28 PROVEN |
| Line coverage | 99.37% |
| Clippy warnings | 0 (three modes) |
| CPU delegation overhead | +1.7% total |

### Tier 2: BarraCUDA GPU (6 delegations)

GPU delegations via `#[cfg(feature = "barracuda-gpu")]`:

| Delegation | barracuda Op | Impact |
|-----------|-------------|--------|
| `lyapunov_exponent` | `spectral::lyapunov_exponent` | Transfer matrix on GPU |
| `lyapunov_averaged` | `spectral::lyapunov_averaged` | Multi-realization average |
| `almost_mathieu_hamiltonian` | `spectral::almost_mathieu_hamiltonian` | Quasiperiodic construction |
| `almost_mathieu_eigenvalues` | `spectral::find_all_eigenvalues` | **47.7× Exp 009** (Sturm tridiag) |
| `level_spacing_ratio` | `spectral::level_spacing_ratio` | Anderson phase classification |
| `tikhonov_solve` | `linalg::solve_f64_cpu` | Spectral function reconstruction |

**Benchmark**: 20.4s (CPU-only) → 9.2s (barracuda-gpu) = **2.2× overall**.
Exp 009 quasiperiodic: 11.7s → 0.244s = **47.7× speedup**.

### Tier 3: metalForge Cross-Substrate (live hardware)

| Substrate | Hardware | Validation |
|-----------|----------|-----------|
| CPU | Intel i9-12900K (16C/24T) | All 288 validation checks + 323 Rust tests |
| GPU (consumer) | NVIDIA RTX 4070 (12 GB VRAM) | groundspring-forge GPU inventory + dispatch |
| GPU (compute) | NVIDIA Titan V (12 GB HBM2) | Cross-vendor parity (Exp 027) |
| NPU | BrainChip AKD1000 (80 NPs, 10 MB SRAM) | Exp 028: Anderson regime classification, ~51µs DMA |

**metalForge validation binaries**:
- `validate-metalforge-inventory` — hardware discovery
- `validate-metalforge-gpu` — GPU dispatch validation
- `validate-metalforge-cross-substrate` — cross-substrate agreement

31 metalForge checks PASS across all substrates.

---

## Part 6: Barracuda Evolution Summary (V7 → V27)

| Version | Delegations | Tests | Checks | Key Milestone |
|:-------:|:-----------:|:-----:|:------:|---------------|
| V7 | 5 | 104 | 119 | First deep audit |
| V10 | 11 | 116 | 119 | First complete rewiring |
| V12 | 20 | 182 | 185 | S64 stats absorption wave |
| V13 | 24 | 225 | 185 | Sturm tridiag → 49.5× Exp 009 |
| V16 | 26 | 225 | 185 | rawr_mean (#26) |
| V20 | 27 | 225 | 185 | Hill kinetics (#27) |
| V21 | 27 | 225 | 225 | Dual-mode CI, 0 clippy×3 |
| V23 | 27 | 292 | 258 | Bazavov trio (Exp 019-021) |
| V25 | 27 | 302 | 279 | WDM experiments (Exp 025-027) |
| V26 | 29 | 314 | 288 | metalForge live hardware, NPU |
| **V27** | **29** | **323** | **288** | **Docs audit, paper controls, handoff** |

---

## Action Items for ToadStool

1. **Absorb `batched_multinomial.wgsl`** (126 lines) — enables GPU rarefaction
   for Exp 004, 016, 023. Production-ready in `metalForge/shaders/`.

2. **Absorb `mc_et0_propagate.wgsl`** (135 lines) — GPU MC uncertainty
   propagation through FAO-56. Wraps the already-absorbed `Op::Fao56Et0`.

3. **Consider `jackknife_leave_one_out` primitive** — embarrassingly parallel,
   N independent subsets. High GPU potential for large N (lattice QCD).

4. **Consider `grid_search_2d` primitive** — embarrassingly parallel (T₀, κ₂)
   grid for freeze-out inverse problems. Generalizes to any 2D grid search.

5. **Adopt `if let Ok` as standard delegation pattern** — proven across 29
   groundSpring delegations, prevents silent failure masking.

6. **Adopt determinism test pattern** — bitwise float comparison with
   `#[expect(clippy::float_cmp)]` for all stateful computations.

---

*groundSpring V27 — February 27, 2026*
