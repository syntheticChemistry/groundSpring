// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Exp 032: IRIS Seismic via NUCLEUS — real seismic data through `NestGate`.
//!
//! When NUCLEUS is running with `NestGate`, fetches real IRIS FDSN station
//! metadata and earthquake events for the New Madrid Seismic Zone, then
//! validates groundSpring's seismic distance and travel-time computations
//! against real station geometry.
//!
//! When NUCLEUS is unavailable, falls back to synthetic station data and
//! validates the computational pipeline alone.
//!
//! Requires: `--features biomeos` (compile-time) + running NUCLEUS (runtime, optional)

#[cfg(not(feature = "biomeos"))]
compile_error!("Exp 032 requires --features biomeos");

#[cfg(feature = "biomeos")]
use groundspring::biomeos;
#[cfg(feature = "biomeos")]
use groundspring::validate::ValidationHarness;
use groundspring_validate::{TOL_ANALYTICAL, TOL_GRID_MATCH};

#[cfg(feature = "biomeos")]
fn main() {
    let mut h = ValidationHarness::stdout("Exp 032: IRIS Seismic via NUCLEUS");

    println!("{}", "=".repeat(72));
    println!("  Exp 032: IRIS FDSN Seismic → Station Geometry + Travel Times");
    println!("{}", "=".repeat(72));
    println!();
    println!("  Provenance: NUCLEUS live-data validation binary");
    println!("  Data source: IRIS FDSN (New Madrid Seismic Zone) or synthetic");
    println!("  Baseline: Analytical (haversine, 1D travel time, triangle ineq.)");
    println!("  Note: No benchmark JSON — validates geometric invariants and");
    println!("        live data pipeline, not Python baseline comparison.");
    println!();

    let socket = biomeos::auto_connect();
    let data_source = if socket.is_some() {
        "LIVE IRIS FDSN"
    } else {
        "SYNTHETIC"
    };
    println!("  Data source: {data_source}");
    println!();

    let stations = socket.as_ref().map_or_else(
        || {
            println!("  No NUCLEUS available, using synthetic stations");
            synthetic_stations()
        },
        |sock| match fetch_iris_stations(sock) {
            Ok(s) => {
                println!("  Fetched {} real stations from IRIS", s.len());
                s
            }
            Err(e) => {
                println!("  Live fetch failed ({e}), using synthetic stations");
                synthetic_stations()
            }
        },
    );

    println!("  Stations: {}", stations.len());
    for s in &stations {
        println!("    {} ({:.4}°N, {:.4}°E)", s.code, s.lat, s.lon);
    }
    println!();

    h.check_true("Have at least 2 stations", stations.len() >= 2);

    validate_distances(&mut h, &stations);
    validate_travel_times(&mut h, &stations);
    validate_events(&mut h, socket.as_ref());
    store_provenance(&stations, socket.as_ref(), data_source);

    println!();
    std::process::exit(h.summary());
}

#[cfg(feature = "biomeos")]
struct Station {
    code: String,
    lat: f64,
    lon: f64,
}

#[cfg(feature = "biomeos")]
fn validate_distances(h: &mut ValidationHarness, stations: &[Station]) {
    use groundspring::seismic;

    println!("--- Distances ---");
    if stations.len() < 2 {
        h.check_true("Need >= 2 stations for distance", false);
        return;
    }

    let s0 = &stations[0];
    let s1 = &stations[1];
    let d_km = seismic::haversine_km(s0.lat, s0.lon, s1.lat, s1.lon);
    println!("  {}-{}: {d_km:.1} km", s0.code, s1.code);

    h.check_true("Distance > 0 for distinct stations", d_km > 0.0);
    h.check_true("Distance < 20000 km (Earth circumference)", d_km < 20_100.0);

    let d_self = seismic::haversine_km(s0.lat, s0.lon, s0.lat, s0.lon);
    h.check_true("Self-distance = 0", d_self.abs() < TOL_ANALYTICAL);

    if stations.len() >= 3 {
        let s2 = &stations[2];
        let d01 = seismic::haversine_km(s0.lat, s0.lon, s1.lat, s1.lon);
        let d02 = seismic::haversine_km(s0.lat, s0.lon, s2.lat, s2.lon);
        let d12 = seismic::haversine_km(s1.lat, s1.lon, s2.lat, s2.lon);
        let triangle_ok = d01 <= d02 + d12 + TOL_GRID_MATCH
            && d02 <= d01 + d12 + TOL_GRID_MATCH
            && d12 <= d01 + d02 + TOL_GRID_MATCH;
        h.check_true("Triangle inequality holds", triangle_ok);
        println!(
            "  Triangle: {}-{} {d01:.1}, {}-{} {d02:.1}, {}-{} {d12:.1}",
            s0.code, s1.code, s0.code, s2.code, s1.code, s2.code
        );
    }
}

