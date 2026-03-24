// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team

//! Integration tests that spawn validation binaries and assert exit codes.
//!
//! Each binary follows the hotSpring pattern: exit 0 = all checks pass,
//! exit 1 = any check failed. This catches regressions that unit tests
//! miss — for example, a tolerance constant change that makes a binary
//! silently fail.

use std::process::Command;

fn run_validator(name: &str) -> i32 {
    let output = Command::new("cargo")
        .args(["run", "--release", "--bin", name])
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn {name}: {e}"));
    output.status.code().unwrap_or(-1)
}

macro_rules! validator_test {
    ($test_name:ident, $bin_name:literal) => {
        #[test]
        fn $test_name() {
            let code = run_validator($bin_name);
            assert_eq!(
                code, 0,
                "{} exited with code {code} (expected 0 = all pass)",
                $bin_name
            );
        }
    };
}

validator_test!(validate_decompose_passes, "validate_decompose");
validator_test!(validate_rarefaction_passes, "validate_rarefaction");
validator_test!(validate_seismic_passes, "validate_seismic");
validator_test!(validate_weather_passes, "validate_weather");
validator_test!(validate_fao56_passes, "validate_fao56");
validator_test!(
    validate_signal_specificity_passes,
    "validate_signal_specificity"
);
validator_test!(validate_rawr_passes, "validate_rawr");
validator_test!(validate_anderson_passes, "validate_anderson");
validator_test!(validate_quasiperiodic_passes, "validate_quasiperiodic");
validator_test!(validate_bistable_passes, "validate_bistable");
validator_test!(validate_multisignal_passes, "validate_multisignal");
validator_test!(validate_transport_passes, "validate_transport");
validator_test!(validate_resampling_conv_passes, "validate_resampling_conv");
validator_test!(validate_drift_passes, "validate_drift");
validator_test!(
    validate_uncertainty_bridge_passes,
    "validate_uncertainty_bridge"
);
validator_test!(validate_rare_biosphere_passes, "validate_rare_biosphere");
validator_test!(validate_quasispecies_passes, "validate_quasispecies");
validator_test!(validate_band_edge_passes, "validate_band_edge");
validator_test!(validate_jackknife_passes, "validate_jackknife");
validator_test!(validate_freeze_out_passes, "validate_freeze_out");
validator_test!(validate_spectral_recon_passes, "validate_spectral_recon");
validator_test!(validate_et0_anderson_passes, "validate_et0_anderson");
validator_test!(validate_notill_sampling_passes, "validate_notill_sampling");
validator_test!(
    validate_aggregate_stability_passes,
    "validate_aggregate_stability"
);
validator_test!(validate_precision_drift_passes, "validate_precision_drift");
validator_test!(
    validate_size_convergence_passes,
    "validate_size_convergence"
);
validator_test!(validate_vendor_parity_passes, "validate_vendor_parity");
validator_test!(validate_tissue_anderson_passes, "validate_tissue_anderson");
validator_test!(validate_et0_methods_passes, "validate_et0_methods");
