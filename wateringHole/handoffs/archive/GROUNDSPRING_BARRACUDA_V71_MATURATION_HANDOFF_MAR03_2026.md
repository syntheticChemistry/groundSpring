# SPDX-License-Identifier: AGPL-3.0-only

# groundSpring → barraCuda Handoff: V71 / Ecosystem Maturation

**Date**: March 3, 2026
**From**: groundSpring V71
**To**: barraCuda / toadStool / ecoPrimals ecosystem
**barraCuda pin**: `f6895ca` (v0.3.1)
**toadStool pin**: S93 (`9319668d`)
**License**: AGPL-3.0-only
**Covers**: V70 → V71 (barraCuda 0.3.1 maturation, toadStool S93 untangle confirmation)

---

## Executive Summary

- **barraCuda matured**: v0.2.1 → v0.3.1. Complete toadStool untangle (toadstool_integration.rs deleted, npu/ops deleted, zero cross-deps). 2,965 tests. Version alignment resolved. tarpc/JSON-RPC parity.
- **toadStool matured**: S87 → S93. Embedded barracuda deprecated and rewired to standalone. D-DF64 ownership transferred to barraCuda. Root docs refocused on owned work.
- **groundSpring validated**: 786+ tests pass with zero changes against barraCuda 0.3.1. Zero breaking impact.
- **Ecosystem synchronized**: 16 new wateringHole handoffs absorbed, documenting the full budding-to-untangle arc.

---

## Part 1: barraCuda 0.3.1 Assessment

### What changed (b53c3de → f6895ca)

| Area | Change |
|------|--------|
| Version | 0.2.1 → 0.3.1 (aligned across Cargo.toml, CHANGELOG, spec) |
| toadStool deps | **ZERO** — `toadstool_integration.rs` deleted, `toadstool` feature removed |
| akida-driver deps | **ZERO** — `npu/ops/*` and `npu/ml_backend.rs` deleted |
| NPU stubs | `npu/constants.rs`, `event_codec.rs`, `npu_executor.rs` remain (detection + executor shell) |
| Tests | 2,831 → 2,965 (+134 new: cpu_executor dispatch, benchmarks, vendor, cubic_spline, IPC E2E) |
| tarpc parity | `fhe_ntt`, `fhe_pointwise_mul`, `compute_dispatch`, `tensor_create` signatures now match JSON-RPC |
| New modules | `unified_hardware/mod.rs` (233 lines): traits, discovery, scheduler, transfer cost modelling |
| New modules | `device/vendor.rs`: canonical GPU vendor ID constants |
| New modules | `benchmarks/operations.rs`, `benchmarks/report.rs`: structured benchmark infrastructure |
| println cleanup | 14 `println!` → `tracing::info!` |
| blake3 | `features = ["pure"]` available (no C SIMD dep) |

### Breaking changes in 0.3.1

| Change | groundSpring Impact |
|--------|-------------------|
| `MatmulResult` fields renamed | **None** — not used |
| `FheResult` split into `FheNttResult` / `FhePointwiseMulResult` | **None** — not used |
| `DispatchResult` fields changed | **None** — not used |
| `compute_dispatch` tarpc signature changed | **None** — not used via tarpc |
| FHE tarpc signatures expanded | **None** — not used |
| `tensor_create` tarpc signature expanded | **None** — not used |

### Maturity classification

**Nascent-Stable**: barraCuda builds in CI with sourDough checked out. The local buildability gap (requires `../sourDough` sibling) is the only remaining friction. 2,965 tests pass. Version-aligned at 0.3.1. Zero cross-deps on toadStool. hotSpring validated 716/716 tests on first path swap.

---

## Part 2: toadStool S88-S93 Assessment

| Session | Key Change | groundSpring Impact |
|---------|-----------|-------------------|
| S88 | Cross-spring absorption, API gaps, CI hardening | No direct impact |
| S89 | barraCuda budding completion, document | Confirms architecture |
| S90-S92 | Deep audit, sovereignty evolution, debris cleanup — 2,966 files changed, rewired to standalone barraCuda | No direct impact |
| S93 | Clean root docs, refocus on owned work | Clarity on toadStool scope |
| S93 | D-DF64 ownership transferred to barraCuda team | Precision strategy now barraCuda-owned |

