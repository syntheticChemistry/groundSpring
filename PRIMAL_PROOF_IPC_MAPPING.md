# groundSpring — Primal-Proof IPC Mapping

**Date**: May 10, 2026
**groundSpring**: V140 (Tier 2 converged: toadstool.validate + toadstool.list_workloads + barracuda.precision.route + shader.compile.wgsl. Tier 4 IPC-first, LTEE B1-B4 complete with tolerances.toml, --format json on all 38 binaries)
**barraCuda**: v0.4.0
**primalSpring**: v0.9.25

---

## Purpose

This document maps every `barracuda::` library call in groundSpring to its
equivalent JSON-RPC method name, enabling the transition from in-process
library linkage to sovereign NUCLEUS deployment where barraCuda runs as a
separate ecobin.

groundSpring's `barracuda` Cargo feature gate has been flipped to IPC-first:
`default = []` (Tier 4). The `local` feature enables direct library linkage.
When `barracuda` is off, calls route through IPC via `CompositionContext`.

---

## Mapping Table

### Statistics (`barracuda::stats::*`)

| Library Call | JSON-RPC Method | Domain | Notes |
|-------------|----------------|--------|-------|
| `barracuda::stats::shannon(&counts)` | `tensor.stats.shannon` | ecology | Shannon diversity H' |
| `barracuda::stats::simpson(&counts)` | `tensor.stats.simpson` | ecology | Simpson diversity index |
| `barracuda::stats::bray_curtis(a, b)` | `tensor.stats.bray_curtis` | ecology | Dissimilarity metric |
| `barracuda::stats::pielou_evenness(&counts)` | `tensor.stats.pielou` | ecology | Evenness J' |
| `barracuda::stats::diversity::chao1_classic(counts)` | `tensor.stats.chao1` | ecology | Richness estimator |
| `barracuda::stats::evolution::detection_power(a, d)` | `tensor.stats.detection_power` | ecology | Rare taxa |
| `barracuda::stats::evolution::detection_threshold(a, p)` | `tensor.stats.detection_threshold` | ecology | Rare taxa |
| `barracuda::stats::evolution::kimura_fixation_prob(n, s, p)` | `tensor.stats.kimura_fixation` | genetics | Fixation probability |
| `barracuda::stats::jackknife::jackknife_mean_variance(data)` | `tensor.stats.jackknife_mean` | resampling | Delete-one jackknife |
| `barracuda::stats::chi2::chi2_decomposed_weighted(...)` | `tensor.stats.chi2_decomposed` | fitting | Weighted chi-squared |
| `barracuda::stats::spectral_density::marchenko_pastur_bounds(g)` | `tensor.stats.mp_bounds` | spectral | RMT bounds |
| `barracuda::stats::spectral_density::empirical_spectral_density(e, n)` | `tensor.stats.esd` | spectral | Empirical density |

### Spectral Analysis (`barracuda::spectral::*`)

| Library Call | JSON-RPC Method | Domain | Notes |
|-------------|----------------|--------|-------|
| `barracuda::spectral::spectral_bandwidth(eigenvalues)` | `tensor.spectral.bandwidth` | condensed-matter | Eigenvalue spread |
| `barracuda::spectral::spectral_condition_number(eigenvalues)` | `tensor.spectral.condition` | condensed-matter | Matrix conditioning |
| `barracuda::spectral::classify_spectral_phase(e, upper)` | `tensor.spectral.phase` | condensed-matter | Phase classification |

### GPU Operations (`barracuda::ops::*`)

| Library Call | JSON-RPC Method | Domain | Notes |
|-------------|----------------|--------|-------|
| `barracuda::ops::sum_reduce_f64::SumReduceF64::mean(dev, data)` | `tensor.reduce.mean` | compute | GPU mean reduction |
| `barracuda::ops::peak_detect_f64::PeakDetectF64::new(d, n)` | `tensor.peak_detect` | compute | Peak detection |
| `barracuda::ops::fused_map_reduce_f64::FusedMapReduceF64` | `tensor.fused_map_reduce` | compute | Fused GPU kernel |
| `barracuda::ops::bio::WrightFisherGpu` | `tensor.bio.wright_fisher` | genetics | GPU drift simulation |
| `barracuda::ops::bio::BatchedMultinomialGpu` | `tensor.bio.multinomial` | ecology | GPU rarefaction |

### Linear Algebra (`barracuda::linalg::*`)

| Library Call | JSON-RPC Method | Domain | Notes |
|-------------|----------------|--------|-------|
| `barracuda::linalg::dense_matmul` | `tensor.matmul` | compute | Dense matrix multiply |
| `barracuda::linalg::tridiag_eigh` | `tensor.linalg.tridiag_eigh` | compute | Eigendecomposition |

### Tolerances and Utilities

| Library Call | JSON-RPC Method | Domain | Notes |
|-------------|----------------|--------|-------|
| `barracuda::tolerances::*` | N/A (constants) | — | Compile-time constants, no IPC |
| `barracuda::device::WgpuDevice` | N/A (device handle) | — | GPU handle, no IPC equivalent |

---

## Feature Flag Strategy

```toml
[features]
default = []                    # IPC-first (Tier 4) — no library coupling
local = ["barracuda"]           # Opt-in library linkage for local compute
barracuda = ["dep:barracuda"]   # Direct library linkage (enabled by `local`)
barracuda-gpu = ["barracuda", "barracuda/gpu", ...]  # GPU path
```

### Transition Path

1. **V126**: `barracuda` was default. Library calls were direct.
2. **V140 (current)**: Tier 2 converged. `toadstool.validate`, `toadstool.list_workloads` (filter param), `barracuda.precision.route`, `shader.compile.wgsl` (coralReef FECS). `barracuda` removed from `default`. LTEE B1-B4 complete with `tolerances.toml`. `--format json` on all 38 binaries. barraCuda v0.4.0. `roles::GPU_MATH` added. Feature flags documented.
   `local` feature enables library linkage. All 284 `barracuda::` references
   are behind `#[cfg(feature = "barracuda")]`; IPC fallback paths active
   when the feature is off. `CompositionContext` routes through biomeOS.
3. **Next**: Wire `primal-proof` parallel validation (library vs IPC comparison).

---

## IPC Routing via biomeos

The `biomeos` module already routes to deployed primals via JSON-RPC:

| biomeos Function | JSON-RPC Path | Primal |
|-----------------|--------------|--------|
| `biomeos::compute_execute(op, params)` | `compute.execute` | barraCuda |
| `biomeos::compute_submit(op, params)` | `compute.submit` | barraCuda |
| `biomeos::compute_capabilities()` | `compute.capabilities` | barraCuda |
| `biomeos::storage_get(key, fam)` | `storage.get` | NestGate |
| `biomeos::storage_put(key, val, fam)` | `storage.put` | NestGate |
| `biomeos::capability_call(cap, op, args)` | `capability.call` | Any |

### CompositionContext routing (via primalSpring v0.9.25)

```rust
let mut ctx = CompositionContext::from_live_discovery_with_fallback();
ctx.call("tensor", "stats.mean", json!({"data": [1.0, 2.0, 3.0]}));
ctx.call("compute", "compute.dispatch", json!({...}));
ctx.call("storage", "storage.put", json!({...}));
ctx.health_check("security");
```

---

## License

AGPL-3.0-or-later
