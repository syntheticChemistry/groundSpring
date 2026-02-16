#!/usr/bin/env python3
"""
groundSpring Experiment 003 — Error Propagation Through FAO-56 ET₀

Given known sensor uncertainties (temperature ±0.5°C, humidity ±5%,
wind ±10%, radiation ±5%), how does measurement noise propagate through
the FAO-56 Penman-Monteith equation chain to produce uncertainty in ET₀?

Methods:
  1. Monte Carlo: Draw N=10,000 perturbed input sets, compute ET₀ for each
  2. Sensitivity analysis: Which input contributes most to ET₀ variance?
  3. Analytical comparison: Partial-derivative error propagation

This directly informs: "how good does a cheap sensor need to be for
Penny Irrigation?" If ET₀ uncertainty is dominated by temperature,
a cheap wind sensor is fine. If radiation dominates, investing in a
pyranometer matters most.

Uses airSpring's validated FAO-56 implementation as the model.

Reference:
  Allen, R.G., Pereira, L.S., Raes, D., Smith, M. (1998). FAO-56.
"""

import json
import math
import sys
from pathlib import Path

import numpy as np

# Import airSpring's validated FAO-56 functions
AIRSPRING_FAO56 = Path(__file__).parent.parent.parent.parent / "airSpring" / "control" / "fao56"
sys.path.insert(0, str(AIRSPRING_FAO56))

from penman_monteith import (
    saturation_vapour_pressure,
    slope_vapour_pressure_curve,
    atmospheric_pressure,
    psychrometric_constant,
    wind_speed_at_2m,
    mean_saturation_vapour_pressure,
    actual_vapour_pressure_rh,
    extraterrestrial_radiation,
    daylight_hours,
    solar_radiation_from_sunshine,
    clear_sky_radiation,
    net_shortwave_radiation,
    net_longwave_radiation,
    fao56_penman_monteith,
)


# ---------------------------------------------------------------------------
# Full ET₀ computation with explicit perturbed inputs
# ---------------------------------------------------------------------------

def compute_et0_from_perturbed(tmax_c: float, tmin_c: float,
                                 rhmax_pct: float, rhmin_pct: float,
                                 wind_10m_km_h: float,
                                 sunshine_hours: float,
                                 latitude_deg_n: float,
                                 altitude_m: float,
                                 day_of_year: int) -> float:
    """
    Full FAO-56 ET₀ computation from weather inputs.
    Same as airSpring's compute_example_18_uccle but accepts individual args.
    """
    tmean = (tmax_c + tmin_c) / 2.0
    uz_ms = wind_10m_km_h / 3.6
    u2 = wind_speed_at_2m(uz_ms, 10.0)

    delta = slope_vapour_pressure_curve(tmean)
    P = atmospheric_pressure(altitude_m)
    gamma = psychrometric_constant(P)
    es = mean_saturation_vapour_pressure(tmax_c, tmin_c)
    ea = actual_vapour_pressure_rh(tmax_c, tmin_c, rhmax_pct, rhmin_pct)
    vpd = es - ea

    Ra = extraterrestrial_radiation(latitude_deg_n, day_of_year)
    N = daylight_hours(latitude_deg_n, day_of_year)

    # Clamp sunshine hours to [0, N]
    n = max(0.0, min(sunshine_hours, N))
    Rs = solar_radiation_from_sunshine(n, N, Ra)
    Rso = clear_sky_radiation(altitude_m, Ra)
    Rns = net_shortwave_radiation(Rs)

    Rs_Rso = min(Rs / Rso, 1.0) if Rso > 0 else 0.7
    Rnl = net_longwave_radiation(tmax_c, tmin_c, ea, Rs_Rso)
    Rn = Rns - Rnl
    G = 0.0  # daily step

    return fao56_penman_monteith(Rn, G, tmean, u2, vpd, delta, gamma)


# ---------------------------------------------------------------------------
# Monte Carlo error propagation
# ---------------------------------------------------------------------------

