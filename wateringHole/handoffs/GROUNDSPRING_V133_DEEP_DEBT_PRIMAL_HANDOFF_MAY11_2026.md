# groundSpring V133 — Deep Debt V2 + Primal/Spring Handoff

**Date:** May 11, 2026
**From:** groundSpring V133 (River Delta Evolution + Deep Debt V2)
**To:** primalSpring audit, upstream primal teams, spring teams, projectNUCLEUS

## Summary

groundSpring V133 completes the River Delta Evolution cycle: Tier 4 metalForge barracuda decoupling, plasmidBin refresh, foundation Thread 7 seeding, deep debt cleanup (nucleus.rs panic fix, benchmark correctness, script coverage expansion). All post-interstadial targets confirmed. Zero clippy warnings, zero fmt diff, 1,101 tests + 11 doctests passing.

## What Changed (V131→V133)

### V132 — River Delta Evolution
- **Tier 4 metalForge decoupling**: barracuda now `optional = true` in metalForge. `pollster` replaces `barracuda::device::test_pool::tokio_block_on`. wgpu gains `vulkan` backend for standalone GPU probing. `cargo check --workspace` succeeds without barraCuda source tree.
- **barracuda-gpu feature fix**: Added `barracuda/domain-genomics` — `gillespie.rs` and `drift/mod.rs` need `barracuda::ops::bio` which is gated behind that feature.
- **plasmidBin refresh**: 1.1M stripped binary (was 1.3M from March 28). BLAKE3: `f6fb35332d600eca56988bc53c288bbbd5c8317e04fb23fe24f11718991e2e69`.
- **Foundation Thread 7 seeded**: `validation/anderson-20260511/` — 18/18 targets PASS with braid.json + PROVENANCE_MANIFEST.md.

### V133 — Deep Debt V2
- **nucleus.rs**: `panic!` on UID discovery → `tracing::error!` + fallback to UID `"0"`. No production panics remain.
- **bench_rust_vs_python.py**: `python_pass`/`rust_pass` fields were hardcoded `True` — now reflect actual subprocess exit codes.
- **bench_barracuda_cpu_vs_python.py**: `NPU_EXPERIMENTS` was dead code — now wired into `main()` with hardware detection.
- **run_all_baselines.sh**: Phase 2 section added: `validate_all` aggregator, `groundspring_guidestone`, 4 biomeOS validators with socket detection.
- **NUCLEUS workload TOML**: `$SPRINGS_ROOT` → `${SPRINGS_ROOT:-<default>}` shell-default pattern.

## Current State

| Metric | Value |
|--------|-------|
| Version | V133 |
| Tests | 1,101 lib + 11 doctests |
| Validation checks | 395/395 (35 experiments) |
| metalForge checks | 138 |
| Clippy warnings | 0 (pedantic + nursery) |
| GuideStone level | L4 (5 modular NUCLEUS layer modules) |
| barraCuda delegations | 110 (67 CPU + 43 GPU) |
| plasmidBin binary | 1.1M stripped |
| Foundation seeding | Thread 7: 18/18 PASS |

## Post-Interstadial Targets — All Confirmed

| Target | Status |
|--------|--------|
| UniBin | `groundspring_unibin` 1.1M stripped |
| skunkBat Rust IPC | `src/ipc/skunkbat.rs` wired |
| `method.register` | `biomeos/registration.rs` absorbed |
| CI cross-sync 413 | `>= 401` assertion passing |
| `composition.status` | `biomeos/health.rs` absorbed |
| NUCLEUS workload | `groundspring-geochemistry-validation.toml` live |
| Tier 4 IPC-first | barracuda optional everywhere (groundspring + metalForge) |
| plasmidBin binary | Deployed to `infra/plasmidBin/springs/` |
| Foundation seeded | Thread 7 Anderson Mathematics 18/18 |
| JH-5 ready | skunkBat IPC wired, no action needed |

## Upstream Gaps — For Primal Teams

### HIGH PRIORITY: NestGate Not Live

NestGate HTTP API is required for Exp 029-032 (real data acquisition from NOAA, NCBI, IRIS). Currently all biomeOS validators (`validate_real_ghcnd_et0`, `validate_real_ncbi_16s`, `validate_iris_seismic`, `validate_nucleus_stack`) require a live NestGate socket. Without it:
- Foundation data chains cannot be automated
- Thread 6 targets (which overlap ag/soil/hydrology) cannot be validated against real data
- `run_all_baselines.sh` Phase 2 biomeOS section will always skip

**Action needed**: NestGate team P1 rebuild to expose HTTP content-addressed storage API. Minimum surface: `PUT /content`, `GET /content/{hash}`, `GET /health`.

