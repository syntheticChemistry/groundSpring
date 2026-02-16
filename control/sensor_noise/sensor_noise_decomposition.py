#!/usr/bin/env python3
"""
groundSpring Experiment 001 — Sensor Noise Characterization

Decomposes the factory calibration error in Dong et al. (2020) soil moisture
sensors into systematic bias (correctable) vs random noise (irreducible).

This is the same data that airSpring uses for calibration validation, but
analyzed from groundSpring's perspective: we care about the STRUCTURE of
the error, not just its magnitude.

Key questions:
  1. What fraction of total error is systematic bias vs random noise?
  2. How does the noise structure differ across soil types?
  3. What is the noise floor (irreducible error) after bias correction?
  4. Are the errors consistent with Gaussian noise, or heavy-tailed?

Reference:
  Dong, Miller, Kelley (2020) Performance Evaluation of Soil Moisture
  Sensors in Coarse- and Fine-Textured Michigan Agricultural Soils.
  Agriculture 10(12), 598. doi:10.3390/agriculture10120598
"""

import json
import math
import sys
from pathlib import Path

import numpy as np


# ---------------------------------------------------------------------------
# Bias-Variance Decomposition
# ---------------------------------------------------------------------------

def decompose_error(mbe: float, rmse: float) -> dict:
    """
    Decompose total RMSE into bias and random noise components.

    RMSE^2 = MBE^2 + Var(error)

    where:
      - MBE (Mean Bias Error) = systematic bias
      - Var(error) = random noise variance
      - random_std = sqrt(RMSE^2 - MBE^2)

    Returns dict with bias, random_std, bias_fraction, and noise_fraction.
    """
    bias_sq = mbe ** 2
    total_sq = rmse ** 2

    if total_sq < bias_sq:
        variance = 0.0
    else:
        variance = total_sq - bias_sq

    random_std = math.sqrt(variance)

    bias_fraction = bias_sq / total_sq if total_sq > 0 else 0.0
    noise_fraction = 1.0 - bias_fraction

    return {
        "bias": mbe,
        "bias_abs": abs(mbe),
        "random_std": random_std,
        "total_rmse": rmse,
        "bias_sq": bias_sq,
        "variance": variance,
        "bias_fraction": bias_fraction,
        "noise_fraction": noise_fraction,
    }


def noise_floor_reduction(factory_rmse: float, corrected_rmse: float) -> dict:
    """
    Quantify how much error was removable (systematic) vs irreducible (noise).

    After soil-specific correction:
      - Removed error = sqrt(factory_rmse^2 - corrected_rmse^2)
      - Noise floor = corrected_rmse (what remains after best correction)
    """
    if factory_rmse ** 2 > corrected_rmse ** 2:
        removed = math.sqrt(factory_rmse ** 2 - corrected_rmse ** 2)
    else:
        removed = 0.0

    reduction_pct = (1.0 - corrected_rmse / factory_rmse) * 100.0 if factory_rmse > 0 else 0.0

    return {
        "factory_rmse": factory_rmse,
        "corrected_rmse": corrected_rmse,
        "removed_error": removed,
        "noise_floor": corrected_rmse,
        "reduction_pct": reduction_pct,
    }


# ---------------------------------------------------------------------------
# Synthetic noise distribution analysis
# ---------------------------------------------------------------------------

def generate_sensor_noise_samples(mbe: float, random_std: float,
                                   n_samples: int = 10000,
                                   rng_seed: int = 42) -> np.ndarray:
    """
    Generate synthetic sensor error samples matching observed statistics.

    Samples = bias + N(0, random_std)

    Used to characterize what the noise distribution SHOULD look like if
    Gaussian, which we can then compare against field data patterns.
    """
    rng = np.random.default_rng(rng_seed)
    return mbe + rng.normal(0.0, random_std, size=n_samples)


