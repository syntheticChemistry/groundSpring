# Exp 010: Bistable Phenotypic Switching

**Domain**: Biological (c-di-GMP, phenotypic plasticity)
**Paper**: Fernandez et al. (2020, PNAS) — V. cholerae cell shape regulation
**Faculty**: Christopher Waters (MMG, MSU)
**Question**: When does stochastic noise push a bistable biological system
across a phenotypic threshold?

## Data Source

ODE model of c-di-GMP regulation with bistable dynamics. Parameters from
Fernandez et al. (2020) supplementary methods: DGC (diguanylate cyclase),
PDE (phosphodiesterase), c-di-GMP, and cell density state variables.
Hill-function cooperative binding creates two stable attractors.

## Method

1. **Deterministic ODE** (RK4): Integrate from two initial conditions (low
   and high c-di-GMP) to verify bistability — both settle to different
   attractors under identical parameters.
2. **Stochastic ODE** (Euler-Maruyama): Add Gaussian noise (σ=0.05) and
   verify noise-induced transitions between states.
3. **Monostable control**: Remove positive feedback (Hill coefficient → 0)
   and confirm convergence to a single attractor.

## Key Result

**Bistability confirmed.** Low initial c-di-GMP → low attractor (~0.035);
high initial c-di-GMP → high attractor (~1.634). Attractor separation ratio
> 10×. Stochastic simulations show noise-induced transitions crossing the
midpoint threshold. Removing cooperative feedback collapses to monostable
behavior.

## Performance

| Metric | Python | Rust | Speedup |
|--------|--------|------|---------|
| Time | 3.58s | 0.19s | **18.5×** |

## Validation

| Phase | Checks | Binary |
|-------|--------|--------|
| Phase 0 (Python) | 10/10 | `control/bistable_switching/bistable_switching.py` |
| Phase 1 (Rust) | 9/9 | `validate-bistable` |

Checks: Cell density carrying capacity, low/high attractor values, attractor
separation, monostable control, determinism, stochastic threshold crossing.

## Barracuda Path

`BistableOde::cpu_derivative` **delegated** to barracuda via the `OdeSystem`
trait. `BistableParams::to_flat()` converts the named parameter struct to
barracuda's flat `&[f64]` convention. RK4 integration and Euler-Maruyama
stochastic integration use the shared `rk4_step` pattern with `mul_add`
for numerical efficiency.

## Modules

`bistable` (new: `BistableParams`, `hill`, `bistable_derivative`, `rk4_step`,
`integrate`, `stochastic_integrate`), `prng`
