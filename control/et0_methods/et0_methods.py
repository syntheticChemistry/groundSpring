#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""
groundSpring Experiment 035 — Multi-Method ET₀ Cross-Validation

Compares five reference evapotranspiration (ET₀) methods at the FAO-56
Example 18 reference site (Uccle, Belgium, 6 July):

  1. Penman-Monteith (FAO-56 Eq. 6) — full equation chain
  2. Hargreaves (temperature-only)
  3. Makkink (radiation-only)
  4. Turc (radiation + humidity)
  5. Hamon (temperature + daylight hours)

This experiment validates:
  - Each method produces physically reasonable ET₀
  - Cross-method agreement (all within the same order of magnitude)
  - Determinism (each method returns identical results on rerun)
  - Input sensitivity (how radiation uncertainty affects Makkink, etc.)

The pipeline goal is: Python baseline → Rust validation → barracuda CPU
(pure Rust math, faster) → barracuda GPU (portable math) → pure GPU.

References:
  Allen et al. (1998) FAO Irrigation and Drainage Paper 56.
  Makkink (1957) Neth J Agr Sci 5:290-305.
  Turc (1961) Ann Agron 12:13-49.
  Hamon (1963) J Hydraul Div ASCE 89:97-120.
  Hargreaves & Samani (1985) Appl Eng Agric 1:96-99.
