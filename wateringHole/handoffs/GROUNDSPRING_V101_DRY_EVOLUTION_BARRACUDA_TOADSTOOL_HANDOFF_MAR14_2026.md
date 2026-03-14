# groundSpring V101 — DRY Evolution + Capability-Based Discovery

**Date**: March 14, 2026
**From**: groundSpring (V101)
**To**: toadStool/barraCuda team, biomeOS, ecoPrimals ecosystem
**Pins**: barraCuda v0.3.5, toadStool S130+ (`bfe7977b`), coralReef Iteration 10 (`d29a734`)
**License**: AGPL-3.0-only

---

## Summary

V101 is a code evolution session building on V100's deep debt audit.
Focus: DRY extraction, capability-based primal sovereignty, upstream API
drift fix, and documentation alignment across all specs and whitePaper.

### Key Changes

1. **ESNConfig upstream sync** — barraCuda's `esn_v2::ESNConfig` added
   `sgd_learning_rate`, `sgd_min_iterations`, `sgd_max_iterations`.
   groundSpring now uses `..ESNConfig::default()` struct update syntax.
   This broke `--all-features` compilation (clippy, doc, CI).

2. **DRY extraction** (net −52 lines, zero behavioral change):
   - `chi2_freeze_out()` — 4 copies of chi-squared freeze-out loop → 1 shared helper
   - `r_squared_from_residuals()` — 3 copies of R² computation → 1 shared helper
   - `validate_bootstrap_inputs()` — 4 identical assertion pairs → 1 validator
   - `const fn from_barracuda_ci()` — 4 identical BootstrapCI→BootstrapResult maps → 1

3. **Capability-based primal discovery** — `validate_nucleus_stack.rs`:
   - Replaced hardcoded `["beardog", "toadstool", "squirrel"]` with `biomeos::discover_primals()`
   - Removed hardcoded `"toadstool"` direct RPC fallbacks — pure capability routing
   - Removed hardcoded `"squirrel"` AI fallback — pure capability routing
   - Production code now has zero hardcoded primal names (only self-ID `FAMILY_ID`)

4. **Doc sovereignty** — removed "ToadStool" from `compute_execute`/`compute_submit`
   API docs, replaced with agnostic "compute provider" language.

5. **Documentation sync** — updated V98/V97→V101 across specs/, whitePaper/,
   CONTROL_EXPERIMENT_STATUS; aligned test counts to canonical 908/936.

### Verification

| Check | Result |
|-------|--------|
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | PASS (0 warnings) |
| `cargo doc --workspace --all-features --no-deps` | PASS (0 warnings) |
| `cargo test --workspace` | 908/908 PASS + 11 doc-tests |

---

## Absorption Opportunity: ESNConfig API Stability

**Problem**: barraCuda added 3 fields to `ESNConfig` without a corresponding
`pub use` of the default constants. Downstream springs cannot reference
`DEFAULT_SGD_LEARNING_RATE` etc.

**Recommendation for barraCuda**:

```rust
// esn_v2/mod.rs — re-export defaults for downstream use
pub use config::{
    ESNConfig, expect_size, validate_config,
    DEFAULT_SGD_LEARNING_RATE, DEFAULT_SGD_MIN_ITERATIONS, DEFAULT_SGD_MAX_ITERATIONS,
};
```

Until then, downstream springs use `..ESNConfig::default()` which is
resilient but loses explicit documentation of which defaults are chosen.

---

## Absorption Opportunity: Bootstrap CI Type Mapping

groundSpring maps `barracuda::stats::bootstrap::BootstrapCI` to its own
`BootstrapResult` in 4 places (now consolidated to 1). If barraCuda
exported a `From<BootstrapCI>` impl or made the type directly usable,
downstream springs could eliminate this mapping entirely.

---

## Absorption Opportunity: `chi2_freeze_out` as Barracuda Primitive

The freeze-out chi-squared evaluation:

```rust
fn chi2_freeze_out(observed: &[f64], mu_b: &[f64], t0: f64, k2: f64, inv_sigma2: f64) -> f64 {
    observed.iter().zip(mu_b.iter())
        .map(|(&o, &mu)| (o - freeze_out_curve(t0, k2, mu)).powi(2) * inv_sigma2)
        .sum()
}
```

This is a model-specific chi-squared. The pattern (observed vs parametric
model, uniform sigma) recurs across springs. If barraCuda added a generic
`chi2_parametric(observed, model_fn, sigma)` primitive, it could absorb
this pattern from all springs.

---

## Primal Sovereignty Findings

### Zero Hardcoded Primal Names in Production Code

After V101, groundSpring production code contains:
- `FAMILY_ID = "groundspring"` — self-identity only
- `SCIENCE_CAPABILITIES` array — self-knowledge of what we provide
- `discover_primals()` — runtime socket scanning, no hardcoded names
- `capability_call()` — routes by capability domain, not primal name
- All primal names in comments/docs are for human readers only

Validation binaries previously had hardcoded primal lists for health
checks. These now use `discover_primals()` → iterate whatever is live.

### Pattern for Other Springs

Springs should follow this sovereignty pattern:
1. **Self-ID**: one `FAMILY_ID` constant
2. **Self-capabilities**: array of what we provide, not what we consume
3. **Runtime discovery**: `discover_primals()` for peer awareness
4. **Capability routing**: `capability_call(socket, "domain.method", params)`
5. **Zero peer knowledge**: never hardcode another primal's name in logic

---

## Delegation Inventory (Unchanged from V100)

**102 active delegations** (61 CPU + 41 GPU). No new delegations.
Full inventory in `specs/BARRACUDA_EVOLUTION.md`.

---

## Evolution Readiness (Updated)

### Tier A — Fully Delegating (No Action)

`stats/*`, `bootstrap`, `rarefaction`, `anderson`, `drift`, `seismic`,
`freeze_out`, `jackknife`, `rare_biosphere`, `quasispecies`, `fao56`,
`gillespie`, `wdm`.

### Tier B — Minor Adaptation Needed

| Module | Status | What Changed |
|--------|--------|-------------|
| `spectral_recon` | FFT delegated, Tikhonov local | No change |
| `esn/classifier` | ESNConfig synced to v0.3.5 | Struct update syntax |

### Tier C — New Shader Required

| Module | Needed | Complexity |
|--------|--------|------------|
| `tissue_anderson` | 3D compartmented geometry kernel | Medium |
| `esn` | Full ESN training pipeline on GPU | Medium |

---

## Code Quality Snapshot

| Metric | Value |
|--------|-------|
| Rust tests (default features) | 908 |
| Rust tests (all features) | 936 |
| Python provenance tests | 287 |
| Validation checks | 395/395 (340 core + 55 NUCLEUS) |
| metalForge checks | 140 |
| Delegations | 102 (61 CPU + 41 GPU) |
| `unsafe` code | Zero (`#![forbid(unsafe_code)]`) |
| TODO/FIXME/HACK | Zero |
| Production `.unwrap()` | Zero |
| Largest file | 706 lines (`freeze_out.rs`, 36% tests) |
| Files > 1000 lines | Zero |
| Clippy warnings | Zero (pedantic + nursery) |

---

## What's Next

1. **PRNG alignment** — when barraCuda adds CPU `xoshiro128**`, align for
   bitwise reproducibility across CPU and GPU paths
2. **`BootstrapCI` type export** — eliminate mapping boilerplate
3. **ESNConfig defaults re-export** — downstream springs get explicit constants
4. **Tolerance tightening** — some validation thresholds have conservative
   margins that could narrow as baselines stabilize
5. **llvm-cov verification** — CI gates at 90%, not yet locally confirmed
