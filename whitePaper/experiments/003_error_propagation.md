# Exp 003: Error Propagation Through FAO-56

**Domain**: Agricultural (evapotranspiration)
**Paper**: FAO Irrigation & Drainage Paper 56, Example 18 (Uccle, Belgium)
**Question**: How do sensor uncertainties propagate through the Penman-Monteith equation?

## Data Source

FAO-56 Example 18 reference values.
WMO sensor uncertainty specifications.
Open data — published standard reference.

## Method

Monte Carlo propagation (N=10,000) through the full Penman-Monteith chain.
Sensitivity analysis via variance fraction decomposition.
Analytical Taylor expansion comparison.

## Key Result

**Humidity is the bottleneck** — 66% of ET₀ variance comes from humidity
sensor uncertainty, despite temperature being the headline variable.
A $5 humidity sensor upgrade has more impact than a $50 pyranometer upgrade.

## Validation

| Phase | Checks | Binary |
|-------|--------|--------|
| Phase 0 (Python) | PASS | `control/error_propagation/error_propagation_fao56.py` |
| Phase 1 (Rust) | 15/15 | `validate-fao56` |

## Barracuda Path

FAO-56 equation chain **absorbed upstream** as `Op::Fao56Et0` (ToadStool S49).
MC noise wrapper (`mc_et0_propagate.wgsl`) **absorbed S72** — local shader removed V62.

## Modules

`fao56`, `prng`, `stats`
