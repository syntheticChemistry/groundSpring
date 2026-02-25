// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Validation binary for seismic travel-time computation and source inversion.
//!
//! Hardcoded expected values from `benchmark_seismic.json`.
//! Provenance: IASP91 velocity model (Kennett & Engdahl 1991),
//! synthetic NMSZ scenario with Vp = 5.8 km/s upper crust.

use groundspring::seismic::{
    grid_search_inversion, haversine_km, travel_time_1d, GridSearchConfig, Station,
};
use groundspring::validate;

#[allow(clippy::too_many_lines)]
fn main() {
    validate::reset();

    println!("{}", "=".repeat(72));
    println!("groundSpring Rust Validation: Seismic Wave Propagation");
    println!("  Reference: Kennett & Engdahl (1991) IASP91");
    println!("{}", "=".repeat(72));

    // From benchmark_seismic.json
    let vp = 5.8; // upper crust Vp from IASP91
    let true_lat = 37.5;
    let true_lon = -89.0;
    let true_depth = 10.0;

    let stations = vec![
        Station {
            code: "WVT".into(),
            lat: 36.13,
            lon: -87.83,
        },
        Station {
            code: "CCM".into(),
            lat: 38.06,
            lon: -91.24,
        },
        Station {
            code: "SIUC".into(),
            lat: 37.71,
            lon: -89.22,
        },
        Station {
            code: "SLM".into(),
            lat: 38.63,
            lon: -90.24,
        },
        Station {
            code: "USIN".into(),
            lat: 37.97,
            lon: -87.35,
        },
        Station {
            code: "PLAL".into(),
            lat: 34.98,
            lon: -88.08,
        },
        Station {
            code: "WVT2".into(),
            lat: 38.23,
            lon: -86.29,
        },
    ];

    // ------------------------------------------------------------------
    // Haversine known values
    // ------------------------------------------------------------------
    println!("\n--- Haversine Distance ---");

    let _ = validate::check_approx(
        "Zero distance",
        haversine_km(37.5, -89.0, 37.5, -89.0),
        0.0,
        1e-10,
    );

    let ny_london = haversine_km(40.7128, -74.0060, 51.5074, -0.1278);
    let _ = validate::check_range("NY-London ~5570 km", ny_london, 5520.0, 5620.0);

    // ------------------------------------------------------------------
    // Travel time
    // ------------------------------------------------------------------
    println!("\n--- Travel Time ---");

    let tt_100 = travel_time_1d(100.0, 0.0, 6.0);
    let _ = validate::check_approx("100km/6.0km/s = 16.667s", tt_100, 100.0 / 6.0, 1e-10);

    // Travel time increases with distance
    let t1 = travel_time_1d(100.0, 10.0, vp);
    let t2 = travel_time_1d(200.0, 10.0, vp);
    let _ = validate::check_true("Travel time increases with distance", t2 > t1);

    // ------------------------------------------------------------------
    // Forward model
    // ------------------------------------------------------------------
    println!("\n--- Forward Model ---");

    let observed: Vec<(String, f64)> = stations
        .iter()
        .map(|s| {
            let dist = haversine_km(true_lat, true_lon, s.lat, s.lon);
            let tt = travel_time_1d(dist, true_depth, vp);
            (s.code.clone(), tt)
        })
        .collect();

    // All travel times positive
    let _ = validate::check_true(
        "All travel times positive",
        observed.iter().all(|(_, t)| *t > 0.0),
    );

    // Monotonic with distance (after sorting)
    let mut by_dist: Vec<(f64, f64)> = stations
        .iter()
        .zip(&observed)
        .map(|(s, (_, t))| (haversine_km(true_lat, true_lon, s.lat, s.lon), *t))
        .collect();
    by_dist.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let _ = validate::check_true(
        "Travel time monotonic with distance",
        by_dist.windows(2).all(|w| w[0].1 <= w[1].1),
    );

    // ------------------------------------------------------------------
    // Grid-search inversion (clean)
    // ------------------------------------------------------------------
    println!("\n--- Grid-Search Inversion (no noise) ---");

    let config = GridSearchConfig {
        lat_range: (35.0, 40.0),
        lon_range: (-92.0, -86.0),
        depth_range: (0.0, 30.0),
        grid_spacing_deg: 0.05,
        depth_spacing_km: 2.0,
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

    // Acceptance criteria from benchmark_seismic.json
    let _ = validate::check_max("Location error (km)", loc_error, 30.0);
    let _ = validate::check_max("Depth error (km)", depth_error, 15.0);
    let _ = validate::check_max("RMS residual (s)", result.rms_residual_s, 2.0);

    let exit_code = validate::summary("Rust Validation: Seismic Inversion");
    std::process::exit(exit_code);
}
