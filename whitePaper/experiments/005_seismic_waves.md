# Exp 005: Seismic Wave Propagation

**Domain**: Geological (seismology)
**Source**: Synthetic New Madrid Seismic Zone earthquake
**Question**: How does arrival time noise affect source localization?

## Data Source

Synthetic NMSZ earthquake (M~3, 10km depth) with 7 regional stations.
IASP91 1D velocity model.
Open system — reproducible from source parameters + station coords.

## Method

1D travel-time forward model + grid-search inversion + Nelder-Mead refinement.
Monte Carlo uncertainty quantification (50 noise realizations, ±0.5s).

## Key Result

**Horizontal location is well-constrained; depth is not.** With surface
stations only, the depth-origin time tradeoff makes depth poorly determined
(±8.5km depth vs ±2.1km horizontal). More stations help with diminishing
returns above 5.

## Validation

| Phase | Checks | Binary |
|-------|--------|--------|
| Phase 0 (Python) | PASS | `control/seismic/seismic_inversion.py` |
| Phase 1 (Rust) | 9/9 | `validate-seismic` |

## Barracuda Path

Tier B — grid-search dispatch as 3D workgroup. No existing barracuda op.

## Modules

`seismic`, `stats`
