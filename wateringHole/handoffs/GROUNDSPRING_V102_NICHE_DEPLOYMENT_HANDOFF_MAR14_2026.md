# groundSpring V102 — Niche Deployment via biomeOS Graph Composition

**Date**: March 14, 2026
**From**: groundSpring
**To**: biomeOS, ToadStool, BarraCUDA, Provenance Trio (rhizoCrypt, LoamSpine, sweetGrass)
**Status**: V102 — Spring-as-Niche deployed

---

## What Changed

groundSpring is now a deployable niche that biomeOS composes from a deploy graph. The previous model (standalone validation library) has been evolved to a **UniBin binary + deploy graph + niche YAML** following the `SPRING_AS_NICHE_DEPLOYMENT_STANDARD`.

### New Artifacts

| Artifact | Path | Purpose |
|----------|------|---------|
| UniBin binary | `crates/groundspring/src/bin/groundspring_primal.rs` | `groundspring server/status/version` |
| Server module | `crates/groundspring/src/biomeos/server.rs` | UDS socket bind + JSON-RPC accept |
| Dispatch module | `crates/groundspring/src/dispatch.rs` | `measurement.*` → library function routing |
| Provenance module | `crates/groundspring/src/provenance.rs` | Trio lifecycle wrappers |
| Deploy graph | `graphs/groundspring_deploy.toml` | Canonical niche deploy (5 phases) |
| Niche YAML | `niches/groundspring-measurement.yaml` | BYOB definition |

### Measurement Domain

Capability namespace changed from `science.*` to `measurement.*`:

| Method | Library Target |
|--------|---------------|
| `measurement.noise_decomposition` | `decompose::decompose_error` |
| `measurement.anderson_validation` | `anderson::lyapunov_averaged` |
| `measurement.uncertainty_budget` | `bootstrap::bootstrap_mean` + `jackknife::jackknife_mean_variance` |
| `measurement.et0_propagation` | `fao56::daily_et0` |
| `measurement.regime_classification` | `esn::spectral_features` + `esn::classify_by_spacing_ratio` |
| `measurement.spectral_features` | `spectral_recon::tikhonov_solve` |
| `measurement.parity_check` | CPU vs GPU value comparison |
| `measurement.freeze_out` | `freeze_out::grid_fit_2d` |

Two-phase registration: domain `measurement` with semantic mappings, then per-capability.

### Graph Evolution

All 6 graphs updated:
- `groundspring_deploy.toml` (NEW) — canonical niche deploy
- `groundspring_tower_bootstrap.toml` — V102, capability-based discovery, measurement namespace
- `groundspring_nucleus_local.toml` — V102, measurement capabilities, Provenance Trio wired
- `groundspring_nucleus_node.toml` — V102, measurement namespace
- `groundspring_validation.toml` — V102, measurement namespace, Provenance Trio wired
- `groundspring_cross_substrate.toml` — V102, measurement namespace, Provenance Trio wired

Key evolution: all `rpc_call target = "nestgate"` nodes evolved to `capability_call capability = "storage.put"`.

---

## For biomeOS Team

### Deploy Graph Ready for `execute_graph`

`graphs/groundspring_deploy.toml` follows the standard:
- Phase 1: Tower (BearDog + Songbird)
- Phase 2: Optional dependencies (ToadStool, NestGate) with `fallback = "skip"`
- Phase 3: groundSpring (`by_capability = "measurement"`)
- Phase 4: Health validation
- Phase 5: Provenance session (optional)

Expected invocation: `biomeos deploy graphs/groundspring_deploy.toml` or `biomeos niche deploy groundspring-measurement`

### Capability Registration

On startup, groundSpring calls:
1. `capability.register` with `capability = "measurement"` + `semantic_mappings` (domain-level)
2. `capability.register` for each of 8 `measurement.*` capabilities (individual)
3. `lifecycle.register` (Neural API lifecycle)

On shutdown: `capability.deregister` for all capabilities.

### Socket Convention

`$XDG_RUNTIME_DIR/biomeos/groundspring-{FAMILY_ID}.sock`

Fallback chain: `GROUNDSPRING_SOCKET` → `BIOMEOS_SOCKET_DIR` → `XDG_RUNTIME_DIR/biomeos/` → `/run/user/{uid}/biomeos/` → `/tmp/`

---

## For ToadStool / BarraCUDA Team

No API changes. groundSpring continues to consume `compute.execute` via capability routing. The dispatch module wraps library calls that already delegate to barracuda; the JSON-RPC layer adds no overhead to the compute path.

---

## For Provenance Trio

Trio wired at graph level, not library level:
- `provenance.session_create` — on niche deploy + validation start
- `provenance.session_dehydrate` — after results storage
- `contribution.recordDehydration` — after dehydrate

All nodes use `fallback = "skip"` — trio is an enhancement, not a requirement.

`provenance.rs` module provides convenience wrappers for the binary lifecycle (startup session, shutdown commit).

---

## Verification

- `cargo check --features biomeos`: PASS
- `cargo clippy --features biomeos -- -D warnings`: PASS
- `cargo fmt --all -- --check`: PASS
- All 908 default-feature tests: PASS
- UniBin binary builds: `cargo build --features biomeos --bin groundspring`

---

## Next Steps

1. biomeOS: test `execute_graph` with `groundspring_deploy.toml`
2. biomeOS: validate niche YAML parsing and BYOB flow
3. ToadStool: no action needed — existing `compute.execute` routing works
4. Provenance Trio: validate trio graph nodes when trio primals are available
5. Songbird: validate IPC mesh with groundSpring as niche participant
