// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Validation binary for seismic travel-time computation and source inversion.
//!
//! Station coordinates, velocity model, and acceptance criteria loaded from
//! the benchmark JSON — single source of truth with full provenance.
//!
//! Reference: Kennett & Engdahl (1991) IASP91, Geophysical Journal International.

use groundspring::seismic::{
    grid_search_inversion, haversine_km, travel_time_1d, GridSearchConfig, Station,
};
use groundspring::validate::ValidationHarness;
use serde_json::Value;

const BENCHMARK: &str = include_str!("../../../control/seismic/benchmark_seismic.json");

fn f64_field(v: &Value, key: &str) -> f64 {
    v[key]
        .as_f64()
        .unwrap_or_else(|| panic!("missing f64 field: {key}"))
}

#[allow(clippy::too_many_lines)]
fn main() {
    let bench: Value = serde_json::from_str(BENCHMARK).expect("valid benchmark JSON");
    let mut h = ValidationHarness::stdout("Rust Validation: Seismic Inversion");

    println!("{}", "=".repeat(72));
    println!("groundSpring Rust Validation: Seismic Wave Propagation");
    println!(
        "  Source: {}",
        bench["_source"]
            .as_str()
            .unwrap_or("IASP91 synthetic scenario")
    );
    println!(
        "  Provenance: commit {}, {}",
        bench["_provenance"]["baseline_commit"]
            .as_str()
            .unwrap_or("unknown"),
        bench["_provenance"]["baseline_date"]
            .as_str()
            .unwrap_or("unknown"),
    );
    println!("{}", "=".repeat(72));

    let vp = f64_field(&bench["travel_time_model"]["layers"][0], "vp_km_s");

    let src = &bench["test_scenario"]["true_source"];
    let true_lat = f64_field(src, "lat");
    let true_lon = f64_field(src, "lon");
    let true_depth = f64_field(src, "depth_km");

    let stations: Vec<Station> = bench["test_scenario"]["stations"]
        .as_array()
        .expect("stations array")
        .iter()
        .map(|s| Station {
            code: s["code"].as_str().expect("station code").into(),
            lat: f64_field(s, "lat"),
            lon: f64_field(s, "lon"),
        })
        .collect();

    let criteria = &bench["inversion_config"]["acceptance_criteria"];
    let max_loc_error = f64_field(criteria, "location_error_km_max");
    let max_depth_error = f64_field(criteria, "depth_error_km_max");
    let max_rms = f64_field(criteria, "rms_residual_s_max");

    let grid = &bench["inversion_config"]["grid_search"];

    // ── Haversine ───────────────────────────────────────────────────
    println!("\n--- Haversine Distance ---");

    h.check_approx(
        "Zero distance",
        haversine_km(true_lat, true_lon, true_lat, true_lon),
        0.0,
        1e-10,
    );

    let ny_london = haversine_km(40.7128, -74.0060, 51.5074, -0.1278);
    h.check_range("NY-London ~5570 km", ny_london, 5520.0, 5620.0);

    // ── Travel time ─────────────────────────────────────────────────
    println!("\n--- Travel Time ---");

    let tt_100 = travel_time_1d(100.0, 0.0, 6.0);
    h.check_approx("100km/6.0km/s = 16.667s", tt_100, 100.0 / 6.0, 1e-10);

    let t1 = travel_time_1d(100.0, 10.0, vp);
    let t2 = travel_time_1d(200.0, 10.0, vp);
    h.check_true("Travel time increases with distance", t2 > t1);

    // ── Forward model ───────────────────────────────────────────────
    println!("\n--- Forward Model ---");

    let observed: Vec<(String, f64)> = stations
        .iter()
        .map(|s| {
            let dist = haversine_km(true_lat, true_lon, s.lat, s.lon);
            let tt = travel_time_1d(dist, true_depth, vp);
            (s.code.clone(), tt)
        })
        .collect();

    h.check_true(
        "All travel times positive",
        observed.iter().all(|(_, t)| *t > 0.0),
    );

    let mut by_dist: Vec<(f64, f64)> = stations
        .iter()
        .zip(&observed)
        .map(|(s, (_, t))| (haversine_km(true_lat, true_lon, s.lat, s.lon), *t))
        .collect();
    by_dist.sort_by(|a, b| a.0.total_cmp(&b.0));
    h.check_true(
        "Travel time monotonic with distance",
        by_dist.windows(2).all(|w| w[0].1 <= w[1].1),
    );

    // ── Grid-search inversion ───────────────────────────────────────
    println!("\n--- Grid-Search Inversion (no noise) ---");

    let lat_range = grid["lat_range"].as_array().expect("lat_range");
    let lon_range = grid["lon_range"].as_array().expect("lon_range");
    let depth_range = grid["depth_range_km"].as_array().expect("depth_range_km");

    let config = GridSearchConfig {
        lat_range: (
            lat_range[0].as_f64().expect("lat_min"),
            lat_range[1].as_f64().expect("lat_max"),
        ),
        lon_range: (
            lon_range[0].as_f64().expect("lon_min"),
            lon_range[1].as_f64().expect("lon_max"),
        ),
        depth_range: (
            depth_range[0].as_f64().expect("depth_min"),
            depth_range[1].as_f64().expect("depth_max"),
        ),
        grid_spacing_deg: f64_field(grid, "grid_spacing_deg"),
        depth_spacing_km: f64_field(grid, "depth_spacing_km"),
        vp,
    };
    let result = grid_search_inversion(&observed, &stations, &config);

    let loc_error = haversine_km(result.lat, result.lon, true_lat, true_lon);
    let depth_error = (result.depth_km - true_depth).abs();

    println!(
        "  Inverted: ({:.2}°N, {:.2}°E), depth={:.1}km",
        result.lat, result.lon, result.depth_km
    );
    println!("  Location error: {loc_error:.2} km");
    println!("  Depth error:    {depth_error:.2} km");
    println!("  RMS residual:   {:.4} s", result.rms_residual_s);

    h.check_max("Location error (km)", loc_error, max_loc_error);
    h.check_max("Depth error (km)", depth_error, max_depth_error);
    h.check_max("RMS residual (s)", result.rms_residual_s, max_rms);

    let exit_code = h.summary();
    std::process::exit(exit_code);
}
