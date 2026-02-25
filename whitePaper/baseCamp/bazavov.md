# Bazavov — Inverse Problems & Spectral Reconstruction

**Faculty**: Alexei Bazavov (CMSE + Physics, Michigan State University)
**Domain**: Lattice QCD, statistical mechanics, high-precision inverse problems
**groundSpring Connection**: Exp 005 (seismic inversion) generalized to subpercent precision

---

## Why This Matters for groundSpring

groundSpring Exp 005 solves a simple inverse problem: given noisy P-wave arrival
times at 7 stations, locate the earthquake source. Bazavov's lattice QCD work
contains a rich set of related inverse problems that demand 10-100× tighter
precision — spectral reconstruction from noisy correlator data, freeze-out
conditions from experimental observables, and vacuum polarization from lattice
measurements.

The mathematical structure is identical: noisy observations → forward model →
inverse problem → uncertainty quantification. The difference is scale: seismic
inversion tolerates ±2 km; lattice QCD requires subpercent agreement.

## Papers for Reproduction

### Tier 1 (Priority)

**Paper #6**: Bazavov et al. (2025) "Spectral reconstruction inverse problem
in lattice QCD." arXiv 2501.12259.

- **Open Data**: Lattice ensembles from MILC Collaboration (public via ILDG/USQCD)
- **Open Code**: Analysis scripts expected in arXiv supplementary
- **groundSpring Modules**: `stats` (RMSE, R²), `decompose` (bias-variance on
  reconstruction residuals), new `spectral` module (regularized inverse)
- **BarraCUDA Needs**: FFT (gap), matrix inverse (exists), regularization (partial)
- **Control Plan**: Python baseline → Rust CPU → barracuda GPU → metalForge

### Tier 2

**Paper #7**: Bazavov et al. (2025) "Hadronic vacuum polarization for the muon
g-2." Phys Rev D 111, 094508. DOI: 10.1103/PhysRevD.111.094508

- **Open Data**: HotQCD / MILC lattice configurations
- **Method**: Jackknife error estimation at subpercent precision
- **groundSpring Modules**: `stats` (bootstrap/jackknife extension), `decompose`

**Paper #8**: Bazavov et al. (2016) "Curvature of the freeze-out line in heavy
ion collisions." Phys Rev D 93, 014512.

- **Open Data**: STAR/PHENIX beam energy scan data (BNL)
- **Method**: Inferring freeze-out conditions from observables (inverse problem)
- **groundSpring Modules**: `seismic`-style grid search generalized to QCD phase diagram

## BarraCUDA Kernel Requirements

| Primitive | Status | Notes |
|-----------|--------|-------|
| FFT (real, complex) | **Gap** — not in barracuda | Required for spectral reconstruction |
| Matrix inverse | Exists (`linalg::solve`) | CPU; GPU via Cholesky |
| Regularization (Tikhonov) | Partial | Ridge regression exists; extend to L-curve |
| Bootstrap/jackknife | `stats::bootstrap_*` | CPU exists; GPU embarrassingly parallel |
| Sparse eigensolve | `BatchedEighGpu` | For Lanczos on large lattices |

## Three-Tier Control Plan

| Tier | Validation | Status |
|------|-----------|--------|
| CPU | Python baseline matches Rust | Queued |
| GPU | barracuda GPU matches CPU | Blocked by FFT gap |
| metalForge | Cross-substrate agreement | After GPU tier |

## Cross-Spring

- **hotSpring**: Bazavov is already in hotSpring's paper queue (Phase C/D nuclear EOS).
  groundSpring adds the noise/uncertainty perspective.
- **Shared kernel need**: FFT is needed by both hotSpring (PPPM) and groundSpring
  (spectral reconstruction). Joint priority for ToadStool.
