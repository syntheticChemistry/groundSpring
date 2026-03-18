# groundSpring V114 → toadStool / barraCuda Evolution Handoff

**Date**: March 17, 2026
**From**: groundSpring V114
**To**: toadStool, barraCuda, coralReef, ecosystem
**Supersedes**: V113 (GROUNDSPRING_V113_TOADSTOOL_BARRACUDA_EVOLUTION_HANDOFF_MAR16_2026.md,
GROUNDSPRING_V113_ECOSYSTEM_RESILIENCE_HANDOFF_MAR16_2026.md)
**Pins**: barraCuda v0.3.5, toadStool S156+, coralReef Iteration 52+
**License**: AGPL-3.0-or-later

## Executive Summary

V114 is a cross-ecosystem deep absorption release. groundSpring absorbed
patterns from 6 sibling springs (neuralSpring, airSpring, wetSpring,
healthSpring, ludoSpring, hotSpring) and evolved its entire validation
binary suite to zero `.expect()`, zero `eprintln!`, and zero bare `as` casts.
The codebase now passes all quality gates with 715+ tests, zero clippy
warnings (pedantic + nursery), and zero doc warnings.

## Part 1: What V114 Absorbed

| Pattern | Source Spring | groundSpring Module | Impact |
|---------|-------------|---------------------|--------|
| `safe_cast` expansion (`usize_u32`, `f64_f32`) | neuralSpring S147 | `cast` module, 30+ call sites | Bare `as` casts → checked helpers; GPU dispatch safety |
| `BiomeOsError::is_recoverable()` / `is_retriable()` | airSpring/wetSpring | `biomeos/mod.rs` | IPC retry classification |
| `BiomeOsError::is_method_not_found()` | ludoSpring/healthSpring | `biomeos/mod.rs` | Graceful capability degradation |
| `health.liveness` / `health.readiness` probes | wetSpring/airSpring/healthSpring | `dispatch.rs` | Standard biomeOS health surface |
| `resilient_call()` | petalTongue/rhizoCrypt pattern | `biomeos/resilience.rs` | CircuitBreaker + RetryPolicy wrapper |
| `extract_rpc_result()` | ludoSpring/healthSpring | `biomeos/protocol.rs` | Centralized JSON-RPC result/error extraction |
| `FAMILY_ID`-aware discovery | toadStool/songBird | `primal_names.rs` | Multi-tenant socket paths |
| Zero `eprintln!` in production | neuralSpring pattern | 8+ validation binaries | `OrExit` or `tracing` for all error paths |
| `NEURAL_API_SOCKET_NAMES` centralization | ecosystem standard | `primal_names.rs`, 4 consumer files | Zero hardcoded socket name strings |
| `.expect()` → `OrExit` | wetSpring V123 | 17 validation binaries, 80+ call sites | Zero `.expect()` in validation binaries |
| Bare `as` → `cast::` helpers | neuralSpring safe_cast | `drift`, `gillespie`, `anderson`, `bootstrap`, `tissue_anderson`, `sweeps`, `quasispecies` | Checked numeric conversions |
| Primal composition guidance | ecosystem standard | wateringHole handoff | How groundSpring fits in NUCLEUS atomic types |

## Part 2: Delegation Map (102 Active — Unchanged)

**61 CPU delegations + 41 GPU delegations across 22 modules.**

No new delegations in V114 — this release focused on code quality
evolution rather than new API surface.

| Category | Count | Key APIs |
|----------|-------|----------|
| Stats | 34 | mean, percentile, bootstrap, correlation, regression, diversity, spectral density |
| Linalg | 4 | solve_f64, cholesky_f64, eigh_f64, GemmF64 (transpose) |
| Ops | 14 | peak_detect, FFT, grid_search, batched_multinomial, reduce, correlation, covariance, autocorrelation, ODE, elementwise |
| Spectral | 16 | anderson 1D–4D, almost_mathieu, spectral diagnostics, find_w_c |
| Numerical | 3 | trapz, bistable_ode, multisignal_ode |
| Optimize | 3 | lbfgs, batched_nelder_mead, brent |
| Special | 1 | localization_length |