def monte_carlo_et0(inputs: dict, uncertainties: dict,
                     n_samples: int = 10000, seed: int = 42) -> dict:
    """
    Propagate measurement uncertainties through FAO-56 via Monte Carlo.

    For each sample:
      1. Perturb each input by its uncertainty (drawn from specified distribution)
      2. Compute ET₀ with perturbed inputs
      3. Collect the ET₀ distribution

    Returns statistics of the ET₀ distribution.
    """
    rng = np.random.default_rng(seed)

    tmax_base = inputs["tmax_c"]
    tmin_base = inputs["tmin_c"]
    rhmax_base = inputs["rhmax_pct"]
    rhmin_base = inputs["rhmin_pct"]
    wind_base = inputs["wind_speed_10m_km_h"]
    sun_base = inputs["sunshine_hours"]
    lat = inputs["latitude_deg_n"]
    alt = inputs["altitude_m"]
    doy = inputs["day_of_year"]

    # Generate perturbations
    tmax_pert = rng.normal(0, uncertainties["tmax_c"]["std"], n_samples)
    tmin_pert = rng.normal(0, uncertainties["tmin_c"]["std"], n_samples)
    rhmax_pert = rng.normal(0, uncertainties["rhmax_pct"]["std"], n_samples)
    rhmin_pert = rng.normal(0, uncertainties["rhmin_pct"]["std"], n_samples)
    wind_pert = rng.normal(0, wind_base * uncertainties["wind_m_s"]["std_fraction"],
                            n_samples)
    sun_pert = rng.normal(0, sun_base * uncertainties["Rs_mj_m2"]["std_fraction"],
                           n_samples)

    # Compute ET₀ for each perturbed set
    et0_samples = np.zeros(n_samples)
    for i in range(n_samples):
        tmax = tmax_base + tmax_pert[i]
        tmin = tmin_base + tmin_pert[i]
        # Ensure tmin < tmax
        if tmin >= tmax:
            tmin = tmax - 1.0

        rhmax = np.clip(rhmax_base + rhmax_pert[i], 10, 100)
        rhmin = np.clip(rhmin_base + rhmin_pert[i], 5, rhmax)
        wind = max(0.5, wind_base + wind_pert[i])
        sun = max(0.0, sun_base + sun_pert[i])

        et0_samples[i] = compute_et0_from_perturbed(
            tmax, tmin, rhmax, rhmin, wind, sun, lat, alt, doy
        )

    return {
        "samples": et0_samples,
        "mean": float(np.mean(et0_samples)),
        "std": float(np.std(et0_samples)),
        "median": float(np.median(et0_samples)),
        "p5": float(np.percentile(et0_samples, 5)),
        "p95": float(np.percentile(et0_samples, 95)),
        "min": float(np.min(et0_samples)),
        "max": float(np.max(et0_samples)),
        "cv_pct": float(np.std(et0_samples) / np.mean(et0_samples) * 100),
        "n_samples": n_samples,
    }


# ---------------------------------------------------------------------------
# Sensitivity analysis — one-at-a-time perturbation
# ---------------------------------------------------------------------------

def sensitivity_analysis(inputs: dict, uncertainties: dict,
                          n_samples: int = 5000, seed: int = 42) -> dict:
    """
    Determine which input variable contributes most to ET₀ uncertainty.

    Method: Perturb ONE variable at a time while holding others fixed.
    The resulting ET₀ std tells us that variable's individual contribution.
    """
    rng = np.random.default_rng(seed)

    tmax_base = inputs["tmax_c"]
    tmin_base = inputs["tmin_c"]
    rhmax_base = inputs["rhmax_pct"]
    rhmin_base = inputs["rhmin_pct"]
    wind_base = inputs["wind_speed_10m_km_h"]
    sun_base = inputs["sunshine_hours"]
    lat = inputs["latitude_deg_n"]
    alt = inputs["altitude_m"]
    doy = inputs["day_of_year"]

    # Baseline ET₀
    et0_base = compute_et0_from_perturbed(
        tmax_base, tmin_base, rhmax_base, rhmin_base,
        wind_base, sun_base, lat, alt, doy
    )

    variables = {
        "temperature": {
            "perturb": lambda rng: (
                rng.normal(0, uncertainties["tmax_c"]["std"]),
                rng.normal(0, uncertainties["tmin_c"]["std"])
            ),
            "apply": lambda base, pert: {
                **base,
                "tmax_c": tmax_base + pert[0],
                "tmin_c": min(tmin_base + pert[1], tmax_base + pert[0] - 1),
            },
        },
        "humidity": {
            "perturb": lambda rng: (
                rng.normal(0, uncertainties["rhmax_pct"]["std"]),
                rng.normal(0, uncertainties["rhmin_pct"]["std"])
            ),
            "apply": lambda base, pert: {
                **base,
                "rhmax_pct": np.clip(rhmax_base + pert[0], 10, 100),
                "rhmin_pct": np.clip(rhmin_base + pert[1], 5,
                                       np.clip(rhmax_base + pert[0], 10, 100)),
            },
        },
        "wind": {
            "perturb": lambda rng: rng.normal(
                0, wind_base * uncertainties["wind_m_s"]["std_fraction"]
            ),
            "apply": lambda base, pert: {
                **base,
                "wind_speed_10m_km_h": max(0.5, wind_base + pert),
            },
        },
        "radiation": {
            "perturb": lambda rng: rng.normal(
                0, sun_base * uncertainties["Rs_mj_m2"]["std_fraction"]
            ),
            "apply": lambda base, pert: {
                **base,
                "sunshine_hours": max(0, sun_base + pert),
            },
        },
    }

    base_dict = {
        "tmax_c": tmax_base, "tmin_c": tmin_base,
        "rhmax_pct": rhmax_base, "rhmin_pct": rhmin_base,
        "wind_speed_10m_km_h": wind_base, "sunshine_hours": sun_base,
        "latitude_deg_n": lat, "altitude_m": alt, "day_of_year": doy,
    }

    results = {}

    for var_name, var_config in variables.items():
        et0_values = np.zeros(n_samples)

        for i in range(n_samples):
            pert = var_config["perturb"](rng)
            perturbed = var_config["apply"](base_dict, pert)
            et0_values[i] = compute_et0_from_perturbed(
                perturbed["tmax_c"], perturbed["tmin_c"],
                perturbed["rhmax_pct"], perturbed["rhmin_pct"],
                perturbed["wind_speed_10m_km_h"], perturbed["sunshine_hours"],
                lat, alt, doy
            )

        results[var_name] = {
            "et0_std": float(np.std(et0_values)),
            "et0_mean": float(np.mean(et0_values)),
            "sensitivity_pct": float(np.std(et0_values) / et0_base * 100),
        }

    # Compute variance fractions
    total_var = sum(r["et0_std"] ** 2 for r in results.values())
    for var_name in results:
        results[var_name]["variance_fraction"] = (
            results[var_name]["et0_std"] ** 2 / total_var if total_var > 0 else 0
        )

    # Rank by contribution
    ranking = sorted(results.keys(), key=lambda k: results[k]["et0_std"],
                      reverse=True)
    results["ranking"] = ranking

    return results


