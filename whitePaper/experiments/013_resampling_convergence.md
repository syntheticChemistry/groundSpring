# Exp 013: Resampling Convergence

**Domain**: Statistics (bootstrap methodology)
**Paper**: Lee & Liu (2024) IEEE BIBM — statistical resampling optimization
**Question**: How many resampling replicates are enough? When does adding more replicates stop improving the confidence interval?

## Data Source

Synthetic: Gaussian (μ=5, σ=2), log-normal, and heavy-tailed (t, df=3) distributions.
Open system — reproducible from distribution parameters + PRNG seed.

## Method

Run bootstrap and RAWR at geometrically increasing replicate counts
(100, 200, 500, 1000, 2000, 5000, 10000). Track CI width convergence
and relative change between successive counts. Measure coverage at
n=1000 replicates.

## Key Result

**Both methods converge by ~2000 replicates for Gaussian data.**
- 5k→10k relative width change: <1% for both bootstrap and RAWR
- Bootstrap coverage at n=1000: ~95% (Gaussian)
- Heavy-tailed data produces wider CIs (as expected)
- RAWR provides comparable convergence to standard bootstrap

For most groundSpring experiments, n=2000 replicates is sufficient.
This saves ~5× compute vs n=10000 with negligible accuracy loss.

## Performance

| Metric | Python | Rust | Speedup |
|--------|--------|------|---------|
| Time | — | 0.5s | — |

## Validation

| Phase | Checks | Binary |
|-------|--------|--------|
| Phase 0 (Python) | 10/10 | `control/resampling_convergence/resampling_convergence.py` |
| Phase 1 (Rust) | 8/8 | `validate-resampling-conv` |

## V18 Changes

- Python baseline confirmed: 10/10 PASS (lognormal CI width tolerance justified at 1.5×)
- DOI added to benchmark JSON; baseline_commit stamped

## Barracuda Path

`bootstrap_mean` already delegated to `barracuda::stats::bootstrap_mean` (CPU).
`rawr_mean` delegated to `barracuda::stats::rawr_mean` (S66). Embarrassingly parallel
for future GPU batching.

## Modules

`bootstrap`, `prng`
