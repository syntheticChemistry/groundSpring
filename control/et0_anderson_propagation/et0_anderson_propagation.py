# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
#!/usr/bin/env python3
"""
groundSpring Experiment 022 — Soil Moisture → Anderson Uncertainty Propagation

Chains FAO-56 ET₀ uncertainty through soil moisture dynamics to Anderson
localization length. Answers: "How much does the 66% humidity-dominated
ET₀ error affect localization length predictions?"

Pipeline:
  1. Sample FAO-56 inputs with uncertainties (humidity dominates at 66%)
  2. Compute ET₀ for each MC sample via FAO-56 Penman-Monteith
  3. Map ET₀ → θ (soil moisture) via a simple water balance
  4. Map θ → W_eff (effective disorder) via linear mapping
  5. Compute γ = lyapunov_exponent(W_eff) via Anderson transfer matrix
  6. Compute ξ = 1/γ (localization length)
  7. Report: CV(ξ) at each stage, contribution of each FAO-56 input

References:
  Allen et al. (1998) FAO Irrigation and Drainage Paper 56
  Bourgain & Kachkovskiy (2018) GAFA 29:3-43
"""

from __future__ import annotations

import json
import os
import sys
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from common import (
    check_range,
    check_true,
    fail_count,
    print_summary,
    reset_counters,
)

# ---------------------------------------------------------------------------
# FAO-56 module discovery (mirrors error_propagation_fao56.py)
# ---------------------------------------------------------------------------


def _discover_fao56_capability() -> Path | None:
    """Discover FAO-56 Penman-Monteith module at runtime."""
    module_file = "penman_monteith.py"
    capability_path = Path("control") / "fao56"

    env_fao = os.environ.get("FAO56_MODULE_PATH")
    if env_fao:
        p = Path(env_fao)
        if (p / module_file).exists():
            return p

    eco_root = os.environ.get("ECOPRIMALS_ROOT")
    if eco_root is None:
        eco_root_path = Path(__file__).resolve().parent.parent.parent.parent
    else:
        eco_root_path = Path(eco_root)

    if eco_root_path.is_dir():
        for sibling in sorted(eco_root_path.iterdir()):
            if not sibling.is_dir():
                continue
            candidate = sibling / capability_path / module_file
            if candidate.exists():
                return candidate.parent

    return None


_fao56_path = _discover_fao56_capability()
if _fao56_path is None:
    print("ERROR: Cannot discover FAO-56 module.")
    print("  groundSpring Exp 022 requires a sibling primal that provides")
    print("  control/fao56/penman_monteith.py, or set FAO56_MODULE_PATH.")
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
    wind_speed_at_2m,
)


# ---------------------------------------------------------------------------
# FAO-56 ET₀ with direct Rs input (benchmark uses rs_mj_m2_day)
# ---------------------------------------------------------------------------


def compute_et0_from_rs(
    tmax_c: float,
    tmin_c: float,
    rhmax_pct: float,
    rhmin_pct: float,
    wind_10m_m_s: float,
    rs_mj_m2_day: float,
    latitude_deg: float,
    altitude_m: float,
    day_of_year: int,
) -> float:
    """FAO-56 ET₀ computation with direct solar radiation input."""
    tmean = (tmax_c + tmin_c) / 2.0
    u2 = wind_speed_at_2m(wind_10m_m_s, 10.0)

    delta = slope_vapour_pressure_curve(tmean)
    P = atmospheric_pressure(altitude_m)
    gamma = psychrometric_constant(P)
    es = mean_saturation_vapour_pressure(tmax_c, tmin_c)
    ea = actual_vapour_pressure_rh(tmax_c, tmin_c, rhmax_pct, rhmin_pct)
    vpd = es - ea

    Ra = extraterrestrial_radiation(latitude_deg, day_of_year)
    Rs = rs_mj_m2_day
    Rso = clear_sky_radiation(altitude_m, Ra)
    Rns = net_shortwave_radiation(Rs)
    Rs_Rso = min(Rs / Rso, 1.0) if Rso > 0 else 0.7
    Rnl = net_longwave_radiation(tmax_c, tmin_c, ea, Rs_Rso)
    Rn = Rns - Rnl
    G = 0.0

    return float(fao56_penman_monteith(Rn, G, tmean, u2, vpd, delta, gamma))


