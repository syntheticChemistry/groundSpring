# groundSpring → barraCuda Handoff: V70 / Budding Evolution

**Date**: March 3, 2026
**From**: groundSpring V70
**To**: barraCuda / ecoPrimals ecosystem
**barraCuda pin**: `b53c3de` (v0.2.1)
**ToadStool pin**: S87 (`2dc26792`) — historical, dependency now standalone
**License**: AGPL-3.0-only
**Covers**: V69 → V70 (barraCuda budding, S80-S87 absorption)

---

## Executive Summary

- **Budding complete**: barracuda dependency rewired from `phase1/toadstool/crates/barracuda` to standalone `barraCuda/crates/barracuda`
- **Zero breaking changes**: None of barraCuda 0.3.0 API changes affect groundSpring
- **5 new delegations wired** from S80-S87 evolution (81 total, up from 76)
- **786 tests** passed, 0 failed; zero unsafe / zero TODO / zero `.unwrap()` / zero `#[allow]` without reason
- **akida-driver** remains at `phase1/toadstool` (NPU support not yet budded)

---

## Part 1: The Budding

### What happened

barraCuda has budded from `phase1/toadstool/crates/barracuda` into a standalone primal at `ecoPrimals/barraCuda/`. Like yeast budding, the daughter cell (barraCuda) carries the full genome (all 767 WGSL shaders, 956 .rs files) and can now evolve independently.

### Why

- **Complexity isolation**: toadStool's orchestration layer (hardware selection, multi-framework routing) was entangled with barraCuda's math engine during development
- **Independent evolution**: barraCuda needs to evolve its math/shader stack without coordinating toadStool orchestration changes
- **Cleaner dependency graph**: Springs depend on barraCuda directly for math; toadStool orchestrates above

### Dependency change

```
Before:
  groundSpring → phase1/toadstool/crates/barracuda  (embedded)
  groundSpring → phase1/toadstool/crates/neuromorphic/akida-driver  (NPU)

After:
  groundSpring → barraCuda/crates/barracuda  (standalone primal)
  groundSpring → phase1/toadstool/crates/neuromorphic/akida-driver  (NPU, unchanged)
```

### Files changed

| File | Change |
|------|--------|
| `crates/groundspring/Cargo.toml` | barracuda path: `../../../phase1/toadstool/crates/barracuda` → `../../../barraCuda/crates/barracuda` |
| `metalForge/forge/Cargo.toml` | barracuda path: same change |

### Breaking changes audit

None of barraCuda 0.3.0 breaking changes affect groundSpring:

| barraCuda 0.3.0 Change | groundSpring Impact |
|------------------------|-------------------|
| `read_f64_raw` / `read_i32_raw` take `&WgpuDevice` | Not used |
| `sparsity_sampler` requires `F: Fn + Sync` | Not used |
| `PppmGpu` API changes | Not used |
| `ComputeGraph::new` requires `Arc<WgpuDevice>` | Not used |
| MSRV 1.80 → 1.87 | Compatible (groundSpring uses 2021 edition) |

---

## Part 2: S80-S87 Absorption

### New delegations wired (5)

| Delegation | barraCuda API | groundSpring Location | Session |
|------------|--------------|----------------------|---------|
| `StatefulPipeline<WaterBalanceState>` | `barracuda::pipeline::StatefulPipeline` | `fao56/pipeline.rs::seasonal_multi_day` | S80 |
| `BatchedEncoder` | `barracuda::device::batched_encoder::BatchedEncoder` | `metalForge/forge/src/lib.rs` (documented, available) | S80 |
| `batched_nelder_mead_gpu` | `barracuda::optimize::batched_nelder_mead_gpu` | `freeze_out.rs::nelder_mead_multi_start` | S80 |
| Device-lost resilience | `BarracudaError::is_device_lost()` | `metalForge/forge/src/harness.rs::check_gpu_resilient` | S87 |
| Spectral diagnostics | `barracuda::spectral::{spectral_bandwidth, spectral_condition_number, classify_spectral_phase}` | `anderson.rs::spectral_diagnostics` | S79 |

### S80-S87 evolution timeline

| Session | Key Changes | groundSpring Impact |
|---------|------------|-------------------|
| S79 | `SpectralAnalysis`, `spectral_bandwidth`, `spectral_condition_number`, `classify_spectral_phase` | New `spectral_diagnostics()` function |
| S80 | `BatchedEncoder`, `batched_nelder_mead_gpu`, `StatefulPipeline<S>`, `BatchedMultinomialGpu` V37 config, `GillespieGpu` improvements | 3 new delegations |
| S81-S82 | `ComputeDispatch` +16 ops, OS memory detection, creation.rs DRY | No direct impact |
| S84-S86 | `ComputeDispatch` +33 ops (111→144), hydrology module split | No direct impact (already wired) |
| S87 | FHE shader fix, `BarracudaError::is_device_lost()`, unsafe audit | Device-lost resilience wired |

### Updated delegation count

