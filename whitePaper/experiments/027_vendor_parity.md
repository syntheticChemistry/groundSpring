# Experiment 027 — GPU Vendor Parity for WDM Observables

| Field | Value |
|-------|-------|
| **Domain** | WDM Molecular Dynamics |
| **Question** | Do GPU vendor/driver differences affect transport coefficient results? |
| **Method** | Same Green-Kubo integration with simulated vendor perturbation (ε ~ 1e-10), parity testing |
| **Reference** | hotSpring GPU vendor parity framework; IEEE 754-2019 |
| **Checks** | 7 Python, 7 Rust |
| **Key Finding** | Vendor differences at 1e-12 relative level; correlation 1.000000; chi²/DOF ≈ 0 |

## Pipeline

- Python → `control/vendor_parity/vendor_parity.py`
- Benchmark → `control/vendor_parity/benchmark_vendor_parity.json`
- Rust → `crates/groundspring-validate/src/validate_vendor_parity.rs`

## Validation Checks

1. Max relative difference < 1e-5
2. Mean relative difference < 1e-6
3. Vendor correlation > 0.999999
4. Bias fraction < 10%
5. Max absolute difference < 1e-6
6. All observables within tolerance
7. Chi-squared per DOF < 5.0