# ---------------------------------------------------------------------------
# Water balance: θ(t+1) = θ(t) + P/D - ET₀·Kc/D
# ---------------------------------------------------------------------------


def run_water_balance(
    et0_daily: float,
    n_days: int,
    theta_init: float,
    precip_mm: float,
    soil_depth_mm: float,
    crop_coef: float,
    theta_min: float,
    theta_max: float,
) -> float:
    """Simple daily water balance; returns final θ."""
    theta = theta_init
    D = soil_depth_mm
    for _ in range(n_days):
        theta = theta + precip_mm / D - et0_daily * crop_coef / D
        theta = np.clip(theta, theta_min, theta_max)
    return float(theta)


# ---------------------------------------------------------------------------
# θ → W_eff mapping (higher θ → lower disorder)
# ---------------------------------------------------------------------------


def theta_to_disorder(theta: float, slope: float, intercept: float) -> float:
    """Map soil moisture to Anderson disorder: W = intercept + slope * (1 - θ)."""
    return intercept + slope * (1.0 - theta)


# ---------------------------------------------------------------------------
# Anderson model: transfer-matrix Lyapunov (adds disorder variance → amplification)
# ---------------------------------------------------------------------------


def lyapunov_exponent_1d(disorder_w: float, energy: float, n: int, seed: int) -> float:
    """Transfer-matrix Lyapunov exponent for 1D Anderson model."""
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


# ---------------------------------------------------------------------------
# Monte Carlo propagation
# ---------------------------------------------------------------------------


def propagate_et0_to_xi(
    fao56: dict,
    water: dict,
    anderson: dict,
    prop: dict,
    rng: np.random.Generator,
) -> dict:
    """Monte Carlo: FAO-56 inputs → ET₀ → θ → W_eff → ξ."""
    n_mc = prop["n_mc_samples"]
    n_days = prop["n_days"]
    unc = fao56["uncertainties"]

    et0_samples = np.zeros(n_mc)
    theta_samples = np.zeros(n_mc)
    xi_samples = np.zeros(n_mc)

    for i in range(n_mc):
        tmax = fao56["tmax_c"] + rng.normal(0, unc["tmax_sigma"])
        tmin = fao56["tmin_c"] + rng.normal(0, unc["tmin_sigma"])
        tmin = min(tmin, tmax - 0.5)
        rhmax = np.clip(fao56["rhmax_pct"] + rng.normal(0, unc["rh_sigma"]), 10, 100)
        rhmin = np.clip(fao56["rhmin_pct"] + rng.normal(0, unc["rh_sigma"]), 5, rhmax)
        wind = max(0.5, fao56["wind_10m_m_s"] + rng.normal(0, unc["wind_sigma"]))
        rs = max(0.1, fao56["rs_mj_m2_day"] + rng.normal(0, unc["rs_sigma"]))

        et0 = compute_et0_from_rs(
            tmax, tmin, rhmax, rhmin, wind, rs,
            fao56["latitude_deg"], fao56["altitude_m"], fao56["day_of_year"],
        )
        et0 = max(0.1, et0)
        et0_samples[i] = et0

        theta = run_water_balance(
            et0, n_days,
            water["theta_initial"], water["daily_precip_mm"],
            water["soil_depth_mm"], water["crop_coefficient"],
            water["theta_min"], water["theta_max"],
        )
        theta_samples[i] = theta

        w_eff = theta_to_disorder(
            theta,
            anderson["theta_to_disorder_slope"],
            anderson["theta_to_disorder_intercept"],
        )
        w_eff = max(w_eff, 0.1)
        gamma = lyapunov_averaged(
            w_eff, 0.0,
            anderson["chain_length"],
            anderson["n_realizations"],
            42 + i,
        )
        xi = 1.0 / max(gamma, 1e-10)
        xi_samples[i] = xi

    et0_mean = float(np.mean(et0_samples))
    et0_std = float(np.std(et0_samples))
    et0_cv = et0_std / max(et0_mean, 1e-10)

    theta_mean = float(np.mean(theta_samples))
    theta_std = float(np.std(theta_samples))
    theta_cv = theta_std / max(theta_mean, 1e-10)

    xi_mean = float(np.mean(xi_samples))
    xi_std = float(np.std(xi_samples))
    xi_cv = xi_std / max(xi_mean, 1e-10)

    return {
        "et0_mean": et0_mean,
        "et0_std": et0_std,
        "et0_cv": et0_cv,
        "theta_mean": theta_mean,
        "theta_std": theta_std,
        "theta_cv": theta_cv,
        "xi_mean": xi_mean,
        "xi_std": xi_std,
        "xi_cv": xi_cv,
    }