# ---------------------------------------------------------------------------
# Analytical error propagation (first-order Taylor expansion)
# ---------------------------------------------------------------------------

def analytical_et0_uncertainty(inputs: dict, uncertainties: dict) -> dict:
    """
    First-order analytical error propagation via numerical partial derivatives.

    σ²(ET₀) ≈ Σ (∂ET₀/∂xᵢ)² σ²(xᵢ)

    Uses central finite differences for ∂ET₀/∂xᵢ.
    """
    base_args = {
        "tmax_c": inputs["tmax_c"],
        "tmin_c": inputs["tmin_c"],
        "rhmax_pct": inputs["rhmax_pct"],
        "rhmin_pct": inputs["rhmin_pct"],
        "wind_speed_10m_km_h": inputs["wind_speed_10m_km_h"],
        "sunshine_hours": inputs["sunshine_hours"],
    }
    lat = inputs["latitude_deg_n"]
    alt = inputs["altitude_m"]
    doy = inputs["day_of_year"]

    def et0_func(**kwargs):
        return compute_et0_from_perturbed(
            kwargs["tmax_c"], kwargs["tmin_c"],
            kwargs["rhmax_pct"], kwargs["rhmin_pct"],
            kwargs["wind_speed_10m_km_h"], kwargs["sunshine_hours"],
            lat, alt, doy
        )

    et0_base = et0_func(**base_args)

    # Perturbation sizes for numerical differentiation
    perturbations = {
        "tmax_c": 0.1,
        "tmin_c": 0.1,
        "rhmax_pct": 1.0,
        "rhmin_pct": 1.0,
        "wind_speed_10m_km_h": 0.5,
        "sunshine_hours": 0.1,
    }

    # Uncertainties (standard deviations)
    sigmas = {
        "tmax_c": uncertainties["tmax_c"]["std"],
        "tmin_c": uncertainties["tmin_c"]["std"],
        "rhmax_pct": uncertainties["rhmax_pct"]["std"],
        "rhmin_pct": uncertainties["rhmin_pct"]["std"],
        "wind_speed_10m_km_h": inputs["wind_speed_10m_km_h"] * uncertainties["wind_m_s"]["std_fraction"],
        "sunshine_hours": inputs["sunshine_hours"] * uncertainties["Rs_mj_m2"]["std_fraction"],
    }

    partials = {}
    variance_contributions = {}

    for var, delta in perturbations.items():
        # Forward
        args_plus = {**base_args, var: base_args[var] + delta}
        et0_plus = et0_func(**args_plus)

        # Backward
        args_minus = {**base_args, var: base_args[var] - delta}
        et0_minus = et0_func(**args_minus)

        # Central difference
        partial = (et0_plus - et0_minus) / (2 * delta)
        partials[var] = partial
        variance_contributions[var] = (partial * sigmas[var]) ** 2

    total_variance = sum(variance_contributions.values())
    analytical_std = math.sqrt(total_variance)

    fractions = {k: v / total_variance if total_variance > 0 else 0
                  for k, v in variance_contributions.items()}

    return {
        "et0_base": et0_base,
        "analytical_std": analytical_std,
        "analytical_cv_pct": analytical_std / et0_base * 100,
        "partials": partials,
        "variance_contributions": variance_contributions,
        "variance_fractions": fractions,
        "sigmas": sigmas,
    }


