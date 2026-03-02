// SPDX-License-Identifier: AGPL-3.0-only
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
use groundspring_validate::{array_field, f64_field, print_provenance_header, TOL_ANALYTICAL};
use serde_json::Value;

const BENCHMARK: &str = include_str!("../../../control/seismic/benchmark_seismic.json");

/// Ground-truth source location for inversion validation.
struct SourceTruth {
    lat: f64,
    lon: f64,
    depth_km: f64,
}

/// Acceptance thresholds loaded from the benchmark JSON.
struct AcceptanceCriteria {
    location_error_km: f64,
    depth_error_km: f64,
    rms_residual_s: f64,
}

/// Forward-model checks: travel times are positive and monotonic.
fn validate_forward_model<'a>(
    h: &mut ValidationHarness,
    stations: &'a [Station],
    truth: &SourceTruth,
    vp: f64,
) -> Vec<(&'a str, f64)> {
    println!("\n--- Forward Model ---");

    let observed: Vec<(&str, f64)> = stations
        .iter()
        .map(|s| {
            let dist = haversine_km(truth.lat, truth.lon, s.lat, s.lon);
            let tt = travel_time_1d(dist, truth.depth_km, vp);
            (s.code.as_str(), tt)
        })
        .collect();

    h.check_true(
        "All travel times positive",
        observed.iter().all(|(_, t)| *t > 0.0),
    );

    let mut by_dist: Vec<(f64, f64)> = stations
        .iter()
        .zip(&observed)
        .map(|(s, (_, t))| (haversine_km(truth.lat, truth.lon, s.lat, s.lon), *t))
        .collect();
    by_dist.sort_by(|a, b| a.0.total_cmp(&b.0));
    h.check_true(
        "Travel time monotonic with distance",
        by_dist.windows(2).all(|w| w[0].1 <= w[1].1),
    );

    observed
}

/// Grid-search inversion and error checks.
fn validate_inversion(
    h: &mut ValidationHarness,
    observed: &[(&str, f64)],
    stations: &[Station],
    grid: &Value,
    vp: f64,
    truth: &SourceTruth,
    criteria: &AcceptanceCriteria,
) {
    println!("\n--- Grid-Search Inversion (no noise) ---");

    let lat_range = array_field(grid, "lat_range");
    let lon_range = array_field(grid, "lon_range");
    let depth_range = array_field(grid, "depth_range_km");

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
    let result = grid_search_inversion(observed, stations, &config);

    let loc_error = haversine_km(result.lat, result.lon, truth.lat, truth.lon);
    let depth_error = (result.depth_km - truth.depth_km).abs();

    println!(
        "  Inverted: ({:.2}°N, {:.2}°E), depth={:.1}km",
        result.lat, result.lon, result.depth_km
    );
    println!("  Location error: {loc_error:.2} km");
    println!("  Depth error:    {depth_error:.2} km");
    println!("  RMS residual:   {:.4} s", result.rms_residual_s);

    h.check_max("Location error (km)", loc_error, criteria.location_error_km);
    h.check_max("Depth error (km)", depth_error, criteria.depth_error_km);
    h.check_max(
        "RMS residual (s)",
        result.rms_residual_s,
        criteria.rms_residual_s,
    );
}

fn run() -> i32 {
    let bench: Value = serde_json::from_str(BENCHMARK).expect("valid benchmark JSON");
    let mut h = ValidationHarness::stdout("Rust Validation: Seismic Inversion");

    print_provenance_header(&bench, "Seismic Wave Propagation");

    let vp = f64_field(&bench["travel_time_model"]["layers"][0], "vp_km_s");

    let src = &bench["test_scenario"]["true_source"];
    let truth = SourceTruth {
        lat: f64_field(src, "lat"),
        lon: f64_field(src, "lon"),
        depth_km: f64_field(src, "depth_km"),
    };

    let stations: Vec<Station> = array_field(&bench["test_scenario"], "stations")
        .iter()
        .map(|s| Station {
            code: s["code"].as_str().expect("station code").into(),
            lat: f64_field(s, "lat"),
            lon: f64_field(s, "lon"),
        })
        .collect();

    let criteria_json = &bench["inversion_config"]["acceptance_criteria"];
    let criteria = AcceptanceCriteria {
        location_error_km: f64_field(criteria_json, "location_error_km_max"),
        depth_error_km: f64_field(criteria_json, "depth_error_km_max"),
        rms_residual_s: f64_field(criteria_json, "rms_residual_s_max"),
    };

    let grid = &bench["inversion_config"]["grid_search"];

    // ── Haversine ───────────────────────────────────────────────────
    println!("\n--- Haversine Distance ---");

    h.check_approx(
        "Zero distance",
        haversine_km(truth.lat, truth.lon, truth.lat, truth.lon),
        0.0,
        TOL_ANALYTICAL,
    );

    let hav = &bench["haversine_reference"];
    let ny_london = haversine_km(
        f64_field(hav, "ny_lat"),
        f64_field(hav, "ny_lon"),
        f64_field(hav, "london_lat"),
        f64_field(hav, "london_lon"),
    );
    let (hav_lo, hav_hi) = groundspring_validate::f64_range(&hav["ny_london_range"]);
    h.check_range("NY-London ~5570 km", ny_london, hav_lo, hav_hi);

    // ── Travel time ─────────────────────────────────────────────────
    println!("\n--- Travel Time ---");

    let tt_100 = travel_time_1d(100.0, 0.0, 6.0);
    h.check_approx(
        "100km/6.0km/s = 16.667s",
        tt_100,
        100.0 / 6.0,
        TOL_ANALYTICAL,
    );

    let t1 = travel_time_1d(100.0, 10.0, vp);
    let t2 = travel_time_1d(200.0, 10.0, vp);
    h.check_true("Travel time increases with distance", t2 > t1);

    let observed = validate_forward_model(&mut h, &stations, &truth, vp);
    validate_inversion(&mut h, &observed, &stations, grid, vp, &truth, &criteria);

    h.summary()
}

fn main() {
    std::process::exit(run());
}

#[cfg(test)]
mod tests {
    #[test]
    fn validation_passes() {
        assert_eq!(super::run(), 0);
    }
}
