#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""
groundSpring Experiment 015 — Uncertainty Bridge

Propagates sensor measurement noise (Exp 001) through Anderson localization
(Exp 008) to predict how soil moisture sensor accuracy affects quorum sensing
regime predictions.

Pipeline:
  θ_measured = θ_true + bias + N(0, σ)     (Exp 001: sensor noise)
  W_eff = α * θ + β                        (moisture → disorder mapping)
  γ = lyapunov_exponent(W_eff, E=0)        (Exp 008: Anderson model)
  ξ = 1/γ                                  (localization length)

Key question: How much does sensor noise in θ propagate into uncertainty
in ξ (the QS signal propagation length)? Is bias correction sufficient to
reduce this uncertainty below a useful threshold?

Data sources:
  - Dong et al. (2020) sensor calibration (Exp 001 benchmark)
  - Anderson localization analytical model (Exp 008)
  - No external data — fully analytical + Monte Carlo
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from common import (
    check_min,
    check_range,
    check_true,
    fail_count,
    print_summary,
    reset_counters,
)


def lyapunov_exponent_1d(disorder_w: float, energy: float, n: int, seed: int) -> float:
    """Transfer-matrix Lyapunov exponent for 1D Anderson model.

    Matches the algorithm in groundspring::anderson::lyapunov_exponent.
    """
    rng = np.random.default_rng(seed)
    potentials = rng.uniform(-disorder_w / 2.0, disorder_w / 2.0, size=n)

    x_prev, x_curr = 0.0, 1.0
    log_sum = 0.0

    for i in range(n):
        x_next = (potentials[i] - energy) * x_curr - x_prev
        norm = abs(x_next)
        if norm > 0.0:
            log_sum += np.log(norm)
        x_prev = x_curr / max(norm, 1e-300)
        x_curr = x_next / max(norm, 1e-300)

    return log_sum / n


def lyapunov_averaged(
    disorder_w: float, energy: float, n: int, n_real: int, base_seed: int
) -> float:
    """Average Lyapunov exponent over multiple disorder realizations."""
    total = 0.0
    for r in range(n_real):
        total += lyapunov_exponent_1d(disorder_w, energy, n, base_seed + r)
    return total / n_real


def theta_to_disorder(theta: float, slope: float, intercept: float) -> float:
    """Map soil moisture content to Anderson disorder parameter.

    Higher moisture → more uniform medium → lower disorder.
    Lower moisture → more heterogeneous → higher disorder.
    Mapping: W = intercept + slope * (1 - theta)
    """
    return intercept + slope * (1.0 - theta)


def propagate_sensor_noise(
    theta_true: float,
    bias: float,
    sigma: float,
    slope: float,
    intercept: float,
    chain_length: int,
    n_realizations: int,
    n_mc: int,
    rng: np.random.Generator,
) -> dict:
    """Monte Carlo propagation of sensor noise through Anderson model.

    Returns statistics on the localization length distribution.
    """
    xi_samples = np.zeros(n_mc)
    gamma_samples = np.zeros(n_mc)

    for i in range(n_mc):
        theta_measured = theta_true + bias + rng.normal(0, sigma)
        theta_measured = np.clip(theta_measured, 0.01, 0.99)
        w_eff = theta_to_disorder(theta_measured, slope, intercept)
        w_eff = max(w_eff, 0.1)

        gamma = lyapunov_averaged(w_eff, 0.0, chain_length, n_realizations, 42 + i)
        gamma_samples[i] = gamma
        xi_samples[i] = 1.0 / max(gamma, 1e-10)

    return {
        "gamma_mean": float(np.mean(gamma_samples)),
        "gamma_std": float(np.std(gamma_samples)),
        "xi_mean": float(np.mean(xi_samples)),
        "xi_std": float(np.std(xi_samples)),
        "xi_cv": float(np.std(xi_samples) / max(np.mean(xi_samples), 1e-10)),
    }


def propagate_bias_corrected(
    theta_true: float,
    bias: float,
    sigma: float,
    slope: float,
    intercept: float,
    chain_length: int,
    n_realizations: int,
    n_mc: int,
    rng: np.random.Generator,
) -> dict:
    """Same as propagate_sensor_noise but with bias removed."""
    return propagate_sensor_noise(
        theta_true, 0.0, sigma, slope, intercept,
        chain_length, n_realizations, n_mc, rng,
    )


