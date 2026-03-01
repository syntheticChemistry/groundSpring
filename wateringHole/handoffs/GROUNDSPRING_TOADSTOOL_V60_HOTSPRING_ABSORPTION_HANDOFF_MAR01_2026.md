# groundSpring → ToadStool V60 Handoff: hotSpring Cross-Spring Absorption

**Date**: March 1, 2026
**From**: groundSpring (V60)
**To**: ToadStool / BarraCUDA team
**ToadStool pin**: S71+++ (`8dc01a37`)
**License**: AGPL-3.0-or-later
**Supersedes**: V59 (ToadStool S71+++ Catch-Up)

---

## Executive Summary

- **hotSpring cross-spring absorption**: DriftMonitor, ClassificationUncertainty,
  concept edge detection — 3 native implementations inspired by hotSpring/Nautilus
- **Nautilus Shell optional dep**: `bingocube-nautilus` wired as `nautilus` feature
  gate — evolutionary reservoir computing accessible to groundSpring consumers
- **620 workspace tests** (+7 from V59), all PASS, all quality gates green
- **61 active delegations**: 37 CPU + 20 GPU + 4 cross-spring (unchanged from V59)
- **No new barracuda delegations** — V60 adds native algorithms that could become
  future barracuda candidates

---

## Part 1: What groundSpring Absorbed from hotSpring/Nautilus

### 1.1 DriftMonitor (`drift.rs`)

**Lineage**: bingoCube Nautilus Shell `constraints.rs` → groundSpring native

Monitors `N_e`·`s` (effective population size × selection coefficient) across
generations to detect when genetic drift overwhelms selection:

```rust
pub struct DriftMonitor {
    history: Vec<(usize, f64)>,
    drift_threshold: f64,
    consecutive_drift: usize,
}

pub enum DriftAction {
    Continue,
    IncreaseSelection,
    IncreasePopulation,
}
```

- Records `(generation, N_e * s)` per generation
- Detects drift when `N_e·s < threshold` for 3+ consecutive generations
- Recommends action: continue, increase selection pressure, or increase population
- 5 tests covering strong selection, drift detection, prolonged drift, recovery, custom threshold

**barracuda candidate**: `DriftMonitor` is lightweight (scalar tracking), but
the underlying `N_e·s` computation at scale (large populations, many generations)
could benefit from GPU batch computation.

### 1.2 ClassificationUncertainty (`esn.rs`)

**Lineage**: hotSpring `MultiHeadNpu` `HeadGroupDisagreement` → groundSpring native

Epistemic uncertainty metrics for regime classification:

```rust
pub struct ClassificationUncertainty {
    pub confidence: f64,
    pub entropy: f64,
    pub margin: f64,
}
```

- `confidence`: max probability across classes
- `entropy`: Shannon entropy of the distribution (normalized)
- `margin`: gap between top-2 probabilities
- `is_boundary(confidence_threshold, margin_threshold)`: detects regime transitions
- 3 tests (confident, boundary, empty input)

**barracuda candidate**: `classification_uncertainty()` is a softmax normalization +
entropy computation — could be a `FusedMapReduceF64` variant for batch classification.

### 1.3 Concept Edge Detection (`esn.rs`)

**Lineage**: bingoCube Nautilus Shell `NautilusBrain::detect_concept_edges()` → groundSpring native

Leave-one-out cross-validation over disorder sweep data to identify parameter regions
where the model breaks down (regime boundaries):

```rust
pub fn detect_concept_edges(
    disorder_values: &[f64],
    target_labels: &[f64],
    threshold: f64,
) -> Vec<(f64, f64)>
```

- Returns `(disorder_value, loo_error)` pairs where error exceeds threshold
- Uses linear interpolation from neighboring points as LOO prediction
- Identifies phase transition boundaries in Anderson localization sweeps
- 2 tests (transition detection, insufficient points)

**barracuda candidate**: LOO cross-validation over N points requires N predictions
— embarrassingly parallel across leave-one-out subsets. Good GPU candidate for
large disorder sweeps (N > 1000).

### 1.4 Nautilus Feature Gate

```toml
[features]
nautilus = ["dep:bingocube-nautilus"]

[dependencies]
bingocube-nautilus = { path = "../../../primalTools/bingoCube/nautilus", optional = true }
```

