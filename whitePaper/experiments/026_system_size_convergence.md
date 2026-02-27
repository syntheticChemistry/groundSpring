# Experiment 026 — System-size Convergence for WDM Transport

| Field | Value |
|-------|-------|
| **Domain** | WDM Molecular Dynamics |
| **Question** | At what system size N does consumer GPU transport converge to the thermodynamic limit? |
| **Method** | Synthetic D(N) = D∞ + α/N^(1/d) with noise, linear regression extrapolation |
| **Reference** | Yeh & Hummer (2004) J. Phys. Chem. B 108:15873; Dünweg & Kremer (1993) |
| **Checks** | 7 Python, 7 Rust |
| **Key Finding** | Finite-size correction fits with R² > 0.999; extrapolation within 1% of true D∞ |

## Pipeline

- Python → `control/size_convergence/size_convergence.py`
- Benchmark → `control/size_convergence/benchmark_size_convergence.json`
- Rust → `crates/groundspring-validate/src/validate_size_convergence.rs`

## Validation Checks

1. Extrapolated D∞ within tolerance of true value
2. Fitted α in expected range
3. R² above 0.95
4. Extrapolation relative error < 10%
5. Convergence achieved by N_max
6. Mean D at largest N in expected range
7. Residual standard deviation bounded