# ---------------------------------------------------------------------------
# Sensitivity: which FAO-56 input dominates ET₀ variance?
# ---------------------------------------------------------------------------


def sensitivity_et0(
    fao56: dict,
    prop: dict,
    rng: np.random.Generator,
    n_per_var: int = 200,
) -> dict:
    """One-at-a-time perturbation to rank FAO-56 input contributions."""
    unc = fao56["uncertainties"]
    base_et0 = compute_et0_from_rs(
        fao56["tmax_c"], fao56["tmin_c"],
        fao56["rhmax_pct"], fao56["rhmin_pct"],
        fao56["wind_10m_m_s"], fao56["rs_mj_m2_day"],
        fao56["latitude_deg"], fao56["altitude_m"], fao56["day_of_year"],
    )

    variables = {
        "temperature": (
            lambda: (
                fao56["tmax_c"] + rng.normal(0, unc["tmax_sigma"]),
                min(fao56["tmin_c"] + rng.normal(0, unc["tmin_sigma"]),
                    fao56["tmax_c"] + rng.normal(0, unc["tmax_sigma"]) - 0.5),
            ),
            ("tmax", "tmin"),
        ),
        "humidity": (
            lambda: (
                np.clip(fao56["rhmax_pct"] + rng.normal(0, unc["rh_sigma"]), 10, 100),
                np.clip(fao56["rhmin_pct"] + rng.normal(0, unc["rh_sigma"]), 5, 100),
            ),
            ("rhmax", "rhmin"),
        ),
        "wind": (
            lambda: max(0.5, fao56["wind_10m_m_s"] + rng.normal(0, unc["wind_sigma"])),
            ("wind",),
        ),
        "radiation": (
            lambda: max(0.1, fao56["rs_mj_m2_day"] + rng.normal(0, unc["rs_sigma"])),
            ("rs",),
        ),
    }

    results: dict = {}
    for var_name, (perturb_fn, keys) in variables.items():
        et0_vals = []
        for _ in range(n_per_var):
            tmax, tmin = fao56["tmax_c"], fao56["tmin_c"]
            rhmax, rhmin = fao56["rhmax_pct"], fao56["rhmin_pct"]
            wind = fao56["wind_10m_m_s"]
            rs = fao56["rs_mj_m2_day"]

            if "tmax" in keys:
                tmax, tmin = perturb_fn()
            elif "rhmax" in keys:
                rhmax, rhmin = perturb_fn()
            elif "wind" in keys:
                wind = perturb_fn()
            else:
                rs = perturb_fn()

            et0 = compute_et0_from_rs(
                tmax, tmin, rhmax, rhmin, wind, rs,
                fao56["latitude_deg"], fao56["altitude_m"], fao56["day_of_year"],
            )
            et0_vals.append(et0)

        std = float(np.std(et0_vals))
        results[var_name] = {"et0_std": std, "et0_mean": float(np.mean(et0_vals))}

    total_var = sum(r["et0_std"] ** 2 for r in results.values())
    for var_name in results:
        v = results[var_name]["et0_std"] ** 2
        results[var_name]["variance_fraction"] = v / total_var if total_var > 0 else 0

    ranking = sorted(results.keys(), key=lambda k: results[k]["et0_std"], reverse=True)
    results["ranking"] = ranking
    return results


