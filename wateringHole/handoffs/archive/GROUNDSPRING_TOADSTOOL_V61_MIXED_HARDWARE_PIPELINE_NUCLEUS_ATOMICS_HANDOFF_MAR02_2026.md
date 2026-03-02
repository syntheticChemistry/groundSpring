# groundSpring → ToadStool V61 Handoff: Mixed-Hardware Pipeline + NUCLEUS Atomics

**Date**: March 2, 2026
**From**: groundSpring (V61)
**To**: ToadStool / BarraCUDA team
**ToadStool pin**: S71+++ (`8dc01a37`)
**License**: AGPL-3.0-or-later
**Supersedes**: V60 (hotSpring Cross-Spring Absorption)

---

## Executive Summary

- **Mixed-hardware pipeline dispatch**: New `pipeline.rs` module chains workloads
  across NPU → GPU → CPU substrates with automatic fallback and transfer cost
  modeling. Validates the infrastructure for ToadStool's cross-substrate streaming.
- **`PCIe` topology**: New `topology.rs` models device adjacency and bandwidth tiers
  (NvLink, `PCIe` P2P, `PCIe` host bounce, `PCIe` low). Enables NPU↔GPU P2P
  detection — the foundation for bypassing CPU round-trips in mixed hardware.
- **NUCLEUS atomic types**: `TowerAtomic`, `NodeAtomic`, `NestAtomic`, `FullNucleus`
  model the biomeOS compositions with sovereign degradation (Full → Node+Nest →
  Node → Tower → Sovereign).
- **Fallback chains**: `dispatch::fallback_chain()` returns ordered substrate lists
  for graceful runtime degradation (GPU NativeF64 → GPU → NPU → CPU).
- **668 workspace tests** (+35 from V60), 42/42 mixed-hardware validation checks,
  120 metalForge tests (up from 85), all quality gates green.
- **61 active delegations**: unchanged — V61 builds infrastructure, not new delegations.
- **Deep idiomatic debt pass**: 13 clippy fixes, iterator chains, `serde_json::json!`
  macro migration, `mul_add` for suboptimal flops, `#[expect]` with reasons, 17 new
  unit tests for intermediate functions.

---

## Part 1: New metalForge Infrastructure

### 1.1 `topology.rs` — PCIe Topology and Device Adjacency

Models interconnect between substrates for transfer cost estimation.

```
| Tier       | Bandwidth  | Latency | Example                          |
|------------|-----------|---------|----------------------------------|
| Local      | ∞         | 0       | Same device (GPU→GPU shader)     |
| NvLink     | 300 GB/s  | ~1µs   | Multi-GPU NvLink bridge          |
| PciePeer   | 15.8 GB/s | ~5µs   | PCIe 4.0 x16 P2P (GPU↔NPU)      |
| PcieHost   | 15.8 GB/s | ~50µs  | PCIe 4.0 x16 via CPU (bounce)   |
| PcieLow    | 0.5 GB/s  | ~100µs | PCIe 2.0 x1 (AKD1000)           |
| Network    | varies    | ~1ms   | LAN via NUCLEUS node             |
```

Key APIs:
- `Topology::infer(substrates)` — build topology from inventory
- `topology.best_link(from, to)` — lowest-latency path
- `topology.has_p2p(from, to)` — P2P capable?
- `topology.transfer_time_us(from, to, bytes)` — estimated cost

**ToadStool relevance**: When ToadStool streams data between GPU and NPU,
this topology informs whether to use P2P DMA or bounce through host memory.
AKD1000 at `PCIe` 2.0 x1 means NPU→GPU is `PcieLow` (500 MB/s) — small
classification outputs (256 bytes) transfer in ~100µs, but large feature
tensors should stay on-device.

### 1.2 `pipeline.rs` — Multi-Stage Pipeline Dispatch

Chains workloads across substrates with data flow modeling:

```
Stage 0: NPU (int8 classify)  →  regime labels [0,1,2]
Stage 1: GPU (f64 Lyapunov)   ←  regime labels → full spectrum
Stage 2: CPU (provenance)     ←  spectrum → stored results
```

Key types:
- `Pipeline::new(name).stage(s1).stage(s2)` — builder pattern
- `pipeline.plan(substrates, topology)` → `ResolvedPipeline`
- `FallbackPolicy::Degrade | Skip | Fail` — per-stage failure handling
- `TransferStrategy::PeerToPeer | HostBounce | None` — resolved transfers

**ToadStool relevance**: This models the dispatch pattern ToadStool will
eventually execute — accepting a pipeline definition via biomeOS
`compute.submit` and streaming between GPU/NPU buffers.

### 1.3 `dispatch.rs` — Fallback Chain Evolution

New `fallback_chain()` alongside existing `route()`:

```rust
pub fn fallback_chain<'a>(
    workload: &Workload,
    substrates: &'a [Substrate],
) -> Vec<Decision<'a>>
```

Returns ordered list: preferred → GPU (NativeF64 first) → NPU → CPU.
Caller tries substrates in order; if first fails, next is ready.

### 1.4 `atomic.rs` — NUCLEUS Atomic Types

| Atomic | Composition | Capabilities |
|--------|-------------|-------------|
| `TowerAtomic` | BearDog + Songbird | SecureIpc |
| `NodeAtomic` | Tower + ToadStool + Inventory + Topology | + ComputeDispatch, NpuInference, PipelineOrchestration |
| `NestAtomic` | Tower + NestGate | + DataStorage, LiveData |
| `FullNucleus` | All primals + Squirrel | All capabilities |

Sovereign degradation: Full NUCLEUS → Node+Nest → Node → Tower → Sovereign (local only).

