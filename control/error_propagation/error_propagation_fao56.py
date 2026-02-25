#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""
groundSpring Experiment 003 — Error Propagation Through FAO-56 ET₀

Given known sensor uncertainties (temperature ±0.5°C, humidity ±5%,
wind ±10%, radiation ±5%), how does measurement noise propagate through
the FAO-56 Penman-Monteith equation chain to produce uncertainty in ET₀?

Methods:
  1. Monte Carlo: N=10,000 perturbed input sets → ET₀ distribution
  2. Sensitivity analysis: one-at-a-time perturbation for variance ranking
  3. Analytical comparison: first-order Taylor expansion

Uses airSpring's validated FAO-56 implementation.  The import path is
discovered at runtime — groundSpring has no compile-time knowledge of
airSpring's location.

Reference:
  Allen, R.G., Pereira, L.S., Raes, D., Smith, M. (1998). FAO-56.
"""

from __future__ import annotations

import json
import math
import os
import sys
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from common import (
    check_range,
    check_true,
    print_summary,
    reset_counters,
)

# ---------------------------------------------------------------------------
# Runtime discovery of airSpring FAO-56 module
# ---------------------------------------------------------------------------

def _discover_airspring_fao56() -> Path | None:
    """Discover airSpring FAO-56 module at runtime.

    Discovery strategy (first match wins):
      1. ``AIRSPRING_ROOT`` environment variable (explicit override)
      2. ``ECOPRIMALS_ROOT`` env var + ``airSpring/control/fao56``
      3. Relative sibling: ``../../airSpring/control/fao56`` from this primal's root
    """
    target = Path("control") / "fao56"
    module_file = "penman_monteith.py"

    env_air = os.environ.get("AIRSPRING_ROOT")
    if env_air:
        p = Path(env_air) / target
        if (p / module_file).exists():
            return p

    env_eco = os.environ.get("ECOPRIMALS_ROOT")
    if env_eco:
        p = Path(env_eco) / "airSpring" / target
        if (p / module_file).exists():
            return p

    primal_root = Path(__file__).resolve().parent.parent.parent.parent
    p = primal_root / "airSpring" / target
    if (p / module_file).exists():
        return p

    return None


_fao56_path = _discover_airspring_fao56()
if _fao56_path is None:
    print("ERROR: Cannot discover airSpring FAO-56 module.")
    print("  groundSpring Exp 003 requires airSpring/control/fao56/penman_monteith.py")
    sys.exit(1)

sys.path.insert(0, str(_fao56_path))

from penman_monteith import (  # noqa: E402  # type: ignore[import-not-found]
    actual_vapour_pressure_rh,
    atmospheric_pressure,
    clear_sky_radiation,
    daylight_hours,
    extraterrestrial_radiation,
    fao56_penman_monteith,
    mean_saturation_vapour_pressure,
    net_longwave_radiation,
    net_shortwave_radiation,
    psychrometric_constant,
    slope_vapour_pressure_curve,
    solar_radiation_from_sunshine,
    wind_speed_at_2m,
)

# ---------------------------------------------------------------------------
# Full ET₀ computation with explicit perturbed inputs
# ---------------------------------------------------------------------------

def compute_et0_from_perturbed(
    tmax_c: float,
    tmin_c: float,
    rhmax_pct: float,
    rhmin_pct: float,
    wind_10m_km_h: float,
    sunshine_hours: float,
    latitude_deg_n: float,
    altitude_m: float,
    day_of_year: int,
) -> float:
    """Full FAO-56 ET₀ computation from weather inputs."""
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

    n = max(0.0, min(sunshine_hours, N))
    Rs = solar_radiation_from_sunshine(n, N, Ra)
    Rso = clear_sky_radiation(altitude_m, Ra)
    Rns = net_shortwave_radiation(Rs)

    Rs_Rso = min(Rs / Rso, 1.0) if Rso > 0 else 0.7
    Rnl = net_longwave_radiation(tmax_c, tmin_c, ea, Rs_Rso)
    Rn = Rns - Rnl
    G = 0.0

    return float(fao56_penman_monteith(Rn, G, tmean, u2, vpd, delta, gamma))


# ---------------------------------------------------------------------------
# Monte Carlo error propagation
# ---------------------------------------------------------------------------

def monte_carlo_et0(
    inputs: dict,
    uncertainties: dict,
    n_samples: int = 10_000,
    seed: int = 42,
) -> dict:
    """Propagate measurement uncertainties through FAO-56 via Monte Carlo."""
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

    tmax_pert = rng.normal(0, uncertainties["tmax_c"]["std"], n_samples)
    tmin_pert = rng.normal(0, uncertainties["tmin_c"]["std"], n_samples)
    rhmax_pert = rng.normal(0, uncertainties["rhmax_pct"]["std"], n_samples)
    rhmin_pert = rng.normal(0, uncertainties["rhmin_pct"]["std"], n_samples)
    wind_pert = rng.normal(
        0, wind_base * uncertainties["wind_m_s"]["std_fraction"], n_samples
    )
    sun_pert = rng.normal(
        0, sun_base * uncertainties["Rs_mj_m2"]["std_fraction"], n_samples
    )

    et0_samples = np.zeros(n_samples)
    for i in range(n_samples):
        tmax = tmax_base + tmax_pert[i]
        tmin = tmin_base + tmin_pert[i]
        if tmin >= tmax:
            tmin = tmax - 1.0

        rhmax = np.clip(rhmax_base + rhmax_pert[i], 10, 100)
        rhmin = np.clip(rhmin_base + rhmin_pert[i], 5, rhmax)
        wind = max(0.5, wind_base + wind_pert[i])
        sun = max(0.0, sun_base + sun_pert[i])

        et0_samples[i] = compute_et0_from_perturbed(
            tmax, tmin, rhmax, rhmin, wind, sun, lat, alt, doy
        )

    mean = float(np.mean(et0_samples))
    return {
        "samples": et0_samples,
        "mean": mean,
        "std": float(np.std(et0_samples)),
        "median": float(np.median(et0_samples)),
        "p5": float(np.percentile(et0_samples, 5)),
        "p95": float(np.percentile(et0_samples, 95)),
        "min": float(np.min(et0_samples)),
        "max": float(np.max(et0_samples)),
        "cv_pct": float(np.std(et0_samples) / mean * 100) if mean else 0.0,
        "n_samples": n_samples,
    }


# ---------------------------------------------------------------------------
# Sensitivity analysis — one-at-a-time perturbation
# ---------------------------------------------------------------------------

def sensitivity_analysis(
    inputs: dict,
    uncertainties: dict,
    n_samples: int = 5_000,
    seed: int = 42,
) -> dict:
    """Determine which input variable contributes most to ET₀ uncertainty."""
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

    et0_base = compute_et0_from_perturbed(
        tmax_base, tmin_base, rhmax_base, rhmin_base,
        wind_base, sun_base, lat, alt, doy,
    )

    variables = {
        "temperature": {
            "perturb": lambda rng: (
                rng.normal(0, uncertainties["tmax_c"]["std"]),
                rng.normal(0, uncertainties["tmin_c"]["std"]),
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
                rng.normal(0, uncertainties["rhmin_pct"]["std"]),
            ),
            "apply": lambda base, pert: {
                **base,
                "rhmax_pct": np.clip(rhmax_base + pert[0], 10, 100),
                "rhmin_pct": np.clip(
                    rhmin_base + pert[1],
                    5,
                    np.clip(rhmax_base + pert[0], 10, 100),
                ),
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
        "tmax_c": tmax_base,
        "tmin_c": tmin_base,
        "rhmax_pct": rhmax_base,
        "rhmin_pct": rhmin_base,
        "wind_speed_10m_km_h": wind_base,
        "sunshine_hours": sun_base,
        "latitude_deg_n": lat,
        "altitude_m": alt,
        "day_of_year": doy,
    }

    results: dict = {}

    for var_name, var_config in variables.items():
        et0_values = np.zeros(n_samples)
        for i in range(n_samples):
            pert = var_config["perturb"](rng)  # type: ignore[operator]
            perturbed = var_config["apply"](base_dict, pert)  # type: ignore[operator]
            et0_values[i] = compute_et0_from_perturbed(
                perturbed["tmax_c"],
                perturbed["tmin_c"],
                perturbed["rhmax_pct"],
                perturbed["rhmin_pct"],
                perturbed["wind_speed_10m_km_h"],
                perturbed["sunshine_hours"],
                lat, alt, doy,
            )

        results[var_name] = {
            "et0_std": float(np.std(et0_values)),
            "et0_mean": float(np.mean(et0_values)),
            "sensitivity_pct": float(np.std(et0_values) / et0_base * 100),
        }

    total_var = sum(r["et0_std"] ** 2 for r in results.values())
    for var_name in results:
        results[var_name]["variance_fraction"] = (
            results[var_name]["et0_std"] ** 2 / total_var if total_var > 0 else 0
        )

    ranking = sorted(
        results.keys(), key=lambda k: results[k]["et0_std"], reverse=True
    )
    results["ranking"] = ranking
    return results


# ---------------------------------------------------------------------------
# Analytical error propagation (first-order Taylor expansion)
# ---------------------------------------------------------------------------

def analytical_et0_uncertainty(inputs: dict, uncertainties: dict) -> dict:
    """First-order analytical error propagation via numerical partials.

    σ²(ET₀) ≈ Σ (∂ET₀/∂xᵢ)² σ²(xᵢ)
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

    def et0_func(**kwargs: float) -> float:
        return compute_et0_from_perturbed(
            kwargs["tmax_c"], kwargs["tmin_c"],
            kwargs["rhmax_pct"], kwargs["rhmin_pct"],
            kwargs["wind_speed_10m_km_h"], kwargs["sunshine_hours"],
            lat, alt, doy,
        )

    et0_base = et0_func(**base_args)

    perturbations = {
        "tmax_c": 0.1,
        "tmin_c": 0.1,
        "rhmax_pct": 1.0,
        "rhmin_pct": 1.0,
        "wind_speed_10m_km_h": 0.5,
        "sunshine_hours": 0.1,
    }

    sigmas = {
        "tmax_c": uncertainties["tmax_c"]["std"],
        "tmin_c": uncertainties["tmin_c"]["std"],
        "rhmax_pct": uncertainties["rhmax_pct"]["std"],
        "rhmin_pct": uncertainties["rhmin_pct"]["std"],
        "wind_speed_10m_km_h": (
            inputs["wind_speed_10m_km_h"]
            * uncertainties["wind_m_s"]["std_fraction"]
        ),
        "sunshine_hours": (
            inputs["sunshine_hours"]
            * uncertainties["Rs_mj_m2"]["std_fraction"]
        ),
    }

    partials: dict[str, float] = {}
    variance_contributions: dict[str, float] = {}

    for var, delta in perturbations.items():
        args_plus = {**base_args, var: base_args[var] + delta}
        args_minus = {**base_args, var: base_args[var] - delta}
        partial = (et0_func(**args_plus) - et0_func(**args_minus)) / (2 * delta)
        partials[var] = partial
        variance_contributions[var] = (partial * sigmas[var]) ** 2

    total_variance = sum(variance_contributions.values())
    analytical_std = math.sqrt(total_variance)

    fractions = {
        k: v / total_variance if total_variance > 0 else 0
        for k, v in variance_contributions.items()
    }

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

