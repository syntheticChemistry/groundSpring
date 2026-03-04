# groundSpring → barraCuda V74 Deep Debt + ToadStool/barraCuda Catch-Up Handoff

**Date**: March 4, 2026
**From**: groundSpring V74
**To**: barraCuda team / ToadStool / ecoPrimals ecosystem
**barraCuda pin**: `f6895ca` (v0.3.1)
**toadStool pin**: `9319668d` (S93)

---

## Summary

V74 completes the deep debt evolution started in V72-V73, adds clippy pedantic
CI enforcement, and performs a full ToadStool S70→S93 / barraCuda v0.2.0→v0.3.1
catch-up audit. All 81 delegations verified current. No new barraCuda primitives
to wire — the ecosystem is fully synchronized.

## What Changed in V74

### Code Quality

| Change | File(s) | Impact |
|--------|---------|--------|
| `let_and_return` fix | `wdm.rs` | Closure simplified to direct expression |
| `too_many_lines` fix | `drift.rs` | `wf_batch_gpu` split into 3 functions |
| `eps::UNDERFLOW` integration | `linalg.rs` | QL convergence guard against subnormals |
| Clippy pedantic CI | `.github/workflows/ci.yml` | `-W clippy::pedantic` on all 3 feature modes |
| Tolerance deduplication | `groundspring-validate/src/lib.rs` | 7 constants re-exported from `groundspring::tol` |
| Named optimizer constants | `freeze_out.rs` | 7 L-BFGS + 3 NM + seed, feature-gated |
| Named physical clamp bounds | `fao56/pipeline.rs` | 4 Monte Carlo perturbation limits |
| Named timeout constants | `biomeos/mod.rs` | Connection (5s) + read (30s) timeouts |

### Documentation Catch-Up

| Document | State Before | State After |
|----------|-------------|-------------|
| `ABSORPTION_MANIFEST.md` | V61 (32 delegations, stale Tier B/C) | V74 (81 delegations, Tier B→1 remaining, Tier C→fully absorbed) |
| `BARRACUDA_EVOLUTION.md` | V73 header | V74 header |
| `CROSS_SPRING_SHADER_EVOLUTION.md` | V72 summary | V74 summary (790 tests) |
| `CHANGELOG.md` | V73 latest | V74 entry added |
| `CONTROL_RUN_LOG.md` | Run 42 | Run 43 added |

## ToadStool/barraCuda Audit Results

### ToadStool S87→S93 Evolution (reviewed)

| Session | Key Change | groundSpring Impact |
|---------|-----------|-------------------|
| S87 | FHE shader fix, async-trait reclassification | None — FHE not used |
| S88 | groundSpring V68 absorption (anderson_4d, LbfgsGpu, tridiag_eigenvectors) | Already rewired in V68 |
| S89 | barraCuda extraction to standalone primal | Already rewired in V70 |
| S90-S92 | REST→JSON-RPC, sovereignty deprecations | None — already JSON-RPC |
| S93 | D-DF64 transfer, docs cleanup | None — precision handled by barraCuda |

### barraCuda v0.2.0→v0.3.1 Evolution (reviewed)

| Version | Key Change | groundSpring Impact |
|---------|-----------|-------------------|
| v0.2.0 | Embedded in ToadStool | Historical — extracted in v0.3.0 |
| v0.2.1 | Demarcation, domain feature-gates | Path swap (V70) |
| v0.3.0 | ToadStool untangle (zero cross-deps) | Already pinned |
| v0.3.1 | tarpc parity, blake3 pure, tracing, 73 tests | Already pinned (V71) |

### Verdict: Fully Synchronized

All 81 delegations are current with barraCuda v0.3.1. No new barraCuda
primitives have been added since v0.3.1 that groundSpring needs. The
barraCuda API surface was audited against our delegation inventory —
zero gaps found.

## Remaining Evolution Candidates

| Item | Priority | Blocker |
|------|----------|---------|
| PRNG xorshift64→xoshiro128** alignment | P2 | Full rebaseline of all benchmark JSONs |
| `band_edges` algorithm mismatch | P3 | Brent vs Sturm root-finding difference |

## Validation Benchmark (Run 43)

28 validation binaries, release mode, i9-12900K:

| Mode | Result | Total Time | vs Default |
|------|--------|------------|------------|
| Default (no barracuda) | 28/28 PASS | 21.7s | — |
| barraCuda CPU delegation | 28/28 PASS | 19.6s | **−10%** |
| Cross-spring benchmark | 23/23 PASS | 4.5s | — |

Notable cross-spring speedups in barraCuda mode:

| Binary | Default | barraCuda | Δ | Cross-Spring Origin |
|--------|---------|-----------|---|-------------------|
| validate-fao56 | 74ms | 16ms | **−78%** | airSpring hydrology S66 |
| validate-spectral-recon | 62ms | 12ms | **−81%** | hotSpring Lanczos S59 |
| validate-freeze-out | 30ms | 10ms | **−67%** | airSpring L-BFGS S84 |
| validate-jackknife | 14ms | 6ms | **−57%** | wetSpring stats S70+ |
| validate-precision-drift | 4162ms | 3453ms | **−17%** | hotSpring DF64 S58 |
| validate-drift | 1263ms | 1093ms | **−13%** | neuralSpring Wright-Fisher S66 |

## Quality Gates (Run 43)

| Gate | Result |
|------|--------|
| `cargo fmt --check` | PASS |
| `cargo clippy -- -D warnings -W clippy::pedantic` (default) | PASS |
| `cargo clippy -- -D warnings -W clippy::pedantic` (barracuda) | PASS |
| `cargo doc --workspace --no-deps` | PASS |
| `cargo test --workspace` (default) | 790 PASS |
| `cargo test --workspace --features barracuda` | PASS |
| Line coverage | 97.25% |

## Architecture

```
groundSpring V74  ──→  barraCuda v0.3.1   (WHAT to compute — 81 delegations)
                       └── 767 WGSL shaders, f64-canonical, DF64 universal precision
toadStool S93     ──→  barraCuda v0.3.1   (WHERE/HOW to compute — dispatch)
                       └── akida-driver stays in ToadStool (hardware, not math)
barraCuda v0.3.1  ──→  sourDough          (primal traits only)
```

## Active Handoffs (wateringHole/handoffs/)

| Handoff | Purpose |
|---------|---------|
| **V74 Deep Debt + Catch-Up** (this) | Full ToadStool/barraCuda audit, code quality |
| V73 Tolerance Architecture | 13-tier `tol::`/`eps::` architecture reference |
| Absorption Guide | Delegation patterns, feature gating, evolution requests |

## Next Steps for barraCuda Team

1. **P1 — FFT**: groundSpring needs real-valued FFT for potential future spectral methods
2. **P1 — Tridiag eigenvector solver**: Sturm bisection + inverse iteration (absorbed V68 but not yet in standalone barraCuda v0.3.1 public API)
3. **P2 — PRNG alignment**: When xoshiro128** becomes the canonical PRNG, groundSpring will regenerate all baselines
4. **P2 — Parallel 3D grid dispatch**: For seismic inversion (lat,lon,depth workgroups)