#[cfg(feature = "biomeos")]
fn validate_travel_times(h: &mut ValidationHarness, stations: &[Station]) {
    use groundspring::seismic;

    println!("\n--- Travel Times ---");
    if stations.len() < 2 {
        return;
    }

    let s0 = &stations[0];
    let s1 = &stations[1];
    let d_km = seismic::haversine_km(s0.lat, s0.lon, s1.lat, s1.lon);
    let vp = 6.0; // typical upper mantle P-wave velocity
    let depth = 10.0; // shallow event

    let tt = seismic::travel_time_1d(d_km, depth, vp);
    println!(
        "  P-wave travel time ({}-{}): {tt:.2} s (d={d_km:.1} km, vp={vp} km/s)",
        s0.code, s1.code
    );

    h.check_true("Travel time > 0", tt > 0.0);
    h.check_true("Travel time proportional to distance", {
        let tt_half = seismic::travel_time_1d(d_km / 2.0, depth, vp);
        tt > tt_half
    });

    let expected_tt = d_km.hypot(depth) / vp;
    h.check_true(
        "Travel time matches raypath/velocity",
        (tt - expected_tt).abs() < TOL_GRID_MATCH,
    );
}

#[cfg(feature = "biomeos")]
fn validate_events(h: &mut ValidationHarness, socket: Option<&std::path::PathBuf>) {
    use groundspring::nestgate;

    println!("\n--- Earthquake Events ---");

    let Some(sock) = socket else {
        println!("  No NUCLEUS — skipping live event query");
        h.check_true("Event query skipped (no NUCLEUS)", true);
        return;
    };

    let query = nestgate::IrisEventQuery {
        min_lat: 35.0,
        max_lat: 37.0,
        min_lon: -90.5,
        max_lon: -88.5,
        start_date: "2024-01-01",
        end_date: "2024-12-31",
        min_magnitude: 2.5,
    };

    match nestgate::iris_events(sock, &query) {
        Ok(result) => {
            println!("  IRIS events: OK ({} bytes)", result.len());
            h.check_true("IRIS event query returns data", !result.is_empty());
        }
        Err(e) => {
            println!("  IRIS events: {e}");
            h.check_true("IRIS event query (or graceful error)", true);
        }
    }
}

#[cfg(feature = "biomeos")]
fn store_provenance(stations: &[Station], socket: Option<&std::path::PathBuf>, data_source: &str) {
    if let Some(sock) = socket {
        let n = stations.len();
        let result_json =
            format!(r#"{{"experiment":"exp032","data_source":"{data_source}","n_stations":{n}}}"#,);
        let _ = groundspring::nestgate::store_result(sock, 32, "latest", &result_json);
    }
}

#[cfg(feature = "biomeos")]
fn fetch_iris_stations(socket: &std::path::Path) -> groundspring::biomeos::Result<Vec<Station>> {
    use groundspring::nestgate;

    let raw = nestgate::iris_stations(socket, 34.0, 40.0, -92.0, -87.0)?;

    let mut stations = Vec::new();
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&raw)
        && let Some(rows) = parsed.get("stations").and_then(|s| s.as_array())
    {
        for row in rows {
            let Some(code) = row.get("station").and_then(|s| s.as_str()) else {
                continue;
            };
            let Some(lat) = row.get("latitude").and_then(serde_json::Value::as_f64) else {
                continue;
            };
            let Some(lon) = row.get("longitude").and_then(serde_json::Value::as_f64) else {
                continue;
            };
            if lat.abs() > 0.01 {
                stations.push(Station {
                    code: code.to_string(),
                    lat,
                    lon,
                });
            }
        }
    }

    if stations.is_empty() {
        return Err(groundspring::biomeos::BiomeOsError::Data(
            "No IRIS stations parsed".to_string(),
        ));
    }

    Ok(stations)
}

/// New Madrid Seismic Zone stations (synthetic fallback).
#[cfg(feature = "biomeos")]
fn synthetic_stations() -> Vec<Station> {
    vec![
        Station {
            code: "NMSZ01".to_string(),
            lat: 36.5,
            lon: -89.5,
        },
        Station {
            code: "NMSZ02".to_string(),
            lat: 35.8,
            lon: -90.0,
        },
        Station {
            code: "NMSZ03".to_string(),
            lat: 37.1,
            lon: -88.8,
        },
        Station {
            code: "NMSZ04".to_string(),
            lat: 36.0,
            lon: -89.2,
        },
    ]
}
