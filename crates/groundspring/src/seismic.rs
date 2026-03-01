// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Seismic wave propagation and source inversion.
//!
//! Provides travel-time computation and grid-search earthquake location
//! using the IASP91 simplified velocity model.
//!
//! # barracuda delegation
//!
//! [`grid_search_inversion`] is embarrassingly parallel — each
//! (lat, lon, depth) candidate evaluates independently. GPU promotion
//! dispatches as a 3D workgroup with per-point RMS reduction.
//! [`haversine_km`] and [`travel_time_1d`] stay local (scalar trig).

use crate::cast::{f64_usize, usize_f64};

/// Earth's mean radius in kilometers.
const EARTH_RADIUS_KM: f64 = 6371.0;

/// Great-circle (haversine) distance between two points in km.
#[must_use]
pub fn haversine_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let phi1 = lat1.to_radians();
    let phi2 = lat2.to_radians();
    let dphi = (lat2 - lat1).to_radians();
    let dlam = (lon2 - lon1).to_radians();

    let a =
        (phi1.cos() * phi2.cos()).mul_add((dlam / 2.0).sin().powi(2), (dphi / 2.0).sin().powi(2));

    EARTH_RADIUS_KM * 2.0 * a.sqrt().atan2((1.0 - a).sqrt())
}

/// Simplified 1D P-wave travel time (seconds).
///
/// Straight-ray approximation through uniform crust.  Adequate for
/// regional distances (<500 km) and shallow sources.
#[must_use]
pub fn travel_time_1d(distance_km: f64, depth_km: f64, vp_km_s: f64) -> f64 {
    let raypath = distance_km.hypot(depth_km);
    raypath / vp_km_s
}

/// A seismic station.
#[derive(Debug, Clone)]
pub struct Station {
    /// Station code.
    pub code: String,
    /// Latitude in degrees.
    pub lat: f64,
    /// Longitude in degrees.
    pub lon: f64,
}

/// Configuration for grid-search source inversion.
#[derive(Debug, Clone)]
pub struct GridSearchConfig {
    /// Latitude range (min, max) in degrees.
    pub lat_range: (f64, f64),
    /// Longitude range (min, max) in degrees.
    pub lon_range: (f64, f64),
    /// Depth range (min, max) in km.
    pub depth_range: (f64, f64),
    /// Grid spacing in degrees for lat/lon.
    pub grid_spacing_deg: f64,
    /// Depth spacing in km.
    pub depth_spacing_km: f64,
    /// P-wave velocity in km/s.
    pub vp: f64,
}

/// Result of a source inversion.
#[derive(Debug, Clone)]
pub struct InversionResult {
    /// Estimated source latitude.
    pub lat: f64,
    /// Estimated source longitude.
    pub lon: f64,
    /// Estimated source depth in km.
    pub depth_km: f64,
    /// Estimated origin time in seconds.
    pub origin_time_s: f64,
    /// RMS of travel-time residuals.
    pub rms_residual_s: f64,
}

/// Estimate origin time and RMS residual from paired observed/predicted travel times.
///
/// Origin time is the mean of (observed − predicted). RMS is computed from
/// residuals after subtracting the estimated origin time.
fn origin_time_and_rms(obs_times: &[f64], pred_tt: &[f64]) -> (f64, f64) {
    let n = usize_f64(obs_times.len());
    let t0: f64 = obs_times
        .iter()
        .zip(pred_tt.iter())
        .map(|(o, p)| o - p)
        .sum::<f64>()
        / n;

    let rms = (obs_times
        .iter()
        .zip(pred_tt.iter())
        .map(|(o, p)| (o - (t0 + p)).powi(2))
        .sum::<f64>()
        / n)
        .sqrt();

    (t0, rms)
}

/// Grid-search earthquake location by minimizing RMS travel-time residual.
///
/// For each candidate source position, estimates origin time as the mean
/// of (observed − predicted travel time), then computes RMS of residuals.
///
/// The `observed` slice pairs station codes (any type implementing
/// `AsRef<str>`) with arrival times, avoiding forced `String` ownership.
#[must_use]
pub fn grid_search_inversion<S: AsRef<str>>(
    observed: &[(S, f64)],
    stations: &[Station],
    config: &GridSearchConfig,
) -> InversionResult {
    #[cfg(feature = "barracuda-gpu")]
    {
        if let Some(result) = grid_search_inversion_gpu(observed, stations, config) {
            return result;
        }
    }
    grid_search_inversion_cpu(observed, stations, config)
}

