# Ecosystem Leverage Guide

> How groundSpring absorbs from and contributes to the ecoPrimals ecosystem.
>
> **Last updated**: March 23, 2026 (V123 — 110 delegations, 1020+ tests, barraCuda v0.3.7)

## Leverage Philosophy

groundSpring follows the **Write → Absorb → Lean** cycle:

1. **Write** locally — pure Rust CPU implementations as validation references
2. **Absorb** upstream — barraCuda absorbs shaders and math primitives
3. **Lean** on upstream — groundSpring rewires to `barracuda::*` via `#[cfg]`

The CPU implementation is always the **validation reference**. GPU paths
are for throughput. Both produce identical results within documented
tolerances.

## What groundSpring Consumes

### barraCuda (110 delegations)

| Domain | CPU | GPU | Key primitives |
|--------|-----|-----|----------------|
| Stats | 30 | — | pearson, spearman, rmse, mae, mbe, nse, r², ia, hit_rate, bootstrap, jackknife, moving_window, regression |
| Spectral | — | 15 | Anderson 1D–4D, Lanczos, Almost-Mathieu, Wegner RG, band detect |
| Bio ops | — | 5 | Gillespie, Wright-Fisher, BatchedMultinomial |
| Linalg | — | 2 | eigh_f64, solve_f64 + Cholesky |
| Optimize | 2 | — | L-BFGS, batched Nelder-Mead |
| Pipeline | — | 5 | FAO-56 batch, Hargreaves GPU, McEt0, seasonal, water balance |
| ODE | 2 | — | BistableOde, MultiSignalOde |
| Reduce | — | 4 | sum, variance, fused map-reduce, correlation |
| ESN | — | 1 | esn_v2::ESN regime classification |
| Grid | — | 1 | grid_search_3d |

### toadStool (compute orchestration)

- `compute.execute` / `compute.submit` via Neural API
- `PrecisionRoutingAdvice` for f64→Titan V, f32→RTX 4070
- Hardware discovery (`probe.rs` OnceLock cache)
- akida-driver for NPU inference

### NestGate / Squirrel (storage + data)

- `storage.put` / `storage.get` for provenance records
- `data.ncbi_search` / `data.ncbi_fetch` for real 16S data
- `data.noaa_ghcnd` for real weather station data
- `data.iris_stations` / `data.iris_events` for seismic data

### coralReef (sovereign compiler)

- `shader.compile.wgsl` / `shader.compile.batch` for sovereign shader compilation
- `PrecisionRoutingAdvice` alignment

### Provenance Trio (rhizoCrypt + loamSpine + sweetGrass)

- Session start/commit via `provenance.rs`
- Contribution attribution
- Circuit-breaker resilience (`biomeos/resilience.rs`)

## What groundSpring Contributes

### Upstream to barraCuda

groundSpring has contributed the following to barraCuda absorption:

| Contribution | Session | Status |
|-------------|---------|--------|
| Anderson Lyapunov WGSL shaders (f32 + f64) | S50 | Absorbed |
| Bias-variance decomposition API | S64 | Absorbed |
| Rarefaction diversity metrics (shannon, simpson, chao1, bray_curtis) | S64 | Absorbed |
| Agreement metrics (rmse, mbe, mae, nse, r², ia, hit_rate) | S64 | Absorbed |
| Batched multinomial GPU kernel | S76 | Absorbed |
| MC ET₀ propagation GPU kernel | S72 | Absorbed |
| RAWR Dirichlet-weighted bootstrap mean | S66 | Absorbed |
| Hill function kinetics | S68 | Absorbed |
| Anderson 4D + Wegner block RG | S88 | Absorbed |
| Seasonal pipeline GPU | S88 | Absorbed |

### Cross-Spring Patterns

Patterns pioneered or refined in groundSpring and adopted by siblings:

| Pattern | Adopted by |
|---------|-----------|
| `DispatchOutcome<T>` typed dispatch | ludoSpring V24 |
| `OrExit<T>` zero-panic validation | wetSpring, ludoSpring |
| `deny.toml` C-dep ban (14 crates) | wetSpring V128, healthSpring V37 |
| `cast` module safe numeric helpers | airSpring, wetSpring, healthSpring |
| `ValidationSink` trait (V116) | ludoSpring, rhizoCrypt, primalSpring |
| 13-tier tolerance architecture | Referenced by airSpring, wetSpring |
| `FEATURE_GATES` niche metadata | healthSpring V37 |

## Remaining Absorption Candidates

| Item | Blocker | Action |
|------|---------|--------|
| `prng::Xorshift64` → `Xoshiro128StarStar` | Full rebaseline needed | Phase 2b |
| Local WGSL reference shaders (2) | Kept as provenance artifacts | No action |

## Discovery Configuration

groundSpring discovers ecosystem primals entirely at runtime:

- **Self-knowledge**: `primal_names::SELF_ID` = `"groundspring"`
- **Role discovery**: `primal_names::roles::*` constants + `discover_socket()`
- **Capability routing**: `capability.call` via biomeOS — no hardcoded primal names
- **Graceful degradation**: all IPC paths fall back to local computation