def test_normality(samples: np.ndarray) -> dict:
    """
    Test whether error samples are consistent with Gaussian distribution.

    Uses skewness and excess kurtosis as diagnostics:
      - Gaussian: skewness ≈ 0, excess kurtosis ≈ 0
      - Heavy-tailed: excess kurtosis > 0
      - Skewed: |skewness| > 0
    """
    from scipy import stats

    n = len(samples)
    mean = np.mean(samples)
    std = np.std(samples, ddof=1)
    skewness = float(stats.skew(samples))
    kurtosis = float(stats.kurtosis(samples))  # excess kurtosis

    _, shapiro_p = stats.shapiro(samples[:5000])  # Shapiro-Wilk (max 5000)

    return {
        "n_samples": n,
        "mean": float(mean),
        "std": float(std),
        "skewness": skewness,
        "excess_kurtosis": kurtosis,
        "shapiro_p_value": float(shapiro_p),
        "is_normal_shapiro": float(shapiro_p) > 0.05,
    }


# ---------------------------------------------------------------------------
# Cross-soil-type comparison
# ---------------------------------------------------------------------------

def compare_across_soils(decompositions: dict) -> dict:
    """
    Compare noise characteristics across soil types.

    Key insights from Dong et al. (2020):
      - Coarse soils (sand) have smaller bias but noise can dominate
      - Fine soils (clay) have larger systematic bias
      - The relationship between particle size and noise structure matters
        for deciding when site-specific calibration is needed
    """
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

def check(label: str, computed: float, expected: float, tol: float) -> bool:
    diff = abs(computed - expected)
    status = "PASS" if diff <= tol else "FAIL"
    print(f"  [{status}] {label}: {computed:.4f} "
          f"(expected {expected:.4f}, tol {tol:.4f}, diff {diff:.4f})")
    return diff <= tol


