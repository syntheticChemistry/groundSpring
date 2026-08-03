// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Niche deployment self-knowledge for groundSpring.
//!
//! A Spring is a niche validation domain — not a primal. It proves that
//! scientific Python baselines can be faithfully ported to sovereign
//! Rust + GPU compute using the ecoPrimals stack. The niche deploys as
//! a biomeOS graph (`graphs/groundspring_deploy.toml`) that composes
//! real primals (`BearDog`, `Songbird`, `ToadStool`, etc.).
//!
//! This module holds the niche's self-knowledge:
//! - Identity (who am I?)
//! - Capabilities (what do I expose via biomeOS?)
//! - Semantic mappings (capability domain → library methods)
//! - Dependencies (what primals do I need?)
//! - Operation costs (scheduling hints for biomeOS)
//!
//! Other modules (`biomeos`, `dispatch`, `ipc`) reference these constants
//! rather than duplicating string literals. groundSpring only knows itself —
//! it discovers other primals at runtime via capability-based discovery.

pub mod capabilities;

pub use capabilities::{DEPENDENCIES, FEATURE_GATES};

/// Niche identity — used in all JSON-RPC, provenance, and IPC interactions.
///
/// Delegates to [`crate::primal_names::SELF_ID`].
pub const NICHE_ID: &str = crate::primal_names::SELF_ID;

/// Human-readable niche description for biomeOS registration.
pub const NICHE_DESCRIPTION: &str = "Measurement noise characterization and validation";

/// Niche category for biomeOS deployment.
pub const NICHE_CATEGORY: &str = "science";

/// Capability domain for all groundSpring methods.
pub const DOMAIN: &str = "measurement";

/// All capabilities this niche exposes to biomeOS.
///
/// Each string is a fully qualified capability name (`{domain}.{method}`)
/// that biomeOS can route via `capability.call`.
pub const CAPABILITIES: &[&str] = &[
    "measurement.noise_decomposition",
    "measurement.anderson_validation",
    "measurement.parity_check",
    "measurement.et0_propagation",
    "measurement.regime_classification",
    "measurement.uncertainty_budget",
    "measurement.spectral_features",
    "measurement.freeze_out",
    "measurement.bootstrap",
    "measurement.rarefaction",
    "measurement.drift",
    "measurement.band_edge",
    "measurement.rare_biosphere",
    "measurement.gillespie",
    "measurement.bistable",
    "measurement.quasispecies",
];

/// Semantic mappings: short operation name → fully qualified capability.
///
/// biomeOS uses these during domain registration so
/// `capability.call { domain: "measurement", operation: "noise_decomposition" }`
/// routes to the correct JSON-RPC method on our socket.
pub const SEMANTIC_MAPPINGS: &[(&str, &str)] = &[
    ("noise_decomposition", "measurement.noise_decomposition"),
    ("anderson_validation", "measurement.anderson_validation"),
    ("parity_check", "measurement.parity_check"),
    ("et0_propagation", "measurement.et0_propagation"),
    ("regime_classification", "measurement.regime_classification"),
    ("uncertainty_budget", "measurement.uncertainty_budget"),
    ("spectral_features", "measurement.spectral_features"),
    ("freeze_out", "measurement.freeze_out"),
    ("bootstrap", "measurement.bootstrap"),
    ("rarefaction", "measurement.rarefaction"),
    ("drift", "measurement.drift"),
    ("band_edge", "measurement.band_edge"),
    ("rare_biosphere", "measurement.rare_biosphere"),
    ("gillespie", "measurement.gillespie"),
    ("bistable", "measurement.bistable"),
    ("quasispecies", "measurement.quasispecies"),
];

/// Cost estimates for biomeOS Pathway Learner scheduling.
///
/// Each entry: `(capability, estimated_ms, gpu_beneficial)`.
/// `gpu_beneficial = true` means the operation benefits from GPU dispatch.
/// Times are representative for typical validation workloads on i9-12900K / RTX 4070.
pub const COST_ESTIMATES: &[(&str, u32, bool)] = &[
    ("measurement.noise_decomposition", 5, false),
    ("measurement.anderson_validation", 50, true),
    ("measurement.parity_check", 10, false),
    ("measurement.et0_propagation", 15, true),
    ("measurement.regime_classification", 30, true),
    ("measurement.uncertainty_budget", 20, true),
    ("measurement.spectral_features", 25, true),
    ("measurement.freeze_out", 100, true),
    ("measurement.bootstrap", 40, true),
    ("measurement.rarefaction", 10, true),
    ("measurement.drift", 60, true),
    ("measurement.band_edge", 15, true),
    ("measurement.rare_biosphere", 20, true),
    ("measurement.gillespie", 80, true),
    ("measurement.bistable", 30, false),
    ("measurement.quasispecies", 5, false),
];

