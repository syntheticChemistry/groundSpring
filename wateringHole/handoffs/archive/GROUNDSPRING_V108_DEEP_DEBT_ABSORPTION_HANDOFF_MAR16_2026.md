# groundSpring V108 — Deep Debt + Absorption Evolution Handoff

**Date**: March 16, 2026
**From**: groundSpring V108 (39 modules, 906 tests, 102 delegations)
**To**: barraCuda / toadStool / coralReef teams
**Authority**: wateringHole (ecoPrimals Core Standards)
**Supersedes**: GROUNDSPRING_V107_BARRACUDA_TOADSTOOL_ABSORPTION_HANDOFF_MAR16_2026.md
**Pins**: barraCuda v0.3.5, toadStool S155b, coralReef Iteration 49+
**License**: AGPL-3.0-or-later (SCYBORG Provenance Trio)

---

## Executive Summary

V108 is a debt-resolution and absorption-readiness sprint. No new science — this
version evolves code quality, delegation patterns, and ecosystem alignment to
prepare for the next round of barraCuda absorption.

Key changes:
1. **License corrected** to AGPL-3.0-or-later (SCYBORG Provenance Trio alignment)
2. **barraCuda WelfordState CPU delegation** — `std_dev` and `mean_and_std_dev`
   now delegate to `barracuda::stats::welford::WelfordState` when feature-enabled
3. **Tolerance centralization** — test assertions use `crate::tol::ANALYTICAL`
   instead of bare `1e-10` literals; geometry constants extracted with provenance
4. **Typed capability-based discovery** — `discover_by_capability`, `compute_execute`,
   `storage_put`, `storage_get` in `biomeos/interaction.rs`
5. **Python provenance enrichment** — all 29 benchmark JSONs now include
   `python_version` and `numpy_version` in their `_provenance` sections
6. **Result-based validation binaries** — `validate_band_edge.rs` refactored
   from `expect()`/`unwrap()` to structured `BenchResult` error handling

---

## Quality Gates (V108)

| Gate | Status |
|------|--------|
| `cargo test --workspace` | **906 passed, 0 failed** |
| `cargo clippy --workspace -D warnings -W pedantic` | **0 warnings** |
| `cargo fmt --all -- --check` | **0 diff** |
| `cargo doc --workspace -D warnings` | **0 warnings** |
| `pytest tests/test_baseline_integrity.py` | **261 passed** |
| License | **AGPL-3.0-or-later** (SCYBORG trio) |
| `unsafe` in application code | **forbidden** (`#![forbid(unsafe_code)]`) |
| `#![deny(clippy::expect_used, clippy::unwrap_used)]` | **enforced** (all 3 crate roots) |
| Bare numeric literals in production | **0** |
| Hardcoded primal name strings | **0** |
| TODO/FIXME/HACK in .rs | **0** |
| Files > 1000 LOC | **0** |
| Mocks in production | **0** |
| proptest property tests | **14** |

---

## Part 1: New barraCuda Delegation — WelfordState CPU Path

### What changed

`crates/groundspring/src/stats/metrics.rs` now delegates `std_dev` and
`mean_and_std_dev` to `barracuda::stats::welford::WelfordState` when
`#[cfg(feature = "barracuda")]` is enabled:

```rust
#[cfg(feature = "barracuda")]
{
    let state = barracuda::stats::welford::WelfordState::from_slice(values);
    state.population_std_dev()
}
```

The local `welford_population` function remains as the CPU reference fallback
when barracuda is not feature-enabled.

### Why this matters for barraCuda

groundSpring previously used `welford_population` for all CPU stats even when
barraCuda was available. This delegation ensures:
- Single implementation for Welford statistics across the ecosystem
- Consistent numerical behavior (same algorithm, same precision)
- Centralized optimization benefits all springs

### barraCuda action

Verify that `WelfordState::from_slice` and `population_std_dev()` remain stable
API. groundSpring now depends on these at 6+ call sites (through `std_dev` and
`mean_and_std_dev`).

---

## Part 2: Tolerance Centralization

### Tests evolved

All `tissue_anderson/compartments.rs` test assertions migrated from bare
`1e-10` to `crate::tol::ANALYTICAL`:
- `healthy_epidermis_composition_sums_to_one`
- `inflamed_dermis_composition_sums_to_one`
- `disrupted_at_zero_matches_healthy`
- `disrupted_at_one_is_3d`
- `disrupted_clamped_above_one`

### Constants extracted with provenance

`tissue_anderson/geometry.rs` on-site energy values extracted from match arms
to named constants:

| Constant | Value | Provenance |
|----------|-------|------------|
| `TH2_ON_SITE_ENERGY` | 1.2 | Cytokine-mediated polarization (Paper 12) |
| `MAST_CELL_ON_SITE_ENERGY` | 1.0 | Tissue-resident sentinel baseline |
| `NEURON_ON_SITE_ENERGY` | 0.4 | Neural integration potential |
| `EOSINOPHIL_ON_SITE_ENERGY` | - | Tissue damage mediator |
| `FIBROBLAST_ON_SITE_ENERGY` | - | Structural matrix component |

### Validation binary evolution

`validate_band_edge.rs` (Exp 018) evolved:
- `serde_json::from_str(BENCHMARK).expect(...)` → `let Ok(bench) = ... else { return 1; };`
- `tridiag_eigh(...).expect(...)` → `if let Ok((eigenvalues, _)) = ... { ... } else { ... };`
- Determinism check: `f64::EPSILON` → `groundspring::tol::DETERMINISM`

### barraCuda action

Consider adopting the tolerance provenance pattern in `barracuda::tol` —
mathematical derivation, source citation, and validation binary reference
for every tolerance constant. groundSpring's 13-tier system is documented
in `crates/groundspring/src/lib.rs::tol`.

