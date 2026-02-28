# groundSpring — BarraCUDA Requirements

**Last Updated**: February 28, 2026
**Purpose**: GPU kernel requirements, gap analysis, and evolution priorities

---

## Current Status

groundSpring Phase 0 (Python), Phase 1 (Rust), and Phase 2a (barracuda CPU) are **complete**.

- 292/292 validation checks across 28 binaries
- 28 library modules: stats, decompose, fao56, prng, rarefaction, seismic, gillespie, bootstrap, anderson, almost_mathieu, bistable, multisignal, kinetics, transport, drift, rare_biosphere, quasispecies, band_structure, jackknife, freeze_out, spectral_recon, wdm (+cast, validate)
- 444 Rust tests (biomeos) / 410 default + 320 Python tests = 764 total. 0 clippy warnings × 4 feature modes (default, barracuda, barracuda-gpu, biomeos). 39 active delegations + 7 pending ToadStool (30 CPU + 9 GPU). 49 metalForge tests. V42 GPU rewiring + cross-spring benchmark. biomeOS Neural API (V30)
- Two feature gates: `barracuda` (30 active CPU delegations) and `barracuda-gpu` (9 GPU delegations including Sturm tridiag, tikhonov solve, detect_bands, BatchedMultinomialGpu). Three-mode CI validates all configurations.
- 39 active delegations (30 CPU + 9 GPU; includes regression suite, mae, nash_sutcliffe, detect_band_ranges, BatchedMultinomialGpu occupancy + tier rate)
- 2 production WGSL shaders in `metalForge/shaders/` (261 combined lines)
- All matrices use flat row-major `Vec<f64>` — GPU-promotable layout
- Rust is **11.5× faster** than Python (excl. LAPACK-bound); 5.1× overall. Exp 009: **47.7× from Sturm tridiag**
- **28/28 mathematical parity proven** (Python ⇌ Rust; `data/parity_report.json`), all provenance fields stamped, 13 bitwise determinism tests
- metalForge live hardware: RTX 4070, Titan V, AKD1000 NPU (80 NPs, ~51µs DMA), i9-12900K. Architecture-aware routing: f64→Titan V (Volta), f32→RTX 4070 (Ada)

See [BARRACUDA_EVOLUTION.md](BARRACUDA_EVOLUTION.md) for the module-by-module
GPU promotion mapping.

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
Phase 0 (DONE — Python)        Phase 1 (DONE — Rust CPU)       Phase 2a (DONE — Barracuda CPU)
────────────────────           ─────────────────────────       ──────────────────────────────
NumPy MC (N=10k)    ────────→  prng + fao56 (258/258 PASS) →  bootstrap_mean → barracuda
NumPy stats         ────────→  stats (RMSE/MBE/R²/IA/hit) →   pearson_r, spearman_r, std_dev → barracuda
NumPy Gillespie     ────────→  gillespie::birth_death_ssa  →  (GPU-only: GillespieGpu)
NumPy bootstrap     ────────→  bootstrap::rawr_mean        →  (Gap: no RAWR kernel)
NumPy Anderson      ────────→  anderson::lyapunov_*        →  lyapunov_exponent, lyapunov_averaged → barracuda
NumPy ODE           ────────→  bistable + multisignal      →  BistableOde, MultiSignalOde → barracuda
                                                               11.5× faster (excl. LAPACK), 28/28 parity

Phase 2a (DONE)                Phase 2b (GPU — V31 IN PROGRESS)
──────────────                 ────────────────────────────────
39 active + 7 pending (30 CPU + 9 GPU) →  5 modules GPU-dispatch wired (V31), 2 real GPU ops wired (V42). 49 metalForge tests
prng::Xorshift64    ────────→  Tier B: align to barracuda xoshiro128**
fao56::daily_et0    ────────→  Tier C: mc_et0_propagate.wgsl → barracuda
rarefaction         ────────→  Tier C: batched_multinomial.wgsl → barracuda
gillespie           ────────→  GillespieGpu dispatch (barracuda already has it)

Phase 2b (GPU)                 Phase 3 (Faculty Extension)
─────────────                  ──────────────────────────
GPU MC sampling     ────────→  Jackknife/bootstrap (Bazavov precision)
GPU grid search     ────────→  Regularized inversion (spectral recon)
GPU Gillespie       ────────→  Batched trajectory ensemble
GPU Anderson        ────────→  BatchIprGpu for spectral statistics
N/A                 ────────→  FFT (shared with hotSpring)
N/A                 ────────→  RAWR GPU kernel (new)
N/A                 ────────→  Lanczos eigensolve (Kachkovskiy spectral)
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
