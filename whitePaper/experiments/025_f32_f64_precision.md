# Experiment 025 — f32 vs f64 Precision Drift

| Field | Value |
|-------|-------|
| **Domain** | WDM Molecular Dynamics |
| **Question** | Does f32 accumulation introduce systematic bias in Green-Kubo transport coefficient calculations? |
| **Method** | Synthetic VACF (exponential decay + noise), trapezoidal integration in f64 vs f32, bias-variance decomposition |
| **Reference** | IEEE 754-2019; Higham (2002) Accuracy and Stability of Numerical Algorithms |
| **Checks** | 7 Python, 7 Rust |
| **Key Finding** | f32 introduces measurable systematic bias (~28% of total error); absolute errors scale with integral magnitude |

## Pipeline

- Python → `control/precision_drift/precision_drift.py`
- Benchmark → `control/precision_drift/benchmark_precision_drift.json`
- Rust → `crates/groundspring-validate/src/validate_precision_drift.rs`

## Validation Checks

1. f64 matches analytical (noiseless) within 0.1%
2. f32 max relative error vs f64 < 0.5%
3. Mean relative error (bias) in [-0.5%, 0.5%]
4. Bias fraction above 1% (detectable systematic component)
5. Max absolute diffusion error bounded
6. Error-magnitude correlation (larger integrals → larger errors)
7. Relative error standard deviation bounded
