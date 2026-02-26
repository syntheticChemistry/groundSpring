#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
#
# groundSpring — Three-Mode Benchmark
#
# Builds and times all 14 validation binaries in three feature modes:
#   1. default   (no barracuda)
#   2. barracuda (CPU delegations only)
#   3. barracuda-gpu (CPU + GPU delegations)
#
# Usage:
#   bash scripts/three_mode_benchmark.sh

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
)

MODES=("default" "barracuda" "barracuda-gpu")
FEATURES=("" "--features barracuda" "--features barracuda-gpu")

declare -A TIMES
declare -A CHECKS

echo "================================================================"
echo "  groundSpring — Three-Mode Benchmark"
echo "  Date: $(date -Iseconds)"
echo "  ToadStool HEAD: $(cd ../phase1/toadstool && git rev-parse --short HEAD 2>/dev/null || echo 'N/A')"
echo "  groundSpring HEAD: $(git rev-parse --short HEAD)"
echo "================================================================"
echo ""

for m in 0 1 2; do
    mode="${MODES[$m]}"
    feat="${FEATURES[$m]}"
    echo "╔══════════════════════════════════════════════════════════════╗"
    printf "║  Mode: %-52s ║\n" "$mode"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo ""

    echo "  Building..."
    eval "cargo build --release --workspace $feat" 2>&1 | tail -1
    echo ""

    total_ms=0
    total_checks=0

    for bin in "${BINS[@]}"; do
        printf "  %-36s " "$bin"

        start_ns=$(date +%s%N)
        output=$(eval "cargo run --release --bin $bin $feat" 2>&1)
        end_ns=$(date +%s%N)

        elapsed_ms=$(( (end_ns - start_ns) / 1000000 ))
        total_ms=$((total_ms + elapsed_ms))

        pass=$(echo "$output" | grep -oP 'TOTAL: \K[0-9]+(?=/[0-9]+ PASS)' || echo "?")
        total_bin=$(echo "$output" | grep -oP 'TOTAL: [0-9]+/\K[0-9]+' || echo "?")
        total_checks=$((total_checks + total_bin))

        TIMES["${mode}_${bin}"]=$elapsed_ms
        CHECKS["${mode}_${bin}"]="${pass}/${total_bin}"

        printf "%6d ms  %s\n" "$elapsed_ms" "${pass}/${total_bin}"
    done

    TIMES["${mode}_TOTAL"]=$total_ms
    CHECKS["${mode}_TOTAL"]="${total_checks}/${total_checks}"
    echo ""
    printf "  %-36s %6d ms  %d checks\n" "TOTAL" "$total_ms" "$total_checks"
    echo ""
done

echo ""
echo "================================================================"
echo "  COMPARISON TABLE"
echo "================================================================"
echo ""
printf "  %-28s %10s %10s %10s  %s\n" "Binary" "Default" "Barracuda" "Barra-GPU" "Speedup"
printf "  %-28s %10s %10s %10s  %s\n" "---" "---" "---" "---" "---"

for bin in "${BINS[@]}"; do
    t_def=${TIMES["default_${bin}"]}
    t_bar=${TIMES["barracuda_${bin}"]}
    t_gpu=${TIMES["barracuda-gpu_${bin}"]}
    checks=${CHECKS["default_${bin}"]}

    if [ "$t_gpu" -gt 0 ] && [ "$t_def" -gt 0 ]; then
        speedup=$(echo "scale=1; $t_def / $t_gpu" | bc)
    else
        speedup="N/A"
    fi

    printf "  %-28s %8d ms %8d ms %8d ms  %sx %s\n" "$bin" "$t_def" "$t_bar" "$t_gpu" "$speedup" "$checks"
done

t_def_total=${TIMES["default_TOTAL"]}
t_bar_total=${TIMES["barracuda_TOTAL"]}
t_gpu_total=${TIMES["barracuda-gpu_TOTAL"]}
speedup_total=$(echo "scale=1; $t_def_total / $t_gpu_total" | bc)

echo ""
printf "  %-28s %8d ms %8d ms %8d ms  %sx\n" "TOTAL" "$t_def_total" "$t_bar_total" "$t_gpu_total" "$speedup_total"
echo ""
echo "================================================================"
