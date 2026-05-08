# groundSpring V125 — Downstream Absorption Handoff

**Date**: May 8, 2026
**From**: groundSpring V125 (guideStone Level 4)
**To**: foundation, sporeGarden, projectNUCLEUS, all spring teams
**barraCuda**: v0.3.13 | **guideStone**: Level 4

---

## Purpose

This handoff documents how downstream systems can absorb groundSpring's patterns,
validation data, and evolution for their own integration. It covers: foundation
data targets, sporeGarden workloads, sporePrint notebook pattern, composition
experiment scaffold, and deployment patterns for projectNUCLEUS.

---

## 1. For foundation (sporeGarden/foundation)

### Thread 7 Anderson Targets (created)

`data/targets/thread07_anderson_targets.toml` defines 18 groundSpring-specific
validation targets:

| Target | Domain |
|--------|--------|
| Lyapunov exponent (1D–4D) | Anderson localization |
| Localization length | Anderson transport |
| Almost-Mathieu operator | Quasiperiodic spectral theory |
| Band structure detection | Filonov-Kachkovskiy |
| Freeze-out curve fit | Bazavov lattice QCD |
| Spectral reconstruction | Tikhonov regularization |
| Tissue Anderson | Gonzales immunopharmacology |
| NUCLEUS composition parity | Tower / Node / Nest / Cross-atomic / Measurement niche |

### foundation_validate.sh (fixed)

`deploy/foundation_validate.sh` `SCAN_DIRS` now includes spring-specific
directories (`groundspring/`, `airspring/`, `wetspring/`, etc.) in addition
to `thread*` directories. This ensures workloads defined under
`workloads/groundspring/` are discovered and executed.

---

## 2. For sporeGarden (projectNUCLEUS)

### Workloads (created)

Four TOML workloads in `workloads/groundspring/` mirror foundation's definitions
but with `working_dir` adjusted for ironGate deployment (`/opt/ecoPrimals/springs/groundSpring`):

| Workload | Command | Purpose |
|----------|---------|---------|
| `gs-validate-all.toml` | `cargo test --workspace` | Full test suite |
| `gs-guidestone.toml` | `cargo run --bin groundspring_guidestone` | guideStone L4 validation |
| `gs-bench-gpu.toml` | `cargo run --bin benchmark_cross_spring` | GPU benchmark |
| `gs-python-baselines.toml` | `pytest tests/ -v` | Python baseline verification |

### PHASES.md (updated)

groundSpring added to the "Spring science hubs" list under
"sporePrint Integration (primals.eco/lab) — LIVE".

---

## 3. For Other Spring Teams — sporePrint Pattern

### How to replicate

1. **Create frozen data**: `experiments/results/*.json` — dump validation
   outputs as JSON. These are your static snapshots. Load from notebooks.
2. **Copy NOTEBOOK_PATTERN.md**: From `notebooks/NOTEBOOK_PATTERN.md` (mirrors
   the wetSpring/primalSpring pattern).
3. **Create 5 notebooks**:
   - 01: Composition/deploy validation
   - 02: Benchmark comparison (Rust vs Python)
   - 03: Ecosystem evidence (experiments, gaps, timeline)
   - 04: Cross-spring connections (primal consumption matrix)
   - 05: Domain deep dive (security, tolerance, or your specialty)
4. **Update sporeprint/validation-summary.md** with headline numbers.
5. **Push to main** — `notify-sporeprint.yml` fires automatically.

### Key conventions

- Load frozen data via `Path('..') / 'experiments' / 'results'` — no live primals
- Use matplotlib: `#2ecc71` (pass), `#e74c3c` (fail), `#3498db` (info)
- End each notebook with provenance summary linking to primals.eco
- All cells must execute cleanly: `jupyter nbconvert --execute`

### Composition experiment scaffold

primalSpring's `exp095_proto_nucleate_template` is the scaffold for creating
your own composition parity experiment. Each spring should:

1. Clone the template to `experiments/exp094_composition_parity/`
2. Replace `groundspring` method calls with your domain capabilities
3. Validate Tower + Node + Nest + cross-atomic pipeline
4. Add to workspace `Cargo.toml` members

groundSpring's `exp094_composition_parity` and `exp095_measurement_niche` are
working references.

---

## 4. For projectNUCLEUS Deployment

### Deploy graph patterns

groundSpring's 6 deploy graphs demonstrate the canonical deployment patterns:

#### Minimal (tower_bootstrap)
Tower with BearDog + Songbird. Registers all capabilities, runs health check.
Good for CI and smoke tests.

#### Full local (nucleus_local)
All 4 primals (BearDog, Songbird, ToadStool, NestGate) plus groundSpring.
Validates live data routes (NCBI, NOAA, IRIS) and compute dispatch.

#### Node atomic (nucleus_node)
Tower + ToadStool only. Runs all validation binaries through GPU dispatch.
Compares CPU reference against GPU output.

#### Cross-substrate (cross_substrate)
metalForge discovery → CPU/GPU/NPU compute → parity check → provenance trio.
Validates hardware-agnostic correctness.

### Capability registration

All graphs use `capability.register` (not `registry.register`) with the full
16-method + 2-health capability set. The registration step should come after
Tower health is confirmed.

### Health contract

All deployed springs should implement:
- `health.liveness` — returns 200 if process is alive
- `health.readiness` — returns 200 if all capabilities are operational

These map to Kubernetes liveness/readiness probes via biomeOS.

### Provenance lineage

All graphs include a `provenance.session_create` step that establishes lineage.
The session ID flows through subsequent validation steps. Use
`provenance.session_close` at the end with a summary of results.

---

## 5. Tolerance and Test Patterns

### Tolerance tiers

groundSpring defines 4 tolerance tiers in `crates/groundspring/src/tol.rs`:

| Tier | Value | Use |
|------|-------|-----|
| EXACT | 1e-15 | Deterministic algorithms, bitwise reproducible |
| ANALYTICAL | 1e-6 | Analytical solutions, known-answer tests |
| STOCHASTIC | 0.05 | Stochastic algorithms, Monte Carlo |
| QUANTIZED | 0.25 | NPU int8 round-trip quantization |

These are the canonical constants for the ecosystem. metalForge's
`ToleranceTier` enum delegates to these. Other springs should import or
mirror this tier structure.

### Test module pattern

Every `.rs` source file should have a `#[cfg(test)]` module at the bottom.
Even for modules that need live NUCLEUS connections, you can test:
- Constants (positivity, ordering, range)
- Structural properties (non-empty arrays, valid enums)
- Error paths (nonexistent sockets, invalid inputs)
- JSON serialization round-trips
- Metadata (capability counts, domain strings)

groundSpring added 29 such modules in V125 — see the CHANGELOG for the
complete list.