/// Consumed capabilities — what groundSpring calls on other primals.
///
/// groundSpring discovers these at runtime via the discovery provider; it never
/// hardcodes which primal provides them.
pub const CONSUMED_CAPABILITIES: &[&str] = &[
    "crypto.sign",
    "crypto.verify",
    "discovery.find_primals",
    "discovery.query",
    "compute.execute",
    "compute.submit",
    "storage.put",
    "storage.get",
    "data.ncbi_search",
    "data.ncbi_fetch",
    "data.noaa_ghcnd",
    "data.iris_stations",
    "data.iris_events",
    "security.audit_log",
];

/// Number of barraCuda delegations (CPU + GPU).
///
/// Updated V118: 8 new dispatch methods wired through existing barraCuda
/// primitives (`bootstrap`, `rarefaction`, `drift`, `band_edge`,
/// `rare_biosphere`, `gillespie`, `bistable`, `quasispecies`).
pub const DELEGATION_COUNT: (u32, u32) = (67, 43);

// ─── Structured capability metadata (ludoSpring V19 pattern) ────────────────

/// Input requirements for a single capability.
pub struct OperationDeps {
    /// Fully qualified capability name.
    pub capability: &'static str,
    /// Required input fields (JSON-RPC params).
    pub required_inputs: &'static [&'static str],
    /// Optional input fields with defaults.
    pub optional_inputs: &'static [&'static str],
    /// Consumed capabilities called during execution.
    pub calls: &'static [&'static str],
}

/// Scheduling metadata for a single capability.
pub struct CostEstimate {
    /// Fully qualified capability name.
    pub capability: &'static str,
    /// Estimated wall-clock ms on i9-12900K / RTX 4070.
    pub estimated_ms: u32,
    /// Whether GPU dispatch reduces latency.
    pub gpu_beneficial: bool,
    /// Approximate peak memory (bytes, 0 = negligible).
    pub peak_memory_bytes: u64,
    /// Whether the operation is deterministic across runs.
    pub deterministic: bool,
}

