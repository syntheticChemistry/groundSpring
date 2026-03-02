# groundSpring → ToadStool/BarraCUDA V68 Handoff

**Date**: March 2, 2026
**groundSpring version**: V68
**ToadStool pin**: S86 (`7e01ac7e`)
**Delegations**: 44 CPU + 32 GPU = 76 active
**metalForge**: 30 workloads, 30 tolerance specs
**Tests**: 776 passed, 0 failed
**Debt**: zero unsafe / zero TODO / zero .unwrap() / zero #[allow] without reason

---

## Executive Summary

V68 completes the modern ToadStool S86 rewiring cycle. Three new barracuda
delegations wire into groundSpring, each demonstrating cross-spring evolution:

| Delegation | barracuda API | Cross-Spring Origin | Domain Transfer |
|---|---|---|---|
| `lbfgs_refine_barracuda` | `optimize::lbfgs_numerical` | airSpring V035 param fitting → S84 | Agriculture → nuclear physics |
| `tissue_4d_simulation` | `spectral::anderson::anderson_4d` | hotSpring S26 spectral → S84 | Condensed matter → immunology |
| `tissue_4d_rg_coarsen` | `spectral::anderson::wegner_block_4d` | hotSpring condensed matter → S84 | Lattice RG → tissue clustering |

---

## New Delegations

### 1. L-BFGS Post-Grid-Search Refinement (`freeze_out.rs`)

**What**: After the 2D grid search finds the coarse (T₀, κ₂) optimum for the
QCD freeze-out curve, L-BFGS with numerical gradient refines to sub-grid precision.

**barracuda API**:
```rust
barracuda::optimize::lbfgs_numerical(objective, &x0, &config) -> Result<LbfgsResult>
```

**Config used**:
- `memory: 5`, `max_iter: 200`, `gtol: 1e-12`, `ftol: 1e-15`
- `c1: 1e-4`, `c2: 0.9`, `max_linesearch: 40`

**Cross-spring lineage**: airSpring V035 needed L-BFGS for ET₀ parameter
calibration → absorbed into ToadStool S84 → groundSpring uses it to refine
QCD freeze-out fits (Bazavov et al. 2016).

**Fallback**: If `barracuda` feature disabled or L-BFGS result is worse than
grid search, returns the grid-search result unchanged.

### 2. 4D Anderson Tissue Simulation (`tissue_anderson/mod.rs`)

**What**: Constructs a 4D Anderson lattice where dimensions 1–3 are tissue
space and dimension 4 is an immune response gradient (e.g., cytokine
concentration over time). Runs Lanczos eigenvalue analysis to determine
level spacing ratio (Poisson → localized, GOE → extended).

**barracuda API**:
```rust
barracuda::spectral::anderson::anderson_4d(l, disorder, seed) -> SpectralCsrMatrix
```

**Cross-spring lineage**: hotSpring S26 spectral theory (Anderson localization
in condensed matter, Kachkovskiy 2016) → 2D/3D variants in S59 → 4D variant
in S84 → groundSpring Paper 12 tissue immunology.

### 3. 4D Wegner Block RG Coarsening (`tissue_anderson/mod.rs`)

**What**: Applies Wegner's real-space renormalization group to the 4D Anderson
Hamiltonian, coarsening by factor 2 in each dimension. Returns both fine and
coarse lattice results for comparison. Critical for tissue models where the
relevant length scale is cell clusters, not individual cells.

**barracuda API**:
```rust
barracuda::spectral::anderson::wegner_block_4d(&csr, l) -> SpectralCsrMatrix
```

**Cross-spring lineage**: Wegner (1976) Z. Phys. B 25, 327 → hotSpring
condensed matter (3D RG) → ToadStool S84 (4D extension) → groundSpring
tissue disorder flow analysis.

---

## metalForge Expansion

| Workload | Capabilities | Tolerance Tier | Justification |
|---|---|---|---|
| L-BFGS grid refine (CPU) | F64Compute | Analytical | Numerical gradient + line search FP error |
| Tissue Anderson 4D + Wegner RG | F64Compute + ShaderDispatch | Exact | Deterministic lattice + RG with fixed seed |

---

## Cross-Spring Evolution Highlights (V68)

### hotSpring Precision Shaders → Tissue 4D

The spectral module evolved: `anderson_1d` (S26) → `anderson_2d`/`3d` (S26) →
`anderson_3d_correlated` (S59) → `anderson_4d` + `wegner_block_4d` (S84).

groundSpring asks "does a cytokine signal propagate through 4D tissue?" using
the same mathematics hotSpring uses to ask "does an electron propagate through
a disordered lattice?" — shared barracuda implementation, different domains.

### airSpring Optimizer → Freeze-Out Refinement

airSpring's FAO-56 parameter fitting evolved L-BFGS into barracuda (S84).
groundSpring absorbs it for QCD freeze-out curve refinement — agricultural
sensor calibration enables nuclear physics parameter estimation.

### wetSpring Bio ↔ neuralSpring (Bidirectional)

wetSpring biodiversity primitives (Shannon, Bray-Curtis) hardened by
neuralSpring's metalForge → serve groundSpring's rare biosphere experiments.
neuralSpring's ML shaders (matmul, ESN) flow back into wetSpring for
annotation. This bidirectional cycle is unique — unlike the one-way hotSpring
precision flow.

---

## Action Items for ToadStool

1. **`SeasonalGpuParams` constructor** (from V67): The struct has private
   padding fields. groundSpring works around this with `bytemuck::Zeroable::zeroed()`.
   A `SeasonalGpuParams::new(...)` constructor would be cleaner.

2. **`anderson_4d`/`wegner_block_4d` re-export**: These are in
   `barracuda::spectral::anderson` but not re-exported from `barracuda::spectral`.
   Adding them to the `pub use anderson::{...}` list would simplify imports.

3. **L-BFGS GPU variant**: The current `lbfgs_numerical` is CPU-only. For
   large-scale optimization (many parameters), a GPU L-BFGS with batched
   numerical gradient evaluation would benefit all Springs.

---

## Quality Gates

| Gate | Status |
|---|---|
| `cargo fmt --check` | PASS |
| `cargo clippy --all-targets --all-features -- -W clippy::pedantic -W clippy::nursery` | PASS (zero warnings) |
| `cargo test --workspace` | 776 passed, 0 failed |
| `cargo doc --no-deps` | PASS |
| Zero unsafe | PASS |
| Zero TODO | PASS |
| Zero .unwrap() | PASS |
| Zero #[allow] without reason | PASS |
