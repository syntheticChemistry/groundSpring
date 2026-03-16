// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Semantic method dispatch for the groundSpring JSON-RPC server.
//!
//! Maps `measurement.*` capability methods to groundSpring library functions.
//! biomeOS routes incoming `capability.call` requests to this primal's socket;
//! the dispatch layer translates the semantic method name into the actual
//! library call.
//!
//! Methods follow the Semantic Method Naming Standard:
//! `{domain}.{operation}[.{variant}]`

use serde_json::Value;

// ─── Dispatch Defaults ───────────────────────────────────────────────────────
//
// RPC callers may omit optional parameters; these defaults mirror the
// Bazavov et al. (2016) benchmark configuration and standard spectral
// analysis conventions so that a bare `capability.call` returns sensible
// results without requiring the caller to know domain physics.

/// Default Tikhonov regularization for spectral feature extraction.
///
/// 1e-4 balances noise suppression against spectral peak resolution
/// in the correlator → spectral-function inversion (Exp 028).
const DEFAULT_REGULARIZATION: f64 = 1e-4;

/// Default time-step spacing for correlator τ grid (spectral features).
///
/// 0.1 matches the Euclidean-time lattice spacing convention in
/// hotSpring Exp 015/022 and barraCuda benchmark correlator data.
const DEFAULT_TAU_STEP: f64 = 0.1;

/// Default angular frequency spacing for spectral ω grid.
///
/// 0.2 provides sufficient resolution for Matsubara peak detection
/// while keeping the kernel matrix well-conditioned.
const DEFAULT_OMEGA_STEP: f64 = 0.2;

/// Default measurement uncertainty σ for freeze-out fits.
const DEFAULT_SIGMA: f64 = 1.0;

/// Default T₀ grid lower bound (in `MeV`) — Bazavov et al. (2016).
const DEFAULT_T0_LO: f64 = 100.0;
/// Default T₀ grid upper bound (in `MeV`).
const DEFAULT_T0_HI: f64 = 200.0;
/// Default T₀ grid step size (in `MeV`).
const DEFAULT_T0_STEP: f64 = 1.0;
/// Default κ₂ grid lower bound — Bazavov et al. (2016).
const DEFAULT_K2_LO: f64 = 0.001;
/// Default κ₂ grid upper bound.
const DEFAULT_K2_HI: f64 = 0.05;
/// Default κ₂ grid step size.
const DEFAULT_K2_STEP: f64 = 0.001;

/// Default centre energy for Anderson validation (mid-band).
const DEFAULT_ENERGY: f64 = 0.0;

/// Default bootstrap confidence level (95th percentile).
const DEFAULT_CONFIDENCE: f64 = 0.95;

/// Default station elevation (sea level, metres) for FAO-56 ET₀.
const DEFAULT_ELEVATION_M: f64 = 0.0;

/// Default maximum relative humidity (%) — typical humid climate.
const DEFAULT_RHMAX_PCT: f64 = 80.0;

/// Default minimum relative humidity (%) — typical daytime drop.
const DEFAULT_RHMIN_PCT: f64 = 40.0;

/// Default margin for rule-based regime classification (spacing-ratio window).
const DEFAULT_REGIME_MARGIN: f64 = 0.1;

/// Dispatch a JSON-RPC method call to the appropriate library function.
///
/// Returns `Ok(result_json)` on success or `Err(message)` on failure.
/// Unknown methods return a standard JSON-RPC "method not found" error.
///
/// # Errors
///
/// Returns `Err` with a human-readable message if the method is unknown,
/// required parameters are missing, or the underlying library call fails.
pub fn dispatch(method: &str, params: &Value) -> Result<Value, String> {
    match method {
        "health.check" | "health" => Ok(health_check()),
        "capability.list" => Ok(capability_list()),
        "lifecycle.status" => Ok(lifecycle_status()),

        "measurement.noise_decomposition" => noise_decomposition(params),
        "measurement.anderson_validation" => anderson_validation(params),
        "measurement.uncertainty_budget" => uncertainty_budget(params),
        "measurement.et0_propagation" => et0_propagation(params),
        "measurement.regime_classification" => regime_classification(params),
        "measurement.spectral_features" => spectral_features(params),
        "measurement.parity_check" => parity_check(params),
        "measurement.freeze_out" => freeze_out(params),

        _ => Err(format!("method not found: {method}")),
    }
}

