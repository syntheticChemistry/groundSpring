# groundSpring V102 — BarraCUDA / ToadStool Niche Evolution Handoff

**Date**: March 14, 2026
**From**: groundSpring
**To**: barraCUDA, ToadStool, coralReef
**Covers**: V102 niche deployment, dispatch-through-delegation chain, GPU evolution opportunities
**License**: AGPL-3.0-only

---

## Executive Summary

- groundSpring V102 deploys as a biomeOS niche: UniBin binary + deploy graph + BYOB YAML
- New JSON-RPC dispatch layer (`dispatch.rs`) routes `measurement.*` methods to library functions that already delegate to barraCUDA — **zero compute overhead added**, the RPC envelope is pure routing
- 102 active delegations (61 CPU + 41 GPU) unchanged from V101
- 8 measurement capabilities now exposed as Neural API services via `capability.call`
- Key evolution opportunity: batch dispatch endpoints that compose multiple delegations per RPC call

---

## 1. Delegation-Through-Dispatch Chain

The V102 niche adds a JSON-RPC layer but does not alter the compute path:

```
biomeOS (execute_graph)
  → capability.call("measurement.anderson_validation", params)
    → Songbird IPC → groundspring socket
      → dispatch.rs: anderson_validation(params)
        → crate::anderson::lyapunov_averaged(n_sites, disorder, energy, n_realizations, seed)
          → #[cfg(feature = "barracuda")]  barracuda::spectral::anderson_potential()
          → #[cfg(feature = "barracuda-gpu")] barracuda::ops::anderson_lyapunov_f64_wgsl
```

Every `measurement.*` method resolves to a library function that already has barraCUDA delegation wired. The dispatch layer is a 1:1 mapping with parameter extraction.

### Method → Library → Delegation Map

| RPC Method | Library Target | CPU Delegation | GPU Delegation |
|---|---|---|---|
| `measurement.noise_decomposition` | `decompose::decompose_error` | None (scalar math) | None (scalar) |
| `measurement.anderson_validation` | `anderson::lyapunov_averaged` | `barracuda::spectral::anderson_potential` | `barracuda::ops::anderson_lyapunov_*` |
| `measurement.uncertainty_budget` | `bootstrap::bootstrap_mean` + `jackknife::jackknife_mean_variance` | `barracuda::stats::bootstrap_mean` + `barracuda::stats::jackknife_mean_variance` | `BootstrapMeanGpu` + `jackknife_mean_f64.wgsl` |
| `measurement.et0_propagation` | `fao56::daily_et0` | `barracuda::stats::hydrology::*` | `BatchedElementwiseF64` + `McEt0PropagateGpu` |
| `measurement.regime_classification` | `esn::spectral_features` + `esn::classify_by_spacing_ratio` | `barracuda::stats::level_spacing` | `EsnClassifier` GPU |
| `measurement.spectral_features` | `spectral_recon::build_kernel` + `tikhonov_solve` + `forward_correlator` | `barracuda::ops::fft::Fft1DF64` | Tikhonov GPU dispatch |
| `measurement.parity_check` | CPU vs GPU result comparison | Both tiers run | Both tiers run |
| `measurement.freeze_out` | `freeze_out::grid_fit_2d` | None (CPU grid) | Parallel grid dispatch |

---

## 2. Evolution Opportunities

### 2a. Batch RPC Endpoints

When biomeOS calls `measurement.uncertainty_budget`, the dispatch runs bootstrap + jackknife sequentially. A batched barraCUDA primitive could fuse these:

```
Current:  bootstrap_mean(data, 10000) → jackknife_mean_variance(data)  (2 GPU dispatches)
Proposed: barracuda::stats::uncertainty_budget(data, {bootstrap_n: 10000})  (1 dispatch)
```

Same pattern for `measurement.regime_classification` (spectral_features + classify are currently 2 calls).

**Priority**: P2 — optimization, not correctness.

### 2b. Streaming Dispatch for Continuous Niches

The Neural API supports `Continuous` coordination at fixed Hz (see `08_NICHE_API_PATTERNS.md`). If groundSpring participates in a live monitoring niche (e.g., real-time sensor validation at 1 Hz), the dispatch layer will call `decompose_error` every tick. The current per-call overhead is negligible, but a streaming variant that holds GPU buffers across ticks would reduce allocation:

```
Current:  measurement.noise_decomposition → allocate → compute → deallocate (per tick)
Proposed: barracuda::Pipeline::streaming_decompose(buffer_handle)  (persistent buffers)
```

**Priority**: P3 — future continuous niche support.

### 2c. ToadStool `compute.execute` Routing