## Part 3: Remaining API Requests for barraCuda

| Priority | Request | Context | Status |
|----------|---------|---------|--------|
| P1 | Tridiag eigenvectors | `transport.rs` — local QL retained | Open |
| P1 | GPU FFT (real + complex) | `spectral_recon` — CPU DFT retained | Open |
| P2 | PRNG alignment | xorshift64 vs xoshiro128** | Open |
| P2 | Parallel 3D grid dispatch | seismic.rs, freeze_out.rs | Open |
| P3 | Unified ComputeScheduler | metalForge routes manually | Open |
| P3 | `erfc` large-x stability | hotSpring Exp 046 | Open |

## Part 4: WGSL Shader Candidates for Absorption

Two unique Anderson Lyapunov shaders remain in metalForge:

| Shader | Path | Precision | Status |
|--------|------|-----------|--------|
| `anderson_lyapunov.wgsl` | `metalForge/shaders/` | f64 | Absorption candidate |
| `anderson_lyapunov_f32.wgsl` | `metalForge/shaders/` | f32 fallback | Absorption candidate |

These compute Lyapunov exponents for 1D Anderson models via transfer
matrix method. If `barracuda::spectral` already provides equivalent
GPU kernels, groundSpring can lean on upstream and retire the locals.
Otherwise, these are well-documented absorption targets with binding
layouts and dispatch geometry in the shader headers.

## Part 5: Precision Learnings (Cumulative V97–V114)

| Finding | Detail |
|---------|--------|
| GPU adds ~1 ULP per transcendental | Consistent across all 102 delegations |
| Batch dispatch preserves precision | Embarrassingly parallel — no degradation |
| Reduce ops lose ~1 tier | Non-deterministic summation order |
| Division-by-zero is main NaN source | `eps::SAFE_DIV` (1e-10) in kernels |
| f32 accumulation biases ~28% | Green-Kubo in WDM (Exp 025); f64 required |
| NVK/Titan V NAK issues | Prefer proprietary driver for production |
| Bare `as` casts hide truncation | V114 `cast::` helpers surface errors explicitly |

## Part 6: IPC and Discovery Evolution (V114)

1. **`FAMILY_ID`-aware discovery**: `primal_names::family_id()` reads `FAMILY_ID` env var, `discover_socket()` checks family-prefixed paths first
2. **`health.liveness`** / **`health.readiness`**: Standard probe endpoints in dispatch table
3. **`resilient_call()`**: Wraps any `FnMut() -> Result<T, E>` with CircuitBreaker + RetryPolicy
4. **`extract_rpc_result()`**: Centralizes JSON-RPC 2.0 result/error extraction from response bodies
5. **`BiomeOsError` query methods**: `is_recoverable()`, `is_retriable()`, `is_method_not_found()` for IPC retry logic

**Recommendation for toadStool**: The `health.liveness`/`health.readiness`
pattern is now consistent across wetSpring, airSpring, healthSpring, and
groundSpring. Consider standardizing the response schema if not already.

## Part 7: Quality Gates

| Gate | Status |
|------|--------|
| `cargo test --workspace --lib` (715 tests) | PASS |
| `cargo clippy --workspace --all-targets -D warnings -W clippy::pedantic -W clippy::nursery` | 0 warnings |
| `cargo fmt --all -- --check` | 0 diff |
| `cargo doc --workspace --no-deps` | 0 warnings |
| Validation binaries (29/29) | PASS at all 3 tiers |
| barracuda delegations | 102 (61 CPU + 41 GPU) |
| `.expect()` in validation binaries | 0 |
| `eprintln!` in production | 0 |
| Bare `as` casts in library | Minimal (checked alternatives used) |
| `#[allow()]` in production | 0 (all `#[expect(reason)]`) |
| `unsafe` code | 0 (`#![forbid(unsafe_code)]`) |

---

**groundSpring V114 | 39 modules | 35 experiments | 715+ tests | 102 delegations | AGPL-3.0-or-later**
