# groundSpring V115 → toadStool / barraCuda Evolution Handoff

**Date**: March 18, 2026
**From**: groundSpring V115
**To**: toadStool, barraCuda, coralReef, ecosystem
**Supersedes**: V114 (GROUNDSPRING_V114_TOADSTOOL_BARRACUDA_EVOLUTION_HANDOFF_MAR17_2026.md)
**Pins**: barraCuda v0.3.5, toadStool S156+, coralReef Iteration 52+
**License**: AGPL-3.0-or-later

## Executive Summary

V115 is a deep debt + idiomatic API evolution release. groundSpring evolved
all panicking public APIs (`assert!` in library code) to return `Result<T, InputError>`,
hardened CI with nursery lint enforcement and all-features coverage, and
achieved ecoBin compliance with C-dependency banning in `deny.toml`. The
codebase now has zero panicking public entry points, zero C dependencies,
and UniBin-compliant flag handling.

## Part 1: What V115 Changed

### API Evolution: `assert!` → `Result<T, InputError>`

| Module | Function(s) | Old Signature | New Return Type |
|--------|------------|---------------|-----------------|
| `bootstrap` | `bootstrap_mean`, `rawr_mean`, `bootstrap_median`, `bootstrap_std` | `BootstrapResult` (panics on bad input) | `Result<BootstrapResult, InputError>` |
| `quasispecies` | `quasispecies_simulation`, `quasispecies_simulation_batch` | `Vec<f64>` (panics) | `Result<Vec<f64>, InputError>` |
| `drift` | `wright_fisher_fixation` | `bool` (panics) | `Result<bool, InputError>` |
| `drift` | `neutral_diversity_trajectory` | `Vec<f64>` (panics) | `Result<Vec<f64>, InputError>` |

**Impact on barraCuda consumers**: If barraCuda or toadStool call any of
these functions directly, callers must now handle `Result`. The `InputError`
enum provides structured variants: `InsufficientData`, `OutOfRange`,
`LengthMismatch`.

**Caller convention**:
- Tests: `.unwrap()` (behind `#[expect(clippy::unwrap_used)]`)
- Validation binaries: `.or_exit("context")` via `OrExit` trait
- Library dispatch: `?` propagation with `map_err`

### CI Hardening

| Change | Before | After |
|--------|--------|-------|
| Clippy nursery | In workspace Cargo.toml only | Enforced in CI (`-W clippy::nursery` on all 3 invocations) |
| `cargo doc` | Default features only | `--all-features` (catches biomeos/npu/tarpc-ipc docs) |
| `cargo test` | Default + feature-gated | + `--all-features` job |
| biomeOS validation | Not in CI | 4 binaries with graceful skip |
| metalForge validation | 4 binaries | 11 binaries (+ titan-v, cross-substrate, nestgate-ncbi, nucleus-pipeline, gpu-tier, pure-gpu-workloads, mixed-hardware) |
| aarch64 cross-compile | `--no-default-features` only | + `--features barracuda` (main compute path) |

### ecoBin Compliance

| Change | Detail |
|--------|--------|
| `deny.toml` C-dep ban | 14 crates banned: `openssl-sys`, `openssl`, `native-tls`, `aws-lc-sys`, `aws-lc-rs`, `ring`, `libz-sys`, `bzip2-sys`, `curl-sys`, `libsqlite3-sys`, `cmake`, `cc`, `pkg-config`, `vcpkg` |
| UniBin flags | `--help`/`-h` and `--version`/`-V` in primal binary |
| Niche YAML | `cost_estimates` section (8 capabilities × estimated_ms, gpu_beneficial, peak_memory_mb, deterministic) |

### Hardcoded Fallback Evolution

- `validate_nestgate_ncbi.rs`: `NESTGATE_ADDRESS` env-var discovery added
  before host/port fallback — follows `primal_names::address_env_var()` convention
- Last `.expect()` in production code eliminated → `.or_exit()`

## Part 2: Delegation Map (102 Active — Unchanged)

**61 CPU delegations + 41 GPU delegations across 22 modules.**

No new delegations in V115 — this release focused on API safety and CI
infrastructure rather than new compute surface.

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

## Part 5: Precision Learnings (Cumulative V97–V115)

| Finding | Detail |
|---------|--------|
| GPU adds ~1 ULP per transcendental | Consistent across all 102 delegations |
| Batch dispatch preserves precision | Embarrassingly parallel — no degradation |
| Reduce ops lose ~1 tier | Non-deterministic summation order |
| Division-by-zero is main NaN source | `eps::SAFE_DIV` (1e-10) in kernels |
| f32 accumulation biases ~28% | Green-Kubo in WDM (Exp 025); f64 required |
| NVK/Titan V NAK issues | Prefer proprietary driver for production |
| Bare `as` casts hide truncation | V114 `cast::` helpers surface errors explicitly |
| `assert!` in library code is a consumer hazard | V115 `Result` returns let consumers decide error policy |

## Part 6: IPC and Discovery Evolution (Cumulative V114–V115)

1. **`FAMILY_ID`-aware discovery**: `primal_names::family_id()` reads `FAMILY_ID` env var
2. **`health.liveness`** / **`health.readiness`**: Standard probe endpoints in dispatch table
3. **`resilient_call()`**: CircuitBreaker + RetryPolicy wrapper
4. **`extract_rpc_result()`**: Centralized JSON-RPC 2.0 result/error extraction
5. **`BiomeOsError` query methods**: `is_recoverable()`, `is_retriable()`, `is_method_not_found()`
6. **`NESTGATE_ADDRESS` env-var**: Environment-first discovery for NestGate service

**Recommendation for toadStool**: The `deny.toml` C-dependency banning pattern
is now portable — consider adopting the same 14-crate deny list in barraCuda
and toadStool to enforce ecoBin compliance at the dependency level.

## Part 7: Quality Gates

| Gate | Status |
|------|--------|
| `cargo test --workspace` (930+ tests) | PASS |
| `cargo test --workspace --all-features` | PASS |
| `cargo clippy --workspace --all-targets -D warnings -W clippy::pedantic -W clippy::nursery` | 0 warnings |
| `cargo fmt --all -- --check` | 0 diff |
| `cargo doc --workspace --all-features --no-deps` | 0 warnings |
| Validation binaries (29/29) | PASS at all 3 tiers |
| barraCuda delegations | 102 (61 CPU + 41 GPU) |
| `.expect()` in production | 0 |
| `eprintln!` in production | 0 |
| Panicking public APIs | 0 (all return `Result`) |
| `#[allow()]` in production | 0 (all `#[expect(reason)]`) |
| `unsafe` code | 0 (`#![forbid(unsafe_code)]`) |
| C dependencies | 0 (14 crates banned in `deny.toml`) |

---

**groundSpring V115 | 39 modules | 35 experiments | 930+ tests | 102 delegations | AGPL-3.0-or-later**