ToadStool routes `compute.execute` requests to hardware (GPU/NPU/CPU). The niche deploy graph wires ToadStool as an optional dependency:

```toml
# graphs/groundspring_deploy.toml
[[nodes]]
id = "germinate_toadstool"
depends_on = ["germinate_beardog"]
output = "toadstool_genesis"
fallback = "skip"
[nodes.primal]
by_capability = "compute"
```

When ToadStool is present, barraCUDA GPU ops route through it for hardware-aware scheduling. When absent, groundSpring falls back to CPU. No ToadStool API changes needed.

---

## 3. Current Delegation Inventory (V102)

Unchanged from V101:

| Category | CPU | GPU | Total |
|---|---|---|---|
| Stats (mean, std, correlation, regression, chi²) | 15 | 12 | 27 |
| Bio (Gillespie, multinomial, Wright-Fisher, Hill, Monod) | 8 | 6 | 14 |
| Spectral (Anderson, Almost-Mathieu, Lanczos, ESN, ESD) | 12 | 9 | 21 |
| Hydrology (FAO-56, Hargreaves, Makkink, Turc, Hamon) | 8 | 3 | 11 |
| Bootstrap/Jackknife/RAWR | 4 | 3 | 7 |
| FFT, Autocorrelation, Peak detection | 3 | 3 | 6 |
| Other (seismic, drift, precision, covariance) | 11 | 5 | 16 |
| **Total** | **61** | **41** | **102** |

---

## 4. Deploy Graph for ToadStool Team

`graphs/groundspring_deploy.toml` references ToadStool as an optional Phase 2 dependency:

- `germinate_toadstool` with `fallback = "skip"` and `by_capability = "compute"`
- When present: GPU dispatch paths activate via `get_device_f64_safe()`
- When absent: CPU-only mode, all 61 CPU delegations active, 0 GPU
- `PrecisionRoutingAdvice` from barraCUDA guides f64 vs DF64 vs f32 shader selection

---

## 5. Niche YAML for Ecosystem Team

`niches/groundspring-measurement.yaml` defines the BYOB customization:

```yaml
customization:
  options:
    gpu_enabled:
      type: boolean
      default: false
      description: "Enable ToadStool GPU acceleration"
      affects: ["toadstool"]
```

When `gpu_enabled: true`, ToadStool is germinated and GPU dispatch activates. The barraCUDA delegation chain handles this transparently.

---

## 6. Absorption Targets (New in V102)

### Already absorbed (no action needed)
- All 102 delegations verified against barraCUDA v0.3.5
- `EsnClassifier` GPU path operational
- `Fft1DF64` wired for spectral reconstruction
- `BootstrapMeanGpu` for uncertainty budget

### New absorption opportunities
| Source | Target | Priority | Notes |
|---|---|---|---|
| `dispatch.rs` parameter extraction | N/A | None | Pure JSON parsing, not compute |
| `provenance.rs` capability call wrappers | N/A | None | IPC convenience, not compute |
| Batch `uncertainty_budget` | `barracuda::stats::uncertainty_budget` | P2 | Fuse bootstrap + jackknife |
| Batch `regime_classification` | `barracuda::stats::regime_classification` | P2 | Fuse spectral_features + classify |
| Streaming decompose | `barracuda::Pipeline::streaming_decompose` | P3 | Continuous niche support |

### PRNG alignment (ongoing)
- `prng::Xorshift64` remains local (not in barraCUDA)
- Roadmap: align to `xoshiro256++` when barraCUDA absorbs it
- Currently documented in `specs/BARRACUDA_EVOLUTION.md` Tier B

---

## 7. coralReef Notes

coralReef sovereign compilation of f64 shared-memory reduction shaders remains the path to native GPU binary execution without NVK/NAK. The niche deployment does not change this roadmap — coralReef is a ToadStool concern, not a groundSpring concern. groundSpring consumes via `get_device_f64_safe()` which routes to DF64 or CPU when native f64 shared memory is broken.

---

## Verification

```
cargo check --features biomeos: PASS
cargo clippy --features biomeos -- -D warnings: PASS
cargo test --workspace --lib: 703 PASS (538 + 138 + 27)
cargo test --workspace: 908 PASS (default features)
```

---

## Next Steps

1. **barraCUDA**: No blocking action. Consider P2 batch primitives when capacity allows.
2. **ToadStool**: No blocking action. `compute.execute` routing works as-is.
3. **coralReef**: No change. Sovereign f64 pipeline evolution independent.
4. **biomeOS**: Test `neural_api.execute_graph` with `groundspring_deploy.toml`.
