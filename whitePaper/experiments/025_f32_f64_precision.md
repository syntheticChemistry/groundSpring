# Exp 025: f32 vs f64 Precision Drift

**Domain**: WDM Molecular Dynamics
**Paper**: IEEE 754-2019; Higham (2002) Accuracy and Stability of Numerical Algorithms
**Faculty**: baseCamp Sub-thesis 07
**Question**: Does f32 accumulation introduce systematic bias in Green-Kubo
transport coefficient calculations?

## Data Source

Synthetic velocity autocorrelation functions (VACF) with exponential decay
plus Gaussian noise. Parameters chosen to match WDM conditions: decay rates
spanning fast (electronic) to slow (ionic) timescales, noise levels matching
typical molecular dynamics statistical variance.

## Method

1. **Synthetic VACF**: Generate exponential-decay autocorrelation C(t) = D₀·exp(−t/τ)
   with additive Gaussian noise N(0, σ²)
2. **Dual-precision integration**: Trapezoidal integration in both f64 (reference)
   and f32 (consumer GPU baseline)
3. **Error decomposition**: Bias = mean(D_f32 − D_f64), noise = std(D_f32 − D_f64)
4. **Scaling analysis**: How does error grow with integral magnitude (longer tails)?
5. **Bias fraction**: systematic / (systematic + random) error partition

## Key Results

- f64 matches analytical (noiseless) within 0.1%
- f32 introduces measurable systematic bias (~28% of total error)
- Absolute errors scale with integral magnitude — longer autocorrelation
  tails accumulate more rounding error
- Bias fraction above 1% confirms a detectable systematic component
- Error-magnitude correlation: larger integrals → larger errors

This is the floating-point analog of Exp 001's sensor bias: a correctable
systematic component that dominates when accumulating many small values.

## Performance

| Metric | Python | Rust | Speedup |
|--------|--------|------|---------|
| Time | 0.42s | 0.07s | **6.0×** |

## Validation

| Phase | Checks | Binary |
|-------|--------|--------|
| Phase 0 (Python) | 7/7 | `control/precision_drift/precision_drift.py` |
| Phase 1 (Rust) | 7/7 | `validate-precision-drift` |

Checks: f64-analytical match, f32 max relative error, mean relative error,
bias fraction, max absolute error, error-magnitude correlation, relative
error standard deviation.

## Barracuda Path

Uses `wdm` module. `finite_size_extrapolate` delegated to
`barracuda::stats::regression::fit_linear`. Green-Kubo integration is the
GPU promotion target — trapezoidal accumulation is embarrassingly parallel
when split across time windows.

## Modules

`wdm::precision_drift` (`generate_vacf`, `integrate_trapezoidal_f64`,
`integrate_trapezoidal_f32`, `bias_variance_decompose`)
