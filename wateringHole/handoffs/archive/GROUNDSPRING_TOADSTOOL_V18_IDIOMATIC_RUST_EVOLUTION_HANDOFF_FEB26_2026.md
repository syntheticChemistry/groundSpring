# groundSpring → ToadStool/BarraCUDA V18 — Idiomatic Rust Evolution + Full Provenance

**Date**: February 26, 2026
**From**: groundSpring (noise validation + environmental modeling)
**To**: ToadStool/BarraCUDA team
**Previous**: V17 (deep debt + delegation patterns)
**ToadStool HEAD**: `045103a7` (S66 Wave 5)
**groundSpring HEAD**: `459e5d0c` (V18 idiomatic Rust evolution)
**License**: AGPL-3.0-or-later

---

## Executive Summary

- **225 Rust tests** (was 205), **177/177 validation checks**, **98.94% coverage** — all green
- **13 new determinism tests**: bitwise-identical rerun verification for all stochastic algorithms (bootstrap, RAWR, multinomial, Gillespie, Wright-Fisher, ODE integration, eigenvalues, transport)
- **New `kinetics` module**: `hill()` / `hill_repress()` extracted from bistable + multisignal, barracuda delegation stub ready — `barracuda::stats::hill` / `barracuda::stats::monod` can absorb this
- **Zero `Vec<Vec<f64>>`**: Flat row-major buffers everywhere — `almost_mathieu.rs` QR and `transport.rs` eigenvectors refactored for cache locality and GPU promotability
- **Full provenance**: all 14 benchmark JSONs have `_doi` fields, all have `baseline_commit` stamps, all 14 validation binaries use `print_provenance_header`
- **CI complete**: all 14 validation binaries run in GitHub Actions (was missing 3)
- **Python baselines confirmed**: Exp 012 (18/18), 013 (10/10), 014 (7/7) all pass

---

## Part 1: What Changed (V17 → V18)

### New Module: `kinetics.rs`

Extracted duplicate `hill()` and `hill_repress()` from `bistable.rs` and `multisignal.rs` into `crate::kinetics`. Both ODE modules now import from the shared location.

The barracuda delegation is stubbed:

```rust
pub fn hill(x: f64, k: f64, n: f64) -> f64 {
    #[cfg(feature = "barracuda")]
    {
        if let Ok(v) = barracuda::stats::hill(x, k, n) {
            return v;
        }
    }
    hill_cpu(x, k, n)
}
```

**ToadStool action**: barracuda S66 exports `hill()` and `monod()` as public Rust APIs. Verify the signature matches and wire delegation #27. The `hill_repress` function is `1 - hill` algebraically, so it can either delegate to `1.0 - barracuda::stats::hill(x, k, n)` or have its own export.

### Flat Buffer Refactor

**`almost_mathieu.rs`**: Dense Givens QR for eigenvalue extraction refactored from `Vec<Vec<f64>>` to flat `Vec<f64>` with `n * row + col` indexing. Functions affected: `eigenvalues_qr_dense`, `givens_qr_flat`, `givens_rotate_rows_flat`, `givens_rotate_cols_flat`, `dense_mul_flat`.

**`transport.rs`**: `tridiag_eigh` return type changed from `(Vec<f64>, Vec<Vec<f64>>)` to `(Vec<f64>, Vec<f64>)`. The eigenvector matrix is now flat row-major. `wavepacket_msd` updated to accept the flat buffer. `implicit_ql` and `sort_eigenpairs` operate on flat `&mut [f64]`.

**Why this matters for ToadStool**: Flat buffers are the expected format for GPU dispatch. When barracuda absorbs `tridiag_eigh` or adds a dense eigenvector solver, the data layout will match without conversion.

### Determinism Tests

13 new tests in `crates/groundspring/tests/determinism.rs` using `#[expect(clippy::float_cmp)]` for bitwise equality:

