#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 ecoPrimals / Squirrel Team
#
# groundSpring — Run All Validation (Python Phase 0 + Rust Phase 1)
#
# Usage:
#   bash scripts/run_all_baselines.sh
#
# Writes results to data/validation_log_<timestamp>.txt

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(dirname "$SCRIPT_DIR")"

cd "$ROOT"

TIMESTAMP="$(date -Iseconds)"
LOG_DIR="$ROOT/data"
mkdir -p "$LOG_DIR"
LOG_FILE="$LOG_DIR/validation_log_$(date +%Y%m%d_%H%M%S).txt"

EXPERIMENTS=()
RESULTS=()

run_experiment() {
    local name="$1"
    local cmd="$2"

    echo ""
    echo "================================================================"
    echo "  Running: $name"
    echo "================================================================"

    if eval "$cmd"; then
        RESULTS+=("PASS")
        echo ""
        echo "  >>> $name: PASS"
    else
        RESULTS+=("FAIL")
        echo ""
        echo "  >>> $name: FAIL"
    fi
    EXPERIMENTS+=("$name")
}

{
echo "================================================================"
echo "  groundSpring — Full Validation Suite"
echo "  Date: $TIMESTAMP"
echo "================================================================"

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  PHASE 0: Python Baselines                                 ║"
echo "╚══════════════════════════════════════════════════════════════╝"

run_experiment \
    "Exp 001: Sensor Noise (Python)" \
    "python3 control/sensor_noise/sensor_noise_decomposition.py"

run_experiment \
    "Exp 002: Observation Gap (Python)" \
    "python3 control/observation_gap/observation_gap.py"

run_experiment \
    "Exp 003: Error Propagation (Python)" \
    "python3 control/error_propagation/error_propagation_fao56.py"

run_experiment \
    "Exp 004: Sequencing Noise (Python)" \
    "python3 control/sequencing_noise/sequencing_noise.py"

run_experiment \
    "Exp 005: Seismic Inversion (Python)" \
    "python3 control/seismic/seismic_inversion.py"

run_experiment \
    "Exp 006: Signal Specificity (Python)" \
    "python3 control/signal_specificity/signal_specificity.py"

run_experiment \
    "Exp 007: RAWR Resampling (Python)" \
    "python3 control/rawr_resampling/rawr_resampling.py"

run_experiment \
    "Exp 008: Anderson Localization (Python)" \
    "python3 control/anderson_localization/anderson_localization.py"

run_experiment \
    "Exp 009: Quasiperiodic Localization (Python)" \
    "python3 control/quasiperiodic/quasiperiodic_localization.py"

run_experiment \
    "Exp 010: Bistable Switching (Python)" \
    "python3 control/bistable_switching/bistable_switching.py"

run_experiment \
    "Exp 011: Multi-Signal QS (Python)" \
    "python3 control/multisignal_qs/multisignal_qs.py"

run_experiment \
    "Exp 012: Spin Chain Transport (Python)" \
    "python3 control/spin_transport/spin_chain_transport.py"

run_experiment \
    "Exp 013: Resampling Convergence (Python)" \
    "python3 control/resampling_convergence/resampling_convergence.py"

run_experiment \
    "Exp 014: Drift vs Selection (Python)" \
    "python3 control/drift_selection/drift_selection.py"

run_experiment \
    "Exp 015: Uncertainty Bridge (Python)" \
    "python3 control/uncertainty_bridge/uncertainty_bridge.py"

run_experiment \
    "Exp 016: Rare Biosphere (Python)" \
    "python3 control/rare_biosphere/rare_biosphere.py"

run_experiment \
    "Exp 017: Quasispecies Threshold (Python)" \
    "python3 control/quasispecies_threshold/quasispecies_threshold.py"

run_experiment \
    "Exp 018: Band Edge (Python)" \
    "python3 control/band_edge/band_edge.py"

run_experiment \
    "Exp 019: Jackknife Estimation (Python)" \
    "python3 control/jackknife_estimation/jackknife_estimation.py"

run_experiment \
    "Exp 020: Freeze Out Inverse (Python)" \
    "python3 control/freeze_out_inverse/freeze_out_inverse.py"

run_experiment \
    "Exp 021: Spectral Recon (Python)" \
    "python3 control/spectral_recon/spectral_recon.py"

run_experiment \
    "Exp 022: ET0 Anderson Propagation (Python)" \
    "python3 control/et0_anderson_propagation/et0_anderson_propagation.py"

run_experiment \
    "Exp 023: No-Till Sampling (Python)" \
    "python3 control/notill_sampling/notill_sampling.py"

run_experiment \
    "Exp 024: Aggregate Stability (Python)" \
    "python3 control/aggregate_stability/aggregate_stability.py"

run_experiment \
    "Exp 025: Precision Drift (Python)" \
    "python3 control/precision_drift/precision_drift.py"

run_experiment \
    "Exp 026: Size Convergence (Python)" \
    "python3 control/size_convergence/size_convergence.py"

run_experiment \
    "Exp 027: Vendor Parity (Python)" \
    "python3 control/vendor_parity/vendor_parity.py"

run_experiment \
    "Exp 028: NPU Anderson (Python)" \
    "python3 control/npu_anderson/npu_anderson.py"

run_experiment \
    "Exp 035: ET0 Methods (Python)" \
    "python3 control/et0_methods/et0_methods.py"

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  PHASE 1: Rust Validation Binaries                         ║"
echo "╚══════════════════════════════════════════════════════════════╝"

if command -v cargo &> /dev/null; then
    cargo build --release --workspace 2>&1 | tail -3

    run_experiment \
        "Rust: Bias-Variance Decomposition" \
        "cargo run --release --bin validate-decompose"

    run_experiment \
        "Rust: Rarefaction" \
        "cargo run --release --bin validate-rarefaction"

    run_experiment \
        "Rust: Seismic Inversion" \
        "cargo run --release --bin validate-seismic"

    run_experiment \
        "Rust: Weather Model-Observation Gap" \
        "cargo run --release --bin validate-weather"

    run_experiment \
        "Rust: FAO-56 Error Propagation" \
        "cargo run --release --bin validate-fao56"

    run_experiment \
        "Rust: Signal Specificity" \
        "cargo run --release --bin validate-signal-specificity"

    run_experiment \
        "Rust: RAWR Resampling" \
        "cargo run --release --bin validate-rawr"

    run_experiment \
        "Rust: Anderson Localization" \
        "cargo run --release --bin validate-anderson"

    run_experiment \
        "Rust: Quasiperiodic Localization" \
        "cargo run --release --bin validate-quasiperiodic"

    run_experiment \
        "Rust: Bistable Switching" \
        "cargo run --release --bin validate-bistable"

    run_experiment \
        "Rust: Multi-Signal QS" \
        "cargo run --release --bin validate-multisignal"

    run_experiment \
        "Rust: Spin Chain Transport" \
        "cargo run --release --bin validate-transport"

    run_experiment \
        "Rust: Resampling Convergence" \
        "cargo run --release --bin validate-resampling-conv"

    run_experiment \
        "Rust: Drift vs Selection" \
        "cargo run --release --bin validate-drift"

    run_experiment \
        "Rust: Uncertainty Bridge" \
        "cargo run --release --bin validate-uncertainty-bridge"

    run_experiment \
        "Rust: Rare Biosphere" \
        "cargo run --release --bin validate-rare-biosphere"

    run_experiment \
        "Rust: Quasispecies Threshold" \
        "cargo run --release --bin validate-quasispecies"

    run_experiment \
        "Rust: Band Edge" \
        "cargo run --release --bin validate-band-edge"

    run_experiment \
        "Rust: Jackknife Estimation" \
        "cargo run --release --bin validate-jackknife"

    run_experiment \
        "Rust: Freeze Out Inverse" \
        "cargo run --release --bin validate-freeze-out"

    run_experiment \
        "Rust: Spectral Recon" \
        "cargo run --release --bin validate-spectral-recon"

    run_experiment \
        "Rust: ET0 Anderson Propagation" \
        "cargo run --release --bin validate-et0-anderson"

    run_experiment \
        "Rust: No-Till Sampling" \
        "cargo run --release --bin validate-notill-sampling"

    run_experiment \
        "Rust: Aggregate Stability" \
        "cargo run --release --bin validate-aggregate-stability"

    run_experiment \
        "Rust: Precision Drift" \
        "cargo run --release --bin validate-precision-drift"

    run_experiment \
        "Rust: Size Convergence" \
        "cargo run --release --bin validate-size-convergence"

    run_experiment \
        "Rust: Vendor Parity" \
        "cargo run --release --bin validate-vendor-parity"

    run_experiment \
        "Rust: ET0 Methods" \
        "cargo run --release --bin validate-et0-methods"

    run_experiment \
        "Rust: Tissue Anderson" \
        "cargo run --release --bin validate-tissue-anderson"

    if [[ -e /dev/akida0 ]]; then
        cargo build --release --workspace --features npu 2>&1 | tail -1
        run_experiment \
            "Rust: NPU Anderson (--features npu)" \
            "cargo run --release --bin validate-npu-anderson --features npu"
    else
        echo ""
        echo "  [SKIP] Rust: NPU Anderson — /dev/akida0 not present"
    fi
else
    echo "  [SKIP] cargo not found — Rust validation skipped"
fi

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  PYTEST: Unit + Determinism Tests                          ║"
echo "╚══════════════════════════════════════════════════════════════╝"

if command -v python3 &> /dev/null; then
    python3 -m pytest tests/ -v 2>&1 || echo "  [WARN] Some pytest tests failed"
fi

# Summary
echo ""
echo "================================================================"
echo "  GRAND SUMMARY"
echo "================================================================"

N_PASS=0
N_FAIL=0
for i in "${!EXPERIMENTS[@]}"; do
    status="${RESULTS[$i]}"
    echo "  ${EXPERIMENTS[$i]}: $status"
    if [ "$status" = "PASS" ]; then
        ((N_PASS++)) || true
    else
        ((N_FAIL++)) || true
    fi
done

echo ""
echo "  Total: $N_PASS PASS, $N_FAIL FAIL out of ${#EXPERIMENTS[@]} experiments"
echo "  Log:   $LOG_FILE"
echo "================================================================"

if [ "$N_FAIL" -gt 0 ]; then
    exit 1
fi

} 2>&1 | tee "$LOG_FILE"
