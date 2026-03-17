# groundSpring V113 → Ecosystem Resilience Handoff

**Date**: March 16, 2026
**From**: groundSpring V113
**To**: barraCuda, toadStool, coralReef, ecosystem
**Authority**: Control experiment status + changelog
**Supersedes**: V112 (GROUNDSPRING_V112_DEEP_DEBT_OREXIT_HANDOFF_MAR16_2026.md)
**Pins**: barraCuda v0.3.5, toadStool S155b+, coralReef Iteration 52+
**License**: AGPL-3.0-or-later

## Executive Summary

V113 absorbs the GemmF64 transpose from barraCuda v0.3.5, adds IPC resilience
primitives, extends capability parsing to 4 formats, and standardizes exit codes.

1. GemmF64 `execute_gemm_ex(trans_a=true)` for Tikhonov `KᵀK`, `KᵀG` — P1 resolved
2. `RetryPolicy` + `CircuitBreaker` for IPC resilience (petalTongue/rhizoCrypt)
3. 4-format capability parsing: flat, objects, `capabilities`-wrapped, `result`-wrapped
4. `exit_code` constants per UNIBIN_ARCHITECTURE_STANDARD
5. Deep debt: `#[expect(reason)]`, hardcoded primal name, thiserror evolution

## Quality Gates

| Gate | Status |
|------|--------|
| `cargo test` (618 unit + 24 integration) | PASS |
| `cargo clippy -D warnings` (core) | 0 warnings |
| `cargo clippy -D warnings` (validate) | 0 warnings |
| `cargo fmt --check` | 0 diff |
| Validation binaries (29/29) | PASS at all 3 tiers |
| barracuda delegations | 102 (61 CPU + 41 GPU) |

## Part 1: GemmF64 Transpose Delegation

The P1 request for `GemmF64` transpose flags (V112 handoff) is now resolved.
`tikhonov_solve` in `spectral_recon.rs` delegates `KᵀK` and `KᵀG` to
`execute_gemm_ex(trans_a=true)` when a GPU device is available.

CPU fallback retained for `tikhonov_solve_cpu` (parity testing) and when no GPU.

## Part 2: IPC Resilience

New `biomeos::resilience` module:
- `RetryPolicy`: configurable max retries, initial delay, max delay, multiplier
- `CircuitBreaker`: Closed/Open/HalfOpen with failure threshold and cooldown

Ready for use in provenance trio IPC (rhizoCrypt, loamSpine, sweetGrass).

## Part 3: 4-Format Capability Parsing

`extract_capabilities` now handles:
1. Flat array: `["compute.execute"]`
2. Object array: `[{"name": "...", "version": "1.0"}]`
3. Wrapped: `{"capabilities": [...]}`
4. Result-wrapped: `{"result": [...]}` (JSON-RPC)
5. Double-nested: `{"capabilities": {"capabilities": [...]}}`

## Part 4: Remaining Evolution Requests for barraCuda

| Priority | Request | Status |
|----------|---------|--------|
| ~~P1~~ | ~~GemmF64 transpose flags~~ | **RESOLVED in V113** |
| P1 | Tridiag eigenvectors | Open — local QL retained |
| P1 | GPU FFT (real + complex) | Open — CPU DFT retained |
| P2 | PRNG alignment | Open |
| P2 | Parallel 3D grid dispatch | Open |
| P3 | Unified ComputeScheduler | Open |
| P3 | `erfc` large-x stability | Open |

---

**groundSpring V113 | 39 modules | 35 experiments | 618+ tests | 102 delegations | AGPL-3.0-or-later**