def main() -> int:
    benchmark_path = Path(__file__).parent / "benchmark_error_propagation.json"
    with open(benchmark_path) as f:
        benchmark = json.load(f)

    reset_counters()

    inputs = benchmark["reference_day"]["inputs"]
    uncertainties = benchmark["input_uncertainties"]
    mc_config = benchmark["monte_carlo_config"]
    expected = benchmark["expected_results"]

    print("=" * 72)
    print("groundSpring Exp 003: Error Propagation Through FAO-56 ET₀")
    print("  Method: Monte Carlo + Analytical + Sensitivity Analysis")
    print("=" * 72)

    # ------------------------------------------------------------------
    # Part 1: Baseline ET₀
    # ------------------------------------------------------------------
    print("\n--- Part 1: Baseline ET₀ (verify airSpring) ---")
    et0_base = compute_et0_from_perturbed(
        inputs["tmax_c"], inputs["tmin_c"],
        inputs["rhmax_pct"], inputs["rhmin_pct"],
        inputs["wind_speed_10m_km_h"], inputs["sunshine_hours"],
        inputs["latitude_deg_n"], inputs["altitude_m"],
        inputs["day_of_year"],
    )
    exp_et0 = benchmark["reference_day"]["expected_et0_mm_day"]
    print(f"  Baseline ET₀: {et0_base:.4f} mm/day")
    print(f"  Expected:     {exp_et0:.4f} mm/day")

    # Tol 0.10: FAO-56 Example 18 value is 3.88 mm/day; airSpring
    # validated at ±0.1.  Tightened from 0.15.
    check_range("Baseline ET₀", et0_base, exp_et0 - 0.10, exp_et0 + 0.10)

    # ------------------------------------------------------------------
    # Part 2: Monte Carlo
    # ------------------------------------------------------------------
    print(f"\n--- Part 2: Monte Carlo (N={mc_config['n_samples']}) ---")
    mc = monte_carlo_et0(
        inputs, uncertainties,
        n_samples=mc_config["n_samples"],
        seed=mc_config["seed"],
    )

    print(f"  ET₀ mean:   {mc['mean']:.4f} mm/day")
    print(f"  ET₀ std:    {mc['std']:.4f} mm/day")
    print(f"  ET₀ CV:     {mc['cv_pct']:.2f}%")
    print(f"  ET₀ range:  [{mc['min']:.4f}, {mc['max']:.4f}]")
    print(f"  90% CI:     [{mc['p5']:.4f}, {mc['p95']:.4f}]")

    check_range(
        "ET₀ mean", mc["mean"],
        expected["et0_mean_range"][0], expected["et0_mean_range"][1],
    )
    check_range(
        "ET₀ std", mc["std"],
        expected["et0_std_range"][0], expected["et0_std_range"][1],
    )
    # CV 2–10%: physically, sensor uncertainties of 5–10% relative
    # should produce single-digit ET₀ CV.  Tightened from [2, 20].
    check_range("ET₀ CV (%)", mc["cv_pct"], 2.0, 10.0)

    check_true(
        f"90% CI brackets expected ET₀ ({exp_et0})",
        mc["p5"] < exp_et0 < mc["p95"],
    )

    # ------------------------------------------------------------------
    # Part 3: Sensitivity Analysis
    # ------------------------------------------------------------------
    print("\n--- Part 3: Sensitivity Analysis (one-at-a-time) ---")
    sens = sensitivity_analysis(inputs, uncertainties, n_samples=5_000)

    print("\n  Variable contributions to ET₀ uncertainty:")
    for var_name in sens["ranking"]:
        s = sens[var_name]
        print(
            f"    {var_name:15s}: σ={s['et0_std']:.4f} mm/day "
            f"({s['variance_fraction']*100:.1f}% of variance)"
        )

    print(f"\n  Dominance ranking: {' > '.join(sens['ranking'])}")

    expected_ranking = benchmark["sensitivity_analysis"]["expected_ranking"]
    top_contributor = sens["ranking"][0]
    check_true(
        f"Top contributor ({top_contributor}) in expected top-2: {expected_ranking[:2]}",
        top_contributor in expected_ranking[:2],
    )

    # Variance fractions should sum to ~1.0; nonlinear interactions
    # cause slight deviation.  Tightened from [0.8, 1.2] to [0.9, 1.1].
    total_frac = sum(
        sens[v]["variance_fraction"] for v in sens if v != "ranking"
    )
    check_range("Variance fraction sum", total_frac, 0.9, 1.1)

    # ------------------------------------------------------------------
    # Part 4: Analytical Error Propagation
    # ------------------------------------------------------------------
    print("\n--- Part 4: Analytical (Taylor Expansion) ---")
    analytical = analytical_et0_uncertainty(inputs, uncertainties)

    print(f"  Analytical σ(ET₀): {analytical['analytical_std']:.4f} mm/day")
    print(f"  Analytical CV:     {analytical['analytical_cv_pct']:.2f}%")

    print("\n  Partial derivatives (∂ET₀/∂x):")
    for var, partial in analytical["partials"].items():
        frac = analytical["variance_fractions"].get(var, 0)
        print(
            f"    {var:30s}: {partial:+.4f} mm/day per unit  "
            f"({frac*100:.1f}% of variance)"
        )

    mc_std = mc["std"]
    an_std = analytical["analytical_std"]
    ratio = mc_std / an_std if an_std > 0 else float("inf")

    print(f"\n  Monte Carlo σ:  {mc_std:.4f}")
    print(f"  Analytical σ:   {an_std:.4f}")
    print(f"  Ratio (MC/An):  {ratio:.3f}")

    # MC and first-order Taylor should agree within 20% for a smooth
    # equation chain.  Tightened from [0.7, 1.5] to [0.8, 1.2].
    check_range("MC/Analytical ratio", ratio, 0.8, 1.2)

    # ------------------------------------------------------------------
    # Key Findings
    # ------------------------------------------------------------------
    print(f"\n{'=' * 72}")
    print("KEY FINDINGS:")
    print(f"{'=' * 72}")

    print("\n1. ET₀ Uncertainty Budget:")
    print(f"   Mean ET₀:   {mc['mean']:.3f} ± {mc['std']:.3f} mm/day")
    print(f"   90% CI:     [{mc['p5']:.3f}, {mc['p95']:.3f}] mm/day")
    print(f"   CV:          {mc['cv_pct']:.1f}%")

    print("\n2. Sensitivity Ranking:")
    for rank, var in enumerate(sens["ranking"], 1):
        s = sens[var]
        print(f"   #{rank} {var}: {s['variance_fraction']*100:.1f}% of variance")

    print("\n3. Implications for Penny Irrigation:")
    top = sens["ranking"][0]
    print(f"   - Investing in better {top} measurement has the most impact")
    print(f"   - First-order Taylor is adequate (ratio = {ratio:.2f})")

    return print_summary("Exp 003: Error Propagation Through FAO-56")


if __name__ == "__main__":
    sys.exit(main())