// ─── Lifecycle Methods ───────────────────────────────────────────────────────

static START_TIME: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();

/// Initialize the start time. Call once at server startup.
pub fn init_start_time() {
    START_TIME.get_or_init(std::time::Instant::now);
}

fn uptime_secs() -> u64 {
    START_TIME
        .get()
        .map_or(0, |start| start.elapsed().as_secs())
}

fn health_check() -> Value {
    serde_json::json!({
        "status": "healthy",
        "primal": crate::biomeos::FAMILY_ID,
        "version": env!("CARGO_PKG_VERSION"),
        "capabilities": crate::biomeos::MEASUREMENT_CAPABILITIES,
        "uptime_seconds": uptime_secs(),
    })
}

fn capability_list() -> Value {
    serde_json::json!({
        "domain": crate::biomeos::MEASUREMENT_DOMAIN,
        "capabilities": crate::biomeos::MEASUREMENT_CAPABILITIES,
    })
}

fn lifecycle_status() -> Value {
    serde_json::json!({
        "name": crate::biomeos::FAMILY_ID,
        "family_id": crate::biomeos::FAMILY_ID,
        "version": env!("CARGO_PKG_VERSION"),
        "capabilities": crate::biomeos::MEASUREMENT_CAPABILITIES,
        "uptime_seconds": uptime_secs(),
    })
}

// ─── Measurement Methods ─────────────────────────────────────────────────────

fn noise_decomposition(params: &Value) -> Result<Value, String> {
    let observed = extract_f64_array(params, "observed")?;
    let modeled = extract_f64_array(params, "modeled")?;

    if observed.len() != modeled.len() {
        return Err("observed and modeled must have equal length".to_string());
    }

    let rmse = crate::stats::rmse(&observed, &modeled);
    let mbe = crate::stats::mbe(&observed, &modeled);
    let decomp = crate::decompose::decompose_error(mbe, rmse);

    Ok(serde_json::json!({
        "rmse": rmse,
        "mbe": mbe,
        "bias_fraction": decomp.bias_fraction,
        "noise_fraction": decomp.noise_fraction,
    }))
}

fn anderson_validation(params: &Value) -> Result<Value, String> {
    let n_sites = extract_usize(params, "n_sites", 10_000)?;
    let disorder = extract_f64(params, "disorder", 4.0);
    let energy = extract_f64(params, "energy", DEFAULT_ENERGY);
    let n_realizations = extract_usize(params, "n_realizations", 20)?;
    let seed = extract_u64(params, "seed", 42);

    let gamma = crate::anderson::lyapunov_averaged(n_sites, disorder, energy, n_realizations, seed);
    let loc_length = crate::anderson::localization_length(gamma);

    Ok(serde_json::json!({
        "gamma": gamma,
        "localization_length": loc_length,
        "n_sites": n_sites,
        "disorder": disorder,
    }))
}

fn uncertainty_budget(params: &Value) -> Result<Value, String> {
    let data = extract_f64_array(params, "data")?;
    let confidence = extract_f64(params, "confidence", DEFAULT_CONFIDENCE);
    let n_bootstrap = extract_usize(params, "n_bootstrap", 10_000)?;
    let seed = extract_u64(params, "seed", 42);

    let boot_mean = crate::bootstrap::bootstrap_mean(&data, n_bootstrap, confidence, seed);
    let jk = crate::jackknife::jackknife_mean_variance(&data)
        .map_err(|e| format!("jackknife error: {e}"))?;

    Ok(serde_json::json!({
        "bootstrap": {
            "estimate": boot_mean.estimate,
            "ci_lower": boot_mean.ci_lower,
            "ci_upper": boot_mean.ci_upper,
            "std_error": boot_mean.std_error,
        },
        "jackknife": {
            "estimate": jk.estimate,
            "variance": jk.variance,
            "std_error": jk.std_error,
        },
    }))
}

