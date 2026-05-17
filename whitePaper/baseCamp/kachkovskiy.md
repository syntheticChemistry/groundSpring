# Kachkovskiy — Anderson Localization & Spectral Theory

**Faculty**: Ilya Kachkovskiy (Mathematics, Michigan State University)
**Previously**: Institute for Advanced Study, UC Irvine
**Co-author**: Jean Bourgain (Fields Medal, 1994)
**Domain**: Anderson localization, quasiperiodic operators, spectral theory
**groundSpring Connection**: Exp 008 (Anderson localization) and Exp 009 (Almost-Mathieu quasiperiodic) — both DONE. Mathematical foundation for ALL five pillars.

---

## Why This Matters for groundSpring

Kachkovskiy's work provides the rigorous mathematical theory behind
groundSpring's central question: **when does signal propagate through a noisy
system, and when does noise win?**

Anderson localization (P.W. Anderson, Nobel Prize 1977) proves that disorder
in a medium can trap waves — preventing propagation entirely. This is the
mathematical formalization of groundSpring's noise-vs-signal framework:

| groundSpring Pillar | Anderson Localization Analog |
|--------------------|-----------------------------|
| Signal vs Noise (Exp 001) | Localized vs extended states |
| Inverse Problems (Exp 005) | Spectral edge constrains inverse solutions |
| Sensing Systems (Exp 002) | Coupled sensors = interacting particles |
| Temporal Dynamics (Exp 004) | Quasiperiodic potential = structured noise |
| Spatial Propagation (Exp 005) | Transport through disordered chain |

## Papers for Reproduction

### Tier 1 (Priority)

**Paper #15**: Bourgain & Kachkovskiy (2018) "Anderson localization for two
interacting quasiperiodic particles." GAFA 29:3-43.
DOI: 10.1007/s00039-019-00478-4

- **Open Data**: Fully analytical — parameters specified in theorems
- **Open Code**: Numerical verification reimplementable from paper
- **groundSpring Modules**: `seismic` (wave propagation), `anderson` module
  (`lyapunov_exponent`, `lyapunov_averaged`, `analytical_localization_length` delegated to barracuda)
- **BarraCUDA Needs**: Sparse eigensolve (Lanczos — exists in barracuda),
  transfer matrix computation, Anderson model solver
- **Control Plan**: Python numerical verification → Rust CPU → barracuda GPU

**Paper #16**: Jitomirskaya & Kachkovskiy (2018) "All couplings localization
for quasiperiodic operators with Lipschitz monotone potentials." JEMS 21:777-795.

- **Open Data**: Analytical; Almost-Mathieu model parameters from paper
- **groundSpring Modules**: `anderson` (extended: `almost_mathieu_potential`,
  `almost_mathieu_hamiltonian`, `level_spacing_ratio`)
- **BarraCUDA Needs**: `almost_mathieu_hamiltonian` **delegated** to barracuda-gpu
  (coupling convention adjusted at delegation boundary)
- **Control Plan**: **DONE** — 8/8 Python, 8/8 Rust. Aubry-André transition at
  λ=2 confirmed, Herman's formula verified, level statistics distinguish phases.

### Tier 2

**Paper #17**: Kachkovskiy (2016) "On transport properties of isotropic
quasiperiodic XY spin chains." CMP 345:659-673.

- **Method**: Energy transport through disordered chains — mathematical framework
  for seismic wave propagation through heterogeneous crust

**Paper #18**: Filonov & Kachkovskiy (2018) "On the structure of band edges
of 2d periodic elliptic operators." Acta Math 221:59-80.

- **Method**: Band edges = frequencies where waves transition from propagating to
  evanescent. The mathematical boundary between "signal gets through" and
  "noise kills it."

## BarraCUDA Kernel Requirements

