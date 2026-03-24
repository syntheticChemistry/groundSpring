// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team
#![forbid(unsafe_code)]
#![deny(clippy::expect_used, clippy::unwrap_used)]

//! Shared helpers for groundSpring validation binaries.
//!
//! Provides typed accessors for benchmark JSON fields and a standard
//! provenance header printer. Each validation binary loads its benchmark
//! via `include_str!` and parses it with `serde_json`; these helpers
//! eliminate the repeated boilerplate across binaries.

pub mod accessors;
pub mod provenance;
pub mod tolerances;
pub use accessors::*;
pub use provenance::*;
pub use tolerances::*;

use serde_json::Value;
use std::fmt;

/// Zero-panic exit trait for validation binaries.
///
/// Replaces the `let Ok(v) = expr else { eprintln!("FATAL: ..."); return 1; }`
/// boilerplate in every validation binary with a clean `.or_exit(msg)` call.
///
/// Pattern source: wetSpring V123 / healthSpring V31 `OrExit<T>`.
pub trait OrExit<T> {
    /// Unwrap the value or print `msg` to stderr and exit with code 1.
    fn or_exit(self, msg: &str) -> T;
}

impl<T, E: fmt::Display> OrExit<T> for Result<T, E> {
    fn or_exit(self, msg: &str) -> T {
        match self {
            Ok(v) => v,
            Err(e) => {
                eprintln!("FATAL: {msg}: {e}");
                std::process::exit(exit_code::GENERAL_ERROR);
            }
        }
    }
}

impl<T> OrExit<T> for Option<T> {
    fn or_exit(self, msg: &str) -> T {
        self.unwrap_or_else(|| {
            eprintln!("FATAL: {msg}");
            std::process::exit(exit_code::GENERAL_ERROR);
        })
    }
}

/// Standardized exit codes per `UNIBIN_ARCHITECTURE_STANDARD`.
///
/// Pattern source: sweetGrass v0.7.19 `exit_code` module.
pub mod exit_code {
    /// Successful execution.
    pub const SUCCESS: i32 = 0;
    /// General runtime failure.
    pub const GENERAL_ERROR: i32 = 1;
    /// Configuration or benchmark parsing error.
    pub const CONFIG_ERROR: i32 = 78;
    /// Network or IPC error.
    pub const NETWORK_ERROR: i32 = 76;
}

