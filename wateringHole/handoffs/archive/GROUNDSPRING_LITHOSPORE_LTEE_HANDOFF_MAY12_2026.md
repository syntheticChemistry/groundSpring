# groundSpring → lithoSpore LTEE Data Handoff

**Date**: May 12, 2026
**From**: groundSpring (V138)
**To**: lithoSpore team (new spinup)
**Status**: B1–B4 ALL COMPLETE — ready for ingestion

---

## What You're Getting

groundSpring has completed 4 LTEE reproductions. Each has:
1. A **Python baseline** (Tier 1) that produces reference values
2. An **`expected_values.json`** with computed parameters for validation
3. A **Rust validation binary** (Tier 2) that independently reproduces the result
4. A **benchmark JSON** with frozen experiment parameters
5. `--format json` CLI flag for structured output (projectNUCLEUS Tier 2 ingestion)

All artifacts are validated: Python and Rust agree. Zero manual intervention needed.

---

## Module Mapping

| lithoSpore Module | groundSpring Source | Paper | Exp | Checks |
|-------------------|---------------------|-------|-----|--------|
| `ltee-fitness` (module 1) | `control/ltee_fitness_dynamics/` | Wiser et al. 2013 *Science* | 036 | 10/10 |
| `ltee-mutation` (module 2) | `control/ltee_neutral_mutation/` | Barrick et al. 2009 *Nature* | 037 | 8/8 |
| `ltee-clonal` (module 3) | `control/ltee_clonal_interference/` | Good et al. 2017 *Nature* | 038 | 7/7 |
| `ltee-citrate` (module 4) | `control/ltee_citrate_innovation/` | Blount et al. 2008/2012 | 039 | 8/8 |

---

## Per-Module Details

### Module 1: `ltee-fitness` — Wiser et al. 2013 Fitness Dynamics

**Scientific content**: Power-law, hyperbolic, and logarithmic models fit to 50,000-generation LTEE fitness trajectories. Jackknife variance estimation. AIC/BIC model selection (power-law wins).

**Key files**:
- `control/ltee_fitness_dynamics/expected_values.json` — model fit parameters (RSS, R², AIC, BIC), jackknife estimates, 50 generations × mean_fitness data series
- `control/ltee_fitness_dynamics/benchmark_ltee_fitness.json` — frozen experiment parameters
- `control/ltee_fitness_dynamics/ltee_fitness_dynamics.py` — Python baseline (9/9 checks)
- `crates/groundspring-validate/src/validate_ltee_fitness.rs` — Rust binary (10/10 checks)

**Ingestion notes**: The `expected_values.json` has `model_fits.power_law`, `.hyperbolic`, `.logarithmic` blocks with RSS/R²/AIC/BIC. Power-law fit: `w(t) = 1 + A·t^b`. BLAKE3-hashable.

### Module 2: `ltee-mutation` — Barrick et al. 2009 Neutral Mutation

**Scientific content**: Kimura fixation probability (1/N for neutral mutations), molecular clock rate validation (μ per generation), Wright-Fisher drift dynamics.

**Key files**:
- `control/ltee_neutral_mutation/expected_values.json` — fixation probability, molecular clock rate, Pearson r, drift metrics
- `control/ltee_neutral_mutation/benchmark_ltee_neutral.json` — frozen experiment parameters
- `control/ltee_neutral_mutation/ltee_neutral_mutation.py` — Python baseline (8/8 checks)
- `crates/groundspring-validate/src/validate_ltee_neutral.rs` — Rust binary (8/8 checks)

**Ingestion notes**: Core values are `kimura_fixation_probability` (expected: 1/N) and `molecular_clock_rate` (expected: μ per generation). Both are scalar floats.

### Module 3: `ltee-clonal` — Good et al. 2017 Clonal Interference

**Scientific content**: Wright-Fisher simulation at 4 population sizes (100, 1K, 10K, 100K). Demonstrates fixation probability monotonically decreasing with N (clonal interference regime), log-fitness adaptation rate scaling sublinearly.

**Key files**:
- `control/ltee_clonal_interference/expected_values.json` — per-population-size fixation rates, adaptation rates, trajectory statistics
- `control/ltee_clonal_interference/benchmark_ltee_clonal.json` — frozen experiment parameters
- `control/ltee_clonal_interference/ltee_clonal_interference.py` — Python baseline (7/7 checks)
- `crates/groundspring-validate/src/validate_ltee_clonal.rs` — Rust binary (7/7 checks)

**Ingestion notes**: Contains `trajectories` with per-population-size data and `population_sizes` array. The key result is the monotonic decrease of fixation probability with N.

### Module 4: `ltee-citrate` — Blount et al. 2008/2012 Citrate Innovation

**Scientific content**: Two-hit potentiation-actualization cascade. Models the historical contingency of the Cit+ key innovation — replay experiments from earlier clones have lower probability of evolving Cit+ than replays from later (potentiated) clones.

**Key files**:
- `control/ltee_citrate_innovation/expected_values.json` — potentiation fractions, Cit+ fractions, replay probabilities, analytical vs empirical two-hit waiting times
- `control/ltee_citrate_innovation/benchmark_ltee_citrate.json` — frozen experiment parameters
- `control/ltee_citrate_innovation/ltee_citrate_innovation.py` — Python baseline (8/8 checks)
- `crates/groundspring-validate/src/validate_ltee_citrate.rs` — Rust binary (8/8 checks)

**Ingestion notes**: `replay_probabilities` map is keyed by generation offset. Calibrated for rare events (p_pot=2e-6, p_act=5e-5 per generation).

---

## How to Ingest

1. **BLAKE3-hash** each `expected_values.json` for content-addressable storage
2. **Parse** the JSON — all values are flat or one-level-nested, no complex types
3. **Validate** by running the Rust binary: `cargo run --bin validate_ltee_<module> -- --format json`
4. **CI integration**: Each binary exits 0 on PASS, 1 on FAIL. JSON output has `{"status": "PASS", "checks": N, "passed": N, ...}`

---

## PRNG Note

groundSpring uses `Xorshift64` for all stochastic simulations. Python baselines use `numpy.random.default_rng` (PCG64). The two PRNGs are **not bit-identical** — validation is statistical (distribution properties), not sequence-identical. This is by design: the reproductions validate the science (statistical properties of the models), not the random number generator.

GAP-GS-011 tracks the future migration to xoshiro128** for GPU alignment. This does not affect lithoSpore ingestion — the `expected_values.json` files contain statistical parameters, not raw trajectories.

---

## Open Items (Not Blocking lithoSpore)

| Item | Status | Notes |
|------|--------|-------|
| B5+ queue (B6–B9) | QUEUED | Not started — not on critical path for lithoSpore modules 1–4 |
| coralReef IPC | Stub exists | Blocked on coralReef SM rebuild (GAP-GS-002) |
| PRNG Phase 2b | Deferred | Blocked on barraCuda team (GAP-GS-011) |

---

## Contact

groundSpring artifacts live at `springs/groundSpring/control/ltee_*/`. The Rust validation binaries are in `crates/groundspring-validate/src/validate_ltee_*.rs`. The provenance registry in `crates/groundspring-validate/src/lib.rs` lists all 31 benchmarks including the 4 LTEE entries.

Foundation Thread 5 (LTEE evolutionary dynamics) and Thread 7 (Anderson Mathematics) both reference groundSpring data. Your ingestion of modules 1–4 closes the Pillar 4 exit gate.
