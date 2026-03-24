// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Centralized provenance registry for all Python baselines.
//!
//! Every experiment's Python baseline is registered here with its script
//! path, benchmark JSON path, experiment number, and domain. This module
//! is the single source of truth for "which Python script generated which
//! benchmark JSON" — matching neuralSpring S174's provenance header pattern.
//!
//! # Provenance contract
//!
//! Each entry asserts:
//! 1. The `script` exists in `control/` and is runnable with `python3`.
//! 2. The `benchmark_json` exists and contains `_provenance.validation_script`.
//! 3. The Rust validation binary (`validate_*`) compares against values in
//!    that JSON within named tolerances from [`crate::tol`].
//!
//! If any baseline drifts, the corresponding validation binary will fail
//! with exit code 1 — by design, not by accident.

/// A single experiment baseline entry.
#[derive(Debug, Clone, Copy)]
pub struct BaselineEntry {
    /// Experiment number (e.g., 1 for Exp 001).
    pub exp_id: u32,
    /// Short experiment name used in binary names and paths.
    pub name: &'static str,
    /// Path to the Python generation script (relative to repo root).
    pub script: &'static str,
    /// Path to the benchmark JSON (relative to repo root).
    pub benchmark_json: &'static str,
    /// Rust validation binary name.
    pub validator: &'static str,
    /// Science domain label.
    pub domain: &'static str,
}

