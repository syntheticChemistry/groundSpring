# groundSpring V89 — Rewire to barraCuda/toadStool/coralReef Evolution

**Date**: March 6, 2026
**From**: groundSpring V89
**To**: barraCuda, toadStool, coralReef
**Pins**: barraCuda `ed82625`, toadStool S128b `22d1a2c7`, coralReef Phase 9 `b7f8ab4`

## Executive Summary

groundSpring rewired to absorb all breaking changes from barraCuda, toadStool, and
coralReef evolution since V88:

- **tarpc 0.35 to 0.37** (aligned with barraCuda workspace)
- **barracuda::ops GPU-gated**: pub mod ops moved behind cfg(feature = "gpu");
  groundSpring rarefaction::multinomial_sample re-gated from barracuda to barracuda-gpu
- **domain-esn feature**: barraCuda esn_v2 now requires domain-esn; wired into
  groundSpring barracuda-gpu feature
- **Rust 2024 unsafe model**: set_var/remove_var are unsafe; workspace lint changed from
  forbid to deny, all lib.rs files assert #![forbid(unsafe_code)]
- **8 collapsible_if** warnings resolved using Rust 2024 let chains
- **Unfulfilled lint expectation** removed from ipc.rs (tarpc 0.37 no longer triggers
  clippy::too_many_arguments)

## Quality Gates

| Gate | Status |
|------|--------|
| cargo fmt --check | PASS |
| cargo clippy --workspace --all-features | PASS (zero warnings) |
| cargo check --workspace --all-features | PASS |
| cargo test (lib, CPU-only barracuda) | PASS (476/476) |
| cargo test (integration, CPU-only) | PASS (24/24) |
| Python provenance (test_baseline_integrity.py) | PASS (261/261) |
| GPU tests (--all-features) | BLOCKED (see below) |

## GPU Test Regression (barraCuda ed82625)

Six GPU-dispatched tests fail when barracuda-gpu is enabled:

| Test | Symptom |
|------|---------|
| stats::metrics::std_dev_known_value | population sigma returns 0 instead of 2.0 |
| stats::agreement::r2_constant_observation | R-squared not approx 0 for constant obs |
| stats::agreement::r2_mean_model_is_zero | R-squared not approx 0 for mean model |
| stats::agreement::nse_mean_model_is_zero | NSE not approx 0 for mean model |
| stats::correlation::pearson_r_squared_matches_r2_for_linear | r-squared != R-squared |
| rarefaction::rarefaction_at_depth_convergence | genera_mean off by > 0.5 |

**Root cause**: Commit ed82625 ("Wire Fp64Strategy into SumReduceF64 and
VarianceReduceF64") introduced a regression in GPU reduce ops. On RTX 4070 (Hybrid
Fp64Strategy), VarianceReduceF64::population_std and SumReduceF64::mean return
incorrect values. All 500 CPU tests pass.

**groundSpring delegation pattern is correct**: if let Some(v) = gpu_fn() with CPU
fallback. The issue is that the GPU ops return Ok(wrong_value) rather than Err, so
the fallback never triggers.

### Evolution Request to barraCuda (P0 - Blocks GPU Parity)

1. Verify SumReduceF64::mean and VarianceReduceF64::population_std on Hybrid devices
   (RTX 40xx / RDNA3) after Fp64Strategy wiring
2. If the DF64 shader path is the issue, consider returning Err(ShaderCompilation) when
   DF64 produces incorrect results rather than returning zero

## Absorption Report

### Absorbed from barraCuda

| Change | groundSpring Action |
|--------|-------------------|
| pub mod ops behind cfg(feature = "gpu") | multinomial_sample re-gated to barracuda-gpu |
| esn_v2 behind domain-esn feature | Added barracuda/domain-esn to barracuda-gpu feature |
| tarpc 0.37 workspace bump | Upgraded from 0.35 |
| device_arc to device_clone | Not used by groundSpring (no change needed) |
| inner_arc removed | Not used by groundSpring |
| Maintain::Wait to PollType::Wait | Already migrated in V77 |
| etcetera removed | Not used by groundSpring |
| wgpu 22 to 28 | Already migrated in V77 |
| entry_point: Some("main") | Already correct |
| set_bind_group(0, Some(&bg), &[]) | Already correct |
| Bounded poll timeout | No change needed |
| NVK SPIR-V exclusion | No change needed (WGSL-only) |

### Absorbed from toadStool (S97 to S128b)

| Change | groundSpring Action |
|--------|-------------------|
| HardwareFingerprint.sovereign_binary_capable | Documented; no code dependency |
| GpuAdapterInfo.precision_routing() PrecisionRoutingAdvice | Documented for future use |
| f64_shared_memory_reliable (always false naga bug) | Documented; aligns with V84 finding |
| SubstrateType expanded (8 variants) | Documented; no code dependency |
| Shader compilation IPC (shader.compile.*) | Documented for future coralReef integration |

### Absorbed from coralReef (Phase 6 to Phase 9)

| Change | groundSpring Action |
|--------|-------------------|
| Sovereign pipeline complete | Documented; integration via coral-gpu future |
| Zero C dependencies | No change needed |
| compile_wgsl(), compile_spirv() APIs | Documented for future use |
| 13-tier tolerance alignment (tol::ANALYTICAL) | Already aligned |

## Evolution Requests

### To barraCuda

| Priority | Request |
|----------|---------|
| **P0** | Fix SumReduceF64/VarianceReduceF64 regression on Hybrid devices (ed82625) |
| **P1** | Expose multinomial_sample_cpu outside cfg(feature = "gpu") -- it is a CPU function |
| **P2** | PRNG alignment: xorshift64 to xoshiro128** migration path (Tier B blocker) |
| **P2** | GPU test coverage patterns (mock/stub device for CI) |

### To toadStool

| Priority | Request |
|----------|---------|
| **P2** | Initialize log subscriber (e.g., env_logger) for groundSpring warnings |

### To coralReef

No changes requested. Phase 9 sovereign pipeline is aligned.

## Delegation Inventory (unchanged from V88)

93 active delegations (56 CPU + 37 GPU). No new delegations in V89.

## What V89 Changed (file summary)

| File | Change |
|------|--------|
| Cargo.toml (workspace) | unsafe_code lint: forbid to deny |
| crates/groundspring/Cargo.toml | tarpc 0.37, barracuda-gpu += domain-esn |
| crates/groundspring/src/lib.rs | #![forbid(unsafe_code)] |
| crates/groundspring-validate/src/lib.rs | #![forbid(unsafe_code)] |
| metalForge/forge/src/lib.rs | #![forbid(unsafe_code)] |
| crates/groundspring/src/ipc.rs | Removed unfulfilled #[expect] |
| crates/groundspring/src/rarefaction.rs | multinomial_sample gate: barracuda to barracuda-gpu |
| crates/groundspring/src/fao56/mod.rs | Collapsible if (let chain) |
| crates/groundspring/src/biomeos/discovery.rs | Collapsible if (let chain) |
| crates/groundspring-validate/src/validate_real_ghcnd_et0.rs | Collapsible if + re-indent |
| crates/groundspring-validate/src/validate_iris_seismic.rs | Collapsible if + re-indent |
| crates/groundspring-validate/src/validate_anderson.rs | Collapsible if |
| metalForge/forge/src/bin/validate_nucleus_pipeline.rs | Collapsible if (triple let chain) |
| crates/groundspring/tests/biomeos_integration.rs | unsafe for set_var/remove_var |

---

*This handoff is unidirectional: groundSpring to ecosystem. groundSpring has no reverse
dependencies on toadStool, coralReef, or other springs.*