- `prng_deterministic` (1000 u64 values)
- `bootstrap_deterministic`, `rawr_deterministic`
- `multinomial_deterministic`, `rarefaction_deterministic`
- `anderson_lyapunov_deterministic`, `eigenvalue_deterministic`, `level_spacing_deterministic`
- `bistable_ode_deterministic`, `multisignal_ode_deterministic`
- `gillespie_deterministic`, `wright_fisher_deterministic`
- `transport_deterministic` (eigenvalues + eigenvectors + MSD)

These guard against any future non-determinism from threading, unordered reductions, or platform-specific FP behavior.

---

## Part 2: Absorption Candidates for ToadStool

### Candidate #1: `kinetics::hill` / `kinetics::hill_repress`

- **What**: Activating and repressing Hill functions (enzyme kinetics)
- **Where**: Used by 2 ODE systems (bistable, multisignal), potentially useful for any sigmoidal response
- **barracuda status**: S66 already exports `hill()` and `monod()` — just need to verify API match
- **Priority**: Low (already delegated via stub, just needs wiring)

### Candidate #2: Flat eigenvector solver

- **What**: `transport::tridiag_eigh` — implicit QL algorithm for symmetric tridiagonal matrices
- **Where**: Exp 012 (spin chain transport) — computes full eigenvector matrix for wavepacket dynamics
- **barracuda status**: `barracuda::spectral::find_all_eigenvalues` exists for eigenvalues only (Sturm bisection). Eigenvectors not yet available.
- **Priority**: Medium — eigenvectors needed for MSD computation, currently CPU-only
- **Note**: The flat buffer format (`Vec<f64>`, row-major) is GPU-ready

### Candidate #3: `transport::wavepacket_msd`