/// GPU-accelerated seismic inversion: pre-evaluate RMS residuals on CPU into
/// a 3D grid, then use barracuda's `grid_search_3d` for parallel argmin.
///
/// The forward model (haversine + travel time) runs on CPU; only the 3D
/// minimum search is GPU-dispatched. For large grids (>10K points) the
/// parallel reduction significantly outperforms sequential scanning.
///
/// Cross-spring lineage: `grid_search_3d_f64.wgsl` — groundSpring forward
/// model + barracuda `ComputeDispatch` (absorbed S71+++).
#[cfg(feature = "barracuda-gpu")]
fn grid_search_inversion_gpu<S: AsRef<str>>(
    observed: &[(S, f64)],
    stations: &[Station],
    config: &GridSearchConfig,
) -> Option<InversionResult> {
    let device = crate::gpu::get_device()?;

    let obs_map: std::collections::HashMap<&str, f64> = observed
        .iter()
        .map(|(code, t)| (code.as_ref(), *t))
        .collect();

    let n_lat =
        1 + f64_usize(((config.lat_range.1 - config.lat_range.0) / config.grid_spacing_deg).ceil());
    let n_lon =
        1 + f64_usize(((config.lon_range.1 - config.lon_range.0) / config.grid_spacing_deg).ceil());
    let n_depth = 1 + f64_usize(
        ((config.depth_range.1 - config.depth_range.0) / config.depth_spacing_km).ceil(),
    );

    let lat_grid: Vec<f64> = (0..n_lat)
        .map(|i| usize_f64(i).mul_add(config.grid_spacing_deg, config.lat_range.0))
        .collect();
    let lon_grid: Vec<f64> = (0..n_lon)
        .map(|i| usize_f64(i).mul_add(config.grid_spacing_deg, config.lon_range.0))
        .collect();
    let depth_grid: Vec<f64> = (0..n_depth)
        .map(|i| usize_f64(i).mul_add(config.depth_spacing_km, config.depth_range.0))
        .collect();

    let total = n_lat * n_lon * n_depth;
    let mut rms_values = Vec::with_capacity(total);
    let mut pred_tt = Vec::with_capacity(stations.len());
    let mut obs_times = Vec::with_capacity(stations.len());

    for &lat in &lat_grid {
        for &lon in &lon_grid {
            for &depth in &depth_grid {
                pred_tt.clear();
                obs_times.clear();
                for sta in stations {
                    if let Some(&obs_t) = obs_map.get(sta.code.as_str()) {
                        let dist = haversine_km(lat, lon, sta.lat, sta.lon);
                        pred_tt.push(travel_time_1d(dist, depth, config.vp));
                        obs_times.push(obs_t);
                    }
                }
                if obs_times.is_empty() {
                    rms_values.push(f64::INFINITY);
                    continue;
                }
                let (_, rms) = origin_time_and_rms(&obs_times, &pred_tt);
                rms_values.push(rms);
            }
        }
    }

    let result = barracuda::ops::grid::grid_search_3d(
        &device,
        &lat_grid,
        &lon_grid,
        &depth_grid,
        &rms_values,
    )
    .ok()?;

    let lat = lat_grid[result.min_ix as usize];
    let lon = lon_grid[result.min_iy as usize];
    let depth = depth_grid[result.min_iz as usize];

    pred_tt.clear();
    obs_times.clear();
    for sta in stations {
        if let Some(&obs_t) = obs_map.get(sta.code.as_str()) {
            let dist = haversine_km(lat, lon, sta.lat, sta.lon);
            pred_tt.push(travel_time_1d(dist, depth, config.vp));
            obs_times.push(obs_t);
        }
    }
    let (t0, rms) = origin_time_and_rms(&obs_times, &pred_tt);

    Some(InversionResult {
        lat,
        lon,
        depth_km: depth,
        origin_time_s: t0,
        rms_residual_s: rms,
    })
}

