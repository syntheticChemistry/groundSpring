// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Seismic wave propagation and source inversion.
//!
//! Provides travel-time computation and grid-search earthquake location
//! using the IASP91 simplified velocity model.

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

/// Grid-search earthquake location by minimizing RMS travel-time residual.
///
/// For each candidate source position, estimates origin time as the mean
/// of (observed − predicted travel time), then computes RMS of residuals.
#[must_use]
pub fn grid_search_inversion(
    observed: &[(String, f64)],
    stations: &[Station],
    config: &GridSearchConfig,
) -> InversionResult {
    let mut best_rms = f64::INFINITY;
    let mut best = InversionResult {
        lat: 0.0,
        lon: 0.0,
        depth_km: 0.0,
        origin_time_s: 0.0,
        rms_residual_s: f64::INFINITY,
    };

    let obs_map: std::collections::HashMap<&str, f64> = observed
        .iter()
        .map(|(code, t)| (code.as_str(), *t))
        .collect();

    let n_lat =
        1 + ((config.lat_range.1 - config.lat_range.0) / config.grid_spacing_deg).ceil() as usize;
    let n_lon =
        1 + ((config.lon_range.1 - config.lon_range.0) / config.grid_spacing_deg).ceil() as usize;
    let n_depth = 1
        + ((config.depth_range.1 - config.depth_range.0) / config.depth_spacing_km).ceil() as usize;

    for i_lat in 0..n_lat {
        let lat = (i_lat as f64).mul_add(config.grid_spacing_deg, config.lat_range.0);
        for i_lon in 0..n_lon {
            let lon = (i_lon as f64).mul_add(config.grid_spacing_deg, config.lon_range.0);
            for i_depth in 0..n_depth {
                let depth = (i_depth as f64).mul_add(config.depth_spacing_km, config.depth_range.0);
                let mut pred_tt = Vec::with_capacity(stations.len());
                let mut obs_times = Vec::with_capacity(stations.len());

                for sta in stations {
                    if let Some(&obs_t) = obs_map.get(sta.code.as_str()) {
                        let dist = haversine_km(lat, lon, sta.lat, sta.lon);
                        let tt = travel_time_1d(dist, depth, config.vp);
                        pred_tt.push(tt);
                        obs_times.push(obs_t);
                    }
                }

                if obs_times.is_empty() {
                    continue;
                }

                let n = obs_times.len() as f64;
                let t0: f64 = obs_times
                    .iter()
                    .zip(&pred_tt)
                    .map(|(o, p)| o - p)
                    .sum::<f64>()
                    / n;

                let rms = (obs_times
                    .iter()
                    .zip(&pred_tt)
                    .map(|(o, p)| (o - (t0 + p)).powi(2))
                    .sum::<f64>()
                    / n)
                    .sqrt();

                if rms < best_rms {
                    best_rms = rms;
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

        let observed: Vec<(String, f64)> = stations
            .iter()
            .map(|s| {
                let dist = haversine_km(true_lat, true_lon, s.lat, s.lon);
                let tt = travel_time_1d(dist, true_depth, vp);
                (s.code.clone(), tt)
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
