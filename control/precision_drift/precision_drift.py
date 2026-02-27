# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
#!/usr/bin/env python3
"""
groundSpring Experiment 025 — f32 vs f64 Transport Coefficient Precision Drift

Validates the analysis methodology for detecting f32→f64 precision drift in
WDM (Warm Dense Matter) transport coefficient calculations via Green-Kubo
integration of velocity autocorrelation functions.

Core idea:
  1. Synthetic VACF: C(t) = c0 * exp(-t/tau) with additive noise
  2. Trapezoidal integration in f64 (reference) vs f32 accumulation (GPU-like)
  3. Bias-variance decomposition of f32-f64 differences
  4. Error-magnitude correlation (larger integrals → larger rounding drift)

References:
  IEEE 754-2019, Higham (2002) Accuracy and Stability of Numerical Algorithms
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from common import (
    check_max,
    check_min,
    check_range,
    decompose_error,
    print_summary,
    reset_counters,
)


# ---------------------------------------------------------------------------
# VACF generation and integration
# ---------------------------------------------------------------------------


def synthetic_vacf_noisy(
    c0: float,
    tau: float,
    n_steps: int,
    dt: float,
    noise_amplitude: float,
    rng: np.random.Generator,
) -> np.ndarray:
    """Generate synthetic VACF with decay-enveloped noise.

    vacf = c0 * exp(-t/tau) + noise_amplitude * N(0,1) * exp(-t/tau)
    """
    t = np.arange(n_steps, dtype=np.float64) * dt
    decay = np.exp(-t / tau)
    signal = c0 * decay
    noise = noise_amplitude * rng.standard_normal(n_steps) * decay
    return signal + noise


def green_kubo_f32_scalar(vacf: np.ndarray, dt: float) -> float:
    """Trapezoidal integration with f32 accumulator (matches Rust behavior).

    Uses scalar np.float32 accumulator in a loop, not vectorized.
    """
    n = len(vacf)
    if n < 2:
        return 0.0
    acc = np.float32(0.5 * (vacf[0] + vacf[n - 1]))
    for i in range(1, n - 1):
        acc += np.float32(vacf[i])
    return float(acc * np.float32(dt))


def main() -> int:
    benchmark_path = Path(__file__).parent / "benchmark_precision_drift.json"
    with open(benchmark_path) as f:
        benchmark = json.load(f)

    reset_counters()

    print("=" * 72)
    print("groundSpring Exp 025: f32 vs f64 Precision Drift")
    print("  WDM transport coefficient Green-Kubo integration")
    print("=" * 72)

    model = benchmark["model"]
    exp = benchmark["expected_results"]

    c0 = model["c0"]
    dt = model["dt"]
    d_dim = model["d_dim"]
    n_steps = model["n_steps"]
    tau_values = model["tau_values"]
    noise_amplitude = model["noise_amplitude"]
    n_realizations = model["n_realizations"]
    seed = model["seed"]

    rng = np.random.default_rng(seed)

    # Noiseless run for f64 vs analytical (check 1)
    # Finite window: ∫₀ᵀ c0*exp(-t/τ) dt = c0*τ*(1 - exp(-T/τ))
    t_max = (n_steps - 1) * dt
    f64_noiseless_max_rel_err = 0.0
    for tau in tau_values:
        vacf_clean = c0 * np.exp(-np.arange(n_steps, dtype=np.float64) * dt / tau)
        integral_f64_clean = float(np.trapezoid(vacf_clean, dx=dt))
        analytical_finite = c0 * tau * (1.0 - np.exp(-t_max / tau))
        if analytical_finite != 0:
            rel_err = abs(integral_f64_clean - analytical_finite) / analytical_finite
            f64_noiseless_max_rel_err = max(f64_noiseless_max_rel_err, rel_err)

    # Collect results across all tau and realizations (noisy)
    all_integral_f64: list[float] = []
    all_integral_f32: list[float] = []
    all_analytical: list[float] = []
    tau_per_realization: list[float] = []

    for tau in tau_values:
        analytical_integral = c0 * tau

        integrals_f64: list[float] = []
        integrals_f32: list[float] = []
        rel_errors: list[float] = []

        for _ in range(n_realizations):
            vacf = synthetic_vacf_noisy(
                c0, tau, n_steps, dt, noise_amplitude, rng
            )
            integral_f64 = float(np.trapezoid(vacf, dx=dt))
            integral_f32 = green_kubo_f32_scalar(vacf, dt)

            all_integral_f64.append(integral_f64)
            all_integral_f32.append(integral_f32)
            all_analytical.append(analytical_integral)
            tau_per_realization.append(tau)

            integrals_f64.append(integral_f64)
            integrals_f32.append(integral_f32)
            if integral_f64 != 0:
                rel_errors.append((integral_f32 - integral_f64) / integral_f64)

        mean_f64 = float(np.mean(integrals_f64))
        std_f64 = float(np.std(integrals_f64))
        mean_rel = float(np.mean(rel_errors)) if rel_errors else 0.0
        print(
            f"\n  tau={tau:.1f}: f64 mean={mean_f64:.6f}, std={std_f64:.6f}, "
            f"mean_rel_err={mean_rel:.6f}"
        )

    # Derive global stats
    all_relative_errors = [
        (all_integral_f32[i] - all_integral_f64[i]) / all_integral_f64[i]
        for i in range(len(all_integral_f64))
        if all_integral_f64[i] != 0
    ]
    all_abs_errors = [
        abs(all_integral_f32[i] - all_integral_f64[i])
        for i in range(len(all_integral_f64))
        if all_integral_f64[i] != 0
    ]
    all_f64_magnitudes = [
        abs(all_integral_f64[i])
        for i in range(len(all_integral_f64))
        if all_integral_f64[i] != 0
    ]

    # --- Validation checks ---
    print("\n--- Validation Checks ---")

    # Check 1: f64 max relative error vs analytical (noiseless VACF)
    check_max(
        "f64 max relative error vs analytical (noiseless)",
        f64_noiseless_max_rel_err,
        exp["f64_analytical_max_error"],
    )

    # Check 2: f32 max relative error vs f64
    f32_max_rel_err = max(abs(r) for r in all_relative_errors) if all_relative_errors else 0.0
    check_max(
        "f32 max relative error vs f64",
        f32_max_rel_err,
        exp["f32_relative_error_max"],
    )

    # Check 3: Mean relative error (bias) in range
    mean_rel_err = float(np.mean(all_relative_errors)) if all_relative_errors else 0.0
    lo, hi = exp["mean_relative_error_range"]
    check_range(
        "Mean relative error (bias detection) in range",
        mean_rel_err,
        lo,
        hi,
    )

    # Check 4: Bias fraction above minimum
    errors = np.array(
        [
            all_integral_f32[i] - all_integral_f64[i]
            for i in range(len(all_integral_f64))
        ]
    )
    mbe = float(np.mean(errors))
    rmse = float(np.sqrt(np.mean(errors**2)))
    decomp = decompose_error(mbe, rmse)
    check_min(
        "Bias fraction above minimum",
        decomp["bias_fraction"],
        exp["bias_fraction_min"],
    )

    # Check 5: Max absolute diffusion error (f32 vs f64, not vs analytical)
    max_diff_err = 0.0
    for i in range(len(all_integral_f64)):
        d_f64 = all_integral_f64[i] / d_dim
        d_f32 = all_integral_f32[i] / d_dim
        max_diff_err = max(max_diff_err, abs(d_f32 - d_f64))
    check_max(
        "Max absolute diffusion error (f32 vs f64)",
        max_diff_err,
        exp["max_diffusion_absolute_error"],
    )

    # Check 6: Error-magnitude correlation (|f32-f64| vs expected integral c0*tau)
    abs_errors_arr = np.array(all_abs_errors)
    expected_magnitudes = np.array(
        [c0 * tau_per_realization[i] for i in range(len(all_integral_f64)) if all_integral_f64[i] != 0]
    )
    if len(abs_errors_arr) >= 2 and len(abs_errors_arr) == len(expected_magnitudes):
        corr = float(np.corrcoef(abs_errors_arr, expected_magnitudes)[0, 1])
        if np.isnan(corr):
            corr = 0.0
    else:
        corr = 0.0
    check_min(
        "Error-magnitude correlation (larger integrals → larger errors)",
        corr,
        exp["error_magnitude_correlation_min"],
    )

    # Check 7: Relative error std bounded
    rel_err_std = float(np.std(all_relative_errors)) if all_relative_errors else 0.0
    check_max(
        "Relative error std",
        rel_err_std,
        exp["relative_error_std_max"],
    )

    return print_summary("Exp 025: f32 vs f64 Precision Drift")


if __name__ == "__main__":
    sys.exit(main())