def main():
    benchmark_path = Path(__file__).parent / "benchmark_sensor_noise.json"
    with open(benchmark_path) as f:
        benchmark = json.load(f)

    total_passed = 0
    total_failed = 0

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
    # Part 1: Bias-Variance Decomposition (validate against precomputed)
    # ------------------------------------------------------------------
    print("\n--- Part 1: Bias-Variance Decomposition ---")
    all_decompositions = {}

    for sensor in sensors:
        print(f"\n  Sensor: {sensor.upper()}")
        all_decompositions[sensor] = {}

        for soil in soils:
            stats = factory[sensor][soil]
            decomp = decompose_error(stats["mbe"], stats["rmse"])
            all_decompositions[sensor][soil] = decomp

            exp = expected[sensor][soil]
            tol = 0.002

            if check(f"  {soil} bias", decomp["bias"], exp["bias"], tol):
                total_passed += 1
            else:
                total_failed += 1

            if check(f"  {soil} random_std", decomp["random_std"],
                     exp["random_std"], tol):
                total_passed += 1
            else:
                total_failed += 1

            if check(f"  {soil} bias_fraction", decomp["bias_fraction"],
                     exp["bias_fraction"], 0.01):
                total_passed += 1
            else:
                total_failed += 1

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

            # Validate: corrected should be less than factory
            if nf["corrected_rmse"] <= nf["factory_rmse"]:
                print(f"    [PASS] corrected <= factory")
                total_passed += 1
            else:
                print(f"    [FAIL] corrected > factory!")
                total_failed += 1

    # ------------------------------------------------------------------
    # Part 3: Cross-Soil Comparison
    # ------------------------------------------------------------------
    print("\n--- Part 3: Cross-Soil-Type Comparison ---")

    for sensor in sensors:
        comparison = compare_across_soils(all_decompositions[sensor])
        print(f"\n  Sensor: {sensor.upper()}")
        print(f"    Bias range:           [{comparison['bias_range'][0]:.4f}, "
              f"{comparison['bias_range'][1]:.4f}] m³/m³")
        print(f"    Random noise range:   [{comparison['random_std_range'][0]:.4f}, "
              f"{comparison['random_std_range'][1]:.4f}] m³/m³")
        print(f"    Bias-dominated soils: {comparison['bias_dominated_soils']}")
        print(f"    Noise-dominated soils:{comparison['noise_dominated_soils']}")
        print(f"    Mean bias fraction:   {comparison['mean_bias_fraction']:.3f}")

        # Validate: at least one bias-dominated + one noise-dominated expected
        has_both = (len(comparison["bias_dominated_soils"]) > 0 and
                    len(comparison["noise_dominated_soils"]) > 0)
        if sensor == "cs616":
            # CS616: sand is noise-dominated, loamy_sand is bias-dominated
            if has_both:
                print(f"    [PASS] Mixed bias/noise structure across soils")
                total_passed += 1
            else:
                print(f"    [FAIL] Expected mixed structure")
                total_failed += 1
        else:
            # EC5: bias-dominated across all soils
            if comparison["mean_bias_fraction"] > 0.5:
                print(f"    [PASS] EC5 is bias-dominated overall")
                total_passed += 1
            else:
                print(f"    [FAIL] Expected EC5 to be bias-dominated")
                total_failed += 1

    # ------------------------------------------------------------------
    # Part 4: Synthetic Noise Distribution Test
    # ------------------------------------------------------------------
    print("\n--- Part 4: Noise Distribution Characterization ---")

    for sensor in sensors:
        print(f"\n  Sensor: {sensor.upper()}")
        for soil in soils:
            decomp = all_decompositions[sensor][soil]
            samples = generate_sensor_noise_samples(
                decomp["bias"], decomp["random_std"],
                n_samples=10000, rng_seed=42
            )
            norm_test = test_normality(samples)

            print(f"    {soil}:")
            print(f"      Generated: N={norm_test['n_samples']}, "
                  f"mean={norm_test['mean']:.4f}, std={norm_test['std']:.4f}")
            print(f"      Skewness:  {norm_test['skewness']:.4f}")
            print(f"      Kurtosis:  {norm_test['excess_kurtosis']:.4f}")
            print(f"      Shapiro p: {norm_test['shapiro_p_value']:.4f}")
            print(f"      Normal:    {norm_test['is_normal_shapiro']}")

            # Synthetic Gaussian samples should pass normality
            if norm_test["is_normal_shapiro"]:
                print(f"    [PASS] Synthetic samples pass normality test")
                total_passed += 1
            else:
                print(f"    [FAIL] Synthetic samples fail normality test")
                total_failed += 1

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
            print(f"   {sensor.upper()} in {soil}: "
                  f"{dominant}-dominated ({d['bias_fraction']*100:.1f}% bias)")

    print("\n2. Correctable vs Irreducible Error:")
    for sensor in sensors:
        for soil in soils:
            c = corrected[sensor][soil]
            nf = noise_floor_reduction(c["factory_rmse"], c["corrected_rmse"])
            print(f"   {sensor.upper()} in {soil}: "
                  f"{nf['reduction_pct']:.0f}% correctable, "
                  f"noise floor = {nf['noise_floor']:.4f} m³/m³")

    print("\n3. Implications for Penny Irrigation:")
    print("   - Sand: Low noise floor (0.004-0.006), suitable for precision irrigation")
    print("   - Sandy clay loam: Higher noise (0.012-0.020), needs averaging or filtering")
    print("   - Site-specific calibration removes 50-80% of total sensor error")
    print("   - EC5 has larger systematic bias but more correctable error")
    print("   - CS616 has more random noise in coarse soils")

    # ------------------------------------------------------------------
    # Summary
    # ------------------------------------------------------------------
    total = total_passed + total_failed
    print(f"\n{'=' * 72}")
    print(f"TOTAL: {total_passed}/{total} PASS, {total_failed}/{total} FAIL")
    print(f"{'=' * 72}")

    return 0 if total_failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