fn grid_search_inversion_cpu<S: AsRef<str>>(
    observed: &[(S, f64)],
    stations: &[Station],
    config: &GridSearchConfig,
) -> InversionResult {
    let mut best = InversionResult {
        lat: 0.0,
        lon: 0.0,
        depth_km: 0.0,
        origin_time_s: 0.0,
        rms_residual_s: f64::INFINITY,
    };

    let obs_map: std::collections::HashMap<&str, f64> = observed
        .iter()
        .map(|(code, t)| (code.as_ref(), *t))
        .collect();

    let n_lat =
        1 + f64_usize(((config.lat_range.1 - config.lat_range.0) / config.grid_spacing_deg).ceil());
    let n_lon =
        1 + f64_usize(((config.lon_range.1 - config.lon_range.0) / config.grid_spacing_deg).ceil());
    let n_depth = 1 + f64_usize(
        ((config.depth_range.1 - config.depth_range.0) / config.depth_spacing_km).ceil(),
    );

    let mut pred_tt = Vec::with_capacity(stations.len());
    let mut obs_times = Vec::with_capacity(stations.len());

    for i_lat in 0..n_lat {
        let lat = usize_f64(i_lat).mul_add(config.grid_spacing_deg, config.lat_range.0);
        for i_lon in 0..n_lon {
            let lon = usize_f64(i_lon).mul_add(config.grid_spacing_deg, config.lon_range.0);
            for i_depth in 0..n_depth {
                let depth =
                    usize_f64(i_depth).mul_add(config.depth_spacing_km, config.depth_range.0);

                pred_tt.clear();
                obs_times.clear();

                for sta in stations {
                    if let Some(&obs_t) = obs_map.get(sta.code.as_str()) {
                        let dist = haversine_km(lat, lon, sta.lat, sta.lon);
                        pred_tt.push(travel_time_1d(dist, depth, config.vp));
                        obs_times.push(obs_t);
                    }
                }

                if obs_times.is_empty() {
                    continue;
                }

                let (t0, rms) = origin_time_and_rms(&obs_times, &pred_tt);
                if rms < best.rms_residual_s {
                    best = InversionResult {
                        lat,
                        lon,
                        depth_km: depth,
                        origin_time_s: t0,
                        rms_residual_s: rms,
                    };
                }
            }
        }
    }

    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn haversine_zero() {
        assert!(haversine_km(37.5, -89.0, 37.5, -89.0).abs() < 1e-10);
    }

    #[test]
    fn haversine_ny_london() {
        let d = haversine_km(40.7128, -74.0060, 51.5074, -0.1278);
        assert!((d - 5570.0).abs() < 50.0, "NY-London should be ~5570 km");
    }

    #[test]
    fn travel_time_proportional() {
        let t1 = travel_time_1d(100.0, 10.0, 6.0);
        let t2 = travel_time_1d(200.0, 10.0, 6.0);
        assert!(t2 > t1);
    }

    #[test]
    fn travel_time_known_value() {
        // 100km horizontal, 0km depth, 6 km/s → 100/6 = 16.667s
        let t = travel_time_1d(100.0, 0.0, 6.0);
        assert!((t - 100.0 / 6.0).abs() < 1e-10);
    }

    #[test]
    fn grid_search_recovers_clean_source() {
        let stations = vec![
            Station {
                code: "A".into(),
                lat: 37.0,
                lon: -90.0,
            },
            Station {
                code: "B".into(),
                lat: 38.0,
                lon: -88.0,
            },
            Station {
                code: "C".into(),
                lat: 36.5,
                lon: -89.5,
            },
        ];

        let true_lat = 37.5;
        let true_lon = -89.0;
        let true_depth = 10.0;
        let vp = 5.8;

        let observed: Vec<(&str, f64)> = stations
            .iter()
            .map(|s| {
                let dist = haversine_km(true_lat, true_lon, s.lat, s.lon);
                let tt = travel_time_1d(dist, true_depth, vp);
                (s.code.as_str(), tt)
            })
            .collect();

        let config = GridSearchConfig {
            lat_range: (36.0, 39.0),
            lon_range: (-91.0, -87.0),
            depth_range: (0.0, 20.0),
            grid_spacing_deg: 0.1,
            depth_spacing_km: 5.0,
            vp,
        };
        let result = grid_search_inversion(&observed, &stations, &config);

        let loc_error = haversine_km(result.lat, result.lon, true_lat, true_lon);
        assert!(
            loc_error < 20.0,
            "Grid search should recover source within 20 km, got {loc_error:.1}"
        );
    }
}
