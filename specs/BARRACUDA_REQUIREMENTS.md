# groundSpring — BarraCUDA Requirements

**Last Updated**: February 12, 2026
**Purpose**: GPU kernel requirements, gap analysis, and evolution priorities

---

## Current Status

groundSpring Phase 0 is Python-only (NumPy, SciPy, ObsPy). No Rust or GPU code yet. The experiments establish noise characterization baselines that inform all other springs.

---

## Kernel Requirements for Rust/GPU Evolution

### Phase 1 — Statistical Primitives (Core Methodology)

| Operation | Exp | Current (Python) | GPU Target | Priority |
|-----------|-----|-----------------|------------|----------|
| Monte Carlo sampling | Exp 003 | NumPy random, N=10,000 | Parallel MC on GPU — 100k+ samples in one dispatch | **P0** |
| Bias-variance decomposition | Exp 001 | NumPy mean/std | FusedMapReduceF64 — already in ToadStool | **P0** |
| Sensitivity analysis (Sobol) | Exp 003 | Manual parameter sweep | Parallel Sobol indices via GPU sampling | **P1** |
| Rarefaction curves | Exp 004 | NumPy multinomial | Parallel bootstrap resampling — embarrassingly parallel | **P1** |
| Grid search (least squares) | Exp 005 | SciPy minimize | GPU parameter sweep — all grid points in parallel | **P1** |

### Phase 2 — Inverse Problem Solvers

| Operation | Exp | Current (Python) | GPU Target | Priority |
|-----------|-----|-----------------|------------|----------|
| Nonlinear least squares | Exp 005 | SciPy minimize (L-BFGS-B) | GPU-batched Levenberg-Marquardt | **P1** |
| Travel time computation | Exp 005 | IASP91 1D model | Parallel ray tracing for all source-station pairs | **P1** |
| Haversine distance | Exp 005 | NumPy trig | Fused GPU kernel (trivial) | **P2** |

### Phase 3 — Faculty Extension Requirements

| Operation | Paper | GPU Target | Priority | Effort |
|-----------|-------|------------|----------|--------|
| **FFT (spectral reconstruction)** | Bazavov 2025 | Complex FFT for lattice correlator analysis | **P1** | High — shared need with hotSpring |
| **Jackknife/bootstrap** | Bazavov 2025, Liu 2021 | Parallel resampling with structured covariance | **P1** | Low — embarrassingly parallel |
| **Stochastic simulation (Gillespie)** | Waters/Massie 2012 | Parallel trajectory ensemble on GPU | **P1** | Medium — PRNG + exponential |
| **Lanczos iterative eigensolve** | Kachkovskiy 2016, 2018 | Large sparse Hamiltonian diagonalization for spectral analysis. Foundation for Anderson localization studies | **P1** | Medium — tridiagonalization + QR; shared with hotSpring |
| **Sparse matrix-vector product (SpMV)** | Kachkovskiy (all papers) | Inner-loop of Lanczos. CSR-format SpMV on GPU — required for any spectral method at scale | **P1** | Medium — CSR SpMV shader |
| **Bifurcation analysis** | Waters/Fernandez 2020 | Parameter continuation + eigenvalue tracking | **P2** | Medium — `BatchedEighGpu` + sweep |
| **RAWR resampling** | Liu/Wang 2021 | Weighted resampling with random walks | **P2** | Medium — PRNG + weighted sampling |
| **Regularized inversion** | Bazavov 2025 | Tikhonov/Maximum entropy for spectral reconstruction | **P2** | High — matrix solve + regularization |
| **Matrix exponentiation** | Kachkovskiy 2016 | Time evolution exp(iHt) for transport analysis. General matrix exp — Cayley exists for SU(3); need general case | **P2** | Medium |

---

## Existing ToadStool Kernels That Apply

| ToadStool Kernel | groundSpring Use |
|-----------------|-----------------|
| `FusedMapReduceF64` | Bias-variance decomposition, MC statistics |
| `VarianceF64` | Uncertainty estimation for all experiments |
| `BatchedEighGpu` | Eigenvalue computation for bifurcation analysis |
| `GemmF64` | Matrix operations in least squares, covariance |
| `CorrelationF64` | Cross-sensor correlation analysis |

---

## BarraCUDA Evolution Path for groundSpring

```
Phase 0 (DONE — Python)       Phase 1 (Rust — NEXT)
────────────────────          ─────────────────────
NumPy MC (N=10k)   ────────→  GPU MC (N=100k+) via FusedMapReduce
SciPy minimize     ────────→  GPU Levenberg-Marquardt
NumPy stats        ────────→  FusedMapReduce + Variance
ObsPy travel times ────────→  Sovereign Rust 1D ray tracer
Manual Sobol       ────────→  GPU Sobol indices

Phase 1 (Rust)                Phase 2 (Faculty Extension)
──────────────                ──────────────────────────
GPU MC sampling    ────────→  Jackknife/bootstrap (Bazavov precision)
GPU least squares  ────────→  Regularized inversion (spectral recon)
N/A                ────────→  FFT (shared with hotSpring)
N/A                ────────→  Gillespie simulation (biological noise)
N/A                ────────→  RAWR resampling (Liu method)
N/A                ────────→  Lanczos eigensolve (Kachkovskiy spectral)
N/A                ────────→  SpMV (shared with hotSpring lattice)
```

---

## Cross-Spring Kernel Sharing

| Kernel | groundSpring | Also Used By |
|--------|-------------|-------------|
| FFT | Spectral reconstruction | hotSpring (lattice QCD), wetSpring (signal processing) |
| Monte Carlo | Error propagation | hotSpring (nuclear EOS), neuralSpring (training) |
| Gillespie | Biological noise modeling | wetSpring (c-di-GMP dynamics) |
| Bootstrap | Confidence estimation | wetSpring (rarefaction), neuralSpring (model uncertainty) |
| Eigensolve (dense) | Bifurcation analysis | hotSpring (HFB), wetSpring (PCoA) |
| Lanczos (sparse) | Anderson localization, spectral analysis | hotSpring (Dirac spectrum), neuralSpring (Hessian eigenvalues) |
| SpMV | Lanczos inner loop | hotSpring (lattice gauge), neuralSpring (sparse attention) |

groundSpring's statistical and spectral primitives are infrastructure for every spring.
