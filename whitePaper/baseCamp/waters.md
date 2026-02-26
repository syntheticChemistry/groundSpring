# Waters — Biological Signal Specificity

**Faculty**: Christopher Waters (MMG, Michigan State University)
**Domain**: Quorum sensing, c-di-GMP signaling, biofilm formation
**groundSpring Connection**: Exp 006 (signal specificity), Exp 010 (bistable switching), Exp 011 (multi-signal QS) — all DONE. Exp 001 (sensor noise decomposition) applied to biological sensing.

---

## Why This Matters for groundSpring

groundSpring Exp 001 decomposes soil moisture sensor error into correctable bias
and irreducible noise. Waters' c-di-GMP research asks the same question inside
a living cell: when 60+ enzymes control a single diffusible signaling molecule,
how does the cell achieve high-specificity signaling? The answer — spatial
sequestration and kinetic partitioning — is the biological equivalent of
site-specific sensor calibration.

## Papers for Reproduction

### Tier 1 (Priority)

**Paper #9**: Massie et al. (2012) "Quantification of High Specificity Cyclic
di-GMP Signaling." PNAS 109:12746-51. DOI: 10.1073/pnas.1115663109

- **Open Data**: Supplementary data tables in PNAS SI; FRET quantification
  available in supplementary figures
- **Open Code**: ODE model parameters published; reimplementable from Methods
- **groundSpring Modules**: `decompose` (signal vs noise in FRET data),
  `stats` (R², correlation), `gillespie` module (stochastic SSA noise floor)
- **BarraCUDA Needs**: `GillespieGpu` (exists in barracuda — GPU-only, no CPU fallback;
  needs PRNG alignment from Xorshift64 → xoshiro128** before delegation), ODE integrator
  (exists: `BatchedOdeRK4`). Gillespie delegation is Tier B: requires rebaseline of
  all stochastic experiments once PRNG alignment is resolved.
- **Control Plan**: Python ODE → Rust CPU → barracuda GPU (Gillespie + ODE)

### Tier 1 (continued)

**Paper #10**: Fernandez et al. (2020) "V. cholerae adapts to sessile and
motile lifestyles by c-di-GMP regulation of cell shape." PNAS 117:29046-29054.

- **Open Data**: Flow cytometry data in PNAS SI
- **Open Code**: ODE model parameters published; reimplementable from Methods
- **groundSpring Modules**: `bistable` (ODE integration, RK4, Euler-Maruyama),
  `prng` (stochastic noise)
- **BarraCUDA Needs**: `BistableOde::cpu_derivative` **delegated** via `OdeSystem` trait
- **Control Plan**: **DONE** — 10/10 Python, 9/9 Rust, **18.5× faster**

**Paper #11**: Srivastava et al. (2011) "Integration of Cyclic di-GMP and
Quorum Sensing in the Control of vpsT and aphA." J Bacteriology 193:6331-41.

- **Open Data**: qRT-PCR fold-change data in supplementary tables
- **Open Code**: ODE model parameters published; reimplementable from Methods
- **groundSpring Modules**: `multisignal` (dual-signal ODE, RK4, Euler-Maruyama),
  `prng` (stochastic noise)
- **BarraCUDA Needs**: `MultiSignalOde::cpu_derivative` **delegated** via `OdeSystem` trait
- **Control Plan**: **DONE** — 9/9 Python, 8/8 Rust, **46.2× faster**

## BarraCUDA Kernel Requirements

| Primitive | Status | Notes |
|-----------|--------|-------|
| Gillespie SSA | Exists (`GillespieGpu`) | Stochastic simulation algorithm |
| Batched ODE | Exists (`BatchedOdeRK4`) | For deterministic sweeps |
| PRNG streams | Exists (`PrngXoshiro`) | For stochastic runs |
| Bifurcation | Partial | Eigenvalue analysis via `BatchedEighGpu` |

## Three-Tier Control Plan

| Tier | Validation | Status |
|------|-----------|--------|
| CPU | Python ODE baseline matches Rust (Exp 006, 010, 011) | **DONE** (29/29 checks, 18–46× faster) |
| GPU | Gillespie + ODE GPU matches CPU statistics | Exp 006: PRNG alignment; Exp 010/011: ODE delegated |
| metalForge | Cross-substrate stochastic agreement | After GPU tier |

## Cross-Spring

- **wetSpring**: Waters is a primary wetSpring faculty member (QS, ODE,
  cooperation game theory). groundSpring adds the noise quantification layer.
- **Shared with wetSpring**: Experiments 020, 022, 032-037 (Waters ODE/QS work)
