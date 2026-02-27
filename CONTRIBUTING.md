# Contributing to groundSpring

SPDX-License-Identifier: AGPL-3.0-or-later

## Architecture

groundSpring is a **Spring** in the ecoPrimals ecosystem — a validation target
that proves Python baselines can be faithfully ported to Rust and eventually
promoted to GPU acceleration via the Write → Absorb → Lean cycle.

```
control/             Python Phase 0 experiments (28 experiments across 9 domains)
  common.py          Shared statistical primitives
  sensor_noise/      Exp 001: Bias-variance decomposition
  observation_gap/   Exp 002: Model-observation gap
  error_propagation/ Exp 003: FAO-56 Monte Carlo
  sequencing_noise/  Exp 004: Rarefaction analysis
  seismic/           Exp 005: Seismic inversion
  signal_specificity/ Exp 006: c-di-GMP Gillespie SSA
  rawr_resampling/   Exp 007: RAWR bootstrap
  anderson_localization/ Exp 008: Anderson localization
  quasiperiodic/       Exp 009: Almost-Mathieu quasiperiodic
  bistable_switching/  Exp 010: Bistable phenotypic switching
  multisignal_qs/      Exp 011: Multi-signal QS integration
  spin_transport/      Exp 012: Spin chain transport (Kachkovskiy 2016)
  resampling_convergence/ Exp 013: Resampling convergence (Lee & Liu 2024)
  drift_selection/    Exp 014: Drift vs selection (R. Anderson 2022)
  uncertainty_bridge/ Exp 015: Uncertainty bridge (sensor noise → Anderson → QS)
  rare_biosphere/     Exp 016: Rare biosphere signal detection (R. Anderson 2015)
  quasispecies_threshold/ Exp 017: Eigen error threshold (quasispecies)
  band_edge/          Exp 018: Band edge structure (transfer matrix)
  jackknife_estimation/ Exp 019: Jackknife variance (Bazavov 2025)
  freeze_out_inverse/ Exp 020: Freeze-out inverse (Bazavov 2016)
  spectral_recon/     Exp 021: Spectral reconstruction (Bazavov 2025)
crates/
  groundspring/            Rust library (26 modules)
    src/stats/             RMSE, MBE, R², IA, hit rate, Pearson/Spearman, covariance,
                           norm_cdf/ppf, chi2_statistic, mean, std, percentile (3 submodules)
    src/decompose.rs       Bias-variance decomposition, noise floor
    src/fao56.rs           FAO-56 Penman-Monteith equation chain
    src/prng.rs            Xorshift64 PRNG, Box-Muller normal sampling, binomial
    src/rarefaction.rs     Multinomial sampling, Shannon, evenness
    src/seismic.rs         Haversine, travel time, grid-search inversion
    src/gillespie.rs       Gillespie SSA for stochastic kinetics
    src/bootstrap.rs       Bootstrap + RAWR confidence intervals (bootstrap_mean delegated)
    src/anderson.rs        Anderson localization, Lyapunov exponents, analytical ξ(W,E)
    src/almost_mathieu.rs  Almost-Mathieu quasiperiodic localization, level spacing
    src/kinetics.rs       Hill-function kinetics (hill, hill_repress) shared by bistable + multisignal
    src/bistable.rs        Bistable ODE (RK4, Euler-Maruyama, BistableOde delegation)
    src/multisignal.rs     Multi-signal QS ODE (dual-signal integration, ODE delegation)
    src/transport.rs       Spin chain transport, tridiag_eigh eigenvector primitive
    src/drift.rs           Wright-Fisher fixation, Kimura fixation probability
    src/cast.rs            Centralized numeric casts (usize_f64, f64_usize, u64_f64)
    src/validate.rs        Struct-based ValidationHarness
    src/rare_biosphere.rs  Chao1, detection power, abundance-occupancy, singleton fraction
    src/quasispecies.rs    Eigen error threshold, master frequency, Wright-Fisher mutation
    src/band_structure.rs  Transfer matrix, band edge detection, periodic Hamiltonian
    src/jackknife.rs       Jackknife variance, bias correction, leave-one-out resampling
    src/freeze_out.rs     Freeze-out temperature inversion, hadron yield fitting
    src/spectral_recon.rs  Spectral reconstruction from Euclidean correlators
    src/wdm.rs             WDM transport: precision drift, size convergence, vendor parity
    src/npu.rs             NPU integration for AKD1000 neuromorphic inference (behind npu feature)
  groundspring-validate/   28 validation binaries (hotSpring pattern)
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
8. **Primal isolation.** groundSpring does not hardcode sibling primal names.
   Discovery is capability-based: scan for the needed file/module, not the
   primal name. Use `FAO56_MODULE_PATH` or `ECOPRIMALS_ROOT` env vars.
9. **No duplicate math.** If barracuda has a primitive, use it (behind feature gate).
10. **Graceful barracuda fallback.** All `#[cfg(feature = "barracuda")]` blocks
    use `if let Ok` with a CPU fallback that is always compiled. Never `.expect()`
    or `.unwrap()` on barracuda calls in production code.

## Development

### Rust

```bash
cargo test --workspace                         # 410 tests, all PASS
cargo test --workspace --features biomeos      # 442 tests (adds 32 biomeos tests)
cargo clippy --workspace -- -D warnings        # zero warnings × 4 modes
cargo fmt --check                              # clean
cargo llvm-cov --workspace                     # 99.37% workspace line coverage

# Four-mode CI: default, barracuda-CPU, barracuda-GPU, biomeos
cargo test --workspace
cargo test --workspace --features barracuda
cargo test --workspace --features barracuda-gpu
cargo test --workspace --features biomeos

# Three-mode benchmark (local vs barracuda vs barracuda-gpu)
bash scripts/three_mode_benchmark.sh

# Validation binaries (hotSpring pattern: exit 0 = pass, exit 1 = fail)
cargo run --bin validate-decompose
cargo run --bin validate-rarefaction
cargo run --bin validate-seismic
cargo run --bin validate-weather
cargo run --bin validate-fao56
cargo run --bin validate-signal-specificity
cargo run --bin validate-rawr
cargo run --bin validate-anderson
cargo run --bin validate-quasiperiodic
cargo run --bin validate-bistable
cargo run --bin validate-multisignal
cargo run --bin validate-transport
cargo run --bin validate-resampling-conv
cargo run --bin validate-drift
cargo run --bin validate-uncertainty-bridge
cargo run --bin validate-rare-biosphere
cargo run --bin validate-quasispecies
cargo run --bin validate-band-edge
cargo run --bin validate-jackknife
cargo run --bin validate-freeze-out
cargo run --bin validate-spectral-recon
cargo run --bin validate-et0-anderson
cargo run --bin validate-notill
cargo run --bin validate-aggregate
cargo run --bin validate-precision-drift
cargo run --bin validate-size-convergence
cargo run --bin validate-vendor-parity
cargo run --bin validate-npu-anderson

# metalForge live hardware validation
cargo run --bin validate-metalforge-inventory
cargo run --bin validate-metalforge-gpu
cargo run --bin validate-metalforge-cross-substrate

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