/// Complete registry of all 29 Python baselines.
///
/// Ordered by experiment number. Each entry corresponds to one
/// `control/<experiment>/` directory and one `validate_*` binary.
pub const BASELINES: &[BaselineEntry] = &[
    BaselineEntry {
        exp_id: 1,
        name: "sensor_noise",
        script: "control/sensor_noise/sensor_noise_decomposition.py",
        benchmark_json: "control/sensor_noise/benchmark_sensor_noise.json",
        validator: "validate_decompose",
        domain: "measurement",
    },
    BaselineEntry {
        exp_id: 2,
        name: "observation_gap",
        script: "control/observation_gap/observation_gap.py",
        benchmark_json: "control/observation_gap/benchmark_observation_gap.json",
        validator: "validate_weather",
        domain: "measurement",
    },
    BaselineEntry {
        exp_id: 3,
        name: "error_propagation",
        script: "control/error_propagation/error_propagation_fao56.py",
        benchmark_json: "control/error_propagation/benchmark_error_propagation.json",
        validator: "validate_fao56",
        domain: "hydrology",
    },
    BaselineEntry {
        exp_id: 4,
        name: "sequencing_noise",
        script: "control/sequencing_noise/sequencing_noise.py",
        benchmark_json: "control/sequencing_noise/benchmark_sequencing_noise.json",
        validator: "validate_rarefaction",
        domain: "genomics",
    },
    BaselineEntry {
        exp_id: 5,
        name: "seismic",
        script: "control/seismic/seismic_inversion.py",
        benchmark_json: "control/seismic/benchmark_seismic.json",
        validator: "validate_seismic",
        domain: "geophysics",
    },
    BaselineEntry {
        exp_id: 6,
        name: "signal_specificity",
        script: "control/signal_specificity/signal_specificity.py",
        benchmark_json: "control/signal_specificity/benchmark_signal_specificity.json",
        validator: "validate_signal_specificity",
        domain: "biochemistry",
    },
    BaselineEntry {
        exp_id: 7,
        name: "rawr_resampling",
        script: "control/rawr_resampling/rawr_resampling.py",
        benchmark_json: "control/rawr_resampling/benchmark_rawr_resampling.json",
        validator: "validate_rawr",
        domain: "statistics",
    },
    BaselineEntry {
        exp_id: 8,
        name: "anderson_localization",
        script: "control/anderson_localization/anderson_localization.py",
        benchmark_json: "control/anderson_localization/benchmark_anderson_localization.json",
        validator: "validate_anderson",
        domain: "condensed_matter",
    },
    BaselineEntry {
        exp_id: 9,
        name: "quasiperiodic",
        script: "control/quasiperiodic/quasiperiodic_localization.py",
        benchmark_json: "control/quasiperiodic/benchmark_quasiperiodic.json",
        validator: "validate_quasiperiodic",
        domain: "condensed_matter",
    },
    BaselineEntry {
        exp_id: 10,
        name: "bistable_switching",
        script: "control/bistable_switching/bistable_switching.py",
        benchmark_json: "control/bistable_switching/benchmark_bistable.json",
        validator: "validate_bistable",
        domain: "biochemistry",
    },
    BaselineEntry {
        exp_id: 11,
        name: "multisignal_qs",
        script: "control/multisignal_qs/multisignal_qs.py",
        benchmark_json: "control/multisignal_qs/benchmark_multisignal.json",
        validator: "validate_multisignal",
        domain: "biochemistry",
    },
    BaselineEntry {
        exp_id: 12,
        name: "spin_transport",
        script: "control/spin_transport/spin_chain_transport.py",
        benchmark_json: "control/spin_transport/benchmark_spin_transport.json",
        validator: "validate_transport",
        domain: "condensed_matter",
    },
    BaselineEntry {
        exp_id: 13,
        name: "resampling_convergence",
        script: "control/resampling_convergence/resampling_convergence.py",
        benchmark_json: "control/resampling_convergence/benchmark_resampling_convergence.json",
        validator: "validate_resampling_conv",
        domain: "statistics",
    },
    BaselineEntry {
        exp_id: 14,
        name: "drift_selection",
        script: "control/drift_selection/drift_selection.py",
        benchmark_json: "control/drift_selection/benchmark_drift_selection.json",
        validator: "validate_drift",
        domain: "population_genetics",
    },
    BaselineEntry {
        exp_id: 15,
        name: "uncertainty_bridge",
        script: "control/uncertainty_bridge/uncertainty_bridge.py",
        benchmark_json: "control/uncertainty_bridge/benchmark_uncertainty_bridge.json",
        validator: "validate_uncertainty_bridge",
        domain: "cross_domain",
    },
    BaselineEntry {
        exp_id: 16,
        name: "rare_biosphere",
        script: "control/rare_biosphere/rare_biosphere.py",
        benchmark_json: "control/rare_biosphere/benchmark_rare_biosphere.json",
        validator: "validate_rare_biosphere",
        domain: "genomics",
    },
    BaselineEntry {
        exp_id: 17,
        name: "quasispecies_threshold",
        script: "control/quasispecies_threshold/quasispecies_threshold.py",
        benchmark_json: "control/quasispecies_threshold/benchmark_quasispecies.json",
        validator: "validate_quasispecies",
        domain: "evolutionary_biology",
    },
    BaselineEntry {
        exp_id: 18,
        name: "band_edge",
        script: "control/band_edge/band_edge.py",
        benchmark_json: "control/band_edge/benchmark_band_edge.json",
        validator: "validate_band_edge",
        domain: "condensed_matter",
    },
    BaselineEntry {
        exp_id: 19,
        name: "jackknife_estimation",
        script: "control/jackknife_estimation/jackknife_estimation.py",
        benchmark_json: "control/jackknife_estimation/benchmark_jackknife.json",
        validator: "validate_jackknife",
        domain: "statistics",
    },
    BaselineEntry {
        exp_id: 20,
        name: "freeze_out_inverse",
        script: "control/freeze_out_inverse/freeze_out_inverse.py",
        benchmark_json: "control/freeze_out_inverse/benchmark_freeze_out.json",
        validator: "validate_freeze_out",
        domain: "lattice_qcd",
    },
    BaselineEntry {
        exp_id: 21,
        name: "spectral_recon",
        script: "control/spectral_recon/spectral_recon.py",
        benchmark_json: "control/spectral_recon/benchmark_spectral_recon.json",
        validator: "validate_spectral_recon",
        domain: "lattice_qcd",
    },
    BaselineEntry {
        exp_id: 22,
        name: "et0_anderson_propagation",
        script: "control/et0_anderson_propagation/et0_anderson_propagation.py",
        benchmark_json: "control/et0_anderson_propagation/benchmark_et0_anderson.json",
        validator: "validate_et0_anderson",
        domain: "hydrology",
    },
    BaselineEntry {
        exp_id: 23,
        name: "notill_sampling",
        script: "control/notill_sampling/notill_sampling.py",
        benchmark_json: "control/notill_sampling/benchmark_notill_sampling.json",
        validator: "validate_notill_sampling",
        domain: "soil_science",
    },
    BaselineEntry {
        exp_id: 24,
        name: "aggregate_stability",
        script: "control/aggregate_stability/aggregate_stability.py",
        benchmark_json: "control/aggregate_stability/benchmark_aggregate_stability.json",
        validator: "validate_aggregate_stability",
        domain: "soil_science",
    },
    BaselineEntry {
        exp_id: 25,
        name: "precision_drift",
        script: "control/precision_drift/precision_drift.py",
        benchmark_json: "control/precision_drift/benchmark_precision_drift.json",
        validator: "validate_precision_drift",
        domain: "numerical_methods",
    },
    BaselineEntry {
        exp_id: 26,
        name: "size_convergence",
        script: "control/size_convergence/size_convergence.py",
        benchmark_json: "control/size_convergence/benchmark_size_convergence.json",
        validator: "validate_size_convergence",
        domain: "numerical_methods",
    },
    BaselineEntry {
        exp_id: 27,
        name: "vendor_parity",
        script: "control/vendor_parity/vendor_parity.py",
        benchmark_json: "control/vendor_parity/benchmark_vendor_parity.json",
        validator: "validate_vendor_parity",
        domain: "gpu_validation",
    },
    BaselineEntry {
        exp_id: 28,
        name: "npu_anderson",
        script: "control/npu_anderson/npu_anderson.py",
        benchmark_json: "control/npu_anderson/benchmark_npu_anderson.json",
        validator: "validate_npu_anderson",
        domain: "neuromorphic",
    },
    BaselineEntry {
        exp_id: 29,
        name: "et0_methods",
        script: "control/et0_methods/et0_methods.py",
        benchmark_json: "control/et0_methods/benchmark_et0_methods.json",
        validator: "validate_et0_methods",
        domain: "hydrology",
    },
];

