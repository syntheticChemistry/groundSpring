# Exp 012: Spin Chain Transport

**Domain**: Mathematics (spectral theory / quantum dynamics)
**Paper**: Kachkovskiy (2016) Comm Math Phys 345:659-673
**Question**: When energy is injected at one site in a noisy medium, does it propagate (ballistic) or stay trapped (localized)?

## Data Source

Analytical: 1D Almost-Mathieu Hamiltonian with golden-ratio frequency.
Transport exponents from spectral theory (Aubry-André transition at λ=2).
Open system — fully specified by Hamiltonian parameters.

## Method

Build tridiagonal Hamiltonian → eigendecompose → time-evolve wavepacket via
ψ(t) = Σ_k U_{j,k} U_{n₀,k} exp(-i E_k t). Track mean square displacement
σ²(t) = Σ_j (j-n₀)² |ψ_j(t)|². Extract transport exponent β from log-log fit.

## Key Result

**Transport exponent confirms Aubry-André transition.**
- Extended phase (λ<2): β ≈ 0.93 (ballistic) — signal propagates
- Critical point (λ=2): β ≈ 0.45 (anomalous) — partial propagation
- Localized phase (λ>2): β ≈ 0 — noise wins, signal trapped

Lyapunov exponent cross-check confirms localization. Normalization preserved
to machine precision throughout time evolution.

## Performance

| Metric | Python | Rust | Speedup |
|--------|--------|------|---------|
| Time | TBD | 2.3s | TBD |

## Validation

| Phase | Checks | Binary |
|-------|--------|--------|
| Phase 0 (Python) | TBD | `control/spin_transport/spin_chain_transport.py` |
| Phase 1 (Rust) | 18/18 | `validate-transport` |

## Barracuda Path

`tridiag_eigh` — **gap**: no eigenvector solver in barracuda (eigenvalues only via Sturm).
Future: `barracuda::spectral::tridiag_eigh` for eigenvalues + eigenvectors.
`wavepacket_msd` — pure math on eigenpairs, suitable for GPU parallelization.

## Modules

`transport`, `almost_mathieu`, `anderson`
