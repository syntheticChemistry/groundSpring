# groundSpring V113 → toadStool / barraCuda Evolution Handoff

**Date**: March 16, 2026
**From**: groundSpring V113
**To**: toadStool, barraCuda
**License**: AGPL-3.0-or-later
**Pins**: barraCuda v0.3.5, toadStool S156+, coralReef Iteration 52+

## Summary

groundSpring V113 completes three sprints of deep debt evolution (V111–V113).
This handoff communicates what groundSpring absorbed, what it learned, and
what remains open for barraCuda/toadStool evolution.

## Part 1: GemmF64 Transpose — Resolved

The P1 `GemmF64` transpose request from V112 is now resolved.

`spectral_recon::tikhonov_solve` delegates `KᵀK` and `KᵀG` to
`barracuda::ops::linalg::GemmF64::execute_gemm_ex(trans_a=true)` when GPU
is available. CPU fallback retained for parity testing.

**Thank you for the fast turnaround** — this eliminated ~40 lines of local
matrix code and completes the Tikhonov GPU pipeline.

## Part 2: Ecosystem Patterns Absorbed (V111–V113)

| Pattern | Source | groundSpring Module |
|---------|--------|---------------------|
| `thiserror` for error types | ecosystem standard | `error.rs`, `biomeos/mod.rs`, `ipc.rs`, `validate/lib.rs` |
| `DispatchOutcome` enum | airSpring V0.8.3 | `biomeos/protocol.rs` |
| `OrExit<T>` trait | wetSpring V123 | `validate/lib.rs` |
| `exit_code` constants | sweetGrass v0.7.19 | `validate/lib.rs` |
| `RetryPolicy` + `CircuitBreaker` | petalTongue/rhizoCrypt | `biomeos/resilience.rs` |
| 4-format capability parsing | airSpring V0.8.7 | `biomeos/interaction.rs` |
| `socket_env_var()` / `address_env_var()` | sweetGrass v0.7.18 | `primal_names.rs` |
| `#[expect(reason)]` over `#[allow()]` | wetSpring V122 | 95+ files |
| Config injection `_with_env` DI | ecosystem standard | `biomeos/*.rs`, `gpu.rs`, `ipc.rs` |
| Safe casts via `crate::cast` | ecosystem standard | 25+ call sites |
| `parse_benchmark()` | wetSpring V123 | 28 validation binaries |

## Part 3: Delegation Map (102 Active)

**61 CPU delegations + 41 GPU delegations across 22 modules.**

| Category | Count | Key APIs |
|----------|-------|----------|
| Stats | 34 | mean, percentile, bootstrap, correlation, regression, diversity, spectral density |
| Linalg | 4 | solve_f64, cholesky_f64, eigh_f64, ridge_regression |
| Ops | 14 | peak_detect, FFT, grid_search, batched_multinomial, reduce, correlation, covariance, autocorrelation, ODE, elementwise |
| Spectral | 16 | anderson 1D–4D, almost_mathieu, spectral diagnostics, find_w_c |
| Numerical | 3 | trapz, bistable_ode, multisignal_ode |
| Optimize | 3 | lbfgs, batched_nelder_mead, brent |
| Special | 1 | localization_length |
| **NEW V113** | +1 | GemmF64::execute_gemm_ex (Tikhonov KᵀK, KᵀG) |

## Part 4: Remaining API Requests

| Priority | Request | Context | Status |
|----------|---------|---------|--------|
| ~~P1~~ | ~~GemmF64 transpose~~ | spectral_recon | **Resolved V113** |
| P1 | Tridiag eigenvectors | transport.rs — local QL retained | Open |
| P1 | GPU FFT (real + complex) | spectral_recon — CPU DFT retained | Open |
| P2 | PRNG alignment | xorshift64 vs xoshiro128** | Open |
| P2 | Parallel 3D grid dispatch | seismic.rs, freeze_out.rs | Open |
| P3 | Unified ComputeScheduler | metalForge routes manually | Open |
| P3 | `erfc` large-x stability | hotSpring Exp 046 | Open |

## Part 5: Precision Learnings from 102 Delegations

| Finding | Detail |
|---------|--------|
| GPU adds ~1 ULP per transcendental | Consistent across all delegations |
| Batch dispatch preserves precision | Embarrassingly parallel — no degradation |
| Reduce ops lose ~1 tier | Non-deterministic summation order |
| Division-by-zero is main NaN source | `eps::SAFE_DIV` (1e-10) in kernels |
| f32 accumulation biases ~28% | Green-Kubo in WDM; f64 required |
| NVK/Titan V NAK issues | Prefer proprietary driver for production |

## Part 6: What groundSpring Does NOT Need

| Item | Reason |
|------|--------|
| Chao1 delegation | Chao 1984 formula; barracuda uses Chao & Chiu 2016 |
| Small matrix transpose | n_omega ≤ 200; dispatch overhead exceeds benefit |
| PRNG replacement | Local xorshift64 intentional for baseline reproducibility |
| Bootstrap delegation | I/O-bound, not compute-bound |

## Part 7: IPC and Discovery Evolution

groundSpring V113 has evolved its IPC surface:

1. **Typed IpcError**: `Connect`, `Transport`, `Remote`, `Discovery` variants
2. **DispatchOutcome**: `Ok`, `ProtocolError`, `ApplicationError` classification
3. **4-format capability parsing**: flat, objects, capabilities-wrapped, result-wrapped
4. **RetryPolicy + CircuitBreaker**: Ready for provenance trio calls
5. **socket_env_var() / address_env_var()**: Generic primal discovery
6. **exit_code constants**: SUCCESS, GENERAL_ERROR, CONFIG_ERROR, NETWORK_ERROR
7. **compute.dispatch.***: submit, result, capabilities (direct toadStool dispatch)

**Recommendation for toadStool**: Ensure `compute.dispatch.*` methods return
standard JSON-RPC error codes. ProtocolError range: -32700 to -32600.
ApplicationError: all other negative codes.

---

**groundSpring V113 | 39 modules | 35 experiments | 930+ tests | 102 delegations | AGPL-3.0-or-later**
