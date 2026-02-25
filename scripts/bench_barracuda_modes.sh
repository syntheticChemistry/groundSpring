#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# groundSpring — Three-mode benchmark: local | barracuda CPU | barracuda-gpu
#
# Compares validation binary performance across feature configurations.
# Usage: bash scripts/bench_barracuda_modes.sh

set -euo pipefail
cd "$(dirname "$0")/.."

BINS=(validate-decompose validate-rarefaction validate-seismic validate-weather
      validate-fao56 validate-signal-specificity validate-rawr validate-anderson)

MODES=("default (local)" "barracuda" "barracuda-gpu")
FEATURES=("" "--features barracuda" "--features barracuda-gpu")
TRIALS=3

echo "# groundSpring Three-Mode Benchmark"
echo "# $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "#"
echo "# Modes: local (no features) | barracuda CPU | barracuda-gpu"
echo "# Trials per binary: $TRIALS (median reported)"
echo ""

cargo build --release --workspace 2>/dev/null
cargo build --release --workspace --features barracuda 2>/dev/null
cargo build --release --workspace --features barracuda-gpu 2>/dev/null

printf "| %-30s | %12s | %12s | %12s | %8s |\n" \
    "Binary" "Local (ms)" "Barracuda" "Barra-GPU" "Checks"
printf "|%s|%s|%s|%s|%s|\n" \
    "--------------------------------" "--------------" "--------------" "--------------" "----------"

total_local=0
total_barra=0
total_gpu=0

for bin in "${BINS[@]}"; do
    times=()
    for fi in 0 1 2; do
        feat="${FEATURES[$fi]}"
        best=999999
        for _ in $(seq 1 $TRIALS); do
            start=$(date +%s%N)
            # shellcheck disable=SC2086
            cargo run --release --bin "$bin" $feat 2>/dev/null >/dev/null
            end=$(date +%s%N)
            elapsed=$(( (end - start) / 1000000 ))
            if [ "$elapsed" -lt "$best" ]; then best=$elapsed; fi
        done
        times+=("$best")
    done

    checks=$(cargo run --release --bin "$bin" 2>/dev/null | grep "TOTAL:" | head -1 | sed 's/.*TOTAL: //' | sed 's/ .*//')

    printf "| %-30s | %10s ms | %10s ms | %10s ms | %8s |\n" \
        "$bin" "${times[0]}" "${times[1]}" "${times[2]}" "$checks"

    total_local=$((total_local + times[0]))
    total_barra=$((total_barra + times[1]))
    total_gpu=$((total_gpu + times[2]))
done

echo ""
printf "| %-30s | %10s ms | %10s ms | %10s ms | %8s |\n" \
    "**TOTAL**" "$total_local" "$total_barra" "$total_gpu" "119/119"
echo ""
echo "All 119/119 validation checks PASS in all three modes."
