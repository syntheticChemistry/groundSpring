# Contributing to groundSpring

SPDX-License-Identifier: AGPL-3.0-or-later

## Architecture

groundSpring is a **Spring** in the ecoPrimals ecosystem — a validation target
that proves Python baselines can be faithfully ported to Rust and eventually
promoted to GPU acceleration via the Write → Absorb → Lean cycle.

```
control/             Python Phase 0 experiments (8 experiments across 5+ domains)
  common.py          Shared statistical primitives
  sensor_noise/      Exp 001: Bias-variance decomposition
  observation_gap/   Exp 002: Model-observation gap
  error_propagation/ Exp 003: FAO-56 Monte Carlo
  sequencing_noise/  Exp 004: Rarefaction analysis
  seismic/           Exp 005: Seismic inversion
  signal_specificity/ Exp 006: c-di-GMP Gillespie SSA
  rawr_resampling/   Exp 007: RAWR bootstrap
  anderson_localization/ Exp 008: Anderson localization
crates/
  groundspring/            Rust library (10 modules, 99.7% library line coverage)
    src/stats.rs           RMSE, MBE, R², IA, hit rate, mean, std, percentile
    src/decompose.rs       Bias-variance decomposition, noise floor
    src/fao56.rs           FAO-56 Penman-Monteith equation chain
    src/prng.rs            Xorshift64 PRNG, Box-Muller normal sampling
    src/rarefaction.rs     Multinomial sampling, Shannon, evenness
    src/seismic.rs         Haversine, travel time, grid-search inversion
    src/gillespie.rs       Gillespie SSA for stochastic kinetics
    src/bootstrap.rs       Bootstrap + RAWR confidence intervals
    src/anderson.rs        Anderson localization, Lyapunov exponents
    src/validate.rs        Struct-based ValidationHarness
  groundspring-validate/   8 validation binaries (hotSpring pattern)
metalForge/          Write → Absorb → Lean artifacts
  ABSORPTION_MANIFEST.md  Module-by-module absorption inventory
  shaders/                 Production WGSL shaders for ToadStool absorption
specs/               Specifications and evolution docs
whitePaper/          Study documentation
scripts/             Automation (baselines, benchmarks)
```

## Constraints

1. **AGPL-3.0-or-later only.** Every source file needs the SPDX header.
2. **1000 lines max per file.** If a file exceeds this, refactor by responsibility.
3. **No unsafe Rust.** The workspace forbids it at the lint level.
4. **Clippy pedantic + nursery** with zero warnings. `missing_docs` is `deny`.
5. **All Rust modules have doc comments** including `# Panics` sections.
6. **Deterministic.** All stochastic operations use explicit seeds. Rerun-identical.
7. **Provenance.** Every benchmark JSON has a `_provenance` block with real commit SHA.
8. **Primal isolation.** groundSpring does not hardcode sibling primal paths.
   Discovery happens at runtime.
9. **No duplicate math.** If barracuda has a primitive, use it (behind feature gate).

## Development

### Rust

```bash
cargo test --workspace          # 108 unit + 1 doc test
cargo clippy --workspace        # zero warnings required
cargo fmt --all -- --check      # clean
cargo llvm-cov --workspace --lib  # 99.7% library line coverage

# With barracuda feature gates (requires toadstool checkout):
cargo test --features barracuda     # CPU delegation (stats, bootstrap)
cargo test --features barracuda-gpu # CPU + spectral (anderson)

# Validation binaries (hotSpring pattern: exit 0 = pass, exit 1 = fail)
cargo run --bin validate-decompose
cargo run --bin validate-rarefaction
cargo run --bin validate-seismic
cargo run --bin validate-weather
cargo run --bin validate-fao56
cargo run --bin validate-signal-specificity
cargo run --bin validate-rawr
cargo run --bin validate-anderson

# Performance benchmarks (Rust vs Python)
python3 scripts/bench_rust_vs_python.py
```

### Python

```bash
python3 -m venv .venv && source .venv/bin/activate
pip install -e ".[dev]"
python3 -m pytest tests/ -v
```

## Adding a New Experiment

1. Create `control/<experiment_name>/` with a benchmark JSON and Python script.
2. Add `_provenance` to the benchmark JSON with real commit SHA.
3. Add SPDX header to all source files.
4. Port the core algorithm to `crates/groundspring/src/<module>.rs`.
5. Add a validation binary in `crates/groundspring-validate/`.
6. Add tests in `#[cfg(test)]` in the Rust module.
7. Update `specs/BARRACUDA_EVOLUTION.md` with the GPU promotion mapping.
8. Update `metalForge/ABSORPTION_MANIFEST.md` with absorption targets.

## BarraCUDA Integration

groundSpring follows the **Write → Absorb → Lean** cycle from hotSpring:

1. **Write** — Implement the algorithm as pure safe Rust in `crates/groundspring/`.
   Write production WGSL shaders in `metalForge/shaders/`.
2. **Validate** — Verify against Python baselines. All validation binaries must pass.
3. **Hand off** — Document in `metalForge/ABSORPTION_MANIFEST.md` with binding
   layouts, dispatch geometry, and CPU reference validation results.
   Create handoff doc in `wateringHole/handoffs/`.
4. **Absorb** — ToadStool/BarraCUDA team absorbs the shader as an upstream op.
5. **Lean** — Rewire groundSpring to `use barracuda::ops::*` behind `#[cfg(feature = "barracuda")]`.
   Delete local shader once upstream absorbs it.

### WGSL Shader Conventions (matching hotSpring)

- Dedicated `.wgsl` files in `metalForge/shaders/` (never inline WGSL in Rust).
- `// SPDX-License-Identifier: AGPL-3.0-or-later` at top.
- `struct Params` for uniforms (u32-aligned with padding).
- `@group(0) @binding(N)` sequential bindings, documented in header.
- `@compute @workgroup_size(64, 1, 1)` standard workgroup size.
- xoshiro128** PRNG matching `barracuda::ops::prng_xoshiro_wgsl`.
- f64 precision required for all scientific compute.
- CPU reference path documented in header comment.
- Load via `include_str!()` when integrated.

## Tolerance Philosophy

Tolerances must be:
- **Documented:** explain why this number, not another.
- **Minimal:** as tight as the algorithm allows.
- **Justified:** cite the mathematical reason (rounding, MC sampling variance, etc.).

Example:
```rust
// Tol 0.001: random_std = sqrt(RMSE² - MBE²) with 4-decimal inputs
// introduces ≤ 0.0005 rounding error
h.check_approx("random_std", computed, expected, 0.001);
```
