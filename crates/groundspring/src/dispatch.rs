// SPDX-License-Identifier: AGPL-3.0-or-later
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

use crate::error::DispatchError;

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

// ─── Method-Body Defaults ────────────────────────────────────────────────────
//
// Inline numeric defaults that appear in dispatch method bodies.
// Named here for provenance tracking and consistent documentation.

/// Default reproducibility seed for stochastic methods.
///
/// The answer to the Ultimate Question — ensures deterministic results
/// when callers omit the seed parameter, matching the convention used
/// across hotSpring, wetSpring, and airSpring dispatch layers.
const DEFAULT_SEED: u64 = 42;

/// Default Anderson lattice size — 10 000 sites.
///
/// Provenance: standard 1D lattice length in Kachkovskiy (Paper 2)
/// and Anderson localization finite-size scaling studies. Balances
/// accuracy with sub-second evaluation on CPU.
const DEFAULT_ANDERSON_N_SITES: u64 = 10_000;

/// Default Anderson disorder strength W = 4.0.
///
/// Provenance: W = 4.0 sits in the strongly localized regime for 1D
/// Anderson (all states are localized for W > 0), producing
/// localization lengths accessible to finite-size lattices.
/// Validated: hotSpring Exp 015 disorder sweeps, groundSpring Exp 031.
const DEFAULT_ANDERSON_DISORDER: f64 = 4.0;

/// Default number of disorder realizations for Anderson averaging.
///
/// 20 realizations balances statistical averaging with evaluation time.
/// Provenance: finite-size analysis convention in Papers 2 & 3.
const DEFAULT_ANDERSON_REALIZATIONS: u64 = 20;

/// Default bootstrap replicate count — 10 000.
///
/// Standard recommendation (Efron & Tibshirani 1993) for percentile-
/// bootstrap CIs with moderate sample sizes.
const DEFAULT_N_BOOTSTRAP: u64 = 10_000;

/// Default spectral ω grid size — 50 points.
///
/// Sufficient resolution for Matsubara peak detection in lattice QCD
/// correlator spectral reconstruction (Exp 028).
const DEFAULT_N_OMEGA: u64 = 50;