| Primitive | Status | Notes |
|-----------|--------|-------|
| Lanczos eigensolve | Exists in barracuda `spectral` | For Anderson model |
| Almost-Mathieu operator | Exists in barracuda `spectral` | For quasiperiodic models |
| Anderson 1D/2D/3D | Exists in barracuda `spectral` | `anderson_3d_correlated`, `anderson_sweep_averaged`, `find_w_c` (S59) |
| Transfer matrix | Partial | SpMV exists; chain multiplication needed |
| Level statistics | Exists in barracuda `spectral` | Wigner-Dyson vs Poisson |

## Three-Tier Control Plan

| Tier | Validation | Status |
|------|-----------|--------|
| CPU | Python models match Rust (Exp 008 + 009) | **DONE** (16/16 PASS; 008: 29.9×, 009: parity proven) |
| GPU | barracuda `spectral` matches CPU | `lyapunov_*` delegated, `almost_mathieu_hamiltonian` delegated, `analytical_localization_length` delegated |
| metalForge | Cross-substrate for large lattice | After GPU tier |

## Cross-Spring

- **hotSpring**: Kachkovskiy is in hotSpring's paper queue (spectral theory,
  Anderson localization). Shared barracuda `spectral` module.
- **Shared kernel**: Lanczos eigensolve is needed by both hotSpring (nuclear EOS)
  and groundSpring (Anderson model). Joint priority.

## Extension Roadmap (historical — written at V114)

### V115 Capabilities

- All public APIs now return `Result` — zero panicking entry points
- CI: nursery lints enforced, `--all-features` doc/test, aarch64 cross-compile
- ecoBin: 14 C-dependency crates banned; UniBin flags in primal binary

### V114 Capabilities

- `cast::` helpers in anderson/mod.rs — checked numeric conversions for disorder sweep indices
- `.expect()` → `OrExit` in validation binaries (Exp 008, 009)
- `health.liveness`/`health.readiness` for NUCLEUS deployment
- `resilient_call()` for IPC to ToadStool large-lattice GPU dispatch

### Large-Lattice DF64 Roadmap

| Lattice Size | Precision | Substrate | Status |
|-------------|-----------|-----------|--------|
| L=100-1000 (1D) | f64 | CPU/GPU | Done (Exp 008) |
| L=50-200 (2D/3D) | f64 | GPU (barracuda spectral) | Done (Exp 008) |
| L=14-20 (large 3D) | DF64 (f64-pair) | Titan V (biomeGate) | Planned — hotSpring DF64 shaders |
| L=50+ (large 3D) | DF64 | LAN (2x Titan V + 2x MI50) | Planned — requires 10G cables |

### Extension Opportunities

- **Two-particle Anderson** (Bourgain & Kachkovskiy 2018): Interaction-induced delocalization — relevant to coupled sensor systems
- **Band edge 2D periodic** (Filonov & Kachkovskiy 2018): Frequency-dependent signal filtering through periodic tissue layers (Paper 12 connection)
- **XY spin chain transport** (Kachkovskiy 2016): Already Exp 012 — extend to larger chains via LAN GPU

### Compute Budget

| Workload | Single GPU (RTX 4070) | biomeGate (2x Titan V) | LAN |
|----------|-----------------------|------------------------|-----|
| 1D Anderson sweep L=10000 | ~5min | ~1min | N/A |
| 3D Anderson L=14 (DF64) | ~30min | ~5min (HBM2) | N/A |
| 3D Anderson L=20 (DF64) | ~4h | ~30min | ~10min |
| Two-particle L=500 | ~2h | ~20min | ~5min |

### Primal Wiring

- ToadStool: `compute.execute` for GPU spectral dispatch (Lanczos, Anderson sweep)
- coralReef: Sovereign shader compilation for DF64 Anderson kernels on Titan V
- Node Atomic on biomeGate: 2x Titan V (HBM2) + 2x MI50 for large-lattice DF64 work
- LAN: Distributed Anderson sweep across multiple Node Atomics
