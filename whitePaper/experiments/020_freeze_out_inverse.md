# Exp 020: Freeze-Out Inverse Problem

**Domain**: Inverse problems (lattice QCD, heavy ion physics)
**Paper**: Bazavov et al. (2016) Phys Rev D 93, 014512
**Faculty**: Alexei Bazavov (CMSE + Physics, MSU)
**Question**: Can chi-squared grid-search fitting recover freeze-out curve
parameters from noisy polynomial observables?

## Data Source

Freeze-out curve model: T_f(μ_B) = T0(1 − κ₂(μ_B/T0)²). Synthetic
observables at μ_B = 0, 50, 100, …, 400 MeV with additive Gaussian
noise. True parameters T0=155 MeV, κ₂=0.013. Extends Exp 005 seismic
inversion (grid-search inverse problem).

## Method

1. **Forward model**: Freeze-out curve T_f(μ_B) = T0(1 − κ₂(μ_B/T0)²).
   Validated at μ_B=0 (T_f=T0) and μ_B=400.
2. **Chi-squared**: χ² = Σ(obs_i − model_i)²/σ². χ²/dof at truth
   should be reasonable (not overfitting).
3. **2D grid search**: Scan (T0, κ₂) over parameter space. Find
   minimum χ². Recover true parameters within tolerance.
4. **Replicate coverage**: 50 noise realizations; fraction with
   recovery within tolerance.
5. **Noise sensitivity**: Lower noise improves recovery precision.

## Key Result

**Grid-search chi-squared recovers freeze-out parameters from noisy data.**
- Forward model T_f(0)=T0, T_f(400) matches analytical formula
- Chi-squared at truth: χ²/dof reasonable
- Single realization: T0 and κ₂ recovered within tolerance
- Replicate coverage: ≥70% of realizations recover parameters
- Lower noise yields better recovery (noise degrades precision)
- Deterministic given fixed observations

**Bazavov et al. (2016) inferred** freeze-out curvature from beam
energy scan data. This experiment validates the inverse problem
structure: noisy observables → forward model → grid-search fit.

## Validation

| Phase | Checks | Binary |
|-------|--------|--------|
| Phase 0 (Python) | 8/8 | `control/freeze_out_inverse/freeze_out_inverse.py` |
| Phase 1 (Rust) | 8/8 | `validate-freeze-out` |

## Barracuda Path

Grid search is embarrassingly parallel across (T0, κ₂) points — ideal
for GPU dispatch. Chi-squared computation is a reduction. Same
structure as Exp 005 seismic grid-search (Tier B adapt).

## Modules

`freeze_out` (`freeze_out_curve`, `chi_squared`, `chi_squared_per_dof`,
`grid_fit_2d`, `GridFitConfig`), `prng`