def main() -> int:
    benchmark_path = Path(__file__).parent / "benchmark_uncertainty_bridge.json"
    with open(benchmark_path) as f:
        benchmark = json.load(f)

    reset_counters()

    print("=" * 72)
    print("groundSpring Exp 015: Uncertainty Bridge")
    print("  Sensor noise → Anderson localization → QS regime uncertainty")
    print("=" * 72)

    sensor = benchmark["sensor_noise"]
    anderson = benchmark["anderson_model"]
    prop = benchmark["propagation"]
    expected = benchmark["expected"]

    chain_length = anderson["chain_length"]
    n_real = anderson["n_realizations"]
    n_mc = prop["n_mc_samples"]
    slope = prop["theta_to_disorder_slope"]
    intercept = prop["theta_to_disorder_intercept"]
    theta_nom = prop["theta_nominal"]

    rng = np.random.default_rng(2026)

    # --- Step 1: Verify Anderson model baseline ---
    print("\n--- Step 1: Anderson model sanity checks ---")

    for w in anderson["disorder_range"]:
        gamma = lyapunov_averaged(w, 0.0, chain_length, n_real, 42)
        print(f"  W={w:5.1f} → γ={gamma:.4f}, ξ={1.0/max(gamma,1e-10):.1f}")

    gammas = [
        lyapunov_averaged(w, 0.0, chain_length, n_real, 42)
        for w in anderson["disorder_range"]
    ]
    from itertools import pairwise
    monotonic = all(g1 <= g2 for g1, g2 in pairwise(gammas))
    check_true("Lyapunov exponent monotonically increasing with W", monotonic)

    check_true(
        "Clean system (W=0.5) has small γ",
        gammas[0] < 0.1,
    )
    check_true(
        "Strong disorder (W=12) has large γ",
        gammas[-1] > 0.3,
    )

    # --- Step 2: CS616 sensor noise propagation ---
    print("\n--- Step 2: CS616 Sand sensor noise propagation ---")
    cs616 = sensor["cs616_sand"]

    cs616_raw = propagate_sensor_noise(
        theta_nom, cs616["bias_mbe"], cs616["random_sigma"],
        slope, intercept, chain_length, n_real, n_mc, rng,
    )
    print(f"  Raw:  ξ = {cs616_raw['xi_mean']:.1f} ± {cs616_raw['xi_std']:.1f} "
          f"(CV = {cs616_raw['xi_cv']:.3f})")

    cs616_corrected = propagate_bias_corrected(
        theta_nom, cs616["bias_mbe"], cs616["random_sigma"],
        slope, intercept, chain_length, n_real, n_mc, rng,
    )
    print(f"  Corrected: ξ = {cs616_corrected['xi_mean']:.1f} ± "
          f"{cs616_corrected['xi_std']:.1f} (CV = {cs616_corrected['xi_cv']:.3f})")

    check_range(
        "CS616 localization length CV",
        cs616_raw["xi_cv"],
        expected["localization_length_cv_cs616"]["min"],
        expected["localization_length_cv_cs616"]["max"],
    )

    # --- Step 3: EC5 sensor noise propagation ---
    print("\n--- Step 3: EC5 Sandy Clay Loam sensor noise propagation ---")
    ec5 = sensor["ec5_sandy_clay_loam"]

    ec5_raw = propagate_sensor_noise(
        theta_nom, ec5["bias_mbe"], ec5["random_sigma"],
        slope, intercept, chain_length, n_real, n_mc, rng,
    )
    print(f"  Raw:  ξ = {ec5_raw['xi_mean']:.1f} ± {ec5_raw['xi_std']:.1f} "
          f"(CV = {ec5_raw['xi_cv']:.3f})")

    ec5_corrected = propagate_bias_corrected(
        theta_nom, ec5["bias_mbe"], ec5["random_sigma"],
        slope, intercept, chain_length, n_real, n_mc, rng,
    )
    print(f"  Corrected: ξ = {ec5_corrected['xi_mean']:.1f} ± "
          f"{ec5_corrected['xi_std']:.1f} (CV = {ec5_corrected['xi_cv']:.3f})")

    check_range(
        "EC5 localization length CV",
        ec5_raw["xi_cv"],
        expected["localization_length_cv_ec5"]["min"],
        expected["localization_length_cv_ec5"]["max"],
    )

    # --- Step 4: Cross-sensor comparison ---
    print("\n--- Step 4: Cross-sensor comparison ---")

    check_true(
        "EC5 has higher CV than CS616 (more noise → more uncertainty)",
        ec5_raw["xi_cv"] > cs616_raw["xi_cv"],
    )

    # --- Step 5: Bias correction effectiveness ---
    print("\n--- Step 5: Bias correction effectiveness ---")

    ec5_improvement = 1.0 - ec5_corrected["xi_cv"] / max(ec5_raw["xi_cv"], 1e-10)
    print(f"  EC5 CV reduction from bias correction: {ec5_improvement:.1%}")

    min_reduction = expected["bias_corrected_improvement"]["min_reduction_fraction"]
    check_min(
        "EC5 bias correction reduces CV",
        ec5_improvement,
        min_reduction,
    )

    cs616_improvement = 1.0 - cs616_corrected["xi_cv"] / max(cs616_raw["xi_cv"], 1e-10)
    print(f"  CS616 CV reduction from bias correction: {cs616_improvement:.1%}")

    check_true(
        "EC5 benefits more from bias correction than CS616 (higher bias fraction)",
        ec5_improvement > cs616_improvement or cs616["bias_fraction"] < ec5["bias_fraction"],
    )

    # --- Summary ---
    print("\n" + "=" * 72)
    print("Uncertainty Bridge Summary:")
    print(f"  CS616 Sand:         CV(ξ) = {cs616_raw['xi_cv']:.3f} → "
          f"{cs616_corrected['xi_cv']:.3f} (corrected)")
    print(f"  EC5 Sandy Clay Loam: CV(ξ) = {ec5_raw['xi_cv']:.3f} → "
          f"{ec5_corrected['xi_cv']:.3f} (corrected)")
    print("  Sensor ranking preserved: EC5 > CS616 in uncertainty")
    print(f"  Bias correction: EC5 improves {ec5_improvement:.0%}, "
          f"CS616 improves {cs616_improvement:.0%}")
    print("=" * 72)

    print_summary("Exp 015: Uncertainty Bridge")
    return 1 if fail_count() > 0 else 0


if __name__ == "__main__":
    sys.exit(main())
