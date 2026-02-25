# Exp 006: Enzymatic Signal Specificity

**Domain**: Biological (stochastic biochemistry)
**Paper**: Massie et al. (2012, PNAS) — c-di-GMP signaling specificity
**Question**: How does enzymatic noise limit intracellular signal detection?

## Data Source

Analytical birth-death kinetics (α/β = steady state).
Massie et al. PNAS 2012 — open access publication.
Open system — Gillespie SSA with known rates + PRNG seed.

## Method

Gillespie stochastic simulation algorithm (birth-death process).
Analytical comparison: mean = α/β, variance ≈ mean (Poisson).
Signal-to-noise ratio sweep across production rates α.

## Key Result

**Signal specificity is noise-limited at low expression.** At α=2, the
cell barely distinguishes signal from fluctuations (SNR < 1). At α=20,
SNR crosses 2.0 — the signal is reliably detectable. This is the biological
analog of sensor noise (Exp 001).

## Performance

| Metric | Python | Rust | Speedup |
|--------|--------|------|---------|
| Time | 26.2s | 0.85s | **30.9×** |

## Validation

| Phase | Checks | Binary |
|-------|--------|--------|
| Phase 0 (Python) | 12/12 | `control/signal_specificity/signal_specificity.py` |
| Phase 1 (Rust) | 12/12 | `validate-signal-specificity` |

## Barracuda Path

`barracuda::ops::bio::GillespieGpu` exists — GPU-only (no CPU fallback).
Dispatches parallel trajectory ensembles. Requires `barracuda-gpu` feature.

## Modules

`gillespie`, `prng`
