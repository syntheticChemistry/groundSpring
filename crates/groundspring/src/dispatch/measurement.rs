// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Measurement method implementations for JSON-RPC dispatch.
//!
//! Each function maps a single `measurement.*` capability method to the
//! underlying groundSpring library call, translating JSON-RPC parameters
//! into typed Rust arguments and serializing results back to JSON.

use serde_json::Value;

use crate::error::DispatchError;

use super::defaults::{
    DEFAULT_ANDERSON_DISORDER, DEFAULT_ANDERSON_N_SITES, DEFAULT_ANDERSON_REALIZATIONS,
    DEFAULT_CONFIDENCE, DEFAULT_ELEVATION_M, DEFAULT_ENERGY, DEFAULT_K2_HI, DEFAULT_K2_LO,
    DEFAULT_K2_STEP, DEFAULT_N_BOOTSTRAP, DEFAULT_N_OMEGA, DEFAULT_OMEGA_STEP,
    DEFAULT_REGIME_MARGIN, DEFAULT_REGULARIZATION, DEFAULT_RHMAX_PCT, DEFAULT_RHMIN_PCT,
    DEFAULT_SEED, DEFAULT_SIGMA, DEFAULT_T0_HI, DEFAULT_T0_LO, DEFAULT_T0_STEP, DEFAULT_TAU_STEP,
};
use super::extract::{
    extract_f64, extract_f64_array, extract_u64, extract_u64_array, extract_usize, require_f64,
};

pub(super) fn noise_decomposition(params: &Value) -> Result<Value, DispatchError> {
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

pub(super) fn anderson_validation(params: &Value) -> Result<Value, DispatchError> {
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

pub(super) fn uncertainty_budget(params: &Value) -> Result<Value, DispatchError> {
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

pub(super) fn et0_propagation(params: &Value) -> Result<Value, DispatchError> {
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

pub(super) fn regime_classification(params: &Value) -> Result<Value, DispatchError> {
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

pub(super) fn spectral_features(params: &Value) -> Result<Value, DispatchError> {
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

pub(super) fn parity_check(params: &Value) -> Result<Value, DispatchError> {
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

pub(super) fn freeze_out(params: &Value) -> Result<Value, DispatchError> {
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

pub(super) fn bootstrap(params: &Value) -> Result<Value, DispatchError> {
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

pub(super) fn rarefaction(params: &Value) -> Result<Value, DispatchError> {
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

pub(super) fn drift(params: &Value) -> Result<Value, DispatchError> {
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

pub(super) fn band_edge(params: &Value) -> Result<Value, DispatchError> {
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

pub(super) fn rare_biosphere(params: &Value) -> Result<Value, DispatchError> {
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

pub(super) fn gillespie(params: &Value) -> Result<Value, DispatchError> {
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

pub(super) fn bistable(params: &Value) -> Result<Value, DispatchError> {
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

pub(super) fn quasispecies(params: &Value) -> Result<Value, DispatchError> {
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
