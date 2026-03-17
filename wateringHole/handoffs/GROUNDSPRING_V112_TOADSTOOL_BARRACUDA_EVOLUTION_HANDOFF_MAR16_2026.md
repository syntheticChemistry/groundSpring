# groundSpring V112 → toadStool / barraCuda Evolution Handoff

**Date**: March 16, 2026
**From**: groundSpring V112
**To**: toadStool, barraCuda
**License**: AGPL-3.0-or-later
**Pins**: barraCuda v0.3.5, toadStool S155b+

## Summary

groundSpring V112 completes two sprints (V111 + V112) of deep debt evolution. This handoff communicates:
1. What groundSpring learned about barraCuda/toadStool APIs through 102 delegations
2. What groundSpring evolved that toadStool/barraCuda should absorb
3. Outstanding API requests by priority

## Part 1: Ecosystem Patterns to Absorb

### 1.1 `thiserror` for Error Types
groundSpring evolved all error types from manual `Display` + `Error` impls to `thiserror::Error` derives. This pattern is now standard across wetSpring V123, healthSpring V31, and groundSpring V112. barraCuda v0.3.5 should evaluate adopting `thiserror` for its error types.

### 1.2 `#[expect(reason)]` over `#[allow()]`
groundSpring V110 migrated 95 files from `#[allow()]` to `#[expect(reason)]`. Stale suppressions now produce compile warnings. barraCuda should adopt this pattern — no `#[allow()]` should remain in production code.

### 1.3 `OrExit<T>` for Validation Binaries
Zero-boilerplate exit pattern adopted by wetSpring, healthSpring, and groundSpring. If barraCuda has validation binaries, they should use this pattern:
```rust
pub trait OrExit<T> {
    fn or_exit(self, msg: &str) -> T;
}
```

### 1.4 `DispatchOutcome` for RPC Classification
groundSpring classifies JSON-RPC responses as `Ok`, `ProtocolError`, or `ApplicationError`. toadStool should ensure `compute.dispatch.*` methods return standard JSON-RPC error codes (-32700 to -32600 for protocol, other ranges for application).

### 1.5 Generic Primal Discovery
`socket_env_var(primal_name)` generates `"{UPPER_NAME}_SOCKET"`. toadStool's `ComputeScheduler` should use generic discovery helpers rather than hardcoded primal socket paths.

### 1.6 Config Injection / DI Pattern
All environment variable reads in groundSpring now have `_with_env` variants that accept an injected reader closure. toadStool should consider this pattern for testability — no direct `std::env::var` in library code.

### 1.7 Safe Casts
Every numeric `as` cast in groundSpring either uses `From` (lossless), `crate::cast` helpers (saturating), or `#[expect(reason)]` with justification. barraCuda should audit its `as` casts similarly.

## Part 2: barraCuda API Evolution Requests

### P1: GemmF64 Transpose Flags
**Context**: `spectral_recon.rs` keeps local `mat_transpose_mul` / `mat_transpose_vec` because `barracuda::ops::linalg::GemmF64` lacks transpose control.
**Ask**: Add `transpose_a: bool, transpose_b: bool` parameters to `GemmF64::dispatch()`.
**Impact**: Eliminates ~40 lines of local matrix code in groundSpring.

### P1: Tridiag Eigenvectors
**Context**: `transport.rs` needs eigenvectors from tridiagonal matrices. barracuda's Sturm bisection returns eigenvalues only. groundSpring uses local QL iteration for eigenvectors.
**Ask**: Add eigenvector computation to `barracuda::spectral` (QL or divide-and-conquer for tridiagonal).
**Impact**: Would enable full spin-chain transport delegation to GPU.

### P1: GPU FFT (Real + Complex)
**Context**: Tikhonov spectral reconstruction has Cholesky solve on GPU but FFT still on CPU.
**Ask**: `barracuda::ops::fft::Fft1DF64` exists but GPU FFT is needed for the full pipeline.
**Impact**: Full spectral reconstruction pipeline on GPU.

### P2: PRNG Alignment
**Context**: groundSpring uses local `xorshift64`; barracuda uses `xoshiro128**`. Full baseline regeneration would be needed for bitwise GPU–CPU reproducibility.
**Ask**: Either provide `xorshift64` as an option in barracuda, or accept that PRNG alignment requires baseline regeneration.
**Impact**: Affects bootstrap reproducibility in metalForge three-tier parity tests.

### P2: Parallel 3D Grid Dispatch
**Context**: `seismic.rs` and `freeze_out.rs` do exhaustive 3D grid searches on CPU.
**Ask**: `barracuda::ops::grid::grid_search_3d` GPU variant for embarrassingly parallel grid evaluation.
**Impact**: Would accelerate inverse problem validation.

### P3: Unified ComputeScheduler
**Context**: metalForge manually routes to CPU/GPU/NPU based on feature flags and device availability.
**Ask**: toadStool `ComputeScheduler` that handles device selection, fallback, and load balancing.
**Impact**: Simplifies metalForge substrate selection logic.

### P3: `erfc` Large-x Stability
**Context**: hotSpring Exp 046 flagged numerical cancellation in `erfc(x) = 1 - erf(x)` at large x.
**Ask**: Direct asymptotic expansion for `erfc` at large x (barraCuda ISSUE-006).
**Impact**: Precision for Gaussian-tail computations.

## Part 3: What groundSpring Does NOT Need

| Item | Reason |
|------|--------|
| Chao1 delegation | Stays local — Chao 1984 formula; barracuda uses Chao & Chiu 2016 |
| Small matrix transpose | n_omega ≤ 200; dispatch overhead exceeds benefit |
| PRNG replacement | Local `xorshift64` is intentional for baseline reproducibility |
| Bootstrap delegation | Resampling is I/O-bound, not compute-bound |

## Part 4: Precision Learnings

| Finding | Detail |
|---------|--------|
| GPU adds ~1 ULP per transcendental | Consistent across all 102 delegations |
| Batch dispatch preserves precision | Embarrassingly parallel — no degradation |
| Reduce ops lose ~1 tier | Non-deterministic summation order |
| Division-by-zero is main NaN source | Use `eps::SAFE_DIV` (1e-10) in kernels |
| f32 accumulation biases ~28% | Green-Kubo in WDM; f64 required for physics |
| NVK/Titan V NAK issues | `WgpuDevice` prefers proprietary driver for production |

## Part 5: Current Delegation Map

**102 delegations total** (61 CPU + 41 GPU):
- **Stats**: 34 ops (mean, percentile, bootstrap, correlation, regression, diversity, spectral density)
- **Linalg**: 4 ops (solve_f64, cholesky_f64, eigh_f64, ridge_regression)
- **Ops**: 14 ops (peak_detect, FFT, grid_search, batched_multinomial, reduce, correlation, covariance, autocorrelation, ODE, elementwise)
- **Spectral**: 16 ops (anderson variants 1D–4D, almost_mathieu, spectral diagnostics, find_w_c)
- **Numerical**: 3 ops (trapz, bistable_ode, multisignal_ode)
- **Optimize**: 3 ops (lbfgs, batched_nelder_mead, brent)
- **Special**: 1 op (localization_length)

**Three-tier parity**: 29/29 validation binaries PASS at CPU / barracuda-CPU / barracuda-GPU.

---

**groundSpring V112 | 39 modules | 35 experiments | 912+ tests | 102 delegations | AGPL-3.0-or-later**
