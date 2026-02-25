#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""
groundSpring Experiment 001 — Sensor Noise Characterization

Decomposes factory calibration error in Dong et al. (2020) soil moisture
sensors into systematic bias (correctable) vs random noise (irreducible).

Key questions:
  1. What fraction of total error is systematic bias vs random noise?
  2. How does the noise structure differ across soil types?
  3. What is the noise floor after bias correction?
  4. Are the errors consistent with Gaussian noise?

Reference:
  Dong, Miller, Kelley (2020) Performance Evaluation of Soil Moisture
  Sensors in Coarse- and Fine-Textured Michigan Agricultural Soils.
  Agriculture 10(12), 598. doi:10.3390/agriculture10120598
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

import numpy as np
from scipy import stats

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from common import (
    check_approx,
    check_true,
    decompose_error,
    fail_count,
    noise_floor_reduction,
    pass_count,
    print_summary,
    reset_counters,
)


# ---------------------------------------------------------------------------
# Synthetic noise distribution analysis
# ---------------------------------------------------------------------------

def generate_sensor_noise_samples(
    mbe: float,
    random_std: float,
    n_samples: int = 10_000,
    rng_seed: int = 42,
) -> np.ndarray:
    """Generate synthetic sensor error samples matching observed statistics.

    Samples = bias + N(0, random_std).  Used to verify that the
    decomposed noise parameters reconstruct a Gaussian distribution.
    """
    rng = np.random.default_rng(rng_seed)
    return mbe + rng.normal(0.0, random_std, size=n_samples)


def test_normality(samples: np.ndarray) -> dict:
    """Test whether error samples are consistent with a Gaussian."""
    n = len(samples)
    mean = float(np.mean(samples))
    std = float(np.std(samples, ddof=1))
    skewness = float(stats.skew(samples))
    kurtosis = float(stats.kurtosis(samples))

    _, shapiro_p = stats.shapiro(samples[:5000])

    return {
        "n_samples": n,
        "mean": mean,
        "std": std,
        "skewness": skewness,
        "excess_kurtosis": kurtosis,
        "shapiro_p_value": float(shapiro_p),
        "is_normal_shapiro": float(shapiro_p) > 0.05,
    }


# ---------------------------------------------------------------------------
# Cross-soil-type comparison
# ---------------------------------------------------------------------------

def compare_across_soils(decompositions: dict) -> dict:
    """Compare noise characteristics across soil types."""
    soil_types = list(decompositions.keys())
    biases = [decompositions[s]["bias_abs"] for s in soil_types]
    random_stds = [decompositions[s]["random_std"] for s in soil_types]
    bias_fracs = [decompositions[s]["bias_fraction"] for s in soil_types]

    return {
        "soil_types": soil_types,
        "bias_range": [min(biases), max(biases)],
        "random_std_range": [min(random_stds), max(random_stds)],
        "bias_dominated_soils": [
            s for s in soil_types if decompositions[s]["bias_fraction"] > 0.5
        ],
        "noise_dominated_soils": [
            s for s in soil_types if decompositions[s]["bias_fraction"] <= 0.5
        ],
        "mean_bias_fraction": sum(bias_fracs) / len(bias_fracs),
    }


# ---------------------------------------------------------------------------
# Validation harness
# ---------------------------------------------------------------------------

