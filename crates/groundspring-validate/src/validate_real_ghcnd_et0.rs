// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team
#![forbid(unsafe_code)]

//! Exp 029: Real GHCND ET₀ — Hargreaves vs Penman-Monteith on live weather data.
//!
//! When NUCLEUS is running with `NestGate`, fetches real NOAA GHCND daily weather
//! data for a Michigan station and compares Hargreaves and Penman-Monteith ET₀
//! estimates on the real observations.
//!
//! When NUCLEUS is unavailable, falls back to synthetic weather data and validates
//! the computational pipeline alone. The science is valid either way — live data
//! just adds a real-world anchor.
//!
//! Requires: `--features biomeos` (compile-time) + running NUCLEUS (runtime, optional)

#[cfg(not(feature = "biomeos"))]
compile_error!("Exp 029 requires --features biomeos");

#[cfg(feature = "biomeos")]
fn main() {
    use groundspring::biomeos;
    use groundspring::fao56::{self, DailyWeatherInputs};
    use groundspring::validate::ValidationHarness;

    let mut h = ValidationHarness::stdout("Exp 029: Real GHCND ET₀");

    println!("{}", "=".repeat(72));
    println!("  Exp 029: Real GHCND Weather → Hargreaves vs Penman-Monteith ET₀");
    println!("{}", "=".repeat(72));
    println!();
    println!("  Provenance: NUCLEUS live-data validation binary");
    println!("  Data source: NOAA GHCND (USW00094847 Lansing, MI) or synthetic");
    println!("  Baseline: Analytical (FAO-56 Penman-Monteith / Hargreaves 1985)");
    println!("  Note: No benchmark JSON — validates computational pipeline and");
    println!("        method agreement, not Python baseline comparison.");
    println!();

    let socket = biomeos::auto_connect();
    let data_source = if socket.is_some() {
        "LIVE NOAA GHCND"
    } else {
        "SYNTHETIC"
    };
    println!("  Data source: {data_source}");
    println!();

    let weather_days: Vec<DailyWeatherInputs> = socket.as_ref().map_or_else(
        || {
            println!("  No NUCLEUS available, using synthetic data");
            synthetic_weather()
        },
        |sock| match fetch_live_weather(sock) {
            Ok(days) => {
                println!("  Fetched {} days of live GHCND data", days.len());
                days
            }
            Err(e) => {
                println!("  Live fetch failed ({e}), using synthetic data");
                synthetic_weather()
            }
        },
    );

    let mut pm_values = Vec::with_capacity(weather_days.len());
    let mut harg_values = Vec::with_capacity(weather_days.len());

    for day in &weather_days {
        let pm = fao56::daily_et0(day);
        let harg =
            fao56::hargreaves_et0(day.tmax_c, day.tmin_c, day.latitude_deg_n, day.day_of_year);

        pm_values.push(pm);
        harg_values.push(harg);
    }

    println!("  Penman-Monteith: {} days", pm_values.len());
    println!("  Hargreaves:      {} days", harg_values.len());

    h.check_true("PM produces valid ET₀", !pm_values.is_empty());
    h.check_true("Hargreaves produces valid ET₀", !harg_values.is_empty());

    let pm_reasonable = pm_values.iter().all(|&v| (0.0..=15.0).contains(&v));
    let harg_reasonable = harg_values.iter().all(|&v| (0.0..=15.0).contains(&v));
    h.check_true("PM values in [0, 15] mm/day", pm_reasonable);
    h.check_true("Hargreaves values in [0, 15] mm/day", harg_reasonable);

    if !pm_values.is_empty() && !harg_values.is_empty() {
        let n = pm_values.len().min(harg_values.len());
        let pm_mean = groundspring::stats::mean(&pm_values[..n]);
        let harg_mean = groundspring::stats::mean(&harg_values[..n]);

        println!();
        println!("  PM mean ET₀:        {pm_mean:.3} mm/day");
        println!("  Hargreaves mean ET₀: {harg_mean:.3} mm/day");

        if harg_mean > 0.0 {
            let ratio = pm_mean / harg_mean;
            println!("  Ratio PM/Harg:       {ratio:.3}");
            h.check_range(
                "PM/Hargreaves ratio",
                ratio,
                groundspring_validate::SANITY_PM_HARG_RATIO.0,
                groundspring_validate::SANITY_PM_HARG_RATIO.1,
            );
        }

        let diffs: Vec<f64> = pm_values[..n]
            .iter()
            .zip(&harg_values[..n])
            .map(|(pm, harg)| (pm - harg).abs())
            .collect();
        let mean_abs_diff = groundspring::stats::mean(&diffs);
        println!("  Mean |PM - Harg|:    {mean_abs_diff:.3} mm/day");
        h.check_max(
            "Mean absolute difference",
            mean_abs_diff,
            groundspring_validate::SANITY_PM_HARG_DIFF_MAX,
        );
    }

    if let Some(ref sock) = socket {
        let result_json = format!(
            r#"{{"experiment":"exp029","data_source":"{data_source}","pm_days":{},"harg_days":{}}}"#,
            pm_values.len(),
            harg_values.len()
        );
        let _ = groundspring::nestgate::store_result(sock, 29, "latest", &result_json);
    }

    println!();
    std::process::exit(h.summary());
}

