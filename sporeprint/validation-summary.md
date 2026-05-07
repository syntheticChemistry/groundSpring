+++
title = "groundSpring Validation Summary"
description = "Measurement noise and uncertainty — 395/395 Rust + 287 Python checks, uncertainty budget for every spring"
date = 2026-05-06

[taxonomies]
primals = ["barracuda", "toadstool"]
springs = ["groundspring", "hotspring", "wetspring", "neuralspring", "airspring"]
+++

## Status

- **395/395 Rust + 287 Python** cross-validated checks, 936 Rust tests
- **5 pillars**: Signal vs Noise, Inverse Problems, Sensing, Temporal, Spatial
- **30+ papers reproduced** across 7 researchers (Bazavov, Waters, Liu, Kachkovskiy, Dolson, Anderson, Gonzales)
- Contributes uncertainty budget to **every baseCamp paper**
- 102 barraCuda delegations validated across RTX 4070, Titan V, AKD1000

## Key Validation Binaries

<!-- TODO: Update with actual binary names from target/release/ -->
- `validate_signal_noise` — sensor decomposition
- `validate_inverse_problems` — error propagation
- `validate_spectral` — Anderson, Almost-Mathieu, band edge
- `validate_jackknife` — QCD statistical estimation

## Workload TOMLs

Not yet created — contribute to `projectNUCLEUS/workloads/groundspring/`.

## See Also

- [Spring Catalog](https://primals.eco/architecture/spring-catalog-status-science-and-evolution/) on primals.eco
- All baseCamp papers (groundSpring contributes uncertainty to all)
