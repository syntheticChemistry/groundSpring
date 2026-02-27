# Experiment 022: ET₀ → Anderson Uncertainty Propagation

**Domain**: Cross-spring (FAO-56 + Anderson localization)
**Paper**: Allen et al. (1998) FAO-56; Bourgain & Kachkovskiy (2018) GAFA 29:3-43
**Phase 0**: 7/7 PASS (Python)
**Phase 1**: 7/7 PASS (Rust)
**Barracuda**: Uses fao56 + anderson modules

## Question

How much does the 66% humidity-dominated ET₀ error affect Anderson localization length predictions?

## Pipeline

1. Sample FAO-56 inputs with documented uncertainties (humidity dominates at 66%)
2. Compute ET₀ via Penman-Monteith for each MC sample
3. Water balance: ET₀ → θ (soil moisture) over 30 days
4. Map θ → effective disorder W_eff via linear mapping
5. Anderson localization: W_eff → γ (Lyapunov exponent) → ξ = 1/γ
6. Report: CV at each stage, humidity dominance, propagation ratio

## Key Results

- ET₀ mean: 3.89 mm/day, CV: 0.043
- θ final: 0.22, CV: 0.065
- ξ CV: 0.040 (propagation ratio 0.94× ET₀ CV)
- Humidity dominates ET₀ variance (51%)
- ET₀ uncertainty propagates through the full Anderson chain

## Files

| File | Description |
|------|-------------|
| `control/et0_anderson_propagation/et0_anderson_propagation.py` | Python baseline |
| `control/et0_anderson_propagation/benchmark_et0_anderson.json` | Benchmark config |
| `crates/groundspring-validate/src/validate_et0_anderson.rs` | Rust validation binary |

## Cross-Spring

Extends Exp 003 (FAO-56 uncertainty) and Exp 008 (Anderson localization).
Contributes uncertainty budget for baseCamp Sub-thesis 06.
