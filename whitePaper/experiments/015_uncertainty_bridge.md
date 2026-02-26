# Experiment 015 — Uncertainty Bridge: Sensor Noise → Anderson → QS

## Summary

Cross-domain uncertainty propagation experiment that bridges three validated
groundSpring modules: sensor noise decomposition (Exp 001), Anderson
localization (Exp 008), and the noise characterization framework.

**Core question**: How much does soil moisture sensor noise propagate
through physical models into predictions about signal propagation
(localization) length?

## Pipeline

```
θ_measured = θ_true + bias + N(0,σ)     Exp 001: sensor noise model
W_eff = α(1 − θ) + β                   moisture → disorder mapping
γ = lyapunov_averaged(W_eff, E=0)       Exp 008: Anderson model
ξ = 1/γ                                localization length
```

## Scientific Motivation

The Anderson-QS bridge (Gen3 Sub-thesis 01+06) proposes that Anderson
localization in disordered soil provides a null hypothesis for quorum
sensing signal range: if physical disorder alone limits signal propagation
to length ξ, then any observed QS communication beyond ξ implies active
biological amplification.

This experiment asks the precursor question: given that we measure θ with
known sensor noise (Exp 001), how confident can we be in our estimate of ξ?

## Data Sources

- **Dong et al. (2020)** sensor calibration: CS616 Sand (bias=-0.010, σ=0.014) and EC5 Sandy Clay Loam (bias=-0.050, σ=0.027)
- **Anderson analytical model**: 1D transfer matrix, chain length 200, 10 realizations per disorder value
- **No external data**: fully analytical + Monte Carlo

## Key Results

| Sensor | CV(ξ) raw | CV(ξ) corrected | Improvement |
|--------|-----------|-----------------|-------------|
| CS616 Sand | 0.027–0.032 | 0.026–0.031 | ~5% |
| EC5 Sandy Clay Loam | 0.043–0.041 | 0.043–0.042 | ~0% |

**Key finding**: At the disorder levels corresponding to typical soil
moisture (θ≈0.30), the Lyapunov exponent is in the saturated regime
where bias correction has minimal effect on ξ uncertainty. This is
physically meaningful: in strongly disordered media, the localization
length is insensitive to small perturbations in disorder strength.

Sensor ranking is preserved: EC5 (higher noise) produces higher CV(ξ)
than CS616 (lower noise), as expected.

## Phase 0 Status

- Python baseline: 8/8 PASS
- Rust validation: 8/8 PASS

## Modules Used

- `groundspring::anderson` — Lyapunov exponent, localization length
- `groundspring::prng` — Xorshift64 for Monte Carlo noise generation
- `groundspring::validate` — Validation harness

## References

- Dong et al. (2020) "Calibration and validation of soil moisture sensors" Agriculture 10:598
- Bourgain & Kachkovskiy (2018) "Anderson localization for two interacting quasiperiodic particles" GAFA 29:3-43
- Allen et al. (1998) FAO Irrigation and Drainage Paper 56
