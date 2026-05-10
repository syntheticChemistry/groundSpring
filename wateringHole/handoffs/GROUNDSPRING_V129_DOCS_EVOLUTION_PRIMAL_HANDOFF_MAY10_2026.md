# groundSpring V129 — Documentation Evolution + Primal/Spring Team Handoff

**Date**: May 10, 2026
**From**: groundSpring V129
**To**: All primal teams, all spring teams, primalSpring audit
**Status**: 1,101 Rust tests PASS, 287 Python tests, 395/395 validation checks, 0 clippy warnings, 0 fmt diff

---

## What Changed in V129

### Deep Debt Audit — All Clean
- **No files >800 lines** — largest Rust file is 751 lines (`validate.rs`)
- **No unsafe code** — `#![forbid(unsafe_code)]` on all lib/bin roots
- **No TODO/FIXME/HACK/unimplemented!/todo!** in any Rust crate
- **No production mocks** — all mock/stub/fake confined to `#[cfg(test)]`
- **External deps justified** — all crates.io deps necessary (serde, tracing, thiserror, wgpu, tarpc, tokio, clap)

### Documentation Alignment (13 files updated)
- Test count unified to **1,101** across all living docs
- Version references unified to **V129**
- Stale `barraCuda v0.3.7` references updated to **v0.3.13**
- `SECURITY.md` updated from V124 to V129
- `PAPER_REVIEW_QUEUE.md` header modernized from V124 to V129
- `PRIMAL_INTERACTION_EVOLUTION.md` header updated to V129 with Tier 4 / biomeOS v3.51 / skunkBat
- baseCamp validation chain now documents full 4-phase evolution path

### Stale Test Fixed
- `test_exp033_tissue_anderson` marked `@pytest.mark.skip` — Exp 033 is NUCLEUS-only (no Python baseline)

---

## For Primal Teams

### barraCuda
- **110 delegations active** (67 CPU + 43 GPU)
- **Tier 4 IPC-first**: groundSpring's default build no longer links `barracuda`. All 284 `barracuda::` references are behind `#[cfg(feature = "barracuda")]`. IPC via `CompositionContext` is the default path. `--features local` enables direct library linkage for benchmarking
- **GPU gaps remaining**: `transport::tridiag_eigh` needs `linalg::eigh_f64` GPU eigenvectors; `prng::Xorshift64` vs `PrngXoshiro` alignment needs baseline regeneration
- **Kokkos parity**: `bench_gpu_vs_kokkos` and `kokkos_baseline/` are the reference anchors. No cuBLAS/MAGMA/oneMKL benchmarks — Kokkos is our GPU reference
- **Cross-substrate**: CPU ↔ GPU ↔ NPU validated via `validate_metalforge_cross_substrate.rs` (regime/scaling agreement, not bitwise due to PRNG stream differences)

### toadStool
- **S158+ pinned** — `compute.execute` and `compute.submit` validated in Exp 031 NUCLEUS stack
- **Pipeline dispatch**: metalForge validates 30 workloads (24 GPU + 2 NPU + 2 CPU-only + 2 mixed)
- **Foundation workloads**: `gs-validate-all.toml`, `gs-guidestone.toml`, `gs-bench-gpu.toml`, `gs-python-baselines.toml` all reference toadStool dispatch

### bearDog
- **Required dependency** — cryptographic identity and trust for all IPC
- **skunkBat audit**: `security.audit_log` now wired in all 6 deploy graphs (non-blocking, `fallback = "skip"`)

### songbird
- **Required dependency** — service discovery and IPC mesh
- **5-tier socket discovery**: env var → family-qualified → flat → registry.json → tmp fallback

### nestgate
- **Optional dependency** — data pipelines for NCBI, NOAA/GHCND, IRIS/FDSN
- **Exp 029–032** exercise NestGate via NUCLEUS with sovereign fallback to synthetic data

### coralReef
- **Iteration 55+ pinned** — sovereign shader compiler for GPU dispatch
- **2 production WGSL shaders**: `anderson_lyapunov`, `anderson_lyapunov_f32`
- **metalForge validates** shader outputs against CPU reference

### biomeOS
- **v3.51 absorbed**: `composition.status` (health/monitoring) + `method.register` (dynamic registration for 16 `measurement.*` methods)
- **Neural API**: Tower + Node + Squirrel validated; deploy graphs use biomeOS orchestration

### skunkBat
- **`security.audit_log`** wired into all 6 deploy graphs: `groundspring_deploy`, `groundspring_validation`, `groundspring_cross_substrate`, `groundspring_nucleus_local`, `groundspring_tower_bootstrap`, `groundspring_nucleus_node`
- **Pattern**: `by_capability = "defense"`, `fallback = "skip"`, non-blocking event forwarding

### rhizoCrypt / loamSpine / sweetGrass
- **Provenance trio** referenced in deploy graphs and foundation validation
- **foundation/deploy/foundation_validate.sh** binds validation to NestGate/rhizoCrypt/loamSpine/sweetGrass JSON-RPC hashing

