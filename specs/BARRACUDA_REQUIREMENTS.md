# groundSpring — BarraCUDA Requirements

**Last Updated**: May 8, 2026 (V125 — 110 delegations (67 CPU + 43 GPU), 41 modules, barraCuda v0.3.13. V125: deep debt evolution + guideStone L4)
**Purpose**: GPU kernel requirements, gap analysis, and evolution priorities

---

## Current Status

groundSpring Phase 0 (Python), Phase 1 (Rust), and Phase 2a (barracuda CPU) are **complete**.

- 395/395 validation checks across 35 binaries (340 core + 55 NUCLEUS)
- 41 library modules: stats, decompose, fao56, prng, rarefaction, seismic, gillespie, bootstrap, anderson, almost_mathieu, bistable, multisignal, kinetics, transport, drift, rare_biosphere, quasispecies, band_structure, jackknife, freeze_out, spectral_recon, wdm, biomeos, nestgate, esn, lanczos, linalg, error, tissue_anderson, niche, primal_names, ipc, rawr (+cast, validate, npu, dispatch, tol, eps, gpu)
- 965+ Rust tests + 287 Python tests. 0 clippy warnings (pedantic + nursery). ≥92% library line coverage. 110 delegations (67 CPU + 43 GPU) — barraCuda v0.3.13. 138 metalForge checks. biomeOS Neural API live (V113). `PrecisionRoutingAdvice` wired into 11 GPU dispatch paths.
- Two feature gates: `barracuda` (67 active CPU delegations) and `barracuda-gpu` (43 GPU delegations including Sturm tridiag, tikhonov solve, detect_bands, BatchedMultinomialGpu). Three-mode CI validates all configurations.
- 110 delegations (67 CPU + 43 GPU; includes GPU grid adapters, GPU stats dispatch, batch APIs, regression suite, kimura, jackknife, fao56_et0, thornthwaite_et0, thornthwaite_heat_index, fit_all, chao1, error_threshold, detection_power, detection_threshold, bootstrap_mean, shannon — barraCuda v0.3.13)
- 2 production WGSL shaders in `metalForge/shaders/` (261 combined lines)
- All matrices use flat row-major `Vec<f64>` — GPU-promotable layout
- Rust is **11.6× faster** than Python (excl. LAPACK-bound); 5.1× overall. Exp 009: **47.7× from Sturm tridiag**
- **29/29 mathematical parity proven** (Python ⇌ Rust; generate via `python3 scripts/parity_report.py`), all provenance fields stamped, 13 bitwise determinism tests
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
| **FFT (spectral reconstruction)** | Bazavov 2025 | Complex FFT for lattice correlator analysis | **WIRED** | `spectral_recon::fft_power_spectrum()` delegates to `Fft1DF64` (V93). CPU DFT fallback. |
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
                                                               11.6× faster (excl. LAPACK), 28/28 parity

Phase 2a (DONE)                Phase 2b (GPU — V31 IN PROGRESS)
──────────────                 ────────────────────────────────
73 active (43 CPU + 30 GPU) → GPU grid adapters + 3 new CPU delegations (V53), 12-workload benchmark. 187 metalForge checks
prng::Xorshift64    ────────→  Tier B: align to barracuda xoshiro128**
fao56::daily_et0    ────────→  Tier C: mc_et0_propagate.wgsl absorbed S72 ✓
rarefaction         ────────→  Tier C: batched_multinomial.wgsl absorbed S76 ✓
gillespie           ────────→  GillespieGpu dispatch (barracuda already has it)

Phase 2b (GPU)                 Phase 3 (Faculty Extension)
─────────────                  ──────────────────────────
GPU MC sampling     ────────→  Jackknife/bootstrap (Bazavov precision)
GPU grid search     ────────→  Regularized inversion (spectral recon)
GPU Gillespie       ────────→  Batched trajectory ensemble
GPU Anderson        ────────→  BatchIprGpu for spectral statistics
barracuda::ops::fft ────────→  FFT 1D/2D/3D f32/f64 (WIRED V93: spectral_recon::fft_power_spectrum)
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
