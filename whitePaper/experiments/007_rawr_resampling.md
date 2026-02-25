# Exp 007: RAWR Resampling

**Domain**: Statistics (bootstrap methods)
**Paper**: Wang et al. (2021, ISMB/Bioinformatics) — RAWR weighted resampling
**Question**: Does RAWR improve bootstrap confidence intervals for structured data?

## Data Source

Synthetic test cases: Gaussian (μ=5, σ=2), log-normal (skewed),
AR(1) correlated (ρ=0.5).
Open system — reproducible from distribution parameters + PRNG seed.

## Method

Standard percentile bootstrap vs RAWR (Bayesian bootstrap with Dirichlet weights).
Coverage analysis: does the 95% CI contain the true mean over 200 trials?
RMSE comparison: point estimate accuracy.

## Key Result

**RAWR provides different variance structure, not necessarily tighter coverage.**
For Gaussian data, both methods achieve >92% coverage. For skewed data,
standard bootstrap slightly outperforms (93.5% vs 82%). RAWR's value lies
in its analytical weight structure for structured data (time series, spatial).

## Performance

| Metric | Python | Rust | Speedup |
|--------|--------|------|---------|
| Time | 4.4s | 0.60s | **7.3×** |

Lower speedup reflects NumPy's vectorized dot products vs Rust's scalar loop.

## Validation

| Phase | Checks | Binary |
|-------|--------|--------|
| Phase 0 (Python) | 11/11 | `control/rawr_resampling/rawr_resampling.py` |
| Phase 1 (Rust) | 11/11 | `validate-rawr` |

## Barracuda Path

`bootstrap_mean` **delegated** to `barracuda::stats::bootstrap_mean` (CPU).
`rawr_mean` — **gap**: no RAWR kernel in barracuda. New `ops::rawr_weighted_mean_f64`
needed. Embarrassingly parallel — suitable for GPU.

## Modules

`bootstrap`, `prng`, `stats`