#[cfg(feature = "biomeos")]
/// GHCND station parameters for Exp 029.
///
/// Station metadata sourced from NOAA GHCND:
/// <https://www.ncei.noaa.gov/access/search/data-search/daily-summaries?stations=USW00094847>
///
/// | Field      | Value            | Source                              |
/// |------------|------------------|-------------------------------------|
/// | Station    | `USW00094847`    | NOAA GHCND network ID               |
/// | Latitude   | 42.78 °N         | GHCND station metadata               |
/// | Altitude   | 265.0 m          | GHCND station metadata               |
/// | Period     | 2024-06-01..30   | Summer window for ET₀ validation     |
mod station {
    pub const GHCND_STATION: &str = "USW00094847";
    pub const GHCND_START: &str = "2024-06-01";
    pub const GHCND_END: &str = "2024-06-30";
    pub const GHCND_VARS: &[&str] = &["TMAX", "TMIN"];
    pub const LATITUDE_DEG_N: f64 = 42.78;
    pub const ALTITUDE_M: f64 = 265.0;
    pub const RHMAX_PCT: f64 = 85.0;
    pub const RHMIN_PCT: f64 = 45.0;
    pub const WIND_SPEED_10M_KM_H: f64 = 9.0;
    pub const SUNSHINE_HOURS: f64 = 10.0;
}

fn fetch_live_weather(
    socket: &std::path::Path,
) -> groundspring::biomeos::Result<Vec<groundspring::fao56::DailyWeatherInputs>> {
    use groundspring::nestgate;

    let raw = nestgate::noaa_ghcnd(
        socket,
        station::GHCND_STATION,
        station::GHCND_START,
        station::GHCND_END,
        station::GHCND_VARS,
    )?;

    let mut days = Vec::new();
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&raw)
        && let Some(results) = parsed
            .get("data")
            .and_then(|d| d.get("results"))
            .and_then(|r| r.as_array())
    {
        let mut tmax_map = std::collections::BTreeMap::new();
        let mut tmin_map = std::collections::BTreeMap::new();

        for record in results {
            let Some(datatype) = record["datatype"].as_str() else {
                continue;
            };
            let Some(date) = record["date"].as_str() else {
                continue;
            };
            let Some(value) = record["value"].as_f64() else {
                continue;
            };
            let value_c = value / 10.0;

            match datatype {
                "TMAX" => {
                    tmax_map.insert(date.to_string(), value_c);
                }
                "TMIN" => {
                    tmin_map.insert(date.to_string(), value_c);
                }
                _ => {}
            }
        }

        for (i, (date, tmax)) in tmax_map.iter().enumerate() {
            if let Some(&tmin) = tmin_map.get(date) {
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "day-of-year index; max i < 31, so 152 + i fits u16"
                )]
                let doy = 152 + i as u16;
                days.push(groundspring::fao56::DailyWeatherInputs {
                    tmax_c: *tmax,
                    tmin_c: tmin,
                    rhmax_pct: station::RHMAX_PCT,
                    rhmin_pct: station::RHMIN_PCT,
                    wind_speed_10m_km_h: station::WIND_SPEED_10M_KM_H,
                    sunshine_hours: station::SUNSHINE_HOURS,
                    latitude_deg_n: station::LATITUDE_DEG_N,
                    altitude_m: station::ALTITUDE_M,
                    day_of_year: doy,
                });
            }
        }
    }

    if days.is_empty() {
        return Err(groundspring::biomeos::BiomeOsError::Data(
            "No valid weather records parsed".to_string(),
        ));
    }

    Ok(days)
}

#[cfg(feature = "biomeos")]
fn synthetic_weather() -> Vec<groundspring::fao56::DailyWeatherInputs> {
    (1..=30)
        .map(|day| {
            let d = f64::from(day);
            let tmax = 5.0f64.mul_add(d / 30.0, 25.0);
            let tmin = 3.0f64.mul_add(d / 30.0, 12.0);
            let doy = 152 + day;

            groundspring::fao56::DailyWeatherInputs {
                tmax_c: tmax,
                tmin_c: tmin,
                rhmax_pct: station::RHMAX_PCT,
                rhmin_pct: station::RHMIN_PCT,
                wind_speed_10m_km_h: station::WIND_SPEED_10M_KM_H,
                sunshine_hours: station::SUNSHINE_HOURS,
                latitude_deg_n: station::LATITUDE_DEG_N,
                altitude_m: station::ALTITUDE_M,
                day_of_year: doy,
            }
        })
        .collect()
}
