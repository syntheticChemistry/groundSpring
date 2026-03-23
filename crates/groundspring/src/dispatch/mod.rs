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
//!
//! # Module structure
//!
//! - `defaults` — Named dispatch constants for parameter resolution
//! - `extract` — JSON-RPC parameter extraction helpers
//! - `lifecycle` — Health, capability, and status methods
//! - `measurement` — Domain-specific measurement method implementations

mod defaults;
mod extract;
mod lifecycle;
mod measurement;

use serde_json::Value;

use crate::error::DispatchError;

pub use lifecycle::init_start_time;

/// Known legacy prefixes that older callers may prepend to method names.
///
/// `normalize_method` strips these so both `"groundspring.measurement.bootstrap"`
/// and `"measurement.bootstrap"` route to the same handler.
///
/// Absorbed from barraCuda v0.3.7 / wetSpring V132 `normalize_method()` pattern.
const LEGACY_PREFIXES: &[&str] = &["groundspring.", "barracuda."];

/// Strip legacy primal-name prefixes from a JSON-RPC method name.
///
/// The ecosystem Semantic Method Naming Standard uses bare `domain.operation`
/// names (e.g., `"measurement.bootstrap"`). Older callers may prefix with the
/// primal name (`"groundspring.measurement.bootstrap"`). This function
/// normalizes both forms to the canonical bare name.
///
/// Returns the input unchanged if no legacy prefix is found.
///
/// Absorbed from barraCuda v0.3.7 / wetSpring V132.
#[must_use]
pub fn normalize_method(method: &str) -> &str {
    for prefix in LEGACY_PREFIXES {
        if let Some(stripped) = method.strip_prefix(prefix) {
            return stripped;
        }
    }
    method
}

/// Dispatch a JSON-RPC method call to the appropriate library function.
///
/// Normalizes legacy-prefixed method names before routing. Returns
/// `Ok(result_json)` on success or a typed [`DispatchError`] on failure.
/// Unknown methods return [`DispatchError::MethodNotFound`].
///
/// # Errors
///
/// Returns `Err` if the method is unknown, required parameters are missing,
/// or the underlying library call fails with an [`crate::error::InputError`].
pub fn dispatch(method: &str, params: &Value) -> Result<Value, DispatchError> {
    let method = normalize_method(method);
    match method {
        "health.check" | "health" => Ok(lifecycle::health_check()),
        "health.liveness" => Ok(lifecycle::health_liveness()),
        "health.readiness" => Ok(lifecycle::health_readiness()),
        "capability.list" | "capabilities.list" => Ok(lifecycle::capability_list()),
        "lifecycle.status" => Ok(lifecycle::lifecycle_status()),

        "measurement.noise_decomposition" => measurement::noise_decomposition(params),
        "measurement.anderson_validation" => measurement::anderson_validation(params),
        "measurement.uncertainty_budget" => measurement::uncertainty_budget(params),
        "measurement.et0_propagation" => measurement::et0_propagation(params),
        "measurement.regime_classification" => measurement::regime_classification(params),
        "measurement.spectral_features" => measurement::spectral_features(params),
        "measurement.parity_check" => measurement::parity_check(params),
        "measurement.freeze_out" => measurement::freeze_out(params),

        "measurement.bootstrap" => measurement::bootstrap(params),
        "measurement.rarefaction" => measurement::rarefaction(params),
        "measurement.drift" => measurement::drift(params),
        "measurement.band_edge" => measurement::band_edge(params),
        "measurement.rare_biosphere" => measurement::rare_biosphere(params),
        "measurement.gillespie" => measurement::gillespie(params),
        "measurement.bistable" => measurement::bistable(params),
        "measurement.quasispecies" => measurement::quasispecies(params),

        _ => Err(DispatchError::MethodNotFound(method.to_owned())),
    }
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

    // ── normalize_method tests ──────────────────────────────────────────

    #[test]
    fn normalize_method_strips_groundspring_prefix() {
        assert_eq!(
            normalize_method("groundspring.measurement.bootstrap"),
            "measurement.bootstrap"
        );
    }

    #[test]
    fn normalize_method_strips_barracuda_prefix() {
        assert_eq!(
            normalize_method("barracuda.measurement.drift"),
            "measurement.drift"
        );
    }

    #[test]
    fn normalize_method_passes_bare_name_through() {
        assert_eq!(
            normalize_method("measurement.bootstrap"),
            "measurement.bootstrap"
        );
    }

    #[test]
    fn normalize_method_preserves_unknown_prefix() {
        assert_eq!(
            normalize_method("wetspring.measurement.test"),
            "wetspring.measurement.test"
        );
    }

    #[test]
    fn dispatch_legacy_prefixed_method() {
        init_start_time();
        let v = dispatch("groundspring.health.check", &Value::Null).unwrap();
        assert_eq!(v["status"], "healthy");
    }

    #[test]
    fn dispatch_capabilities_list_plural() {
        let v = dispatch("capabilities.list", &Value::Null).unwrap();
        assert_eq!(v["domain"], "measurement");
    }
}