fn et0_propagation(params: &Value) -> Result<Value, String> {
    let tmax = require_f64(params, "temperature_max")?;
    let tmin = require_f64(params, "temperature_min")?;
    let wind = require_f64(params, "wind_speed")?;
    let sunshine = require_f64(params, "sunshine_hours")?;
    let lat = require_f64(params, "latitude")?;
    let doy_u64 = params
        .get("day_of_year")
        .and_then(Value::as_u64)
        .ok_or("missing day_of_year")?;
    let doy = u16::try_from(doy_u64).map_err(|_| "day_of_year out of u16 range")?;
    let elevation = extract_f64(params, "elevation", DEFAULT_ELEVATION_M);
    let rhmax = extract_f64(params, "rhmax", DEFAULT_RHMAX_PCT);
    let rhmin = extract_f64(params, "rhmin", DEFAULT_RHMIN_PCT);

    let inp = crate::fao56::DailyWeatherInputs {
        tmax_c: tmax,
        tmin_c: tmin,
        rhmax_pct: rhmax,
        rhmin_pct: rhmin,
        wind_speed_10m_km_h: wind,
        sunshine_hours: sunshine,
        latitude_deg_n: lat,
        altitude_m: elevation,
        day_of_year: doy,
    };

    let et0 = crate::fao56::daily_et0(&inp);

    Ok(serde_json::json!({
        "et0_mm_day": et0,
        "method": "FAO-56 Penman-Monteith",
    }))
}

fn regime_classification(params: &Value) -> Result<Value, String> {
    let mut eigenvalues = extract_f64_array(params, "eigenvalues")?;
    let margin = extract_f64(params, "margin", DEFAULT_REGIME_MARGIN);

    let features = crate::esn::spectral_features(&mut eigenvalues);
    let label = crate::esn::classify_by_spacing_ratio(features[0], margin);

    Ok(serde_json::json!({
        "label": format!("{label:?}"),
        "mean_spacing_ratio": features[0],
        "spectral_rigidity": features[1],
        "ipr": features[2],
    }))
}

fn spectral_features(params: &Value) -> Result<Value, String> {
    let correlator = extract_f64_array(params, "correlator")?;
    let n_omega = extract_usize(params, "n_omega", 50)?;
    let alpha = extract_f64(params, "regularization", DEFAULT_REGULARIZATION);

    let n_tau = correlator.len();
    let tau: Vec<f64> = (0..n_tau)
        .map(|i| crate::cast::usize_f64(i) * DEFAULT_TAU_STEP)
        .collect();
    let omega: Vec<f64> = (0..n_omega)
        .map(|i| crate::cast::usize_f64(i) * DEFAULT_OMEGA_STEP)
        .collect();
    let kernel = crate::spectral_recon::build_kernel(&tau, &omega);
    let rho = crate::spectral_recon::tikhonov_solve(&kernel, &correlator, alpha, n_tau, n_omega);
    let peak = crate::spectral_recon::peak_index(&rho);
    let roundtrip = crate::spectral_recon::forward_correlator(&kernel, &rho, n_tau, n_omega);
    let residual = crate::spectral_recon::rmse(&correlator, &roundtrip);

    Ok(serde_json::json!({
        "spectral_function": rho,
        "peak_index": peak,
        "residual_rmse": residual,
        "n_omega": n_omega,
    }))
}

fn parity_check(params: &Value) -> Result<Value, String> {
    let cpu_values = extract_f64_array(params, "cpu_values")?;
    let gpu_values = extract_f64_array(params, "gpu_values")?;
    let tolerance = extract_f64(params, "tolerance", crate::tol::EXACT);

    if cpu_values.len() != gpu_values.len() {
        return Err("cpu_values and gpu_values must have equal length".to_string());
    }

    let max_diff = cpu_values
        .iter()
        .zip(gpu_values.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0_f64, f64::max);

    let parity = max_diff <= tolerance;

    Ok(serde_json::json!({
        "parity": parity,
        "max_difference": max_diff,
        "tolerance": tolerance,
        "n_values": cpu_values.len(),
    }))
}

