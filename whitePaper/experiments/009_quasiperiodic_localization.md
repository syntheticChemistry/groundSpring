# Exp 009: Almost-Mathieu Quasiperiodic Localization

**Domain**: Mathematics (quasiperiodic operators, spectral theory)
**Paper**: Jitomirskaya & Kachkovskiy (2018, JEMS) — All couplings localization
**Faculty**: Ilya Kachkovskiy (Mathematics, MSU)
**Question**: Does the Aubry-André metal-insulator transition at λ=2 hold, and
how do level statistics distinguish extended from localized phases?

## Data Source

Almost-Mathieu operator with coupling λ, golden-ratio frequency α = (√5−1)/2,
phase θ. The Hamiltonian is a tridiagonal matrix with quasiperiodic diagonal
entries V_n = λ cos(2παn + θ). Fully analytical — reproducible from λ, N, α, θ.

## Method

1. **Lyapunov exponent**: Transfer-matrix method (shared with Exp 008 Anderson
   module). Herman's formula: γ = ln(λ/2) for λ > 2.
2. **Eigenvalue statistics**: QR decomposition of the Hamiltonian matrix.
   Level spacing ratio ⟨r⟩ distinguishes extended (quasi-integrable, ⟨r⟩ → 1)
   from localized (Poisson, ⟨r⟩ → 0.39) phases.
3. **Coupling sweep**: γ(λ) computed across extended (λ < 2) and localized
   (λ > 2) regimes, verifying the critical point.

## Key Result

**Aubry-André transition at λ=2 confirmed.** For λ < 2 all states are
extended (γ = 0); for λ > 2 all states are localized with γ = ln(λ/2)
matching Herman's analytical formula. Level spacing ratios clearly distinguish
the two phases: extended regime ⟨r⟩ ≈ 0.91 (quasi-integrable), localized
regime ⟨r⟩ ≈ 0.39 (Poisson).

## Performance

| Metric | Python | Rust | Speedup |
|--------|--------|------|---------|
| Time | 0.65s | 12.16s | 0.1× * |

\* Rust validation uses a custom QR eigenvalue solver to prove mathematical
parity; Python delegates to numpy/LAPACK for dense eigenvalues. The math is
correct (parity proven), but cannot compete with LAPACK for this workload.
Barracuda GPU kernels will close this gap.

## Validation

| Phase | Checks | Binary |
|-------|--------|--------|
| Phase 0 (Python) | 8/8 | `control/quasiperiodic/quasiperiodic_localization.py` |
| Phase 1 (Rust) | 8/8 | `validate-quasiperiodic` |

Checks: Herman's formula accuracy (λ=3,4,5), clean system γ=0, critical
point at λ=2, γ monotonically increasing, extended ⟨r⟩ > localized ⟨r⟩.

## Barracuda Path

`almost_mathieu_hamiltonian` **delegated** to barracuda-gpu for Hamiltonian
construction (coupling convention: barracuda uses 2λ cos(…), groundSpring
adjusts the coupling parameter at the delegation boundary). Lyapunov
computation reuses the Exp 008 Anderson delegation (`spectral::lyapunov_*`).
Eigenvalue computation currently uses a custom QR solver; future barracuda
GPU eigensolve (Lanczos or divide-and-conquer) would eliminate the LAPACK gap.

## Modules

`anderson` (extended with `almost_mathieu_potential`, `almost_mathieu_hamiltonian`,
`level_spacing_ratio`), `prng`
