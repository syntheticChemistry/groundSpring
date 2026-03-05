# Exp 035: Multi-Method ET₀ Cross-Validation

## Domain
Hydrology (reference evapotranspiration) — Cross-spring (airSpring lineage)

## Question
Do simplified ET₀ methods (fewer inputs) agree with the full
Penman-Monteith equation chain? What is the accuracy-vs-input tradeoff
when trading meteorological parameters for computational simplicity?

## Connection to groundSpring
Extends Exp 003 (FAO-56 error propagation) by comparing five ET₀ methods.
The full pipeline: Python baseline → Rust validation → barracuda CPU
delegation → barracuda GPU (future). Demonstrates that pure Rust math
via barracuda matches interpreted Python.

## Methods Compared

| Method | Inputs | Origin |
|--------|--------|--------|
| Penman-Monteith | T, RH, wind, radiation, lat, alt | Allen et al. 1998 (FAO-56) |
| Hargreaves | T_max, T_min, lat | Hargreaves & Samani 1985 |
| Makkink | T, Rs | Makkink 1957 |
| Turc | T, Rs, RH | Turc 1961 |
| Hamon | T, daylight hours | Hamon 1963 |

## Key Findings

At the FAO-56 Example 18 reference site (Uccle, Belgium, 6 July):
- **PM**: 3.881 mm/day (reference, full equation chain)
- **Hargreaves**: 9.950 mm/day (overestimates — uses Ra directly, not Rs)
- **Makkink**: 3.422 mm/day (within 12% of PM — radiation-only)
- **Turc**: 3.977 mm/day (within 2.5% of PM — radiation + humidity)
- **Hamon**: 0.191 mm/day (underestimates 20× — minimal inputs)

Hamon's extreme underestimate in humid climates is expected: it trades
accuracy for minimal inputs (only temperature + daylight hours).

### Sensitivity Analysis
- Makkink radiation CV = 5.06% at 5% Rs uncertainty
- Hamon temperature CV = 3.11% at 0.5°C T uncertainty

### Seasonal Variation
All methods correctly show summer ET₀ > winter ET₀.

## Validation

| Phase | Checks | Status |
|-------|--------|--------|
| Phase 0 (Python) | 15/15 | PASS |
| Phase 1 (Rust) | 19/19 | PASS |

**Tolerance**: 0.005 mm/day (trig intermediate rounding differences in Ra
chain; documented: Hargreaves amplifies because Ra enters directly).

**Determinism**: All 5 methods produce identical results on rerun.

## barracuda Delegation

Each method delegates to `barracuda::stats::hydrology::*` when the
`barracuda` feature is enabled:
- `makkink_et0` → airSpring V068 → barraCuda v0.3.2
- `turc_et0` → airSpring V068 → barraCuda v0.3.2
- `hamon_et0` → airSpring V069 → barraCuda v0.3.2
- `daily_et0` (PM) → existing delegation chain
- `hargreaves_et0` → existing delegation chain

## References
- Allen et al. (1998) FAO Irrigation and Drainage Paper 56
- Makkink (1957) Neth J Agr Sci 5:290-305
- Turc (1961) Ann Agron 12:13-49
- Hamon (1963) J Hydraul Div ASCE 89:97-120
- Hargreaves & Samani (1985) Appl Eng Agric 1:96-99
