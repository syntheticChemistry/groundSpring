# groundSpring V77 — wgpu 28 Migration + barraCuda v0.3.3 Sync

**Date:** 2026-03-05
**From:** groundSpring V77
**To:** barraCuda team, toadStool team, ecoPrimals ecosystem
**Supersedes:** V76 (structural evolution + deep debt zero)
**barraCuda pin:** v0.3.3 (`4629bdd`)
**toadStool pin:** S94b (`9d359814`)
**groundSpring tests:** 790 passed, 0 failed
**License:** AGPL-3.0-only

---

## Executive Summary

V77 completes the wgpu 22 → 28 migration, synchronizing groundSpring with
barraCuda v0.3.3 and toadStool S94b. All 81 barracuda delegation sites are
unaffected (CPU/GPU function signatures unchanged). Two metalForge files
that use raw wgpu APIs were migrated to the wgpu 28 API. Deep debt zero
maintained from V76.

## What Changed

### 1. Dependency Bumps
- `wgpu` 22 → 28 in `crates/groundspring/Cargo.toml`
- `wgpu` 22 → 28 in `metalForge/forge/Cargo.toml`

### 2. wgpu 28 API Migration

**`validate_metalforge_titan_v.rs`** (7 changes):
- `entry_point: "main"` → `entry_point: Some("main")`
- `set_bind_group(0, &bg, &[])` → `set_bind_group(0, Some(&bg), &[])`
- `Instance::new(desc)` → `Instance::new(&desc)`
- `enumerate_adapters(Backends::all())` → async via `tokio_block_on`
- `Maintain::Wait` → `PollType::Wait { submission_index: None, timeout: None }`
- `DeviceDescriptor` + `experimental_features`, `trace` fields
- `request_device(&desc, None)` → `request_device(&desc)` (trace path removed)

**`probe.rs`** (2 changes):
- `Instance::new(desc)` → `Instance::new(&desc)`
- `enumerate_adapters(Backends::all())` → async via `tokio_block_on`

### 3. Free Upgrades from barraCuda v0.3.3

| Feature | Benefit |
|---------|---------|
| DF64 precision tiers | 15 ops auto-select f64/DF64/f32 per GPU |
| Fused reductions | Single-pass Welford mean+variance, 5-acc Pearson |
| TensorContext pooling | Pipeline/buffer caching for stats ops |
| DF64 naga rewriter fix | Titan V compound assignment works correctly |
| sourdough-core removal | Broken path dep eliminated |

### 4. What Did NOT Change

- All 81 delegation call sites (barracuda CPU/GPU signatures unchanged)
- barracuda path in Cargo.toml (already points to standalone `barraCuda/`)
- Feature gating (`barracuda`, `barracuda-gpu`)
- Tolerance architecture (`tol::`, `eps::`)
- Python baselines
- Deep debt zero status

## Quality Gates

- `cargo fmt --check`: PASS
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS
- `cargo doc --workspace --no-deps`: PASS
- `cargo test --workspace`: 790 passed, 0 failed
- Zero TODO/FIXME/HACK in Rust source
- Zero `unwrap()` in production code
- Zero unsafe code

## Migration Pattern Reference

For any spring migrating from wgpu 22 → 28, these are the API changes:

```rust
// Instance creation — now takes reference
Instance::new(&InstanceDescriptor { ... })

// enumerate_adapters — now async
let adapters = tokio_block_on(instance.enumerate_adapters(Backends::all()));

// DeviceDescriptor — new required fields
DeviceDescriptor {
    label: Some("..."),
    required_features: ...,
    required_limits: ...,
    memory_hints: MemoryHints::Performance,
    experimental_features: ExperimentalFeatures::default(),
    trace: Trace::default(),
}

// request_device — trace path removed
adapter.request_device(&desc)  // was: adapter.request_device(&desc, None)

// ComputePipelineDescriptor — entry_point is Option
entry_point: Some("main")  // was: entry_point: "main"

// set_bind_group — bind group is Option
pass.set_bind_group(0, Some(&bg), &[])  // was: pass.set_bind_group(0, &bg, &[])

// device.poll — Maintain removed, use PollType
device.poll(PollType::Wait { submission_index: None, timeout: None })
// was: device.poll(Maintain::Wait)
```