/// Input requirements per capability for biomeOS orchestration.
///
/// biomeOS Pathway Learner uses these to validate inputs before routing
/// and to construct dependency graphs for multi-step pipelines.
///
/// Field names match the JSON-RPC parameter keys in the `dispatch` module
/// (available behind the `biomeos` feature gate).
pub static OPERATION_DEPENDENCIES: &[OperationDeps] = &[
    OperationDeps {
        capability: "measurement.noise_decomposition",
        required_inputs: &["observed", "modeled"],
        optional_inputs: &[],
        calls: &[],
    },
    OperationDeps {
        capability: "measurement.anderson_validation",
        required_inputs: &[],
        optional_inputs: &["n_sites", "disorder", "energy", "n_realizations", "seed"],
        calls: &["compute.execute"],
    },
    OperationDeps {
        capability: "measurement.parity_check",
        required_inputs: &["cpu_values", "gpu_values"],
        optional_inputs: &["tolerance"],
        calls: &[],
    },
    OperationDeps {
        capability: "measurement.et0_propagation",
        required_inputs: &[
            "temperature_max",
            "temperature_min",
            "wind_speed",
            "sunshine_hours",
            "latitude",
            "day_of_year",
        ],
        optional_inputs: &["elevation", "rhmax", "rhmin"],
        calls: &[],
    },
    OperationDeps {
        capability: "measurement.regime_classification",
        required_inputs: &["eigenvalues"],
        optional_inputs: &["margin"],
        calls: &["compute.execute"],
    },
    OperationDeps {
        capability: "measurement.uncertainty_budget",
        required_inputs: &["data"],
        optional_inputs: &["confidence", "n_bootstrap", "seed"],
        calls: &[],
    },
    OperationDeps {
        capability: "measurement.spectral_features",
        required_inputs: &["correlator"],
        optional_inputs: &["n_omega", "regularization"],
        calls: &[],
    },
    OperationDeps {
        capability: "measurement.freeze_out",
        required_inputs: &["observed", "mu_b"],
        optional_inputs: &[
            "sigma", "t0_lo", "t0_hi", "t0_step", "k2_lo", "k2_hi", "k2_step",
        ],
        calls: &["compute.execute"],
    },
    OperationDeps {
        capability: "measurement.bootstrap",
        required_inputs: &["data"],
        optional_inputs: &["statistic", "n_replicates", "confidence", "seed"],
        calls: &[],
    },
    OperationDeps {
        capability: "measurement.rarefaction",
        required_inputs: &["counts", "depths"],
        optional_inputs: &[],
        calls: &[],
    },
    OperationDeps {
        capability: "measurement.drift",
        required_inputs: &[],
        optional_inputs: &["pop_size", "selection", "initial_freq", "n_trials", "seed"],
        calls: &["compute.execute"],
    },
    OperationDeps {
        capability: "measurement.band_edge",
        required_inputs: &["potential"],
        optional_inputs: &["hopping", "e_lo", "e_hi", "n_points"],
        calls: &[],
    },
    OperationDeps {
        capability: "measurement.rare_biosphere",
        required_inputs: &["counts"],
        optional_inputs: &["target_power"],
        calls: &[],
    },
    OperationDeps {
        capability: "measurement.gillespie",
        required_inputs: &["synthesis_rates", "degradation_rate"],
        optional_inputs: &["initial", "t_max", "n_trajectories", "seed"],
        calls: &["compute.execute"],
    },
    OperationDeps {
        capability: "measurement.bistable",
        required_inputs: &[],
        optional_inputs: &["initial_cdg", "dt", "n_steps"],
        calls: &[],
    },
    OperationDeps {
        capability: "measurement.quasispecies",
        required_inputs: &["sigma"],
        optional_inputs: &["genome_length", "mu"],
        calls: &[],
    },
];

/// Structured cost estimates per capability for biomeOS scheduling.
///
/// Times measured on i9-12900K / RTX 4070 with typical validation workloads.
/// biomeOS uses these for Pathway Learner scheduling and resource allocation.
pub static STRUCTURED_COST_ESTIMATES: &[CostEstimate] = &[
    CostEstimate {
        capability: "measurement.noise_decomposition",
        estimated_ms: 5,
        gpu_beneficial: false,
        peak_memory_bytes: 1024 * 1024,
        deterministic: true,
    },
    CostEstimate {
        capability: "measurement.anderson_validation",
        estimated_ms: 50,
        gpu_beneficial: true,
        peak_memory_bytes: 64 * 1024 * 1024,
        deterministic: true,
    },
    CostEstimate {
        capability: "measurement.parity_check",
        estimated_ms: 10,
        gpu_beneficial: false,
        peak_memory_bytes: 512 * 1024,
        deterministic: true,
    },
    CostEstimate {
        capability: "measurement.et0_propagation",
        estimated_ms: 15,
        gpu_beneficial: true,
        peak_memory_bytes: 8 * 1024 * 1024,
        deterministic: true,
    },
    CostEstimate {
        capability: "measurement.regime_classification",
        estimated_ms: 30,
        gpu_beneficial: true,
        peak_memory_bytes: 16 * 1024 * 1024,
        deterministic: false,
    },
    CostEstimate {
        capability: "measurement.uncertainty_budget",
        estimated_ms: 20,
        gpu_beneficial: true,
        peak_memory_bytes: 4 * 1024 * 1024,
        deterministic: true,
    },
    CostEstimate {
        capability: "measurement.spectral_features",
        estimated_ms: 25,
        gpu_beneficial: true,
        peak_memory_bytes: 8 * 1024 * 1024,
        deterministic: true,
    },
    CostEstimate {
        capability: "measurement.freeze_out",
        estimated_ms: 100,
        gpu_beneficial: true,
        peak_memory_bytes: 128 * 1024 * 1024,
        deterministic: true,
    },
    CostEstimate {
        capability: "measurement.bootstrap",
        estimated_ms: 40,
        gpu_beneficial: true,
        peak_memory_bytes: 16 * 1024 * 1024,
        deterministic: true,
    },
    CostEstimate {
        capability: "measurement.rarefaction",
        estimated_ms: 10,
        gpu_beneficial: true,
        peak_memory_bytes: 4 * 1024 * 1024,
        deterministic: true,
    },
    CostEstimate {
        capability: "measurement.drift",
        estimated_ms: 60,
        gpu_beneficial: true,
        peak_memory_bytes: 8 * 1024 * 1024,
        deterministic: true,
    },
    CostEstimate {
        capability: "measurement.band_edge",
        estimated_ms: 15,
        gpu_beneficial: true,
        peak_memory_bytes: 2 * 1024 * 1024,
        deterministic: true,
    },
    CostEstimate {
        capability: "measurement.rare_biosphere",
        estimated_ms: 20,
        gpu_beneficial: true,
        peak_memory_bytes: 4 * 1024 * 1024,
        deterministic: true,
    },
    CostEstimate {
        capability: "measurement.gillespie",
        estimated_ms: 80,
        gpu_beneficial: true,
        peak_memory_bytes: 32 * 1024 * 1024,
        deterministic: true,
    },
    CostEstimate {
        capability: "measurement.bistable",
        estimated_ms: 30,
        gpu_beneficial: false,
        peak_memory_bytes: 1024 * 1024,
        deterministic: true,
    },
    CostEstimate {
        capability: "measurement.quasispecies",
        estimated_ms: 5,
        gpu_beneficial: false,
        peak_memory_bytes: 512 * 1024,
        deterministic: true,
    },
];

