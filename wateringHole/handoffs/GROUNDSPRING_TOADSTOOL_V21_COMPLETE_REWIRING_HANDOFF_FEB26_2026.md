# groundSpring → ToadStool/BarraCUDA V21 — Complete Barracuda Rewiring

**Date**: February 26, 2026
**From**: groundSpring (noise validation + environmental modeling)
**To**: ToadStool/BarraCUDA team
**Previous**: V20 (S68 catch-up + hill delegation)
**ToadStool HEAD**: `f0feb226` (S68 — universal precision, zero f32-only)
**License**: AGPL-3.0-or-later

---

## Executive Summary

- **`--features barracuda` now compiles cleanly** — zero warnings in both CPU-only and barracuda-delegated modes (was 17 dead-code warnings + 3 import warnings + 4 needless_return lints).
- **226/226 tests pass in both modes** — dual-mode CI validates cross-spring math correctness.
- **CPU delegation overhead: +1.7%** — functionally free (16,447ms → 16,722ms total across 15 experiments).
- **Anderson/RAWR faster with barracuda** — barracuda's optimized implementations outperform groundSpring's CPU path (742ms vs 831ms, 604ms vs 640ms).
- **Domain guard fix**: `kinetics::hill` now applies biological convention (x ≤ 0 → 0) before delegation, preventing barracuda's pure-math `(-1)^2 / (1+1) = 0.5` from violating enzyme kinetics semantics.

---

## Part 1: What Was Fixed

### Domain Guard (Test-Breaking Bug)

barracuda's `stats::hill(-1.0, 1.0, 2.0)` returns `0.5` (pure math). groundSpring's biological convention requires `0.0` for non-positive concentrations. The V20 delegation bypassed this guard:

```rust
// V20 — Bug: negative x reaches barracuda
#[cfg(feature = "barracuda")]
return barracuda::stats::hill(x, k, n);

// V21 — Fixed: domain guard before delegation
if x <= 0.0 { return 0.0; }
#[cfg(feature = "barracuda")]
return barracuda::stats::hill(x, k, n);
```

**Recommendation for ToadStool**: Consider adding optional domain-clamped variants (`hill_bio`, `hill_chem`) that enforce non-negative inputs. The pure-math `hill()` is correct for mathematics; the guard belongs in the biological domain layer.

### Dead-Code Warnings (17 Functions)

All `_cpu` fallback functions now gated with `#[cfg(not(feature = "barracuda"))]`:

| File | Functions Gated |
|------|----------------|
| anderson.rs | `DERRIDA_GARDNER_CONSTANT`, `analytical_localization_length_cpu` |
| bistable.rs | `bistable_derivative_cpu` |
| multisignal.rs | `multisignal_derivative_cpu` |
| rarefaction.rs | `shannon_diversity_cpu`, `evenness_cpu` |
| stats/distributions.rs | `norm_cdf_cpu`, `erf_cpu`, `norm_ppf_cpu` |
| stats/metrics.rs | `rmse_cpu`, `mbe_cpu`, `r_squared_cpu`, `index_of_agreement_cpu`, `hit_rate_cpu`, `mean_cpu`, `percentile_cpu` |

### Import Gating

| File | Import | Gate |
|------|--------|------|
| bistable.rs | `use crate::kinetics::hill` | `#[cfg(not(feature = "barracuda"))]` |
| multisignal.rs | `use crate::kinetics::{hill, hill_repress}` | `#[cfg(not(feature = "barracuda"))]` |
| stats/metrics.rs | `use crate::cast::f64_usize` | `#[cfg(not(feature = "barracuda"))]` |

### Needless-Return Cleanup

Four `#[cfg(feature = "barracuda")]` blocks used `return [...]` where the block is the only code path. Changed to expression position (clippy `needless_return`):
- `bistable.rs`: `return [result[0]...]` → `[result[0]...]`
- `multisignal.rs`: same pattern
- `rarefaction.rs` (shannon): `return barracuda::stats::shannon(...)` → `barracuda::stats::shannon(...)`
- `rarefaction.rs` (evenness): same pattern

---

## Part 2: CPU vs Barracuda Benchmark

All 15 validation binaries, `--release` mode:

| Experiment | CPU-only | Barracuda | Δ | Delegations Active |
|-----------|----------|-----------|---|-------------------|
| 001 decompose | 69ms | 84ms | +22% | rmse, mbe, r² |
| 002 weather | 67ms | 78ms | +16% | rmse, mbe, r², ioa, mean |
| 003 seismic | 119ms | 128ms | +8% | none in hot path |
| 004 rarefaction | 70ms | 93ms | +33% | shannon, evenness |
| 005 fao56 | 82ms | 95ms | +16% | none in hot path |
| 006 signal | 855ms | 919ms | +7% | bootstrap_mean |
| 007 rawr | 640ms | **604ms** | **-6%** | rawr_mean |
| 008 anderson | 831ms | **742ms** | **-11%** | lyapunov_exponent |
| 009 quasiperiodic | 11,750ms | 11,836ms | +1% | hamiltonian, eigenvalues, lsr |
| 010 bistable | 173ms | 198ms | +14% | bistable_derivative, hill |
| 011 multisignal | 101ms | 128ms | +27% | multisignal_derivative, hill |
| 012 transport | 313ms | 365ms | +17% | none (eigenvector gap) |
| 013 resampling | 123ms | 166ms | +35% | bootstrap_mean |
| 014 drift | 1,146ms | 1,155ms | +1% | none (WF CPU-only) |
| 015 uncertainty | 108ms | 131ms | +21% | lyapunov_exponent |
| **Total** | **16,447ms** | **16,722ms** | **+1.7%** | |

**Key insights**:
1. Small experiments (+14-35%) show call overhead from barracuda's indirection
2. Heavy experiments (Anderson -11%, RAWR -6%) are *faster* — barracuda's implementations benefit from ToadStool-side optimization
3. Total overhead is negligible (+1.7%) — CPU delegation is functionally free
4. Real speedup opportunity: GPU delegation for Exp 009 (eigensolver, 11.8s) and Exp 014 (Wright-Fisher, 1.2s)

---

## Part 3: CI Evolution

CI now validates both modes:

```yaml
- run: cargo clippy --workspace -- -D warnings
- run: cargo clippy --workspace --features barracuda -- -D warnings
- run: cargo test --workspace
- run: cargo test --workspace --features barracuda
```

This ensures cross-spring math correctness is validated on every push.

---

## Part 4: Cross-Spring Shader Evolution

groundSpring now documents the full lineage of its 27 delegations through 5 springs' shader contributions. Key findings:

| Origin | Shader Categories | groundSpring Delegations |
|--------|------------------|------------------------|
| hotSpring | Precision (DF64), spectral (Lanczos, Anderson, Sturm) | #9-12, #23-24 (Lyapunov, eigenvalues, LSR) |
| wetSpring | Bio-stats, ODE, diversity, Gillespie | #13-14, #20, #25 (ODE derivatives, Shannon, evenness) |
| airSpring | Error metrics, FAO-56 | #15-22 (RMSE, MBE, R², etc.) |
| neuralSpring | Spectral density, dispatch pattern, PRNG | Blueprint for GPU dispatch |
| groundSpring | Validation patterns, RAWR | #26-27 (rawr_mean, hill) |

---

## Part 5: Remaining Delegation Opportunities

| Candidate | barracuda API | Priority | Blocker |
|-----------|--------------|----------|---------|
| `transport::tridiag_eigh` (eigenvectors) | None — only eigenvalues via Sturm | High | ToadStool needs eigenvector solver |
| `drift::wright_fisher_fixation` | `WrightFisherGpu` | Medium | GPU-only, needs CPU path |
| `gillespie::birth_death_ssa` | `GillespieGpu` | Medium | GPU-only, needs CPU path |
| `rarefaction::multinomial_sample` | `multinomial_sample_cpu` | Low | Signature adapter (u64 vs u32) |
| `fao56::*` (18 functions) | FAO-56 WGSL shaders | Low | Batch dispatch, not per-call |
| `bistable/multisignal::rk4_step` | `rk45_solve` | Low | Trait-based API vs fixed-array |

---

## Verification Commands

```bash
cargo test --workspace                           # 226 tests, CPU-only
cargo test --workspace --features barracuda      # 226 tests, barracuda-delegated
cargo clippy --workspace -- -D warnings          # zero warnings, CPU-only
cargo clippy --workspace --features barracuda -- -D warnings  # zero warnings, barracuda

# Release benchmark
time cargo run --release --bin validate-anderson                    # ~831ms CPU
time cargo run --release --features barracuda --bin validate-anderson  # ~742ms barracuda
```

---

*groundSpring V21 | February 26, 2026 | Complete barracuda rewiring | 15 experiments, 226 tests (dual-mode), 185/185 checks, 27 delegations (22 CPU + 5 GPU)*
