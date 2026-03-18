# Tolerance Registry

> Complete inventory of named tolerance constants, epsilon guards, and
> validation-specific thresholds used across groundSpring.
>
> **Last updated**: March 18, 2026 (V117 — 13-tier `tol::` architecture,
> 4 `eps::` guards, 15 validation-specific constants)

## Philosophy

Every floating-point comparison in groundSpring uses a **named constant**
with documented provenance. Bare float literals are forbidden in assertions
and validation checks. Each tolerance tier maps to a specific numerical
regime and cites the mathematical bound that justifies it.

## Library Tolerances (`groundspring::tol`)

Source: `crates/groundspring/src/lib.rs`

| Constant | Value | Regime | Provenance |
|----------|-------|--------|------------|
| `DETERMINISM` | 1e-15 | Bitwise reproducibility | ~5× f64 ε (2.22e-16); covers FMA contraction across x86/ARM |
| `STRICT` | 1e-14 | Extended precision summation | Neumaier compensated sum; stricter than `EXACT` |
| `EXACT` | 1e-12 | Summation-only paths | Kahan O(N·ε), N ≤ 10⁴ |
| `ANALYTICAL` | 1e-10 | One transcendental (sqrt, ln) | IEEE 754 1 ULP + composition padding |
| `INTEGRATION` | 1e-8 | ODE RK4 accumulation | O(dt⁴), dt=0.01, 1000 steps |
| `CDF_APPROX` | 1e-6 | CDF/erf approximation | A&S 7.1.26 max error 1.5e-7; chi² compounds twice |
| `ROUNDTRIP` | 1e-5 | CDF↔PPF round-trip | PPF Newton (3 steps); ≈ CDF_APPROX², padded for edges |
| `RECONSTRUCTION` | 1e-4 | Tikhonov regularized RMSE | Hansen (1998) well-conditioned problems |
| `LITERATURE` | 0.001 | Published 3–4 sig figs | Scientific literature convention |
| `DECOMPOSITION` | 0.005 | Bias-variance fractions | Pythagorean identity rounding ~0.5% |
| `STOCHASTIC` | 0.01 | O(1/√N) mean estimator | CLT: σ/√N, N=10⁴, σ≈1 → SE≈0.01 |
| `NORM_2PCT` | 0.02 | Integral conservation | Trapezoidal O(h²), N≤100, padded for boundaries |
| `EQUILIBRIUM` | 0.1 | Physical measurement precision | Sensor/weather ~10%; ODE steady-state convergence |

## Production Epsilon Guards (`groundspring::eps`)

Source: `crates/groundspring/src/lib.rs`

| Constant | Value | Purpose |
|----------|-------|---------|
| `SAFE_DIV` | 1e-10 | Division guard: `x / y.max(eps::SAFE_DIV)` |
| `SSA_FLOOR` | 1e-15 | Gillespie SSA steady-state guard (~10× f64::EPSILON) |
| `LOG_FLOOR` | 1e-15 | Near-zero guard for log/entropy sums |
| `UNDERFLOW` | 1e-300 | Condition number / matrix element underflow guard |

## Validation-Specific Tolerances (`groundspring_validate::tolerances`)

Source: `crates/groundspring-validate/src/tolerances.rs`

### Re-exports from `tol::`

| Constant | Source |
|----------|--------|
| `TOL_EXACT` | `tol::EXACT` |
| `TOL_ANALYTICAL` | `tol::ANALYTICAL` |
| `TOL_LITERATURE` | `tol::LITERATURE` |
| `TOL_DECOMPOSITION` | `tol::DECOMPOSITION` |
| `TOL_STOCHASTIC_MEAN` | `tol::STOCHASTIC` |
| `TOL_EQUILIBRIUM` | `tol::EQUILIBRIUM` |
| `TOL_DETERMINISM` | `tol::DETERMINISM` |

### Validation-only constants

| Constant | Value | Purpose |
|----------|-------|---------|
| `TOL_RAREFACTION_PROP` | 0.05 | Multinomial sampling variance at N ≈ 50k |
| `TOL_REGIME` | 0.5 | Coarse stochastic regime classification |
| `TOL_GRID_MATCH` | 0.01 | Grid-search disorder/coupling matching |
| `TOL_MONOTONIC_SLACK` | 0.15 | Finite-sampling non-monotonicity slack |
| `TOL_ET0` | 0.005 | Rust vs Python ET₀ rounding (Hargreaves worst-case 0.004) |
| `THRESHOLD_GOOD_R2` | 0.95 | Strong regression fit (R² ≥ 0.95) |
| `THRESHOLD_GOOD_IA` | 0.9 | Willmott Index of Agreement threshold |
| `THRESHOLD_LARGE_GAMMA` | 0.3 | Anderson strong-disorder Lyapunov threshold |
| `EPS_SAFE_DIV` | 1e-10 | Validation division guard |
| `EPS_SAFE_DIV_STRICT` | 1e-20 | Strict guard for diffusion coefficients (~1e-15 m²/s floor) |
| `ET0_PLAUSIBLE_MIN_MM` | 0.01 | Physical lower bound for daily ET₀ (FAO-56 Table 2) |
| `ET0_PLAUSIBLE_MAX_MM` | 15.0 | Physical upper bound for daily ET₀ (FAO-56 Appendix A) |

## Python Mirror

Source: `control/tolerances.py`

The Python tolerance module mirrors all 28 Rust constants (library + validation)
with provenance comments linking back to the Rust source. This ensures
benchmark JSONs produced by Python baselines use identical thresholds.

## Adding New Tolerances

1. Determine the mathematical regime (which `tol::` tier applies?)
2. If no existing tier fits, add to `tol::` with provenance comment
3. If validation-only, add to `tolerances.rs`
4. Mirror in `control/tolerances.py`
5. Document in this registry