# ---------------------------------------------------------------------------
# Main validation harness
# ---------------------------------------------------------------------------


def main() -> int:
    benchmark_path = Path(__file__).parent / "benchmark_et0_anderson.json"
    with open(benchmark_path) as f:
        benchmark = json.load(f)

    reset_counters()

    fao56 = benchmark["fao56_inputs"]
    water = benchmark["water_balance"]
    anderson = benchmark["anderson_model"]
    prop = benchmark["propagation"]
    expected = benchmark["expected"]

    rng = np.random.default_rng(prop.get("mc_seed", 2026))

    print("=" * 72)
    print("groundSpring Exp 022: ET₀ → Anderson Uncertainty Propagation")
    print("  FAO-56 → Water balance → θ → W_eff → ξ (localization length)")
    print("=" * 72)

    # --- Step 1: Monte Carlo propagation ---
    print("\n--- Step 1: Monte Carlo propagation ---")
    result = propagate_et0_to_xi(fao56, water, anderson, prop, rng)

    print(f"  ET₀:   mean={result['et0_mean']:.3f} mm/day, CV={result['et0_cv']:.4f}")
    print(f"  θ:     mean={result['theta_mean']:.3f}, CV={result['theta_cv']:.4f}")
    print(f"  ξ:     mean={result['xi_mean']:.1f}, CV={result['xi_cv']:.4f}")

    check_range(
        "ET₀ mean",
        result["et0_mean"],
        expected["et0_mean_range"][0],
        expected["et0_mean_range"][1],
    )
    check_range(
        "ET₀ CV",
        result["et0_cv"],
        expected["et0_cv_range"][0],
        expected["et0_cv_range"][1],
    )
    check_range(
        "θ final mean",
        result["theta_mean"],
        expected["theta_final_range"][0],
        expected["theta_final_range"][1],
    )
    check_range(
        "θ CV",
        result["theta_cv"],
        expected["theta_cv_range"][0],
        expected["theta_cv_range"][1],
    )
    check_range(
        "ξ CV",
        result["xi_cv"],
        expected["xi_cv_range"][0],
        expected["xi_cv_range"][1],
    )

    # --- Step 2: Sensitivity — humidity dominates ET₀ ---
    print("\n--- Step 2: FAO-56 input sensitivity ---")
    sens = sensitivity_et0(fao56, prop, rng)
    for var_name in sens["ranking"]:
        frac = sens[var_name].get("variance_fraction", 0)
        print(f"  {var_name:12s}: {frac*100:.1f}% of ET₀ variance")

    humidity_dominates = sens["ranking"][0] == "humidity"
    check_true(
        "Humidity dominates ET₀ uncertainty",
        humidity_dominates,
    )

    # --- Step 3: Uncertainty propagation ---
    print("\n--- Step 3: Uncertainty propagation ---")
    ratio = result["xi_cv"] / max(result["et0_cv"], 1e-10)
    ratio_min = expected.get("xi_cv_to_et0_cv_ratio_min", 0.5)
    print(f"  ξ CV ({result['xi_cv']:.4f}) / ET₀ CV ({result['et0_cv']:.4f}) = ratio {ratio:.3f}")
    check_true(
        f"ET₀ uncertainty propagates through Anderson (ratio {ratio:.3f} >= {ratio_min})",
        ratio >= ratio_min,
    )

    # --- Summary ---
    print("\n" + "=" * 72)
    print("Exp 022 Summary:")
    print(f"  ET₀ CV: {result['et0_cv']:.4f} → θ CV: {result['theta_cv']:.4f} "
          f"→ ξ CV: {result['xi_cv']:.4f}")
    print(f"  Dominant FAO-56 input: {sens['ranking'][0]}")
    print("=" * 72)

    print_summary("Exp 022: ET₀ → Anderson Propagation")
    return 1 if fail_count() > 0 else 0


if __name__ == "__main__":
    sys.exit(main())