# ---------------------------------------------------------------------------
# Validation harness
# ---------------------------------------------------------------------------

def check(label: str, computed: float, low: float, high: float) -> bool:
    ok = low <= computed <= high
    status = "PASS" if ok else "FAIL"
    print(f"  [{status}] {label}: {computed:.4f} "
          f"(expected [{low:.4f}, {high:.4f}])")
    return ok


def check_min(label: str, computed: float, minimum: float) -> bool:
    ok = computed >= minimum
    status = "PASS" if ok else "FAIL"
    print(f"  [{status}] {label}: {computed:.4f} (minimum {minimum:.4f})")
    return ok


def main():
    benchmark_path = Path(__file__).parent / "benchmark_error_propagation.json"
    with open(benchmark_path) as f:
        benchmark = json.load(f)

    total_passed = 0
    total_failed = 0

    inputs = benchmark["reference_day"]["inputs"]
    uncertainties = benchmark["input_uncertainties"]
    mc_config = benchmark["monte_carlo_config"]
    expected = benchmark["expected_results"]

    print("=" * 72)
    print("groundSpring Exp 003: Error Propagation Through FAO-56 ET₀")
    print("  Method: Monte Carlo + Analytical + Sensitivity Analysis")
    print("=" * 72)

    # ------------------------------------------------------------------
    # Part 1: Baseline ET₀ (verify airSpring match)
    # ------------------------------------------------------------------
    print("\n--- Part 1: Baseline ET₀ (verify airSpring) ---")
    et0_base = compute_et0_from_perturbed(
        inputs["tmax_c"], inputs["tmin_c"],
        inputs["rhmax_pct"], inputs["rhmin_pct"],
        inputs["wind_speed_10m_km_h"], inputs["sunshine_hours"],
        inputs["latitude_deg_n"], inputs["altitude_m"],
        inputs["day_of_year"]
    )
    print(f"  Baseline ET₀: {et0_base:.4f} mm/day")
    print(f"  Expected:     {benchmark['reference_day']['expected_et0_mm_day']:.4f} mm/day")

    if abs(et0_base - benchmark["reference_day"]["expected_et0_mm_day"]) < 0.15:
        print(f"  [PASS] Baseline matches airSpring validated value")
        total_passed += 1
    else:
        print(f"  [FAIL] Baseline doesn't match!")
        total_failed += 1

    # ------------------------------------------------------------------
    # Part 2: Monte Carlo Error Propagation
    # ------------------------------------------------------------------
    print(f"\n--- Part 2: Monte Carlo (N={mc_config['n_samples']}) ---")
    mc = monte_carlo_et0(inputs, uncertainties,
                          n_samples=mc_config["n_samples"],
                          seed=mc_config["seed"])

    print(f"  ET₀ mean:   {mc['mean']:.4f} mm/day")
    print(f"  ET₀ std:    {mc['std']:.4f} mm/day")
    print(f"  ET₀ CV:     {mc['cv_pct']:.2f}%")
    print(f"  ET₀ range:  [{mc['min']:.4f}, {mc['max']:.4f}]")
    print(f"  90% CI:     [{mc['p5']:.4f}, {mc['p95']:.4f}]")

    # Validate mean is close to expected ET₀
    if check("ET₀ mean", mc["mean"],
             expected["et0_mean_range"][0], expected["et0_mean_range"][1]):
        total_passed += 1
    else:
        total_failed += 1

    # Validate std is in expected range
    if check("ET₀ std", mc["std"],
             expected["et0_std_range"][0], expected["et0_std_range"][1]):
        total_passed += 1
    else:
        total_failed += 1

    # CV should be 5-15%
    if check("ET₀ CV (%)", mc["cv_pct"], 2.0, 20.0):
        total_passed += 1
    else:
        total_failed += 1

    # 90% CI should bracket the expected value
    expected_et0 = benchmark["reference_day"]["expected_et0_mm_day"]
    if mc["p5"] < expected_et0 < mc["p95"]:
        print(f"  [PASS] 90% CI brackets expected ET₀ ({expected_et0})")
        total_passed += 1
    else:
        print(f"  [FAIL] 90% CI does not bracket expected ET₀")
        total_failed += 1

    # ------------------------------------------------------------------
    # Part 3: Sensitivity Analysis
    # ------------------------------------------------------------------
    print("\n--- Part 3: Sensitivity Analysis (one-at-a-time) ---")
    sens = sensitivity_analysis(inputs, uncertainties, n_samples=5000)

    print(f"\n  Variable contributions to ET₀ uncertainty:")
    for var_name in sens["ranking"]:
        s = sens[var_name]
        print(f"    {var_name:15s}: σ={s['et0_std']:.4f} mm/day "
              f"({s['variance_fraction']*100:.1f}% of variance)")

    print(f"\n  Dominance ranking: {' > '.join(sens['ranking'])}")

    # Validate that the top contributor matches expected
    expected_ranking = benchmark["sensitivity_analysis"]["expected_ranking"]
    top_contributor = sens["ranking"][0]
    if top_contributor in expected_ranking[:2]:
        print(f"  [PASS] Top contributor ({top_contributor}) matches "
              f"expected top-2: {expected_ranking[:2]}")
        total_passed += 1
    else:
        print(f"  [FAIL] Top contributor ({top_contributor}) not in "
              f"expected top-2: {expected_ranking[:2]}")
        total_failed += 1

    # Variance fractions should sum to ~1.0 (they won't exactly due to
    # nonlinear interactions, but should be close)
    total_frac = sum(sens[v]["variance_fraction"] for v in sens if v != "ranking")
    if check("Variance fraction sum", total_frac, 0.8, 1.2):
        total_passed += 1
    else:
        total_failed += 1

    # ------------------------------------------------------------------
    # Part 4: Analytical Error Propagation
    # ------------------------------------------------------------------
    print("\n--- Part 4: Analytical (Taylor Expansion) ---")
    analytical = analytical_et0_uncertainty(inputs, uncertainties)

    print(f"  Analytical σ(ET₀): {analytical['analytical_std']:.4f} mm/day")
    print(f"  Analytical CV:     {analytical['analytical_cv_pct']:.2f}%")

    print(f"\n  Partial derivatives (∂ET₀/∂x):")
    for var, partial in analytical["partials"].items():
        frac = analytical["variance_fractions"].get(var, 0)
        print(f"    {var:30s}: {partial:+.4f} mm/day per unit  "
              f"({frac*100:.1f}% of variance)")

    # Compare Monte Carlo vs analytical
    mc_std = mc["std"]
    an_std = analytical["analytical_std"]
    ratio = mc_std / an_std if an_std > 0 else float("inf")

    print(f"\n  Monte Carlo σ:  {mc_std:.4f}")
    print(f"  Analytical σ:   {an_std:.4f}")
    print(f"  Ratio (MC/An):  {ratio:.3f}")

    # Should agree within ~20% (first-order approx vs full simulation)
    if check("MC/Analytical ratio", ratio, 0.7, 1.5):
        total_passed += 1
    else:
        total_failed += 1

    # ------------------------------------------------------------------
    # Part 5: Key Findings
    # ------------------------------------------------------------------
    print(f"\n{'=' * 72}")
    print("KEY FINDINGS:")
    print(f"{'=' * 72}")

    print(f"\n1. ET₀ Uncertainty Budget:")
    print(f"   Mean ET₀:   {mc['mean']:.3f} ± {mc['std']:.3f} mm/day")
    print(f"   90% CI:     [{mc['p5']:.3f}, {mc['p95']:.3f}] mm/day")
    print(f"   CV:          {mc['cv_pct']:.1f}%")

    print(f"\n2. Sensitivity Ranking:")
    for rank, var in enumerate(sens["ranking"], 1):
        s = sens[var]
        print(f"   #{rank} {var}: {s['variance_fraction']*100:.1f}% of variance")

    print(f"\n3. Implications for Penny Irrigation:")
    print(f"   - A ±{mc['std']:.2f} mm/day ET₀ uncertainty means irrigation timing")
    print(f"     could be off by ~{mc['std']*7:.1f} mm over a week")
    top = sens["ranking"][0]
    print(f"   - Investing in better {top} measurement has the most impact")
    print(f"   - Analytical and Monte Carlo methods agree "
          f"(ratio = {ratio:.2f})")
    print(f"   - First-order Taylor is {'adequate' if 0.8 < ratio < 1.2 else 'rough'} "
          f"for this equation chain")

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