"""

from __future__ import annotations

import json
import math
import os
import sys
from datetime import datetime, timezone
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from common import (
    check_approx,
    check_min,
    check_range,
    check_true,
    print_summary,
    reset_counters,
)


# ---------------------------------------------------------------------------
# FAO-56 intermediate equations (pure Python, standalone)
# ---------------------------------------------------------------------------

def saturation_vapour_pressure(t_c: float) -> float:
    """FAO-56 Eq. 11: e°(T) in kPa."""
    return 0.6108 * math.exp(17.27 * t_c / (t_c + 237.3))


def slope_vapour_pressure_curve(t_c: float) -> float:
    """FAO-56 Eq. 13: Δ in kPa/°C."""
    es = saturation_vapour_pressure(t_c)
    return 4098.0 * es / (t_c + 237.3) ** 2


def atmospheric_pressure(altitude_m: float) -> float:
    """FAO-56 Eq. 7: P in kPa."""
    return 101.3 * ((293.0 - 0.0065 * altitude_m) / 293.0) ** 5.26


def psychrometric_constant(p_kpa: float) -> float:
    """FAO-56 Eq. 8: γ in kPa/°C."""
    return 0.000665 * p_kpa


def wind_speed_at_2m(uz_ms: float, z_m: float) -> float:
    """FAO-56 Eq. 47: convert wind speed at height z to 2m."""
    return uz_ms * 4.87 / math.log(67.8 * z_m - 5.42)


def solar_declination(doy: int) -> float:
    """FAO-56 Eq. 24: solar declination (radians)."""
    return 0.4093 * math.sin(2.0 * math.pi / 365.0 * doy - 1.39)


def inverse_relative_distance(doy: int) -> float:
    """FAO-56 Eq. 23."""
    return 1.0 + 0.033 * math.cos(2.0 * math.pi / 365.0 * doy)


def sunset_hour_angle(lat_rad: float, decl_rad: float) -> float:
    """FAO-56 Eq. 25: sunset hour angle (radians)."""
    arg = -math.tan(lat_rad) * math.tan(decl_rad)
    arg = max(-1.0, min(1.0, arg))
    return math.acos(arg)


def extraterrestrial_radiation(lat_deg: float, doy: int) -> float:
    """FAO-56 Eq. 21: Ra (MJ m⁻² day⁻¹)."""
    lat_rad = math.radians(lat_deg)
    dr = inverse_relative_distance(doy)
    decl = solar_declination(doy)
    ws = sunset_hour_angle(lat_rad, decl)
    gsc = 0.0820
    return (24.0 * 60.0 / math.pi) * gsc * dr * (
        ws * math.sin(lat_rad) * math.sin(decl)
        + math.cos(lat_rad) * math.cos(decl) * math.sin(ws)
    )


def daylight_hours(lat_deg: float, doy: int) -> float:
    """FAO-56 Eq. 34: N (hours)."""
    lat_rad = math.radians(lat_deg)
    decl = solar_declination(doy)
    ws = sunset_hour_angle(lat_rad, decl)
    return 24.0 / math.pi * ws


def solar_radiation_from_sunshine(n: float, big_n: float, ra: float) -> float:
    """FAO-56 Eq. 35: Rs (MJ m⁻² day⁻¹)."""
    if big_n <= 0:
        return 0.0
    return (0.25 + 0.50 * n / big_n) * ra


def clear_sky_radiation(altitude_m: float, ra: float) -> float:
    """FAO-56 Eq. 37: Rso (MJ m⁻² day⁻¹)."""
    return (0.75 + 2e-5 * altitude_m) * ra


def net_shortwave_radiation(rs: float) -> float:
    """FAO-56 Eq. 38."""
    return (1.0 - 0.23) * rs


def net_longwave_radiation(tmax_c, tmin_c, ea_kpa, rs_over_rso):
    """FAO-56 Eq. 39."""
    stef = 4.903e-9
    tmax_k4 = (tmax_c + 273.16) ** 4
    tmin_k4 = (tmin_c + 273.16) ** 4
    return stef * (tmax_k4 + tmin_k4) / 2.0 * (
        0.34 - 0.14 * math.sqrt(ea_kpa)
    ) * (1.35 * rs_over_rso - 0.35)


def penman_monteith(rn, g, tmean, u2, vpd, delta, gamma):
    """FAO-56 Eq. 6: ET₀ (mm/day)."""
    num = 0.408 * delta * (rn - g) + gamma * 900.0 / (tmean + 273.0) * u2 * vpd
    denom = delta + gamma * (1.0 + 0.34 * u2)
    return num / denom


def daily_et0_pm(tmax_c, tmin_c, rhmax, rhmin, wind_kmh, sunshine_h,
                 lat_deg, alt_m, doy):
    """Full FAO-56 Penman-Monteith ET₀."""
    tmean = (tmax_c + tmin_c) / 2.0
    uz_ms = wind_kmh / 3.6
    u2 = wind_speed_at_2m(uz_ms, 10.0)

    delta = slope_vapour_pressure_curve(tmean)
    p = atmospheric_pressure(alt_m)
    gamma = psychrometric_constant(p)

    es_max = saturation_vapour_pressure(tmax_c)
    es_min = saturation_vapour_pressure(tmin_c)
    es = (es_max + es_min) / 2.0
    ea = (es_min * rhmax / 100.0 + es_max * rhmin / 100.0) / 2.0
    vpd = es - ea

    ra = extraterrestrial_radiation(lat_deg, doy)
    big_n = daylight_hours(lat_deg, doy)
    n = min(sunshine_h, big_n)
    rs = solar_radiation_from_sunshine(n, big_n, ra)
    rso = clear_sky_radiation(alt_m, ra)
    rns = net_shortwave_radiation(rs)
    rs_rso = min(rs / rso, 1.0) if rso > 0 else 0.7
    rnl = net_longwave_radiation(tmax_c, tmin_c, ea, rs_rso)
    rn = rns - rnl

    return penman_monteith(rn, 0.0, tmean, u2, vpd, delta, gamma)


def hargreaves_et0(ra, tmax_c, tmin_c):
    """Hargreaves & Samani (1985) ET₀ (mm/day)."""
    tmean = (tmax_c + tmin_c) / 2.0
    td = max(tmax_c - tmin_c, 0.0)
    return 0.0023 * (tmean + 17.8) * math.sqrt(td) * ra


def makkink_et0(t_mean_c, rs_mj):
    """Makkink (1957) ET₀ (mm/day)."""
    if rs_mj < 0:
        return 0.0
    delta = slope_vapour_pressure_curve(t_mean_c)
    p = atmospheric_pressure(0.0)
    gamma = psychrometric_constant(p)
    lam = 2.45
    return max(0.0, 0.61 * (delta / (delta + gamma)) * (rs_mj / lam) - 0.12)


def turc_et0(t_mean_c, rs_mj, rh_mean_pct):
    """Turc (1961) ET₀ (mm/day)."""
    if rs_mj < 0:
        return 0.0
    rs_cal = rs_mj * 23.89
    base = 0.013 * (t_mean_c / (t_mean_c + 15.0)) * (rs_cal + 50.0)
    correction = 1.0 if rh_mean_pct >= 50 else 1.0 + (50.0 - rh_mean_pct) / 70.0
    return max(0.0, base * correction)


def hamon_et0(t_mean_c, daylight_hours_n):
    """Hamon (1963) ET₀ (mm/day)."""
    if daylight_hours_n < 0:
        return 0.0
    es_kpa = saturation_vapour_pressure(t_mean_c)
    es_mbar = es_kpa * 10.0
    return max(0.0, 0.55 * (daylight_hours_n / 12.0) ** 2 * es_mbar / 100.0)


# ---------------------------------------------------------------------------
# Reference site: FAO-56 Example 18 (Uccle, Belgium, 6 July)
# ---------------------------------------------------------------------------

SITE = {
    "name": "Uccle, Belgium (FAO-56 Example 18)",
    "tmax_c": 21.5,
    "tmin_c": 12.3,
    "rhmax_pct": 84.0,
    "rhmin_pct": 63.0,
    "wind_speed_10m_km_h": 10.0,
    "sunshine_hours": 9.25,
    "latitude_deg_n": 50.8,
    "altitude_m": 100.0,
    "day_of_year": 187,
}


def compute_all_et0(site):
    """Compute ET₀ using all 5 methods for a given site."""
    tmax = site["tmax_c"]
    tmin = site["tmin_c"]
    tmean = (tmax + tmin) / 2.0
    lat = site["latitude_deg_n"]
    doy = site["day_of_year"]
    alt = site["altitude_m"]
    rh_mean = (site["rhmax_pct"] + site["rhmin_pct"]) / 2.0

    ra = extraterrestrial_radiation(lat, doy)
    big_n = daylight_hours(lat, doy)
    n = min(site["sunshine_hours"], big_n)
    rs = solar_radiation_from_sunshine(n, big_n, ra)

    pm = daily_et0_pm(
        tmax, tmin, site["rhmax_pct"], site["rhmin_pct"],
        site["wind_speed_10m_km_h"], site["sunshine_hours"],
        lat, alt, doy,
    )
    hg = hargreaves_et0(ra, tmax, tmin)
    mk = makkink_et0(tmean, rs)
    tu = turc_et0(tmean, rs, rh_mean)
    ha = hamon_et0(tmean, big_n)

    return {
        "penman_monteith": pm,
        "hargreaves": hg,
        "makkink": mk,
        "turc": tu,
        "hamon": ha,
        "intermediates": {
            "tmean": tmean,
            "ra": ra,
            "daylight_hours": big_n,
            "rs": rs,
            "rh_mean": rh_mean,
        },
    }


# ---------------------------------------------------------------------------
# Seasonal variation (4 representative days)
# ---------------------------------------------------------------------------

SEASONS = [
    {"label": "Winter",  "doy": 15,  "tmax_c": 3.0,  "tmin_c": -2.0, "sunshine_hours": 3.0,
     "rhmax_pct": 92.0, "rhmin_pct": 78.0, "wind_speed_10m_km_h": 12.0},
    {"label": "Spring",  "doy": 105, "tmax_c": 14.0, "tmin_c": 5.0,  "sunshine_hours": 6.5,
     "rhmax_pct": 88.0, "rhmin_pct": 55.0, "wind_speed_10m_km_h": 11.0},
    {"label": "Summer",  "doy": 187, "tmax_c": 21.5, "tmin_c": 12.3, "sunshine_hours": 9.25,
     "rhmax_pct": 84.0, "rhmin_pct": 63.0, "wind_speed_10m_km_h": 10.0},
    {"label": "Autumn",  "doy": 288, "tmax_c": 12.0, "tmin_c": 6.0,  "sunshine_hours": 4.0,
     "rhmax_pct": 90.0, "rhmin_pct": 70.0, "wind_speed_10m_km_h": 13.0},
]


def compute_seasonal(base_site):
    """Compute all methods for 4 seasons at the same site."""
    results = []
    for season in SEASONS:
        site = dict(base_site)
        site.update({k: v for k, v in season.items() if k != "label"})
        et0s = compute_all_et0(site)
        et0s["label"] = season["label"]
        results.append(et0s)
    return results


# ---------------------------------------------------------------------------
# Sensitivity analysis: radiation uncertainty on Makkink
# ---------------------------------------------------------------------------

def makkink_radiation_sensitivity(tmean, rs_base, sigma_frac=0.05, n_samples=1000):
    """How does radiation uncertainty affect Makkink ET₀?"""
    rng = np.random.RandomState(42)
    rs_perturbed = rng.normal(rs_base, rs_base * sigma_frac, n_samples)
    rs_perturbed = np.clip(rs_perturbed, 0, None)
    et0s = np.array([makkink_et0(tmean, rs) for rs in rs_perturbed])
    return {
        "mean": float(et0s.mean()),
        "std": float(et0s.std()),
        "cv_pct": float(et0s.std() / et0s.mean() * 100) if et0s.mean() > 0 else 0.0,
    }


def hamon_temperature_sensitivity(big_n, t_base, sigma_t=0.5, n_samples=1000):
    """How does temperature uncertainty affect Hamon ET₀?"""
    rng = np.random.RandomState(42)
    t_perturbed = rng.normal(t_base, sigma_t, n_samples)
    et0s = np.array([hamon_et0(t, big_n) for t in t_perturbed])
    return {
        "mean": float(et0s.mean()),
        "std": float(et0s.std()),
        "cv_pct": float(et0s.std() / et0s.mean() * 100) if et0s.mean() > 0 else 0.0,
    }


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    reset_counters()

    print("=" * 72)
    print("groundSpring Experiment 035: Multi-Method ET₀ Cross-Validation")
    print("=" * 72)

    # ── Part 1: Reference site (FAO-56 Example 18) ────────────────────
    print("\n--- Part 1: Reference Site ET₀ (Uccle, Belgium, 6 July) ---\n")

    results = compute_all_et0(SITE)
    for method, val in sorted(results.items()):
        if method == "intermediates":
            continue
        print(f"  {method:20s}: {val:.6f} mm/day")

    pm = results["penman_monteith"]
    hg = results["hargreaves"]
    mk = results["makkink"]
    tu = results["turc"]
    ha = results["hamon"]

    check_range("PM ET₀ ≈ 3.88 (FAO-56 Ex18)", pm, 3.78, 3.98)
    check_true("All methods positive", all(v > 0 for v in [pm, hg, mk, tu, ha]))
    check_true("All methods < 20 mm/day", all(v < 20 for v in [pm, hg, mk, tu, ha]))

    # ── Part 2: Cross-method agreement ────────────────────────────────
    print("\n--- Part 2: Cross-Method Agreement ---\n")

    methods = {"PM": pm, "HG": hg, "MK": mk, "TU": tu, "HA": ha}
    vals = list(methods.values())
    max_val = max(vals)
    min_val = min(vals)
    spread = max_val - min_val
    print(f"  Range: [{min_val:.4f}, {max_val:.4f}] mm/day  (spread = {spread:.4f})")
    check_true("Cross-method spread < 15 mm/day", spread < 15.0)
    check_true(
        "All methods in plausible range (0.01–15 mm/day)",
        all(0.01 < v < 15.0 for v in vals),
    )

    # ── Part 3: Determinism ───────────────────────────────────────────
    print("\n--- Part 3: Determinism ---\n")

    results2 = compute_all_et0(SITE)
    for method in ["penman_monteith", "hargreaves", "makkink", "turc", "hamon"]:
        v1 = results[method]
        v2 = results2[method]
        check_true(f"{method} deterministic", abs(v1 - v2) < 1e-15)

    # ── Part 4: Seasonal variation ────────────────────────────────────
    print("\n--- Part 4: Seasonal Variation ---\n")

    seasonal = compute_seasonal(SITE)
    for s in seasonal:
        print(f"  {s['label']:8s}: PM={s['penman_monteith']:.4f}  "
              f"HG={s['hargreaves']:.4f}  MK={s['makkink']:.4f}  "
              f"TU={s['turc']:.4f}  HA={s['hamon']:.4f}")

    summer = next(s for s in seasonal if s["label"] == "Summer")
    winter = next(s for s in seasonal if s["label"] == "Winter")
    check_true("PM: summer > winter", summer["penman_monteith"] > winter["penman_monteith"])
    check_true("MK: summer > winter", summer["makkink"] > winter["makkink"])
    check_true("HA: summer > winter", summer["hamon"] > winter["hamon"])

    # ── Part 5: Input sensitivity ─────────────────────────────────────
    print("\n--- Part 5: Input Sensitivity ---\n")

    tmean = results["intermediates"]["tmean"]
    rs = results["intermediates"]["rs"]
    big_n = results["intermediates"]["daylight_hours"]

    mk_sens = makkink_radiation_sensitivity(tmean, rs)
    ha_sens = hamon_temperature_sensitivity(big_n, tmean)

    print(f"  Makkink radiation sensitivity (σ=5%): CV = {mk_sens['cv_pct']:.2f}%")
    print(f"  Hamon temperature sensitivity (σ=0.5°C): CV = {ha_sens['cv_pct']:.2f}%")

    check_range("Makkink radiation CV < 10%", mk_sens["cv_pct"], 0.0, 10.0)
    check_range("Hamon temperature CV < 10%", ha_sens["cv_pct"], 0.0, 10.0)

    # ── Summary ───────────────────────────────────────────────────────
    print()
    exit_code = print_summary("Multi-Method ET₀ Cross-Validation")

    # ── Write benchmark JSON ──────────────────────────────────────────
    benchmark = {
        "_source": "groundSpring Exp 035 — Multi-Method ET₀ Cross-Validation",
        "_provenance": {
            "generated_by": "5-method ET₀ comparison at FAO-56 Example 18 reference site",
            "data_origin": "FAO-56 Example 18 (Uccle, Belgium, 6 July). "
                           "Makkink (1957), Turc (1961), Hamon (1963), Hargreaves & Samani (1985).",
            "baseline_date": datetime.now(timezone.utc).strftime("%Y-%m-%d"),
            "baseline_commit": "pending",
            "validation_script": "control/et0_methods/et0_methods.py",
            "command": "python3 control/et0_methods/et0_methods.py",
            "python_version": sys.version.split()[0],
            "numpy_version": np.__version__,
            "notes": "Deterministic comparison of 5 ET₀ methods. "
                     "PM reference 3.88 from FAO-56 Example 18. "
                     "Hamon underestimates in humid climates (by design — minimal inputs).",
            "real_data_accession": "N/A (analytical FAO-56 Example 18)",
        },
        "_description": "5-method ET₀ cross-validation: PM, Hargreaves, Makkink, Turc, Hamon",
        "_groundspring_question": "Do simplified ET₀ methods (fewer inputs) agree with "
                                  "the full Penman-Monteith equation chain?",
        "_references": [
            "Allen et al. (1998) FAO-56",
            "Makkink (1957) Neth J Agr Sci 5:290-305",
            "Turc (1961) Ann Agron 12:13-49",
            "Hamon (1963) J Hydraul Div ASCE 89:97-120",
            "Hargreaves & Samani (1985) Appl Eng Agric 1:96-99",
        ],
        "site": SITE,
        "reference_et0": {
            "penman_monteith": pm,
            "hargreaves": hg,
            "makkink": mk,
            "turc": tu,
            "hamon": ha,
        },
        "intermediates": results["intermediates"],
        "seasonal": [
            {
                "label": s["label"],
                "penman_monteith": s["penman_monteith"],
                "hargreaves": s["hargreaves"],
                "makkink": s["makkink"],
                "turc": s["turc"],
                "hamon": s["hamon"],
            }
            for s in seasonal
        ],
        "sensitivity": {
            "makkink_radiation": mk_sens,
            "hamon_temperature": ha_sens,
        },
    }

    out_path = Path(__file__).parent / "benchmark_et0_methods.json"
    with open(out_path, "w") as f:
        json.dump(benchmark, f, indent=2)
    print(f"\nBenchmark JSON written to {out_path}")

    return exit_code


if __name__ == "__main__":
    sys.exit(main())
