# groundSpring → ToadStool V56 Handoff: NUCLEUS Integration + barracuda Evolution

**Date**: March 1, 2026
**From**: groundSpring (V56)
**To**: ToadStool / BarraCUDA team
**ToadStool pin**: S70+++ (`1dd7e338`)
**License**: AGPL-3.0-only
**Supersedes**: V55 (barracuda Evolution Complete)

---

## Executive Summary

- **32 experiments** validated (28 core + 4 NUCLEUS), **347/347 checks** PASS
- **57 active barracuda delegations** (38 CPU + 19 GPU), 1 evolution candidate
- **biomeOS Neural API live**: Tower (BearDog), Node (ToadStool), Squirrel validated
- **NestGate data pipelines**: NCBI, NOAA GHCND, IRIS FDSN — sovereign fallback on all
- **Key learning**: `compute.execute` is for ToadStool-native workloads (containers, WASM,
  GPU job scheduling), not for forwarding physics. groundSpring uses barracuda directly.
- **622 Rust workspace tests** (biomeos feature), 375 Python, 95 three-tier parity tests

---

## Part 1: What Changed Since V55

V55 completed the barracuda evolution review (57 delegations, full inventory, cross-spring
lineage). V56 adds the NUCLEUS Neural API integration layer — how groundSpring interacts
with the rest of the ecoPrimals ecosystem through biomeOS.

### New Modules

| Module | Feature | Purpose |
|--------|---------|---------|
| `biomeos.rs` | `biomeos` | Neural API client: socket discovery, `auto_connect()`, `capability_call()`, health checks |
| `nestgate.rs` | `biomeos` | NestGate data pipeline: NCBI search/fetch, NOAA GHCND, IRIS stations/events, provenance |

### New Experiments (Exp 029–032)

| Exp | Name | Checks | NUCLEUS Required | Data Source |
|-----|------|--------|-----------------|-------------|
| 029 | Real GHCND ET₀ | 6/6 | Optional | NestGate NOAA CDO / synthetic |
| 030 | Real NCBI 16S | 9/9 | Optional | NestGate NCBI SRA / synthetic |
| 031 | NUCLEUS Stack | 28/28 | For live paths | All primals (adaptive) |
| 032 | IRIS Seismic | 12/12 | Optional | NestGate IRIS FDSN / synthetic |

### New biomeOS Client Functions

| Function | Semantic Name | Target Primal |
|----------|--------------|---------------|
| `capability_call()` | Any semantic method | biomeOS router |
| `storage_put()` | `storage.put` | NestGate |
| `storage_get()` | `storage.get` | NestGate |
| `compute_execute()` | `compute.execute` | ToadStool |
| `compute_submit()` | `compute.submit` | ToadStool |
| `compute_capabilities()` | `compute.capabilities` | ToadStool |

---

## Part 2: Barracuda Delegation Inventory (Unchanged from V55)

57 active delegations: 38 CPU + 19 GPU. 1 evolution candidate (band_edges).
Full inventory in V55 handoff (now in `handoffs/archive/`). No new delegations
added in V56 — the focus was NUCLEUS integration, not barracuda extension.

---

## Part 3: What groundSpring Learned About ToadStool as Node

### 3a. `compute.execute` Is Not for Physics

groundSpring initially attempted to route physics computations (Lyapunov exponents)
through `compute.execute`. This failed because ToadStool's `execute_workload` endpoint
dispatches ToadStool-native workloads (containers, WASM modules, GPU job scheduling),
not arbitrary function calls.

**Correct architecture**:
```
groundSpring physics → barracuda::stats::* (direct Rust call, no RPC)
                     → GPU dispatch via wgpu (when barracuda-gpu enabled)

groundSpring NUCLEUS → biomeOS Neural API → ToadStool (health, capabilities, version)
                                          → NestGate (storage, data)
                                          → BearDog (crypto, beacon)
                                          → Squirrel (AI health)
```

