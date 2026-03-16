#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
"""
Download seismic data from IRIS Web Services.

IRIS (Incorporated Research Institutions for Seismology) provides:
  - Free, public access to seismic waveform and metadata
  - No API key required
  - Decades of global station data
  - Multiple data formats (miniSEED, SAC, etc.)

This script downloads station metadata and (optionally) waveform data
for stations near the New Madrid Seismic Zone (NMSZ).

Usage:
    python3 scripts/download_iris.py --stations
    python3 scripts/download_iris.py --event 2023-01-01 --magnitude 3.0

Output:
    data/iris/stations_nmsz.csv
    data/iris/event_*.mseed (if waveforms requested)
"""

import argparse
from pathlib import Path

import pandas as pd
import requests

IRIS_STATION_URL = "https://service.iris.edu/fdsnws/station/1/query"
IRIS_EVENT_URL = "https://service.iris.edu/fdsnws/event/1/query"


def fetch_stations(min_lat: float, max_lat: float,
                    min_lon: float, max_lon: float,
                    network: str = "*",
                    channel: str = "BH?") -> pd.DataFrame:
    """
    Fetch station metadata from IRIS FDSN Station web service.

    Returns DataFrame of stations within the bounding box.
    """
    params = {
        "format": "text",
        "level": "station",
        "minlatitude": min_lat,
        "maxlatitude": max_lat,
        "minlongitude": min_lon,
        "maxlongitude": max_lon,
        "channel": channel,
    }

    if network != "*":
        params["network"] = network

    resp = requests.get(IRIS_STATION_URL, params=params, timeout=30)
    if resp.status_code != 200:
        print(f"IRIS station API error: {resp.status_code}")
        print(f"Response: {resp.text[:500]}")
        return pd.DataFrame()

    lines = resp.text.strip().split("\n")
    if len(lines) < 2:
        return pd.DataFrame()

    header = lines[0].lstrip("#").strip().split("|")
    data = []
    for line in lines[1:]:
        if line.startswith("#"):
            continue
        fields = line.strip().split("|")
        if len(fields) >= len(header):
            data.append(fields[:len(header)])

    df = pd.DataFrame(data, columns=header)

    # Clean up column names
    df.columns = [c.strip() for c in df.columns]

    # Convert numeric columns
    for col in ["Latitude", "Longitude", "Elevation"]:
        if col in df.columns:
            df[col] = pd.to_numeric(df[col], errors="coerce")

    return df


def fetch_events(min_lat: float, max_lat: float,
                  min_lon: float, max_lon: float,
                  start: str, end: str,
                  min_mag: float = 2.5) -> pd.DataFrame:
    """Fetch earthquake events from IRIS FDSN Event web service."""
    params = {
        "format": "text",
        "minlatitude": min_lat,
        "maxlatitude": max_lat,
        "minlongitude": min_lon,
        "maxlongitude": max_lon,
        "starttime": start,
        "endtime": end,
        "minmagnitude": min_mag,
        "orderby": "magnitude",
    }

    resp = requests.get(IRIS_EVENT_URL, params=params, timeout=30)
    if resp.status_code != 200:
        print(f"IRIS event API error: {resp.status_code}")
        return pd.DataFrame()

    lines = resp.text.strip().split("\n")
    if len(lines) < 2:
        return pd.DataFrame()

    header = lines[0].lstrip("#").strip().split("|")
    data = []
    for line in lines[1:]:
        if line.startswith("#"):
            continue
        fields = line.strip().split("|")
        if len(fields) >= len(header):
            data.append(fields[:len(header)])

    return pd.DataFrame(data, columns=[c.strip() for c in header])


def main():
    parser = argparse.ArgumentParser(
        description="Download seismic data from IRIS Web Services (free, public)")
    parser.add_argument("--stations", action="store_true",
                        help="Download station metadata for NMSZ region")
    parser.add_argument("--events", action="store_true",
                        help="Download earthquake events")
    parser.add_argument("--start", default="2023-01-01",
                        help="Start date for events")
    parser.add_argument("--end", default="2024-01-01",
                        help="End date for events")
    parser.add_argument("--min-mag", type=float, default=2.5,
                        help="Minimum magnitude for events")
    args = parser.parse_args()

    out_dir = Path(__file__).parent.parent / "data" / "iris"
    out_dir.mkdir(parents=True, exist_ok=True)

    # New Madrid Seismic Zone bounding box
    min_lat, max_lat = 34.0, 40.0
    min_lon, max_lon = -92.0, -85.0

    print("=" * 65)
    print("  groundSpring — IRIS Seismic Data Download")
    print("=" * 65)
    print(f"  Region: NMSZ ({min_lat}-{max_lat}°N, {min_lon}-{max_lon}°E)")

    if args.stations or (not args.events):
        print("\n--- Station Metadata ---")
        df_stations = fetch_stations(min_lat, max_lat, min_lon, max_lon)

        if not df_stations.empty:
            sta_path = out_dir / "stations_nmsz.csv"
            df_stations.to_csv(sta_path, index=False)
            print(f"  Stations found: {len(df_stations)}")
            print(f"  Saved: {sta_path}")

            if "Station" in df_stations.columns:
                print("\n  Sample stations:")
                for _, row in df_stations.head(10).iterrows():
                    print(f"    {row.get('Station', '?'):>6s} | "
                          f"{row.get('Network', '?'):>3s} | "
                          f"{row.get('Latitude', 0):>8.3f} "
                          f"{row.get('Longitude', 0):>9.3f} | "
                          f"{row.get('SiteName', '?')}")
        else:
            print("  No stations returned (API may be temporarily unavailable)")

    if args.events:
        print("\n--- Earthquake Events ---")
        print(f"  Period: {args.start} to {args.end}, min mag: {args.min_mag}")

        df_events = fetch_events(min_lat, max_lat, min_lon, max_lon,
                                  args.start, args.end, args.min_mag)

        if not df_events.empty:
            evt_path = out_dir / f"events_{args.start}_{args.end}.csv"
            df_events.to_csv(evt_path, index=False)
            print(f"  Events found: {len(df_events)}")
            print(f"  Saved: {evt_path}")
        else:
            print("  No events found in this region/period")


if __name__ == "__main__":
    main()