---

## For Spring Teams

### Patterns to Absorb from groundSpring V129

1. **Tier 4 IPC-first**: Remove `barracuda` from `default` features, add `local = ["barracuda"]` for opt-in direct linkage. Wire `CompositionContext` for all cross-primal calls. This is the path all springs should follow per primalSpring post-interstadial guidance.

2. **biomeOS v3.51 absorption**: Implement client-side `composition.status()` and `register_methods()`. Pattern is in `crates/groundspring/src/biomeos/health.rs` and `biomeos/registration.rs`.

3. **skunkBat audit wiring**: Add `security.audit_log` nodes to deploy graphs with `fallback = "skip"`. Pattern is in all 6 `graphs/*.toml` files.

4. **Deep debt audit methodology**: Run this checklist:
   - `rg 'unsafe' --type rust` (should be zero in production)
   - `rg 'TODO|FIXME|HACK' --type rust` (should be zero)
   - `rg 'unimplemented!|todo!' --type rust` (should be zero)
   - `rg 'mock|stub|fake' --type rust` outside `#[cfg(test)]` (should be zero)
   - `wc -l` on all `.rs` files (none should exceed 800 lines)
   - `cargo clippy --workspace --all-targets` (zero warnings)

5. **Documentation alignment**: Ensure `CONTEXT.md`, `README.md`, and all spec docs reference consistent test counts, version numbers, and feature status.

6. **Feature-gate constants**: All primal names should go through `primal_names::roles::*` constants, not string literals. Feature gate names (Cargo features) are self-knowledge and can be string literals.

7. **5-tier socket discovery**: Use `primal_names::discover_socket()` pattern: env var → family-qualified → flat → registry.json → tmp fallback. Never hardcode socket paths.

### Cross-Spring Dependencies (groundSpring consumes)
| Capability | Provider | Required |
|-----------|----------|----------|
| `crypto.sign` / `crypto.verify` | bearDog | Yes |
| `discovery.find_primals` / `discovery.query` | songbird | Yes |
| `compute.execute` / `compute.submit` | toadStool | No (sovereign fallback) |
| `storage.put` / `storage.get` | nestgate | No (synthetic fallback) |
| `data.ncbi_search` / `data.ncbi_fetch` | nestgate | No |
| `data.noaa_ghcnd` | nestgate | No |
| `data.iris_stations` / `data.iris_events` | nestgate | No |
| `security.audit_log` | skunkBat | No (fallback: skip) |

### Cross-Spring Dependencies (groundSpring provides)
16 `measurement.*` capabilities registered via `method.register` in biomeOS v3.51:
- `measurement.noise_decomposition`, `measurement.anderson_validation`, `measurement.bootstrap`, `measurement.rarefaction`, `measurement.drift`, `measurement.rare_biosphere`, `measurement.gillespie`, `measurement.bistable`, `measurement.quasispecies`, `measurement.band_edge`, `measurement.parity_check`, `measurement.et0_propagation`, `measurement.freeze_out`, `measurement.regime_classification`, `measurement.spectral_features`, `measurement.uncertainty_budget`

---

## For NUCLEUS Deployment

### Foundation Integration
- **Threads 6 (Agricultural Science) + 7 (Anderson Mathematics)** in `foundation/expressions/MEASUREMENT_SCIENCE.md`
- **4 workloads**: `gs-validate-all.toml`, `gs-guidestone.toml`, `gs-bench-gpu.toml`, `gs-python-baselines.toml`
- **Targets**: `thread07_anderson_targets.toml` with reproducible numerical targets and tolerances

### Deploy Graphs
6 deploy graphs in `graphs/`, all with skunkBat audit logging:
- `groundspring_deploy.toml` — standard deployment
- `groundspring_validation.toml` — validation pipeline
- `groundspring_cross_substrate.toml` — CPU + GPU + NPU parity
- `groundspring_nucleus_local.toml` — local NUCLEUS deployment
- `groundspring_tower_bootstrap.toml` — Tower bootstrap
- `groundspring_nucleus_node.toml` — node-level NUCLEUS deployment

---

## Remaining Gaps

| Gap | Owner | Status |
|-----|-------|--------|
| Exp 002 real NOAA CDO data | groundSpring + nestgate | Pending live data download |
| Exp 012 GPU eigenvectors | barraCuda | `tridiag_eigh` → `linalg::eigh_f64` not in barracuda GPU |
| Exp 017 GPU eigenvectors | barraCuda | Partial (eigenvalues GPU, eigenvectors CPU-only) |
| PRNG alignment | barraCuda + groundSpring | `Xorshift64` vs `PrngXoshiro` baseline regeneration |
| Criterion benchmarks | groundSpring | No `#[bench]` or criterion regression suite for library crate |
| pytest 67 failures | groundSpring | Kokkos binary naming (hyphens vs underscores in scripts) |
| ToadStool env expansion | projectNUCLEUS | Gap 8: toadStool doesn't expand `${VAR}` but foundation TOMLs use `${GROUNDSPRING_ROOT}` |
