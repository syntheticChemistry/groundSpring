# Exp 011: Multi-Signal QS Integration

**Domain**: Biological (quorum sensing, multi-input signal fusion)
**Paper**: Srivastava et al. (2011, J Bacteriology) — Dual-signal QS in V. cholerae
**Faculty**: Christopher Waters (MMG, MSU)
**Question**: How does multi-input signal fusion behave in a noisy
quorum-sensing environment, and does redundancy sharpen regulation?

## Data Source

ODE model of V. cholerae dual-signal quorum sensing. Two autoinducer
systems (CAI-1 via CqsS, AI-2 via LuxPQ) converge on the master regulator
HapR, which represses biofilm formation via DGC/c-di-GMP. Parameters from
Srivastava et al. (2011) supplementary methods.

## Method

1. **Deterministic ODE** (RK4): Compare single-signal (CAI-1 only, AI-2 only)
   versus dual-signal steady states for HapR and biofilm levels.
2. **Stochastic ODE** (Euler-Maruyama): Add Gaussian noise (σ=0.05) and
   compare variance of HapR under single vs dual signaling.
3. **Biological validation**: Confirm that dual signaling increases HapR
   (more input), which *represses* biofilm (HapR inhibits DGC), and that
   dual signaling produces lower HapR variance (more robust regulation).

## Key Result

**Dual signaling sharpens regulation.** Dual-signal HapR > single-signal
HapR (both inputs contribute). Higher HapR represses DGC more effectively,
leading to lower biofilm. Critically, dual signaling produces lower HapR
variance than single signaling — the system integrates redundant signals
for more robust (less noisy) regulatory output.

## Performance

| Metric | Python | Rust | Speedup |
|--------|--------|------|---------|
| Time | 4.30s | 0.09s | **46.2×** |

## Validation

| Phase | Checks | Binary |
|-------|--------|--------|
| Phase 0 (Python) | 9/9 | `control/multisignal_qs/multisignal_qs.py` |
| Phase 1 (Rust) | 8/8 | `validate-multisignal` |

Checks: Cell density carrying capacity, positive HapR and biofilm, dual
HapR > single HapR, dual biofilm < single biofilm (HapR repression),
determinism, low-noise agreement, dual-signal lower HapR variance.

## Barracuda Path

`MultiSignalOde::cpu_derivative` **delegated** to barracuda via the
`OdeSystem` trait. `MultiSignalParams::to_flat()` converts the named
parameter struct to barracuda's flat `&[f64]` convention. Same RK4/E-M
pattern as Exp 010.

## Modules

`multisignal` (new: `MultiSignalParams`, `hill`, `hill_repress`,
`multisignal_derivative`, `rk4_step`, `integrate`, `stochastic_integrate`),
`prng`
