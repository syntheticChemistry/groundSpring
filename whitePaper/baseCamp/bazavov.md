# Alexei Bazavov — Inverse Problems & Spectral Reconstruction

**Faculty**: Alexei Bazavov (CMSE + Physics, Michigan State University)
**Domain**: Lattice QCD, statistical mechanics, high-precision inverse problems
**groundSpring Experiments**: Exp 019 (jackknife), Exp 020 (freeze-out), Exp 021 (spectral recon)
**Status**: Phase 0 + Phase 1 DONE — 9/9 + 8/8 + 8/8 PASS

---

## Why This Matters for groundSpring

groundSpring Exp 005 solves a simple inverse problem: given noisy P-wave arrival
times at 7 stations, locate the earthquake source. Bazavov's lattice QCD work
contains a rich set of related inverse problems that demand 10-100× tighter
precision. **Three Bazavov papers are now reproduced** in groundSpring:

| Paper | groundSpring Exp | Checks | Method |
|-------|:----------------:|:------:|--------|
| Bazavov et al. (2025) Phys Rev D 111, 094508 | **Exp 019** | 9/9 Py, 9/9 Rust | Jackknife error estimation |
| Bazavov et al. (2016) Phys Rev D 93, 014512 | **Exp 020** | 8/8 Py, 8/8 Rust | Freeze-out chi-squared inverse |
| Bazavov et al. (2025) arXiv 2501.12259 | **Exp 021** | 8/8 Py, 8/8 Rust | Tikhonov spectral reconstruction |

The mathematical structure is identical: noisy observations → forward model →
inverse problem → uncertainty quantification.

## Papers (All Reproduced)

### Exp 019 — Paper 7: Jackknife Error Estimation

**Bazavov et al. (2025)** "Hadronic vacuum polarization for the muon g-2."
Phys Rev D 111, 094508. DOI: 10.1103/PhysRevD.111.094508

- **Method**: Delete-one jackknife, block jackknife, bias correction
- **groundSpring Modules**: `jackknife`, `bootstrap` (comparison)
- **Extends**: Exp 007 RAWR (complementary resampling)

### Exp 020 — Paper 8: Freeze-Out Inverse Problem

**Bazavov et al. (2016)** "Curvature of the freeze-out line in heavy ion
collisions." Phys Rev D 93, 014512.

- **Method**: Chi-squared grid-search to recover T0, κ₂ from noisy observables
- **groundSpring Modules**: `freeze_out`
- **Extends**: Exp 005 seismic (grid-search inverse)

### Exp 021 — Paper 6: Spectral Function Reconstruction

**Bazavov et al. (2025)** "Spectral reconstruction inverse problem in lattice
QCD." arXiv 2501.12259.

- **Method**: Tikhonov-regularized inversion of Laplace kernel
- **groundSpring Modules**: `spectral_recon`
- **Most advanced**: Ill-posed integral equation → regularized least squares

## BarraCUDA Kernel Requirements

| Primitive | Status | Notes |
|-----------|--------|-------|
| FFT (real, complex) | **Gap** — not in barracuda | Optional for spectral; discretized formulation works |
| Matrix inverse / Cholesky | Exists | Tikhonov solve in `spectral_recon` |
| Regularization (Tikhonov) | **DONE** | `spectral_recon::tikhonov_solve` |
| Bootstrap/jackknife | `stats::bootstrap_*`, `jackknife` | CPU complete; GPU embarrassingly parallel |
| Grid search | `freeze_out::grid_fit_2d` | Same structure as Exp 005 seismic |

## Three-Tier Control Plan

| Tier | Validation | Status |
|------|-----------|--------|
| CPU | Python baseline matches Rust | **DONE** (25/25 across Exp 019-021) |
| GPU | barracuda GPU matches CPU | Partial — `tikhonov_solve` GPU delegated; `grid_fit_2d` and `jackknife_mean_variance` now delegated |
| metalForge | Cross-substrate agreement | In progress — 19 workloads defined, `forge::tolerance` module (4 tiers) |

## Cross-Spring

- **hotSpring**: Bazavov is in hotSpring's paper queue (Phase C/D nuclear EOS).
  groundSpring adds the noise/uncertainty perspective. Exp 019-021 validate
  inverse problem primitives for lattice QCD.
- **Shared kernel need**: Grid-search and Cholesky are shared with seismic (Exp 005).

## Extension Roadmap (historical — written at V114)

### V115 Capabilities

- All public APIs now return `Result` — zero panicking entry points for library consumers
- CI: nursery lints enforced, `--all-features` doc/test, biomeOS + metalForge validation jobs
- ecoBin: 14 C-dependency crates banned in `deny.toml`

### V114 Capabilities

- `.expect()` → `OrExit` in validation binaries (validate-jackknife, validate-freeze-out, validate-spectral-recon)
- GemmF64 transpose delegation (V113) for Tikhonov KᵀK/KᵀG — GPU pipeline complete
- `health.liveness`/`health.readiness` for NUCLEUS deployment
- `resilient_call()` for IPC to ToadStool GPU dispatch

### Extension Opportunities

- **GPU FFT**: Real + complex FFT in barraCuda (P1 request) would enable frequency-domain spectral reconstruction
- **NUCLEUS distributed**: Freeze-out parameter sweep across multiple Node Atomics on LAN
- **Cross-spring with hotSpring**: Bazavov papers connect directly to hotSpring lattice QCD (Phase C/D) — shared jackknife and spectral reconstruction primitives
- **Spectral recon at scale**: Larger correlator matrices (N_tau > 200) benefit from GPU Cholesky + Tikhonov pipeline

### Compute Budget

| Workload | Single GPU (RTX 4070) | LAN |
|----------|-----------------------|-----|
| Jackknife (N=1000, block sizes 1-50) | ~10s | N/A |
| Freeze-out grid search (100×100 grid) | ~30s | N/A |
| Spectral recon (N_tau=200, N_omega=200) | ~5s (GPU) | N/A |
| Distributed parameter sweep (10K points) | ~30min | ~5min |

### Primal Wiring

- ToadStool: `compute.execute` for GPU Tikhonov pipeline and grid search
- coralReef: Sovereign shader compilation for precision-critical inverse problems
- NestGate: Storage for large correlator datasets from hotSpring
- Node Atomic on biomeGate: Titan V (HBM2) for high-precision Tikhonov at scale