### Architecture confirmed (S89 handoff)

```
Springs ──> barraCuda    (direct cargo dep — WHAT to compute)
toadStool ──> barraCuda  (as compute backend — WHERE/HOW to compute)
barraCuda ──> sourDough  (primal traits only)
```

barraCuda has **zero** dependencies on toadStool. The untangle is complete.

### D-DF64 transfer (S93)

barraCuda now owns:
- Precision strategy selection (`Fp64Strategy::Native` vs `Hybrid` vs `Concurrent`)
- `df64_rewrite` as default path for consumer GPUs
- Cross-precision validation (f64 vs df64 vs f32)
- Per-op DF64 shader variants (25 hand-written WGSL files)

toadStool continues to provide raw hardware capability data that barraCuda consumes.

---

## Part 3: wateringHole Sync

### New handoffs absorbed (16)

| Handoff | Key Content |
|---------|------------|
| `BARRACUDA_S89_EXTRACTION_COMPLETE_MAR02_2026` | barraCuda live as standalone, 2,832 tests, quality gates clean |
| `BARRACUDA_S89_UNTANGLE_AND_HANDOFF_MAR03_2026` | Complete untangle, architecture doc, migration guide, feature flag map |
| `TOADSTOOL_S93_DF64_HANDOFF_MAR03_2026` | D-DF64 ownership transfer to barraCuda |
| `TOADSTOOL_S88_BARRACUDA_PRIMAL_BUDDING_PROPOSAL_MAR02_2026` | Budding architecture proposal |
| `TOADSTOOL_BARRACUDA_S86_COMPUTE_DISPATCH_EVOLUTION_HANDOFF_MAR02_2026` | ComputeDispatch 111→144 ops |
| `TOADSTOOL_BARRACUDA_S87_DEEP_DEBT_EVOLUTION_HANDOFF_MAR02_2026` | FHE fix, unsafe audit |
| `TOADSTOOL_BARRACUDA_S88_SPRING_ABSORPTION_HANDOFF_MAR02_2026` | Spring absorption patterns |
| `HOTSPRING_V0615_DEEP_DEBT_TOADSTOOL_ABSORPTION_HANDOFF_MAR02_2026` | hotSpring deep debt cleanup |
| `HOTSPRING_V0615_NAUTILUS_EVOLUTION_ECOSYSTEM_MAR01_2026` | Nautilus evolution |
| `HOTSPRING_V0615_TOADSTOOL_S78_SYNC_HANDOFF_MAR02_2026` | hotSpring-toadStool sync |
| Others | Spring-specific evolution and absorption handoffs |

### PRIMAL_REGISTRY status

groundSpring is listed under Domain Validation Primals in the registry. The registry still describes toadStool's embedded barracuda count (687 WGSL shaders) — this should be updated to reflect the standalone barraCuda and its current 767 WGSL shaders. No action needed from groundSpring side.

---

## Part 4: Ecosystem Guidance

### For other Springs migrating to standalone barraCuda

1. **Path swap only**: Change `phase1/toadStool/crates/barracuda` → `barraCuda/crates/barracuda` in Cargo.toml
2. **Zero code changes**: All `use barracuda::*` imports work identically (confirmed by hotSpring 716/716, groundSpring 786+/786+)
3. **akida-driver stays with toadStool**: This is architecturally correct. toadStool owns hardware ("WHERE and HOW") — including NPU silicon, compilation for NPU, and dispatch. akida-driver is hardware, not math.
4. **sourDough**: barraCuda requires `../sourDough` sibling for standalone builds; clone `ecoPrimals/sourDough` if building barraCuda directly
5. **0.3.1 breaking changes**: Only affect tarpc RPC types (MatmulResult, FheResult, DispatchResult, compute_dispatch, fhe_ntt/fhe_pointwise_mul, tensor_create signatures). If you don't use tarpc dispatch directly, zero impact.

### Architecture: precision path vs hardware path

The NPU pipeline crosses both primals cleanly:

```
barraCuda: fp64 math → precision quantization (fp64 → df64 → f32 → int8 → int4)
                 │
                 │  "WHAT to compute, at what precision"
                 │
toadStool: akida-driver → compile for NPU → dispatch to AKD1000 silicon
                 │
                 │  "WHERE to run it, HOW to compile it"
                 │
Hardware:  BrainChip AKD1000 (160 NPUs, ~48µs inference, DMA round-trip ~51µs)
```

