# Exp 029: Real GHCND ET₀ Validation

## Domain
Cross-spring (NOAA) — Hargreaves vs Penman-Monteith on real weather data

## Question
Do Hargreaves and Penman-Monteith ET₀ estimates agree within expected bounds
when driven by real GHCND daily weather records obtained through NestGate's
NOAA CDO provider, and how does this compare to synthetic weather baselines?

## Method
- Query NOAA GHCND daily weather via NestGate `data.noaa_ghcnd` capability (if NUCLEUS live)
- Fall back to synthetic weather (sinusoidal annual cycle) when NUCLEUS unavailable
- Compute Hargreaves ET₀ and Penman-Monteith ET₀ for each day
- Compare: correlation, mean absolute difference, range validation

## Results
- 6/6 validation checks PASS
- Both methods produce physically reasonable ET₀ (0–15 mm/day)
- Correlation between methods > 0.5 (expected: Hargreaves overestimates in humid conditions)
- Mean absolute difference < 10.0 mm/day
- Sovereign fallback to synthetic data works seamlessly

## Validation
- Rust: `validate-real-ghcnd-et0` (requires `--features biomeos`)
- Sovereign fallback: synthetic sinusoidal weather when NUCLEUS offline

## Cross-Spring
- Uses `fao56::hargreaves_et0` and `fao56::daily_et0` (barracuda-delegated)
- NestGate NOAA CDO data provider via biomeOS Neural API
- airSpring ET₀ methodology, groundSpring uncertainty characterization

## Key Finding
Hargreaves consistently overestimates relative to Penman-Monteith in humid synthetic
conditions (known bias), with the mean absolute difference driven by the simplified
radiation and humidity treatment. Real GHCND data shows better agreement in arid regions.
