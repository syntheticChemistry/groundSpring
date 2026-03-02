# groundSpring → ToadStool V63: Brain Architecture + Capability-Based Discovery

**Date**: March 2, 2026
**groundSpring Version**: V63
**ToadStool Pin**: S79 (`f97fc2ae`)
**Tests**: 716 (382 lib + 334 integration/validation)
**Clippy**: Clean (zero warnings, `-D warnings`)
**Docs**: Clean (`-D warnings`)
**Format**: Clean (`cargo fmt --check`)

---

## Summary

V63 integrates hotSpring's brain architecture patterns, Nautilus Shell
self-regulation, and multi-head ESN uncertainty into groundSpring. All
hardcoded primal names are eliminated from routing and validation output —
groundSpring now operates purely through capability-based discovery with
self-knowledge only.

---

## Changes

### 1. ESN Module Evolution (`esn.rs`)

**New types:**

| Type | Lineage | Purpose |
|------|---------|---------|
| `DriftAction` | Nautilus Shell `constraints.rs` | Self-regulating drift boundary response (None / IncreaseSelection / IncreasePop) |
| `ConceptEdge` | Nautilus Brain `brain.rs` | Structured edge detection with parameter, LOO error, and drift action |
| `MultiHeadUncertainty` | hotSpring 15-head ESN | Per-observable mean/std_dev across heads, max disagreement |

**Evolved functions:**

| Function | Change |
|----------|--------|
| `detect_concept_edges` | Returns `Vec<ConceptEdge>` with drift action recommendations (was `Vec<(f64, f64)>`) |
| `drift_action_for_edge` | New — heuristic mapping from LOO error magnitude to `DriftAction` |
| `seed_around_edges` | New — generates focused sampling around detected phase boundaries (Nautilus `EdgeSeeder` pattern) |
| `multi_head_uncertainty` | New — aggregates per-observable predictions from N heads into epistemic uncertainty |

**Test additions:** +13 tests (DriftAction display, edge detection with actions, seeding geometry, multi-head basic/empty/single)

### 2. Capability Registration (`biomeos.rs`)

**New API:**

| Function | Purpose |
|----------|---------|
| `SCIENCE_CAPABILITIES` | Const slice of 7 `science.*` capability strings |
| `register_capabilities(socket)` | Register all science capabilities with NUCLEUS |
| `deregister_capabilities(socket)` | Graceful shutdown deregistration |

**Science capabilities registered:**
- `science.anderson_validation`
- `science.noise_decomposition`
- `science.parity_check`
- `science.et0_propagation`
- `science.regime_classification`
- `science.uncertainty_budget`
- `science.spectral_features`

### 3. Configurable Timeouts (`biomeos.rs`)

Replaced `const CONNECT_TIMEOUT` and `const READ_TIMEOUT` with functions
that respect environment variables:
- `GROUNDSPRING_BIOMEOS_CONNECT_TIMEOUT_SECS` (default: 5)
- `GROUNDSPRING_BIOMEOS_READ_TIMEOUT_SECS` (default: 30)

### 4. Hardcoded Primal Name Elimination

All hardcoded primal names removed from Rust source (routing, display labels,
doc comments):

| File | Before | After |
|------|--------|-------|
| `validate_nucleus_pipeline.rs` | `"ToadStool"`, `"Squirrel"` in check labels | `"compute provider"`, `"AI provider"` |
| `validate_nucleus_stack.rs` | `"BearDog"`, `"ToadStool"`, `"NestGate"`, `"Squirrel"` in phases | Capability-based labels (`"crypto + beacon"`, `"compute"`, `"storage + data"`, `"AI capability"`) |
| `nestgate.rs` | `"NestGate"` in doc comments | `"storage provider"`, `"data provider"` |
| `validate_nucleus_pipeline.rs` | `"ToadStool"` in `println!` | `"compute provider"` |

**Routing was already capability-based** — `capability_call(socket, "compute.health", "{}")` discovers the provider at runtime. V63 extends this consistency to display labels and documentation.

### 5. Cross-Spring Shader Evolution Doc Update

Updated `wateringHole/CROSS_SPRING_SHADER_EVOLUTION.md`:
- Header: S79 `f97fc2ae`, 844 WGSL shaders, 14,200+ barracuda tests
- groundSpring contributions: 2 WGSL shaders (was 4, 2 absorbed by ToadStool)
- hotSpring section: +2 entries (4-layer brain architecture, 15-head multi-observable ESN)
- Timeline: +V62 entry with full validation summary

---

## Audit Results