fn freeze_out(params: &Value) -> Result<Value, String> {
    let observed = extract_f64_array(params, "observed")?;
    let mu_b = extract_f64_array(params, "mu_b")?;

    let config = crate::freeze_out::GridFitConfig {
        observed: &observed,
        mu_b: &mu_b,
        sigma: extract_f64(params, "sigma", DEFAULT_SIGMA),
        t0_lo: extract_f64(params, "t0_lo", DEFAULT_T0_LO),
        t0_hi: extract_f64(params, "t0_hi", DEFAULT_T0_HI),
        t0_step: extract_f64(params, "t0_step", DEFAULT_T0_STEP),
        k2_lo: extract_f64(params, "k2_lo", DEFAULT_K2_LO),
        k2_hi: extract_f64(params, "k2_hi", DEFAULT_K2_HI),
        k2_step: extract_f64(params, "k2_step", DEFAULT_K2_STEP),
    };

    let fit =
        crate::freeze_out::grid_fit_2d(&config).map_err(|e| format!("freeze_out error: {e}"))?;

    Ok(serde_json::json!({
        "t0": fit.t0,
        "kappa2": fit.kappa2,
        "chi_squared": fit.chi_squared,
        "chi2_per_dof": fit.chi2_per_dof,
    }))
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn extract_f64_array(params: &Value, key: &str) -> Result<Vec<f64>, String> {
    params
        .get(key)
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(Value::as_f64).collect::<Vec<_>>())
        .ok_or_else(|| format!("missing or invalid array: {key}"))
}

fn extract_f64(params: &Value, key: &str, default: f64) -> f64 {
    params.get(key).and_then(Value::as_f64).unwrap_or(default)
}

fn require_f64(params: &Value, key: &str) -> Result<f64, String> {
    params
        .get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| format!("missing {key}"))
}

fn extract_u64(params: &Value, key: &str, default: u64) -> u64 {
    params.get(key).and_then(Value::as_u64).unwrap_or(default)
}

fn extract_usize(params: &Value, key: &str, default: u64) -> Result<usize, String> {
    let v = extract_u64(params, key, default);
    usize::try_from(v).map_err(|_| format!("{key} too large for usize"))
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]
mod tests {
    use super::*;

    #[test]
    fn dispatch_unknown_method_returns_error() {
        let result = dispatch("nonexistent.method", &Value::Null);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("method not found"));
    }

    #[test]
    fn dispatch_health_check() {
        init_start_time();
        let result = dispatch("health.check", &Value::Null);
        assert!(result.is_ok());
        let v = result.unwrap();
        assert_eq!(v["status"], "healthy");
        assert_eq!(v["primal"], crate::biomeos::FAMILY_ID);
    }

    #[test]
    fn dispatch_capability_list() {
        let result = dispatch("capability.list", &Value::Null);
        assert!(result.is_ok());
        let v = result.unwrap();
        assert_eq!(v["domain"], "measurement");
    }

    #[test]
    fn dispatch_lifecycle_status() {
        init_start_time();
        let result = dispatch("lifecycle.status", &Value::Null);
        assert!(result.is_ok());
        let v = result.unwrap();
        assert_eq!(v["name"], crate::biomeos::FAMILY_ID);
        assert_eq!(v["family_id"], crate::biomeos::FAMILY_ID);
    }

    #[test]
    fn dispatch_noise_decomposition() {
        let params = serde_json::json!({
            "observed": [1.0, 2.0, 3.0, 4.0],
            "modeled": [1.5, 2.5, 3.5, 4.5],
        });
        let result = dispatch("measurement.noise_decomposition", &params);
        assert!(result.is_ok());
        let v = result.unwrap();
        assert!(v.get("rmse").is_some());
        assert!(v.get("bias_fraction").is_some());
    }

    #[test]
    fn dispatch_parity_check_matching() {
        let params = serde_json::json!({
            "cpu_values": [1.0, 2.0, 3.0],
            "gpu_values": [1.0, 2.0, 3.0],
            "tolerance": 1e-10,
        });
        let result = dispatch("measurement.parity_check", &params);
        assert!(result.is_ok());
        let v = result.unwrap();
        assert_eq!(v["parity"], true);
    }

    #[test]
    fn dispatch_parity_check_mismatch() {
        let params = serde_json::json!({
            "cpu_values": [1.0, 2.0, 3.0],
            "gpu_values": [1.0, 2.0, 4.0],
            "tolerance": 1e-10,
        });
        let result = dispatch("measurement.parity_check", &params);
        assert!(result.is_ok());
        let v = result.unwrap();
        assert_eq!(v["parity"], false);
    }
}
