# Contributing to groundSpring

SPDX-License-Identifier: AGPL-3.0-or-later

## Architecture

groundSpring is a **Spring** in the ecoPrimals ecosystem — a validation target
that proves Python baselines can be faithfully ported to Rust and eventually
promoted to GPU acceleration.

```
control/             Python Phase 0 experiments (5 pillars)
  common.py          Shared statistical primitives
  sensor_noise/      Exp 001: Bias-variance decomposition
  observation_gap/   Exp 002: Model-observation gap
  error_propagation/ Exp 003: FAO-56 Monte Carlo
  sequencing_noise/  Exp 004: Rarefaction analysis
  seismic/           Exp 005: Seismic inversion
crates/
  groundspring/      Rust library (Phase 1)
  groundspring-validate/  Validation binaries (hotSpring pattern)
tests/               Python pytest suite
specs/               Specifications and evolution docs
whitePaper/          Study documentation
```

## Constraints

1. **AGPL-3.0-or-later only.** Every source file needs the SPDX header.
2. **1000 lines max per file.** If a file exceeds this, refactor by responsibility.
3. **No unsafe Rust.** The workspace forbids it at the lint level.
4. **Clippy pedantic + nursery** with zero warnings.
5. **All Rust modules have doc comments** including `# Panics` sections.
6. **Deterministic.** All stochastic operations use explicit seeds. Rerun-identical.
7. **Provenance.** Every benchmark JSON has a `_provenance` block.
8. **Primal isolation.** groundSpring does not hardcode sibling primal paths.
   Discovery happens at runtime.

## Development

### Python

```bash
python3 -m venv .venv && source .venv/bin/activate
pip install -e ".[dev]"    # or: pip install numpy scipy pandas requests pytest ruff
python3 -m pytest tests/ -v
bash scripts/run_all_baselines.sh
```

### Rust

```bash
cargo test --workspace
cargo clippy --workspace   # zero warnings required
cargo fmt --check

# Validation binaries (hotSpring pattern: exit 0 = pass, exit 1 = fail)
cargo run --bin validate-decompose
cargo run --bin validate-rarefaction
cargo run --bin validate-seismic
```

## Adding a New Experiment

1. Create `control/<experiment_name>/` with a benchmark JSON and Python script.
2. Add `_provenance` to the benchmark JSON.
3. Add SPDX header to all source files.
4. Port the core algorithm to `crates/groundspring/src/<module>.rs`.
5. Add a validation binary in `crates/groundspring-validate/`.
6. Add tests in `tests/` and `#[cfg(test)]` in the Rust module.
7. Update `specs/BARRACUDA_EVOLUTION.md` with the GPU promotion mapping.

## Tolerance Philosophy

Tolerances must be:
- **Documented:** explain why this number, not another.
- **Minimal:** as tight as the algorithm allows.
- **Justified:** cite the mathematical reason (rounding, MC sampling variance, etc.).

Example:
```python
# Tol 0.001: random_std = sqrt(RMSE² - MBE²) with 4-decimal inputs
# introduces ≤ 0.0005 rounding error
check_approx("random_std", computed, expected, 0.001)
```