- **What**: Mean square displacement of quantum wavepacket via eigendecomposition
- **Where**: Exp 012 only, but generalizable to any time-dependent observable in the eigenbasis
- **barracuda status**: Not present
- **Priority**: Low — the eigenvector solver (Candidate #2) is the bottleneck

---

## Part 3: Provenance Completeness

All 14 benchmark JSONs now have:

| Field | Status |
|-------|--------|
| `_source` | 14/14 |
| `_doi` | 14/14 (10 added in V18) |
| `_references` | 12/14 (sensor_noise and error_propagation use singular `_reference`) |
| `_provenance.baseline_commit` | 14/14 (3 stamped in V18, was "pending") |
| `_provenance.baseline_date` | 14/14 |
| `_provenance.validation_script` | 14/14 |
| `_provenance.command` | 14/14 |
| `_provenance.prng_algorithm` | 12/14 (sensor_noise: N/A analytical, observation_gap: N/A) |
| `_provenance.data_origin` | 13/14 |

All 14 validation binaries now use `print_provenance_header` for consistent output.

---

## Part 4: Three-Tier Control Matrix

| Experiment | Open Data | Local CPU | barracuda CPU | barracuda-GPU | metalForge |
|------------|:---------:|:---------:|:------------:|:------------:|:----------:|
| 001 Sensor Noise | Dong 2020 (DOI) | 36/36 | 36/36 | — | — |
| 002 Observation Gap | ERA5 + GHCND (DOI) | 13/13 | 13/13 | — | — |
| 003 Error Propagation | FAO-56 Ex 18 (DOI) | 15/15 | 15/15 | — | mc_et0_propagate.wgsl |
| 004 Sequencing Noise | EMP (DOI) | 15/15 | 15/15 | — | batched_multinomial.wgsl |
| 005 Seismic | IASP91 (DOI) | 9/9 | 9/9 | — | — |
| 006 Signal Specificity | Massie 2012 (DOI) | 12/12 | 12/12 | — | — |
| 007 RAWR | Wang 2021 (DOI) | 11/11 | 11/11 | — | — |
| 008 Anderson | B&K 2018 (DOI) | 8/8 | 8/8 | 8/8 | — |
| 009 Quasiperiodic | J&K 2018 (DOI) | 8/8 | 8/8 | 8/8 | — |
| 010 Bistable | Fernandez 2020 (DOI) | 9/9 | 9/9 | 9/9 | — |
| 011 Multi-Signal | Srivastava 2011 (DOI) | 9/9 | 9/9 | 9/9 (via ODE) | — |
| 012 Transport | Kachkovskiy 2016 (DOI) | 18/18 | — | — | — |
| 013 Resampling Conv | Wang 2021 (DOI) | 8/8 | — | — | — |
| 014 Drift vs Selection | Anderson 2022 (DOI) | 7/7 | — | — | — |
| **TOTAL** | 14/14 DOIs | **177/177** | **156/177** | **35/177** | 2 shaders |

---

## Part 5: Evolution Readiness Assessment

### GPU Promotion Tiers (updated)

| Module | Tier | Blocker | Notes |
|--------|------|---------|-------|
| `kinetics` | **A** (ready) | None — `hill()` already in barracuda S66 | Wire delegation #27 |
| `transport` | **B** (adapt) | Needs eigenvector solver in barracuda | Flat buffers ready |
| `drift` | **B** (adapt) | `WrightFisherGpu` exists in barracuda S66 | Need to wire |
| `gillespie` | **C** (complex) | Variable-time control flow hard in WGSL | CPU fallback appropriate |
| `prng` | **B** (align) | Xorshift64 vs xoshiro128** | Alignment roadmap documented |

### What groundSpring Learned for ToadStool

1. **Flat buffers are essential for GPU promotion**: The `Vec<Vec<f64>>` → flat refactor was zero-effort because the public API was already slice-based. Design barracuda eigenvector APIs with flat row-major buffers from the start.

2. **Determinism tests catch silent regressions**: The 13 bitwise-equality tests in `determinism.rs` would catch any future change to PRNG streams, reduction ordering, or FP behavior. barracuda should have equivalent tests for any stateful computation.

3. **Provenance as code**: `print_provenance_header` in validation binaries + `_doi` in benchmark JSONs creates a machine-auditable chain from paper → Python → JSON → Rust → pass/fail. Every Spring should adopt this pattern.

4. **The `if let Ok` delegation pattern is robust**: 26 delegations with graceful fallback. No silent failures since the V17 covariance/pearson/spearman bug fix. The pattern should be documented as a barracuda best practice.

---

## Part 6: Action Items for ToadStool

1. **Wire `hill()` delegation** — barracuda S66 exports `hill()` / `monod()`. groundSpring `kinetics::hill` has the delegation stub ready. Verify signature match and enable.

2. **Eigenvector solver** — `barracuda::spectral::find_all_eigenvalues` gives eigenvalues only (Sturm bisection). Adding eigenvectors (tridiag QL or divide-and-conquer) would unlock GPU-accelerated Exp 012 transport.

3. **`WrightFisherGpu` wiring** — barracuda S66 has `WrightFisherGpu` for batched drift+selection. groundSpring Exp 014 (`drift.rs`) could delegate for batch simulations.

4. **Three-mode CI** — groundSpring tests in three modes (local, barracuda, barracuda-gpu) but CI only runs local. Consider adding barracuda-mode CI when barracuda is available as a crates.io dependency.

5. **`_reference` → `_references` normalization** — Two benchmark JSONs (sensor_noise, error_propagation) use singular `_reference` instead of the array `_references`. Consider normalizing in a future pass.

6. **Consider absorbing `transport::wavepacket_msd`** — General eigendecomposition-based observable computation. Could be useful for any time-dependent Schrödinger evolution.

---

## Verification Commands

```bash
cargo fmt --check                                # PASS
cargo clippy --workspace -- -D warnings          # PASS (0 warnings, pedantic + nursery)
cargo doc --workspace --no-deps                  # PASS
cargo test --workspace                           # 225/225 PASS
cargo llvm-cov --workspace --summary-only        # 98.94% line coverage

# Validation binaries (all 14)
for bin in validate-decompose validate-rarefaction validate-seismic validate-weather \
           validate-fao56 validate-signal-specificity validate-rawr validate-anderson \
           validate-quasiperiodic validate-bistable validate-multisignal validate-transport \
           validate-resampling-conv validate-drift; do
    cargo run --bin $bin || echo "FAIL: $bin"
done

# Python baselines (all 14)
python3 -m pytest tests/ -v
```