/// Parse a benchmark JSON string, exiting on failure.
///
/// Replaces the repeated `let Ok(bench) = serde_json::from_str::<Value>(s)
/// else { eprintln!("FATAL: ..."); return 1; }` pattern in every validation binary.
#[must_use]
pub fn parse_benchmark(json_str: &str) -> Value {
    serde_json::from_str::<Value>(json_str).or_exit("invalid benchmark JSON")
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn print_provenance_header_succeeds() {
        let bench = json!({
            "_source": "Test experiment",
            "_provenance": {
                "baseline_commit": "abc1234",
                "baseline_date": "2026-02-27",
                "validation_script": "control/test/test.py",
                "command": "python3 control/test/test.py"
            }
        });
        try_print_provenance_header(&bench, "Test Title").unwrap();
    }

    #[test]
    fn provenance_requires_script_and_command() {
        let bench = json!({
            "_source": "Test experiment",
            "_provenance": {
                "baseline_commit": "abc1234",
                "baseline_date": "2026-02-27"
            }
        });
        assert!(try_print_provenance_header(&bench, "No Script").is_err());
    }

    #[test]
    fn try_print_provenance_header_err_on_missing_source() {
        let bench = json!({"_source": null, "_provenance": {}});
        assert!(try_print_provenance_header(&bench, "Fallback").is_err());
    }

    #[test]
    fn benchmark_json_round_trip() {
        let original = include_str!("../../../control/sensor_noise/benchmark_sensor_noise.json");
        let parsed: serde_json::Value = serde_json::from_str(original).unwrap();
        let serialized = serde_json::to_string_pretty(&parsed).unwrap();
        let reparsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(
            parsed, reparsed,
            "benchmark JSON round-trip must be lossless"
        );

        assert!(
            parsed.get("_source").is_some(),
            "benchmark must have _source"
        );
        assert!(
            parsed.get("_provenance").is_some(),
            "benchmark must have _provenance"
        );
        let prov = &parsed["_provenance"];
        assert!(
            prov.get("baseline_commit").is_some(),
            "provenance must have baseline_commit"
        );
        assert!(
            prov.get("baseline_date").is_some(),
            "provenance must have baseline_date"
        );
        assert!(
            prov.get("validation_script")
                .or_else(|| parsed.get("validation_script"))
                .is_some(),
            "provenance must have validation_script"
        );
        assert!(
            prov.get("command")
                .or_else(|| parsed.get("command"))
                .is_some(),
            "provenance must have command"
        );
    }

    /// Provenance registry completeness test (neuralSpring V120 pattern).
    ///
    /// Verifies all 29 benchmark JSONs are present and parseable at compile
    /// time. If a benchmark is added or removed, update `EXPECTED_BENCHMARKS`.
    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "registry test is necessarily long — one entry per benchmark JSON"
    )]
    fn provenance_registry_completeness() {
        const EXPECTED_BENCHMARKS: usize = 29;

        let benchmarks: &[(&str, &str)] = &[
            (
                "sensor_noise",
                include_str!("../../../control/sensor_noise/benchmark_sensor_noise.json"),
            ),
            (
                "observation_gap",
                include_str!("../../../control/observation_gap/benchmark_observation_gap.json"),
            ),
            (
                "error_propagation",
                include_str!("../../../control/error_propagation/benchmark_error_propagation.json"),
            ),
            (
                "sequencing_noise",
                include_str!("../../../control/sequencing_noise/benchmark_sequencing_noise.json"),
            ),
            (
                "seismic",
                include_str!("../../../control/seismic/benchmark_seismic.json"),
            ),
            (
                "signal_specificity",
                include_str!(
                    "../../../control/signal_specificity/benchmark_signal_specificity.json"
                ),
            ),
            (
                "rawr_resampling",
                include_str!("../../../control/rawr_resampling/benchmark_rawr_resampling.json"),
            ),
            (
                "anderson_localization",
                include_str!(
                    "../../../control/anderson_localization/benchmark_anderson_localization.json"
                ),
            ),
            (
                "quasiperiodic",
                include_str!("../../../control/quasiperiodic/benchmark_quasiperiodic.json"),
            ),
            (
                "bistable",
                include_str!("../../../control/bistable_switching/benchmark_bistable.json"),
            ),
            (
                "multisignal",
                include_str!("../../../control/multisignal_qs/benchmark_multisignal.json"),
            ),
            (
                "spin_transport",
                include_str!("../../../control/spin_transport/benchmark_spin_transport.json"),
            ),
            (
                "resampling_convergence",
                include_str!(
                    "../../../control/resampling_convergence/benchmark_resampling_convergence.json"
                ),
            ),
            (
                "drift_selection",
                include_str!("../../../control/drift_selection/benchmark_drift_selection.json"),
            ),
            (
                "uncertainty_bridge",
                include_str!(
                    "../../../control/uncertainty_bridge/benchmark_uncertainty_bridge.json"
                ),
            ),
            (
                "rare_biosphere",
                include_str!("../../../control/rare_biosphere/benchmark_rare_biosphere.json"),
            ),
            (
                "quasispecies",
                include_str!("../../../control/quasispecies_threshold/benchmark_quasispecies.json"),
            ),
            (
                "band_edge",
                include_str!("../../../control/band_edge/benchmark_band_edge.json"),
            ),
            (
                "jackknife",
                include_str!("../../../control/jackknife_estimation/benchmark_jackknife.json"),
            ),
            (
                "freeze_out",
                include_str!("../../../control/freeze_out_inverse/benchmark_freeze_out.json"),
            ),
            (
                "spectral_recon",
                include_str!("../../../control/spectral_recon/benchmark_spectral_recon.json"),
            ),
            (
                "et0_anderson",
                include_str!(
                    "../../../control/et0_anderson_propagation/benchmark_et0_anderson.json"
                ),
            ),
            (
                "notill_sampling",
                include_str!("../../../control/notill_sampling/benchmark_notill_sampling.json"),
            ),
            (
                "aggregate_stability",
                include_str!(
                    "../../../control/aggregate_stability/benchmark_aggregate_stability.json"
                ),
            ),
            (
                "precision_drift",
                include_str!("../../../control/precision_drift/benchmark_precision_drift.json"),
            ),
            (
                "size_convergence",
                include_str!("../../../control/size_convergence/benchmark_size_convergence.json"),
            ),
            (
                "vendor_parity",
                include_str!("../../../control/vendor_parity/benchmark_vendor_parity.json"),
            ),
            (
                "npu_anderson",
                include_str!("../../../control/npu_anderson/benchmark_npu_anderson.json"),
            ),
            (
                "et0_methods",
                include_str!("../../../control/et0_methods/benchmark_et0_methods.json"),
            ),
        ];

        assert_eq!(
            benchmarks.len(),
            EXPECTED_BENCHMARKS,
            "benchmark count mismatch: got {}, expected {EXPECTED_BENCHMARKS}",
            benchmarks.len()
        );

        for (name, json_str) in benchmarks {
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(json_str);
            assert!(
                parsed.is_ok(),
                "benchmark '{name}' failed to parse: {}",
                parsed.unwrap_err()
            );
        }
    }
}