`NodeAtomic` hosts the metalForge inventory and topology, enabling
`node.plan_pipeline(pipeline)` — ToadStool's future entry point for
mixed-hardware dispatch.

---

## Part 2: Deep Debt Pass (V61 pre-work)

### Clippy / Idiomatic Fixes
- 13 clippy errors resolved (anderson, esn, fao56, freeze_out, lanczos)
- `needless_return`, `doc_markdown`, `assertions_on_constants`, `suboptimal_flops`,
  `cast_lossless`, `items_after_statements`, `must_use_candidate`, `redundant_clone`
- Iterator chain refactoring: `flat_map` in `spectral_recon`, `mul_add` in `wdm`
- `serde_json::json!` migration in `nestgate.rs` (5 manual `format!` → typed macros)
- `.or_else()` chaining in `nestgate::fetch_cached`
- `try_f64_field`, `try_usize_field`, `try_str_field` Result-based API in validate lib

### Provenance / Hardcoding
- Provenance headers added to 4 NUCLEUS validation binaries
- Hardcoded `/tmp/` path → `std::env::temp_dir()` in `validate_nucleus_stack`
- `unwrap()` → `.expect("...")` guards in `benchmark_cross_spring`, `validate_nucleus_stack`
- Primal socket names → named constants in `validate_nestgate_ncbi`

### Coverage
- 17 new unit tests (fao56 intermediates, npu dequantize, validate try_* API)

---

## Part 3: Validation Summary

```
Python Phase 0:        375/375 PASS (+3 skipped)
Rust Phase 1 (core):   292/292 PASS (27 binaries)
Rust workspace tests:  668/668 PASS (default)
  groundspring lib:    374 tests
  groundspring-validate: 13 tests  
  groundspring-forge:  120 tests (+35 new)
  parity tests:        109 tests
  integration tests:   52 tests
metalForge validation: 130/132 PASS (2 expected NPU failures)
Mixed-hardware:        42/42 PASS (NEW)
Quality gates:         clippy (pedantic+nursery), fmt, doc — all PASS
```

---

## Part 4: Absorption Guidance for ToadStool

### New: Pipeline Dispatch Protocol

ToadStool should consider absorbing the pipeline concept for its streaming
dispatch. The protocol in `pipeline.rs` maps directly to ToadStool's buffer
management:

1. `Pipeline` → ToadStool accepts pipeline definition via `compute.submit`
2. `Stage` → Each stage maps to a barracuda dispatch + buffer allocation
3. `TransferStrategy::PeerToPeer` → ToadStool keeps buffers on-device
4. `TransferStrategy::HostBounce` → ToadStool uses `wgpu::Buffer::map_read()`

### New: Topology-Aware Batch Sizing

`AdaptiveBatch::for_gpu()` already sizes batches to fit VRAM. The new
`Topology::transfer_time_us()` enables ToadStool to factor in transfer
overhead when choosing between:
- Large batch on slow NPU → GPU transfer (optimize for fewer transfers)
- Small batch on fast GPU → GPU transfer (optimize for latency)

### Existing: 6 Not-Yet-Delegated Items (unchanged from V60)

| Module | Target | Blocker |
|--------|--------|---------|
| `gillespie::birth_death_ssa` | `GillespieGpu` | GPU-only, no CPU fallback |
| `drift::wright_fisher_fixation` | `WrightFisherGpu` | GPU dispatch, needs device |
| `spectral_recon::cholesky_solve` | `linalg::CholeskyF64` | GPU linalg, needs device |
| `transport::tridiag_eigh` | `linalg::eigh_f64` | GPU eigenvectors, needs device |
| `rarefaction::multinomial_sample` | `BatchedMultinomialGpu` | Signature mismatch |
| `prng::Xorshift64` | `PrngXoshiro` | Baseline regeneration needed |

### New: DriftMonitor + ClassificationUncertainty (from V60)

Future barracuda candidates:
- `drift::DriftMonitor` — could be GPU-accelerated for large populations
- `esn::ClassificationUncertainty` — softmax + entropy could batch on GPU
- `esn::detect_concept_edges()` — LOO cross-validation is embarrassingly parallel

---

## Part 5: Cross-Spring Learnings

### For wetSpring
- Mixed-hardware pipeline pattern applies directly to wetSpring's LC-MS/PFAS
  workflows (NPU peak detection → GPU fragmentation modeling → CPU provenance)
- `Topology::has_p2p()` is relevant for wetSpring's real-time nanopore workflows

### For hotSpring
- `FullNucleus::degradation_level()` models the 4-layer brain architecture
  (RTX 3090 + Titan V + CPU + NPU) as atomic compositions
- Fallback chains formalize hotSpring's existing GPU → CPU fallback patterns

### For ToadStool
- `Pipeline` maps to ToadStool's streaming dispatch model
- `BandwidthTier` maps to ToadStool's `BandwidthTier` (S62 evolution)
- `NodeAtomic` is ToadStool's view of itself within a NUCLEUS node

---

## Quality Certification

| Gate | Status |
|------|--------|
| `cargo fmt --check` | PASS |
| `cargo clippy --workspace --all-targets` | PASS (0 warnings — pedantic + nursery) |
| `cargo clippy --features barracuda-gpu` | PASS (0 warnings) |
| `cargo doc --workspace --no-deps` | PASS (0 warnings) |
| `cargo test --workspace` | 668/668 PASS |
| `cargo test --features barracuda` | 668/668 PASS |
| Validation binaries (default) | 292/292 PASS |
| metalForge validation | 130/132 PASS |
| Mixed-hardware validation | 42/42 PASS |
| `python3 -m pytest tests/` | 375/375 PASS (+3 skipped) |
| Unsafe code | 0 (workspace lint) |
| TODO/FIXME | 0 |
| Production mocks | 0 |
