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

- **Language:** 100% Rust, zero C dependencies (`cargo-deny` enforced)
- **Architecture:** 3-crate workspace (`groundspring` library, `groundspring-validate` binaries, `groundspring-forge` GPU/hardware dispatch)
- **Communication:** JSON-RPC 2.0 over Unix domain sockets (behind `biomeos` feature)
- **License:** AGPL-3.0-or-later
- **Tests:** 990+ Rust tests + 287 Python provenance tests
- **Coverage:** ≥92% library line coverage (`cargo llvm-cov --workspace --lib`)
- **MSRV:** Rust 1.87 (2024 edition)
- **Crate count:** 3 workspace crates
- **Validation checks:** 395/395 PASS across 34 binaries
- **barraCuda delegations:** 110 active (67 CPU + 43 GPU)

## Key Capabilities (JSON-RPC methods)

When running as a biomeOS primal (`--features biomeos`), groundSpring exposes
16 `measurement.*` methods via JSON-RPC:

- `measurement.decompose` — Bias-variance decomposition
- `measurement.rmse`, `measurement.mbe` — Agreement metrics
- `measurement.hit_rate` — Precipitation occurrence detection
- `measurement.rarefaction` — Taxonomic rarefaction curves
- `measurement.diversity` — Shannon/Simpson diversity indices
- `measurement.bootstrap` — Confidence interval estimation
- `measurement.anderson` — Anderson localization diagnostics
- `measurement.seismic` — Travel time and source inversion
- `measurement.gillespie` — Stochastic chemical kinetics
- Plus NestGate data pipeline methods (NCBI, NOAA GHCND, IRIS FDSN)

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
- [hotSpring](https://github.com/syntheticChemistry/hotSpring) — thermal simulation (cross-spring shader source)
- [wetSpring](https://github.com/syntheticChemistry/wetSpring) — microbiome ecology (cross-spring bio shaders)
- [airSpring](https://github.com/syntheticChemistry/airSpring) — atmospheric science (FAO-56 source)

## Design Philosophy

groundSpring is built using AI-assisted constrained evolution. Rust's compiler
constraints (ownership, lifetimes, type system) reshape the fitness landscape and
drive specialization. The evolution path is: Python baseline → Rust validation →
GPU acceleration (barraCuda) → sovereign pipeline (coralReef/toadStool). Every
tolerance is a named constant with documented provenance. Every validation binary
follows the hotSpring pattern: hardcoded expected values, explicit PASS/FAIL per
check, exit 0 on all pass / exit 1 on any failure.
