# Kachkovskiy — Anderson Localization & Spectral Theory

**Faculty**: Ilya Kachkovskiy (Mathematics, Michigan State University)
**Previously**: Institute for Advanced Study, UC Irvine
**Co-author**: Jean Bourgain (Fields Medal, 1994)
**Domain**: Anderson localization, quasiperiodic operators, spectral theory
**groundSpring Connection**: Exp 008 (Anderson localization) — DONE. Mathematical foundation for ALL five pillars.

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
- **groundSpring Modules**: New `quasiperiodic` module (structured noise analysis)
- **BarraCUDA Needs**: Almost-Mathieu operator (exists in barracuda `spectral`)

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
| CPU | Python Anderson model matches Rust | **DONE** (8/8 PASS, 29.8× faster) |
| GPU | barracuda `spectral` matches CPU | `lyapunov_exponent`, `lyapunov_averaged` delegated; `analytical_localization_length` → `barracuda::special::anderson_transport::localization_length` |
| metalForge | Cross-substrate for large lattice | After GPU tier |

## Cross-Spring

- **hotSpring**: Kachkovskiy is in hotSpring's paper queue (spectral theory,
  Anderson localization). Shared barracuda `spectral` module.
- **Shared kernel**: Lanczos eigensolve is needed by both hotSpring (nuclear EOS)
  and groundSpring (Anderson model). Joint priority.
