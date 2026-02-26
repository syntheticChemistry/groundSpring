# Exp 021: Spectral Function Reconstruction

**Domain**: Inverse problems (lattice QCD, spectral analysis)
**Paper**: Bazavov et al. (2025) arXiv 2501.12259
**Faculty**: Alexei Bazavov (CMSE + Physics, MSU)
**Question**: Can Tikhonov-regularized inversion recover a spectral peak
from a noisy Euclidean correlator?

## Data Source

Spectral function: Gaussian peak at ω_center=3.0, width=0.5, amplitude=1.0.
Euclidean correlator G(τ) = ∫ K(τ,ω) ρ(ω) dω with Laplace kernel
K(τ,ω) = exp(−τω). Grid: n_τ=30, n_ω=60, τ_max=2.0, ω_max=8.0. Additive
Gaussian noise on correlator. Most advanced inverse problem in groundSpring.

## Method

1. **Kernel and forward model**: Build K(τ,ω), compute G = K ρ. Noiseless
   roundtrip (solve then forward) should have negligible RMSE.
2. **Cholesky solver**: Tikhonov (K^T K + λI) ρ = K^T G. Residual check.
3. **Noisy reconstruction**: Add noise to G, solve at optimal λ. Peak
   location error and positivity of recovered ρ.
4. **Regularization trade-off**: Scan λ ∈ [1e-8, 1e-6, 1e-4, 1e-2, 1.0].
   Small λ amplifies noise; large λ over-smooths. Optimal λ minimizes
   reconstruction RMSE.
5. **Determinism**: Fixed seed yields identical reconstruction.

## Key Result

**Tikhonov regularization recovers spectral peak from noisy correlator.**
- Noiseless forward RMSE < 1e-6
- Cholesky residual < 1e-6
- Noisy case: peak location error < 1.0 (ω units)
- Peak value positive (physical)
- Regularization: small λ amplifies noise (high RMSE); large λ
  over-smooths (high RMSE); optimal λ ≈ 1e-4 minimizes RMSE
- Deterministic reconstruction

**Bazavov et al. (2025) apply** this methodology to lattice QCD
spectral functions. groundSpring validates the core Tikhonov
machinery: ill-posed integral equation → regularized least squares.

## Validation

| Phase | Checks | Binary |
|-------|--------|--------|
| Phase 0 (Python) | 8/8 | `control/spectral_recon/spectral_recon.py` |
| Phase 1 (Rust) | 8/8 | `validate-spectral-recon` |

## Barracuda Path

Kernel build and forward correlator are matrix-vector products.
Tikhonov solve requires Cholesky (exists in barracuda). Regularization
scan parallelizes across λ values. FFT not required for this
discretized formulation.

## Modules

`spectral_recon` (`build_kernel`, `forward_correlator`, `gaussian_peak`,
`tikhonov_solve`, `peak_index`, `rmse`)