/// Number of registered baselines.
pub const BASELINE_COUNT: usize = BASELINES.len();

/// Look up a baseline by experiment ID.
#[must_use]
pub fn by_exp_id(exp_id: u32) -> Option<&'static BaselineEntry> {
    BASELINES.iter().find(|e| e.exp_id == exp_id)
}

/// Look up a baseline by experiment name.
#[must_use]
pub fn by_name(name: &str) -> Option<&'static BaselineEntry> {
    BASELINES.iter().find(|e| e.name == name)
}

/// Look up a baseline by validator binary name.
#[must_use]
pub fn by_validator(validator: &str) -> Option<&'static BaselineEntry> {
    BASELINES.iter().find(|e| e.validator == validator)
}

/// List all baselines in a given science domain.
pub fn by_domain(domain: &str) -> impl Iterator<Item = &'static BaselineEntry> {
    BASELINES.iter().filter(move |e| e.domain == domain)
}

/// Unique domain names across all baselines.
#[must_use]
pub fn domains() -> Vec<&'static str> {
    let mut ds: Vec<&str> = BASELINES.iter().map(|e| e.domain).collect();
    ds.sort_unstable();
    ds.dedup();
    ds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_count_is_29() {
        assert_eq!(BASELINE_COUNT, 29);
    }

    #[test]
    fn all_exp_ids_unique() {
        let mut ids: Vec<u32> = BASELINES.iter().map(|e| e.exp_id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), BASELINE_COUNT);
    }

    #[test]
    fn all_names_unique() {
        let mut names: Vec<&str> = BASELINES.iter().map(|e| e.name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), BASELINE_COUNT);
    }

    #[test]
    fn all_validators_unique() {
        let mut vals: Vec<&str> = BASELINES.iter().map(|e| e.validator).collect();
        vals.sort_unstable();
        vals.dedup();
        assert_eq!(vals.len(), BASELINE_COUNT);
    }

    #[test]
    fn lookup_by_exp_id() {
        let entry = by_exp_id(1);
        assert!(entry.is_some());
        assert_eq!(entry.map(|e| e.name), Some("sensor_noise"));
    }

    #[test]
    fn lookup_by_name() {
        let entry = by_name("anderson_localization");
        assert!(entry.is_some());
        assert_eq!(entry.map(|e| e.exp_id), Some(8));
    }

    #[test]
    fn lookup_by_validator() {
        let entry = by_validator("validate_fao56");
        assert!(entry.is_some());
        assert_eq!(entry.map(|e| e.exp_id), Some(3));
    }

    #[test]
    fn lookup_missing_returns_none() {
        assert!(by_exp_id(999).is_none());
        assert!(by_name("nonexistent").is_none());
        assert!(by_validator("validate_nothing").is_none());
    }

    #[test]
    fn domain_filter() {
        let condensed: Vec<_> = by_domain("condensed_matter").collect();
        assert!(condensed.len() >= 3);
        for entry in &condensed {
            assert_eq!(entry.domain, "condensed_matter");
        }
    }

    #[test]
    fn domains_are_nonempty() {
        let ds = domains();
        assert!(
            ds.len() >= 8,
            "expected at least 8 domains, got {}",
            ds.len()
        );
    }

    #[test]
    fn scripts_have_py_extension() {
        for entry in BASELINES {
            assert!(
                std::path::Path::new(entry.script)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("py")),
                "{} script does not end with .py: {}",
                entry.name,
                entry.script
            );
        }
    }

    #[test]
    fn benchmark_jsons_have_json_extension() {
        for entry in BASELINES {
            assert!(
                std::path::Path::new(entry.benchmark_json)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("json")),
                "{} benchmark_json does not end with .json: {}",
                entry.name,
                entry.benchmark_json
            );
        }
    }
}