**toadStool action**: No action needed. This is correct architecture. Physics through
barracuda (direct), infrastructure through Neural API (RPC).

### 3b. Capability Registry Alignment

groundSpring uses semantic names (`storage.put`, `compute.execute`) that biomeOS
translates to primal-specific methods. The translations in
`biomeOS/config/capability_registry.toml` were aligned:

| Semantic | biomeOS Translation | NestGate Actual |
|----------|-------------------|----------------|
| `storage.put` | `storage.store` | `storage.store` |
| `storage.get` | `storage.retrieve` | `storage.retrieve` |
| `compute.execute` | `execute_workload` | ToadStool native |
| `compute.submit` | `submit_workload` | ToadStool native |

**toadStool action**: Ensure `compute.health`, `compute.capabilities`, `compute.version`
are stable endpoints. groundSpring depends on them for NUCLEUS health validation (Exp 031).

### 3c. Compute Capabilities Response

ToadStool returns a 641-byte JSON capabilities document. groundSpring uses this to verify
GPU availability. The specific fields groundSpring checks:

- Response is valid JSON
- Response is non-empty (len > 0)

**toadStool action**: Consider standardizing a capabilities schema so consumers can
reliably query specific capability types (e.g., "has WGSL shader dispatch", "has f64
support", "available VRAM").

---

## Part 4: Deployment Graphs

groundSpring provides biomeOS deployment graphs in `graphs/`:

| Graph | Purpose |
|-------|---------|
| `groundspring_tower_bootstrap.toml` | Bootstrap Tower for sovereign crypto + identity |
| `groundspring_nucleus_node.toml` | Deploy ToadStool as NUCLEUS compute node |
| `groundspring_validation.toml` | Run validation suite through biomeOS |
| `groundspring_cross_substrate.toml` | metalForge cross-substrate dispatch |

These use biomeOS template variables (`${XDG_RUNTIME_DIR}`, `${FAMILY_ID}`) and are
not device-specific.

---

## Part 5: barracuda Evolution Recommendations (Carried from V55)

### Priority 1: band_edges Evolution Candidate
- **Problem**: Transfer-matrix half-trace scan vs eigenvalue extraction algorithm mismatch
- **barracuda**: `barracuda::spectral::detect_bands` uses eigenvalue extraction
- **groundSpring**: `band_structure::find_band_edges` uses transfer-matrix half-trace scan
- **Action**: Adopt transfer-matrix approach in barracuda or provide adapter

### Priority 2: PRNG Alignment
- **Problem**: `lyapunov_averaged` uses different seed strategies between barracuda and local
- **barracuda**: `base_seed + r * 1000` per realization
- **local**: `base_seed + i`
- **Action**: Standardize PRNG seeding strategy

### Priority 3: Benchmark GPU Coverage
- **Problem**: Several GPU-capable benchmarks report `gpu_ms: None`
- **Anderson Lyapunov**, **seismic grid search**, **freeze-out grid fit**, **rare biosphere**
  all have GPU paths but aren't exercised in the CPU-vs-GPU benchmark
- **Action**: Wire GPU benchmark entries for these workloads

---

## Part 6: Cross-Spring Shader Lineage (Unchanged)

Full cross-spring lineage documented in `wateringHole/CROSS_SPRING_SHADER_EVOLUTION.md`.
Five springs contribute to barracuda's 700+ WGSL shaders.

---

## Action Items Summary

| Priority | Action | Owner |
|----------|--------|-------|
| High | Keep `compute.health`/`capabilities`/`version` stable | ToadStool |
| High | band_edges algorithm alignment | ToadStool + groundSpring |
| Medium | Standardize capabilities response schema | ToadStool |
| Medium | PRNG seeding alignment for `lyapunov_averaged` | barracuda |
| Low | Wire GPU benchmark entries for 4 missing workloads | groundSpring |

---

**License**: AGPL-3.0-only
