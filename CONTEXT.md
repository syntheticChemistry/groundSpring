# Context — groundSpring

## What This Is

groundSpring is a pure Rust scientific validation library and binary suite that
characterizes the gap between what models predict and what instruments measure.
It reproduces published results from 35 experiments across geochemistry, soil
physics, agricultural meteorology, mathematical physics, microbial ecology, and
inverse problems. It is part of the ecoPrimals sovereign computing ecosystem — a
collection of self-contained binaries that coordinate via JSON-RPC 2.0 over Unix
sockets, with zero compile-time coupling between components.

## Role in the Ecosystem

groundSpring is a **spring** — a validation target that proves Python scientific
baselines can be faithfully ported to pure Rust and then promoted to GPU
acceleration. While hotSpring validates clean nuclear math and airSpring validates
FAO-56 equations, groundSpring lives where those models meet the physical world:
sensor noise, measurement drift, inverse problems, and spatial propagation. It
provides labeled "dirty data" baselines that other springs and primals depend on
for noise characterization and uncertainty quantification.

## Technical Facts

- **Language:** Rust library + validators, Python Phase 0 baselines + notebooks. Zero C dependencies (`cargo-deny` enforced)
- **Architecture:** 5-crate workspace (`groundspring` library, `groundspring-validate` binaries, `groundspring-forge` GPU/hardware dispatch, `exp094_composition_parity`, `exp095_measurement_niche`)
- **Eukaryotic UniBin:** Single binary (`groundspring_unibin`) with `certify`, `validate`, `status`, `version` subcommands via `clap`
- **Certification organelle:** `certification/` module — Properties 1-5 (bare, Tier 1) + Layers 2-4 (NUCLEUS composition, Tier 2)
- **Validation scenarios:** `validation/scenarios/` registry — 10 tracks with `ScenarioMeta` (id, track, tier, provenance)
- **IPC tree:** `src/ipc/` with per-primal modules (`barracuda.rs`, `toadstool.rs`, `nestgate.rs`, `beardog.rs`, `songbird.rs`, `skunkbat.rs`)
- **Communication:** JSON-RPC 2.0 over Unix domain sockets (behind `biomeos` feature); `tarpc` IPC (behind `tarpc-ipc` feature)
- **License:** AGPL-3.0-or-later
- **Tests:** 1,101 Rust tests + 287 Python provenance tests
- **Clippy:** Zero warnings on all targets (lib + bin + test), pedantic + nursery
- **Coverage:** ≥92% library line coverage (`cargo llvm-cov --workspace --lib`)
- **MSRV:** Rust 1.87 (2024 edition)
- **Validation checks:** 395/395 PASS across 35 binaries
- **guideStone:** Level 4 (bare + NUCLEUS composition parity)
- **barraCuda delegations:** 110 active (67 CPU + 43 GPU) — `barracuda` is `optional = true`, feature-gated
- **Logging:** Unified `tracing` (zero `log::` calls)
- **Fossil record:** `fossilRecord/` with 3 dated prokaryotic snapshots (validate binaries, guidestone, experiment crates)
- **primalSpring:** v0.9.25 pinned for `CompositionContext`, `ScenarioMeta`, `ScenarioRegistry`
- **Tier 4 IPC-first:** `barracuda` removed from default features; IPC via `CompositionContext` is the default; `local` feature for opt-in direct library linkage
- **biomeOS v3.51:** `composition.status` (health/monitoring) + `method.register` (dynamic registration) absorbed
- **skunkBat:** `security.audit_log` wired into all 6 deploy graphs (non-blocking, `fallback = "skip"`)

## Key Capabilities (JSON-RPC methods)

When running as a biomeOS primal (`--features biomeos`), groundSpring exposes
16 `measurement.*` methods via JSON-RPC (registered in `capability_registry.toml`):

- `measurement.noise_decomposition` — Bias-variance error decomposition (RMSE, MBE, R², IA)
- `measurement.anderson_validation` — Anderson localization Lyapunov exponent
- `measurement.bootstrap` — Confidence interval estimation (mean, median, std)
- `measurement.rarefaction` — Multinomial rarefaction for sequencing noise
- `measurement.drift` — Drift vs selection (Wright-Fisher, Kimura)
- `measurement.rare_biosphere` — Rare biosphere detection (Chao1)
- `measurement.gillespie` — Stochastic chemical kinetics (SSA)
- `measurement.bistable` — Bistable phenotypic switching (c-di-GMP)
- `measurement.quasispecies` — Eigen quasispecies error threshold
- `measurement.band_edge` — Band structure of periodic tight-binding chains
- `measurement.parity_check` — Cross-substrate parity validation (CPU vs GPU)
- `measurement.et0_propagation` — FAO-56 Penman-Monteith ET₀ uncertainty
- `measurement.freeze_out` — Freeze-out chi-squared fitting
- `measurement.regime_classification` — ESN-based regime classification
- `measurement.spectral_features` — Spectral function reconstruction (Tikhonov)
- `measurement.uncertainty_budget` — Multi-source uncertainty budget

## What This Does NOT Do

- Does not compile WGSL shaders (that is coralReef)
- Does not manage hardware discovery or dispatch (that is toadStool)
- Does not own GPU math primitives (those live in barraCuda; groundSpring consumes them)
- Does not perform ML training or inference (that is neuralSpring)
- Does not orchestrate primal lifecycles (that is biomeOS)

## Related Repositories

- [wateringHole](https://github.com/ecoPrimals/wateringHole) — ecosystem standards and registry
- [barraCuda](https://github.com/ecoPrimals/barraCuda) — GPU math primitives (consumed via feature flags)
- [toadStool](https://github.com/ecoPrimals/toadStool) — hardware discovery and compute orchestration
- [coralReef](https://github.com/ecoPrimals/coralReef) — sovereign GPU compiler
- [primalSpring](https://github.com/ecoPrimals/primalSpring) — ecosystem intermediary, certification/validation patterns
- [hotSpring](https://github.com/syntheticChemistry/hotSpring) — thermal simulation (cross-spring shader source)
- [wetSpring](https://github.com/syntheticChemistry/wetSpring) — microbiome ecology (cross-spring bio shaders)
- [airSpring](https://github.com/syntheticChemistry/airSpring) — atmospheric science (FAO-56 source)

## Design Philosophy

groundSpring is built using AI-assisted constrained evolution. Rust's compiler
constraints (ownership, lifetimes, type system) reshape the fitness landscape and
drive specialization. The evolution path is: Python baseline → Rust validation →
GPU acceleration (barraCuda) → sovereign pipeline (coralReef/toadStool) →
eukaryotic UniBin (certification + validation scenarios). Every tolerance is a
named constant with documented provenance. Every validation binary follows the
hotSpring pattern: hardcoded expected values, explicit PASS/FAIL per check,
exit 0 on all pass / exit 1 on any failure.
