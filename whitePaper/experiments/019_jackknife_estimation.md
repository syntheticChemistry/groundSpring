# Exp 019: Jackknife Error Estimation

**Domain**: Statistics (resampling, variance estimation)
**Paper**: Bazavov et al. (2025) Phys Rev D 111, 094508
**Faculty**: Alexei Bazavov (CMSE + Physics, MSU)
**Question**: Does delete-one jackknife resampling achieve subpercent accuracy
for variance estimation and bias correction, and how does it compare with
bootstrap?

## Data Source

Synthetic test cases: Gaussian (μ=5, σ=2), exponential (rate=0.5),
correlated data for block jackknife. Open system — reproducible from
distribution parameters + PRNG seed. Extends Exp 007 RAWR methodology.

## Method

1. **Delete-one jackknife**: Leave-one-out pseudo-values for mean and
   variance of mean. Variance estimate σ²_JK = (n−1)/n Σ(θ̂_{-i} − θ̂_·)².
2. **Bias correction**: Jackknife bias for biased variance estimator
   (divide by n instead of n−1). Corrected = biased + (n−1)(θ̂_full − θ̂_·).
3. **Block jackknife**: For correlated data, delete blocks of size b.
   Variance increases with block size as autocorrelation is preserved.
4. **Jackknife vs bootstrap**: Compare variance of mean for IID data;
   ratio should be near 1.0.

## Key Result

**Jackknife provides unbiased variance estimation with bias correction.**
- Gaussian and exponential: jackknife mean and variance match analytical
  expectations within tolerance
- Bias correction reduces error on the biased variance estimator
- Block jackknife: variance increases monotonically with block size
- Jackknife/bootstrap variance ratio ≈ 1.0 for IID data
- Deterministic given fixed input

**Bazavov et al. (2025) use** jackknife for subpercent error estimation
in hadronic vacuum polarization. groundSpring validates the core
delete-one and block-jackknife machinery.

## Validation

| Phase | Checks | Binary |
|-------|--------|--------|
| Phase 0 (Python) | 9/9 | `control/jackknife_estimation/jackknife_estimation.py` |
| Phase 1 (Rust) | 9/9 | `validate-jackknife` |

## Barracuda Path

Jackknife is embarrassingly parallel across leave-one-out replicates.
Block jackknife parallelizes across block sizes. Bootstrap comparison
reuses existing `bootstrap_mean` delegation. CPU-only for now.

## Modules

`jackknife` (`jackknife_mean_variance`, `jackknife_bias`,
`block_jackknife_variance`, `leave_one_out_biased_variance`),
`bootstrap` (for comparison), `prng`