| Category | V69 | V70 | Change |
|----------|-----|-----|--------|
| CPU delegations | 44 | 47 | +3 (StatefulPipeline, Nelder-Mead, spectral diagnostics) |
| GPU delegations | 32 | 34 | +2 (device-lost harness, BatchedEncoder reference) |
| **Total** | **76** | **81** | **+5** |

---

## Part 3: Architecture Demarcation

With barraCuda as a standalone primal, the dependency direction is now:

```
Springs ──> barraCuda (direct cargo dep — WHAT to compute)
toadStool ──> barraCuda (as compute backend — WHERE/HOW to compute)
barraCuda ──> sourDough (primal traits only)
```

barraCuda has **zero** dependencies on toadStool, songBird, bearDog, or nestGate. The `toadstool` feature flag in barraCuda is optional and only needed when running inside toadStool's hardware selection layer.

groundSpring's `akida-driver` dependency remains at `phase1/toadstool` because NPU support has not yet budded. This is expected to follow in a future barraCuda release.

---

## Part 4: Cross-Spring Bidirectional Flow (V70 View)

| Flow | What | When | Impact |
|------|------|------|--------|
| hotSpring → all | DF64 precision, `Fp64Strategy`, Lanczos, Anderson spectral | S26-S87 | f64-class math on consumer GPUs |
| hotSpring → groundSpring | `anderson_4d`, `wegner_block_4d` | S84 | Tissue immunology (Paper 12) |
| wetSpring → groundSpring | Shannon/Simpson diversity, `BatchedMultinomialGpu` | S64 | Rarefaction, rare biosphere |
| neuralSpring → all | `pow_f64` polyfill fix, AlphaFold2 Evoformer | S-17, S69 | GPU compatibility |
| airSpring → groundSpring | Regression, hydrology, L-BFGS, `StatefulPipeline` | S66-S80 | WDM, ET₀, freeze-out, multi-day |
| groundSpring → wetSpring | `rawr_mean` bootstrap, `batched_multinomial` | S66 | Rarefaction CI |
| groundSpring → all | `InterconnectTopology`, `SubstratePipeline` | S81 | metalForge cross-hardware dispatch |
| barraCuda S67-S68 | Universal precision: "Math is universal, precision is silicon" | S67-S68 | One shader → F16/F32/F64/Df64 |
| barraCuda S79 | Spectral stats: bandwidth, condition number, phase | S79 | Anderson diagnostics |
| barraCuda S80 | StatefulPipeline, BatchedEncoder, batched Nelder-Mead | S80 | Multi-day, dispatch, optimization |
| barraCuda S87 | Device-lost resilience | S87 | GPU pipeline robustness |

---

## Part 5: Documentation Updates

All non-archive documentation updated to reflect barraCuda budding:

| Category | Files Updated |
|----------|--------------|
| Rust doc comments | 18 source files — `` `ToadStool` `` → `barraCuda` |
| Specs | `BARRACUDA_EVOLUTION.md` — path and budding note |
| Changelog | `CHANGELOG.md` — V70 entry |
| Cross-spring | `CROSS_SPRING_SHADER_EVOLUTION.md` — budding note |
| White paper | `STUDY.md`, `CAPABILITY_SURFACE.md`, `neuralAPI/README.md` |

---

## Part 6: Quality Gates

| Gate | Status |
|------|--------|
| `cargo fmt --check` | PASS |
| `cargo clippy --all-targets --all-features -- -W clippy::pedantic -W clippy::nursery` | PASS (zero warnings) |
| `cargo test --workspace` | 786 passed, 0 failed |
| `cargo doc --no-deps` | PASS |
| Zero unsafe | PASS |
| Zero TODO | PASS |
| Zero .unwrap() | PASS |

---

## Part 7: Provenance

| Metric | Value |
|--------|-------|
| barraCuda pin | `b53c3de` (v0.2.1) |
| ToadStool pin | S87 (`2dc26792`) — historical |
| Active delegations | 81 (47 CPU + 34 GPU) |
| groundSpring tests | 786 |
| Python parity | 28/28 |
| Debt | Zero |

---

## Part 8: barraCuda Actions

### P0 — Immediate

None. Budding is clean, zero breaking changes.

### P1 — Next Release

- ~~**akida-driver budding**~~: **Corrected in V71** — akida-driver is hardware, not math. It belongs permanently with toadStool ("WHERE and HOW"). barraCuda owns the precision/quantization path (fp64 → int4); toadStool owns the NPU driver and compilation.
- **tarpc alignment**: groundSpring uses tarpc 0.35, barraCuda uses 0.34 — no conflict currently but should align on next release
- **Cargo.toml version alignment**: barraCuda crate shows v0.2.0 but README/CHANGELOG say v0.3.0

### P2 — Evolution

- **`compile_shader_universal` for metalForge**: groundSpring's custom shaders could use `compile_shader_universal(src, Fp64Strategy)` for auto-precision selection per hardware
- **`Fp64Strategy::Concurrent` for validation**: groundSpring's three-tier parity tests could run DF64 and native f64 side-by-side for precision delta measurement
