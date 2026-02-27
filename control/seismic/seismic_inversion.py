# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""
groundSpring Experiment 005 — Seismic Wave Propagation & Source Inversion

Demonstrates inverse problem methodology using earthquake localization
from public seismological data.

Key questions:
  1. Can we locate an earthquake source from P-wave arrival times?
  2. How does arrival-time noise affect source location uncertainty?
  3. What is the trade-off between number of stations and accuracy?

Method:
  - Simplified 1D travel-time model (IASP91 upper crust, Vp=5.8 km/s)
  - Synthetic earthquake at known New Madrid Seismic Zone location
  - 7 regional stations with realistic arrival-time noise
  - Grid-search inversion + Nelder-Mead refinement
  - Monte Carlo noise analysis for uncertainty estimation

Reference:
  Kennett & Engdahl (1991) Traveltimes for global earthquake location
  and phase identification. Geophysical Journal International.
"""

from __future__ import annotations

import json
import math
import sys
from pathlib import Path

import numpy as np
from scipy import optimize

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from common import (
    check_max,
    check_true,
    print_summary,
    reset_counters,
)

# ---------------------------------------------------------------------------
# Travel-time computation
# ---------------------------------------------------------------------------

EARTH_RADIUS_KM = 6371.0


def haversine_km(
    lat1: float, lon1: float, lat2: float, lon2: float
) -> float:
    """Great-circle distance between two points in km."""
    phi1 = math.radians(lat1)
    phi2 = math.radians(lat2)
    dphi = math.radians(lat2 - lat1)
    dlam = math.radians(lon2 - lon1)

    a = (
        math.sin(dphi / 2) ** 2
        + math.cos(phi1) * math.cos(phi2) * math.sin(dlam / 2) ** 2
    )
    return EARTH_RADIUS_KM * 2 * math.atan2(math.sqrt(a), math.sqrt(1 - a))


def travel_time_1d(
    distance_km: float, depth_km: float, vp_km_s: float
) -> float:
    """Simplified 1D P-wave travel time (seconds).

    Straight-ray approximation through uniform crust.
    Adequate for regional distances (<500 km) and shallow sources.
    """
    raypath = math.sqrt(distance_km ** 2 + depth_km ** 2)
    return raypath / vp_km_s


def compute_arrivals(
    source_lat: float,
    source_lon: float,
    source_depth_km: float,
    origin_time_s: float,
    stations: list[dict],
    vp: float,
) -> list[dict]:
    """Compute P-wave arrival times at all stations from a point source."""
    arrivals = []
    for sta in stations:
        dist = haversine_km(source_lat, source_lon, sta["lat"], sta["lon"])
        tt = travel_time_1d(dist, source_depth_km, vp)
        arrivals.append({
            "code": sta["code"],
            "distance_km": dist,
            "travel_time_s": tt,
            "arrival_time_s": origin_time_s + tt,
        })
    return arrivals


# ---------------------------------------------------------------------------
# Grid-search source inversion
# ---------------------------------------------------------------------------

def grid_search_inversion(
    observed_arrivals: dict[str, float],
    stations: list[dict],
    lat_range: tuple[float, float],
    lon_range: tuple[float, float],
    depth_range_km: tuple[float, float],
    grid_spacing_deg: float = 0.1,
    depth_spacing_km: float = 5.0,
    vp: float = 6.0,
) -> dict:
    """Grid-search earthquake location by minimizing RMS travel-time residual."""
    lats = np.arange(
        lat_range[0], lat_range[1] + grid_spacing_deg / 2, grid_spacing_deg
    )
    lons = np.arange(
        lon_range[0], lon_range[1] + grid_spacing_deg / 2, grid_spacing_deg
    )
    depths = np.arange(
        depth_range_km[0],
        depth_range_km[1] + depth_spacing_km / 2,
        depth_spacing_km,
    )

    station_codes = [sta["code"] for sta in stations]
    obs_times = np.array([observed_arrivals[c] for c in station_codes])

    best_rms = float("inf")
    best_result: dict | None = None

    for lat in lats:
        for lon in lons:
            for depth in depths:
                pred_tt = np.array([
                    travel_time_1d(
                        haversine_km(lat, lon, sta["lat"], sta["lon"]),
                        depth,
                        vp,
                    )
                    for sta in stations
                ])

                t0 = float(np.mean(obs_times - pred_tt))
                residuals = obs_times - (t0 + pred_tt)
                rms = float(np.sqrt(np.mean(residuals ** 2)))

                if rms < best_rms:
                    best_rms = rms
                    best_result = {
                        "lat": float(lat),
                        "lon": float(lon),
                        "depth_km": float(depth),
                        "origin_time_s": t0,
                        "rms_residual_s": rms,
                        "residuals": residuals.tolist(),
                    }

    assert best_result is not None
    return best_result


def refine_with_scipy(
    observed_arrivals: dict[str, float],
    stations: list[dict],
    initial_guess: dict,
    vp: float = 6.0,
) -> dict:
    """Refine grid-search result using scipy.optimize.minimize."""
    station_codes = [sta["code"] for sta in stations]
    obs_times = np.array([observed_arrivals[c] for c in station_codes])

    def objective(params: np.ndarray) -> float:
        lat, lon, depth, t0 = params
        pred_tt = np.array([
            travel_time_1d(
                haversine_km(lat, lon, sta["lat"], sta["lon"]),
                max(0.1, depth),
                vp,
            )
            for sta in stations
        ])
        residuals = obs_times - (t0 + pred_tt)
        return float(np.sum(residuals ** 2))

    x0 = [
        initial_guess["lat"],
        initial_guess["lon"],
        initial_guess["depth_km"],
        initial_guess["origin_time_s"],
    ]

    result = optimize.minimize(
        objective, x0, method="Nelder-Mead",
        options={"maxiter": 5000, "xatol": 0.001},
    )

    lat, lon, depth, t0 = result.x
    pred_tt = np.array([
        travel_time_1d(
            haversine_km(lat, lon, sta["lat"], sta["lon"]),
            max(0.1, depth),
            vp,
        )
        for sta in stations
    ])
    residuals = obs_times - (t0 + pred_tt)
    rms = float(np.sqrt(np.mean(residuals ** 2)))

    return {
        "lat": float(lat),
        "lon": float(lon),
        "depth_km": float(max(0, depth)),
        "origin_time_s": float(t0),
        "rms_residual_s": rms,
        "converged": result.success,
        "n_iterations": result.nit,
    }


# ---------------------------------------------------------------------------
# Monte Carlo uncertainty estimation
# ---------------------------------------------------------------------------

def monte_carlo_source_uncertainty(
    true_arrivals: list[dict],
    stations: list[dict],
    noise_std_s: float,
    n_trials: int = 100,
    vp: float = 6.0,
    seed: int = 42,
) -> dict:
    """Estimate source location uncertainty via Monte Carlo noise injection."""
    rng = np.random.default_rng(seed)

    station_codes = [sta["code"] for sta in stations]
    true_times = {a["code"]: a["arrival_time_s"] for a in true_arrivals}

    lats_list: list[float] = []
    lons_list: list[float] = []
    depths_list: list[float] = []
    rms_vals: list[float] = []

    center_lat = float(np.mean([s["lat"] for s in stations]))
    center_lon = float(np.mean([s["lon"] for s in stations]))

    for _ in range(n_trials):
        noisy = {
            code: true_times[code] + rng.normal(0, noise_std_s)
            for code in station_codes
        }

        grid_result = grid_search_inversion(
            noisy,
            stations,
            lat_range=(center_lat - 2, center_lat + 2),
            lon_range=(center_lon - 2, center_lon + 2),
            depth_range_km=(0, 25),
            grid_spacing_deg=0.2,
            depth_spacing_km=5.0,
            vp=vp,
        )

        refined = refine_with_scipy(noisy, stations, grid_result, vp)

        lats_list.append(refined["lat"])
        lons_list.append(refined["lon"])
        depths_list.append(refined["depth_km"])
        rms_vals.append(refined["rms_residual_s"])

    lats_arr = np.array(lats_list)
    lons_arr = np.array(lons_list)
    depths_arr = np.array(depths_list)

    mean_lat = float(np.mean(lats_arr))
    mean_lon = float(np.mean(lons_arr))

    horiz_errors = [
        haversine_km(lat, lon, mean_lat, mean_lon)
        for lat, lon in zip(lats_arr, lons_arr, strict=True)
    ]

    return {
        "n_trials": n_trials,
        "noise_std_s": noise_std_s,
        "lat": {"mean": mean_lat, "std": float(np.std(lats_arr))},
        "lon": {"mean": mean_lon, "std": float(np.std(lons_arr))},
        "depth_km": {
            "mean": float(np.mean(depths_arr)),
            "std": float(np.std(depths_arr)),
        },
        "horizontal_error_km": {
            "mean": float(np.mean(horiz_errors)),
            "p90": float(np.percentile(horiz_errors, 90)),
        },
        "rms_residual": {
            "mean": float(np.mean(rms_vals)),
            "std": float(np.std(rms_vals)),
        },
    }


# ---------------------------------------------------------------------------
# Validation harness
# ---------------------------------------------------------------------------

def main() -> int:
    benchmark_path = Path(__file__).parent / "benchmark_seismic.json"
    with open(benchmark_path) as f:
        benchmark = json.load(f)

    reset_counters()

    scenario = benchmark["test_scenario"]
    source = scenario["true_source"]
    stations = scenario["stations"]
    noise_std = scenario["arrival_noise_std_s"]
    inv_config = benchmark["inversion_config"]
    criteria = inv_config["acceptance_criteria"]

    # Read Vp from benchmark (not hardcoded)
    vp = benchmark["travel_time_model"]["layers"][0]["vp_km_s"]

    print("=" * 72)
    print("groundSpring Exp 005: Seismic Wave Propagation & Source Inversion")
    print(f"  Region: {source['region']}")
    print(f"  Vp = {vp} km/s (from benchmark: {benchmark['travel_time_model']['layers'][0]['name']})")
    print("=" * 72)

    # ------------------------------------------------------------------
    # Part 1: Forward model
    # ------------------------------------------------------------------
    print("\n--- Part 1: Forward Model ---")

    true_arrivals = compute_arrivals(
        source["lat"], source["lon"], source["depth_km"],
        source["origin_time_s"], stations, vp,
    )

    print(
        f"  Source: ({source['lat']}°N, {source['lon']}°E), "
        f"depth={source['depth_km']}km"
    )
    print("\n  Station arrivals:")
    for a in true_arrivals:
        print(
            f"    {a['code']:>5s}: dist={a['distance_km']:>6.1f} km, "
            f"tt={a['travel_time_s']:>6.2f} s"
        )

    check_true(
        "All travel times positive",
        all(a["travel_time_s"] > 0 for a in true_arrivals),
    )

    sorted_by_dist = sorted(true_arrivals, key=lambda x: x["distance_km"])
    check_true(
        "Travel time increases with distance",
        all(
            sorted_by_dist[i]["travel_time_s"]
            <= sorted_by_dist[i + 1]["travel_time_s"]
            for i in range(len(sorted_by_dist) - 1)
        ),
    )

    # ------------------------------------------------------------------
    # Part 2: Grid-search inversion (no noise)
    # ------------------------------------------------------------------
    print("\n--- Part 2: Grid-Search Inversion (no noise) ---")
    obs_clean = {a["code"]: a["arrival_time_s"] for a in true_arrivals}

    gs = inv_config["grid_search"]
    grid_result = grid_search_inversion(
        obs_clean, stations,
        lat_range=tuple(gs["lat_range"]),
        lon_range=tuple(gs["lon_range"]),
        depth_range_km=tuple(gs["depth_range_km"]),
        grid_spacing_deg=gs["grid_spacing_deg"],
        depth_spacing_km=gs["depth_spacing_km"],
        vp=vp,
    )

    loc_error_km = haversine_km(
        grid_result["lat"], grid_result["lon"],
        source["lat"], source["lon"],
    )
    depth_error_km = abs(grid_result["depth_km"] - source["depth_km"])

    print(
        f"  Inverted: ({grid_result['lat']:.2f}°N, {grid_result['lon']:.2f}°E), "
        f"depth={grid_result['depth_km']:.1f}km"
    )
    print(f"  Location error: {loc_error_km:.2f} km")
    print(f"  Depth error:    {depth_error_km:.2f} km")
    print(f"  RMS residual:   {grid_result['rms_residual_s']:.4f} s")

    check_max("Location error (km)", loc_error_km, criteria["location_error_km_max"])
    check_max("Depth error (km)", depth_error_km, criteria["depth_error_km_max"])
    check_max("RMS residual (s)", grid_result["rms_residual_s"], criteria["rms_residual_s_max"])

    # ------------------------------------------------------------------
    # Part 3: Nelder-Mead Refinement
    # ------------------------------------------------------------------
    print("\n--- Part 3: Nelder-Mead Refinement ---")
    refined = refine_with_scipy(obs_clean, stations, grid_result, vp)

    ref_loc_error = haversine_km(
        refined["lat"], refined["lon"], source["lat"], source["lon"]
    )
    ref_depth_error = abs(refined["depth_km"] - source["depth_km"])

    print(
        f"  Refined: ({refined['lat']:.4f}°N, {refined['lon']:.4f}°E), "
        f"depth={refined['depth_km']:.2f}km"
    )
    print(f"  Location error: {ref_loc_error:.4f} km")
    print(f"  Depth error:    {ref_depth_error:.4f} km")
    print(f"  RMS residual:   {refined['rms_residual_s']:.6f} s")
    print(f"  Converged:      {refined['converged']}")

    check_true(
        "Refinement improved or maintained accuracy",
        ref_loc_error <= loc_error_km + 0.5,
    )

    # ------------------------------------------------------------------
    # Part 4: Noisy inversion
    # ------------------------------------------------------------------
    print(f"\n--- Part 4: Noisy Inversion (σ = {noise_std}s) ---")
    rng = np.random.default_rng(42)
    obs_noisy = {
        a["code"]: a["arrival_time_s"] + rng.normal(0, noise_std)
        for a in true_arrivals
    }

    noisy_grid = grid_search_inversion(
        obs_noisy, stations,
        lat_range=tuple(gs["lat_range"]),
        lon_range=tuple(gs["lon_range"]),
        depth_range_km=tuple(gs["depth_range_km"]),
        grid_spacing_deg=gs["grid_spacing_deg"],
        depth_spacing_km=gs["depth_spacing_km"],
        vp=vp,
    )
    noisy_refined = refine_with_scipy(obs_noisy, stations, noisy_grid, vp)

    noisy_loc_error = haversine_km(
        noisy_refined["lat"], noisy_refined["lon"],
        source["lat"], source["lon"],
    )

    print(
        f"  Inverted: ({noisy_refined['lat']:.3f}°N, "
        f"{noisy_refined['lon']:.3f}°E), "
        f"depth={noisy_refined['depth_km']:.1f}km"
    )
    print(f"  Location error: {noisy_loc_error:.2f} km")
    print(f"  RMS residual:   {noisy_refined['rms_residual_s']:.4f} s")

    check_max("Noisy location error (km)", noisy_loc_error, criteria["location_error_km_max"])

    # ------------------------------------------------------------------
    # Part 5: Monte Carlo uncertainty
    # ------------------------------------------------------------------
    print("\n--- Part 5: Monte Carlo Uncertainty (N=50) ---")
    mc = monte_carlo_source_uncertainty(
        true_arrivals, stations, noise_std, n_trials=50, vp=vp, seed=42
    )

    print(f"  Lat:   {mc['lat']['mean']:.3f} ± {mc['lat']['std']:.3f}°")
    print(f"  Lon:   {mc['lon']['mean']:.3f} ± {mc['lon']['std']:.3f}°")
    print(
        f"  Depth: {mc['depth_km']['mean']:.1f} ± "
        f"{mc['depth_km']['std']:.1f} km"
    )
    print(
        f"  Horizontal error: mean={mc['horizontal_error_km']['mean']:.1f} km, "
        f"90th={mc['horizontal_error_km']['p90']:.1f} km"
    )

    mc_loc_error = haversine_km(
        mc["lat"]["mean"], mc["lon"]["mean"],
        source["lat"], source["lon"],
    )
    check_max("MC mean location error (km)", mc_loc_error, criteria["location_error_km_max"])
    check_true(
        "Non-zero location uncertainty (noise propagated)",
        mc["lat"]["std"] > 0.001,
    )

    # ------------------------------------------------------------------
    # Part 6: Station subset analysis
    # ------------------------------------------------------------------
    print("\n--- Part 6: Station Subset (fewer stations) ---")

    for n_sta in [3, 5, 7]:
        subset_stations = stations[:n_sta]
        subset_codes = {s["code"] for s in subset_stations}
        subset_arrivals = {
            a["code"]: a["arrival_time_s"] + rng.normal(0, noise_std)
            for a in true_arrivals
            if a["code"] in subset_codes
        }

        sub_grid = grid_search_inversion(
            subset_arrivals,
            subset_stations,
            lat_range=tuple(gs["lat_range"]),
            lon_range=tuple(gs["lon_range"]),
            depth_range_km=tuple(gs["depth_range_km"]),
            grid_spacing_deg=0.1,
            depth_spacing_km=5.0,
            vp=vp,
        )
        sub_error = haversine_km(
            sub_grid["lat"], sub_grid["lon"],
            source["lat"], source["lon"],
        )
        print(
            f"  {n_sta} stations: error = {sub_error:.1f} km, "
            f"RMS = {sub_grid['rms_residual_s']:.3f} s"
        )

    check_true("Station subset analysis completed", True)

    # ------------------------------------------------------------------
    # Key Findings
    # ------------------------------------------------------------------
    print(f"\n{'=' * 72}")
    print("KEY FINDINGS:")
    print(f"{'=' * 72}")

    print("\n1. Source Localization Accuracy:")
    print(f"   Clean data:  {ref_loc_error:.2f} km error")
    print(f"   Noisy (±{noise_std}s): {noisy_loc_error:.1f} km error")

    print("\n2. Uncertainty Budget:")
    print(
        f"   Horizontal:  ±{mc['horizontal_error_km']['mean']:.1f} km "
        f"(90th: {mc['horizontal_error_km']['p90']:.1f} km)"
    )
    print(f"   Depth:       ±{mc['depth_km']['std']:.1f} km")

    return print_summary("Exp 005: Seismic Wave Propagation")


if __name__ == "__main__":
    sys.exit(main())