### barraCuda: domain-genomics Feature Discovery

The `barracuda-gpu` feature in groundSpring now correctly includes `barracuda/domain-genomics`. Other springs using Gillespie SSA, Wright-Fisher drift, or any `barracuda::ops::bio` module should verify their feature sets include this flag. The module is gated behind `#[cfg(feature = "domain-genomics")]` in barraCuda `ops/mod.rs:523`.

### PRNG Phase 2b (barraCuda Team)

GPU seed stride alignment (`Xorshift64` vs `Xoshiro128**`) remains a barraCuda deliverable. groundSpring has documented value-level CPU parity (confirmed via Kokkos comparison) but GPU parity is timing-only due to PRNG stream mismatch. No blocking action for groundSpring.

### coralReef: Shader Compilation

`shader.compile.wgsl` method needed for sovereign shader pipeline. Currently metalForge includes shaders (`anderson_lyapunov.wgsl`, `anderson_lyapunov_f32.wgsl`) but compilation is via wgpu native — not through coralReef's sovereign compiler. This blocks full Tier 4 sovereign compute dispatch.

### ToadStool: compute.dispatch Expansion

`compute.dispatch` is wired (neuralSpring exemplar) but groundSpring's metalForge workload routing still uses local dispatch. Evolution target: ToadStool-mediated GPU/NPU assignment via capability query.

## Patterns for Other Springs

### Modular GuideStone (groundSpring Pattern)
Large validation binaries (>800L) should be refactored into library modules with a thin orchestrator binary. groundSpring's guidestone went from 833→128 lines with 5 NUCLEUS layer modules (`bare`, `tower`, `node`, `nest`, `cross`). Each module is testable independently and the orchestrator just sequences them.

### metalForge barracuda Decoupling (Tier 4 Pattern)
To make barracuda optional in a workspace member:
1. `barracuda = { ..., optional = true }` in Cargo.toml
2. Add `pollster` for async wgpu futures (replaces `barracuda::device::test_pool::tokio_block_on`)
3. Feature-gate barracuda-specific APIs with `#[cfg(feature = "barracuda-gpu")]`
4. Add `vulkan` to wgpu features for standalone GPU probing
5. Verify `cargo check --workspace` succeeds without barraCuda source

### Foundation Seeding Pattern
Follow airSpring exemplar: dated validation run directory, braid.json (sweetGrass schema), PROVENANCE_MANIFEST.md with per-target pass/fail table. Binary BLAKE3 hash in both files. Commit targets file if untracked.

### Benchmark Correctness
Capture actual subprocess exit codes in benchmark scripts — don't hardcode `pass=True`. Wire all defined experiment lists (including NPU) into `main()` with hardware detection guards.

## NUCLEUS Composition Patterns

### neuralAPI Deployment
groundSpring exposes 16 `measurement.*` methods via JSON-RPC when running as a biomeOS primal (`--features biomeos`). These methods are registered via `method.register` during startup. neuralAPI consumers discover them via `capability.list` and invoke them through the standard `{method_name}(params)` JSON-RPC call.

### Workload TOML Convention
Use `${ENV_VAR:-<default>}` shell-default patterns in `working_dir` and `command` fields. Bare `$VAR` references are NOT expanded by toadStool.

### Validation Binary Output
Foundation's `foundation_validate.sh` counts `[OK]` / `[FAIL]` lines from workload stdout. Keep validation binaries compatible with this contract (most groundSpring validators use `PASS` / `FAIL` — worth aligning upstream if this becomes the canonical format).

## Deep Debt Audit Results (V133)

| Area | Status |
|------|--------|
| `unsafe` code | Zero — `#![forbid(unsafe_code)]` everywhere |
| `todo!()` / `unimplemented!()` | Zero |
| Mocks in production | Zero |
| `#[allow(dead_code)]` | Zero |
| `Box<dyn Error>` | Zero |
| TODO/FIXME/HACK/XXX | Zero in Rust |
| Files >800 lines | Zero (largest: `validate.rs` at 667) |
| Production panics | Zero (fixed in V133) |
| External deps | All industry-standard (thiserror, serde, clap, tracing, wgpu, pollster) |

## Next Evolution Targets

1. **Full data chains**: NestGate live → Exp 029-032 validated against real NOAA/NCBI/IRIS data
2. **Thread 6 groundSpring targets**: Add groundSpring-specific targets to `thread06_ag_targets.toml` for soil physics / sensor calibration quantities we own
3. **Sovereign shader dispatch**: coralReef compile → toadStool dispatch → metalForge validate
4. **GuideStone L5**: Cross-spring certification tests (Layers 4-5)
5. **fetch_sources.sh**: Extend to support `ag`/`anderson` thread filters
