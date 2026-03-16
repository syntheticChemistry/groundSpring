#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-only
# Copyright (C) 2026 ecoPrimals / Squirrel Team
#
# groundSpring — Three-Tier Parity Certificate
#
# Proves mathematical parity across three tiers:
#   Tier 1: default (pure Rust, no barracuda)
#   Tier 2: barracuda-CPU (Rust + barracuda CPU delegations)
#   Tier 3: barracuda-GPU (Rust + barracuda CPU + GPU delegations)
#
# Each tier runs the same 29 validation binaries and records pass/total.
# Parity = all three tiers produce identical pass counts for every experiment.
#
# Usage:
#   bash scripts/three_tier_parity_report.sh
#
# Output:
#   data/three_tier_parity_report.json — machine-readable certificate
#   Markdown summary to stdout

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(dirname "$SCRIPT_DIR")"
cd "$ROOT"

BINS=(
    validate-decompose
    validate-rarefaction
    validate-seismic
    validate-weather
    validate-fao56
    validate-signal-specificity
    validate-rawr
    validate-anderson
    validate-quasiperiodic
    validate-bistable
    validate-multisignal
    validate-transport
    validate-resampling-conv
    validate-drift
    validate-uncertainty-bridge
    validate-rare-biosphere
    validate-quasispecies
    validate-band-edge
    validate-jackknife
    validate-freeze-out
    validate-spectral-recon
    validate-et0-anderson
    validate-notill-sampling
    validate-aggregate-stability
    validate-precision-drift
    validate-size-convergence
    validate-vendor-parity
    validate-et0-methods
    validate-tissue-anderson
)

EXP_NAMES=(
    "Exp 001: Sensor Noise"
    "Exp 004: Sequencing Noise"
    "Exp 005: Seismic Inversion"
    "Exp 002: Observation Gap"
    "Exp 003: FAO-56 Error Propagation"
    "Exp 006: Signal Specificity"
    "Exp 007: RAWR Resampling"
    "Exp 008: Anderson Localization"
    "Exp 009: Quasiperiodic Localization"
    "Exp 010: Bistable Switching"
    "Exp 011: Multi-Signal QS"
    "Exp 012: Spin Chain Transport"
    "Exp 013: Resampling Convergence"
    "Exp 014: Drift vs Selection"
    "Exp 015: Uncertainty Bridge"
    "Exp 016: Rare Biosphere"
    "Exp 017: Quasispecies Threshold"
    "Exp 018: Band Edge Structure"
    "Exp 019: Jackknife Estimation"
    "Exp 020: Freeze-Out Inverse"
    "Exp 021: Spectral Reconstruction"
    "Exp 022: ET0-Anderson Propagation"
    "Exp 023: No-Till Sampling"
    "Exp 024: Aggregate Stability"
    "Exp 025: Precision Drift"
    "Exp 026: Size Convergence"
    "Exp 027: Vendor Parity"
    "Exp 035: ET0 Methods"
    "Exp 033: Tissue Anderson"
)

MODES=("default" "barracuda" "barracuda-gpu")
FEATURES=("" "--features barracuda" "--features barracuda-gpu")

declare -A CHECKS
declare -A TIMES

DATE=$(date -Iseconds)
BARRACUDA_HEAD=$(cd ../barraCuda && git rev-parse --short HEAD 2>/dev/null || echo 'N/A')
GS_HEAD=$(git rev-parse --short HEAD)

echo "========================================================================"
echo "  groundSpring — Three-Tier Parity Certificate"
echo "  Date: $DATE"
echo "  barraCuda HEAD: $BARRACUDA_HEAD"
echo "  groundSpring HEAD: $GS_HEAD"
echo "========================================================================"
echo ""

for m in 0 1 2; do
    mode="${MODES[$m]}"
    feat="${FEATURES[$m]}"
    echo "── Tier $((m+1)): $mode ──"
    echo ""
    echo "  Building..."
    eval "cargo build --release --workspace $feat" 2>&1 | tail -1
    echo ""

    for i in "${!BINS[@]}"; do
        bin="${BINS[$i]}"
        printf "  %-36s " "$bin"

        start_ns=$(date +%s%N)
        output=$(eval "cargo run --release --bin $bin $feat" 2>&1)
        end_ns=$(date +%s%N)

        elapsed_ms=$(( (end_ns - start_ns) / 1000000 ))
        pass=$(echo "$output" | grep -oP 'TOTAL: \K[0-9]+(?=/[0-9]+ PASS)' || echo "0")
        total=$(echo "$output" | grep -oP 'TOTAL: [0-9]+/\K[0-9]+' || echo "0")

        CHECKS["${mode}_${i}"]="${pass}/${total}"
        TIMES["${mode}_${i}"]=$elapsed_ms

        printf "%s  %6d ms\n" "${pass}/${total}" "$elapsed_ms"
    done
    echo ""
done

echo ""
echo "========================================================================"
echo "  PARITY MATRIX"
echo "========================================================================"
echo ""
printf "  %-36s %10s %10s %12s  %s\n" "Experiment" "Default" "Barra-CPU" "Barra-GPU" "Parity"
printf "  %-36s %10s %10s %12s  %s\n" "---" "---" "---" "---" "---"

all_parity=true
json_entries=""
for i in "${!BINS[@]}"; do
    name="${EXP_NAMES[$i]}"
    c_def="${CHECKS[default_${i}]}"
    c_bar="${CHECKS[barracuda_${i}]}"
    c_gpu="${CHECKS[barracuda-gpu_${i}]}"

    t_def="${TIMES[default_${i}]}"
    t_bar="${TIMES[barracuda_${i}]}"
    t_gpu="${TIMES[barracuda-gpu_${i}]}"

    if [ "$c_def" = "$c_bar" ] && [ "$c_bar" = "$c_gpu" ]; then
        parity="PROVEN"
    else
        parity="MISMATCH"
        all_parity=false
    fi

    printf "  %-36s %10s %10s %12s  %s\n" "$name" "$c_def" "$c_bar" "$c_gpu" "$parity"

    json_entries="${json_entries}{\"name\":\"$name\",\"binary\":\"${BINS[$i]}\",\"default\":{\"checks\":\"$c_def\",\"ms\":$t_def},\"barracuda\":{\"checks\":\"$c_bar\",\"ms\":$t_bar},\"barracuda_gpu\":{\"checks\":\"$c_gpu\",\"ms\":$t_gpu},\"parity\":\"$parity\"},"
done

echo ""
if $all_parity; then
    echo "  ALL 29 EXPERIMENTS: THREE-TIER PARITY PROVEN"
    echo ""
    echo "  Pure Rust math (default) = barracuda CPU = barracuda GPU"
    echo "  barraCuda + toadStool S129 universal precision architecture verified."
else
    echo "  WARNING: PARITY MISMATCH DETECTED"
fi
echo ""
echo "========================================================================"

# JSON certificate
json_entries="${json_entries%,}"
cat > "$ROOT/data/three_tier_parity_report.json" << JSONEOF
{
  "title": "groundSpring Three-Tier Parity Certificate",
  "date": "$DATE",
  "groundspring_head": "$GS_HEAD",
  "barracuda_head": "$BARRACUDA_HEAD",
  "all_parity": $all_parity,
  "tiers": ["default", "barracuda", "barracuda-gpu"],
  "experiments": [$json_entries]
}
JSONEOF

echo ""
echo "  Certificate saved to data/three_tier_parity_report.json"

if $all_parity; then
    exit 0
else
    exit 1
fi
