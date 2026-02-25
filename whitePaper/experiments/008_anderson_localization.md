# Exp 008: Anderson Localization

**Domain**: Mathematics (spectral theory)
**Paper**: Bourgain-Kachkovskiy (2018, GAFA) — Anderson localization in 1D
**Question**: How does disorder strength affect wave localization?

## Data Source

1D Anderson tight-binding model with uniform disorder on [-W/2, W/2].
Anderson (1958) original prediction, Bourgain-Kachkovskiy (2018) formalization.
Open system — reproducible from N, W, energy, seed.

## Method

Transfer-matrix method with vector renormalization (avoids overflow).
Lyapunov exponent γ = (1/N) Σ ln(norm) — rate of exponential localization.
Thouless scaling: γ ≈ W²/C at weak disorder (C ≈ 105 at band center).

## Key Result

**All states are localized in 1D.** Even W=0.5 produces γ > 0 — there are
no extended states. The localization length ξ = 1/γ decreases monotonically
with disorder. This is the mathematical framework for wave propagation noise:
disordered media exponentially attenuate coherent signals.

## Performance

| Metric | Python | Rust | Speedup |
|--------|--------|------|---------|
| Time | 21.4s | 0.72s | **29.8×** |

## Validation

| Phase | Checks | Binary |
|-------|--------|--------|
| Phase 0 (Python) | 8/8 | `control/anderson_localization/anderson_localization.py` |
| Phase 1 (Rust) | 8/8 | `validate-anderson` |

## Barracuda Path

`lyapunov_exponent` and `lyapunov_averaged` **delegated** to
`barracuda::spectral::lyapunov_*` (requires `barracuda-gpu` feature;
`anderson` submodule is private, items re-exported at `spectral` level).
`analytical_localization_length` **delegated** to
`barracuda::special::anderson_transport::localization_length` (CPU, perturbative ξ(W,E)).
`BatchIprGpu` available for GPU spectral statistics.
S59 additions: `anderson_3d_correlated`, `anderson_sweep_averaged`, `find_w_c`
available for future Kachkovskiy extension experiments.

## Modules

`anderson`, `prng`