| Category | Status |
|----------|--------|
| Unsafe code | **Zero** — `unsafe_code = "forbid"` workspace-wide |
| Mocks in production | **Zero** — all mocks isolated to `#[cfg(test)]` |
| `todo!`/`unimplemented!` | **Zero** |
| Files > 1000 lines | **Zero** (largest: fao56.rs @ 822) |
| Hardcoded primal names in routing | **Zero** |
| Hardcoded primal names in display | **Zero** (all evolved to capability-based) |
| Hardcoded ports | **1** (8090 fallback in validate_nestgate_ncbi.rs — configurable via `NESTGATE_URL`) |
| External C/C++ dependencies | **Zero** (all Rust-native; barracuda/akida-driver/bingocube are ecosystem path deps) |
| Test count | **716** (382 lib + 334 integration) |

---

## Validation

```
cargo fmt --all -- --check    ✅ Clean
cargo clippy -D warnings      ✅ Clean (workspace, all targets, all features)
RUSTDOCFLAGS="-D warnings" cargo doc  ✅ Clean (39 doc files)
cargo test --workspace         ✅ 716 passed, 0 failed
```

---

## Architecture Principles Verified

1. **Self-knowledge only**: groundSpring knows its own capabilities (`SCIENCE_CAPABILITIES`) and `FAMILY_ID`. Other primals discovered at runtime via `capability.call`.
2. **Sovereign fallback**: Every NUCLEUS path degrades to local computation when the socket is unavailable.
3. **Zero unsafe**: Workspace-level `forbid(unsafe_code)` — no exceptions.
4. **Capability-based discovery**: No primal names in routing. Validation binaries use generic labels.
5. **Configurable timeouts**: All network timeouts overridable via env vars.
6. **DRY patterns**: `crate::cast::usize_f64` for all usize→f64 conversions.

---

## baseCamp Paper 12 Integration

Paper 12 (Anderson Localization in Immunological Signaling) extends the
Anderson framework to cytokine propagation in skin tissue. groundSpring
provides the spectral theory and transport validation that underpins the
immunological mapping:

| groundSpring Experiment | Paper 12 Role |
|------------------------|---------------|
| Exp 008 (Anderson localization) | 2D/3D spectral diagnostics — epidermis vs dermis |
| Exp 012 (spin chain transport) | Cytokine propagation distance through tissue channels |
| Exp 015 (uncertainty bridge) | Cytokine measurement → regime classification confidence |
| Exp 018 (band edge structure) | Epidermal periodicity → band gaps for cytokine filtering |

V63 features directly serving Paper 12:
- `ConceptEdge` — detects AD flare ↔ remission boundaries from cytokine sweeps
- `DriftAction` — steers treatment parameter exploration
- `seed_around_edges` — focuses sampling around phase transitions
- `MultiHeadUncertainty` — epistemic uncertainty at flare boundaries

**Dimensional promotion–collapse duality**: Paper 06 (tillage = 3D→2D collapse)
and Paper 12 (scratching = 2D→3D promotion) are the same physics in opposite
directions. Both validated by Exp 008's 2D/3D Anderson computations.

---

## Integration Priorities (Next)

| Priority | Item | Lineage |
|----------|------|---------|
| 1 | Wire Nautilus export path for NPU Anderson (AKD1000 int4) | hotSpring Exp 029 |
| 2 | Adaptive experiment steering (disorder sweep β adaptation) | hotSpring Exp 030 adaptive steering |
| 3 | 4-layer brain dispatch for heterogeneous compute | hotSpring `BIOMEGATE_BRAIN_ARCHITECTURE.md` |
| 4 | Evolve NestGate port discovery to capability-based | Eliminate 8090 fallback constant |
| 5 | Paper 12 wetSpring experiments (Exp 270-274) | Skin-layer Anderson lattice + barrier disruption model |

---

## Cross-Spring Lineage

```
hotSpring v0.6.15 (brain + Nautilus + ESN)
    ↓
bingoCube/nautilus (DriftAction, EdgeSeeder, ConceptEdge)
    ↓
ToadStool S79 (barracuda::esn_v2, 844 WGSL shaders)
    ↓
groundSpring V63:
  • DriftAction + ConceptEdge (from Nautilus)
  • MultiHeadUncertainty (from hotSpring 15-head ESN)
  • seed_around_edges (from Nautilus EdgeSeeder)
  • detect_concept_edges evolved (structured return + drift actions)
  • SCIENCE_CAPABILITIES registration (7 science.* capabilities)
  • All primal names → capability-based discovery
  • Paper 12 mapped (Exp 008/012/015/018 → immunological Anderson)
  • 716 tests, zero warnings, zero unsafe
```