/// Returns operation dependencies (wrapper for backward compatibility).
#[must_use]
pub const fn operation_dependencies() -> &'static [OperationDeps] {
    OPERATION_DEPENDENCIES
}

/// Returns structured cost estimates (wrapper for backward compatibility).
#[must_use]
pub const fn cost_estimates() -> &'static [CostEstimate] {
    STRUCTURED_COST_ESTIMATES
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capabilities_all_prefixed_with_domain() {
        for cap in CAPABILITIES {
            assert!(
                cap.starts_with(&format!("{DOMAIN}.")),
                "capability {cap} must start with {DOMAIN}."
            );
        }
    }

    #[test]
    fn semantic_mappings_match_capabilities() {
        for (_, method) in SEMANTIC_MAPPINGS {
            assert!(
                CAPABILITIES.contains(method),
                "mapping target {method} not in CAPABILITIES"
            );
        }
        assert_eq!(SEMANTIC_MAPPINGS.len(), CAPABILITIES.len());
    }

    #[test]
    fn cost_estimates_cover_all_capabilities() {
        for (cap, _, _) in COST_ESTIMATES {
            assert!(
                CAPABILITIES.contains(cap),
                "cost estimate for {cap} not in CAPABILITIES"
            );
        }
        assert_eq!(COST_ESTIMATES.len(), CAPABILITIES.len());
    }

    #[test]
    fn niche_id_matches_family_id_convention() {
        assert!(!NICHE_ID.is_empty());
        assert!(NICHE_ID.chars().all(|c| c.is_ascii_lowercase()));
    }

    #[test]
    fn dependencies_include_required_primals() {
        let required: Vec<&str> = DEPENDENCIES
            .iter()
            .filter(|(_, req, _)| *req)
            .map(|(id, _, _)| *id)
            .collect();
        assert!(required.contains(&crate::primal_names::roles::SECURITY));
        assert!(required.contains(&crate::primal_names::roles::DISCOVERY));
    }

    #[test]
    fn delegation_count_matches_known_total() {
        let (cpu, gpu) = DELEGATION_COUNT;
        assert_eq!(cpu + gpu, 110);
    }

    #[test]
    fn operation_deps_cover_all_capabilities() {
        let deps = operation_dependencies();
        assert_eq!(deps.len(), CAPABILITIES.len());
        for dep in deps {
            assert!(
                CAPABILITIES.contains(&dep.capability),
                "op dep for {} not in CAPABILITIES",
                dep.capability
            );
        }
    }

    #[test]
    fn structured_cost_estimates_cover_all_capabilities() {
        let costs = cost_estimates();
        assert_eq!(costs.len(), CAPABILITIES.len());
        for cost in costs {
            assert!(
                CAPABILITIES.contains(&cost.capability),
                "cost for {} not in CAPABILITIES",
                cost.capability
            );
            assert!(cost.estimated_ms > 0, "zero cost for {}", cost.capability);
        }
    }
}