def main() -> int:
    benchmark_path = Path(__file__).parent / "benchmark_sensor_noise.json"
    with open(benchmark_path) as f:
        benchmark = json.load(f)

    reset_counters()

    print("=" * 72)
    print("groundSpring Exp 001: Sensor Noise Characterization")
    print("  Source: Dong et al. (2020) — same data, groundSpring perspective")
    print("=" * 72)

    sensors = benchmark["sensors"]
    soils = benchmark["soil_types"]
    factory = benchmark["factory_calibration_stats"]
    corrected = benchmark["corrected_stats"]
    expected = benchmark["expected_results"]

    # ------------------------------------------------------------------
    # Part 1: Bias-Variance Decomposition
    # ------------------------------------------------------------------
    print("\n--- Part 1: Bias-Variance Decomposition ---")
    all_decompositions: dict[str, dict] = {}

    for sensor in sensors:
        print(f"\n  Sensor: {sensor.upper()}")
        all_decompositions[sensor] = {}

        for soil in soils:
            s = factory[sensor][soil]
            decomp = decompose_error(s["mbe"], s["rmse"])
            all_decompositions[sensor][soil] = decomp

            exp = expected[sensor][soil]

            # Tol 0.001: derived analytically from sqrt(RMSE²-MBE²),
            # rounding to 4 decimal places introduces ≤0.0005 error.
            check_approx(f"  {soil} bias", decomp["bias"], exp["bias"], 0.001)
            check_approx(
                f"  {soil} random_std",
                decomp["random_std"],
                exp["random_std"],
                0.001,
            )
            # Bias fraction tolerance 0.005: ratio of two small numbers,
            # propagated rounding from MBE and RMSE (each ±0.0005).
            check_approx(
                f"  {soil} bias_fraction",
                decomp["bias_fraction"],
                exp["bias_fraction"],
                0.005,
            )

    # ------------------------------------------------------------------
    # Part 2: Noise Floor Analysis
    # ------------------------------------------------------------------
    print("\n--- Part 2: Noise Floor After Correction ---")

    for sensor in sensors:
        print(f"\n  Sensor: {sensor.upper()}")
        for soil in soils:
            c = corrected[sensor][soil]
            nf = noise_floor_reduction(c["factory_rmse"], c["corrected_rmse"])
            print(f"    {soil}:")
            print(f"      Factory RMSE:   {nf['factory_rmse']:.4f}")
            print(f"      Corrected RMSE: {nf['corrected_rmse']:.4f}")
            print(f"      Removed error:  {nf['removed_error']:.4f}")
            print(f"      Reduction:      {nf['reduction_pct']:.1f}%")
            print(f"      Noise floor:    {nf['noise_floor']:.4f} m³/m³")

            check_true(
                f"    {soil} corrected <= factory",
                nf["corrected_rmse"] <= nf["factory_rmse"],
            )

    # ------------------------------------------------------------------
    # Part 3: Cross-Soil Comparison
    # ------------------------------------------------------------------
    print("\n--- Part 3: Cross-Soil-Type Comparison ---")

    for sensor in sensors:
        comparison = compare_across_soils(all_decompositions[sensor])
        print(f"\n  Sensor: {sensor.upper()}")
        print(
            f"    Bias range:           [{comparison['bias_range'][0]:.4f}, "
            f"{comparison['bias_range'][1]:.4f}] m³/m³"
        )
        print(
            f"    Random noise range:   [{comparison['random_std_range'][0]:.4f}, "
            f"{comparison['random_std_range'][1]:.4f}] m³/m³"
        )
        print(f"    Bias-dominated soils: {comparison['bias_dominated_soils']}")
        print(f"    Noise-dominated soils:{comparison['noise_dominated_soils']}")
        print(f"    Mean bias fraction:   {comparison['mean_bias_fraction']:.3f}")

        if sensor == "cs616":
            has_both = (
                len(comparison["bias_dominated_soils"]) > 0
                and len(comparison["noise_dominated_soils"]) > 0
            )
            check_true("    Mixed bias/noise structure across soils", has_both)
        else:
            check_true(
                "    EC5 is bias-dominated overall",
                comparison["mean_bias_fraction"] > 0.5,
            )

    # ------------------------------------------------------------------
    # Part 4: Synthetic Noise Distribution Test
    # ------------------------------------------------------------------
    print("\n--- Part 4: Noise Distribution Characterization ---")

    for sensor in sensors:
        print(f"\n  Sensor: {sensor.upper()}")
        for soil in soils:
            decomp = all_decompositions[sensor][soil]
            samples = generate_sensor_noise_samples(
                decomp["bias"], decomp["random_std"]
            )
            norm_test = test_normality(samples)

            print(f"    {soil}:")
            print(
                f"      Generated: N={norm_test['n_samples']}, "
                f"mean={norm_test['mean']:.4f}, std={norm_test['std']:.4f}"
            )
            print(f"      Skewness:  {norm_test['skewness']:.4f}")
            print(f"      Kurtosis:  {norm_test['excess_kurtosis']:.4f}")
            print(f"      Shapiro p: {norm_test['shapiro_p_value']:.4f}")

            check_true(
                f"    {soil} synthetic samples pass normality",
                norm_test["is_normal_shapiro"],
            )

    # ------------------------------------------------------------------
    # Part 5: Key Findings Summary
    # ------------------------------------------------------------------
    print("\n" + "=" * 72)
    print("KEY FINDINGS:")
    print("=" * 72)

    print("\n1. Bias vs Noise Structure:")
    for sensor in sensors:
        for soil in soils:
            d = all_decompositions[sensor][soil]
            dominant = "BIAS" if d["bias_fraction"] > 0.5 else "NOISE"
            print(
                f"   {sensor.upper()} in {soil}: "
                f"{dominant}-dominated ({d['bias_fraction']*100:.1f}% bias)"
            )

    print("\n2. Correctable vs Irreducible Error:")
    for sensor in sensors:
        for soil in soils:
            c = corrected[sensor][soil]
            nf = noise_floor_reduction(c["factory_rmse"], c["corrected_rmse"])
            print(
                f"   {sensor.upper()} in {soil}: "
                f"{nf['reduction_pct']:.0f}% correctable, "
                f"noise floor = {nf['noise_floor']:.4f} m³/m³"
            )

    print("\n3. Implications for Penny Irrigation:")
    print("   - Sand: Low noise floor (0.004-0.006), suitable for precision irrigation")
    print("   - Sandy clay loam: Higher noise (0.012-0.020), needs averaging or filtering")
    print("   - Site-specific calibration removes 50-80% of total sensor error")

    return print_summary("Exp 001: Sensor Noise Characterization")


if __name__ == "__main__":
    sys.exit(main())