barraCuda owns the math and the precision path. It knows how to go from fp64 to int4 — that's a math/quantization concern. toadStool owns the hardware driver, knows the NPU silicon topology, compiles the quantized model for the specific NPU, and dispatches it. Neither invades the other's domain.

### For barraCuda team

1. **Precision quantization path** (P1): barraCuda's fp64 → int4 path enables NPU inference. Expose quantization utilities (int4/int8 with scale+zero-point) so Springs can prepare data for NPU dispatch without manual quantization.
2. **tarpc version alignment** (P2): groundSpring uses tarpc 0.35, barraCuda uses 0.34. No conflict currently.
3. **unified_hardware module**: The new `ComputeScheduler`, `TransferCost`, and `HardwareDiscovery` traits are evolution candidates for metalForge integration when ready.
4. **vendor constants**: `device::vendor::VENDOR_*` could replace raw hex comparisons in metalForge GPU routing.

### For toadStool team

1. **Embedded barracuda deprecated**: Confirmed. Springs should not reference `phase1/toadstool/crates/barracuda` anymore.
2. **akida-driver is yours permanently**: NPU hardware compilation and dispatch is toadStool's domain. Springs that need NPU access depend on toadStool for akida-driver — this is correct.
3. **D-DF64 transfer acknowledged**: Precision strategy is barraCuda's domain. toadStool provides hardware capability data that barraCuda consumes for strategy selection.
4. **NPU compilation**: toadStool knows how to compile barraCuda's quantized math for NPU targets. This is the "WHERE and HOW" layer working as designed.

---

## Part 5: Quality Gates

| Gate | Status |
|------|--------|
| `cargo build --workspace` | PASS (barraCuda locked 0.2.0 → 0.3.1) |
| `cargo fmt --check` | PASS |
| `cargo clippy --all-features -- -W clippy::pedantic -W clippy::nursery` | PASS (zero warnings) |
| `cargo test --workspace` | PASS (786+ tests, 0 failures, all validation binaries green) |
| Zero unsafe | PASS |
| Zero TODO | PASS |
| Zero `.unwrap()` | PASS |

---

## Part 6: Provenance

| Metric | V70 | V71 | Change |
|--------|-----|-----|--------|
| barraCuda pin | `b53c3de` (v0.2.1) | `f6895ca` (v0.3.1) | +0.1.0 minor, tarpc parity + untangle |
| toadStool pin | S87 (`2dc26792`) | S93 (`9319668d`) | +6 sessions, full untangle |
| Active delegations | 81 | 81 | Unchanged (no new primitives in 0.3.1 for groundSpring) |
| groundSpring tests | 786 | 786+ | Unchanged |
| Python parity | 28/28 | 28/28 | Unchanged |
| Debt | Zero | Zero | Maintained |
| barraCuda tests upstream | 2,831 | 2,965 | +134 |

---

## Part 7: Next Actions

### P0 — None

All builds clean, all tests pass, no breaking changes, no action required.

### P1 — Evolution candidates (when barraCuda releases)

- **`unified_hardware::ComputeScheduler`**: Could replace metalForge's manual substrate routing with barraCuda's trait-based scheduling
- **`device::vendor::VENDOR_*` constants**: Single source of truth for GPU vendor IDs
- **`benchmarks::report::BenchmarkReport`**: Structured benchmark output for validation binaries
- **Quantization utilities**: barraCuda's precision path (fp64 → int4) for NPU-ready data preparation

### P2 — Ecosystem

- **PRIMAL_REGISTRY**: groundSpring should have its own entry (currently only mentioned in toadStool's "Five-Spring ingestion" note)
- **sourDough clone**: If local barraCuda development needed, clone `ecoPrimals/sourDough` as sibling

### Clarification: akida-driver ownership

The V70 handoff incorrectly listed "akida-driver budding into barraCuda" as P1. This is corrected in V71. akida-driver is hardware, not math — it belongs permanently with toadStool. groundSpring's akida-driver path to `phase1/toadstool/crates/neuromorphic/akida-driver` is architecturally correct.
