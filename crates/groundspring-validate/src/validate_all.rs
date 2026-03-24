// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals / Squirrel Team
#![forbid(unsafe_code)]

//! Meta-validator: runs all core validation binaries in sequence.
//!
//! Follows the hotSpring / neuralSpring `validate_all` convention — a single
//! entry point that executes every validation binary and reports aggregate
//! pass/fail. Hardware-dependent binaries (GPU, NPU, biomeOS) are skipped
//! gracefully when prerequisites are unavailable.

use std::process::{Command, ExitCode};
use std::time::Instant;

/// Core validation binaries that require no special hardware.
const CORE_BINARIES: &[&str] = &[
    "validate_decompose",
    "validate_rarefaction",
    "validate_seismic",
    "validate_weather",
    "validate_fao56",
    "validate_signal_specificity",
    "validate_rawr",
    "validate_anderson",
    "validate_quasiperiodic",
    "validate_bistable",
    "validate_multisignal",
    "validate_transport",
    "validate_resampling_conv",
    "validate_drift",
    "validate_uncertainty_bridge",
    "validate_rare_biosphere",
    "validate_quasispecies",
    "validate_band_edge",
    "validate_jackknife",
    "validate_freeze_out",
    "validate_spectral_recon",
    "validate_et0_anderson",
    "validate_notill_sampling",
    "validate_aggregate_stability",
    "validate_precision_drift",
    "validate_size_convergence",
    "validate_vendor_parity",
    "validate_tissue_anderson",
    "validate_et0_methods",
];

/// Hardware-dependent binaries — skipped (not failed) only when the binary
/// itself signals hardware-unavailable via exit code 2.
const OPTIONAL_BINARIES: &[&str] = &["validate_npu_anderson"];

/// Exit code that hardware-dependent validators return when prerequisites
/// (GPU, NPU, NUCLEUS) are unavailable — distinct from validation failure (1).
const EXIT_HARDWARE_UNAVAILABLE: i32 = 2;

/// Result of running a single validation binary.
enum RunResult {
    Pass,
    Fail,
    HardwareUnavailable,
}

fn run_binary(name: &str) -> RunResult {
    let start = Instant::now();
    let result = Command::new("cargo")
        .args(["run", "--release", "--bin", name])
        .status();

    let elapsed = start.elapsed();
    match result {
        Ok(status) if status.success() => {
            println!("  PASS  {name} ({elapsed:.1?})");
            RunResult::Pass
        }
        Ok(status) => {
            let code = status.code().unwrap_or(-1);
            if code == EXIT_HARDWARE_UNAVAILABLE {
                println!("  SKIP  {name} (hardware unavailable, exit {code}, {elapsed:.1?})");
                RunResult::HardwareUnavailable
            } else {
                eprintln!("  FAIL  {name} (exit {code}, {elapsed:.1?})");
                RunResult::Fail
            }
        }
        Err(e) => {
            eprintln!("  ERROR {name}: {e}");
            RunResult::Fail
        }
    }
}

fn main() -> ExitCode {
    println!(
        "groundSpring validate-all — running {} core + {} optional binaries\n",
        CORE_BINARIES.len(),
        OPTIONAL_BINARIES.len()
    );

    let total_start = Instant::now();
    let mut passed = 0u32;
    let mut failed = 0u32;
    let mut skipped = 0u32;

    println!("=== Core Validation ===");
    for &name in CORE_BINARIES {
        match run_binary(name) {
            RunResult::Pass => passed += 1,
            RunResult::Fail | RunResult::HardwareUnavailable => failed += 1,
        }
    }

    println!("\n=== Optional (hardware-dependent) ===");
    for &name in OPTIONAL_BINARIES {
        match run_binary(name) {
            RunResult::Pass => passed += 1,
            RunResult::HardwareUnavailable => skipped += 1,
            RunResult::Fail => {
                eprintln!("  NOTE  {name} failed for non-hardware reasons — counting as FAIL");
                failed += 1;
            }
        }
    }

    let elapsed = total_start.elapsed();
    println!("\n=== Summary ===");
    println!("  passed:  {passed}");
    println!("  failed:  {failed}");
    println!("  skipped: {skipped}");
    println!("  elapsed: {elapsed:.1?}");

    if failed == 0 {
        println!("\nAll validations PASSED.");
        ExitCode::SUCCESS
    } else {
        eprintln!("\n{failed} validation(s) FAILED.");
        ExitCode::FAILURE
    }
}