/// Dispatch a JSON-RPC method call to the appropriate library function.
///
/// Returns `Ok(result_json)` on success or a typed [`DispatchError`] on failure.
/// Unknown methods return [`DispatchError::MethodNotFound`].
///
/// # Errors
///
/// Returns `Err` if the method is unknown, required parameters are missing,
/// or the underlying library call fails with an [`crate::error::InputError`].
pub fn dispatch(method: &str, params: &Value) -> Result<Value, DispatchError> {
    match method {
        "health.check" | "health" => Ok(health_check()),
        "health.liveness" => Ok(health_liveness()),
        "health.readiness" => Ok(health_readiness()),
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

        "measurement.bootstrap" => bootstrap(params),
        "measurement.rarefaction" => rarefaction(params),
        "measurement.drift" => drift(params),
        "measurement.band_edge" => band_edge(params),
        "measurement.rare_biosphere" => rare_biosphere(params),
        "measurement.gillespie" => gillespie(params),
        "measurement.bistable" => bistable(params),
        "measurement.quasispecies" => quasispecies(params),

        _ => Err(DispatchError::MethodNotFound(method.to_owned())),
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

/// Liveness probe — answers immediately if the process is alive.
///
/// Absorbed from wetSpring V121 / airSpring V0.8.8 / healthSpring V30
/// health probe pattern. biomeOS uses this to distinguish "process is
/// alive" from "process is ready to serve requests".
fn health_liveness() -> Value {
    serde_json::json!({
        "status": "alive",
        "primal": crate::biomeos::FAMILY_ID,
    })
}

/// Readiness probe — confirms the server can accept and process requests.
///
/// Returns capability count and uptime. A positive capability count
/// indicates all dispatch routes are wired and ready.
fn health_readiness() -> Value {
    serde_json::json!({
        "status": "ready",
        "primal": crate::biomeos::FAMILY_ID,
        "version": env!("CARGO_PKG_VERSION"),
        "capabilities_ready": crate::biomeos::MEASUREMENT_CAPABILITIES.len(),
        "uptime_seconds": uptime_secs(),
    })
}

// ─── Measurement Methods ─────────────────────────────────────────────────────

fn noise_decomposition(params: &Value) -> Result<Value, DispatchError> {
    let observed = extract_f64_array(params, "observed")?;
    let modeled = extract_f64_array(params, "modeled")?;

    if observed.len() != modeled.len() {
        return Err(DispatchError::InvalidParam(
            "observed and modeled must have equal length".into(),
        ));
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

fn anderson_validation(params: &Value) -> Result<Value, DispatchError> {
    let n_sites = extract_usize(params, "n_sites", DEFAULT_ANDERSON_N_SITES)?;
    let disorder = extract_f64(params, "disorder", DEFAULT_ANDERSON_DISORDER);
    let energy = extract_f64(params, "energy", DEFAULT_ENERGY);
    let n_realizations = extract_usize(params, "n_realizations", DEFAULT_ANDERSON_REALIZATIONS)?;
    let seed = extract_u64(params, "seed", DEFAULT_SEED);

    let gamma = crate::anderson::lyapunov_averaged(n_sites, disorder, energy, n_realizations, seed);
    let loc_length = crate::anderson::localization_length(gamma);

    Ok(serde_json::json!({
        "gamma": gamma,
        "localization_length": loc_length,
        "n_sites": n_sites,
        "disorder": disorder,
    }))
}

fn uncertainty_budget(params: &Value) -> Result<Value, DispatchError> {
    let data = extract_f64_array(params, "data")?;
    let confidence = extract_f64(params, "confidence", DEFAULT_CONFIDENCE);
    let n_bootstrap = extract_usize(params, "n_bootstrap", DEFAULT_N_BOOTSTRAP)?;
    let seed = extract_u64(params, "seed", DEFAULT_SEED);

    let boot_mean = crate::bootstrap::bootstrap_mean(&data, n_bootstrap, confidence, seed)?;
    let jk = crate::jackknife::jackknife_mean_variance(&data)?;

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

fn et0_propagation(params: &Value) -> Result<Value, DispatchError> {
    let tmax = require_f64(params, "temperature_max")?;
    let tmin = require_f64(params, "temperature_min")?;
    let wind = require_f64(params, "wind_speed")?;
    let sunshine = require_f64(params, "sunshine_hours")?;
    let lat = require_f64(params, "latitude")?;
    let doy_u64 = params
        .get("day_of_year")
        .and_then(Value::as_u64)
        .ok_or_else(|| DispatchError::MissingParam("day_of_year".into()))?;
    let doy = u16::try_from(doy_u64)
        .map_err(|_| DispatchError::InvalidParam("day_of_year out of u16 range".into()))?;
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

fn regime_classification(params: &Value) -> Result<Value, DispatchError> {
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

fn spectral_features(params: &Value) -> Result<Value, DispatchError> {
    let correlator = extract_f64_array(params, "correlator")?;
    let n_omega = extract_usize(params, "n_omega", DEFAULT_N_OMEGA)?;
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

fn parity_check(params: &Value) -> Result<Value, DispatchError> {
    let cpu_values = extract_f64_array(params, "cpu_values")?;
    let gpu_values = extract_f64_array(params, "gpu_values")?;
    let tolerance = extract_f64(params, "tolerance", crate::tol::EXACT);

    if cpu_values.len() != gpu_values.len() {
        return Err(DispatchError::InvalidParam(
            "cpu_values and gpu_values must have equal length".into(),
        ));
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

fn freeze_out(params: &Value) -> Result<Value, DispatchError> {
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

    let fit = crate::freeze_out::grid_fit_2d(&config)?;

    Ok(serde_json::json!({
        "t0": fit.t0,
        "kappa2": fit.kappa2,
        "chi_squared": fit.chi_squared,
        "chi2_per_dof": fit.chi2_per_dof,
    }))
}

// ─── Extended Measurement Methods ────────────────────────────────────────────

fn bootstrap(params: &Value) -> Result<Value, DispatchError> {
    let data = extract_f64_array(params, "data")?;
    let statistic = params
        .get("statistic")
        .and_then(Value::as_str)
        .unwrap_or("mean");
    let n_replicates = extract_usize(params, "n_replicates", DEFAULT_N_BOOTSTRAP)?;
    let confidence = extract_f64(params, "confidence", DEFAULT_CONFIDENCE);
    let seed = extract_u64(params, "seed", DEFAULT_SEED);

    let result = match statistic {
        "median" => crate::bootstrap::bootstrap_median(&data, n_replicates, confidence, seed)?,
        "std" => crate::bootstrap::bootstrap_std(&data, n_replicates, confidence, seed)?,
        _ => crate::bootstrap::bootstrap_mean(&data, n_replicates, confidence, seed)?,
    };

    Ok(serde_json::json!({
        "statistic": statistic,
        "estimate": result.estimate,
        "ci_lower": result.ci_lower,
        "ci_upper": result.ci_upper,
        "std_error": result.std_error,
    }))
}

fn rarefaction(params: &Value) -> Result<Value, DispatchError> {
    let counts = extract_u64_array(params, "counts")?;
    let depths = extract_u64_array(params, "depths")?;

    let curve = crate::rarefaction::analytical_rarefaction(&counts, &depths);
    let shannon = crate::rarefaction::shannon_diversity(&counts);
    let simpson = crate::rarefaction::simpson_diversity(&counts);
    let evn = crate::rarefaction::evenness(&counts);
    let taxa = crate::rarefaction::taxa_detected(&counts);

    Ok(serde_json::json!({
        "rarefaction_curve": curve,
        "shannon": shannon,
        "simpson": simpson,
        "evenness": evn,
        "taxa_detected": taxa,
    }))
}

fn drift(params: &Value) -> Result<Value, DispatchError> {
    let pop_size = extract_usize(params, "pop_size", 1000)?;
    let selection = extract_f64(params, "selection", 0.01);
    let initial_freq = extract_f64(params, "initial_freq", 0.5);
    let n_trials = extract_usize(params, "n_trials", 1000)?;
    let seed = extract_u64(params, "seed", DEFAULT_SEED);

    let n_fixed = crate::drift::wright_fisher_fixation_batch(
        pop_size,
        selection,
        initial_freq,
        n_trials,
        seed,
    );
    let empirical_prob = crate::cast::usize_f64(n_fixed) / crate::cast::usize_f64(n_trials);
    let kimura = crate::drift::kimura_fixation_prob(pop_size, selection, initial_freq);

    Ok(serde_json::json!({
        "n_fixed": n_fixed,
        "n_trials": n_trials,
        "fixation_probability": empirical_prob,
        "kimura_analytical": kimura,
        "pop_size": pop_size,
        "selection": selection,
    }))
}

fn band_edge(params: &Value) -> Result<Value, DispatchError> {
    let potential = extract_f64_array(params, "potential")?;
    let hopping = extract_f64(params, "hopping", 1.0);
    let e_lo = extract_f64(params, "e_lo", -5.0);
    let e_hi = extract_f64(params, "e_hi", 5.0);
    let n_points = extract_usize(params, "n_points", 1000)?;

    let edges = crate::band_structure::find_band_edges(&potential, hopping, e_lo, e_hi, n_points);
    let n_bands = crate::band_structure::count_bands(&potential, hopping, e_lo, e_hi, n_points);

    Ok(serde_json::json!({
        "band_edges": edges,
        "n_bands": n_bands,
        "e_lo": e_lo,
        "e_hi": e_hi,
    }))
}

fn rare_biosphere(params: &Value) -> Result<Value, DispatchError> {
    let counts = extract_u64_array(params, "counts")?;
    let target_power = extract_f64(params, "target_power", 0.95);

    let richness = crate::rare_biosphere::chao1(&counts);
    let n_total: u64 = counts.iter().sum();
    let taxa = crate::rarefaction::taxa_detected(&counts);

    Ok(serde_json::json!({
        "chao1": richness,
        "observed_taxa": taxa,
        "total_reads": n_total,
        "target_power": target_power,
    }))
}

fn gillespie(params: &Value) -> Result<Value, DispatchError> {
    let synthesis_rates = extract_f64_array(params, "synthesis_rates")?;
    let degradation_rate = require_f64(params, "degradation_rate")?;
    let initial = extract_u64(params, "initial", 100);
    let t_max = extract_f64(params, "t_max", 100.0);
    let n_trajectories = extract_usize(params, "n_trajectories", 100)?;
    let seed = extract_u64(params, "seed", DEFAULT_SEED);

    let total_synthesis: f64 = synthesis_rates.iter().sum();
    let analytical_mean = crate::gillespie::steady_state_mean(total_synthesis, degradation_rate);

    let batch = crate::gillespie::birth_death_ssa_batch(
        &synthesis_rates,
        degradation_rate,
        initial,
        t_max,
        n_trajectories,
        t_max * 0.1,
        seed,
    );

    Ok(serde_json::json!({
        "analytical_steady_state": analytical_mean,
        "ensemble_mean": batch.mean,
        "ensemble_variance": batch.variance,
        "n_trajectories": n_trajectories,
    }))
}

fn bistable(params: &Value) -> Result<Value, DispatchError> {
    let initial_cdg = extract_f64(params, "initial_cdg", 0.5);
    let dt = extract_f64(params, "dt", 0.01);
    let n_steps = extract_usize(params, "n_steps", 10_000)?;

    let state0 = [initial_cdg, 0.0, 0.0, 0.0, 0.0];
    let params_ode = crate::bistable::BistableParams::default();

    let final_state = crate::bistable::integrate(&state0, &params_ode, dt, n_steps);

    Ok(serde_json::json!({
        "final_state": final_state.to_vec(),
        "cdg_concentration": final_state[0],
        "dt": dt,
        "n_steps": n_steps,
    }))
}

fn quasispecies(params: &Value) -> Result<Value, DispatchError> {
    let sigma = require_f64(params, "sigma")?;
    let genome_length = extract_usize(params, "genome_length", 100)?;
    let mu = extract_f64(params, "mu", 0.01);

    let threshold = crate::quasispecies::error_threshold(sigma, genome_length);
    let master_freq = crate::quasispecies::master_frequency_analytical(sigma, mu, genome_length);
    let fitness = crate::quasispecies::mean_fitness(sigma, master_freq);

    Ok(serde_json::json!({
        "error_threshold": threshold,
        "master_frequency": master_freq,
        "mean_fitness": fitness,
        "sigma": sigma,
        "genome_length": genome_length,
        "mu": mu,
    }))
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn extract_f64_array(params: &Value, key: &str) -> Result<Vec<f64>, DispatchError> {
    params
        .get(key)
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(Value::as_f64).collect::<Vec<_>>())
        .ok_or_else(|| DispatchError::MissingParam(key.into()))
}

fn extract_u64_array(params: &Value, key: &str) -> Result<Vec<u64>, DispatchError> {
    params
        .get(key)
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(Value::as_u64).collect::<Vec<_>>())
        .ok_or_else(|| DispatchError::MissingParam(key.into()))
}

fn extract_f64(params: &Value, key: &str, default: f64) -> f64 {
    params.get(key).and_then(Value::as_f64).unwrap_or(default)
}

fn require_f64(params: &Value, key: &str) -> Result<f64, DispatchError> {
    params
        .get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| DispatchError::MissingParam(key.into()))
}

fn extract_u64(params: &Value, key: &str, default: u64) -> u64 {
    params.get(key).and_then(Value::as_u64).unwrap_or(default)
}

fn extract_usize(params: &Value, key: &str, default: u64) -> Result<usize, DispatchError> {
    let v = extract_u64(params, key, default);
    usize::try_from(v)
        .map_err(|_| DispatchError::InvalidParam(format!("{key} too large for usize")))
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]
mod tests {
    use super::*;

    #[test]
    fn dispatch_unknown_method_returns_error() {
        let result = dispatch("nonexistent.method", &Value::Null);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, crate::error::DispatchError::MethodNotFound(_)),
            "expected MethodNotFound, got {err:?}"
        );
        assert!(err.to_string().contains("nonexistent.method"));
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
            "tolerance": crate::tol::ANALYTICAL,
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
            "tolerance": crate::tol::ANALYTICAL,
        });
        let result = dispatch("measurement.parity_check", &params);
        assert!(result.is_ok());
        let v = result.unwrap();
        assert_eq!(v["parity"], false);
    }

    #[test]
    fn dispatch_bootstrap_mean() {
        let params = serde_json::json!({
            "data": [1.0, 2.0, 3.0, 4.0, 5.0],
            "n_replicates": 200,
        });
        let v = dispatch("measurement.bootstrap", &params).unwrap();
        assert_eq!(v["statistic"], "mean");
        assert!(v["estimate"].as_f64().unwrap() > 0.0);
        assert!(v["ci_lower"].as_f64().unwrap() < v["ci_upper"].as_f64().unwrap());
    }

    #[test]
    fn dispatch_bootstrap_median() {
        let params = serde_json::json!({
            "data": [1.0, 2.0, 3.0, 4.0, 5.0],
            "statistic": "median",
            "n_replicates": 200,
        });
        let v = dispatch("measurement.bootstrap", &params).unwrap();
        assert_eq!(v["statistic"], "median");
    }

    #[test]
    fn dispatch_rarefaction() {
        let params = serde_json::json!({
            "counts": [100, 200, 300, 50],
            "depths": [100, 200, 400],
        });
        let v = dispatch("measurement.rarefaction", &params).unwrap();
        assert!(v["shannon"].as_f64().unwrap() > 0.0);
        assert!(v["simpson"].as_f64().unwrap() > 0.0);
        assert_eq!(v["taxa_detected"], 4);
    }

    #[test]
    fn dispatch_drift() {
        let params = serde_json::json!({
            "pop_size": 100,
            "selection": 0.01,
            "n_trials": 50,
        });
        let v = dispatch("measurement.drift", &params).unwrap();
        assert!(v["kimura_analytical"].as_f64().unwrap() > 0.0);
        assert_eq!(v["n_trials"], 50);
    }

    #[test]
    fn dispatch_band_edge() {
        let params = serde_json::json!({
            "potential": [0.0, 0.0, 0.0, 0.0],
            "hopping": 1.0,
            "n_points": 100,
        });
        let v = dispatch("measurement.band_edge", &params).unwrap();
        assert!(v["n_bands"].as_u64().unwrap() > 0);
    }

    #[test]
    fn dispatch_rare_biosphere() {
        let params = serde_json::json!({
            "counts": [100, 1, 1, 50, 200],
        });
        let v = dispatch("measurement.rare_biosphere", &params).unwrap();
        assert!(v["chao1"].as_f64().unwrap() >= 5.0);
        assert_eq!(v["observed_taxa"], 5);
    }

    #[test]
    fn dispatch_gillespie() {
        let params = serde_json::json!({
            "synthesis_rates": [10.0],
            "degradation_rate": 0.1,
            "n_trajectories": 10,
            "t_max": 50.0,
        });
        let v = dispatch("measurement.gillespie", &params).unwrap();
        assert!(v["analytical_steady_state"].as_f64().unwrap() > 0.0);
    }

    #[test]
    fn dispatch_bistable() {
        let params = serde_json::json!({
            "initial_cdg": 0.5,
            "n_steps": 100,
        });
        let v = dispatch("measurement.bistable", &params).unwrap();
        assert!(v["cdg_concentration"].as_f64().is_some());
    }

    #[test]
    fn dispatch_quasispecies() {
        let params = serde_json::json!({
            "sigma": 10.0,
            "genome_length": 100,
            "mu": 0.01,
        });
        let v = dispatch("measurement.quasispecies", &params).unwrap();
        let threshold = v["error_threshold"].as_f64().unwrap();
        assert!(threshold > 0.0 && threshold < 1.0);
        assert!(v["master_frequency"].as_f64().unwrap() > 0.0);
    }
}
