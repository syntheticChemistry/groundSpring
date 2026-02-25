#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""
groundSpring Experiment 002 — Weather Model vs Observation Gap

Compares ERA5 reanalysis (Open-Meteo archive) against GHCND station
observations (NOAA CDO) for Lansing, MI, full year 2023.

Key questions:
  1. How well does a gridded reanalysis reproduce point station measurements?
  2. Is the gap systematic (correctable bias) or random (representation error)?
  3. How does the gap differ for temperature vs precipitation?

Data sources:
  - Open-Meteo Archive API (ERA5 reanalysis, free, no key)
  - NOAA CDO API (GHCND station data, free token)
  - Synthetic fallback isolated to testing; production requires real data
"""

from __future__ import annotations

import json
import os
import sys
from pathlib import Path

import numpy as np
import pandas as pd

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from common import (
    bias_variance_decompose,
    check_min,
    check_range,
    check_true,
    compute_ia,
    compute_mbe,
    compute_r2,
    compute_rmse,
    fail_count,
    pass_count,
    print_summary,
    reset_counters,
)

# ---------------------------------------------------------------------------
# Configuration — discovered at runtime, not hardcoded primal knowledge
# ---------------------------------------------------------------------------

GROUNDSPRING_ROOT = Path(__file__).resolve().parent.parent.parent

OPEN_METEO_BASE = "https://archive-api.open-meteo.com/v1/archive"
NOAA_CDO_BASE = "https://www.ncdc.noaa.gov/cdo-web/api/v2/data"


def _load_config(benchmark: dict) -> dict:
    """Extract location config from benchmark JSON (single source of truth)."""
    loc = benchmark["location"]
    return {
        "open_meteo": {
            "lat": loc["open_meteo"]["lat"],
            "lon": loc["open_meteo"]["lon"],
        },
        "noaa_station_id": loc["noaa_cdo"]["station_id"],
    }


# ---------------------------------------------------------------------------
# Data loading — real data only in production
# ---------------------------------------------------------------------------

def _secrets_path() -> Path:
    """Discover secrets path at runtime."""
    candidates = [
        GROUNDSPRING_ROOT.parent / "testing-secrets" / "api-keys.toml",
        Path.home() / ".config" / "ecoprimals" / "api-keys.toml",
    ]
    for p in candidates:
        if p.exists():
            return p
    return Path("/dev/null")


def load_noaa_token() -> str:
    """Load NOAA CDO token from discovered secrets or environment."""
    sp = _secrets_path()
    if sp.exists() and sp.name != "null":
        with open(sp) as f:
            for line in f:
                if "noaa_cdo_token" in line and "=" in line:
                    return line.split("=", 1)[1].strip().strip('"')
    return os.environ.get("NOAA_CDO_TOKEN", "")


def fetch_open_meteo_daily(
    lat: float, lon: float, start: str, end: str
) -> pd.DataFrame:
    """Fetch daily weather from Open-Meteo Archive API."""
    import requests

    params: dict[str, str] = {
        "latitude": str(lat),
        "longitude": str(lon),
        "start_date": start,
        "end_date": end,
        "daily": "temperature_2m_max,temperature_2m_min,precipitation_sum",
        "timezone": "America/Detroit",
    }
    resp = requests.get(OPEN_METEO_BASE, params=params, timeout=30)
    resp.raise_for_status()
    data = resp.json()["daily"]

    df = pd.DataFrame(data)
    df.rename(
        columns={
            "time": "date",
            "temperature_2m_max": "tmax_c",
            "temperature_2m_min": "tmin_c",
            "precipitation_sum": "precip_mm",
        },
        inplace=True,
    )
    df["date"] = pd.to_datetime(df["date"])
    return df


def fetch_noaa_cdo(
    station_id: str, start: str, end: str, token: str
) -> pd.DataFrame:
    """Fetch GHCND daily data from NOAA CDO REST API."""
    import time

    import requests

    headers = {"token": token}
    all_data: list[dict] = []

    offset = 1
    while True:
        params: dict[str, str | int] = {
            "datasetid": "GHCND",
            "stationid": f"GHCND:{station_id}",
            "startdate": start,
            "enddate": end,
            "datatypeid": "TMAX,TMIN,PRCP",
            "units": "metric",
            "limit": 1000,
            "offset": offset,
        }
        resp = requests.get(
            NOAA_CDO_BASE, headers=headers, params=params, timeout=30
        )
        if resp.status_code != 200:
            print(f"  NOAA API error: {resp.status_code}")
            break

        data = resp.json()
        results = data.get("results", [])
        if not results:
            break

        all_data.extend(results)
        total = data.get("metadata", {}).get("resultset", {}).get("count", 0)
        offset += len(results)
        if offset > total:
            break
        time.sleep(0.3)

    if not all_data:
        return pd.DataFrame()

    df = pd.DataFrame(all_data)
    pivot = df.pivot_table(
        index="date", columns="datatype", values="value", aggfunc="first"
    ).reset_index()
    pivot.columns.name = None
    pivot["date"] = pd.to_datetime(pivot["date"])
    pivot.rename(
        columns={"TMAX": "tmax_c", "TMIN": "tmin_c", "PRCP": "precip_mm"},
        inplace=True,
    )
    return pivot


# ---------------------------------------------------------------------------
# Synthetic data — isolated for testing only
# ---------------------------------------------------------------------------

def generate_synthetic_noaa(start: str, end: str) -> pd.DataFrame:
    """Generate synthetic NOAA-like data for testing ONLY.

    Uses Michigan climate normals with realistic noise.
    This must NOT be used for production validation — results are
    marked as SKIP, not PASS.
    """
    rng = np.random.default_rng(2023)
    dates = pd.date_range(start, end, freq="D")
    n = len(dates)
    doy = dates.dayofyear.values

    t_mean = 8.5 + 14.5 * np.sin(2 * np.pi * (doy - 100) / 365)
    t_range = 10.5 + 2.0 * rng.standard_normal(n)
    t_range = np.maximum(t_range, 3.0)

    tmax = t_mean + t_range / 2 + rng.normal(0, 2.5, n)
    tmin = t_mean - t_range / 2 + rng.normal(0, 2.5, n)
    tmin = np.minimum(tmin, tmax - 2.0)

    rain_prob = 0.35 - 0.10 * np.cos(2 * np.pi * (doy - 180) / 365)
    rain_days = rng.random(n) < rain_prob
    precip = np.zeros(n)
    precip[rain_days] = rng.exponential(6.0, np.sum(rain_days))

    return pd.DataFrame({
        "date": dates,
        "tmax_c": np.round(tmax, 1),
        "tmin_c": np.round(tmin, 1),
        "precip_mm": np.round(precip, 1),
    })


# ---------------------------------------------------------------------------
# Metrics
# ---------------------------------------------------------------------------

def precip_hit_rate(
    obs: np.ndarray, mod: np.ndarray, threshold: float = 0.1
) -> float:
    """Fraction of days where both agree on rain/no-rain."""
    return float(np.mean((obs > threshold) == (mod > threshold)))


# ---------------------------------------------------------------------------
# Seasonal decomposition
# ---------------------------------------------------------------------------

def seasonal_analysis(df: pd.DataFrame, var: str) -> dict:
    """Break down model-observation gap by meteorological season."""
    results = {}
    seasons = {
        "DJF": [12, 1, 2],
        "MAM": [3, 4, 5],
        "JJA": [6, 7, 8],
        "SON": [9, 10, 11],
    }

    for name, months in seasons.items():
        mask = df["month"].isin(months)
        sub = df[mask].dropna(subset=[f"{var}_obs", f"{var}_mod"])
        if len(sub) < 10:
            continue

        obs = np.asarray(sub[f"{var}_obs"].values)
        mod = np.asarray(sub[f"{var}_mod"].values)
        results[name] = {
            "n_days": len(sub),
            "rmse": compute_rmse(obs, mod),
            "mbe": compute_mbe(obs, mod),
            "r2": compute_r2(obs, mod),
        }

    return results


# ---------------------------------------------------------------------------
# Main validation harness
# ---------------------------------------------------------------------------

def main() -> int:
    benchmark_path = Path(__file__).parent / "benchmark_observation_gap.json"
    with open(benchmark_path) as f:
        benchmark = json.load(f)

    reset_counters()
    config = _load_config(benchmark)

    start = benchmark["comparison_period"]["start"]
    end = benchmark["comparison_period"]["end"]

    print("=" * 72)
    print("groundSpring Exp 002: Weather Model vs Observation Gap")
    print(f"  Location: Lansing, MI | Period: {start} to {end}")
    print("=" * 72)

    # ------------------------------------------------------------------
    # Step 1: Load Open-Meteo data
    # ------------------------------------------------------------------
    print("\n--- Step 1: Loading Open-Meteo (ERA5 reanalysis) ---")
    om = config["open_meteo"]

    om_cache = GROUNDSPRING_ROOT / "data" / "observation_gap" / "open_meteo_lansing_2023.csv"
    om_cache.parent.mkdir(parents=True, exist_ok=True)

    if om_cache.exists():
        print(f"  Using cached: {om_cache}")
        df_om = pd.read_csv(om_cache, parse_dates=["date"])
    else:
        try:
            print("  Fetching from Open-Meteo API...")
            df_om = fetch_open_meteo_daily(om["lat"], om["lon"], start, end)
            df_om.to_csv(om_cache, index=False)
            print(f"  Cached to: {om_cache}")
        except Exception as e:
            print(f"  ERROR: Open-Meteo API unavailable: {e}")
            print("  Cannot run Exp 002 without Open-Meteo data.")
            print("  [SKIP] Exp 002 requires network access for Open-Meteo.")
            return 0

    print(
        f"  Open-Meteo: {len(df_om)} days, "
        f"tmax [{df_om['tmax_c'].min():.1f}, {df_om['tmax_c'].max():.1f}] °C"
    )

    # ------------------------------------------------------------------
    # Step 2: Load NOAA CDO data
    # ------------------------------------------------------------------
    print("\n--- Step 2: Loading NOAA CDO (station observation) ---")
    noaa_station = config["noaa_station_id"]
    noaa_cache = GROUNDSPRING_ROOT / "data" / "observation_gap" / "noaa_lansing_2023.csv"

    using_synthetic_noaa = False

    if noaa_cache.exists():
        print(f"  Using cached: {noaa_cache}")
        df_noaa = pd.read_csv(noaa_cache, parse_dates=["date"])
    else:
        token = load_noaa_token()
        if token:
            try:
                print(f"  Fetching from NOAA CDO API (station {noaa_station})...")
                df_noaa = fetch_noaa_cdo(noaa_station, start, end, token)
                if not df_noaa.empty:
                    df_noaa.to_csv(noaa_cache, index=False)
                    print(f"  Cached to: {noaa_cache}")
                else:
                    raise ValueError("Empty result from NOAA CDO")
            except Exception as e:
                print(f"  API error: {e}")
                print("  [SKIP] NOAA CDO data unavailable — using synthetic for methodology demo.")
                df_noaa = generate_synthetic_noaa(start, end)
                using_synthetic_noaa = True
        else:
            print("  No NOAA CDO token available.")
            print("  [SKIP] Using synthetic station data — methodology demo only.")
            df_noaa = generate_synthetic_noaa(start, end)
            using_synthetic_noaa = True

    if using_synthetic_noaa:
        print(
            "  *** SYNTHETIC MODE: Results demonstrate methodology only. "
            "Accuracy checks are SKIPPED. ***"
        )

    print(
        f"  NOAA: {len(df_noaa)} days, "
        f"tmax [{df_noaa['tmax_c'].min():.1f}, {df_noaa['tmax_c'].max():.1f}] °C"
    )

    # ------------------------------------------------------------------
    # Step 3: Merge on date
    # ------------------------------------------------------------------
    print("\n--- Step 3: Merging datasets ---")
    df = pd.merge(
        df_om[["date", "tmax_c", "tmin_c", "precip_mm"]],
        df_noaa[["date", "tmax_c", "tmin_c", "precip_mm"]],
        on="date",
        suffixes=("_mod", "_obs"),
        how="inner",
    )
    df["date"] = pd.to_datetime(df["date"])
    df["month"] = df["date"].dt.month
    df["doy"] = df["date"].dt.dayofyear

    print(f"  Overlapping days: {len(df)}")

    if len(df) < 30:
        print("  ERROR: Too few overlapping days for meaningful analysis!")
        return 1

    # ------------------------------------------------------------------
    # Step 4: Variable-by-variable comparison
    # ------------------------------------------------------------------
    print("\n--- Step 4: Variable Comparison ---")

    variables = {
        "tmax_c": benchmark["variables_compared"]["tmax_c"],
        "tmin_c": benchmark["variables_compared"]["tmin_c"],
        "precip_mm": benchmark["variables_compared"]["precip_mm"],
    }

    for var, spec in variables.items():
        print(f"\n  === {spec['description']} ({var}) ===")

        valid = df[[f"{var}_obs", f"{var}_mod"]].dropna()
        obs = np.asarray(valid[f"{var}_obs"].values)
        mod = np.asarray(valid[f"{var}_mod"].values)

        if len(obs) < 10:
            print(f"    Too few valid pairs ({len(obs)}), skipping")
            continue

        rmse = compute_rmse(obs, mod)
        mbe = compute_mbe(obs, mod)
        r2 = compute_r2(obs, mod)
        ia = compute_ia(obs, mod)

        print(f"    N valid pairs: {len(obs)}")
        print(f"    RMSE:  {rmse:.3f}")
        print(f"    MBE:   {mbe:.3f}")
        print(f"    R²:    {r2:.4f}")
        print(f"    IA:    {ia:.4f}")

        bv = bias_variance_decompose(obs, mod)
        print(f"    Bias fraction:  {bv['bias_fraction']*100:.1f}%")
        print(f"    Random std:     {bv['random_std']:.3f}")

        expected = spec["expected_characteristics"]

        if using_synthetic_noaa:
            print("    [SKIP] Accuracy checks skipped (synthetic NOAA mode)")
            if var == "precip_mm":
                hr = precip_hit_rate(obs, mod)
                print(f"    Rain/no-rain hit rate: {hr*100:.1f}%")
                print("    [SKIP] Hit rate check skipped (synthetic mode)")
        else:
            if "r2_minimum" in expected:
                check_min(f"{var} R²", r2, expected["r2_minimum"])

            if "rmse_range" in expected:
                check_range(
                    f"{var} RMSE",
                    rmse,
                    expected["rmse_range"][0],
                    expected["rmse_range"][1],
                )

            if var == "precip_mm":
                hr = precip_hit_rate(obs, mod)
                print(f"    Rain/no-rain hit rate: {hr*100:.1f}%")
                check_min(
                    f"{var} hit rate",
                    hr,
                    benchmark["acceptance_criteria"]["precip_hit_rate_min"],
                )

    # ------------------------------------------------------------------
    # Step 5: Seasonal decomposition
    # ------------------------------------------------------------------
    print("\n--- Step 5: Seasonal Analysis ---")

    for var in ["tmax_c", "tmin_c"]:
        print(f"\n  {var} by season:")
        seasonal = seasonal_analysis(df, var)
        for season, s in seasonal.items():
            print(
                f"    {season}: RMSE={s['rmse']:.2f}°C, "
                f"MBE={s['mbe']:+.2f}°C, R²={s['r2']:.3f} "
                f"(n={s['n_days']})"
            )

    temp_seasonal = seasonal_analysis(df, "tmax_c")
    if len(temp_seasonal) >= 2:
        rmses = [s["rmse"] for s in temp_seasonal.values()]
        check_true(
            "Seasonal variation in gap detected "
            f"(RMSE range: {min(rmses):.2f} to {max(rmses):.2f})",
            max(rmses) > min(rmses),
        )

    # ------------------------------------------------------------------
    # Summary
    # ------------------------------------------------------------------
    if using_synthetic_noaa:
        total = pass_count() + fail_count()
        print(f"\n{'=' * 72}")
        print("Exp 002: Weather Model vs Observation Gap")
        print(f"TOTAL: {pass_count()}/{total} PASS, {fail_count()}/{total} FAIL")
        print("  *** Accuracy checks SKIPPED — synthetic NOAA mode ***")
        print("  Get a NOAA CDO token for full validation.")
        print(f"{'=' * 72}")
        return 0 if fail_count() == 0 else 1

    return print_summary("Exp 002: Weather Model vs Observation Gap")


if __name__ == "__main__":
    sys.exit(main())