Re-exports the full Nautilus Shell API when enabled: `NautilusShell`, `NautilusBrain`,
`DriftMonitor` (Nautilus version), `EdgeSeeder`, `Akd1000Export`.

---

## Part 2: Cross-Spring Concepts for ToadStool Evolution

### 2.1 Evolutionary Reservoir Computing (Nautilus Shell)

The Nautilus Shell in `primalTools/bingoCube/nautilus/` implements evolutionary
reservoir computing on BingoCube boards. Key concepts ToadStool may want to absorb:

| Concept | Description | barracuda Potential |
|---------|-------------|-------------------|
| `NautilusShell` | Evolutionary population on a BingoCube board | Board state management |
| `DriftMonitor` | `N_e·s` tracking with configurable thresholds | Lightweight, stays CPU |
| `EdgeSeeder` | Seed population members at concept edges | Population diversity ops |
| `Akd1000Export` | Int4 weight quantization for NPU export | NPU pipeline accelerator |
| `ChamberConfig` | Board geometry constraints | GPU dispatch topology |

### 2.2 Brain Architecture (hotSpring)

hotSpring's `BIOMEGATE_BRAIN_ARCHITECTURE.md` describes a 4-substrate concurrent
system: NPU (cerebellum), RTX 3090 (motor cortex), Titan V (pre-motor), CPU
(prefrontal cortex). Key pattern: **substrate-specific specialization** with
message-passing coordination. This maps to ToadStool's `metalForge` dispatch —
each substrate handles what it's best at.

### 2.3 Multi-Head ESN Disagreement (hotSpring)

hotSpring's Gen 2 ESN uses multiple reservoir heads with `HeadGroupDisagreement`
to measure epistemic uncertainty. When heads disagree on regime classification,
the system is near a phase boundary. groundSpring's `ClassificationUncertainty`
is a lightweight analog — but ToadStool could evolve `esn_v2` to natively
support multi-head disagreement metrics as a first-class GPU operation.

---

## Part 3: Recommended barracuda Evolutions

| Priority | Candidate | Description | GPU Benefit |
|----------|-----------|-------------|------------|
| Medium | `classification_uncertainty_batch` | Batch softmax + entropy over N samples | FusedMapReduceF64 variant |
| Medium | `loo_cross_validate` | Leave-one-out prediction + error over N sweep points | Embarrassingly parallel (N independent) |
| Low | `drift_monitor_batch` | Track `N_e·s` across G generations for P populations | Batch WrightFisher + scalar tracking |
| Low | `esn_multi_head_disagreement` | Multiple reservoir heads with disagreement metric | Extends existing `esn_v2` |

These are lower priority than the existing Tier B items (PRNG alignment, grid search
ops) but represent the emerging cross-spring capability surface.

---

## Part 4: Quality State

| Gate | Status |
|------|--------|
| `cargo check` (default) | PASS |
| `cargo check --features barracuda` | PASS |
| `cargo check --features barracuda-gpu` | PASS |
| `cargo check --features nautilus` | PASS |
| `cargo clippy -- -D warnings` | 0 warnings |
| `cargo fmt --check` | PASS |
| `cargo test --workspace` | 620 PASS |

---

## Part 5: Delegation Inventory (V60 Current)

No new delegations in V60 — the absorption is native implementations:

| Tier | Count | Notes |
|------|-------|-------|
| CPU active | 37 | unchanged from V59 |
| GPU active | 20 | unchanged from V59 |
| Cross-spring | 4 | unchanged from V59 |
| Evolution candidates | 1 | band_edges (algorithm mismatch) |
| **Total active** | **61** | |
| **New native functions** | **4** | DriftMonitor, DriftAction, ClassificationUncertainty, detect_concept_edges |
| **New tests** | **10** | 5 drift + 3 uncertainty + 2 concept edge |

---

## Part 6: What Changed from V59

| Aspect | V59 | V60 |
|--------|-----|-----|
| Focus | ToadStool S71+++ rewiring | hotSpring/Nautilus absorption |
| Delegations | 61 (37+20+4) | 61 (unchanged) |
| Tests | 613 | 620 (+7 new) |
| New modules | — | DriftMonitor, ClassificationUncertainty, concept edges |
| New deps | — | `bingocube-nautilus` (optional) |
| ToadStool pin | S71+++ (8dc01a37) | S71+++ (unchanged) |

---

*groundSpring V60 — hotSpring Cross-Spring Absorption — March 1, 2026*