---

## Part 3: Typed Capability-Based Discovery

New public API in `biomeos/interaction.rs`:

| Function | Purpose | Discovery Target |
|----------|---------|-----------------|
| `discover_by_capability(cap)` | Find primal advertising a capability | Any primal |
| `compute_execute(params)` | Submit compute request | toadStool (typically) |
| `storage_put(key, value)` | Store data | NestGate (typically) |
| `storage_get(key)` | Retrieve data | NestGate (typically) |

All discovery is **runtime-only** via `capability.list` RPC queries to live
primal sockets. groundSpring has zero compile-time knowledge of which primal
provides any capability.

### toadStool action

If toadStool advertises `compute.execute` in its `capability.list` response,
groundSpring can now route compute requests to it automatically. Verify that:
1. toadStool's `capability.list` includes `compute.execute`
2. The `compute.execute` method accepts the standard params format
3. Response follows JSON-RPC 2.0 result convention

---

## Part 4: Python Provenance Enrichment

All 29 `benchmark_*.json` files now include:

```json
"_provenance": {
    "python_version": "3.11",
    "numpy_version": "1.24",
    "script": "...",
    "git_commit": "...",
    "date": "..."
}
```

This enables downstream consumers (including barraCuda's test harness) to
verify which Python/NumPy versions produced the reference values. Critical
for PRNG alignment and floating-point reproducibility.

---

## Part 5: Active Absorption Candidates (carried from V107)

### P0 — Should absorb (duplicates existing barraCuda ops)

| Local Function | File | barraCuda Equivalent | Status |
|----------------|------|----------------------|--------|
| `welford_population` | stats/metrics.rs | `WelfordState` | **Partially delegated (V108)** — CPU path wired; local fallback remains |
| `mat_transpose_mul` / `mat_transpose_vec` | spectral_recon.rs | `linalg::GemmF64` (batched) | Open — Aᵀ·B for Tikhonov |
| `wdm::vendor_parity_mean_variance` | wdm.rs | `SumReduceF64::mean` + `VarianceReduceF64` | Open |

### P1 — Could promote to GPU

| Local Function | Promotion Path | Status |
|----------------|----------------|--------|
| `bootstrap_median` / `bootstrap_std` | Extend `BootstrapMeanGpu` | Open |
| `moving_window_stats_cpu` | GPU sliding-window reduce | Open |
| `regression::fit_*` (all 5) | Batched GPU regression | Open |
| `freeze_out::chi2_decomposed_weighted` | Parallel per-datum chi² | Open |

---

## Part 6: Cross-Spring Learnings

### 1. SCYBORG Provenance Trio alignment

The ecosystem standard is AGPL-3.0-**or-later**, not `-only`. V108 corrected
302 source files, LICENSE text, and Cargo.toml. All springs should verify
their SPDX headers match `AGPL-3.0-or-later`.

### 2. Typed capability discovery as a pattern

groundSpring's `discover_by_capability` + typed wrappers (`compute_execute`,
`storage_put/get`) provide compile-time type safety over runtime discovery.
This pattern could be standardized across springs — each spring provides
typed wrappers for the capabilities it consumes.

**Recommendation for toadStool**: Consider publishing a `ecoprimals-ipc`
crate with typed discovery traits that springs implement.

### 3. WelfordState as the ecosystem standard for online stats

groundSpring now delegates to `barracuda::stats::welford::WelfordState`
for CPU mean/variance. This validates WelfordState as the canonical
single-pass statistics implementation. Other springs doing CPU stats
should delegate similarly.

### 4. Provenance enrichment pattern

Adding `python_version` and `numpy_version` to benchmark JSONs enables
automated drift detection. If a spring's baselines were generated with
NumPy 1.24 but someone reruns with 1.26, the PRNG stream may differ.
The provenance fields make this detectable.

### 5. Result-based validation binary pattern

groundSpring V108 demonstrates graceful degradation in validation binaries:
instead of panicking on parse failure, return exit code 1 with a diagnostic
message. This enables CI to collect failure reasons from all binaries rather
than stopping at the first panic.

---

## Delegation Map (unchanged from V107)

102 active delegations: 61 CPU + 41 GPU across 8 categories
(stats: 34, ops: 14, spectral: 16, linalg: 4, numerical: 3, optimize: 3,
special: 1, esn: 2). 22+ modules with GPU dispatch paths.

Full inventory in V107 absorption handoff (now archived).

---

## For barraCuda Team — Action Items

| Priority | Action | Context |
|----------|--------|---------|
| P0 | Verify `WelfordState` API stability | groundSpring now delegates at 6+ sites |
| P0 | Verify AGPL-3.0-or-later in barraCuda | SCYBORG trio alignment |
| P1 | Absorb `mat_transpose_mul`/`mat_transpose_vec` | Tikhonov inversion in spectral_recon |
| P1 | Consider `barracuda::tol` provenance pattern | 13-tier tolerance system with citations |
| P2 | Extend `BootstrapMeanGpu` for median/std | groundSpring has CPU-only bootstrap variants |
| P2 | GPU sliding-window reduce op | `moving_window_stats_cpu` promotion candidate |
| P2 | Implement direct `erfc` asymptotic expansion | ISSUE-006, cross-spring benefit |

## For toadStool Team — Action Items

| Priority | Action | Context |
|----------|--------|---------|
| P0 | Ensure `capability.list` includes `compute.execute` | groundSpring auto-routes to it |
| P1 | Define standard `NicheMetadata` struct | groundSpring provides `CostEstimate`, `OperationDeps` |
| P1 | Consider `ecoprimals-ipc` crate for typed discovery | Reusable across springs |
| P2 | Consider `ecoprimals-names` crate | Compile-time primal name constants |
